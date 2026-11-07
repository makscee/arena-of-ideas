import { describe, expect, test } from "vitest";
import { chainCount, chainShown, wireMiddle } from "./chain-wire";

describe("the chain's tray count", () => {
  test("a lone step is no chain", () => {
    expect(chainShown(1)).toBe(false);
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
