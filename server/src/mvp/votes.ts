// Votes, and the overnight check (mission 2, M2-8, makscee/void-board#793).
//
// The overnight check: every idea in `simulating` (its reading picked, M2-6;
// its unit a `candidate` in mvp_units, M2-5) goes through M2-7's tuner
// (scripts/tune-unit.ts, one child process per unit, so the server keeps
// answering), measured against the current pool in the DB. A pass stores the
// tuned row on its unit and moves the idea to `voting`; a fail stores it
// `rejected`, moves the idea to `failed` with the tuner's reason and refunds
// the idea. A
// unit the tuner couldn't finish (cut off at the deadline, or crashed) stays
// in `simulating` for the next night. The job runs it after the day ends
// (rules.dayEndsAt) and stops starting units at NIGHT_HOURS later; the dev
// menu runs it at once.
//
// Votes: a player gets either/or cards, a candidate against a typical live
// unit (one near the median pick rate of the last TYPICAL_DAYS days' tallies),
// in random order, never their own candidate and never a pair they voted on.
// Only ideas in `voting` are on cards. A candidate's score is the share of votes it won (skips not counted) plus a
// novelty bonus for When/Who/Does parts rare in the live pool; it qualifies
// with at least rules.voteMin votes and a score over one half. M2-10 enters
// qualified(candidateScores(rt)) best first.
//
// M3-6 (mission #800): a new version of a Library unit (an `evolve` idea in
// `voting`) is a candidate like an idea's unit, and while its archetype has
// one, the archetype's Library version is a candidate too, "unchanged" (no new
// row: its votes are under the Library unit's id). Anyone may vote on
// "unchanged", its author included. Of one archetype's candidates only the
// best qualified one is its entry; "unchanged" only by scoring above every
// qualified version.
import { spawn } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { ideaKind, type CandidateKind, type CandidateScore, type Idea, type MvpRules, type PlayerRef, type UnitContent, type UnitId, type VoteCard, type VoteRequest } from "../../../src/mvp/contract.js";
import { nextRollover } from "../../../src/mvp/day.js";
import { slugOf } from "../../../src/mvp/tune.js";
import { allDoes, effectKinds, mvpPool, ROWS, type Row } from "../../../src/mvp/units.js";
import { IdeaRefused, refundIdea } from "./ideas.js";
import { lineage } from "./lineage.js";
import type { RunDeps } from "./runs.js";
import type { MvpJob, MvpRuntime } from "./runtime.js";
import type { MvpStore, StoredUnit } from "./store.js";

type VoteDeps = Pick<RunDeps, "store" | "content" | "rules" | "now" | "seed" | "today">;

/** The vote tunables, defaults filled in. */
export function voteRules(rules: Pick<MvpRules, "voteMin" | "voteNoveltyBonus">): { min: number; bonus: number } {
  return { min: Math.max(1, rules.voteMin ?? 5), bonus: Math.max(0, rules.voteNoveltyBonus ?? 0.1) };
}

// ---------- the overnight check ----------

/** M2-7's verdict on one unit: the row with its tuned numbers. */
export interface TuneVerdict {
  pass: boolean;
  reason: string;
  row: Row;
}
/** Tunes one unit against the live `pool`; null when it couldn't finish
 * (aborted, or the tuner failed). */
export type Tuner = (row: Row, pool: Row[], signal: AbortSignal) => Promise<TuneVerdict | null>;

/** Hours after the day end the overnight check may start a unit. */
export const NIGHT_HOURS = 5;
const NIGHT_TICK_MS = 10 * 60_000;
const REPO_ROOT = fileURLToPath(new URL("../../../", import.meta.url));

/** The tuner as a child process: `npm run mvp:tune -- --batch <one row>
 * --pool <the live rows>`, with the full meta check, or the quick one. */
export function childTuner(meta: "full" | "quick"): Tuner {
  return (row, pool, signal) =>
    new Promise((resolve) => {
      const dir = mkdtempSync(join(tmpdir(), "arena-tune-"));
      const file = join(dir, "rows.json");
      const poolFile = join(dir, "pool.json");
      writeFileSync(file, JSON.stringify([row]));
      writeFileSync(poolFile, JSON.stringify(pool));
      const args = ["--import", "tsx/esm", "scripts/tune-unit.ts", "--batch", file, "--pool", poolFile, ...(meta === "quick" ? ["--quick"] : [])];
      const child = spawn(process.execPath, args, { cwd: REPO_ROOT, stdio: ["ignore", "ignore", "pipe"] });
      let tail = "";
      child.stderr.on("data", (d: Buffer) => (tail = (tail + d.toString()).slice(-2000)));
      const kill = () => child.kill("SIGTERM");
      signal.addEventListener("abort", kill, { once: true });
      child.on("close", (code) => {
        signal.removeEventListener("abort", kill);
        let out: TuneVerdict | null = null;
        try {
          if (code === 0) out = (JSON.parse(readFileSync(join(dir, "rows.tuned.json"), "utf8")) as TuneVerdict[])[0] ?? null;
        } catch {
          out = null;
        }
        if (!out && !signal.aborted) console.error(`[overnight] the tuner failed on ${row.name} (exit ${code}):\n${tail}`);
        rmSync(dir, { recursive: true, force: true });
        resolve(out && { pass: out.pass, reason: out.reason, row: out.row });
      });
    });
}

/** Dev servers and the bot (M2-11): a tuner that passes every unit at once
 * with its numbers as read, so an e2e walks the vote without minutes of
 * simulation (main.ts: ARENA_TUNER=instant with MVP_DEV=1). */
export const instantTuner: Tuner = async (row) => ({ pass: true, reason: "instant tuner (dev)", row });

/** A unit id for `name` no unit has had: its slug, or slug-2, slug-3… */
function freshUnitId(store: MvpStore, name: string): UnitId {
  const base = slugOf(name) || "unit";
  let id = base;
  for (let n = 2; store.unit(id); n++) id = `${base}-${n}`;
  return id;
}

type CheckDeps = Pick<RunDeps, "store" | "rules" | "now">;

/** What a player reads when their idea failed the check. */
export function checkFailure(reason: string): string {
  return `It didn't pass the overnight check: ${reason.replace(/\.$/, "")}.`;
}

/** Applies a verdict to its idea: the tuned row on its unit (`rejected` on a
 * fail), the idea moved on (a fail refunded). */
export function judgeIdea(rt: CheckDeps, idea: Idea, unit: StoredUnit, v: TuneVerdict): Idea {
  const now = rt.now().toISOString();
  rt.store.putUnit({ ...unit, status: v.pass ? "candidate" : "rejected", row: { ...v.row, name: unit.row.name } });
  const next: Idea = { ...idea, state: v.pass ? "voting" : "failed", data: { ...idea.data, checkedAt: now, ...(v.pass ? {} : { failure: checkFailure(v.reason) }) } };
  rt.store.putIdea(next);
  if (!v.pass) refundIdea(rt, idea.playerId);
  return next;
}

/** The current pool's rows, in its order: what a candidate is measured
 * against (the code pool before the store is seeded: tests). */
export function poolRows(store: MvpStore): Row[] {
  const pool = store.currentPool();
  if (!pool) return ROWS;
  const rows = pool.unitIds.map((id) => store.unit(id)?.row);
  return rows.every((r) => r) ? (rows as Row[]) : ROWS;
}

/** An idea's unit while it waits for the check: its stored candidate. */
function checkedUnit(store: MvpStore, idea: Idea): StoredUnit | null {
  const u = idea.data.unitId ? store.unit(idea.data.unitId) : null;
  return u && u.status === "candidate" ? u : null;
}

const running = new WeakMap<MvpStore, Promise<number>>();

/** Runs the check on every `simulating` idea with a unit, oldest first, one
 * at a time, starting none after `deadline`. Resolves to the ideas judged.
 * A second call while one runs gets the running one. */
export function overnightCheck(rt: CheckDeps, tuner: Tuner, deadline: Date | null = null): Promise<number> {
  const busy = running.get(rt.store);
  if (busy) return busy;
  const ac = new AbortController();
  const timer = deadline ? setTimeout(() => ac.abort(), Math.max(0, deadline.getTime() - rt.now().getTime())) : null;
  timer?.unref?.();
  const work = (async () => {
    let judged = 0;
    for (const idea of rt.store.ideas({ state: "simulating" })) {
      if (ac.signal.aborted || (deadline && rt.now() >= deadline)) break;
      const unit = checkedUnit(rt.store, idea);
      if (!unit) continue;
      const v = await tuner(unit.row, poolRows(rt.store), ac.signal);
      // Moved on meanwhile: leave it.
      const now = rt.store.idea(idea.ideaId);
      const still = now && checkedUnit(rt.store, now);
      if (!v || now?.state !== "simulating" || !still) continue;
      judgeIdea(rt, now, still, v);
      judged++;
    }
    return judged;
  })().finally(() => {
    if (timer) clearTimeout(timer);
    running.delete(rt.store);
  });
  running.set(rt.store, work);
  return work;
}

/** Ideas waiting for the check (what the dev button starts on). */
export function waitingForCheck(store: MvpStore): number {
  return store.ideas({ state: "simulating" }).filter((i) => checkedUnit(store, i)).length;
}

/** The end of tonight's window when `now` is in it (after the day end, for
 * NIGHT_HOURS), else null. */
export function nightDeadline(now: Date, rules: Pick<MvpRules, "dayEndsAt" | "dayTimeZone">): Date | null {
  const last = nextRollover(new Date(now.getTime() - 86_400_000), rules);
  const end = new Date(last.getTime() + NIGHT_HOURS * 3_600_000);
  return now >= last && now < end ? end : null;
}

/** The job: every NIGHT_TICK_MS, inside the night window, checks what waits
 * with the night tuner (the full meta check). */
export const overnightJob: MvpJob = (rt: MvpRuntime) => {
  const tick = () => {
    const deadline = nightDeadline(rt.now(), rt.rules);
    if (!deadline || !waitingForCheck(rt.store)) return;
    rt.today(); // the day ends first
    overnightCheck(rt, rt.tuner.night, deadline).catch((err) => console.error("[overnight] check failed", err));
  };
  tick();
  const timer = setInterval(tick, NIGHT_TICK_MS);
  timer.unref?.();
  return () => clearInterval(timer);
};

/** Dev: an idea of `playerId`'s in `simulating` without the model (M2-5),
 * its candidate unit a live one renamed: `unit`'s copy, else a random one's.
 * Spends nothing. */
export function seedCandidate(rt: Pick<RunDeps, "store" | "content" | "now" | "seed">, playerId: string, unit?: UnitId): Idea {
  const live = poolRows(rt.store);
  const from = (unit && live.find((r) => slugOf(r.name) === unit)) || live[rt.seed() % live.length]!;
  const n = rt.store.ideas().length + 1;
  const row: Row = { ...from, name: `${from.name} Echo ${n}` };
  const now = rt.now().toISOString();
  const unitId = freshUnitId(rt.store, row.name);
  rt.store.putUnit({ unitId, status: "candidate", row, authorId: playerId, origin: "idea", parentId: null, createdAt: now });
  const idea: Idea = { ideaId: `dev-${n}-${rt.seed().toString(36)}`, playerId, text: `A dev candidate: ${row.name}.`, state: "simulating", createdAt: now, data: { unitId } };
  rt.store.putIdea(idea);
  return idea;
}

// ---------- votes ----------

/** Days of tallies the typical units are read from (today and before). */
export const TYPICAL_DAYS = 3;

/** Live units near the median pick rate: the middle third by picks per
 * finished run over the last TYPICAL_DAYS days; every live unit before any
 * tallies. */
export function typicalUnits(rt: Pick<RunDeps, "store" | "content" | "today">): UnitId[] {
  const live = rt.content.units.map((u) => u.id);
  const seq = rt.today().seq;
  let runs = 0;
  const picks = new Map<UnitId, number>(live.map((id) => [id, 0]));
  for (let d = Math.max(1, seq - TYPICAL_DAYS + 1); d <= seq; d++) {
    const t = rt.store.dayTallies(d);
    runs += t.runs;
    for (const u of t.units) if (picks.has(u.unitId)) picks.set(u.unitId, picks.get(u.unitId)! + u.picks);
  }
  if (runs === 0 || live.length < 3) return live;
  const sorted = [...live].sort((a, b) => picks.get(a)! - picks.get(b)! || (a < b ? -1 : 1));
  const third = Math.floor(sorted.length / 3);
  return sorted.slice(third, sorted.length - third);
}

/** A candidate's unit as the client draws it, under its own id, with its
 * Russian name and line when it has them (M4-5). */
function unitContentOf(id: UnitId, row: Row, texts?: StoredUnit["texts"]): UnitContent {
  return { ...mvpPool([row]).units[0]!, id, ...(texts?.ru ? { texts } : {}) };
}

/** A unit on the vote cards, and what it is. */
interface Candidate {
  unit: StoredUnit;
  kind: CandidateKind;
  rootId: UnitId;
}

/** Candidates being voted on: the units of ideas in `voting` (new ideas and
 * new versions), and for each archetype with a version among them its Library
 * version, "unchanged": the newest one in the Library that has been live,
 * none while a version of it is live. */
function candidates(store: MvpStore): Candidate[] {
  const out: Candidate[] = [];
  const roots = new Set<UnitId>();
  for (const i of store.ideas({ state: "voting" })) {
    const unit = checkedUnit(store, i);
    if (!unit) continue;
    const version = ideaKind(i) === "evolve";
    const rootId = unit.rootId ?? unit.unitId;
    out.push({ unit, kind: version ? "version" : "idea", rootId });
    if (version) roots.add(rootId);
  }
  for (const rootId of roots) {
    const versions = lineage(store, rootId);
    if (versions.some((u) => u.status === "live")) continue;
    // M3-7: a proposal that lost is in the Library too, but was never live.
    const unit = versions.filter((u) => u.status === "library" && (u.origin === "seed" || store.stints(u.unitId).length)).at(-1);
    if (unit) out.push({ unit, kind: "unchanged", rootId });
  }
  return out;
}

/** May `playerId` vote on `c`? Not on their own idea or version; on
 * "unchanged", anyone. */
const mayVote = (c: Candidate, playerId: string) => c.kind === "unchanged" || c.unit.authorId !== playerId;

/** The next card for `player`, or null: the candidate with the fewest votes
 * they may still vote on, against a typical unit they haven't met it with. */
export function nextCard(rt: VoteDeps, player: PlayerRef): VoteCard | null {
  if (player.bot) return null;
  const mine = rt.store.votes({ playerId: player.id });
  const typical = typicalUnits(rt);
  const open = candidates(rt.store)
    .filter((c) => mayVote(c, player.id))
    .map((c) => ({ c, n: rt.store.votes({ candidateId: c.unit.unitId }).length, others: typical.filter((o) => !mine.some((v) => v.candidateId === c.unit.unitId && v.otherId === o)) }))
    .filter((x) => x.others.length)
    .sort((a, b) => a.n - b.n);
  const pick = open[0];
  if (!pick) return null;
  const r = rt.seed();
  const otherId = pick.others[r % pick.others.length]!;
  const live = rt.content.units.find((u) => u.id === otherId)!;
  const otherTexts = rt.store.unit(otherId)?.texts;
  const other: UnitContent = otherTexts?.ru && !live.texts ? { ...live, texts: otherTexts } : live;
  const cand = unitContentOf(pick.c.unit.unitId, pick.c.unit.row, pick.c.unit.texts);
  const pool = mvpPool([pick.c.unit.row]);
  const units: [UnitContent, UnitContent] = (r >> 8) % 2 ? [cand, other] : [other, cand];
  // The live unit's abilities are in the content already; the card carries the candidate's.
  return { candidateId: cand.id, candidateKind: pick.c.kind, otherId, units, pool: { abilities: pool.abilities, statuses: pool.statuses, summons: pool.summons } };
}

/** Records `player`'s vote. Throws IdeaRefused (409 a bot, 404 no such pair,
 * 400 a pick of neither, 409 their own candidate or a pair already voted). */
export function castVote(rt: VoteDeps, player: PlayerRef, req: VoteRequest): void {
  if (player.bot) throw new IdeaRefused(409, "bots don't vote");
  const cand = candidates(rt.store).find((c) => c.unit.unitId === req.candidateId);
  if (!cand || !rt.content.units.some((u) => u.id === req.otherId)) throw new IdeaRefused(404, "no such pair");
  if (req.pick !== null && req.pick !== req.candidateId && req.pick !== req.otherId) throw new IdeaRefused(400, "pick one of the two, or skip");
  if (!mayVote(cand, player.id)) throw new IdeaRefused(409, "you can't vote on your own idea");
  const ok = rt.store.addVote({ playerId: player.id, candidateId: req.candidateId, otherId: req.otherId, pick: req.pick, createdAt: rt.now().toISOString() });
  if (!ok) throw new IdeaRefused(409, "you voted on this pair already");
}

/** A row's parts for novelty: its When, its Who (sleeping and awoken), and
 * every effect kind either form does. */
function partsOf(row: Row): { when: string; who: string[]; does: string[] } {
  const u = mvpPool([row]).units[0]!;
  return {
    when: row.when,
    who: [...new Set([row.who, row.awoken.who ?? row.who])],
    does: effectKinds([...allDoes(u.forms.sleeping), ...allDoes(u.forms.awoken)]),
  };
}

/** 0–1: how rare the row's When, Who and Does parts are among `liveRows`:
 * the mean of the three, each 1 − (live units with that part) / (live units
 * with the most common part of its kind). A part no live unit has scores 1,
 * the most common one 0. */
export function novelty(row: Row, liveRows: Row[]): number {
  if (!liveRows.length) return 1;
  const live = liveRows.map(partsOf);
  const p = partsOf(row);
  const rarity = (values: string[][], mine: string[]) => {
    const count = new Map<string, number>();
    for (const vs of values) for (const v of vs) count.set(v, (count.get(v) ?? 0) + 1);
    const most = Math.max(1, ...count.values());
    return mean(mine.map((v) => 1 - (count.get(v) ?? 0) / most));
  };
  return mean([rarity(live.map((l) => [l.when]), [p.when]), rarity(live.map((l) => l.who), p.who), rarity(live.map((l) => l.does), p.does)]);
}

const mean = (xs: number[]) => (xs.length ? xs.reduce((a, b) => a + b, 0) / xs.length : 0);

/** Every candidate's standing, qualified ones first, best first, each
 * archetype's entry marked (M3-6). */
export function candidateScores(rt: Pick<RunDeps, "store" | "rules">): CandidateScore[] {
  const { min, bonus } = voteRules(rt.rules);
  const live = poolRows(rt.store);
  const scores = candidates(rt.store)
    .map(({ unit: c, kind, rootId }): CandidateScore => {
      const counted = rt.store.votes({ candidateId: c.unitId }).filter((v) => v.pick !== null);
      const won = counted.filter((v) => v.pick === c.unitId).length;
      const share = counted.length ? won / counted.length : 0;
      // "Unchanged" isn't new: no novelty bonus, so a version wins an even
      // vote and unchanged enters only when players prefer it (#800, Maks).
      const nov = kind === "unchanged" ? 0 : novelty(c.row, live);
      const score = share + bonus * nov;
      const r3 = (x: number) => Math.round(x * 1000) / 1000;
      return { unitId: c.unitId, kind, rootId, name: c.row.name, emoji: c.row.emoji, authorId: c.authorId, votes: counted.length, won, share: r3(share), novelty: r3(nov), score: r3(score), qualified: counted.length >= min && score > 0.5, entry: false };
    })
    .sort((a, b) => Number(b.qualified) - Number(a.qualified) || b.score - a.score || b.votes - a.votes);
  // Each archetype's entry: its best qualified version, unless "unchanged"
  // scores above it (on a tie the version enters).
  const best = new Map<UnitId, { version?: CandidateScore; unchanged?: CandidateScore }>();
  for (const s of scores) {
    if (!s.qualified) continue;
    const b = best.get(s.rootId) ?? {};
    if (s.kind === "unchanged") b.unchanged = s;
    else b.version ??= s; // scores are best first
    best.set(s.rootId, b);
  }
  for (const { version, unchanged } of best.values()) {
    const entry = unchanged && (!version || unchanged.score > version.score) ? unchanged : version;
    if (entry) entry.entry = true;
  }
  return scores;
}

/** The candidates that may enter (M2-10), best first: one per archetype (M3-6). */
export function qualified(scores: CandidateScore[]): CandidateScore[] {
  return scores.filter((s) => s.qualified && s.entry).sort((a, b) => b.score - a.score);
}

/** M3-6: the archetypes with versions on the cards, each with its versions
 * and "unchanged", best first (from candidateScores). */
export function contests(scores: CandidateScore[]): { rootId: UnitId; candidates: CandidateScore[] }[] {
  const by = new Map<UnitId, CandidateScore[]>();
  for (const s of scores) if (s.kind !== "idea") by.set(s.rootId, [...(by.get(s.rootId) ?? []), s]);
  return [...by].map(([rootId, cs]) => ({ rootId, candidates: [...cs].sort((a, b) => Number(b.entry) - Number(a.entry) || b.score - a.score || b.votes - a.votes) }));
}

/** Dev: FAKE_VOTES votes on each candidate from fake voters, all but one for it. */
export const FAKE_VOTES = 5;
export function fakeVotes(rt: VoteDeps): CandidateScore[] {
  const typical = typicalUnits(rt);
  for (const { unit: c } of candidates(rt.store)) {
    const n = rt.store.votes({ candidateId: c.unitId }).length;
    for (let i = 0; i < FAKE_VOTES; i++) {
      const otherId = typical[(n + i) % typical.length]!;
      rt.store.addVote({ playerId: `dev-voter-${n + i}`, candidateId: c.unitId, otherId, pick: i === FAKE_VOTES - 1 ? otherId : c.unitId, createdAt: rt.now().toISOString() });
    }
  }
  return candidateScores(rt);
}
