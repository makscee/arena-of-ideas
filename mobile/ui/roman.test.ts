// Tiers read as Roman numerals everywhere (R3-3).
import { expect, test } from "vitest";
import { roman } from "./card";

test("roman: 1-10 as I-X, anything else as digits", () => {
  expect([1, 2, 3, 4].map(roman)).toEqual(["I", "II", "III", "IV"]);
  expect(roman(9)).toBe("IX");
  expect(roman(10)).toBe("X");
  expect(roman(0)).toBe("0");
  expect(roman(11)).toBe("11");
});
