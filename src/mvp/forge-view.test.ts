import { describe, expect, it } from "vitest";
import { claimFixture, forgeFixture } from "./forge-view";

describe("Codex v2 fixture (#1051)", () => {
  const v = forgeFixture("Tester");
  const ids = new Set(v.units.map((u) => u.id));
  it("names only its own units, each pair once in either order", () => {
    const seen = new Set<string>();
    for (const p of v.pairs) {
      expect(ids.has(p.a) && ids.has(p.b)).toBe(true);
      const key = [p.a, p.b].sort().join("+");
      expect(seen.has(key)).toBe(false);
      seen.add(key);
    }
  });
  it("gives each state what its row draws", () => {
    for (const p of v.pairs) {
      if (p.state === "discovered") expect(p.design && p.claimedBy && p.claimedAt).toBeTruthy();
      else expect(p.design).toBeUndefined();
      if (p.state === "unforged") expect(p.wishes).toBeGreaterThanOrEqual(0);
    }
  });
  it("has a claim moment of three candidates: lean A, lean B, the twist", () => {
    const c = claimFixture();
    expect(c.candidates.map((d) => d.lean)).toEqual(["a", "b", "twist"]);
    expect(c.pair.every((id) => ids.has(id))).toBe(true);
    expect(v.pairs.find((p) => [p.a, p.b].sort().join() === [...c.pair].sort().join())?.state).toBe("ready");
  });
});
