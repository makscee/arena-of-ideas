// Golden tests for forms, copies, Awoken and fusion (mission #574, slice 2).

import { describe, expect, it } from "vitest";
import { battle } from "../battle.js";
import { stressAbilities, stressRegistry } from "../content/stress.js";
import type { AbilityDef, AbilityRegistry, BattleEvent } from "../types.js";
import { MVP_RULES, type LineUnit, type MvpContent, type PlayerRef, type UnitContent } from "./contract.js";
import { addCopy, contentFormProblems, formProblems, fuseCheck, fuseUnits, lineUnitOf, mergeTarget } from "./forms.js";
import { toBattleDef } from "./fight.js";

const ab = (name: string, family: AbilityDef["family"], effects: AbilityDef["effects"]): AbilityDef => ({ name, family, effects });
const n = (value: number) => ({ kind: "const" as const, value });

const abilities: AbilityRegistry = {
  ...stressAbilities,
  GiveShield: ab("GiveShield", "Shield", [{ kind: "applyStatus", status: "Shield", stacks: n(2) }]),
  Shoot: ab("Shoot", "Strike", [{ kind: "damage", amount: n(2) }]),
  Volley: ab("Volley", "Strike", [{ kind: "damage", amount: n(3) }]),
};

/** At battle start, shields itself; awoken, shields every ally. */
const Warden: UnitContent = {
  id: "warden",
  name: "Warden",
  emoji: "🛡️",
  tier: 1,
  base: { pwr: 1, hp: 6 },
  forms: {
    sleeping: { when: [{ kind: "trigger", on: { on: "BattleStart" } }], who: [{ kind: "holder" }], does: ["GiveShield"] },
    awoken: { when: [{ kind: "trigger", on: { on: "BattleStart" } }], who: [{ kind: "allAllies" }], does: ["GiveShield"] },
  },
};

/** When an ally is hurt, shoots the front enemy; awoken, a harder volley at every enemy. */
const Archer: UnitContent = {
  id: "archer",
  name: "Archer",
  emoji: "🏹",
  tier: 1,
  base: { pwr: 2, hp: 4 },
  forms: {
    sleeping: { when: [{ kind: "trigger", on: { on: "Hurt", unit: "otherAlly" } }], who: [{ kind: "frontEnemy" }], does: ["Shoot"] },
    awoken: { when: [{ kind: "trigger", on: { on: "Hurt", unit: "otherAlly" } }], who: [{ kind: "allEnemies" }], does: ["Volley"] },
  },
};

const content: MvpContent = { version: "forms-golden", units: [Warden, Archer], abilities, statuses: stressRegistry };
const maks: PlayerRef = { id: "p1", name: "maks", bot: false };

/** A unit grown copy by copy, as the shop merges them. */
function withCopies(u: UnitContent, uid: string, copies: number): LineUnit {
  let x = lineUnitOf(u, uid);
  for (let i = 1; i < copies; i++) x = addCopy(x, content, MVP_RULES);
  return x;
}

describe("copies and awakening", () => {
  it("a bought unit is sleeping, with base stats and the sleeping recipe", () => {
    expect(lineUnitOf(Warden, "u1")).toEqual({
      uid: "u1",
      kind: "unit",
      unitId: "warden",
      name: "Warden",
      emoji: "🛡️",
      copies: 1,
      form: "sleeping",
      stats: { pwr: 1, hp: 6 },
      recipe: Warden.forms.sleeping,
    });
  });

  it("each copy adds +1 PWR / +2 HP and the 2nd copy still sleeps", () => {
    const two = withCopies(Warden, "u1", 2);
    expect(two).toMatchObject({ copies: 2, form: "sleeping", stats: { pwr: 2, hp: 8 } });
    expect(two.recipe).toEqual(Warden.forms.sleeping);
  });

  it("the 3rd copy awakens: the recipe swaps to the awoken form, same When", () => {
    const three = withCopies(Warden, "u1", 3);
    expect(three).toMatchObject({ uid: "u1", copies: 3, form: "awoken", stats: { pwr: 3, hp: 10 } });
    expect(three.recipe).toEqual(Warden.forms.awoken);
    expect(three.recipe.when).toEqual(Warden.forms.sleeping.when);
    expect(three.recipe.who).toEqual([{ kind: "allAllies" }]);
  });

  it("copies past the 3rd only add stats", () => {
    expect(withCopies(Warden, "u1", 5)).toMatchObject({ copies: 5, form: "awoken", stats: { pwr: 5, hp: 14 } });
  });

  it("lineUnitOf with copies matches merging them one by one", () => {
    for (const copies of [1, 2, 3, 4]) expect(lineUnitOf(Warden, "u1", copies)).toEqual(withCopies(Warden, "u1", copies));
  });

  it("is pure: adding a copy leaves the original untouched", () => {
    const one = lineUnitOf(Archer, "u2");
    const snapshot = structuredClone(one);
    addCopy(addCopy(one, content, MVP_RULES), content, MVP_RULES);
    expect(one).toEqual(snapshot);
  });

  it("follows the rules' tunables", () => {
    const rules = { ...MVP_RULES, copiesToAwaken: 2, copyGrowth: { pwr: 2, hp: 3 } };
    expect(addCopy(lineUnitOf(Archer, "u2", 1, rules), content, rules)).toMatchObject({ form: "awoken", stats: { pwr: 4, hp: 7 } });
  });
});

describe("fusion", () => {
  const warden = withCopies(Warden, "u1", 3);
  const archer = withCopies(Archer, "u2", 3);

  it("only two Awoken, unfused, different units fuse", () => {
    expect(fuseCheck(warden, archer)).toBeNull();
    expect(fuseCheck(withCopies(Warden, "u1", 2), archer)).toBe("both units must be Awoken");
    expect(fuseCheck(warden, warden)).toBe("a unit can't fuse with itself");
    const fused = fuseUnits(warden, archer, { name: "Wardcher", discoveredBy: maks }, content);
    expect(fuseCheck(fused, withCopies(Archer, "u3", 3))).toBe("a fused unit is final");
    expect(() => fuseUnits(fused, withCopies(Archer, "u3", 3), { name: "x", discoveredBy: null }, content)).toThrow(/final/);
  });

  it("golden: Warden + Archer = When of Warden, Who of Archer, Does of both in order", () => {
    expect(fuseUnits(warden, archer, { name: "Wardcher", discoveredBy: maks }, content)).toEqual({
      uid: "u1",
      kind: "fused",
      unitId: "warden",
      name: "Wardcher",
      emoji: "🛡️🏹",
      copies: 6,
      form: "awoken",
      stats: { pwr: 3 + 4, hp: 10 + 8 },
      recipe: {
        when: [{ kind: "trigger", on: { on: "BattleStart" } }],
        who: [{ kind: "allEnemies" }],
        does: ["GiveShield", "Volley"],
      },
      fusion: { first: "warden", second: "archer", name: "Wardcher", discoveredBy: maks },
    });
  });

  it("golden: Archer + Warden is a different unit", () => {
    const ba = fuseUnits(archer, warden, { name: "Archden", discoveredBy: null }, content);
    expect(ba).toEqual({
      uid: "u2",
      kind: "fused",
      unitId: "archer",
      name: "Archden",
      emoji: "🏹🛡️",
      copies: 6,
      form: "awoken",
      stats: { pwr: 7, hp: 18 },
      recipe: {
        when: [{ kind: "trigger", on: { on: "Hurt", unit: "otherAlly" } }],
        who: [{ kind: "allAllies" }],
        does: ["Volley", "GiveShield"],
      },
      fusion: { first: "archer", second: "warden", name: "Archden", discoveredBy: null },
    });
    expect(ba.recipe).not.toEqual(fuseUnits(warden, archer, { name: "Archden", discoveredBy: null }, content).recipe);
  });

  it("copies the credit it is given: the namer (slice 10) applies the bot rule", () => {
    expect(fuseUnits(warden, archer, { name: "W", discoveredBy: maks }, content).fusion?.discoveredBy).toEqual(maks);
    expect(fuseUnits(warden, archer, { name: "W", discoveredBy: null }, content).fusion?.discoveredBy).toBeNull();
  });

  it("copies of either part merge into the fused unit, for stats only", () => {
    const fused = fuseUnits(warden, archer, { name: "Wardcher", discoveredBy: maks }, content);
    expect(mergeTarget([fused], "warden")).toBe(0);
    expect(mergeTarget([fused], "archer")).toBe(0);
    expect(mergeTarget([fused], "someone-else")).toBe(-1);
    expect(mergeTarget([lineUnitOf(Archer, "u9"), fused], "warden")).toBe(1);
    const grown = addCopy(addCopy(fused, content, MVP_RULES), content, MVP_RULES);
    expect(grown).toMatchObject({ kind: "fused", form: "awoken", copies: 8, stats: { pwr: 9, hp: 22 } });
    expect(grown.recipe).toEqual(fused.recipe);
    expect(grown.fusion).toEqual(fused.fusion);
  });

  it("mergeTarget finds a plain unit by id", () => {
    expect(mergeTarget([lineUnitOf(Warden, "u1"), lineUnitOf(Archer, "u2")], "archer")).toBe(1);
    expect(mergeTarget([], "archer")).toBe(-1);
  });
});

/** One event as the trace reads it: what happened, to whom, and which of the holder's Does did it. */
function show(e: BattleEvent): string | null {
  if (e.source === "kernel") return null;
  const by = `${e.source.unit}#${e.source.ability}`;
  switch (e.type) {
    case "Hurt": return `Hurt ${e.unit} -${e.amount} by ${by}`;
    case "StatusApplied": return `${e.status} on ${e.unit} by ${by}`;
    default: return null;
  }
}

function fight(a: LineUnit[], b: LineUnit[]): BattleEvent[] {
  return battle({ teamA: a.map(toBattleDef), teamB: b.map(toBattleDef), seed: 7, abilities, statuses: stressRegistry });
}

describe("golden battles with fused units", () => {
  const warden = withCopies(Warden, "u1", 3);
  const archer = withCopies(Archer, "u2", 3);
  const dummies = [lineUnitOf(Warden, "d1"), lineUnitOf(Warden, "d2")];
  const own = (log: BattleEvent[], at = "A1:") => log.filter((e) => e.source !== "kernel" && e.source.unit.startsWith(at) && !e.source.status);

  it("Warden + Archer: at battle start, shields then volleys every enemy, both Does in order", () => {
    const fused = fuseUnits(warden, archer, { name: "Wardcher", discoveredBy: maks }, content);
    const log = fight([fused], dummies);
    expect(log[0]).toMatchObject({ type: "BattleStart", teams: { A: [{ id: "A1:Wardcher", hp: 18, pwr: 7 }] } });
    expect(own(log).map((e) => `${show(e)} ← ${e.causedBy}`)).toEqual([
      "Shield on B1:Warden by A1:Wardcher#0 ← 0",
      "Shield on B2:Warden by A1:Wardcher#0 ← 0",
      "Hurt B1:Warden -1 by A1:Wardcher#1 ← 0",
      "Hurt B2:Warden -1 by A1:Wardcher#1 ← 0",
    ]);
    expect(log.at(-1)).toMatchObject({ type: "BattleEnd", winner: "A" });
  });

  it("Archer + Warden fights differently: it waits for a hurt ally, then volleys its own line", () => {
    // A plain Warden in front takes the hits; the fused unit stands behind it.
    const pair = (first: LineUnit, second: LineUnit) => [lineUnitOf(Warden, "u5"), fuseUnits(first, second, { name: "F", discoveredBy: null }, content)];
    const ab = fight(pair(archer, warden), dummies);
    const ba = fight(pair(warden, archer), dummies);
    expect(ab).not.toEqual(ba);
    expect(own(ba, "A2:").slice(0, 2).map(show)).toEqual(["Shield on B1:Warden by A2:F#0", "Shield on B2:Warden by A2:F#0"]);
    const first = own(ab, "A2:")[0]!;
    expect(own(ab, "A2:").slice(0, 4).map(show)).toEqual([
      "Hurt A1:Warden -2 by A2:F#0",
      "Hurt A2:F -3 by A2:F#0",
      "Shield on A1:Warden by A2:F#1",
      "Shield on A2:F by A2:F#1",
    ]);
    // It fired because an ally was hurt: one step up the causal log is a Hurt on side A.
    const cause = ab.find((e) => e.id === first.causedBy);
    expect(cause).toMatchObject({ type: "Hurt" });
    expect((cause as { unit: string }).unit).toMatch(/^A1:/);
  });
});

describe("unit content forms", () => {
  it("the golden units ship", () => {
    expect(contentFormProblems(content)).toEqual([]);
  });

  it("rejects an awoken form that changes the When", () => {
    const bad: UnitContent = { ...Warden, forms: { ...Warden.forms, awoken: { ...Warden.forms.awoken, when: Archer.forms.sleeping.when } } };
    expect(formProblems(bad, content)).toEqual(["warden: the awoken form must keep the sleeping form's When"]);
  });

  it("rejects an awoken form that upgrades nothing", () => {
    const bad: UnitContent = { ...Warden, forms: { sleeping: Warden.forms.sleeping, awoken: Warden.forms.sleeping } };
    expect(formProblems(bad, content)).toEqual(["warden: the awoken form must upgrade the Who and/or the Does"]);
  });

  it("rejects a sleeping form with two Does and unknown abilities", () => {
    const two: UnitContent = { ...Warden, forms: { ...Warden.forms, sleeping: { ...Warden.forms.sleeping, does: ["GiveShield", "Shoot"] } } };
    expect(formProblems(two, content)).toContain("warden: the sleeping form must Do exactly one thing, got 2");
    const unknown: UnitContent = { ...Archer, forms: { ...Archer.forms, awoken: { ...Archer.forms.awoken, does: ["Nope"] } } };
    expect(formProblems(unknown, content).some((p) => p.startsWith("archer.awoken"))).toBe(true);
  });

  it("flags duplicate ids", () => {
    expect(contentFormProblems({ ...content, units: [Warden, Warden] })).toEqual(["warden: duplicate unit id"]);
  });
});
