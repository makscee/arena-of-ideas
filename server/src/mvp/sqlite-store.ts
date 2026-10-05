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
import type { MvpStore } from "./store.js";

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
const parseAll = <T>(rows: unknown[]): T[] => rows.map((r) => JSON.parse((r as Row).json) as T);

export class SqliteMvpStore implements MvpStore {
  readonly db: Database.Database;

  /** `path` is a file, or ":memory:" for tests. */
  constructor(path: string) {
    this.db = new Database(path);
    this.db.pragma("journal_mode = WAL");
    this.db.pragma("busy_timeout = 5000");
    migrateMvp(this.db);
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
  run(id: string): MvpRunState | undefined { return this.one("SELECT json FROM mvp_runs WHERE id = ?", id); }
  activeRun(playerId: string): MvpRunState | undefined { return this.one("SELECT json FROM mvp_runs WHERE player_id = ? AND over = 0 ORDER BY rowid LIMIT 1", playerId); }

  addGhost(g: Ghost): void {
    this.write("INSERT INTO mvp_ghosts (ghost_id, round, player_id, content_version, json) VALUES (?, ?, ?, ?, ?)", g.ghostId, g.round, g.player.id, g.contentVersion, JSON.stringify(g));
  }
  ghosts(round: number, opts: { excludePlayerId: string; contentVersion: string }): Ghost[] {
    return this.all("SELECT json FROM mvp_ghosts WHERE round = ? AND content_version = ? AND player_id != ? ORDER BY seq", round, opts.contentVersion, opts.excludePlayerId);
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
}
