import { readFileSync } from "node:fs";
import { beforeAll, describe, expect, it } from "vitest";
import { TEST_ROWS, TUNE_DEFAULT, fieldOfReport, knobsOf, liveBand, numbersOf, scrambleRow, slugOf, tuneUnit, withNumbers, type Band, type TuneSettings } from "./tune.js";
import { ROWS, awokenKeeps, mvpPool, type Row } from "./units.js";
import type { Team } from "./meta.js";

const live = (name: string): Row => ROWS.find((r) => r.name === name)!;

describe("a row's numbers", () => {
  it("reads and writes every Does number in order", () => {
    const row = live("Famin"); // "Poison 1", Awoken more "Poison 1 + Curse 1"
    expect(numbersOf(row)).toEqual([
      { word: "Poison", value: 1, awokenOnly: false },
      { word: "Poison", value: 1, awokenOnly: true },
      { word: "Curse", value: 1, awokenOnly: true },
    ]);
    const r2 = withNumbers(row, [2, 3, 1]);
    expect(r2.does).toBe("Poison 2");
    expect(r2.awoken.more).toBe("Poison 3 + Curse 1");
    expect(row.does).toBe("Poison 1");
  });

  it("scrambles inside the live bounds and keeps Awoken a superset", () => {
    for (const name of ["Venomancer", "Famin", "Mesmerist", "Fertilizer"]) {
      const row = { ...live(name), name: `${name} Copy` };
      const r = scrambleRow(row, ROWS, 5);
      for (const k of knobsOf(row, ROWS)) {
        expect(k.get(r)).toBeGreaterThanOrEqual(k.min);
        expect(k.get(r)).toBeLessThanOrEqual(k.max);
      }
      const u = mvpPool([r]).units[0]!;
      expect(awokenKeeps(u.forms.sleeping, u.forms.awoken)).toBeNull();
    }
  });
});

// A small field: 12 of today's breaker teams, spread over the report.
describe("tuneUnit on a small field", () => {
  const report = JSON.parse(readFileSync("docs/mvp/meta-health.json", "utf8"));
  const all = fieldOfReport(report, ROWS.map((r) => slugOf(r.name)));
  const field: Team[] = Array.from({ length: 12 }, (_, i) => all[Math.floor((i * all.length) / 12)]!);
  const s: TuneSettings = { ...TUNE_DEFAULT, gauntlet: 4, searchSeeds: 1, searchSteps: 8, maxEvals: 20, seed: 1 };
  const ids = ["fighter", "nurse", "venomancer", "keeper", "bulwark", "commander", "ruin", "saboteur", "medic", "rose"];
  let band: Band;
  beforeAll(() => {
    band = liveBand(ROWS, field, s, ids);
  }, 120_000);

  it("derives the band from the live units' spread", () => {
    expect(band.live.map((l) => l.unit)).toEqual(ids);
    expect(band.low).toBeLessThan(band.target);
    expect(band.target).toBeLessThan(band.high);
    expect(band.margin).toBeGreaterThanOrEqual(s.awokenMarginFloor);
  });

  it("tunes a scrambled copy of a live unit back into the band", () => {
    const scrambled = scrambleRow({ ...live("Venomancer"), name: "Venomancer Copy" }, ROWS, 3);
    const r = tuneUnit(scrambled, { liveRows: ROWS, field, band, settings: s, meta: null });
    // It started out of the band and was moved into it.
    expect(r.steps[0]!.measure.score).toBeLessThan(band.low);
    expect(r.pass, r.reason).toBe(true);
    expect(r.measure.score).toBeGreaterThanOrEqual(band.low);
    expect(r.measure.score).toBeLessThanOrEqual(band.high);
    expect(r.reason).toMatch(/^fits/);
    expect(r.row).not.toEqual(scrambled);
  }, 120_000);

  it("fails a unit no numbers can tame, with a reason a player can read", () => {
    const r = tuneUnit(TEST_ROWS[0]!, { liveRows: ROWS, field, band, settings: s, meta: null });
    expect(r.pass).toBe(false);
    expect(r.reason).toMatch(/^too strong in a \w+ team, even at its lowest numbers$/);
    expect(r.row.pwr).toBe(Math.min(...ROWS.map((x) => x.pwr)));
  }, 120_000);

  it("refuses a candidate that reuses a live unit's id", () => {
    expect(() => tuneUnit(live("Rose"), { liveRows: ROWS, field, band, settings: s, meta: null })).toThrow(/already a live unit/);
  });
});
