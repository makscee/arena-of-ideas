// The ladder — who a run fights, built out of played runs themselves.
// A Ladder holds, per round number, a pool of team snapshots (ghosts): when a
// run fights at round R its fielded team is first snapshotted into the round-R
// pool, so every fight a run plays leaves an opponent behind for future runs —
// ghosts persist even if the run later dies. At the top sits the champion
// spot, which persists across runs until a run dethrones it.
//
// This module owns the storage boundary: the LadderStore interface and its
// in-memory backing (tests, browser). The file backing lives in
// ladder-file.ts so this module — and index.ts, which the browser imports —
// stays free of node built-ins. The ladder *logic* (opponent draw, champion
// rule) lives in run.ts as ladderFight() and depends only on the interface.
//
// No wall-clock reads anywhere in the kernel: snapshot ordering is the seq
// ordinal (insertion order within a pool); a real timestamp, if a client ever
// wants one, comes in as data.

import { BOOTSTRAP_TEAMS } from "./tunables.js";
import type { UnitDef } from "./types.js";

/** A ghost: a fielded team frozen into a round's pool, plus where it came from. */
export interface TeamSnapshot {
  /** The run that fielded this team — own ghosts are excluded from its draws. */
  runId: string;
  /** The round the team was fielded at (= the pool it lives in). */
  round: number;
  /** Insertion ordinal within its pool — createdAt-style ordering without a clock. */
  seq: number;
  team: UnitDef[];
}

/** The storage boundary the ladder logic depends on — nothing else.
 * Backings: InMemoryLadderStore (below), FileLadderStore (ladder-file.ts).
 * Returned snapshots are owned by the store: treat them as immutable. */
export interface LadderStore {
  /** The round-R pool in insertion (seq) order; [] for an untouched round. */
  poolAt(round: number): readonly TeamSnapshot[];
  /** Append a ghost to its round's pool (snap.seq must be poolAt(round).length). */
  addSnapshot(snap: TeamSnapshot): void;
  /** The current champion team, or null while the spot is vacant. */
  champion(): TeamSnapshot | null;
  /** Crown a new champion. The dethroned team is not removed from anything —
   * its ghosts stay in their pools. */
  setChampion(snap: TeamSnapshot): void;
}

/** The in-memory backing — tests now, the browser client in slice 4.
 * Snapshots are JSON-cloned on write so a stored ghost is exactly what the
 * file backing would round-trip: isolated from later caller mutation, and
 * byte-equivalent across backings. */
export class InMemoryLadderStore implements LadderStore {
  private pools = new Map<number, TeamSnapshot[]>();
  private champ: TeamSnapshot | null = null;

  poolAt(round: number): readonly TeamSnapshot[] {
    return this.pools.get(round) ?? [];
  }

  addSnapshot(snap: TeamSnapshot): void {
    const pool = this.pools.get(snap.round);
    if (pool === undefined) this.pools.set(snap.round, [jsonClone(snap)]);
    else pool.push(jsonClone(snap));
  }

  champion(): TeamSnapshot | null {
    return this.champ;
  }

  setChampion(snap: TeamSnapshot): void {
    this.champ = jsonClone(snap);
  }
}

/** The ladder's id for ghosts that came from no run (the bootstrap seed). */
export const BOOTSTRAP_RUN_ID = "bootstrap";

/** Open a ladder over a store, seeding round 1 from the shipped bootstrap
 * teams if the ladder is empty — a first-ever run always has opponents.
 * (A ladder any run has played on cannot have an empty round-1 pool:
 * snapshot-before-fight put the run's own ghost there.) */
export function openLadder(store: LadderStore): LadderStore {
  if (store.poolAt(1).length === 0) {
    BOOTSTRAP_TEAMS.forEach((team, seq) => {
      store.addSnapshot({ runId: BOOTSTRAP_RUN_ID, round: 1, seq, team: jsonClone(team) as UnitDef[] });
    });
  }
  return store;
}

/** The clone every write path shares — JSON-safe data in, isolated copy out. */
function jsonClone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v)) as T;
}
