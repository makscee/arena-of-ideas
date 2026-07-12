import { describe, expect, it } from "vitest";
import { battle, toJSONL } from "./battle.js";
import { migrateContentEnvelope } from "./content-grammar.js";
import { stressRegistry } from "./content/stress.js";
import { validateAbilityRegistry, validateTeam } from "./validate.js";

const legacy = {
  grammarVersion: 1,
  units: [{ name: "Venomancer", base: { hp: 6, pwr: 1 }, ability: "Venom" }],
  abilities: {
    Venom: {
      name: "Venom", family: "Poison",
      whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
      selectors: [{ kind: "frontEnemy" }],
      effects: [{ kind: "applyStatus", status: "Poison", stacks: { kind: "const", value: 2 } }],
    },
  },
} as const;

const victim = {
  name: "Victim", base: { hp: 12, pwr: 1 },
  triggers: [{ kind: "trigger", on: { on: "BattleStart" } }],
  selectors: [{ kind: "holder" }], abilities: ["Strike"],
} as const;
const strike = { name: "Strike", family: "Strike", effects: [{ kind: "heal", amount: { kind: "const", value: 0 } }] } as const;

describe("content grammar v1 → v2 migration", () => {
  it("moves when/who context to the Unit and stamps deterministic provenance", () => {
    const a = migrateContentEnvelope(legacy, "legacy");
    const b = migrateContentEnvelope(structuredClone(legacy), "legacy");
    expect(a).toEqual(b);
    expect(a.provenance).toEqual({ grammarVersion: 2, migratedFrom: 1 });
    expect(a.units[0]).toEqual({
      name: "Venomancer", base: { hp: 6, pwr: 1 },
      triggers: legacy.abilities.Venom.whens,
      selectors: legacy.abilities.Venom.selectors,
      abilities: ["Venom"],
    });
    expect(a.abilities.Venom).toEqual({ name: "Venom", family: "Poison", effects: legacy.abilities.Venom.effects });
  });

  it("accepts canonical actions and preserves replay behavior", () => {
    const migrated = migrateContentEnvelope(legacy, "legacy");
    const abilities = { ...migrated.abilities, Strike: strike } as never;
    expect(validateAbilityRegistry(abilities, stressRegistry)).toEqual([]);
    expect(validateTeam(migrated.units, stressRegistry, abilities)).toEqual([]);
    const canonical = migrateContentEnvelope({ grammarVersion: 2, units: migrated.units, abilities: migrated.abilities }, "canonical");
    const before = toJSONL(battle({ teamA: legacy.units as never, teamB: [victim] as never, seed: 7, statuses: stressRegistry, abilities: { Venom: legacy.abilities.Venom, Strike: strike } as never }));
    const after = toJSONL(battle({ teamA: canonical.units, teamB: [victim] as never, seed: 7, statuses: stressRegistry, abilities }));
    expect(after).toBe(before);
    expect(after).toContain('"status":"Poison"');
  });

  it("rejects mixed canonical recipe fields before they can create replay ambiguity", () => {
    expect(() => migrateContentEnvelope({
      grammarVersion: 2,
      units: [{ ...legacy.units[0], triggers: legacy.abilities.Venom.whens, selectors: legacy.abilities.Venom.selectors, abilities: ["Venom"] }],
      abilities: { Venom: { name: "Venom", family: "Poison", effects: legacy.abilities.Venom.effects } },
    }, "mixed")).toThrow(/mixed\.units\[0\].*mixed v1\/v2 unit payload/);
    expect(() => migrateContentEnvelope({
      ...legacy,
      units: [{ ...legacy.units[0], condition: { kind: "holderHpAtMost", value: 0 } }],
    }, "mixedCondition")).toThrow(/mixedCondition\.units\[0\].*condition/);
  });

  it("preserves valid migration provenance on canonical reread", () => {
    const first = migrateContentEnvelope(legacy, "legacy");
    const second = migrateContentEnvelope({ ...first.provenance, units: first.units, abilities: first.abilities }, "reread");
    expect(second).toEqual(first);
  });

  it("rejects context smuggled into a canonical Ability", () => {
    expect(() => migrateContentEnvelope({ grammarVersion: 2, units: [], abilities: legacy.abilities }, "mixedAbility"))
      .toThrow(/Ability is only what happens in v2/);
  });
});
