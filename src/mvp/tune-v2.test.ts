// The gate on batch 1's balance (#1046, the orchestrator's reopen cases 3 and
// 4): docs/mvp/tune-v2.json is what `npm run mvp:tune-v2` measured on the
// numbers in src/mvp/units-v2-numbers.ts, and what the report page shows.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { ROWS_V2 } from "./units-v2.js";

type TuneUnit = { name: string; pwr: number; hp: number; awoken: { pwr: number; hp: number }; fieldScore: number; awokenMargin: number; inBand: boolean };
const tune = JSON.parse(readFileSync(new URL("../../docs/mvp/tune-v2.json", import.meta.url), "utf8")) as { median: number; band: number; healthy: boolean; units: TuneUnit[] };

describe("rules v2 tune (batch 1)", () => {
  it("measured the numbers the pool plays", () => {
    for (const u of tune.units) {
      const row = ROWS_V2.find((r) => r.name === u.name)!;
      expect({ name: u.name, pwr: row.pwr, hp: row.hp, awoken: row.awoken }).toEqual({ name: u.name, pwr: u.pwr, hp: u.hp, awoken: u.awoken });
    }
  });
  it("the full meta is healthy", () => expect(tune.healthy).toBe(true));
  it("awakening is an upgrade for every unit", () => {
    expect(tune.units.filter((u) => u.awokenMargin <= 0).map((u) => `${u.name} ${u.awokenMargin}`)).toEqual([]);
  });
  // The orchestrator's ruling (10-14): the bar plus the measure's ±2 noise.
  it("at least 16 of 20 in band, none under 33% or over 67%", () => {
    expect(tune.units.filter((u) => u.inBand).length).toBeGreaterThanOrEqual(16);
    expect(tune.units.filter((u) => u.fieldScore < 0.33 || u.fieldScore > 0.67).map((u) => `${u.name} ${u.fieldScore}`)).toEqual([]);
  });
});
