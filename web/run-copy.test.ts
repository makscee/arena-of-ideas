// Economy copy (slice 5, IA-6): the strings the shop surfaces — income line,
// stakes line, fusion pips — are derived from the tunables, never typed.
// These tests assert against the same exports the run screen reads, so
// retuning a knob cannot leave the UI lying.

import { describe, expect, test } from "vitest";
import { INCOME_PER_ROUND, STACK_THRESHOLD, incomeForRound } from "../src/index.js";
import { fusionPips, incomeLine, stakesLine } from "./run-screen.js";

describe("incomeLine derives from incomeForRound()", () => {
  test("cites the curve's figure for the upcoming round", () => {
    for (const round of [1, 5, 20]) {
      expect(incomeLine(round)).toContain(`+${incomeForRound(round + 1)}g`);
    }
  });

  test("phrasing matches the curve's shape: 'each round' iff the curve is flat", () => {
    if (INCOME_PER_ROUND === 0) expect(incomeLine(1)).toContain("each round");
    else expect(incomeLine(1)).toContain("next round");
  });
});

describe("stakesLine", () => {
  test("names the current lives count", () => {
    expect(stakesLine(4)).toBe("a loss costs a life — 4 lives left");
    expect(stakesLine(1)).toBe("a loss costs a life — 1 life left");
  });
});

describe("fusionPips track copies against STACK_THRESHOLD", () => {
  test("filled pips = copies held, total = the threshold", () => {
    for (let stacks = 1; stacks < STACK_THRESHOLD; stacks++) {
      const pips = fusionPips(stacks);
      expect(pips).toHaveLength(STACK_THRESHOLD);
      expect(pips.split("●").length - 1).toBe(stacks);
      expect(pips.split("○").length - 1).toBe(STACK_THRESHOLD - stacks);
    }
  });

  test("over-threshold stacks clamp instead of overflowing the row", () => {
    expect(fusionPips(STACK_THRESHOLD + 2)).toHaveLength(STACK_THRESHOLD);
  });
});
