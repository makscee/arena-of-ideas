import { describe, expect, test } from "vitest";
import { chainCount, chainRows, chainShown, wireMiddle } from "./chain-wire";

describe("the chain's tray list", () => {
  test("a lone step is no chain", () => {
    expect(chainShown(1)).toBe(false);
    expect(chainRows(["a"], 0)).toEqual([]);
  });
  test("lists the steps so far, numbered from 1, the newest lit", () => {
    const rows = chainRows(["a", "b", "c", "d"], 2);
    expect(rows.map((r) => [r.n, r.step, r.now])).toEqual([[1, "a", false], [2, "b", false], [3, "c", true]]);
  });
  test("never shows a step still to come", () => {
    expect(chainRows(["a", "b", "c"], 0).map((r) => r.step)).toEqual(["a"]);
  });
  test("CHAIN ×N counts the links landed", () => {
    expect(chainCount(0)).toBe(1);
    expect(chainCount(3)).toBe(4);
  });
});

describe("the wire's number", () => {
  test("sits in the middle of the curve", () => {
    expect(wireMiddle({ x: 0, y: 0 }, { x: 50, y: -40 }, { x: 100, y: 0 })).toEqual({ x: 50, y: -20 });
  });
});
