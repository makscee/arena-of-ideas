import Database from "better-sqlite3";
import { drizzle, type BetterSQLite3Database } from "drizzle-orm/better-sqlite3";
import { mkdirSync } from "node:fs";
import { dirname } from "node:path";
import * as schema from "./schema.js";
import { defaultApprovedRegistry, defaultArenaContent, parseArenaContentSnapshot } from "./content.js";

export type DB = BetterSQLite3Database<typeof schema>;
export const ARENA_SCHEMA_VERSION = 1;

/** The exact a68cf5b7 schema. Migration tests build fixtures from this text so
 * the upgrade starts at the released boundary, not at a hand-waved subset. */
export const PRE_AOI62_DDL = `
CREATE TABLE users (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  display_name TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE TABLE sessions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  token_hash TEXT NOT NULL UNIQUE,
  label TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  last_used_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
);
CREATE TABLE email_codes (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL,
  code_hash TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  consumed_at INTEGER
);
CREATE INDEX email_codes_email_idx ON email_codes (email, created_at);
CREATE TABLE ladder_ghosts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  round INTEGER NOT NULL,
  seq INTEGER NOT NULL,
  run_id TEXT NOT NULL,
  user_id TEXT,
  team TEXT NOT NULL,
  UNIQUE (round, seq)
);
CREATE TABLE ladder_champions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  run_id TEXT NOT NULL,
  user_id TEXT,
  round INTEGER NOT NULL,
  seq INTEGER NOT NULL,
  team TEXT NOT NULL
);
CREATE TABLE run_opens (
  run_id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  ghost_watermark INTEGER NOT NULL,
  opened_at INTEGER NOT NULL
);
CREATE TABLE run_pool_serves (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  run_id TEXT NOT NULL,
  round INTEGER NOT NULL,
  served_len INTEGER NOT NULL,
  champion_run_id TEXT NOT NULL,
  served_at INTEGER NOT NULL
);
CREATE UNIQUE INDEX run_pool_serves_view_idx
  ON run_pool_serves (run_id, round, served_len, champion_run_id);
CREATE TABLE run_submissions (
  run_id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  seed INTEGER NOT NULL,
  ended_by TEXT NOT NULL,
  final_round INTEGER NOT NULL,
  submitted_at INTEGER NOT NULL
);
CREATE TABLE ideas (
  id TEXT PRIMARY KEY,
  seq INTEGER NOT NULL UNIQUE,
  author_id TEXT NOT NULL,
  text TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  status TEXT NOT NULL DEFAULT 'on-table',
  bounce_reason TEXT
);
CREATE TABLE idea_votes (
  idea_id TEXT NOT NULL,
  user_id TEXT NOT NULL,
  direction TEXT NOT NULL,
  voted_at INTEGER NOT NULL,
  PRIMARY KEY (idea_id, user_id)
);
`;

const AOI62_DDL = `
ALTER TABLE run_opens ADD COLUMN content_version INTEGER;
ALTER TABLE run_submissions ADD COLUMN content_version INTEGER;
CREATE TABLE season_state (
  id INTEGER PRIMARY KEY CHECK (id = 1),
  season INTEGER NOT NULL CHECK (season >= 1),
  content_version INTEGER NOT NULL CHECK (content_version >= 1)
);
CREATE TABLE content_versions (
  version INTEGER PRIMARY KEY,
  approved_registry TEXT NOT NULL,
  pool TEXT NOT NULL,
  statuses TEXT NOT NULL,
  abilities TEXT NOT NULL,
  created_at INTEGER NOT NULL
);
CREATE TABLE season_freezes (
  season INTEGER PRIMARY KEY CHECK (season >= 1),
  content_version INTEGER NOT NULL CHECK (content_version >= 1),
  selection_receipt_json TEXT NOT NULL,
  frozen_at INTEGER NOT NULL
);
CREATE TABLE idea_builds (
  season INTEGER NOT NULL,
  idea_id TEXT NOT NULL,
  rank INTEGER NOT NULL,
  tally_json TEXT NOT NULL,
  status TEXT NOT NULL,
  candidate_json TEXT,
  candidate_provenance_json TEXT,
  author_user_id TEXT,
  creator_display_name TEXT,
  shipped_units_json TEXT,
  bounce_reason TEXT,
  updated_at INTEGER NOT NULL,
  PRIMARY KEY (season, idea_id)
);
CREATE TABLE season_archives (
  season INTEGER PRIMARY KEY,
  content_version INTEGER NOT NULL,
  final_tower_json TEXT NOT NULL,
  selection_receipt_json TEXT NOT NULL,
  outcome_receipt_json TEXT NOT NULL,
  archived_at INTEGER NOT NULL
);
`;

type Column = { name: string; type: string; notnull: number; pk: number };
type SchemaObject = { type: "table" | "index"; name: string; table: string; sql: string | null };

const LEGACY_SHAPE: Record<string, Array<[string, string, number, number]>> = {
  users: [["id", "TEXT", 0, 1], ["email", "TEXT", 1, 0], ["display_name", "TEXT", 0, 0], ["created_at", "INTEGER", 1, 0], ["updated_at", "INTEGER", 1, 0]],
  sessions: [["id", "TEXT", 0, 1], ["user_id", "TEXT", 1, 0], ["token_hash", "TEXT", 1, 0], ["label", "TEXT", 1, 0], ["created_at", "INTEGER", 1, 0], ["last_used_at", "INTEGER", 1, 0], ["expires_at", "INTEGER", 1, 0]],
  email_codes: [["id", "TEXT", 0, 1], ["email", "TEXT", 1, 0], ["code_hash", "TEXT", 1, 0], ["created_at", "INTEGER", 1, 0], ["expires_at", "INTEGER", 1, 0], ["attempts", "INTEGER", 1, 0], ["consumed_at", "INTEGER", 0, 0]],
  ladder_ghosts: [["id", "INTEGER", 0, 1], ["round", "INTEGER", 1, 0], ["seq", "INTEGER", 1, 0], ["run_id", "TEXT", 1, 0], ["user_id", "TEXT", 0, 0], ["team", "TEXT", 1, 0]],
  ladder_champions: [["id", "INTEGER", 0, 1], ["run_id", "TEXT", 1, 0], ["user_id", "TEXT", 0, 0], ["round", "INTEGER", 1, 0], ["seq", "INTEGER", 1, 0], ["team", "TEXT", 1, 0]],
  run_opens: [["run_id", "TEXT", 0, 1], ["user_id", "TEXT", 1, 0], ["ghost_watermark", "INTEGER", 1, 0], ["opened_at", "INTEGER", 1, 0]],
  run_pool_serves: [["id", "INTEGER", 0, 1], ["run_id", "TEXT", 1, 0], ["round", "INTEGER", 1, 0], ["served_len", "INTEGER", 1, 0], ["champion_run_id", "TEXT", 1, 0], ["served_at", "INTEGER", 1, 0]],
  run_submissions: [["run_id", "TEXT", 0, 1], ["user_id", "TEXT", 1, 0], ["seed", "INTEGER", 1, 0], ["ended_by", "TEXT", 1, 0], ["final_round", "INTEGER", 1, 0], ["submitted_at", "INTEGER", 1, 0]],
  ideas: [["id", "TEXT", 0, 1], ["seq", "INTEGER", 1, 0], ["author_id", "TEXT", 1, 0], ["text", "TEXT", 1, 0], ["created_at", "INTEGER", 1, 0], ["status", "TEXT", 1, 0], ["bounce_reason", "TEXT", 0, 0]],
  idea_votes: [["idea_id", "TEXT", 1, 1], ["user_id", "TEXT", 1, 2], ["direction", "TEXT", 1, 0], ["voted_at", "INTEGER", 1, 0]],
};

function userTables(sqlite: Database.Database): string[] {
  return (sqlite.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").all() as { name: string }[]).map((r) => r.name);
}

function normalizedSql(sql: string | null): string | null {
  return sql?.replace(/\s+/g, " ").replace(/\s*([(),=])\s*/g, "$1").trim() ?? null;
}

function schemaObjects(sqlite: Database.Database): SchemaObject[] {
  return (sqlite.prepare(`
    SELECT type, name, tbl_name AS "table", sql
    FROM sqlite_master
    WHERE type IN ('table', 'index') AND tbl_name NOT LIKE 'sqlite_%'
    ORDER BY type, name
  `).all() as SchemaObject[]).map((object) => ({ ...object, sql: normalizedSql(object.sql) }));
}

function expectedSchemaObjects(ddl: string): SchemaObject[] {
  const canonical = new Database(":memory:");
  try {
    canonical.exec(ddl);
    return schemaObjects(canonical);
  } finally {
    canonical.close();
  }
}

const LEGACY_SCHEMA_OBJECTS = expectedSchemaObjects(PRE_AOI62_DDL);
const FINAL_SCHEMA_OBJECTS = expectedSchemaObjects(`${PRE_AOI62_DDL}\n${AOI62_DDL}`);

function assertShape(
  sqlite: Database.Database,
  expected: Record<string, Array<[string, string, number, number]>>,
  expectedObjects: SchemaObject[],
  label: string,
): void {
  const actualTables = userTables(sqlite);
  const wanted = Object.keys(expected).sort();
  if (JSON.stringify(actualTables) !== JSON.stringify(wanted)) {
    throw new Error(`${label}: unsupported/corrupt table set; expected ${wanted.join(", ")}, got ${actualTables.join(", ")}`);
  }
  for (const [table, want] of Object.entries(expected)) {
    const got = sqlite.prepare(`PRAGMA table_info(${JSON.stringify(table)})`).all() as Column[];
    const shape = got.map((c) => [c.name, c.type.toUpperCase(), c.notnull, c.pk]);
    if (JSON.stringify(shape) !== JSON.stringify(want)) {
      throw new Error(`${label}: unsupported/corrupt ${table} columns; expected ${JSON.stringify(want)}, got ${JSON.stringify(shape)}`);
    }
  }
  const actualObjects = schemaObjects(sqlite);
  if (JSON.stringify(actualObjects) !== JSON.stringify(expectedObjects)) {
    throw new Error(`${label}: unsupported/corrupt DDL, index, uniqueness, or CHECK constraint shape`);
  }
}

function finalShape(): Record<string, Array<[string, string, number, number]>> {
  return {
    ...LEGACY_SHAPE,
    run_opens: [...LEGACY_SHAPE.run_opens!, ["content_version", "INTEGER", 0, 0]],
    run_submissions: [...LEGACY_SHAPE.run_submissions!, ["content_version", "INTEGER", 0, 0]],
    season_state: [["id", "INTEGER", 0, 1], ["season", "INTEGER", 1, 0], ["content_version", "INTEGER", 1, 0]],
    content_versions: [["version", "INTEGER", 0, 1], ["approved_registry", "TEXT", 1, 0], ["pool", "TEXT", 1, 0], ["statuses", "TEXT", 1, 0], ["abilities", "TEXT", 1, 0], ["created_at", "INTEGER", 1, 0]],
    season_freezes: [["season", "INTEGER", 0, 1], ["content_version", "INTEGER", 1, 0], ["selection_receipt_json", "TEXT", 1, 0], ["frozen_at", "INTEGER", 1, 0]],
    idea_builds: [["season", "INTEGER", 1, 1], ["idea_id", "TEXT", 1, 2], ["rank", "INTEGER", 1, 0], ["tally_json", "TEXT", 1, 0], ["status", "TEXT", 1, 0], ["candidate_json", "TEXT", 0, 0], ["candidate_provenance_json", "TEXT", 0, 0], ["author_user_id", "TEXT", 0, 0], ["creator_display_name", "TEXT", 0, 0], ["shipped_units_json", "TEXT", 0, 0], ["bounce_reason", "TEXT", 0, 0], ["updated_at", "INTEGER", 1, 0]],
    season_archives: [["season", "INTEGER", 0, 1], ["content_version", "INTEGER", 1, 0], ["final_tower_json", "TEXT", 1, 0], ["selection_receipt_json", "TEXT", 1, 0], ["outcome_receipt_json", "TEXT", 1, 0], ["archived_at", "INTEGER", 1, 0]],
  };
}

function assertCurrentState(sqlite: Database.Database): void {
  const label = `arena schema v${ARENA_SCHEMA_VERSION}`;
  assertShape(sqlite, finalShape(), FINAL_SCHEMA_OBJECTS, label);
  const integrity = sqlite.pragma("integrity_check", { simple: true });
  if (integrity !== "ok") throw new Error(`${label}: SQLite integrity_check failed: ${String(integrity)}`);
  const states = sqlite.prepare("SELECT season, content_version FROM season_state WHERE id=1").all() as { season: number; content_version: number }[];
  if (states.length !== 1) throw new Error(`${label}: corrupt season_state; expected exactly one id=1 row`);
  const contents = sqlite.prepare("SELECT version, approved_registry, pool, statuses, abilities FROM content_versions ORDER BY version").all() as Array<{
    version: number;
    approved_registry: string;
    pool: string;
    statuses: string;
    abilities: string;
  }>;
  if (!contents.some((content) => content.version === states[0]!.content_version)) {
    throw new Error(`${label}: active content version ${states[0]!.content_version} is missing`);
  }
  for (const content of contents) {
    try {
      parseArenaContentSnapshot({
        approvedRegistry: content.approved_registry,
        pool: content.pool,
        statuses: content.statuses,
        abilities: content.abilities,
      }, `content v${content.version}`);
    } catch (err) {
      throw new Error(`${label}: corrupt content version ${content.version}: ${(err as Error).message}`);
    }
  }
}

function migrate(sqlite: Database.Database): void {
  const version = sqlite.pragma("user_version", { simple: true }) as number;
  if (version > ARENA_SCHEMA_VERSION || version < 0) {
    throw new Error(`unsupported arena schema version ${version}; this build supports ${ARENA_SCHEMA_VERSION}`);
  }
  if (version === ARENA_SCHEMA_VERSION) {
    assertCurrentState(sqlite);
    return;
  }
  const fresh = userTables(sqlite).length === 0;
  if (!fresh) assertShape(sqlite, LEGACY_SHAPE, LEGACY_SCHEMA_OBJECTS, "pre-AOI-62 schema (a68cf5b7)");
  const tx = sqlite.transaction(() => {
    if (fresh) sqlite.exec(PRE_AOI62_DDL);
    sqlite.exec(AOI62_DDL);
    const content = defaultArenaContent();
    const approved = defaultApprovedRegistry();
    sqlite.prepare("INSERT INTO content_versions (version, approved_registry, pool, statuses, abilities, created_at) VALUES (1, ?, ?, ?, ?, 0)")
      .run(JSON.stringify(approved), JSON.stringify(content.pool), JSON.stringify(content.statuses), JSON.stringify(content.abilities));
    sqlite.prepare("INSERT INTO season_state (id, season, content_version) VALUES (1, 1, 1)").run();
    sqlite.pragma(`user_version = ${ARENA_SCHEMA_VERSION}`);
  });
  tx.immediate();
  assertCurrentState(sqlite);
}

/** Open/create and explicitly migrate the SQLite DB. A present unknown or
 * malformed schema fails before Drizzle is exposed; nothing is reset. */
export function openDb(path: string): { db: DB; sqlite: Database.Database } {
  if (path !== ":memory:") mkdirSync(dirname(path), { recursive: true });
  const sqlite = new Database(path);
  try {
    // Validate before changing persistent journal mode: a refused schema/content
    // open must not rewrite or reset any part of the database it rejected.
    migrate(sqlite);
    sqlite.pragma("journal_mode = WAL");
    sqlite.pragma("foreign_keys = ON");
  } catch (err) {
    sqlite.close();
    throw err;
  }
  return { db: drizzle(sqlite, { schema }), sqlite };
}
