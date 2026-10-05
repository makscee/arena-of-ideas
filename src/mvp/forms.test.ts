import { describe, expect, it } from "vitest";
import { MVP_RULES, type LineUnit, type MvpContent, type UnitContent } from "./contract.js";
import { MvpDecisionError } from "./errors.js";
import { addCopy, fuseCheck, fuseUnits, lineUnitOf, mergeTarget } from "./forms.js";

const form = { when: [], who: [], does: ["a"] };
const unit = (id: string): UnitContent => ({ id, name: id, emoji: "x", tier: 1, base: { pwr: 2, hp: 5 }, forms: { sleeping: form, awoken: form } });
const content: MvpContent = { version: "t", units: [unit("a"), unit("b"), unit("c")], abilities: {}, statuses: {} };

describe("MVP forms seam (slice 2 fills the rules)", () => {
  it("builds a unit with N copies merged", () => {
    expect(lineUnitOf(unit("a"), "u1")).toMatchObject({ uid: "u1", kind: "unit", copies: 1, form: "sleeping", stats: { pwr: 2, hp: 5 } });
    expect(lineUnitOf(unit("a"), "u1", 2).stats).toEqual({ pwr: 3, hp: 7 });
  });

  it("adds a copy without touching the input", () => {
    const u = lineUnitOf(unit("a"), "u1");
    const v = addCopy(u, content, MVP_RULES);
    expect(v).toMatchObject({ copies: 2, stats: { pwr: 3, hp: 7 } });
    expect(u).toMatchObject({ copies: 1, stats: { pwr: 2, hp: 5 } });
  });

  it("finds the merge target, fused parts included", () => {
    const fused: LineUnit = { ...lineUnitOf(unit("b"), "u2"), kind: "fused", fusion: { first: "b", second: "c", name: "Bc", discoveredBy: null } };
    const line = [lineUnitOf(unit("a"), "u1"), fused];
    expect(mergeTarget(line, "a")).toBe(0);
    expect(mergeTarget(line, "b")).toBe(1);
    expect(mergeTarget(line, "c")).toBe(1);
    expect(mergeTarget(line, "d")).toBe(-1);
  });

  it("refuses fusion until slice 2", () => {
    const a = lineUnitOf(unit("a"), "u1");
    const b = lineUnitOf(unit("b"), "u2");
    expect(fuseCheck(a, b)).toBe("fusion arrives in slice 2");
    expect(() => fuseUnits(a, b, { name: "Ab", discoveredBy: null }, content)).toThrow(MvpDecisionError);
  });
});
