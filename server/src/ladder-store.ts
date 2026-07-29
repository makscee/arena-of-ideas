/**
 * The SQLite backing of the kernel's LadderStore interface — the shared
 * ladder's storage. Lives in server land (it needs the DB) but holds the same
 * semantics the kernel's backings pin: pools are append-only in seq order,
 * addSnapshot rejects a desynced seq, stored snapshots are isolated from
 * later caller mutation (rows ARE copies), and the champion spot persists.
 *
 * On top of the kernel interface it carries the server-only dimension the
 * shared ladder needs: each ghost/champion row records the OWNING USER of the
 * run that fielded it (null for bootstrap), so a user's ghosts — spanning all
 * their runs — can be excluded from that user's own draws. Champion history
 * is append-only (current = latest row) so run re-derivation can replay a
 * challenge against the champion that was actually seated when the run fought.
 */
import { and, asc, desc, eq, max } from "drizzle-orm";
import { assertBossSeat, assertSeqInOrder } from "../../src/ladder.js";
import type { LadderStore, TeamSnapshot } from "../../src/index.js";
import type { UnitDef } from "../../src/index.js";
import type { DB } from "./db.js";
import { ladderChampions, ladderGhosts } from "./schema.js";

/** A champion row hydrated: the snapshot plus who owns it. */
export interface ChampionRecord {
  snap: TeamSnapshot;
  userId: string | null;
}

export class SqliteLadderStore implements LadderStore {
  private readonly db: DB;

  constructor(db: DB) {
    this.db = db;
  }

  // ---- the kernel interface (userId-less callers are system writes) ----

  poolAt(round: number): readonly TeamSnapshot[] {
    return this.poolVisibleTo(round, null);
  }

  addSnapshot(snap: TeamSnapshot): void {
    this.addGhost(snap, null);
  }

  champion(): TeamSnapshot | null {
    return this.championRecord()?.snap ?? null;
  }

  /** The latest seat written on this floor. ladder_champions is append-only
   * seat history: retaining prior rows preserves replay while the latest row
   * per floor is the live boss. */
  bossAt(floor: number): TeamSnapshot | null {
    return this.bossRecord(floor)?.snap ?? null;
  }

  setBoss(floor: number, snap: TeamSnapshot): void {
    assertBossSeat(floor, snap);
    this.setBossFor(snap, null);
  }

  // ---- the server-side dimension: ghost ownership ----

  /** The round-R pool in seq order; with `excludeUserId`, the pool as that
   * user's runs see it — every ghost the user owns filtered out, bootstrap
   * and other users' ghosts untouched. The filtered view keeps true seq
   * values and is itself append-only over time (ghosts are never removed),
   * which is what makes historical-prefix replay of a submitted run sound. */
  poolVisibleTo(round: number, excludeUserId: string | null): TeamSnapshot[] {
    const rows = this.db
      .select()
      .from(ladderGhosts)
      .where(eq(ladderGhosts.round, round))
      .orderBy(asc(ladderGhosts.seq))
      .all();
    return rows
      .filter((r) => excludeUserId === null || r.userId === null || r.userId !== excludeUserId)
      .map((r) => ({ runId: r.runId, round: r.round, seq: r.seq, team: JSON.parse(r.team) as UnitDef[] }));
  }

  /** Append a ghost owned by `userId` (null = system/bootstrap). The seq
   * precondition is the kernel's: seq must be the FULL pool's length. */
  addGhost(snap: TeamSnapshot, userId: string | null): void {
    assertSeqInOrder(snap, this.poolLength(snap.round));
    this.db
      .insert(ladderGhosts)
      .values({ round: snap.round, seq: snap.seq, runId: snap.runId, userId, team: JSON.stringify(snap.team) })
      .run();
  }

  /** The full (unfiltered) length of a round's pool — the next seq. */
  poolLength(round: number): number {
    return this.db.select().from(ladderGhosts).where(eq(ladderGhosts.round, round)).all().length;
  }

  /** The highest ladder_ghosts row id (0 on an unghosted ladder) — recorded
   * on each run open as provenance of the ladder state the run began against.
   * Replay itself checks the serve record (run_pool_serves), which is
   * strictly stronger than any watermark-derived floor. */
  maxGhostId(): number {
    return this.db.select({ m: max(ladderGhosts.id) }).from(ladderGhosts).all()[0]?.m ?? 0;
  }

  /** The deepest round any ghost sits at (0 on an unghosted ladder) — the
   * ladder's frontier. Honest play cannot read far past it: a round with no
   * visible ghosts is the kernel's outran-every-ghost champion challenge, and
   * the run ends there. The serve route uses this to bound which rounds it
   * will record views for. */
  maxPoolRound(): number {
    return this.db.select({ m: max(ladderGhosts.round) }).from(ladderGhosts).all()[0]?.m ?? 0;
  }

  /** Every live floor seat keyed by floor. Append order resolves overwrites. */
  bossRecords(): Record<string, ChampionRecord> {
    const records: Record<string, ChampionRecord> = {};
    for (const row of this.db.select().from(ladderChampions).orderBy(asc(ladderChampions.id)).all()) {
      records[String(row.round)] = rowToChampion(row);
    }
    return records;
  }

  /** The live boss on one floor, owner included: latest append wins there. */
  bossRecord(floor: number): ChampionRecord | null {
    const row = this.db
      .select()
      .from(ladderChampions)
      .where(eq(ladderChampions.round, floor))
      .orderBy(desc(ladderChampions.id))
      .limit(1)
      .all()[0];
    return row ? rowToChampion(row) : null;
  }

  /** The champion is the boss of the highest occupied floor. Within that
   * floor, the latest append is the current seat after any overwrite. */
  championRecord(): ChampionRecord | null {
    const row = this.db
      .select()
      .from(ladderChampions)
      .orderBy(desc(ladderChampions.round), desc(ladderChampions.id))
      .limit(1)
      .all()[0];
    return row ? rowToChampion(row) : null;
  }

  /** A historical boss identified by both run and floor. The floor matters
   * for bootstrap, whose reserved run id seeds more than one seat. */
  bossByRunId(runId: string, floor: number): ChampionRecord | null {
    const row = this.db
      .select()
      .from(ladderChampions)
      .where(and(eq(ladderChampions.runId, runId), eq(ladderChampions.round, floor)))
      .orderBy(desc(ladderChampions.id))
      .limit(1)
      .all()[0];
    return row ? rowToChampion(row) : null;
  }

  /** The latest summit history row under `runId`. */
  championByRunId(runId: string): ChampionRecord | null {
    const row = this.db
      .select()
      .from(ladderChampions)
      .where(eq(ladderChampions.runId, runId))
      .orderBy(desc(ladderChampions.id))
      .limit(1)
      .all()[0];
    return row ? rowToChampion(row) : null;
  }

  /** Seat a floor boss owned by `userId` — appends to seat history. */
  setBossFor(snap: TeamSnapshot, userId: string | null): void {
    this.db
      .insert(ladderChampions)
      .values({ runId: snap.runId, userId, round: snap.round, seq: snap.seq, team: JSON.stringify(snap.team) })
      .run();
  }

  /** Compatibility name for server call sites that apply a summit win. */
  setChampionFor(snap: TeamSnapshot, userId: string | null): void {
    this.setBossFor(snap, userId);
  }
}

function rowToChampion(row: { runId: string; userId: string | null; round: number; seq: number; team: string }): ChampionRecord {
  return {
    snap: { runId: row.runId, round: row.round, seq: row.seq, team: JSON.parse(row.team) as UnitDef[] },
    userId: row.userId,
  };
}
