import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { toJSONL } from "../../../src/battle.js";
import { ROWS } from "../../../src/mvp/units.js";
import { contentOf } from "./content.js";
import { FIXTURE_SCENE_NAMES, fixtureBattle, fixtureUnitId, withSceneFixtures } from "./scene-fixtures.js";

const GOLDENS = join(import.meta.dirname, "../../../src/words/goldens");
const tester = { id: "p1", name: "Tester", bot: false };

describe("rules v2 fixture scenes (#1045)", () => {
  it("store each fixture's golden battle under a stable id, with the kernel's roster ids", () => {
    for (const name of FIXTURE_SCENE_NAMES) {
      const b = fixtureBattle(name, tester, "v-test", "2026-10-13T20:00:00.000Z");
      expect(b.battleId).toBe(`fixture-${name}`);
      expect(toJSONL(b.log)).toBe(readFileSync(join(GOLDENS, `fixture-${name}.jsonl`), "utf8"));
      const start = b.log[0];
      expect(start?.type).toBe("BattleStart");
      if (start?.type !== "BattleStart") return;
      expect(b.teamA.map((u) => u.id)).toEqual(start.teams.A.map((u) => u.id));
      expect(b.teamB.map((u) => u.id)).toEqual(start.teams.B.map((u) => u.id));
      for (const u of [...b.teamA, ...b.teamB]) expect(u.recipe.does.length).toBeGreaterThan(0);
    }
  });

  it("serve the fixtures' words beside the pool, the pool's own entries winning", () => {
    const pool = contentOf(ROWS);
    const served = withSceneFixtures(pool);
    expect(served.version).toBe(pool.version);
    expect(served.units).toEqual(pool.units);
    for (const k of Object.keys(pool.abilities)) expect(served.abilities[k]).toBe(pool.abilities[k]);
    for (const k of Object.keys(pool.statuses)) expect(served.statuses[k]).toBe(pool.statuses[k]);
    expect(served.abilities.Gutter).toBeDefined();
    expect(served.statuses.Grains).toBeDefined();
    expect(served.left?.some((u) => u.id === fixtureUnitId("Déjà Vu"))).toBe(true);
    expect(fixtureUnitId("Déjà Vu")).toBe("v2-deja-vu");
  });
});
