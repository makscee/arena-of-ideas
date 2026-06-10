// Derived-description tests — the contract: every shipped registry entry and
// every shipped unit ability yields a non-empty sentence, and the known
// wordings are snapshotted so a regression in phrasing is visible in review.

import { describe, expect, test } from "vitest";
import { describeAbility, describeStatus } from "./describe.js";
import { Necromancer, Silencer, Summoner, Venomancer, stressRegistry } from "./content/stress.js";

describe("describeStatus", () => {
  test("every entry in the shipped registry yields a non-empty description", () => {
    for (const [name, def] of Object.entries(stressRegistry)) {
      const text = describeStatus(def);
      expect(text.length, `${name} should describe itself`).toBeGreaterThan(0);
    }
  });

  test("known wordings (the shipped statuses)", () => {
    expect(describeStatus(stressRegistry.Strength!)).toMatchInlineSnapshot(`"+1 pwr per stack."`);
    expect(describeStatus(stressRegistry.Vitality!)).toMatchInlineSnapshot(`"+1 hp per stack."`);
    expect(describeStatus(stressRegistry.Curse!)).toMatchInlineSnapshot(`"-1 pwr per stack."`);
    expect(describeStatus(stressRegistry.Poison!)).toMatchInlineSnapshot(
      `"At the end of each turn: deal damage equal to its stacks to the holder, then consume 1 stack of this status."`,
    );
    expect(describeStatus(stressRegistry.Shield!)).toMatchInlineSnapshot(
      `"When the holder would be hurt: absorb the damage up to its stacks, consuming what it absorbs."`,
    );
    expect(describeStatus(stressRegistry.Freeze!)).toMatchInlineSnapshot(
      `"When the holder would strike: cancel it, consuming 1 stack."`,
    );
    expect(describeStatus(stressRegistry.Blessing!)).toMatchInlineSnapshot(
      `"When the holder would die: cancel the death and heal the holder to hp equal to its stacks, spending this status."`,
    );
  });
});

describe("describeAbility", () => {
  test("every shipped unit ability yields a non-empty description", () => {
    for (const unit of [Venomancer, Summoner, Silencer, Necromancer]) {
      for (const ab of unit.abilities ?? []) {
        expect(describeAbility(ab).length, `${unit.name} should describe its ability`).toBeGreaterThan(0);
      }
    }
  });

  test("known wordings (the shipped stress units)", () => {
    expect(describeAbility(Venomancer.abilities![0]!)).toMatchInlineSnapshot(
      `"After this unit strikes: apply 2 Poison to the front enemy."`,
    );
    expect(describeAbility(Summoner.abilities![0]!)).toMatchInlineSnapshot(
      `"After this unit dies: summon Imp (2 hp, 1 pwr) at the back of this unit's side."`,
    );
    expect(describeAbility(Silencer.abilities![0]!)).toMatchInlineSnapshot(
      `"When the battle begins: silence the front enemy — strip its statuses and disable its abilities for the battle."`,
    );
    expect(describeAbility(Necromancer.abilities![0]!)).toMatchInlineSnapshot(
      `"After an ally dies: return the most recently dead ally to the back of the line at 1 hp."`,
    );
  });

  test("a derived resurrect hp reads 'at hp equal to …', never a trailing ' hp'", () => {
    const text = describeAbility({
      whens: [{ kind: "trigger", on: { on: "Death", unit: "ally" } }],
      selectors: [{ kind: "lastDeadAlly" }],
      effects: [{ kind: "resurrect", hp: { kind: "level", of: "holder" } }],
    });
    expect(text).toMatchInlineSnapshot(
      `"After an ally dies: return the most recently dead ally to the back of the line at hp equal to this unit's level."`,
    );
  });

  test("a status-held ability speaks of the holder", () => {
    const text = describeAbility(stressRegistry.Poison!.abilities[0]!, { holder: "the holder" });
    expect(text).toContain("the holder");
    expect(text).not.toContain("this unit");
  });

  test("multiple whens, a condition, and multiple selectors all surface", () => {
    const text = describeAbility({
      whens: [
        { kind: "trigger", on: { on: "Hurt", unit: "holder" } },
        { kind: "trigger", on: { on: "TurnEnd" } },
      ],
      condition: { kind: "holderHpAtMost", value: 5 },
      selectors: [{ kind: "allAllies" }, { kind: "randomEnemy" }],
      effects: [{ kind: "heal", amount: { kind: "stat", stat: "pwr", of: "holder" } }],
    });
    expect(text).toMatchInlineSnapshot(
      `"After this unit is hurt, or at the end of each turn, while this unit is at 5 hp or less: heal every ally and a random enemy for an amount equal to this unit's pwr."`,
    );
  });
});
