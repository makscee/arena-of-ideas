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
import { statusActionsOf, unitActionsOf } from "./types.js";
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
    expect(describeStatus(stressRegistry.Strength!)).toMatchInlineSnapshot(`"+1 PWR per stack."`);
    expect(describeStatus(stressRegistry.Vitality!)).toMatchInlineSnapshot(`"+1 HP per stack."`);
    expect(describeStatus(stressRegistry.Curse!)).toMatchInlineSnapshot(`"-1 PWR per stack."`);
    expect(describeStatus(stressRegistry.Poison!)).toMatchInlineSnapshot(
      `"Turn end: damage equal to stacks to holder, then spend 1 stack."`,
    );
    expect(describeStatus(stressRegistry.Shield!)).toMatchInlineSnapshot(
      `"Would be hit: block damage up to stacks, spending them."`,
    );
    expect(describeStatus(stressRegistry.Freeze!)).toMatchInlineSnapshot(
      `"Would strike: cancel it, spend 1 stack."`,
    );
    expect(describeStatus(stressRegistry.Blessing!)).toMatchInlineSnapshot(
      `"Would die: cancel death, set holder to HP equal to stacks, spend this status."`,
    );
  });
});

describe("describeAbility", () => {
  test("every shipped unit ability yields a non-empty description", () => {
    for (const unit of [Venomancer, Summoner, Silencer, Necromancer]) {
      for (const ab of unitActionsOf(unit, stressAbilities)) {
        expect(describeAbility(ab).length, `${unit.name} should describe its ability`).toBeGreaterThan(0);
      }
    }
  });

  test("a summon for every ally reads as one per ally, not 'every ally's side' (#587)", () => {
    const ab = { ...unitActionsOf(Summoner, stressAbilities)[0]!, selectors: [{ kind: "allAllies" as const }] };
    expect(describeAbility(ab)).toBe(
      "Dies: summon Imp (1/2) for each ally, if there's room.",
    );
    // Fired at battle start, the holder is still in the line and gets one too.
    const atStart = { ...ab, whens: [{ kind: "trigger" as const, on: { on: "BattleStart" as const } }] };
    expect(describeAbility(atStart)).toBe(
      "Battle start: summon Imp (1/2) for each ally, self included, if there's room.",
    );
  });

  test("known wordings (the shipped stress units)", () => {
    expect(describeAbility(unitActionsOf(Venomancer, stressAbilities)[0]!)).toMatchInlineSnapshot(
      `"Strikes: 2 Poison to front enemy."`,
    );
    expect(describeAbility(unitActionsOf(Summoner, stressAbilities)[0]!)).toMatchInlineSnapshot(
      `"Dies: summon Imp (1/2)."`,
    );
    expect(describeAbility(unitActionsOf(Silencer, stressAbilities)[0]!)).toMatchInlineSnapshot(
      `"Battle start: silence front enemy."`,
    );
    expect(describeAbility(unitActionsOf(Necromancer, stressAbilities)[0]!)).toMatchInlineSnapshot(
      `"Ally dies: revive fallen ally at 1 HP."`,
    );
  });

  test("a derived resurrect hp reads 'at hp equal to …', never a trailing ' hp'", () => {
    const text = describeAbility({
      whens: [{ kind: "trigger", on: { on: "Death", unit: "ally" } }],
      selectors: [{ kind: "lastDeadAlly" }],
      effects: [{ kind: "resurrect", hp: { kind: "level", of: "holder" } }],
    });
    expect(text).toMatchInlineSnapshot(
      `"Ally dies: revive fallen ally at HP equal to level."`,
    );
  });

  test("a status-held ability speaks of the holder", () => {
    const text = describeAbility(statusActionsOf(stressRegistry.Poison!)[0]!, { holder: "holder" });
    expect(text).toContain("to holder");
    expect(text).not.toContain("self");
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
      `"Hit, or turn end, at 5 HP or less: heal all allies and random enemy for PWR."`,
    );
  });
});

describe("abilityChips — the card's terse 3-chip line (#082)", () => {
  test("the shipped stress units read as short trigger/target/action + glyph", () => {
    // Venom: ⚔ Strikes ▸ Front enemy ▸ ☣ Poison 2 (the mockup's canonical row;
    // the action glyph ☣ is the family glyph the card derives, not in the chips).
    expect(abilityChips(unitActionsOf(Venomancer, stressAbilities)[0]!)).toEqual({
      trigger: "Strikes",
      triggerGlyph: "⚔",
      target: "Front enemy",
      action: "Poison 2",
    });
    expect(abilityChips(unitActionsOf(Summoner, stressAbilities)[0]!)).toEqual({
      trigger: "Dies",
      triggerGlyph: "☠",
      target: "Self",
      action: "Summon Imp",
    });
    expect(abilityChips(unitActionsOf(Silencer, stressAbilities)[0]!)).toEqual({
      trigger: "Battle start",
      triggerGlyph: "⚑",
      target: "Front enemy",
      action: "Silence",
    });
    expect(abilityChips(unitActionsOf(Necromancer, stressAbilities)[0]!)).toEqual({
      trigger: "Dies",
      triggerGlyph: "☠",
      target: "Fallen ally",
      action: "Revive",
    });
  });

  test("each chip stays short — no prose: trigger ≤ 3 words, no 'the'/'apply'", () => {
    for (const unit of [Venomancer, Summoner, Silencer, Necromancer]) {
      const c = abilityChips(unitActionsOf(unit, stressAbilities)[0]!);
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
      for (const ab of unitActionsOf(unit, stressAbilities)) {
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
      ...shippedUnits.flatMap((u) => unitActionsOf(u, stressAbilities)),
      ...Object.values(stressRegistry).flatMap(statusActionsOf),
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
    const segs = describeAbilitySegments(unitActionsOf(Venomancer, stressAbilities)[0]!);
    expect(segs.filter((s) => s.statusRef !== undefined)).toEqual([{ text: "Poison", statusRef: "Poison", term: "status:Poison" }]);
    expect(abilityStatusRefs(unitActionsOf(Venomancer, stressAbilities)[0]!)).toEqual(["Poison"]);
  });

  test("consumeStacks of the owning status ('this status') is not a ref", () => {
    const refs = statusActionsOf(stressRegistry.Poison!).flatMap((ab) => abilityStatusRefs(ab));
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
      `"Ally gets Poison: heal self for 2."`,
    );
    const refs = describeAbilitySegments(onAllyPoisoned).filter((s) => s.statusRef !== undefined);
    expect(refs).toEqual([{ text: "Poison", statusRef: "Poison", term: "status:Poison", clause: "when" }]);
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
    expect(describeWhen(w)).toBe("would get a status");
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
      `"Strikes: spend 2 Shield, then 3 damage to front enemy."`,
    );
    const segs = describeAbilitySegments(shieldBreaker);
    expect(segs.filter((s) => s.statusRef !== undefined)).toEqual([{ text: "Shield", statusRef: "Shield", term: "status:Shield" }]);
    expect(segs.map((s) => s.text).join("")).toBe(describeAbility(shieldBreaker));
    expect(abilityStatusRefs(shieldBreaker)).toEqual(["Shield"]);
  });
});
