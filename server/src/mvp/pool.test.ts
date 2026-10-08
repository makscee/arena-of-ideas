// M2-1 (mission #735): units as data. The DB's pool builds exactly the code
// pool, and a world made by main before it (9ef07ac) migrates with every old
// row unchanged and its runs going on.
import { copyFileSync, mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { afterEach, describe, expect, it } from "vitest";
import type { PlayerRef } from "../../../src/mvp/contract.js";
import { archetypeProblems, ROWS } from "../../../src/mvp/units.js";
import { playBotRun, seedChampion } from "./bots.js";
import { contentOf, mvpContent } from "./content.js";
import { endDay } from "./day.js";
import { CUT_IN_ROUND_4, poolContent, seedUnits } from "./pool.js";
import { decide, startRun } from "./runs.js";
import { mvpRuntime } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";

const SQL_DIR = fileURLToPath(new URL("./sql/", import.meta.url));
/** The migrations main had at 9ef07ac, before mission #735. */
const MAIN_9EF07AC = ["04-runs.sql", "11-stats.sql", "13-invites.sql", "13b-join.sql"];
/** The store methods M2-1 added: main's store had none of them. */
const ADDED = ["putUnit", "unit", "units", "putPool", "pool", "currentPool", "putStint", "stints", "seedPool", "addDayTallies", "dayTallies"];

const dirs: string[] = [];
afterEach(() => { for (const d of dirs.splice(0)) rmSync(d, { recursive: true, force: true }); });

const lcg = (start: number) => { let s = start; return () => (s = (s * 1103515245 + 12345) >>> 0); };

/** Every row of every table but mvp_migrations, in rowid order. */
function dump(store: SqliteMvpStore, tables?: string[]): Record<string, unknown[]> {
  const names = tables ?? (store.db.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name LIKE 'mvp_%' AND name != 'mvp_migrations' ORDER BY name").all() as { name: string }[]).map((r) => r.name);
  return Object.fromEntries(names.map((n) => [n, store.db.prepare(`SELECT * FROM ${n} ORDER BY rowid`).all()]));
}

describe("units as data (M2-1)", { timeout: 60_000 }, () => {
  it("seeds 73 live units, the 8 cut in round 4 and one pool that builds byte-identical to the code pool, once", () => {
    for (const store of [new MemoryMvpStore(), new SqliteMvpStore(":memory:")] as MvpStore[]) {
      const at = new Date("2026-10-08T08:00:00.000Z");
      expect(seedUnits(store, at)).toBe(true);
      const content = poolContent(store);
      const code = mvpContent();
      expect(content.version).toBe(code.version);
      expect(JSON.stringify(content)).toBe(JSON.stringify(code));
      expect(store.units({ status: "live" }).map((u) => u.unitId)).toEqual(code.units.map((u) => u.id));
      expect(store.units({ status: "live" })).toHaveLength(73);
      expect(store.units({ status: "library" }).map((u) => u.row.name)).toEqual(["Rat", "Spike", "Wither", "Rot", "Pediatrician", "Redirector", "Director", "Doctor"]);
      expect(store.units()).toHaveLength(81);
      expect(store.units().every((u) => u.origin === "seed" && u.authorId === null && u.parentId === null && u.createdAt === at.toISOString())).toBe(true);
      expect(store.currentPool()).toEqual({ version: code.version, daySeq: 1, unitIds: code.units.map((u) => u.id), createdAt: at.toISOString() });
      expect(store.stints()).toHaveLength(73);
      expect(store.stints("fighter")).toEqual([{ unitId: "fighter", enteredSeq: 1, leftSeq: null, reason: "seed" }]);
      // Idempotent: a second start writes nothing.
      expect(seedUnits(store, new Date())).toBe(false);
      expect(store.units()).toHaveLength(81);
      expect(poolContent(store).version).toBe(code.version);
    }
  });

  it("keeps the library units buildable and distinct from the live ones", () => {
    const lib = contentOf(CUT_IN_ROUND_4);
    expect(lib.units.map((u) => u.id)).toEqual(["rat", "spike", "wither", "rot", "pediatrician", "redirector", "director", "doctor"]);
    expect(archetypeProblems([...contentOf(ROWS).units, ...lib.units])).toEqual([]);
  });

  it("migrates a world made by main at 9ef07ac: every old row reads back unchanged, the content version stays, runs go on", async () => {
    const dir = mkdtempSync(join(tmpdir(), "arena-786-"));
    dirs.push(dir);
    const oldSql = join(dir, "sql");
    mkdirSync(oldSql);
    for (const f of MAIN_9EF07AC) copyFileSync(join(SQL_DIR, f), join(oldSql, f));
    const path = join(dir, "world.db");

    // The old world: main's schema, played through main's store methods only.
    const old = new SqliteMvpStore(path, oldSql);
    const mainStore = new Proxy(old, {
      get(t, k, r) {
        if (typeof k === "string" && ADDED.includes(k)) return k === "addDayTallies" ? () => {} : () => { throw new Error(`main had no ${k}`); };
        const v = Reflect.get(t, k, r);
        return typeof v === "function" ? v.bind(t) : v;
      },
    });
    const oldRt = mvpRuntime({ content: mvpContent(), store: mainStore, seed: lcg(11), now: () => new Date("2026-10-08T08:00:00.000Z") });
    expect(await seedChampion(oldRt)).toBeDefined();
    for (let i = 0; i < 4; i++) playBotRun(oldRt);
    const maks: PlayerRef = { id: "p-maks", name: "makscee", bot: false };
    old.addPlayer(maks);
    const run = startRun(oldRt, maks);
    decide(oldRt, run, { kind: "buy", slot: 0 });
    endDay(oldRt);
    old.putRating({ player: maks, rating: 1234, runs: 7, slays: 1, daysAsChampion: 0, playoffWins: 0 });
    old.setJoinCode("join-1");
    expect(old.db.prepare("SELECT name FROM sqlite_master WHERE name = 'mvp_units'").get()).toBeUndefined();
    const before = dump(old);
    for (const t of ["mvp_runs", "mvp_ghosts", "mvp_battles", "mvp_fusions", "mvp_champions", "mvp_ratings", "mvp_unit_tallies", "mvp_days"])
      expect(before[t]!.length, t).toBeGreaterThan(0);
    const activeBefore = old.activeRun(maks.id)!;
    expect(activeBefore.contentVersion).toBe(mvpContent().version);
    old.close();

    // The new code on the same file: migrate, seed, build the content.
    const store = new SqliteMvpStore(path);
    expect((store.db.prepare("SELECT name FROM mvp_migrations ORDER BY name").all() as { name: string }[]).map((r) => r.name)).toEqual([...MAIN_9EF07AC, "m2-01-units.sql", "m2-03-ideas.sql", "m2-04-ideas.sql"]);
    expect(dump(store, Object.keys(before))).toEqual(before);
    expect(seedUnits(store, new Date())).toBe(true);
    const content = poolContent(store);
    expect(content.version).toBe(mvpContent().version);
    expect(store.db.prepare("SELECT count(*) AS n FROM mvp_units").get()).toEqual({ n: 81 });
    // The pool snapshot is stamped with the day the world was on.
    expect(store.currentPool()!.daySeq).toBe(store.currentDay()!.seq);
    expect(dump(store, Object.keys(before))).toEqual(before);

    // The same run goes on: not ended as content-changed.
    const rt = mvpRuntime({ content, store, seed: lcg(5), now: () => new Date("2026-10-09T08:00:00.000Z") });
    const active = store.activeRun(maks.id)!;
    expect(active).toEqual(activeBefore);
    const res = decide(rt, active, { kind: "reroll" });
    expect(res.run.phase).not.toBe("over");
    expect(store.activeRun(maks.id)?.runId).toBe(activeBefore.runId);
    // A pick is tallied for the day now, beside the per-version tallies.
    decide(rt, store.activeRun(maks.id)!, { kind: "buy", slot: 0 });
    expect(store.dayTallies(store.currentDay()!.seq).units.reduce((n, u) => n + u.picks, 0)).toBe(1);
    store.close();
  });
});
