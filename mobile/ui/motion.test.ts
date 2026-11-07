import { describe, expect, test } from "vitest";
import { lungeVector, LUNGE_MAX_PX } from "./motion";

describe("the striker's lunge (arena#50)", () => {
  test("goes along the real source–target vector, not a fixed up or down", () => {
    const v = lungeVector(60, 80);
    expect(v.x / v.y).toBeCloseTo(60 / 80);
    expect(Math.hypot(v.x, v.y)).toBeCloseTo(28);
  });
  test("a straight-across strike has no sideways part, either way", () => {
    expect(lungeVector(0, 100).x).toBe(0);
    expect(lungeVector(0, 100).y).toBeCloseTo(28);
    expect(lungeVector(0, -100).y).toBeCloseTo(-28);
    expect(lungeVector(-100, 0).x).toBeCloseTo(-28);
  });
  test("is capped at 32 px and never reaches the target", () => {
    const far = lungeVector(1000, 0);
    expect(far.x).toBe(LUNGE_MAX_PX);
    const near = lungeVector(10, 0);
    expect(near.x).toBeLessThan(10);
  });
  test("a target on the same spot gives no motion", () => {
    expect(lungeVector(0, 0)).toEqual({ x: 0, y: 0 });
  });
});
