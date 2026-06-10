// Sweep helper tests — the shared win-rate distribution used by the CLI's
// --sweep mode and the web gauntlet.

import { describe, expect, test } from "vitest";
import { battle, winnerOf } from "./battle.js";
import { summarizeSweep, sweep, sweepOutcome, sweepSeeds } from "./sweep.js";
import { stressRegistry, Venomancer, Summoner } from "./content/stress.js";
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
