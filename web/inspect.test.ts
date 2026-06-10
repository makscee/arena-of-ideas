// unitDefs tests — the inspector's id → UnitDef resolution. Roster units map
// by line order; a summoned unit's def is recovered from the summon effect on
// its source ability, so a mid-battle arrival can still show its abilities.

import { describe, expect, test } from "vitest";
import { Imp, Summoner, battle, stressRegistry, type UnitDef } from "../src/index.js";
import { unitDefs } from "./inspect.js";

const dummy = (name: string, hp = 10, pwr = 3): UnitDef => ({ name, base: { hp, pwr } });

describe("unitDefs", () => {
  test("roster units map to their defs by line order", () => {
    const teams = { A: [dummy("One"), dummy("Two")], B: [dummy("Three")] };
    const log = battle({ teamA: teams.A, teamB: teams.B, seed: 0 });
    const defs = unitDefs(log, teams, stressRegistry);
    expect(defs.get("A1:One")).toBe(teams.A[0]);
    expect(defs.get("A2:Two")).toBe(teams.A[1]);
    expect(defs.get("B1:Three")).toBe(teams.B[0]);
  });

  test("a summoned unit's def is recovered from the summoning ability", () => {
    const teams = { A: [Summoner], B: [dummy("Bruiser", 20, 7)] };
    const log = battle({ teamA: teams.A, teamB: teams.B, seed: 0, statuses: stressRegistry });
    const summon = log.find((e) => e.type === "Summon");
    expect(summon).toBeDefined();
    const defs = unitDefs(log, teams, stressRegistry);
    expect(defs.get((summon as { unit: string }).unit)?.name).toBe(Imp.name);
  });
});
