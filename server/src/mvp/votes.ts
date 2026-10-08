// Votes, and the overnight check (mission 2, M2-8, makscee/void-board#793).
//
// The overnight check: every idea in `simulating` (its unit picked, M2-6) goes
// through M2-7's tuner (scripts/tune-unit.ts, one child process per unit, so
// the server keeps answering). A pass stores the tuned row in mvp_units as a
// `candidate` and moves the idea to `voting`; a fail stores it `rejected`,
// moves the idea to `failed` with the tuner's reason and refunds the idea. A
// unit the tuner couldn't finish (cut off at the deadline, or crashed) stays
// in `simulating` for the next night. The job runs it after the day ends
// (rules.dayEndsAt) and stops starting units at NIGHT_HOURS later; the dev
// menu runs it at once.
//
// Votes: a player gets either/or cards, a candidate against a typical live
// unit (one near the median pick rate of the last TYPICAL_DAYS days' tallies),
// in random order, never their own candidate and never a pair they voted on.
// A candidate's score is the share of votes it won (skips not counted) plus a
// novelty bonus for When/Who/Does parts rare in the live pool; it qualifies
// with at least rules.voteMin votes and a score over one half. M2-10 enters
// qualified(candidateScores(rt)) best first.
import { spawn } from "node:child_process";
import { mkdtempSync, readFileSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import type { CandidateScore, Idea, MvpRules, PlayerRef, UnitContent, UnitId, VoteCard, VoteRequest } from "../../../src/mvp/contract.js";
import { nextRollover } from "../../../src/mvp/day.js";
import { slugOf } from "../../../src/mvp/tune.js";
import { allDoes, effectKinds, mvpPool, ROWS, type Row } from "../../../src/mvp/units.js";
import { IdeaRefused } from "./ideas.js";
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
/** Tunes one unit; null when it couldn't finish (aborted, or the tuner failed). */
export type Tuner = (row: Row, signal: AbortSignal) => Promise<TuneVerdict | null>;

/** Hours after the day end the overnight check may start a unit. */
export const NIGHT_HOURS = 5;
const NIGHT_TICK_MS = 10 * 60_000;
const REPO_ROOT = fileURLToPath(new URL("../../../", import.meta.url));

/** The tuner as a child process: `npm run mvp:tune -- --batch <one row>`,
 * with the full meta check, or the quick one. */
export function childTuner(meta: "full" | "quick"): Tuner {
  return (row, signal) =>
    new Promise((resolve) => {
      const dir = mkdtempSync(join(tmpdir(), "arena-tune-"));
      const file = join(dir, "rows.json");
      writeFileSync(file, JSON.stringify([row]));
      const args = ["--import", "tsx/esm", "scripts/tune-unit.ts", "--batch", file, ...(meta === "quick" ? ["--quick"] : [])];
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

/** A unit id for `name` no unit has had: its slug, or slug-2, slug-3… */
function freshUnitId(store: MvpStore, name: string): UnitId {
  const base = slugOf(name) || "unit";
  let id = base;
  for (let n = 2; store.unit(id); n++) id = `${base}-${n}`;
  return id;
}

/** Gives an idea back to its author (a failed check). */
function refundIdea(store: MvpStore, playerId: string): void {
  const counts = store.ideaCounts(playerId);
  counts.spent = Math.max(0, counts.spent - 1);
  store.putIdeaCounts(playerId, counts);
}

/** Applies a verdict to its idea: the unit stored, the idea moved on. */
export function judgeIdea(rt: Pick<RunDeps, "store" | "now">, idea: Idea, v: TuneVerdict): Idea {
  const now = rt.now().toISOString();
  const unitId = idea.data.unitId ?? freshUnitId(rt.store, v.row.name);
  const unit: StoredUnit = { unitId, status: v.pass ? "candidate" : "rejected", row: v.row, authorId: idea.playerId, origin: "idea", parentId: null, createdAt: now };
  rt.store.putUnit(unit);
  const next: Idea = { ...idea, state: v.pass ? "voting" : "failed", data: { ...idea.data, row: v.row, unitId, checkedAt: now, ...(v.pass ? {} : { reason: v.reason }) } };
  rt.store.putIdea(next);
  if (!v.pass) refundIdea(rt.store, idea.playerId);
  return next;
}

const running = new WeakMap<MvpStore, Promise<number>>();

/** Runs the check on every `simulating` idea with a row, oldest first, one
 * at a time, starting none after `deadline`. Resolves to the ideas judged.
 * A second call while one runs gets the running one. */
export function overnightCheck(rt: Pick<RunDeps, "store" | "now">, tuner: Tuner, deadline: Date | null = null): Promise<number> {
  const busy = running.get(rt.store);
  if (busy) return busy;
  const ac = new AbortController();
  const timer = deadline ? setTimeout(() => ac.abort(), Math.max(0, deadline.getTime() - rt.now().getTime())) : null;
  timer?.unref?.();
  const work = (async () => {
    let judged = 0;
    for (const idea of rt.store.ideas({ state: "simulating" })) {
      if (ac.signal.aborted || (deadline && rt.now() >= deadline)) break;
      if (!idea.data.row) continue;
      const v = await tuner(idea.data.row, ac.signal);
      // Taken back or moved on meanwhile: leave it.
      if (!v || rt.store.idea(idea.ideaId)?.state !== "simulating") continue;
      judgeIdea(rt, rt.store.idea(idea.ideaId)!, v);
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
  return store.ideas({ state: "simulating" }).filter((i) => i.data.row).length;
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

/** Dev: an idea of `playerId`'s in `simulating`, its unit a live one renamed
 * (M2-5 and M2-6 fill in real ones). Spends nothing. */
export function seedCandidate(rt: Pick<RunDeps, "store" | "content" | "now" | "seed">, playerId: string): Idea {
  const live = liveRows(rt.store);
  const from = live[rt.seed() % live.length]!;
  const n = rt.store.ideas().length + 1;
  const row: Row = { ...from, name: `${from.name} Echo ${n}` };
  const idea: Idea = { ideaId: `dev-${n}-${rt.seed().toString(36)}`, playerId, text: `A dev candidate: ${row.name}.`, state: "simulating", createdAt: rt.now().toISOString(), data: { row } };
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

/** A candidate's unit as the client draws it, under its own id. */
function unitContentOf(id: UnitId, row: Row): UnitContent {
  return { ...mvpPool([row]).units[0]!, id };
}

/** The live units' rows: the stored ones, or the code pool before the store
 * is seeded (tests). */
function liveRows(store: MvpStore): Row[] {
  const live = store.units({ status: "live" }).map((u) => u.row);
  return live.length ? live : ROWS;
}

/** Candidates being voted on: stored `candidate` units. */
function candidates(store: MvpStore): StoredUnit[] {
  return store.units({ status: "candidate" });
}

/** The next card for `player`, or null: the candidate with the fewest votes
 * they may still vote on, against a typical unit they haven't met it with. */
export function nextCard(rt: VoteDeps, player: PlayerRef): VoteCard | null {
  if (player.bot) return null;
  const mine = rt.store.votes({ playerId: player.id });
  const typical = typicalUnits(rt);
  const open = candidates(rt.store)
    .filter((c) => c.authorId !== player.id)
    .map((c) => ({ c, n: rt.store.votes({ candidateId: c.unitId }).length, others: typical.filter((o) => !mine.some((v) => v.candidateId === c.unitId && v.otherId === o)) }))
    .filter((x) => x.others.length)
    .sort((a, b) => a.n - b.n);
  const pick = open[0];
  if (!pick) return null;
  const r = rt.seed();
  const otherId = pick.others[r % pick.others.length]!;
  const other = rt.content.units.find((u) => u.id === otherId)!;
  const cand = unitContentOf(pick.c.unitId, pick.c.row);
  const pool = mvpPool([pick.c.row]);
  const units: [UnitContent, UnitContent] = (r >> 8) % 2 ? [cand, other] : [other, cand];
  // The live unit's abilities are in the content already; the card carries the candidate's.
  return { candidateId: cand.id, otherId, units, pool: { abilities: pool.abilities, statuses: pool.statuses, summons: pool.summons } };
}

/** Records `player`'s vote. Throws IdeaRefused (409 a bot, 404 no such pair,
 * 400 a pick of neither, 409 their own candidate or a pair already voted). */
export function castVote(rt: VoteDeps, player: PlayerRef, req: VoteRequest): void {
  if (player.bot) throw new IdeaRefused(409, "bots don't vote");
  const cand = rt.store.unit(req.candidateId);
  if (!cand || cand.status !== "candidate" || !rt.content.units.some((u) => u.id === req.otherId)) throw new IdeaRefused(404, "no such pair");
  if (req.pick !== null && req.pick !== req.candidateId && req.pick !== req.otherId) throw new IdeaRefused(400, "pick one of the two, or skip");
  if (cand.authorId === player.id) throw new IdeaRefused(409, "you can't vote on your own idea");
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

/** Every candidate's standing, qualified ones first, best first. */
export function candidateScores(rt: Pick<RunDeps, "store" | "rules">): CandidateScore[] {
  const { min, bonus } = voteRules(rt.rules);
  const live = liveRows(rt.store);
  return candidates(rt.store)
    .map((c) => {
      const counted = rt.store.votes({ candidateId: c.unitId }).filter((v) => v.pick !== null);
      const won = counted.filter((v) => v.pick === c.unitId).length;
      const share = counted.length ? won / counted.length : 0;
      const nov = novelty(c.row, live);
      const score = share + bonus * nov;
      const r3 = (x: number) => Math.round(x * 1000) / 1000;
      return { unitId: c.unitId, name: c.row.name, emoji: c.row.emoji, authorId: c.authorId, votes: counted.length, won, share: r3(share), novelty: r3(nov), score: r3(score), qualified: counted.length >= min && score > 0.5 };
    })
    .sort((a, b) => Number(b.qualified) - Number(a.qualified) || b.score - a.score || b.votes - a.votes);
}

/** The candidates that may enter (M2-10), best first. */
export function qualified(scores: CandidateScore[]): CandidateScore[] {
  return scores.filter((s) => s.qualified).sort((a, b) => b.score - a.score);
}

/** Dev: FAKE_VOTES votes on each candidate from fake voters, all but one for it. */
export const FAKE_VOTES = 5;
export function fakeVotes(rt: VoteDeps): CandidateScore[] {
  const typical = typicalUnits(rt);
  for (const c of candidates(rt.store)) {
    const n = rt.store.votes({ candidateId: c.unitId }).length;
    for (let i = 0; i < FAKE_VOTES; i++) {
      const otherId = typical[(n + i) % typical.length]!;
      rt.store.addVote({ playerId: `dev-voter-${n + i}`, candidateId: c.unitId, otherId, pick: i === FAKE_VOTES - 1 ? otherId : c.unitId, createdAt: rt.now().toISOString() });
    }
  }
  return candidateScores(rt);
}
