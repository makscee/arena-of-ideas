// The gate on batch 1's balance (#1046, the orchestrator's reopen cases 3 and
// 4): docs/mvp/tune-v2.json is what `npm run mvp:tune-v2` measured on the
// numbers in src/mvp/units-v2-numbers.ts, and what the report page shows.
import { readFileSync } from "node:fs";
import { describe, expect, it } from "vitest";
import { inTierRange, ZERO_PWR } from "./tier-stats.js";
import { ROWS_V2 } from "./units-v2.js";

type TuneUnit = { name: string; pwr: number; hp: number; awoken: { pwr: number; hp: number }; fieldScore: number; awokenMargin: number; inBand: boolean };
const tune = JSON.parse(readFileSync(new URL("../../docs/mvp/tune-v2.json", import.meta.url), "utf8")) as { median: number; band: number; healthy: boolean; units: TuneUnit[] };

describe("rules v2 tune (batches 1 to 3, #1048: tuned all together)", () => {
  it("measured the numbers the pool plays, every unit of it", () => {
    expect(tune.units.map((u) => u.name).sort()).toEqual(ROWS_V2.map((r) => r.name).sort());
    for (const u of tune.units) {
      const row = ROWS_V2.find((r) => r.name === u.name)!;
      expect({ name: u.name, pwr: row.pwr, hp: row.hp, awoken: row.awoken }).toEqual({ name: u.name, pwr: u.pwr, hp: u.hp, awoken: u.awoken });
    }
  });
  it("the full meta is healthy", () => expect(tune.healthy).toBe(true));
  // Maks, 10-17 (#1048): a stat range per tier, PWR at least 1 unless the rule is built around 0
  it("every unit's PWR+HP is inside its tier's range, PWR at least 1 but the units built around 0", () => {
    expect(ROWS_V2.filter((r) => !inTierRange(r)).map((r) => `T${r.tier} ${r.name} ${r.pwr}/${r.hp}`)).toEqual([]);
    expect(ROWS_V2.filter((r) => r.pwr < 1 && !(r.name in ZERO_PWR)).map((r) => r.name)).toEqual([]);
  });
  it("awakening is an upgrade for every unit", () => {
    expect(tune.units.filter((u) => u.awokenMargin <= 0).map((u) => `${u.name} ${u.awokenMargin}`)).toEqual([]);
  });
  // The orchestrator's ruling (10-14): the bar is 16 of 20 in band in each
  // batch, none under 33% or over 67%. The rethink (#1063) missed it twice:
  // 30/40 after the first full retune, then 28/40 (batch 1: 13, batch 2: 15;
  // Sniper 69% on its power floor of 1) after the rule-amount pass (the
  // Butterfly 2/3). Maks's ruling on the card: merge with the numbers stated.
  // Pinned to what that tune measured; the next tune that clears the bar
  // puts the 16 and the 33–67% check back. Batch 3 (#1048): the first tune
  // kept 40/60, then Comet, Puppeteer and Volcano's amounts were lowered and
  // the second tune measured 15, 15 and 19 of 20 (Sniper 74% still alone
  // over 67%); merged with the numbers stated, as ruled. Tier stat ranges
  // (Maks, 10-17): after an amount pass on five units the retune of all 60
  // measured 16, 18 and 14 of 20 (48/60, healthy), Kraken 69% alone over 67%;
  // merged with the numbers stated.
  it("the measured gate (#1048, tier ranges): 16, 18 and 14 of 20 in band, Kraken the one unit over 67%", () => {
    const counts = [];
    for (let b = 0; b < ROWS_V2.length; b += 20) {
      const batch = new Set(ROWS_V2.slice(b, b + 20).map((r) => r.name));
      counts.push(tune.units.filter((u) => batch.has(u.name) && u.inBand).length);
    }
    expect(counts).toEqual([16, 18, 14]);
    expect(tune.units.filter((u) => u.fieldScore < 0.33 || u.fieldScore > 0.67).map((u) => u.name)).toEqual(["Kraken"]);
  });
});
