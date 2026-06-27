// Derived-description tests — the contract: every shipped registry entry and
// every shipped unit ability yields a non-empty sentence, and the known
// wordings are snapshotted so a regression in phrasing is visible in review.

import { describe, expect, test } from "vitest";
import {
  abilityChips,
  abilityStatusRefs,
  describeAbility,
  describeAbilitySegments,
  describeStatus,
  describeStatusSegments,
  describeWhen,
  describeWhenSegments,
} from "./describe.js";
import type { Ability, When } from "./types.js";
import { Necromancer, Silencer, Summoner, Venomancer, stressAbilities, stressRegistry } from "./content/stress.js";
import { BOSS_TEAMS, DEFAULT_RUN_POOL } from "./tunables.js";

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
      for (const ab of (unit.ability !== undefined ? [stressAbilities[unit.ability]!] : [])) {
        expect(describeAbility(ab).length, `${unit.name} should describe its ability`).toBeGreaterThan(0);
      }
    }
  });

  test("known wordings (the shipped stress units)", () => {
    expect(describeAbility(stressAbilities[Venomancer.ability!]!)).toMatchInlineSnapshot(
      `"After this unit strikes: apply 2 Poison to the front enemy."`,
    );
    expect(describeAbility(stressAbilities[Summoner.ability!]!)).toMatchInlineSnapshot(
      `"After this unit dies: summon Imp (2 hp, 1 pwr) at the back of this unit's side."`,
    );
    expect(describeAbility(stressAbilities[Silencer.ability!]!)).toMatchInlineSnapshot(
      `"When the battle begins: silence the front enemy — strip its statuses and disable its abilities for the battle."`,
    );
    expect(describeAbility(stressAbilities[Necromancer.ability!]!)).toMatchInlineSnapshot(
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

describe("abilityChips — the card's terse 3-chip line (#082)", () => {
  test("the shipped stress units read as short trigger/target/action + glyph", () => {
    // Venom: ⚔ On strike ▸ Front enemy ▸ ☣ Poison 2 (the mockup's canonical row;
    // the action glyph ☣ is the family glyph the card derives, not in the chips).
    expect(abilityChips(stressAbilities[Venomancer.ability!]!)).toEqual({
      trigger: "On strike",
      triggerGlyph: "⚔",
      target: "Front enemy",
      action: "Poison 2",
    });
    expect(abilityChips(stressAbilities[Summoner.ability!]!)).toEqual({
      trigger: "On death",
      triggerGlyph: "☠",
      target: "Self",
      action: "Summon Imp",
    });
    expect(abilityChips(stressAbilities[Silencer.ability!]!)).toEqual({
      trigger: "Battle start",
      triggerGlyph: "⚑",
      target: "Front enemy",
      action: "Silence",
    });
    expect(abilityChips(stressAbilities[Necromancer.ability!]!)).toEqual({
      trigger: "On death",
      triggerGlyph: "☠",
      target: "Last dead ally",
      action: "Revive",
    });
  });

  test("each chip stays short — no prose: trigger ≤ 3 words, no 'the'/'apply'", () => {
    for (const unit of [Venomancer, Summoner, Silencer, Necromancer]) {
      const c = abilityChips(stressAbilities[unit.ability!]!);
      expect(c.trigger!.split(" ").length, `${unit.name} trigger terse`).toBeLessThanOrEqual(3);
      expect(c.target!.split(" ").length, `${unit.name} target terse`).toBeLessThanOrEqual(3);
      expect(`${c.trigger} ${c.target} ${c.action}`).not.toMatch(/\bthe\b|\bapply\b|after /i);
    }
  });

  test("a const applyStatus reads 'Status N'; a derived magnitude drops the number", () => {
    const ab: Ability = {
      whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
      selectors: [{ kind: "frontEnemy" }],
      effects: [{ kind: "applyStatus", status: "Poison", stacks: { kind: "stacks" } }],
    };
    expect(abilityChips(ab).action).toBe("Poison");
  });

  test("triggers carry their event glyph; absent when/selector/effect drop the chip", () => {
    expect(abilityChips({ whens: [{ kind: "trigger", on: { on: "TurnEnd" } }], selectors: [], effects: [] })).toEqual({
      trigger: "Turn end",
      triggerGlyph: "⟲",
      target: undefined,
      action: undefined,
    });
    expect(
      abilityChips({ whens: [], selectors: [{ kind: "allEnemies" }], effects: [{ kind: "damage", amount: { kind: "const", value: 3 } }] }),
    ).toEqual({ trigger: undefined, triggerGlyph: undefined, target: "All enemies", action: "Deal 3" });
  });
});

describe("describe segments / status refs", () => {
  const shippedUnits = [...new Set([...DEFAULT_RUN_POOL, ...BOSS_TEAMS.flat()])];

  test("joined ability segments reproduce describeAbility exactly (all shipped content)", () => {
    for (const unit of shippedUnits) {
      for (const ab of (unit.ability !== undefined ? [stressAbilities[unit.ability]!] : [])) {
        const joined = describeAbilitySegments(ab)
          .map((s) => s.text)
          .join("");
        expect(joined, `${unit.name}'s segments should join to its sentence`).toBe(describeAbility(ab));
      }
    }
  });

  test("joined status segments reproduce describeStatus exactly (whole registry)", () => {
    for (const [name, def] of Object.entries(stressRegistry)) {
      const joined = describeStatusSegments(def)
        .map((s) => s.text)
        .join("");
      expect(joined, `${name}'s segments should join to its description`).toBe(describeStatus(def));
    }
  });

  test("every applyStatus/consumeStacks ability yields refs that resolve in the registry", () => {
    const abilities = [
      ...shippedUnits.flatMap((u) => (u.ability !== undefined ? [stressAbilities[u.ability]!] : [])),
      ...Object.values(stressRegistry).flatMap((d) => d.abilities),
    ];
    let applying = 0;
    for (const ab of abilities) {
      const refs = abilityStatusRefs(ab);
      for (const e of ab.effects) {
        if (e.kind === "applyStatus" || (e.kind === "consumeStacks" && e.status !== undefined)) {
          applying++;
          const status = e.kind === "applyStatus" ? e.status : e.status!;
          expect(refs, `the ${e.kind} effect's status should be a ref`).toContain(status);
          expect(stressRegistry[status], `ref ${status} should resolve in the registry`).toBeDefined();
        }
      }
    }
    expect(applying, "the shipped content should exercise status refs at all").toBeGreaterThan(0);
  });

  test("Venomancer's ability marks Poison as a ref, the rest as plain text", () => {
    const segs = describeAbilitySegments(stressAbilities[Venomancer.ability!]!);
    expect(segs.filter((s) => s.statusRef !== undefined)).toEqual([{ text: "Poison", statusRef: "Poison" }]);
    expect(abilityStatusRefs(stressAbilities[Venomancer.ability!]!)).toEqual(["Poison"]);
  });

  test("consumeStacks of the owning status ('this status') is not a ref", () => {
    const refs = stressRegistry.Poison!.abilities.flatMap((ab) => abilityStatusRefs(ab));
    expect(refs).toEqual([]);
  });
});

describe("when-clause status refs (constructed content — shipped whens carry no status)", () => {
  // Editor-made content can put a status name in the when itself ("after
  // Poison lands on an ally"); those names must be tappable refs exactly like
  // effect clauses. No shipped when names a status, so the cases are built.
  const onAllyPoisoned: Ability = {
    whens: [{ kind: "trigger", on: { on: "StatusApplied", unit: "ally", status: "Poison" } }],
    selectors: [{ kind: "holder" }],
    effects: [{ kind: "heal", amount: { kind: "const", value: 2 } }],
  };

  test("a status-pattern when reads the same and marks the status as a ref", () => {
    expect(describeAbility(onAllyPoisoned)).toMatchInlineSnapshot(
      `"After Poison lands on an ally: heal this unit for 2."`,
    );
    const refs = describeAbilitySegments(onAllyPoisoned).filter((s) => s.statusRef !== undefined);
    expect(refs).toEqual([{ text: "Poison", statusRef: "Poison" }]);
    expect(abilityStatusRefs(onAllyPoisoned)).toEqual(["Poison"]);
  });

  test("joined when segments reproduce describeWhen exactly, for every pattern shape", () => {
    const patterns: When[] = [];
    for (const kind of ["trigger", "interceptor"] as const) {
      patterns.push(
        { kind, on: { on: "BattleStart" } },
        { kind, on: { on: "Strike", striker: "enemy" } },
        { kind, on: { on: "Hurt", unit: "holder" } },
        { kind, on: { on: "StatusApplied", unit: "ally", status: "Poison" } },
        { kind, on: { on: "StatusApplied", unit: "any" } },
        { kind, on: { on: "StatusRemoved", unit: "enemy", status: "Shield" } },
        { kind, on: { on: "StatusRemoved" } },
      );
    }
    for (const w of patterns) {
      const joined = describeWhenSegments(w)
        .map((s) => s.text)
        .join("");
      expect(joined, `${w.kind} on ${w.on.on} should join to its clause`).toBe(describeWhen(w));
    }
  });

  test("a statusless when pattern yields no ref", () => {
    const w: When = { kind: "interceptor", on: { on: "StatusApplied", unit: "holder" } };
    expect(describeWhenSegments(w).every((s) => s.statusRef === undefined)).toBe(true);
    expect(describeWhen(w)).toBe("when a status would land on this unit");
  });
});

describe("explicit-status consumeStacks refs (constructed — shipped content has none)", () => {
  // consumeStacks naming a status (not the owning "this status") must surface
  // that status as a ref; no shipped effect uses the explicit form, so the
  // extraction is pinned with a built ability — the in-browser probe, kept.
  const shieldBreaker: Ability = {
    whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
    selectors: [{ kind: "frontEnemy" }],
    effects: [
      { kind: "consumeStacks", status: "Shield", stacks: { kind: "const", value: 2 } },
      { kind: "damage", amount: { kind: "const", value: 3 } },
    ],
  };

  test("the named status is a ref and the sentence still joins exactly", () => {
    expect(describeAbility(shieldBreaker)).toMatchInlineSnapshot(
      `"After this unit strikes: consume 2 stacks of Shield, then deal 3 damage to the front enemy."`,
    );
    const segs = describeAbilitySegments(shieldBreaker);
    expect(segs.filter((s) => s.statusRef !== undefined)).toEqual([{ text: "Shield", statusRef: "Shield" }]);
    expect(segs.map((s) => s.text).join("")).toBe(describeAbility(shieldBreaker));
    expect(abilityStatusRefs(shieldBreaker)).toEqual(["Shield"]);
  });
});
