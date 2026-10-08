import { describe, expect, it } from "vitest";
import { cardIcons } from "./card-icons.js";
import { mvpPool } from "./units.js";

describe("card icon line (R3-4)", () => {
  const pool = mvpPool();
  const unit = (name: string) => pool.units.find((u) => u.name === name)!;
  const ids = (name: string, form: "sleeping" | "awoken" = "sleeping") => cardIcons(unit(name).forms[form], pool.abilities).map((c) => c.icon);

  it("reads When, then Who, then each Does", () => {
    expect(ids("Fighter")).toEqual(["crossed-swords", "targeted", "spiky-explosion"]);
    expect(ids("Taser", "awoken")).toEqual(["flying-flag", "targeted", "snowflake-2", "spiky-explosion"]);
    expect(cardIcons(unit("Taser").forms.awoken, pool.abilities).map((c) => c.role)).toEqual(["when", "who", "does", "does"]);
  });

  it("drops a repeated icon within a role", () => {
    for (const u of pool.units)
      for (const f of ["sleeping", "awoken"] as const) {
        const got = cardIcons(u.forms[f], pool.abilities).map((c) => `${c.role}:${c.icon}`);
        expect(new Set(got).size, `${u.name} ${f}`).toBe(got.length);
      }
  });

  it("keeps a Does that shares its icon with the When (R3-8: the Awoken's new part must show)", () => {
    // {when: "Ally healed", who: it, does: ["Heal 1"]}
    const got = cardIcons({ when: unit("Sanctifier").forms.sleeping.when, who: unit("Nurse").forms.sleeping.who, does: ["Heal 1"] }, pool.abilities);
    expect(got.map((c) => c.role)).toEqual(["when", "who", "does"]);
    expect(got[0]!.icon).toBe(got[2]!.icon);
  });

  it("marks whose event a trigger is with a pip, so Dies and Ally dies differ", () => {
    const spore = cardIcons(unit("Spore").forms.sleeping, pool.abilities);
    const necro = cardIcons(unit("Necromancer").forms.sleeping, pool.abilities);
    expect(spore[0]).toMatchObject({ icon: "death-skull", label: "Dies" });
    expect(spore[0]!.pip).toBeUndefined();
    expect(necro[0]).toMatchObject({ icon: "death-skull", label: "Ally dies", pip: "ally" });
    expect(necro.map((c) => c.icon)).toEqual(["death-skull", "tombstone", "raise-zombie"]);
  });

  it("gives every unit in the pool a When, a Who or Does, in both forms", () => {
    for (const u of pool.units)
      for (const f of ["sleeping", "awoken"] as const) {
        const got = cardIcons(u.forms[f], pool.abilities);
        expect(got[0]?.role, `${u.name} ${f}`).toBe("when");
        expect(got.length, `${u.name} ${f}`).toBeGreaterThanOrEqual(2);
      }
  });

  it("ignores an authored text line: the icons come from the parts", () => {
    const f = unit("Fighter").forms.sleeping;
    expect(cardIcons({ ...f, text: "Hits hard." }, pool.abilities)).toEqual(cardIcons(f, pool.abilities));
  });
});
