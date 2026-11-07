import { describe, expect, test } from "vitest";
import { LADDER_ROWS, ladderAbove } from "./ladder";

const seq = ["s0", "s1", "s2", "s3", "s4", "s5"];

describe("the tray's ladder", () => {
  test("has four rows, the foot included", () => {
    expect(LADDER_ROWS).toBe(4);
    expect(ladderAbove(seq, 3, false)).toHaveLength(3);
  });
  test("the face-off has no steps: every row above the foot is empty but present", () => {
    expect(ladderAbove(seq, -1, false)).toEqual([null, null, null]);
    expect(ladderAbove(seq, 0, false)).toEqual([null, null, null]);
  });
  test("the first steps fill from the foot up", () => {
    expect(ladderAbove(seq, 1, false)).toEqual([null, null, "s0"]);
    expect(ladderAbove(seq, 2, false)).toEqual([null, "s0", "s1"]);
  });
  test("the three steps before the newest stay, across beats and turns", () => {
    expect(ladderAbove(seq, 5, false)).toEqual(["s2", "s3", "s4"]);
  });
  test("a turn's totals take the foot row: the step on screen is the newest above it", () => {
    expect(ladderAbove(seq, 5, true)).toEqual(["s3", "s4", "s5"]);
    expect(ladderAbove(seq, 0, true)).toEqual([null, null, "s0"]);
    expect(ladderAbove(seq, -1, true)).toEqual([null, null, null]);
  });
});
