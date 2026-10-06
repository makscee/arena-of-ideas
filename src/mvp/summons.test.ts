// Summoned units have cards (round 3, R3-5, docs/round3/words.md (3)).
import { describe, expect, it } from "vitest";
import { summonId } from "../describe.js";
import { formSegments, formText } from "./form-text.js";
import { mvpPool } from "./units.js";

describe("summoned units (R3-5)", () => {
  const pool = mvpPool();
  const unit = (name: string) => pool.units.find((u) => u.name === name)!;

  it("lists the 5 summoned bodies, each with an id, an emoji and its numbers", () => {
    expect(pool.summons.map((s) => s.name).sort()).toEqual(["Golem", "Imp", "Treant", "Wolf", "Wraith"]);
    for (const s of pool.summons) {
      expect(s.id).toBe(summonId(s.name));
      expect(s.emoji).toMatch(/\p{Extended_Pictographic}/u);
      expect(s.base.pwr).toBeGreaterThan(0);
    }
    expect(pool.summons.find((s) => s.id === "imp")).toMatchObject({ emoji: "👺", base: { pwr: 1, hp: 2 }, form: null });
  });

  it("resolves every summon effect's unit to exactly one body", () => {
    const ids = new Set(pool.summons.map((s) => s.id));
    expect(ids.size).toBe(pool.summons.length);
    for (const ab of Object.values(pool.abilities))
      for (const e of ab.effects)
        if (e.kind === "summon") expect(ids.has(summonId(e.unit.name)), e.unit.name).toBe(true);
  });

  it("names Planter's Imp as a unit-ref run, and the joined text is unchanged", () => {
    const form = unit("Planter").forms.sleeping;
    const segs = formSegments(form, pool.abilities);
    expect(segs.map((s) => s.text).join("")).toBe(formText(form, pool.abilities));
    expect(formText(form, pool.abilities)).toMatch(/^Battle start: summon Imp \(1\/2\), /);
    const ref = segs.find((s) => s.unitRef);
    expect(ref).toMatchObject({ text: "Imp (1/2)", unitRef: "imp", side: "ally" });
    expect(segs[segs.indexOf(ref!) - 1]).toMatchObject({ text: "summon ", term: "effect:summon" });
    const awoken = formSegments(unit("Planter").forms.awoken, pool.abilities);
    expect(awoken.find((s) => s.unitRef)?.unitRef).toBe("treant");
  });
});
