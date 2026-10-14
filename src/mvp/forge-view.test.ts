import { describe, expect, it } from "vitest";
import { claimFixture, forgeDesignOf, forgeFixture } from "./forge-view";
import type { FusionDesign } from "./codex";

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

describe("a designed fusion's own words (#1120)", () => {
  // Mirror + Raven's Grave Echo as the night batch stored it: its Does are
  // abilities it coined, which the pool's registry doesn't have.
  const graveEcho: FusionDesign = {
    name: "Grave Echo",
    lean: "a",
    recipe: { when: [{ kind: "trigger", on: { on: "Death", unit: "otherAlly" } }], who: [{ kind: "eventUnit" }], does: ["Keep"], also: [{ who: [{ kind: "holder" }], does: ["Inherit"] }] },
    abilities: {
      Keep: { name: "Keep", family: "Arcane", effects: [{ kind: "keepRule" }] },
      Inherit: { name: "Inherit", family: "Control", effects: [{ kind: "applyStatus", status: "Strength", stacks: { kind: "statOf", stat: "pwr", of: "eventUnit" } }] },
    },
    statuses: { Strength: { name: "Strength", statMods: { pwr: 1 }, abilities: [] } },
  } as FusionDesign;
  /** A raw key on screen: an ability id the sentence couldn't say, or a camelCase/dotted key. */
  const rawKey = (text: string, ids: string[]) => ids.find((id) => new RegExp(`(^|[^\\p{L}])${id}([^\\p{L}]|$)`, "u").test(text)) ?? text.match(/\b[a-z]+[A-Z]\w*\b|\b\w+\.\w+\b/)?.[0];
  it("says its coined abilities as a sentence in EN and RU, never as 'keep inherit'", () => {
    const d = forgeDesignOf(graveEcho, [undefined, undefined], { abilities: {}, statuses: {} });
    for (const text of [d.rule.en, d.rule.ru]) {
      expect(text.length).toBeGreaterThan(20);
      expect(rawKey(text, ["Keep", "Inherit"])).toBeUndefined();
    }
  });
  it("the check catches a raw key", () => {
    const bare = forgeDesignOf({ ...graveEcho, abilities: {}, statuses: {} }, [undefined, undefined], { abilities: {}, statuses: {} });
    expect(rawKey(bare.rule.en, ["Keep", "Inherit"])).toBeDefined();
  });
});
