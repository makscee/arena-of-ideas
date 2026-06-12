import Database from "better-sqlite3";
import { drizzle, type BetterSQLite3Database } from "drizzle-orm/better-sqlite3";
import { mkdirSync } from "node:fs";
import { dirname } from "node:path";
import * as schema from "./schema.js";

export type DB = BetterSQLite3Database<typeof schema>;

/**
 * Schema DDL, applied idempotently at open. The arena server owns the auth
 * tables (slice 1) and the shared-ladder tables (slice 2); until a real
 * migration story is needed (a later concern), the canonical shape lives in
 * schema.ts and this DDL mirrors it.
 */
const DDL = `
CREATE TABLE IF NOT EXISTS users (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL UNIQUE,
  display_name TEXT,
  created_at INTEGER NOT NULL,
  updated_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS sessions (
  id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  token_hash TEXT NOT NULL UNIQUE,
  label TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  last_used_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL
);
CREATE TABLE IF NOT EXISTS email_codes (
  id TEXT PRIMARY KEY,
  email TEXT NOT NULL,
  code_hash TEXT NOT NULL,
  created_at INTEGER NOT NULL,
  expires_at INTEGER NOT NULL,
  attempts INTEGER NOT NULL DEFAULT 0,
  consumed_at INTEGER
);
CREATE INDEX IF NOT EXISTS email_codes_email_idx ON email_codes (email, created_at);
CREATE TABLE IF NOT EXISTS ladder_ghosts (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  round INTEGER NOT NULL,
  seq INTEGER NOT NULL,
  run_id TEXT NOT NULL,
  user_id TEXT,
  team TEXT NOT NULL,
  UNIQUE (round, seq)
);
CREATE TABLE IF NOT EXISTS ladder_champions (
  id INTEGER PRIMARY KEY AUTOINCREMENT,
  run_id TEXT NOT NULL,
  user_id TEXT,
  round INTEGER NOT NULL,
  seq INTEGER NOT NULL,
  team TEXT NOT NULL
);
CREATE TABLE IF NOT EXISTS run_submissions (
  run_id TEXT PRIMARY KEY,
  user_id TEXT NOT NULL,
  seed INTEGER NOT NULL,
  ended_by TEXT NOT NULL,
  final_round INTEGER NOT NULL,
  submitted_at INTEGER NOT NULL
);
`;

/** Open (and create if missing) the SQLite DB at `path`, or ":memory:". */
export function openDb(path: string): { db: DB; sqlite: Database.Database } {
  if (path !== ":memory:") {
    try {
      mkdirSync(dirname(path), { recursive: true });
    } catch {
      // ignore — directory may already exist
    }
  }
  const sqlite = new Database(path);
  sqlite.pragma("journal_mode = WAL");
  sqlite.pragma("foreign_keys = ON");
  sqlite.exec(DDL);
  const db = drizzle(sqlite, { schema });
  return { db, sqlite };
}
