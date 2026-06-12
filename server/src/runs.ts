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
 * pool length it drew from), and each ChampionChallenged event names the
 * champion seated at the time, which the append-only champion history can
 * still produce. Replaying against those historical views reproduces the run
 * byte-for-byte — or doesn't, and the submission is rejected with the reason.
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
import { runSubmissions } from "./schema.js";

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

/** A submission failing re-derivation — carries the reason the client sees. */
class SubmissionRejected extends Error {}

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

  let claimed: RunState;
  try {
    claimed = deserializeRun(raw); // loud structure + content gate, the kernel's own check
  } catch (err) {
    throw new SubmissionRejected(`unreadable run: ${(err as Error).message}`);
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
  // The pool and registry travel by value in a serialized run; pin them to the
  // arena's content or the replay would verify a run against invented units.
  if (!isDeepStrictEqual(claimed.pool, content.pool) || !isDeepStrictEqual(claimed.statuses, content.statuses)) {
    throw new SubmissionRejected("run was not played with the arena's content (pool/statuses differ)");
  }

  const steps = extractSteps(claimed.log);
  const view = new ReplayLadderView(store, userId);
  let state = initRun({ seed: claimed.seed, runId: claimed.runId, pool: content.pool, statuses: content.statuses });
  for (const step of steps) {
    if (step.kind === "ladder") {
      view.frame = step;
      state = ladderFight(state, view);
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

/** The decision sequence, recovered from the run log. Only ladder fights are
 * admissible on the shared ladder: an explicit-opponent fight() leaves a
 * FightFought with no draw before it, and is rejected by name. */
type ReplayStep =
  | { kind: "buy"; offer: number }
  | { kind: "reroll" }
  | { kind: "reorder"; from: number; to: number }
  | { kind: "ladder"; claimedSeq: number; championRunId: string | null };

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
        // The events after the snapshot say what kind of fight the run claims:
        // a pool draw (championRunId stays null), a champion challenge, or a
        // vacant-spot crown (dethroned null — never legal here: this ladder
        // seats a bootstrap champion, so the replay's divergence rejects it).
        const next = log[i + 1];
        const championRunId =
          next?.type === "ChampionChallenged" ? next.champion : next?.type === "Crowned" ? next.dethroned : null;
        steps.push({ kind: "ladder", claimedSeq: e.seq, championRunId });
        break;
      }
      case "FightFought": {
        const prev = log[i - 1];
        if (prev?.type !== "OpponentDrawn" && prev?.type !== "ChampionChallenged") {
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
 * prefix, and the claimed champion looked up in the append-only history.
 * Writes are staged, never stored: the store mutates only on acceptance. */
class ReplayLadderView implements LadderStore {
  frame: { claimedSeq: number; championRunId: string | null } | null = null;
  readonly staged: TeamSnapshot[] = [];
  stagedCrown: { snap: TeamSnapshot; challengedRunId: string | null } | null = null;

  constructor(
    private readonly store: SqliteLadderStore,
    private readonly userId: string,
  ) {}

  poolAt(round: number): readonly TeamSnapshot[] {
    const frame = this.mustFrame();
    const visible = this.store.poolVisibleTo(round, this.userId);
    if (visible.length < frame.claimedSeq) {
      throw new SubmissionRejected(
        `run claims a round-${round} pool of ${frame.claimedSeq} ghosts; this ladder has ${visible.length}`,
      );
    }
    return visible.slice(0, frame.claimedSeq);
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
    return rec.snap;
  }

  setChampion(snap: TeamSnapshot): void {
    this.stagedCrown = { snap, challengedRunId: this.mustFrame().championRunId };
  }

  private mustFrame(): { claimedSeq: number; championRunId: string | null } {
    if (this.frame === null) throw new SubmissionRejected("ladder access outside a ladder fight — malformed run log");
    return this.frame;
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
