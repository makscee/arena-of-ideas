import { describe, expect, test } from "vitest";
import { statusesShown } from "./status-row";

describe("status row", () => {
  test("a card's status rows show what fits, two-digit stacks included, else leave room for +n", () => {
    const w = 58; // a 360 px phone's card
    expect(statusesShown([1, 2, 3], w)).toBe(3);
    expect(statusesShown([1, 2, 3, 1, 2, 3], w)).toBe(6); // 3 + 3 one-digit chips
    expect(statusesShown([1, 2, 3, 1, 2, 3, 1], w)).toBe(5); // the 6th place is "+2"
    expect(statusesShown([12, 10, 11, 13], w)).toBe(4); // 2 + 2 two-digit chips
    expect(statusesShown([12, 10, 11, 13, 14], w)).toBe(3); // the 4th place is "+2"
    expect(statusesShown([12, 10, 11, 13, 14, 15], w)).toBe(3);
    expect(statusesShown([], w)).toBe(0);
  });
});
