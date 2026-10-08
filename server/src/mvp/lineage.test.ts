// M3-3 (mission #800): lineage. A world made by mission 2 opens unchanged
// with every unit its own root and every idea "new"; a 3-version chain reads
// back root first.
import { copyFileSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import { ideaKind, type Idea } from "../../../src/mvp/contract.js";
import { ROWS } from "../../../src/mvp/units.js";
import { lineage, rootOf } from "./lineage.js";
import { seedUnits, swapUnit } from "./pool.js";
import { migrateMvp, SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore, type MvpStore, type StoredUnit } from "./store.js";

const dirs: string[] = [];
afterEach(() => { for (const d of dirs.splice(0)) rmSync(d, { recursive: true, force: true }); });

/** Every row of every table, mvp_migrations included, in rowid order. */
function dump(store: SqliteMvpStore): Record<string, unknown[]> {
  const names = (store.db.prepare("SELECT name FROM sqlite_master WHERE type = 'table' AND name LIKE 'mvp_%' ORDER BY name").all() as { name: string }[]).map((r) => r.name);
  return Object.fromEntries(names.map((n) => [n, store.db.prepare(`SELECT * FROM ${n} ORDER BY rowid`).all()]));
}

describe("lineage (M3-3)", () => {
  it("opens a mission-2 world unchanged: no migration, every unit its own root, every idea new", () => {
    const dir = mkdtempSync(join(tmpdir(), "arena-803-"));
    dirs.push(dir);
    const path = join(dir, "world.db");
    const old = new SqliteMvpStore(path);
    expect(seedUnits(old, new Date("2026-10-08T08:00:00.000Z"))).toBe(true);
    swapUnit(old, "fighter", "rat", new Date("2026-10-08T09:00:00.000Z"));
    const at = "2026-10-08T08:00:00.000Z";
    const ideas: Idea[] = [
      { ideaId: "i1", playerId: "p1", text: "a hedgehog that bristles", state: "written", createdAt: at, data: {} },
      { ideaId: "i2", playerId: "p1", text: "a bard who sings the line awake", state: "failed", createdAt: at, data: { failure: "too close to Bard", tries: 1 } },
      { ideaId: "i3", playerId: "p2", text: "a smith who sharpens neighbours", state: "voting", createdAt: at, data: { unitId: "idea-smith", checkedAt: at } },
    ];
    for (const i of ideas) old.putIdea(i);
    old.putIdeaCounts("p1", { spent: 2, granted: 1, forfeited: 0 });
    const before = dump(old);
    old.close();
    const copy = join(dir, "copy.db");
    copyFileSync(path, copy);

    // The new code on a copy: nothing to migrate (mvp_migrations is in the dump), nothing written.
    const store = new SqliteMvpStore(copy);
    expect(dump(store)).toEqual(before);
    expect(migrateMvp(store.db)).toEqual([]);
    expect(store.units({ status: "live" })).toHaveLength(73);
    expect(store.units({ status: "library" })).toHaveLength(8);
    for (const u of store.units()) {
      expect(u.rootId).toBeUndefined();
      expect(rootOf(store, u.unitId)).toBe(u.unitId);
      expect(lineage(store, u.unitId)).toEqual([u]);
    }
    expect(store.ideas().map(ideaKind)).toEqual(["new", "new", "new"]);
    expect(store.ideas().map((i) => i.data.target)).toEqual([undefined, undefined, undefined]);
    expect(store.ideas({ target: "fighter" })).toEqual([]);
    expect(dump(store)).toEqual(before);
    store.close();
  });

  for (const [name, make] of [["memory", () => new MemoryMvpStore()], ["sqlite", () => new SqliteMvpStore(":memory:")]] as [string, () => MvpStore][])
    it(`reads a 3-version chain root first, from any version (${name})`, () => {
      const store = make();
      const unit = (unitId: string, i: number, extra: Partial<StoredUnit> = {}): StoredUnit =>
        ({ unitId, status: "library", row: ROWS[i]!, authorId: null, origin: "seed", parentId: null, createdAt: "2026-10-08T08:00:00.000Z", ...extra });
      const v1 = unit("quillback", 0);
      const other = unit("fodder", 1, { status: "live" });
      const v2 = unit("quillback-2", 0, { authorId: "p-kim", origin: "evolution", parentId: "quillback", rootId: "quillback" });
      const v3 = unit("quillback-3", 0, { status: "live", authorId: "p-lev", origin: "evolution", parentId: "quillback-2", rootId: "quillback" });
      for (const u of [v1, other, v2, v3]) store.putUnit(u);
      for (const id of ["quillback", "quillback-2", "quillback-3"]) {
        expect(rootOf(store, id)).toBe("quillback");
        expect(lineage(store, id)).toEqual([v1, v2, v3]);
      }
      expect(lineage(store, "fodder")).toEqual([other]);
      expect(rootOf(store, "nobody")).toBe("nobody");
      expect(lineage(store, "nobody")).toEqual([]);
    });
});
