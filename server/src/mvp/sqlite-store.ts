// The MVP's SQLite store (mission #574, slice 4): MvpStore on better-sqlite3,
// passed to mvpRuntime by main.ts. It passes the same contract as the memory
// store (./store.contract.ts).
//
// Migrations: every file in ./sql/, in filename order (<slice>-<name>.sql,
// e.g. 04-runs.sql), runs once and is recorded by name in mvp_migrations. A
// slice that needs a new table or column adds its own new file; it never
// edits one that has shipped, since a recorded file doesn't run again.
import Database from "better-sqlite3";
import { readdirSync, readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import type { BattleRecord, Champion, DayState, FightKind, FusionDiscovery, Ghost, PlayerRef, PlayoffResult, Rating, Slay, UnitId } from "../../../src/mvp/contract.js";
import type { MvpRunState } from "../../../src/mvp/run.js";
import { MAX_SESSIONS, NO_IDEAS, nameKey, type DayTallies, type IdeaCounts, type Invite, type MvpStore, type PoolSnapshot, type PoolStint, type StoredUnit, type UnitDayTally, type UnitStatus, type UnitTallies, type UnitTally } from "./store.js";

const SQL_DIR = fileURLToPath(new URL("./sql/", import.meta.url));

/** Applies the ./sql/ files not yet recorded, each in its own transaction.
 * Returns the names it applied. */
export function migrateMvp(db: Database.Database, dir = SQL_DIR): string[] {
  db.exec("CREATE TABLE IF NOT EXISTS mvp_migrations (name TEXT PRIMARY KEY, applied_at TEXT NOT NULL)");
  const done = new Set((db.prepare("SELECT name FROM mvp_migrations").all() as { name: string }[]).map((r) => r.name));
  const applied: string[] = [];
  for (const name of readdirSync(dir).filter((f) => f.endsWith(".sql")).sort()) {
    if (done.has(name)) continue;
    const sql = readFileSync(`${dir}/${name}`, "utf8");
    db.transaction(() => {
      db.exec(sql);
      db.prepare("INSERT INTO mvp_migrations (name, applied_at) VALUES (?, ?)").run(name, new Date().toISOString());
    })();
    applied.push(name);
  }
  return applied;
}

type Row = { json: string };
const parse = <T>(row: unknown): T | undefined => (row ? (JSON.parse((row as Row).json) as T) : undefined);
/** Runs stored before the bench (round 3) have no `bench`: an empty one
 * (their rules have no benchSize, so it stays empty). */
const withBench = (r: MvpRunState | undefined): MvpRunState | undefined => {
  if (r) r.bench ??= [];
  return r;
};
const parseAll = <T>(rows: unknown[]): T[] => rows.map((r) => JSON.parse((r as Row).json) as T);

export class SqliteMvpStore implements MvpStore {
  readonly db: Database.Database;

  /** `path` is a file, or ":memory:" for tests. `sqlDir`: the migrations
   * (tests build an older schema from a copy of fewer files). */
  constructor(path: string, sqlDir = SQL_DIR) {
    this.db = new Database(path);
    this.db.pragma("journal_mode = WAL");
    this.db.pragma("busy_timeout = 5000");
    migrateMvp(this.db, sqlDir);
  }

  close(): void { this.db.close(); }

  private one<T>(sql: string, ...args: unknown[]): T | undefined { return parse<T>(this.db.prepare(sql).get(...args)); }
  private all<T>(sql: string, ...args: unknown[]): T[] { return parseAll<T>(this.db.prepare(sql).all(...args)); }
  private write(sql: string, ...args: unknown[]): void { this.db.prepare(sql).run(...args); }

  addPlayer(p: PlayerRef): void { this.write("INSERT OR REPLACE INTO mvp_players (id, json) VALUES (?, ?)", p.id, JSON.stringify(p)); }
  player(id: string): PlayerRef | undefined { return this.one("SELECT json FROM mvp_players WHERE id = ?", id); }

  putRun(r: MvpRunState): void {
    this.write(
      "INSERT INTO mvp_runs (id, player_id, over, json) VALUES (?, ?, ?, ?) ON CONFLICT(id) DO UPDATE SET player_id = excluded.player_id, over = excluded.over, json = excluded.json",
      r.runId, r.player.id, r.phase === "over" ? 1 : 0, JSON.stringify(r),
    );
  }
  run(id: string): MvpRunState | undefined { return withBench(this.one("SELECT json FROM mvp_runs WHERE id = ?", id)); }
  activeRun(playerId: string): MvpRunState | undefined { return withBench(this.one("SELECT json FROM mvp_runs WHERE player_id = ? AND over = 0 ORDER BY rowid LIMIT 1", playerId)); }

  addGhost(g: Ghost): void {
    this.write("INSERT INTO mvp_ghosts (ghost_id, round, player_id, content_version, json) VALUES (?, ?, ?, ?, ?)", g.ghostId, g.round, g.player.id, g.contentVersion, JSON.stringify(g));
  }
  ghosts(round: number, opts: { excludePlayerId: string; limit?: number }): Ghost[] {
    // The newest `limit` (-1: all) through the (round, seq) index (M2-2), then
    // oldest first.
    return this.all(
      "SELECT json FROM (SELECT seq, json FROM mvp_ghosts WHERE round = ? AND player_id != ? ORDER BY seq DESC LIMIT ?) ORDER BY seq",
      round, opts.excludePlayerId, opts.limit ?? -1,
    );
  }

  putBattle(b: BattleRecord): void { this.write("INSERT OR REPLACE INTO mvp_battles (id, kind, at, json) VALUES (?, ?, ?, ?)", b.battleId, b.kind, b.at, JSON.stringify(b)); }
  battle(id: string): BattleRecord | undefined { return this.one("SELECT json FROM mvp_battles WHERE id = ?", id); }
  battles(opts: { kind?: FightKind; since?: string } = {}): BattleRecord[] {
    return this.all(
      "SELECT json FROM mvp_battles WHERE (@kind IS NULL OR kind = @kind) AND (@since IS NULL OR at >= @since) ORDER BY at, rowid",
      { kind: opts.kind ?? null, since: opts.since ?? null },
    );
  }

  fusion(first: UnitId, second: UnitId): FusionDiscovery | undefined { return this.one("SELECT json FROM mvp_fusions WHERE first = ? AND second = ?", first, second); }
  putFusion(f: FusionDiscovery): void {
    this.write("INSERT INTO mvp_fusions (first, second, json) VALUES (?, ?, ?) ON CONFLICT(first, second) DO UPDATE SET json = excluded.json", f.first, f.second, JSON.stringify(f));
  }
  fusions(): FusionDiscovery[] { return this.all("SELECT json FROM mvp_fusions ORDER BY rowid"); }

  currentDay(): DayState | undefined { return this.one("SELECT json FROM mvp_days ORDER BY seq DESC LIMIT 1"); }
  putDay(d: DayState): void { this.write("INSERT OR REPLACE INTO mvp_days (seq, json) VALUES (?, ?)", d.seq, JSON.stringify(d)); }

  champion(seq: number): Champion | undefined { return this.one("SELECT json FROM mvp_champions WHERE seq = ?", seq); }
  currentChampion(): Champion | undefined { return this.one("SELECT json FROM mvp_champions ORDER BY seq DESC LIMIT 1"); }
  putChampion(c: Champion): void { this.write("INSERT OR REPLACE INTO mvp_champions (seq, json) VALUES (?, ?)", c.seq, JSON.stringify(c)); }
  champions(): Champion[] { return this.all("SELECT json FROM mvp_champions ORDER BY seq"); }

  addSlay(s: Slay): void { this.write("INSERT INTO mvp_slays (seq, json) VALUES (?, ?)", s.seq, JSON.stringify(s)); }
  slays(seq: number): Slay[] { return this.all("SELECT json FROM mvp_slays WHERE seq = ? ORDER BY id", seq); }

  playoff(seq: number): PlayoffResult | undefined { return this.one("SELECT json FROM mvp_playoffs WHERE seq = ?", seq); }
  putPlayoff(p: PlayoffResult): void { this.write("INSERT OR REPLACE INTO mvp_playoffs (seq, json) VALUES (?, ?)", p.seq, JSON.stringify(p)); }

  rating(playerId: string): Rating | undefined { return this.one("SELECT json FROM mvp_ratings WHERE player_id = ?", playerId); }
  putRating(r: Rating): void { this.write("INSERT OR REPLACE INTO mvp_ratings (player_id, json) VALUES (?, ?)", r.player.id, JSON.stringify(r)); }

  addUnitTallies(contentVersion: string, delta: UnitTallies): void {
    this.db.transaction(() => {
      if (delta.runs) {
        this.write(
          "INSERT INTO mvp_run_tallies (content_version, runs) VALUES (?, ?) ON CONFLICT(content_version) DO UPDATE SET runs = runs + excluded.runs",
          contentVersion, delta.runs,
        );
      }
      const up = this.db.prepare(
        "INSERT INTO mvp_unit_tallies (content_version, unit_id, fights, wins, runs) VALUES (?, ?, ?, ?, ?) " +
          "ON CONFLICT(content_version, unit_id) DO UPDATE SET fights = fights + excluded.fights, wins = wins + excluded.wins, runs = runs + excluded.runs",
      );
      for (const u of delta.units) up.run(contentVersion, u.unitId, u.fights, u.wins, u.runs);
    })();
  }
  unitTallies(contentVersion: string): UnitTallies {
    const runs = (this.db.prepare("SELECT runs FROM mvp_run_tallies WHERE content_version = ?").get(contentVersion) as { runs: number } | undefined)?.runs ?? 0;
    const units = this.db
      .prepare("SELECT unit_id AS unitId, fights, wins, runs FROM mvp_unit_tallies WHERE content_version = ? ORDER BY unit_id")
      .all(contentVersion) as UnitTally[];
    return { runs, units };
  }

  playersNamed(name: string, opts: { bots?: boolean } = {}): PlayerRef[] {
    // Compared in JS (nameKey): SQLite's lower() folds ASCII only.
    const key = nameKey(name);
    return this.all<PlayerRef>(`SELECT json FROM mvp_players${opts.bots ? "" : " WHERE json_extract(json, '$.bot') = 0"} ORDER BY rowid`).filter((p) => nameKey(p.name) === key);
  }
  putInvite(i: Invite): void {
    this.write(
      "INSERT INTO mvp_invites (code, name_key, player_id, json) VALUES (?, ?, ?, ?) ON CONFLICT(code) DO UPDATE SET name_key = excluded.name_key, player_id = excluded.player_id, json = excluded.json",
      i.code, nameKey(i.name), i.playerId, JSON.stringify(i),
    );
  }
  invite(code: string): Invite | undefined { return this.one("SELECT json FROM mvp_invites WHERE code = ?", code); }
  invites(): Invite[] { return this.all("SELECT json FROM mvp_invites ORDER BY rowid"); }
  redeemInvite(code: string, tokenHash: string, at: string): string | undefined {
    // IMMEDIATE takes the write lock first, and the session row is written
    // only from the invite row as it is then: a revoke on another connection
    // either ran before (no row, no session) or runs after (and ends it).
    return this.db.transaction(() => {
      const added = this.db
        .prepare("INSERT INTO mvp_sessions (token_hash, player_id, created_at) SELECT ?, player_id, ? FROM mvp_invites WHERE code = ?")
        .run(tokenHash, at, code).changes;
      if (!added) return undefined;
      const { player_id: playerId } = this.db.prepare("SELECT player_id FROM mvp_invites WHERE code = ?").get(code) as { player_id: string };
      this.write("UPDATE mvp_invites SET json = json_set(json, '$.redeemedAt', ?) WHERE code = ? AND json_extract(json, '$.redeemedAt') IS NULL", at, code);
      this.write(
        "DELETE FROM mvp_sessions WHERE player_id = ? AND rowid NOT IN (SELECT rowid FROM mvp_sessions WHERE player_id = ? ORDER BY rowid DESC LIMIT ?)",
        playerId, playerId, MAX_SESSIONS,
      );
      return playerId;
    }).immediate();
  }
  sessionPlayer(tokenHash: string): string | undefined {
    return (this.db.prepare("SELECT player_id FROM mvp_sessions WHERE token_hash = ?").get(tokenHash) as { player_id: string } | undefined)?.player_id;
  }
  joinCode(): string | undefined {
    return (this.db.prepare("SELECT value FROM mvp_settings WHERE key = 'join_code'").get() as { value: string } | undefined)?.value;
  }
  setJoinCode(code: string): void { this.write("INSERT INTO mvp_settings (key, value) VALUES ('join_code', ?) ON CONFLICT(key) DO UPDATE SET value = excluded.value", code); }
  rotateInvite(oldCode: string, next: Invite): number {
    return this.db.transaction(() => {
      this.write("DELETE FROM mvp_invites WHERE code = ?", oldCode);
      this.putInvite(next);
      return this.db.prepare("DELETE FROM mvp_sessions WHERE player_id = ?").run(next.playerId).changes;
    }).immediate();
  }

  putUnit(u: StoredUnit): void {
    this.write(
      "INSERT INTO mvp_units (unit_id, status, author_id, json) VALUES (?, ?, ?, ?) ON CONFLICT(unit_id) DO UPDATE SET status = excluded.status, author_id = excluded.author_id, json = excluded.json",
      u.unitId, u.status, u.authorId, JSON.stringify(u),
    );
  }
  unit(id: UnitId): StoredUnit | undefined { return this.one("SELECT json FROM mvp_units WHERE unit_id = ?", id); }
  units(opts: { status?: UnitStatus } = {}): StoredUnit[] {
    return this.all("SELECT json FROM mvp_units WHERE (@status IS NULL OR status = @status) ORDER BY rowid", { status: opts.status ?? null });
  }
  putPool(p: PoolSnapshot): void { this.write("INSERT INTO mvp_pools (version, day_seq, json) VALUES (?, ?, ?)", p.version, p.daySeq, JSON.stringify(p)); }
  pool(version: string): PoolSnapshot | undefined { return this.one("SELECT json FROM mvp_pools WHERE version = ? ORDER BY seq DESC LIMIT 1", version); }
  currentPool(): PoolSnapshot | undefined { return this.one("SELECT json FROM mvp_pools ORDER BY seq DESC LIMIT 1"); }
  putStint(s: PoolStint): void {
    this.write(
      "INSERT INTO mvp_pool_stints (unit_id, entered_seq, json) VALUES (?, ?, ?) ON CONFLICT(unit_id, entered_seq) DO UPDATE SET json = excluded.json",
      s.unitId, s.enteredSeq, JSON.stringify(s),
    );
  }
  stints(unitId?: UnitId): PoolStint[] {
    return this.all("SELECT json FROM mvp_pool_stints WHERE (@unit IS NULL OR unit_id = @unit) ORDER BY unit_id, entered_seq", { unit: unitId ?? null });
  }
  seedPool(units: StoredUnit[], pool: PoolSnapshot, stints: PoolStint[]): boolean {
    // IMMEDIATE: two servers starting on one file can't both seed.
    return this.db.transaction(() => {
      if (this.db.prepare("SELECT 1 FROM mvp_pools LIMIT 1").get()) return false;
      for (const u of units) this.putUnit(u);
      this.putPool(pool);
      for (const s of stints) this.putStint(s);
      return true;
    }).immediate();
  }
  addDayTallies(daySeq: number, delta: DayTallies): void {
    this.db.transaction(() => {
      if (delta.runs) {
        this.write("INSERT INTO mvp_day_run_tallies (day_seq, runs) VALUES (?, ?) ON CONFLICT(day_seq) DO UPDATE SET runs = runs + excluded.runs", daySeq, delta.runs);
      }
      const up = this.db.prepare(
        "INSERT INTO mvp_unit_day_tallies (day_seq, unit_id, fights, wins, runs, picks) VALUES (?, ?, ?, ?, ?, ?) " +
          "ON CONFLICT(day_seq, unit_id) DO UPDATE SET fights = fights + excluded.fights, wins = wins + excluded.wins, runs = runs + excluded.runs, picks = picks + excluded.picks",
      );
      for (const u of delta.units) up.run(daySeq, u.unitId, u.fights, u.wins, u.runs, u.picks);
    })();
  }
  dayTallies(daySeq: number): DayTallies {
    const runs = (this.db.prepare("SELECT runs FROM mvp_day_run_tallies WHERE day_seq = ?").get(daySeq) as { runs: number } | undefined)?.runs ?? 0;
    const units = this.db
      .prepare("SELECT unit_id AS unitId, fights, wins, runs, picks FROM mvp_unit_day_tallies WHERE day_seq = ? ORDER BY unit_id")
      .all(daySeq) as UnitDayTally[];
    return { runs, units };
  }
  ideaCounts(playerId: string): IdeaCounts { return this.one<IdeaCounts>("SELECT json FROM mvp_idea_counts WHERE player_id = ?", playerId) ?? { ...NO_IDEAS }; }
  putIdeaCounts(playerId: string, c: IdeaCounts): void {
    this.write("INSERT INTO mvp_idea_counts (player_id, json) VALUES (?, ?) ON CONFLICT(player_id) DO UPDATE SET json = excluded.json", playerId, JSON.stringify(c));
  }
}
