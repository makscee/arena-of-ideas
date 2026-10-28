import { describe, expect, it } from "vitest";
import { faceFallback, faceProblems, fusedFace, V2_FACE } from "./face.js";
import { mvpPool } from "./units.js";
import { ROWS_V2 } from "./units-v2.js";

describe("the piece's face (#1291)", () => {
  const pool = mvpPool(ROWS_V2);

  it("every v2 unit has a hand-picked face that fits the piece", () => {
    expect(pool.units).toHaveLength(60);
    for (const u of pool.units) {
      expect(V2_FACE[u.name], u.name).toBeDefined();
      expect(faceProblems(u.face), u.name).toEqual([]);
    }
  });

  it("no face is left over for a unit the pool lost", () => {
    const names = new Set(ROWS_V2.map((r) => r.name));
    expect(Object.keys(V2_FACE).filter((n) => !names.has(n))).toEqual([]);
  });

  it("counter units ring their Charge", () => {
    const ringed = pool.units.filter((u) => u.face?.counter).map((u) => u.name).sort();
    expect(ringed).toEqual(["Archmage", "Battery", "Drummer", "Dynamo", "Hourglass", "Witch Doctor"]);
  });

  it("a row with no picked face falls back to its trigger, and a row's own face wins", () => {
    const row = structuredClone(ROWS_V2.find((r) => r.name === "Hornet")!);
    const fresh = { ...row, name: "Newcomer" };
    expect(mvpPool([fresh]).units[0]!.face).toEqual({ when: { icon: "⚔️", label: "Strikes" }, headline: { icon: "✨" } });
    const own = { ...fresh, face: { when: { icon: "🔔", label: "When healed" }, headline: { icon: "🔁", amount: "=" } } };
    expect(mvpPool([own]).units[0]!.face).toEqual(own.face);
  });

  it("the fallback fits for every trigger in the pool", () => {
    for (const u of pool.units) expect(faceProblems(faceFallback(u.forms.sleeping)), u.name).toEqual([]);
  });

  it("a fusion shows its first part's when and headline", () => {
    expect(fusedFace(V2_FACE.Hourglass!)).toEqual(V2_FACE.Hourglass);
  });

  it("flags a label that won't fit", () => {
    expect(faceProblems({ when: { icon: "🌅", label: "At the very start" }, headline: { icon: "💥" } })).toHaveLength(1);
    expect(faceProblems({ when: { icon: "🌅", label: "From turn 3" }, headline: { icon: "💥", amount: "×2" } })).toEqual([]);
  });
});
