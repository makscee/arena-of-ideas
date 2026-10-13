import { readFileSync } from "node:fs";
import { it } from "vitest";
import { TUNE_DEFAULT, fieldOfReport, liveBand, scrambleRow, slugOf, tuneUnit } from "./tune.js";
import { ROWS } from "./units.js";
it("dbg", () => {
  const report = JSON.parse(readFileSync("docs/mvp/meta-health.json", "utf8"));
  const all = fieldOfReport(report, ROWS.map((r) => slugOf(r.name)));
  const field = Array.from({ length: 12 }, (_, i) => all[Math.floor((i * all.length) / 12)]!);
  const s = { ...TUNE_DEFAULT, gauntlet: 4, searchSeeds: 1, searchSteps: 8, maxEvals: 20, seed: 1 };
  const ids = ["fighter", "nurse", "venomancer", "keeper", "bulwark", "commander", "ruin", "saboteur", "medic", "rose"];
  const band = liveBand(ROWS, field, s, ids);
  console.log("band", JSON.stringify({ low: band.low, high: band.high, target: band.target }));
  for (const name of ["Leech"]) for (const seed of [1,2,4,5,6,7,8,9,10,11,12]) {
    const base = ROWS.find((r) => r.name === name)!;
    const scrambled = scrambleRow({ ...base, name: name + " Copy" }, ROWS, seed);
    const r = tuneUnit(scrambled, { liveRows: ROWS, field, band, settings: s, meta: null });
    console.log(name, seed, "start", r.steps[0]!.measure.score, "pass", r.pass, r.reason, "end", r.measure.score);
  }
}, 120000);
