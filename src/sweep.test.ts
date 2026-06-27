// Sweep helper tests — the shared win-rate distribution used by the CLI's
// --sweep mode and the web gauntlet.

import { describe, expect, test } from "vitest";
import { battle, winnerOf } from "./battle.js";
import { summarizeSweep, sweep, sweepOutcome, sweepSeeds, winRate } from "./sweep.js";
import { stressRegistry, Venomancer, Summoner } from "./content/stress.js";
import { stressAbilities } from "./content/stress.js";
import type { SweepInput } from "./sweep.js";
import type { UnitDef } from "./types.js";

const grunt = (name: string, hp: number, pwr: number): UnitDef => ({ name, base: { hp, pwr } });

// A matchup with mixed outcomes across seeds (first-striker roll decides):
// two identical glass cannons one-shot each other, so each seed is a coin.
const COIN: SweepInput = { teamA: [grunt("Cannon", 1, 9)], teamB: [grunt("Mirror", 1, 9)] };

// A stress-set matchup, statuses included — exactly what the gauntlet runs.
const STRESS: SweepInput = {
  teamA: [Venomancer, grunt("Wall", 12, 1)],
  teamB: [Summoner, grunt("Brute", 8, 2)],
  statuses: stressRegistry,
  abilities: stressAbilities,
};

describe("sweepOutcome", () => {
  test("matches battle() + winnerOf for the same seed", () => {
    for (const seed of [0, 7, 31337]) {
      const o = sweepOutcome(STRESS, seed);
      const log = battle({ ...STRESS, seed });
      const end = log[log.length - 1]!;
      expect(o.seed).toBe(seed);
      expect(o.winner).toBe(winnerOf(log));
      expect(end.type === "BattleEnd" && end.turns).toBe(o.turns);
    }
  });
});

describe("sweepSeeds", () => {
  test("yields n outcomes at consecutive seeds from startSeed", () => {
    const outcomes = [...sweepSeeds(COIN, 5, 10)];
    expect(outcomes.map((o) => o.seed)).toEqual([10, 11, 12, 13, 14]);
  });

  test("a chunked drain sees the same outcomes as a full sweep", () => {
    const full = sweep(STRESS, 20);
    const chunked = [...sweepSeeds(STRESS, 10), ...sweepSeeds(STRESS, 10, 10)];
    expect(chunked).toEqual(full.outcomes);
  });
});

describe("summarizeSweep", () => {
  test("counts sum to n and turns accumulate", () => {
    const stats = summarizeSweep([...sweepSeeds(STRESS, 30)]);
    expect(stats.n).toBe(30);
    expect(stats.aWins + stats.bWins + stats.draws).toBe(30);
    expect(stats.totalTurns).toBeGreaterThanOrEqual(30); // every battle ≥ 1 turn
  });

  test("empty outcomes give zeroes", () => {
    expect(summarizeSweep([])).toEqual({ n: 0, aWins: 0, bWins: 0, draws: 0, totalTurns: 0 });
  });
});

describe("sweep", () => {
  test("deterministic — same inputs, identical result", () => {
    expect(sweep(STRESS, 25)).toEqual(sweep(STRESS, 25));
  });

  test("aggregate fields match a summarize of its own outcomes", () => {
    const r = sweep(COIN, 40);
    expect(summarizeSweep(r.outcomes)).toEqual({
      n: r.n,
      aWins: r.aWins,
      bWins: r.bWins,
      draws: r.draws,
      totalTurns: r.totalTurns,
    });
  });

  test("a coin matchup lands wins on both sides across 40 seeds", () => {
    const r = sweep(COIN, 40);
    expect(r.aWins).toBeGreaterThan(0); // first-striker roll goes both ways
    expect(r.bWins).toBeGreaterThan(0);
    expect(r.draws).toBe(0); // back-strike needs both alive — no mutual kill
  });

  test("startSeed shifts the window: sweep(n=5, start=5) is the tail of sweep(n=10)", () => {
    const head = sweep(COIN, 10);
    const tail = sweep(COIN, 5, 5);
    expect(tail.outcomes).toEqual(head.outcomes.slice(5));
  });
});

// The Run ×N band — the editor's win-rate readout. A band is summarizeSweep's
// counts, read as a win-rate fraction; these pin the four properties the editor
// surfaces: deterministic, lopsided extremes, counts close, mirror sane.
describe("Run ×N band (winRate)", () => {
  // A wall that can't be hurt vs a wall that can't hurt: A always wins.
  const LOPSIDED: SweepInput = { teamA: [grunt("Hammer", 50, 50)], teamB: [grunt("Paper", 1, 0)] };

  test("deterministic band: same inputs run twice give the identical band", () => {
    const a = sweep(STRESS, 50);
    const b = sweep(STRESS, 50);
    expect(winRate(a)).toBe(winRate(b)); // byte-identical, not just close
    expect(a.aWins).toBe(b.aWins);
    expect(a.bWins).toBe(b.bWins);
    expect(a.draws).toBe(b.draws);
  });

  test("a lopsided matchup reports ~100% for A and ~0% for B", () => {
    const r = sweep(LOPSIDED, 50);
    expect(winRate(r, "A")).toBe(1);
    expect(winRate(r, "B")).toBe(0);
  });

  test("the counts sum to runs", () => {
    const r = sweep(STRESS, 37);
    expect(r.n).toBe(37);
    expect(r.aWins + r.bWins + r.draws).toBe(37);
  });

  test("a mirror matchup is sane: a side's win-rate is in [0,1] and A+B+draws cover it", () => {
    const mirror: SweepInput = { teamA: [grunt("Twin", 6, 3)], teamB: [grunt("Twin", 6, 3)] };
    const r = sweep(mirror, 40);
    const wA = winRate(r, "A");
    const wB = winRate(r, "B");
    expect(wA).toBeGreaterThanOrEqual(0);
    expect(wA).toBeLessThanOrEqual(1);
    expect(wA + wB + r.draws / r.n).toBeCloseTo(1, 10);
  });

  test("winRate of an empty sweep is 0, not NaN", () => {
    expect(winRate({ n: 0, aWins: 0, bWins: 0, draws: 0, totalTurns: 0 })).toBe(0);
  });
});
