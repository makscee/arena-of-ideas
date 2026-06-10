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

import { BOOTSTRAP_DEPTH, BOOTSTRAP_TEAMS } from "./tunables.js";
import { assertValidContent } from "./validate.js";
import type { StatusRegistry, UnitDef } from "./types.js";

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
  /** Append a ghost to its round's pool. snap.seq must be poolAt(round).length —
   * a backing throws on desync, because a wrong seq means the caller drew from
   * a pool other than the one it is writing to. */
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
    const pool = this.pools.get(snap.round) ?? [];
    assertSeqInOrder(snap, pool.length);
    pool.push(jsonClone(snap));
    this.pools.set(snap.round, pool);
  }

  champion(): TeamSnapshot | null {
    return this.champ;
  }

  setChampion(snap: TeamSnapshot): void {
    this.champ = jsonClone(snap);
  }
}

/** The serialized ladder — the one shape every persistent backing stores:
 * pools keyed by round number (JSON keys are strings) plus the champion spot.
 * FileLadderStore (ladder-file.ts) writes it to disk; the web client writes
 * it to localStorage; both round-trip byte-equivalent ladders. */
export interface LadderData {
  champion: TeamSnapshot | null;
  pools: Record<string, TeamSnapshot[]>;
}

/** A fresh, empty ladder — what a backing starts from when nothing is stored. */
export function emptyLadderData(): LadderData {
  return { champion: null, pools: {} };
}

/** Parse stored ladder JSON, loudly: a present-but-unreadable ladder must
 * never silently become a fresh one — that would orphan every ghost in it.
 * `label` names the backing in the error (a file path, a storage key). */
export function parseLadderData(raw: string, label: string): LadderData {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (err) {
    throw new Error(`Ladder ${label} is not valid JSON: ${(err as Error).message}`);
  }
  const data = parsed as Partial<LadderData>;
  if (typeof data !== "object" || data === null || typeof data.pools !== "object" || data.pools === null) {
    throw new Error(`Ladder ${label} has no pools object — not a ladder`);
  }
  return { champion: data.champion ?? null, pools: data.pools };
}

/** A LadderStore over a LadderData record with a write-through persist hook —
 * the shared engine of every persistent backing (file, localStorage): same
 * clone-on-write isolation as InMemoryLadderStore, plus `persist(data)` after
 * every mutation. The hook owns serialization and the medium; this class owns
 * the LadderStore semantics, so backings can never disagree on them. */
export class PersistedLadderStore implements LadderStore {
  private readonly data: LadderData;
  private readonly persist: (data: LadderData) => void;

  constructor(data: LadderData, persist: (data: LadderData) => void) {
    this.data = data;
    this.persist = persist;
  }

  poolAt(round: number): readonly TeamSnapshot[] {
    return this.data.pools[String(round)] ?? [];
  }

  addSnapshot(snap: TeamSnapshot): void {
    const pool = (this.data.pools[String(snap.round)] ??= []);
    assertSeqInOrder(snap, pool.length);
    // Clone on write, like InMemoryLadderStore: holding the caller's object by
    // reference would let a later mutation corrupt the stored ghost and the
    // next persist would write the corruption through.
    pool.push(jsonClone(snap));
    this.persist(this.data);
  }

  champion(): TeamSnapshot | null {
    return this.data.champion;
  }

  setChampion(snap: TeamSnapshot): void {
    this.data.champion = jsonClone(snap);
    this.persist(this.data);
  }
}

/** The ladder's id for ghosts that came from no run (the bootstrap seed). */
export const BOOTSTRAP_RUN_ID = "bootstrap";

/** Open a ladder over a store, seeding rounds 1..BOOTSTRAP_DEPTH from the
 * shipped bootstrap teams if the ladder is empty — a first-ever run has a
 * real climb, not a round-2 crown. (A ladder any run has played on cannot
 * have an empty round-1 pool: snapshot-before-fight put the run's own ghost
 * there.) Every bootstrap team passes the content gate against `registry`
 * here, at seed time — a bad team fails loudly at open, never seed-dependently
 * mid-run when a draw happens to land on it. */
export function openLadder(store: LadderStore, registry: StatusRegistry): LadderStore {
  if (store.poolAt(1).length === 0) {
    BOOTSTRAP_TEAMS.slice(0, BOOTSTRAP_DEPTH).forEach((teams, i) => {
      const round = i + 1;
      teams.forEach((team, seq) => {
        assertValidContent(team, registry, `bootstrap round ${round} team ${seq}`);
        store.addSnapshot({ runId: BOOTSTRAP_RUN_ID, round, seq, team: jsonClone(team) });
      });
    });
  }
  return store;
}

/** The seq precondition, enforced (LadderStore.addSnapshot) — shared by both backings. */
export function assertSeqInOrder(snap: TeamSnapshot, poolLength: number): void {
  if (snap.seq !== poolLength) {
    throw new Error(
      `snapshot seq ${snap.seq} desyncs from the round-${snap.round} pool (next seq is ${poolLength})`,
    );
  }
}

/** The clone every write path shares — JSON-safe data in, isolated copy out.
 * Both backings clone on write so a stored ghost is exactly what the file
 * backing round-trips: isolated from later caller mutation, byte-equivalent
 * across backings. */
export function jsonClone<T>(v: T): T {
  return JSON.parse(JSON.stringify(v)) as T;
}
