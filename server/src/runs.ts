/**
 * Run submission — the shared ladder's trust boundary. A finished run arrives
 * in the kernel's serializeRun form (seed + decision log + final state, all by
 * value). NOTHING in it is trusted: the server re-derives the whole run
 * through the kernel's pure transitions and accepts only what it computed
 * itself — ghosts and crowns enter the shared pool from the re-derived state,
 * never from client-claimed stats.
 *
 * How a ladder fight replays: the kernel's opponent draw is deterministic
 * given the run's RNG state and the pool the run saw. Pools are append-only
 * and the server is the only writer, so the pool a client saw is always a
 * PREFIX of the server's pool (per round, in the user-filtered view — a
 * user's own ghosts never appear in their draws). Each Snapshotted event in
 * the log pins that prefix's length (the kernel sets a ghost's seq to the
 * pool length it drew from), and each BossChallenged event names the
 * champion seated at the time, which the append-only champion history can
 * still produce. Replaying against those historical views reproduces the run
 * byte-for-byte — or doesn't, and the submission is rejected with the reason.
 *
 * The claimed prefix is forgeable in both directions, so it is never taken
 * at face value: a log claiming a SHORTER prefix than the player observed
 * cherry-picks the deterministic draw, and a log claiming an EMPTY pool turns
 * the kernel's outran-every-ghost rule into a free champion challenge. Window
 * checks ([what was visible at open, what is visible now]) are not enough —
 * an open never cashed in is a bankable asset whose window widens as the
 * ladder grows, until a genesis open can dodge every ghost that came after it
 * (the banked-open forgery). So the server accepts only views IT HANDED OUT:
 *
 *   1. A run is opened before play (openRun, POST /v1/runs/open) — the row
 *      records the owner, and opens expire after RUN_OPEN_TTL_SECONDS.
 *   2. Every play read goes through the run-scoped serve (servePool, GET
 *      /v1/runs/:runId/pool/:round), which returns the user-filtered pool
 *      AND the seated champion, and RECORDS what it served (run_pool_serves:
 *      prefix length + champion, per runId and round).
 *   3. Replay accepts a claimed Snapshotted.seq only if it equals a length
 *      the server served for that (runId, round), and a champion challenge
 *      only against the champion CO-SERVED with that very view — a run
 *      cannot fight "the past's pool" against "the present's champion".
 *
 * Honest play is never rejected: each fight's view is one serve, re-reads
 * just add rows, a slow run resumed days later replays against the views it
 * actually fetched (the generous TTL only kills banking, not slow play).
 * What remains for a cheater is exactly what an honest slow player could
 * produce anyway — the views the server really served them, inside the TTL.
 * Submitting unopened, expired, or another user's runId is rejected before
 * replay.
 *
 * What acceptance writes: the re-derived ghosts, re-sequenced onto the END of
 * the current pools (the historical seq pinned the draw; insertion order is
 * the server's). A re-derived crown is applied only if the champion the run
 * beat is STILL seated — two players can legally beat the same champion, and
 * the spot goes to the first submission; the loser of that race still lands
 * its ghosts, just not the crown.
 */
import { isDeepStrictEqual } from "node:util";
import {
  applyDecision,
  BOOTSTRAP_RUN_ID,
  challengeBoss,
  deserializeRun,
  initRun,
  InvalidDecisionError,
  ladderFight,
  ValidationError,
  type LadderStore,
  type RunEndReason,
  type RunEvent,
  type RunState,
  type TeamSnapshot,
} from "../../src/index.js";
import { eq } from "drizzle-orm";
import type { ArenaContent } from "./content.js";
import type { DB } from "./db.js";
import type { SqliteLadderStore } from "./ladder-store.js";
import { runOpens, runPoolServes, runSubmissions } from "./schema.js";

export interface SubmitDeps {
  db: DB;
  store: SqliteLadderStore;
  content: ArenaContent;
  /** Unix seconds. */
  clock: () => number;
}

export type SubmitOutcome =
  | { accepted: true; runId: string; endedBy: RunEndReason; finalRound: number; crowned: boolean }
  | { accepted: false; reason: string };

export type OpenOutcome = { opened: true; runId: string } | { opened: false; reason: string };

/** A submission failing re-derivation — carries the reason the client sees. */
class SubmissionRejected extends Error {}

// Replay is synchronous — bound what a submission may cost before replaying.
export const MAX_RUN_BYTES = 256 * 1024;
export const MAX_RUN_LOG_EVENTS = 5_000;
export const MAX_RUN_ID_LENGTH = 128;

/** How long an open run stays submittable. Generously above any honest run's
 * lifetime (a run is a play session, maybe resumed over a few days) — the TTL
 * exists only to bound open-banking: without it, an open held back at ladder
 * genesis stays cashable forever, its recorded views ever more anomalous. */
export const RUN_OPEN_TTL_DAYS = 14;
export const RUN_OPEN_TTL_SECONDS = RUN_OPEN_TTL_DAYS * 24 * 60 * 60;

export type PoolServeOutcome =
  | { served: true; round: number; pool: readonly TeamSnapshot[]; champion: TeamSnapshot | null }
  | { served: false; reason: string };

/** Open a run before playing it: records who owns the runId, and from when
 * the open TTL counts. One-shot per runId, like submission itself (and a
 * submitted runId can never be re-opened, even after its open row is swept). */
export function openRun(deps: Pick<SubmitDeps, "db" | "store" | "clock">, userId: string, runId: string): OpenOutcome {
  if (runId === BOOTSTRAP_RUN_ID) {
    return { opened: false, reason: `runId "${BOOTSTRAP_RUN_ID}" is reserved for the ladder's seed ghosts` };
  }
  if (runId.length === 0 || runId.length > MAX_RUN_ID_LENGTH) {
    return { opened: false, reason: `runId must be 1–${MAX_RUN_ID_LENGTH} characters` };
  }
  if (deps.db.select().from(runOpens).where(eq(runOpens.runId, runId)).all().length > 0) {
    return { opened: false, reason: `run "${runId}" is already open — runIds are one-shot` };
  }
  if (deps.db.select().from(runSubmissions).where(eq(runSubmissions.runId, runId)).all().length > 0) {
    return { opened: false, reason: `run "${runId}" was already submitted — runIds are one-shot` };
  }
  deps.db
    .insert(runOpens)
    .values({ runId, userId, ghostWatermark: deps.store.maxGhostId(), openedAt: deps.clock() })
    .run();
  return { opened: true, runId };
}

/** THE play read: the round's pool as this run's owner sees it (own ghosts
 * excluded) plus the champion seated right now — and a RECORD of exactly that
 * view (run_pool_serves), which is what submission replay later holds every
 * claimed Snapshotted.seq and champion challenge to. Re-reads are free:
 * each distinct view adds a row (identical ones dedupe), and any of them
 * replays — a refresh never bricks a submission. */
export function servePool(
  deps: Pick<SubmitDeps, "db" | "store" | "clock">,
  userId: string,
  runId: string,
  round: number,
): PoolServeOutcome {
  const open = deps.db.select().from(runOpens).where(eq(runOpens.runId, runId)).all()[0];
  if (open === undefined || open.userId !== userId) {
    // One answer for "not open" and "not yours" — no probing other users' runs.
    return { served: false, reason: `run "${runId}" is not open for this user — open it first (POST /v1/runs/open)` };
  }
  if (deps.clock() - open.openedAt > RUN_OPEN_TTL_SECONDS) {
    return { served: false, reason: openExpiredReason(runId) };
  }
  // An EMPTY tower (PRD #085) has no champion yet — production launches empty
  // (createApp → openEmptyLadder) and is FOUNDED by the first completed run's
  // found-floor-1 challenge. The play read must serve that empty tower, not 500:
  // until the tower is founded there is no champion to read and the visible pool
  // is empty, but a run still plays it — a cold-start climb fights a seed-unit
  // team the kernel synthesizes off its own RNG (no served ghost needed), and the
  // founding challenge needs no seated champion. So serve gracefully with a null
  // champion rather than throwing. (Before #085 this threw, so a real empty
  // production tower 500'd its first serve and could never be founded live.)
  const champion = deps.store.championRecord();
  const pool = deps.store.poolVisibleTo(round, userId);
  // Record the view served. The serve row binds submission replay's pool-prefix
  // and champion co-serve checks; an empty tower has no champion, so its row
  // carries "" for the champion id (the dedup key still holds, and replay's
  // co-serve check only ever matches a NON-empty challenge frame).
  deps.db
    .insert(runPoolServes)
    .values({ runId, round, servedLen: pool.length, championRunId: champion?.snap.runId ?? "", servedAt: deps.clock() })
    .onConflictDoNothing()
    .run();
  return { served: true, round, pool, champion: champion?.snap ?? null };
}

function openExpiredReason(runId: string): string {
  return `run "${runId}" was opened more than ${RUN_OPEN_TTL_DAYS} days ago — opens expire; start a fresh run`;
}

/** Submit a finished run for `userId`. Returns the outcome; writes (ghosts,
 * crown, the submission row) happen only on acceptance, in one transaction. */
export function submitRun(deps: SubmitDeps, userId: string, raw: string): SubmitOutcome {
  try {
    return accept(deps, userId, replay(deps, userId, raw));
  } catch (err) {
    if (err instanceof SubmissionRejected || err instanceof InvalidDecisionError || err instanceof ValidationError) {
      return { accepted: false, reason: err.message };
    }
    throw err;
  }
}

// ---------------------------------------------------------------------------
// Replay
// ---------------------------------------------------------------------------

/** What re-derivation produces: the recomputed state plus the store effects
 * the kernel staged through the replay view (nothing written yet). */
interface Rederived {
  state: RunState;
  ghosts: TeamSnapshot[];
  crown: { snap: TeamSnapshot; challengedRunId: string | null } | null;
}

function replay(deps: SubmitDeps, userId: string, raw: string): Rederived {
  const { store, content } = deps;

  // Cost gates first — the replay below is synchronous, so nothing oversized
  // gets to spend server time on it.
  const bytes = Buffer.byteLength(raw, "utf8");
  if (bytes > MAX_RUN_BYTES) {
    throw new SubmissionRejected(`submission is ${bytes} bytes — the limit is ${MAX_RUN_BYTES}`);
  }

  let claimed: RunState;
  try {
    claimed = deserializeRun(raw); // loud structure + content gate, the kernel's own check
  } catch (err) {
    throw new SubmissionRejected(`unreadable run: ${(err as Error).message}`);
  }

  if (claimed.log.length > MAX_RUN_LOG_EVENTS) {
    throw new SubmissionRejected(`run log has ${claimed.log.length} events — the limit is ${MAX_RUN_LOG_EVENTS}`);
  }
  if (claimed.status !== "over") {
    throw new SubmissionRejected("only finished runs are submitted — this one is still active");
  }
  if (claimed.runId === BOOTSTRAP_RUN_ID) {
    throw new SubmissionRejected(`runId "${BOOTSTRAP_RUN_ID}" is reserved for the ladder's seed ghosts`);
  }
  if (deps.db.select().from(runSubmissions).where(eq(runSubmissions.runId, claimed.runId)).all().length > 0) {
    throw new SubmissionRejected(`run "${claimed.runId}" was already submitted — runIds are one-shot`);
  }
  // The open handshake's other half: the run must have been opened, by this
  // user, and inside the open's lifetime — an expired open is the banked-open
  // forgery's raw material, never a submittable run.
  const open = deps.db.select().from(runOpens).where(eq(runOpens.runId, claimed.runId)).all()[0];
  if (open === undefined) {
    throw new SubmissionRejected(`run "${claimed.runId}" was never opened — open a run before playing it`);
  }
  if (open.userId !== userId) {
    throw new SubmissionRejected(`run "${claimed.runId}" was opened by a different user`);
  }
  if (deps.clock() - open.openedAt > RUN_OPEN_TTL_SECONDS) {
    throw new SubmissionRejected(openExpiredReason(claimed.runId));
  }
  // The pool and registry travel by value in a serialized run; pin them to the
  // arena's content or the replay would verify a run against invented units.
  if (
    !isDeepStrictEqual(claimed.pool, content.pool) ||
    !isDeepStrictEqual(claimed.statuses, content.statuses) ||
    !isDeepStrictEqual(claimed.abilities, content.abilities)
  ) {
    throw new SubmissionRejected("run was not played with the arena's content (pool/statuses/abilities differ)");
  }

  // The serve record — every pool view the server handed out for this run.
  // The replay below accepts claimed views only from this set.
  const serves = new Map<number, { len: number; championRunId: string }[]>();
  for (const row of deps.db.select().from(runPoolServes).where(eq(runPoolServes.runId, claimed.runId)).all()) {
    const list = serves.get(row.round) ?? [];
    list.push({ len: row.servedLen, championRunId: row.championRunId });
    serves.set(row.round, list);
  }

  const steps = extractSteps(claimed.log);
  const view = new ReplayLadderView(store, userId, serves);
  let state = initRun({ seed: claimed.seed, runId: claimed.runId, pool: content.pool, statuses: content.statuses, abilities: content.abilities });
  for (const step of steps) {
    if (step.kind === "ladder") {
      view.frame = step;
      state = ladderFight(state, view);
    } else if (step.kind === "challengeBoss") {
      view.frame = step;
      state = challengeBoss(state, view);
    } else {
      state = applyDecision(state, step);
    }
  }

  // The whole state must match — final stats, lives, gold, and every log
  // event. A mutated stat line, a fabricated win, a wrong seed: they all
  // surface here as a divergence between claim and re-derivation.
  if (!isDeepStrictEqual(state, claimed)) {
    throw new SubmissionRejected("run does not replay to its claimed state — submission diverges from re-derivation");
  }
  return { state, ghosts: view.staged, crown: view.stagedCrown };
}

/** The decision sequence, recovered from the run log. Only ladder moves are
 * admissible on the shared ladder: a same-floor climb (ladderFight) or a boss
 * challenge (challengeBoss). An explicit-opponent fight() leaves a FightFought
 * with no ladder event before it, and is rejected by name. */
type ReplayStep =
  | { kind: "buy"; offer: number }
  | { kind: "reroll" }
  | { kind: "reorder"; from: number; to: number }
  | { kind: "ladder"; claimedSeq: number; championRunId: string | null }
  | { kind: "challengeBoss"; claimedSeq: number; championRunId: string | null };

function extractSteps(log: readonly RunEvent[]): ReplayStep[] {
  const steps: ReplayStep[] = [];
  log.forEach((e, i) => {
    switch (e.type) {
      case "Bought":
        steps.push({ kind: "buy", offer: e.offer });
        break;
      case "Rerolled":
        steps.push({ kind: "reroll" });
        break;
      case "Reordered":
        steps.push({ kind: "reorder", from: e.from, to: e.to });
        break;
      case "Snapshotted": {
        // The event after the snapshot says which ladder move the run claims:
        // an OpponentDrawn (a live ghost) OR an OpponentSynthesized (a cold-start
        // synth opponent, #085) means a same-floor climb (ladderFight, no boss
        // read, championRunId stays null); a BossChallenged means a boss challenge
        // (challengeBoss) that fought a seated boss and names it. The boss name
        // fills the same `championRunId` frame the view co-serves against — only
        // the dispatch differs (ladderFight vs challengeBoss). A synth climb
        // re-derives deterministically: the replay re-runs ladderFight against the
        // served view, which synthesizes the same team off the same RNG.
        //
        // An OVERSHOOT (challengeBoss on a vacant floor, 075-3) emits NO
        // Snapshotted — no fight, no ghost — so it never reaches this case; its
        // bare BossChallenged falls through to default and recovers no terminal
        // step, and the replay diverges from the claimed overshoot end. That is
        // the intended rejection: a vacant-floor claim is never honest here (the
        // ladder always seats a champion at the top), so it cannot be cashed in.
        const next = log[i + 1];
        if (next?.type === "BossChallenged") {
          steps.push({ kind: "challengeBoss", claimedSeq: e.seq, championRunId: next.boss });
        } else {
          steps.push({ kind: "ladder", claimedSeq: e.seq, championRunId: null });
        }
        break;
      }
      case "FightFought": {
        const prev = log[i - 1];
        // A ladder fight is preceded by the opponent's provenance: a live draw
        // (OpponentDrawn), a cold-start synthesis (OpponentSynthesized, #085), or
        // a boss challenge (BossChallenged). An explicit-opponent fight() leaves a
        // FightFought with none of these before it — that is the rejected case.
        if (prev?.type !== "OpponentDrawn" && prev?.type !== "OpponentSynthesized" && prev?.type !== "BossChallenged") {
          throw new SubmissionRejected("a shared-ladder run fights only on the ladder — explicit-opponent fight in the log");
        }
        break;
      }
      default:
        break;
    }
  });
  return steps;
}

/** The LadderStore the replay runs against: per fight, the historical view
 * the submitted log pins — the user-filtered pool truncated to the claimed
 * prefix, and the claimed champion looked up in the append-only history. A
 * claimed view is accepted only if the server SERVED it for this run: the
 * prefix length must equal a recorded serve for (runId, round), and a
 * champion challenge must name the champion co-served with that view —
 * nothing replays that the server did not itself hand out. Writes are
 * staged, never stored: the store mutates only on acceptance. */
class ReplayLadderView implements LadderStore {
  private _frame: { claimedSeq: number; championRunId: string | null } | null = null;
  readonly staged: TeamSnapshot[] = [];
  stagedCrown: { snap: TeamSnapshot; challengedRunId: string | null } | null = null;
  /** The round of the in-flight fight — set by poolAt, which the kernel
   * always calls before champion() inside a ladder fight. */
  private round = 0;
  /** The floor this step's serve-check was satisfied at — the floor the run
   * actually drew from (a climb) or fought the boss at (a challenge). An ascend
   * crown reads ONE MORE pool — floor f+1, to seq the new champion's pool-ghost —
   * but that floor is bookkeeping, never a play-time view; the serve-check binds
   * the served floor only, and the f+1 read returns the live (append-only) pool
   * un-gated (its seq is re-assigned at accept anyway). null until the served
   * read happens, reset per step when `frame` is set. */
  private servedFloor: number | null = null;

  constructor(
    private readonly store: SqliteLadderStore,
    private readonly userId: string,
    private readonly serves: ReadonlyMap<number, { len: number; championRunId: string }[]>,
  ) {}

  /** Set the in-flight step. Resets the per-step served-floor latch so each
   * step's serve-check binds its own floor. */
  set frame(f: { claimedSeq: number; championRunId: string | null } | null) {
    this._frame = f;
    this.servedFloor = null;
  }

  poolAt(round: number): readonly TeamSnapshot[] {
    const frame = this.mustFrame();
    this.round = round;
    // An ascend crown's f+1 pool read (after the served floor is already
    // latched, on a higher floor) is kernel bookkeeping to seq the new
    // champion's pool-ghost, not a play-time draw. Serve it the live pool
    // un-gated — accept re-sequences the staged ghost regardless.
    if (this.servedFloor !== null && round !== this.servedFloor) {
      return this.store.poolVisibleTo(round, this.userId);
    }
    const served = this.serves.get(round) ?? [];
    if (!served.some((s) => s.len === frame.claimedSeq)) {
      throw new SubmissionRejected(
        `run claims a round-${round} pool of ${frame.claimedSeq} ghosts — a view this server never served for ` +
          `this run (play reads go through GET /v1/runs/:runId/pool/:round)`,
      );
    }
    this.servedFloor = round;
    // Pools are append-only and every serve precedes its submission, so the
    // served length never exceeds the current pool; were the slice ever to
    // come up short anyway, the re-derived Snapshotted.seq would diverge.
    return this.store.poolVisibleTo(round, this.userId).slice(0, frame.claimedSeq);
  }

  addSnapshot(snap: TeamSnapshot): void {
    this.staged.push(snap);
  }

  champion(): TeamSnapshot | null {
    const frame = this.mustFrame();
    if (frame.championRunId === null) {
      // The run claims no challenge happened (or a vacant-spot crown). Serve
      // the seated champion; if the claim was wrong, the replay diverges.
      return this.store.championRecord()?.snap ?? null;
    }
    const rec = this.store.championByRunId(frame.championRunId);
    if (rec === null) {
      throw new SubmissionRejected(`run claims a champion "${frame.championRunId}" this ladder never seated`);
    }
    // Temporal coherence: the challenge replays only against the champion
    // CO-SERVED with the claimed pool view. A banked view from the past does
    // not buy a challenge against whoever is seated today.
    const served = this.serves.get(this.round) ?? [];
    if (!served.some((s) => s.len === frame.claimedSeq && s.championRunId === frame.championRunId)) {
      throw new SubmissionRejected(
        `run challenges champion "${frame.championRunId}" from a round-${this.round} view this server never ` +
          `served with that champion seated`,
      );
    }
    return rec.snap;
  }

  bossAt(floor: number): TeamSnapshot | null {
    // challengeBoss reads the boss BEFORE poolAt, so set the in-flight round
    // here from the queried floor (= s.round): champion()'s co-served check
    // reads this.round, and a stale round would mis-key the serve lookup.
    this.round = floor;
    // Replay reads the summit through champion(); a per-floor read only ever
    // resolves the floor that is the seated champion's, vacant otherwise.
    const champ = this.champion();
    return champ !== null && champ.round === floor ? champ : null;
  }

  setBoss(floor: number, snap: TeamSnapshot): void {
    // challengeBoss seats the run's ghost as a new summit (snap.round === floor)
    // on a won challenge: stage the crown — acceptance decides. A lost challenge
    // never reaches here, so stagedCrown stays null and no crown is applied.
    this.stagedCrown = { snap, challengedRunId: this.mustFrame().championRunId };
  }

  private mustFrame(): { claimedSeq: number; championRunId: string | null } {
    if (this._frame === null) throw new SubmissionRejected("ladder access outside a ladder fight — malformed run log");
    return this._frame;
  }
}

// ---------------------------------------------------------------------------
// Acceptance
// ---------------------------------------------------------------------------

function accept(deps: SubmitDeps, userId: string, rederived: Rederived): SubmitOutcome {
  const { db, store } = deps;
  const { state, ghosts, crown } = rederived;

  let crowned = false;
  db.transaction(() => {
    // Re-sequence each re-derived ghost onto the end of its current pool: the
    // historical seq already did its job (pinning the draw the run replayed
    // against); insertion order in the shared pool is the server's to assign.
    const storedSeq = new Map<TeamSnapshot, number>();
    for (const ghost of ghosts) {
      const seq = store.poolLength(ghost.round);
      store.addGhost({ ...ghost, seq }, userId);
      storedSeq.set(ghost, seq);
    }
    if (crown !== null) {
      const seated = store.championRecord();
      const challengedStillSeated =
        seated === null ? crown.challengedRunId === null : seated.snap.runId === crown.challengedRunId;
      if (challengedStillSeated) {
        // The crown ghost is one of the staged snapshots — seat it under its
        // re-assigned seq so the champion row matches the stored ghost.
        const seq = storedSeq.get(crown.snap) ?? crown.snap.seq;
        store.setChampionFor({ ...crown.snap, seq }, userId);
        crowned = true;
      }
      // Otherwise: the run legally beat a champion that has since been
      // dethroned — ghosts stand, the crown lapses (first submission wins).
    }
    db.insert(runSubmissions)
      .values({
        runId: state.runId,
        userId,
        seed: state.seed,
        endedBy: state.endedBy!,
        finalRound: state.round,
        submittedAt: deps.clock(),
      })
      .run();
  });

  return { accepted: true, runId: state.runId, endedBy: state.endedBy!, finalRound: state.round, crowned };
}
