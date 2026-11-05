import { describe, expect, it } from "vitest";
import { FIELD, GROW_MAX, linePitch, pieceCap, REACH_X, SHOP_PX, spreadLine } from "./line-layout";

describe("line layout (arena#29 round 4)", () => {
  it("a line's columns fit the field, 1 to 8 units, on both stages", () => {
    for (const o of ["L", "P"] as const) for (let n = 1; n <= 8; n++) expect(linePitch(o, n) * n).toBeLessThanOrEqual(FIELD[o].width);
  });
  it("a short line is spaced closer than a full one, never wider than the shop's slots would allow a piece", () => {
    expect(linePitch("L", 2)).toBeLessThanOrEqual(linePitch("L", 1) + 0);
    expect(linePitch("L", 5)).toBe(160);
    expect(linePitch("P", 5)).toBeLessThanOrEqual(Math.floor(366 / 5));
  });
  it("pieces are at least 0.9 of the shop's and at most 1.4× it, up to five units a side", () => {
    for (const o of ["L", "P"] as const) for (let n = 1; n <= 5; n++) {
      const s = pieceCap(o, n);
      expect(s).toBeGreaterThanOrEqual(SHOP_PX[o] * 0.9);
      expect(s).toBeLessThanOrEqual(SHOP_PX[o] * GROW_MAX);
    }
  });
  it("a long line (past five) shrinks with its columns", () => {
    expect(pieceCap("L", 8)).toBeLessThan(pieceCap("L", 5));
    expect(pieceCap("P", 8)).toBeLessThan(pieceCap("P", 5));
  });
  it("a full-width line's pieces leave their neighbours' reach clear (390 px, five units: faces of 48, 30 px apart)", () => {
    const { pitch, px } = spreadLine(390, 5);
    expect(pitch).toBe(78);
    expect(px).toBeGreaterThanOrEqual(48);
    expect(pitch - px).toBeGreaterThanOrEqual(px * REACH_X);
  });
});
