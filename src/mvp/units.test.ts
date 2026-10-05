import { describe, expect, it } from "vitest";
import { contentFormProblems } from "./forms.js";
import { mvpPool } from "./units.js";

describe("MVP pool (slice 7)", () => {
  const pool = mvpPool();
  const content = { version: "test", ...pool };

  it("has about 80 units, every one shippable in both forms", () => {
    expect(pool.units.length).toBeGreaterThanOrEqual(75);
    expect(contentFormProblems(content)).toEqual([]);
  });

  it("gives every unit its own emoji and a tier", () => {
    const emoji = pool.units.map((u) => u.emoji);
    expect(new Set(emoji).size).toBe(emoji.length);
    for (const t of [1, 2, 3, 4]) expect(pool.units.some((u) => u.tier === t)).toBe(true);
  });

  it("contains chain links: units that listen to what other units emit", () => {
    const listened = new Set(pool.units.map((u) => u.forms.sleeping.when[0]!.on.on));
    for (const e of ["StatusApplied", "Heal", "StatChanged", "Summon", "Death"]) expect(listened).toContain(e);
  });
});
