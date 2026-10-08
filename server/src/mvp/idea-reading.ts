// Reading ideas (mission 2, M2-5, makscee/void-board#792). A job picks
// `written` ideas a few at a time and moves each through the reader
// (./idea-reader.ts):
//   written → reading → pick-archetype   (3 archetypes, checked)
//   its author picks one (POST /ideas/:id/archetype)
//          → reading → pick-reading      (3 readings for it, checked)
//   its author picks one (POST /ideas/:id/reading)
//          → simulating                  (the Row is a `candidate` in mvp_units)
// "None of these" (POST /ideas/:id/none, M2-6) reads a stage once more for
// other options; a second one at the same stage refunds the idea.
// An option that fails ./idea-checks.ts is retried once with its problems,
// then dropped; fewer than 3 shows what there is; none fails the idea with
// the reason and refunds it. A part the game can't say goes to the
// new-words log. A reader error or timeout leaves the idea `reading` and
// retries later: nothing waits on the model.
import type { Idea, IdeaArchetype, MyIdea } from "../../../src/mvp/contract.js";
import { slug, type Row } from "../../../src/mvp/units.js";
import { archetypeDraftProblems, checkReading, rowOf, takenShapes, type ArchetypeDraft } from "./idea-checks.js";
import { ideaReaderFromEnv, type CantExpress, type IdeaReader } from "./idea-reader.js";
import { IdeaRefused, refundIdea } from "./ideas.js";
import type { RunDeps } from "./runs.js";
import type { MvpJob } from "./runtime.js";
import type { MvpStore } from "./store.js";

type ReadDeps = Pick<RunDeps, "store" | "rules" | "now">;

/** Ideas read per wake. */
export const READ_BATCH = 3;
/** Options per stage. */
export const OPTIONS = 3;
export const COULD_NOT = "We couldn't make this idea into a unit yet.";

/** The rows a new unit's shape must differ from: live units and candidates. */
function shapeRows(store: MvpStore): Row[] {
  return store.units().filter((u) => u.status === "live" || u.status === "candidate").map((u) => u.row);
}

/** Wait before try `tries + 1` after a reader error: 1, 2, 4 … 30 minutes. */
export function retryDelayMs(tries: number): number {
  return Math.min(30, 2 ** Math.max(0, tries - 1)) * 60_000;
}

/** The ideas due for the reader, oldest first: `written` ones, and `reading`
 * ones not waiting out a retry (a restart mid-read reads them again). */
export function dueIdeas(deps: ReadDeps, batch = READ_BATCH): Idea[] {
  const now = deps.now().getTime();
  const reading = deps.store.ideas({ state: "reading" }).filter((i) => !i.data.retryAt || Date.parse(i.data.retryAt) <= now);
  return [...reading, ...deps.store.ideas({ state: "written" })].sort((a, b) => a.createdAt.localeCompare(b.createdAt)).slice(0, batch);
}

/** Reads up to `batch` due ideas, one after another. Returns how many it read. */
export async function readIdeas(deps: ReadDeps, reader: IdeaReader, batch = READ_BATCH): Promise<number> {
  const due = dueIdeas(deps, batch);
  for (const idea of due) await readIdea(deps, reader, idea);
  return due.length;
}

/** One idea's next stage: archetypes, or readings once one is picked. */
export async function readIdea(deps: ReadDeps, reader: IdeaReader, idea: Idea): Promise<void> {
  const { store } = deps;
  const at: Idea = { ...idea, state: "reading" };
  store.putIdea(at);
  try {
    if (at.data.archetype) await readReadings(deps, reader, at, at.data.archetype);
    else await readArchetypes(deps, reader, at);
  } catch (err) {
    const tries = (at.data.tries ?? 0) + 1;
    const retryAt = new Date(deps.now().getTime() + retryDelayMs(tries)).toISOString();
    console.warn(`[ideas] reading ${at.ideaId} failed (try ${tries}, again at ${retryAt}): ${(err as Error).message}`);
    store.putIdea({ ...at, data: { ...at.data, tries, retryAt } });
  }
}

async function readArchetypes(deps: ReadDeps, reader: IdeaReader, idea: Idea): Promise<void> {
  const units = shapeRows(deps.store);
  const taken: Pick<Row, "name" | "archetype">[] = deps.store.units().map((u) => u.row);
  const ok: IdeaArchetype[] = [];
  const check = (drafts: ArchetypeDraft[]) => {
    const problems: string[] = [];
    for (const d of drafts) {
      const p = archetypeDraftProblems(d, taken);
      if (p.length) problems.push(...p);
      else if (ok.length < OPTIONS) {
        const a = { name: d.name.trim(), emoji: d.emoji.trim(), line: d.line.trim() };
        ok.push(a);
        taken.push({ name: a.name, archetype: a.line });
      }
    }
    return problems;
  };
  // Options its author turned down (M2-6) are taken: the reader is asked for others.
  const declined = idea.data.declined?.archetypes;
  for (const d of declined ?? []) taken.push({ name: d.name, archetype: d.line });
  const first = await reader.archetypes(idea.text, { want: OPTIONS, units, ...turnedDown(declined?.map((d) => `${d.emoji} ${d.name}: ${d.line}`)) });
  const problems = check(first.options);
  const cant = [...first.cantExpress];
  if (problems.length && ok.length < OPTIONS) {
    const again = await reader.archetypes(idea.text, { want: OPTIONS - ok.length, units, problems });
    check(again.options);
    cant.push(...again.cantExpress);
  }
  const parts = logCantExpress(deps, idea, cant);
  if (!ok.length) return fail(deps, idea, parts);
  deps.store.putIdea({ ...idea, state: "pick-archetype", data: settled({ ...idea.data, archetypes: ok }, parts) });
}

async function readReadings(deps: ReadDeps, reader: IdeaReader, idea: Idea, archetype: IdeaArchetype): Promise<void> {
  const units = shapeRows(deps.store);
  const taken = takenShapes(units);
  const ok: NonNullable<Idea["data"]["readings"]> = [];
  const check = (drafts: Parameters<typeof checkReading>[0][]) => {
    const problems: string[] = [];
    drafts.forEach((d, i) => {
      const r = checkReading(d, archetype, taken);
      if ("problems" in r) problems.push(...r.problems.map((p) => `reading ${i + 1}: ${p}`));
      else if (ok.length < OPTIONS) {
        ok.push(r.reading);
        taken.accept(r.row);
      }
    });
    return problems;
  };
  const declined = idea.data.declined?.readings;
  for (const d of declined ?? []) taken.accept(rowOf(archetype, d));
  const first = await reader.readings(idea.text, archetype, { want: OPTIONS, units, ...turnedDown(declined?.map((d) => d.text.sleeping)) });
  const problems = check(first.options);
  const cant = [...first.cantExpress];
  if (problems.length && ok.length < OPTIONS) {
    const again = await reader.readings(idea.text, archetype, { want: OPTIONS - ok.length, units, problems });
    check(again.options);
    cant.push(...again.cantExpress);
  }
  const parts = logCantExpress(deps, idea, cant);
  if (!ok.length) return fail(deps, idea, parts);
  deps.store.putIdea({ ...idea, state: "pick-reading", data: settled({ ...idea.data, readings: ok }, parts) });
}

/** The reader's problems when its author turned down the last options (M2-6). */
function turnedDown(options: string[] | undefined): { problems?: string[] } {
  return options?.length ? { problems: [`The player turned down these options; give different ones: ${options.join(" | ")}`] } : {};
}

/** The data with this stage's can't-express parts, and no retry pending. */
function settled(data: Idea["data"], parts: string[]): Idea["data"] {
  const { tries: _t, retryAt: _r, ...rest } = data;
  const all = [...new Set([...(rest.cantExpress ?? []), ...parts])];
  return all.length ? { ...rest, cantExpress: all } : rest;
}

/** Logs each missing word once per idea, and returns the parts to show: only
 * those quoted from the idea's own text (the player's words, never the
 * model's), as the text has them. */
function logCantExpress(deps: ReadDeps, idea: Idea, cant: CantExpress[]): string[] {
  const logged = new Set(deps.store.wordRequests().filter((w) => w.ideaId === idea.ideaId).map((w) => w.word.toLowerCase()));
  const parts: string[] = [];
  for (const c of cant) {
    const word = c.word.trim().slice(0, 60);
    const part = c.part.trim().slice(0, 200);
    if (!word) continue;
    if (!logged.has(word.toLowerCase())) {
      logged.add(word.toLowerCase());
      deps.store.addWordRequest({ ideaId: idea.ideaId, word, part, createdAt: deps.now().toISOString() });
    }
    const at = part ? idea.text.toLowerCase().indexOf(part.toLowerCase()) : -1;
    if (at >= 0) parts.push(idea.text.slice(at, at + part.length));
  }
  return [...new Set(parts)];
}

function fail(deps: ReadDeps, idea: Idea, parts: string[]): void {
  const failure = parts.length ? `${COULD_NOT} The game has no words yet for: ${parts.map((p) => `"${p}"`).join(", ")}.` : COULD_NOT;
  deps.store.putIdea({ ...idea, state: "failed", data: { ...settled(idea.data, parts), failure } });
  refundIdea(deps, idea.playerId);
}

// ---------- the author's picks (M2-6 adds the screens) ----------

function ownIdea(deps: ReadDeps, playerId: string, ideaId: string): Idea {
  const idea = deps.store.idea(ideaId);
  if (!idea || idea.playerId !== playerId) throw new IdeaRefused(404, "no such idea");
  return idea;
}

function pickIndex(index: unknown, options: unknown[] | undefined): number {
  if (!Number.isInteger(index) || (index as number) < 0 || (index as number) >= (options?.length ?? 0)) throw new IdeaRefused(400, `pick one of the ${options?.length ?? 0} options`);
  return index as number;
}

/** The player's own idea with its options. */
export function myIdea(deps: ReadDeps, playerId: string, ideaId: string): MyIdea {
  const { playerId: _, ...mine } = ownIdea(deps, playerId, ideaId);
  return mine;
}

/** Picks archetype `index`: the idea goes back to the reader for readings. */
export function pickArchetype(deps: ReadDeps, playerId: string, ideaId: string, index: unknown): void {
  const idea = ownIdea(deps, playerId, ideaId);
  if (idea.state !== "pick-archetype") throw new IdeaRefused(409, "This idea isn't waiting for an archetype.");
  const archetype = idea.data.archetypes![pickIndex(index, idea.data.archetypes)]!;
  deps.store.putIdea({ ...idea, state: "reading", data: settled({ ...idea.data, archetype }, []) });
}

/** Picks reading `index`: its Row becomes a candidate unit by the player, and
 * the idea goes on to the simulation. Another idea may have taken the name or
 * the shape since: then that option goes (and the stage is read again when
 * none is left) and the pick answers 409. */
export function pickReading(deps: ReadDeps, playerId: string, ideaId: string, index: unknown): void {
  const { store } = deps;
  const idea = ownIdea(deps, playerId, ideaId);
  if (idea.state !== "pick-reading") throw new IdeaRefused(409, "This idea isn't waiting for a reading.");
  const i = pickIndex(index, idea.data.readings);
  const archetype = idea.data.archetype!;
  const reading = idea.data.readings![i]!;
  if (archetypeDraftProblems(archetype, store.units().map((u) => u.row)).length) {
    const archetypes = (idea.data.archetypes ?? []).filter((a) => a.name !== archetype.name);
    const { archetype: _a, readings: _r, archetypes: _x, ...data } = idea.data;
    store.putIdea({ ...idea, state: archetypes.length ? "pick-archetype" : "reading", data: archetypes.length ? { ...data, archetypes } : data });
    throw new IdeaRefused(409, `Another unit was just named ${archetype.name}. Pick another archetype.`);
  }
  const checked = checkReading(reading, archetype, takenShapes(shapeRows(store)));
  if ("problems" in checked) {
    const readings = idea.data.readings!.filter((_, k) => k !== i);
    const { readings: _r, ...data } = idea.data;
    store.putIdea({ ...idea, state: readings.length ? "pick-reading" : "reading", data: readings.length ? { ...data, readings } : data });
    throw new IdeaRefused(409, "Another unit just took this reading's shape. Pick another one.");
  }
  const unitId = freshUnitId(store, archetype.name);
  store.putUnit({ unitId, status: "candidate", row: rowOf(archetype, reading), authorId: playerId, origin: "idea", parentId: null, createdAt: deps.now().toISOString() });
  store.putIdea({ ...idea, state: "simulating", data: { ...idea.data, unitId } });
}

/** Why an idea turned down twice came back. */
export const DECLINED_TWICE = "None of the options fit, so the idea is back with you to write again.";

/** "None of these" (M2-6): the stage waiting for a pick is read once more,
 * the options shown kept as turned down so the reader offers others. The
 * same stage turned down again fails the idea and refunds it. */
export function declineOptions(deps: ReadDeps, playerId: string, ideaId: string): void {
  const idea = ownIdea(deps, playerId, ideaId);
  const stage = idea.state === "pick-archetype" ? "archetypes" : idea.state === "pick-reading" ? "readings" : null;
  if (!stage) throw new IdeaRefused(409, "This idea isn't waiting for a pick.");
  const declined = idea.data.declined ?? {};
  if (declined[stage]) {
    deps.store.putIdea({ ...idea, state: "failed", data: { ...idea.data, failure: DECLINED_TWICE } });
    refundIdea(deps, playerId);
    return;
  }
  const { archetypes, readings, ...data } = idea.data;
  const kept = stage === "archetypes" ? { declined: { ...declined, archetypes: archetypes ?? [] } } : { archetypes, declined: { ...declined, readings: readings ?? [] } };
  deps.store.putIdea({ ...idea, state: "reading", data: { ...data, ...kept } });
}

/** A unit id no unit has had: ids are permanent, so the same name gets "-2", "-3"… */
function freshUnitId(store: MvpStore, name: string): string {
  const base = slug(name) || "idea";
  let id = base;
  for (let n = 2; store.unit(id); n++) id = `${base}-${n}`;
  return id;
}

// ---------- the job ----------

/** How often the job wakes: the model every 3 minutes (a small batch), the
 * fake every 5 s so a dev server answers at once; ARENA_IDEA_READ_MS overrides. */
export function readEveryMs(reader: IdeaReader, env: NodeJS.ProcessEnv = process.env): number {
  return Number(env.ARENA_IDEA_READ_MS) || (reader.kind === "fake" ? 5_000 : 3 * 60_000);
}

/** The job with this reader: reads a batch on start and every `everyMs`. */
export function ideaReadingJobWith(reader: IdeaReader, everyMs: number): MvpJob {
  return (rt) => {
    let stopped = false;
    let timer: ReturnType<typeof setTimeout> | undefined;
    const tick = () => {
      void readIdeas(rt, reader)
        .catch((err) => console.warn(`[ideas] the reading job failed: ${(err as Error).message}`))
        .finally(() => {
          if (stopped) return;
          timer = setTimeout(tick, everyMs);
          timer.unref?.();
        });
    };
    tick();
    return () => {
      stopped = true;
      clearTimeout(timer);
    };
  };
}

/** The job main.ts starts: the reader ARENA_IDEA_READER names (fake by default). */
export const ideaReadingJob: MvpJob = (rt) => {
  const reader = ideaReaderFromEnv();
  return ideaReadingJobWith(reader, readEveryMs(reader))(rt);
};
