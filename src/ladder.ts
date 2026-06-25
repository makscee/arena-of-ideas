// The ladder — who a run fights, built out of played runs themselves.
// A Ladder holds, per round number, a pool of team snapshots (ghosts): when a
// run fights at round R its fielded team is first snapshotted into the round-R
// pool, so every fight a run plays leaves an opponent behind for future runs —
// ghosts persist even if the run later dies. Above the pools sits the tower: a
// boss seated per floor (a floor = the round a team was fielded at, so all
// teams on it have comparable power). The champion is no longer a stored spot
// of its own — it is *derived* as the boss of the highest occupied floor, the
// summit of the tower; it persists across runs because the boss it reads does.
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

import { BOSS_TEAMS, BOOTSTRAP_DEPTH, BOOTSTRAP_TEAMS } from "./tunables.js";
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
  /** The boss seated on `floor`, or null while that floor is vacant. */
  bossAt(floor: number): TeamSnapshot | null;
  /** Seat a boss on `floor`. A dethroned boss is not removed from anything —
   * its ghosts stay in their pools. */
  setBoss(floor: number, snap: TeamSnapshot): void;
  /** The current champion: the boss of the highest occupied floor, or null
   * while no floor is occupied. Derived from the boss map, not stored — every
   * backing computes it the same way (deriveChampion), so they never disagree. */
  champion(): TeamSnapshot | null;
}

/** The in-memory backing — tests now, the browser client in slice 4.
 * Snapshots are JSON-cloned on write so a stored ghost is exactly what the
 * file backing would round-trip: isolated from later caller mutation, and
 * byte-equivalent across backings. */
export class InMemoryLadderStore implements LadderStore {
  private pools = new Map<number, TeamSnapshot[]>();
  private bosses = new Map<number, TeamSnapshot>();

  poolAt(round: number): readonly TeamSnapshot[] {
    return this.pools.get(round) ?? [];
  }

  addSnapshot(snap: TeamSnapshot): void {
    const pool = this.pools.get(snap.round) ?? [];
    assertSeqInOrder(snap, pool.length);
    pool.push(jsonClone(snap));
    this.pools.set(snap.round, pool);
  }

  bossAt(floor: number): TeamSnapshot | null {
    return this.bosses.get(floor) ?? null;
  }

  setBoss(floor: number, snap: TeamSnapshot): void {
    this.bosses.set(floor, jsonClone(snap));
  }

  champion(): TeamSnapshot | null {
    return deriveChampion(Object.fromEntries(this.bosses));
  }
}

/** The serialized ladder — the one shape every persistent backing stores:
 * pools keyed by round number, plus the tower's bosses keyed by floor (both
 * Records because JSON keys are strings). The champion is not stored — it is
 * derived from `bosses` on read (deriveChampion), so a backing carries only
 * the per-floor seats, not a redundant summit that could drift from them.
 * FileLadderStore (ladder-file.ts) writes it to disk; the web client writes
 * it to localStorage; both round-trip byte-equivalent ladders. */
export interface LadderData {
  bosses: Record<string, TeamSnapshot>;
  pools: Record<string, TeamSnapshot[]>;
}

/** A fresh, empty ladder — what a backing starts from when nothing is stored. */
export function emptyLadderData(): LadderData {
  return { bosses: {}, pools: {} };
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
  return { bosses: data.bosses ?? {}, pools: data.pools };
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

  bossAt(floor: number): TeamSnapshot | null {
    return this.data.bosses[String(floor)] ?? null;
  }

  setBoss(floor: number, snap: TeamSnapshot): void {
    // Clone on write, like addSnapshot: holding the caller's object by
    // reference would let a later mutation corrupt the seated boss.
    this.data.bosses[String(floor)] = jsonClone(snap);
    this.persist(this.data);
  }

  champion(): TeamSnapshot | null {
    return deriveChampion(this.data.bosses);
  }
}

/** The ladder's id for ghosts that came from no run (the bootstrap seed). */
export const BOOTSTRAP_RUN_ID = "bootstrap";

/** Open a ladder over a store, seeding the whole bootstrap tower if the ladder
 * is empty so a first-ever run has a real climb AND a seated boss to beat — never
 * a round-2 crown, never a free summit taken from a vacant spot. The tower has
 * two parts, sized off BOOTSTRAP_DEPTH:
 *
 *   • Climb floors 1..BOOTSTRAP_DEPTH. Floor f gets its climb pool
 *     (BOOTSTRAP_TEAMS[f-1]) AND a seated boss (BOSS_TEAMS[f-1]). The boss is
 *     ALSO left in floor f's pool as a ghost (next seq, after the climb teams),
 *     mirroring a run-won boss (snapshot-before-fight, then seat the same
 *     snapshot): that pool-ghost is what makes the "demote keeps the unseated
 *     team in the pool" invariant hold for a SEEDED boss too — dethrone floor f's
 *     boss and its team is still drawable as a climb ghost on f. (Sharing a
 *     floor's pool between climb ghosts and the boss-ghost obeys assertSeqInOrder:
 *     the boss-ghost is just the pool's next entry, seq = climb-team count.)
 *
 *   • The summit, floor BOOTSTRAP_DEPTH+1 (BOSS_TEAMS[BOOTSTRAP_DEPTH]). It is
 *     the derived champion. It is seated WITHOUT a pool-ghost, and this is
 *     load-bearing, not an oversight: a run advances a floor on EVERY climb,
 *     win OR loss (a climb loss costs a life but still moves up). So a run sails
 *     up floors 1..BOOTSTRAP_DEPTH regardless of record and lands on the summit
 *     floor as its terminal floor — and the climb there must REJECT (no drawable
 *     ghost) so the only move is to challengeBoss the summit. Had the summit a
 *     pool-ghost, that ghost would be a drawable climb opponent: the run would
 *     "climb" the summit and advance to a vacant floor BOOTSTRAP_DEPTH+2, taking
 *     a free crown there — exactly the trivial round-2 crown this bootstrap
 *     exists to kill (075-2's sweeps: every run crowned, many with losing
 *     records). The summit is the guard; a guard with a climbable ghost is no
 *     guard. The lower bosses keep their pool-ghosts because they are never the
 *     run's terminal floor on a climb (it always passes through them upward), so
 *     their ghost can't leak a free crown — only the summit's could.
 *
 * The champion is DERIVED, never seated as its own concept: floor
 * BOOTSTRAP_DEPTH+1 is the highest occupied floor, so its boss is the summit
 * (the old single BOOTSTRAP_CHAMPION, now BOSS_TEAMS' top entry). The tower grows
 * open-endedly ABOVE the summit through real play — a run that out-climbs to a
 * vacant floor auto-seats there (challengeBoss' kept edge) — so the summit
 * advances without openLadder ever pre-seeding past BOOTSTRAP_DEPTH+1.
 *
 * (A ladder any run has played on cannot have an empty floor-1 pool: snapshot-
 * before-fight put the run's own ghost there — so the poolAt(1) guard reseeds
 * only a truly fresh ladder, never a played-on one.) Every seeded team — climb
 * ghost and boss alike — passes the content gate against `registry` here, at
 * seed time, so a bad team fails loudly at open, never seed-dependently mid-run
 * when a draw or a challenge happens to land on it. */
export function openLadder(store: LadderStore, registry: StatusRegistry): LadderStore {
  if (store.poolAt(1).length === 0) {
    // Climb floors 1..BOOTSTRAP_DEPTH: a climb pool + a seated boss that ALSO
    // lives in the pool as a ghost (the demote-keeps-ghost invariant).
    for (let i = 0; i < BOOTSTRAP_DEPTH; i++) {
      const floor = i + 1;
      const climb = BOOTSTRAP_TEAMS[i] ?? [];
      climb.forEach((team, seq) => {
        assertValidContent(team, registry, `bootstrap round ${floor} team ${seq}`);
        store.addSnapshot({ runId: BOOTSTRAP_RUN_ID, round: floor, seq, team: jsonClone(team) });
      });
      const bossTeam = BOSS_TEAMS[i]!;
      assertValidContent(bossTeam, registry, `bootstrap floor ${floor} boss`);
      const boss: TeamSnapshot = { runId: BOOTSTRAP_RUN_ID, round: floor, seq: climb.length, team: jsonClone([...bossTeam]) };
      store.addSnapshot(boss); // boss-ghost in the pool — demote leaves it drawable
      store.setBoss(floor, boss);
    }
    // The summit, floor BOOTSTRAP_DEPTH+1: seated as the guard with NO pool-ghost,
    // so a run that sailed up the climb floors must CHALLENGE it (an empty climb
    // pool rejects) — the crown is earned, never taken from a vacant floor.
    const summitFloor = BOOTSTRAP_DEPTH + 1;
    const summitTeam = BOSS_TEAMS[BOOTSTRAP_DEPTH]!;
    assertValidContent(summitTeam, registry, `bootstrap floor ${summitFloor} boss (summit)`);
    store.setBoss(summitFloor, { runId: BOOTSTRAP_RUN_ID, round: summitFloor, seq: 0, team: jsonClone([...summitTeam]) });
  }
  return store;
}

/** The champion, derived once for every backing: the boss of the highest
 * occupied floor, or null when no floor is occupied. Floor keys are strings
 * (JSON keys), so compare them as numbers — the summit is the max floor that
 * has a seated boss. Sharing this means InMemory, file, and localStorage
 * backings can never disagree on who the champion is. */
export function deriveChampion(bosses: Record<string, TeamSnapshot>): TeamSnapshot | null {
  let topFloor = -Infinity;
  let champ: TeamSnapshot | null = null;
  for (const key of Object.keys(bosses)) {
    const floor = Number(key);
    if (floor > topFloor) {
      topFloor = floor;
      champ = bosses[key]!;
    }
  }
  return champ;
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
