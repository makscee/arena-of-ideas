import Database from "better-sqlite3";
import { mkdtempSync, readFileSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, test } from "vitest";
import { ARENA_SCHEMA_VERSION, PRE_AOI62_DDL, openDb } from "./db.js";

const dirs: string[] = [];
afterEach(() => { for (const dir of dirs.splice(0)) rmSync(dir, { recursive: true, force: true }); });
function pathOf(name: string) { const dir = mkdtempSync(join(tmpdir(), "aoi62-migration-")); dirs.push(dir); return join(dir, name); }

function legacyFixture(path: string): Record<string, unknown[]> {
  const db = new Database(path);
  db.exec(PRE_AOI62_DDL);
  db.prepare("INSERT INTO users VALUES (?,?,?,?,?)").run("u1", "real@example.com", "Real Player", 10, 11);
  db.prepare("INSERT INTO sessions VALUES (?,?,?,?,?,?,?)").run("s1", "u1", "hash", "web", 12, 13, 9999);
  db.prepare("INSERT INTO email_codes VALUES (?,?,?,?,?,?,?)").run("e1", "real@example.com", "codehash", 1, 2, 1, null);
  const team = JSON.stringify([{ name: "Preserved", base: { hp: 7, pwr: 3 }, ability: "Strike" }]);
  db.prepare("INSERT INTO ladder_ghosts (round,seq,run_id,user_id,team) VALUES (1,0,'run-old','u1',?)").run(team);
  db.prepare("INSERT INTO ladder_champions (run_id,user_id,round,seq,team) VALUES ('run-old','u1',1,0,?)").run(team);
  db.prepare("INSERT INTO run_opens VALUES (?,?,?,?)").run("open-old", "u1", 1, 20);
  db.prepare("INSERT INTO run_pool_serves (run_id,round,served_len,champion_run_id,served_at) VALUES (?,?,?,?,?)").run("open-old", 1, 1, "run-old", 21);
  db.prepare("INSERT INTO run_submissions VALUES (?,?,?,?,?,?)").run("submitted-old", "u1", 42, "crown", 3, 22);
  db.prepare("INSERT INTO ideas VALUES (?,?,?,?,?,?,?)").run("idea-0", 0, "u1", "real persisted idea", 23, "bounced", "old reason");
  db.prepare("INSERT INTO idea_votes VALUES (?,?,?,?)").run("idea-0", "u1", "down", 24);
  const snapshot: Record<string, unknown[]> = {};
  for (const table of ["users", "sessions", "email_codes", "ladder_ghosts", "ladder_champions", "run_opens", "run_pool_serves", "run_submissions", "ideas", "idea_votes"]) {
    snapshot[table] = db.prepare(`SELECT * FROM ${table}`).all();
  }
  db.close();
  return snapshot;
}

function tableNames(db: Database.Database) {
  return db.prepare("SELECT name FROM sqlite_master WHERE type='table' AND name NOT LIKE 'sqlite_%' ORDER BY name").all();
}

function expectRefusedWithoutMutation(path: string, pattern: RegExp): void {
  const before = readFileSync(path);
  expect(() => openDb(path)).toThrow(pattern);
  expect(readFileSync(path)).toEqual(before);
}

describe("explicit a68cf5b7 → AOI-62 SQLite migration", () => {
  test("preserves real rows, seeds v1 exactly once, and reopens idempotently", () => {
    const path = pathOf("fixture.db");
    const before = legacyFixture(path);
    const first = openDb(path);
    expect(first.sqlite.pragma("user_version", { simple: true })).toBe(ARENA_SCHEMA_VERSION);
    for (const [table, rows] of Object.entries(before)) {
      const after = first.sqlite.prepare(`SELECT * FROM ${table}`).all() as Record<string, unknown>[];
      if (table === "run_opens" || table === "run_submissions") {
        expect(after.map(({ content_version: _new, ...old }) => old)).toEqual(rows);
        expect(after[0]!.content_version).toBeNull();
      } else expect(after).toEqual(rows);
    }
    expect(first.sqlite.prepare("SELECT * FROM season_state").all()).toEqual([{ id: 1, season: 1, content_version: 1 }]);
    expect((first.sqlite.prepare("SELECT COUNT(*) c FROM content_versions").get() as { c: number }).c).toBe(1);
    const afterFirst = first.sqlite.serialize();
    first.sqlite.close();

    const reopened = openDb(path);
    expect(reopened.sqlite.serialize()).toEqual(afterFirst);
    expect((reopened.sqlite.prepare("SELECT COUNT(*) c FROM content_versions").get() as { c: number }).c).toBe(1);
    reopened.sqlite.close();
  });

  test("fresh creation and upgraded fixture have the same final table/column schema", () => {
    const oldPath = pathOf("old.db");
    legacyFixture(oldPath);
    const upgraded = openDb(oldPath);
    const fresh = openDb(pathOf("fresh.db"));
    expect(tableNames(fresh.sqlite)).toEqual(tableNames(upgraded.sqlite));
    for (const { name } of tableNames(fresh.sqlite) as { name: string }[]) {
      expect(fresh.sqlite.prepare(`PRAGMA table_info(${JSON.stringify(name)})`).all())
        .toEqual(upgraded.sqlite.prepare(`PRAGMA table_info(${JSON.stringify(name)})`).all());
    }
    upgraded.sqlite.close(); fresh.sqlite.close();
  });

  test("unsupported and corrupt persisted versions fail loudly without reset", () => {
    const unsupported = pathOf("future.db");
    const future = new Database(unsupported); future.pragma("user_version = 99"); future.close();
    expect(() => openDb(unsupported)).toThrow(/unsupported arena schema version 99/);

    const corruptLegacy = pathOf("corrupt-old.db");
    const partial = new Database(corruptLegacy); partial.exec("CREATE TABLE users (id TEXT PRIMARY KEY)"); partial.close();
    expect(() => openDb(corruptLegacy)).toThrow(/unsupported\/corrupt/);

    const corruptCurrent = pathOf("corrupt-current.db");
    const current = openDb(corruptCurrent); current.sqlite.prepare("UPDATE content_versions SET pool='not json' WHERE version=1").run(); current.sqlite.close();
    expectRefusedWithoutMutation(corruptCurrent, /corrupt content version 1.*pool is corrupt JSON/);
  });

  test("rejects dropped released indexes and changed released uniqueness before migration without mutation", () => {
    const droppedPath = pathOf("released-dropped-indexes.db");
    legacyFixture(droppedPath);
    const dropped = new Database(droppedPath);
    dropped.exec("DROP INDEX email_codes_email_idx; DROP INDEX run_pool_serves_view_idx;");
    dropped.close();
    expectRefusedWithoutMutation(droppedPath, /DDL, index, uniqueness, or CHECK constraint shape/);
    const stillLegacy = new Database(droppedPath);
    expect(stillLegacy.pragma("user_version", { simple: true })).toBe(0);
    expect(tableNames(stillLegacy)).not.toContainEqual({ name: "season_state" });
    stillLegacy.close();

    const uniquenessPath = pathOf("released-changed-uniqueness.db");
    legacyFixture(uniquenessPath);
    const uniqueness = new Database(uniquenessPath);
    uniqueness.exec("DROP INDEX run_pool_serves_view_idx; CREATE INDEX run_pool_serves_view_idx ON run_pool_serves (run_id, round, served_len, champion_run_id);");
    uniqueness.close();
    expectRefusedWithoutMutation(uniquenessPath, /DDL, index, uniqueness, or CHECK constraint shape/);
  });

  test("rejects changed final indexes and CHECK DDL without mutating current state", () => {
    const indexPath = pathOf("current-changed-index.db");
    const indexed = openDb(indexPath); indexed.sqlite.close();
    const changedIndex = new Database(indexPath);
    changedIndex.exec("DROP INDEX run_pool_serves_view_idx; CREATE INDEX run_pool_serves_view_idx ON run_pool_serves (run_id, round, served_len, champion_run_id);");
    changedIndex.close();
    expectRefusedWithoutMutation(indexPath, /DDL, index, uniqueness, or CHECK constraint shape/);

    const checkPath = pathOf("current-changed-check.db");
    const checked = openDb(checkPath); checked.sqlite.close();
    const changedCheck = new Database(checkPath);
    changedCheck.exec(`
      ALTER TABLE season_state RENAME TO season_state_old;
      CREATE TABLE season_state (
        id INTEGER PRIMARY KEY,
        season INTEGER NOT NULL,
        content_version INTEGER NOT NULL
      );
      INSERT INTO season_state SELECT * FROM season_state_old;
      DROP TABLE season_state_old;
    `);
    changedCheck.close();
    expectRefusedWithoutMutation(checkPath, /DDL, index, uniqueness, or CHECK constraint shape/);
  });

  test("rejects active valid-JSON wrong-shape pool and malformed inactive content without mutation", () => {
    const activePath = pathOf("active-object-pool.db");
    const active = openDb(activePath);
    active.sqlite.prepare("UPDATE content_versions SET pool='{}' WHERE version=1").run();
    active.sqlite.close();
    expectRefusedWithoutMutation(activePath, /corrupt content version 1.*expected \{ grammarVersion, units, abilities \}/);

    const inactivePath = pathOf("inactive-malformed.db");
    const inactive = openDb(inactivePath);
    inactive.sqlite.prepare(`
      INSERT INTO content_versions (version, approved_registry, pool, statuses, abilities, created_at)
      SELECT 2, approved_registry, '{}', statuses, abilities, 1 FROM content_versions WHERE version=1
    `).run();
    inactive.sqlite.close();
    expectRefusedWithoutMutation(inactivePath, /corrupt content version 2.*expected \{ grammarVersion, units, abilities \}/);
    const unchanged = new Database(inactivePath);
    expect((unchanged.prepare("SELECT COUNT(*) AS count FROM content_versions").get() as { count: number }).count).toBe(2);
    unchanged.close();
  });
});
