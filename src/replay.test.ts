// Replay renderer tests — the renderer is a pure function over the event log:
// full battles render, output is deterministic, and the WHY is in the text
// (cause chains on deaths, explanations on ChainBlocked, intercepts named).

import { describe, expect, test } from "vitest";
import { battle } from "./battle.js";
import { renderReplay } from "./replay.js";
import type { AbilityRegistry, BattleInput, UnitDef } from "./types.js";
import { Necromancer, Silencer, stressRegistry, Summoner, Venomancer } from "./content/stress.js";
import { stressAbilities } from "./content/stress.js";

// A battle exercising the whole stress set: statuses on both sides, a
// summoner, a necromancer, a silencer, shields, freezes, blessings.
const stressBattle: BattleInput = {
  teamA: [
    Venomancer,
    Summoner,
    Necromancer,
    {
      name: "Champion",
      base: { hp: 12, pwr: 2 },
      ability: "Strike",
      statuses: [
        { status: "Strength", stacks: 2 },
        { status: "Shield", stacks: 3 },
      ],
    },
  ],
  teamB: [
    Silencer,
    {
      name: "FrozenBrute",
      base: { hp: 14, pwr: 4 },
      ability: "Strike",
      statuses: [
        { status: "Freeze", stacks: 2 },
        { status: "Curse", stacks: 1 },
      ],
    },
    {
      name: "Martyr",
      base: { hp: 6, pwr: 2 },
      ability: "Strike",
      statuses: [
        { status: "Blessing", stacks: 3 },
        { status: "Vitality", stacks: 2 },
      ],
    },
  ],
  seed: 42,
  statuses: stressRegistry,
  abilities: stressAbilities,
};

describe("replay renderer", () => {
  test("renders a full stress-set battle without throwing, with visible turn structure", () => {
    const text = renderReplay(battle(stressBattle));
    expect(text.length).toBeGreaterThan(0);
    expect(text).toContain("=== BATTLE ===");
    expect(text).toContain("Side A:");
    expect(text).toContain("Side B:");
    expect(text).toContain("--- Turn 1 ---");
    expect(text).toMatch(/=== (Side [AB] wins|Draw) after \d+ turns? ===/);
  });

  test("byte-identical output across two runs with the same seed", () => {
    const text1 = renderReplay(battle(stressBattle));
    const text2 = renderReplay(battle(stressBattle));
    expect(text1).toBe(text2);
  });

  test("a poison death renders its cause chain back to who applied the poison", () => {
    // Venomancer (1 pwr, applies Poison 2 per strike) vs a 20 hp punching bag:
    // strikes alone need 20 turns, but poison stacks net +1 per turn while its
    // tick grows — the victim dies to a poison tick, and the chain must say so.
    const victim: UnitDef = { name: "Victim", base: { hp: 20, pwr: 0 }, ability: "Strike" };
    const log = battle({ teamA: [Venomancer], teamB: [victim], seed: 7, statuses: stressRegistry, abilities: stressAbilities });
    const text = renderReplay(log);

    const poisonDeath = log.find(
      (e) =>
        e.type === "Death" &&
        e.causedBy !== null &&
        (() => {
          const c = log[e.causedBy!];
          return c?.type === "Hurt" && c.source !== "kernel" && c.source.status === "Poison";
        })(),
    );
    expect(poisonDeath).toBeDefined(); // the scenario really is a poison death

    expect(text).toMatch(/Victim dies ← Poison tick \(\d+ dmg\) ← Poison applied turn \d+ by Venomancer/);
  });

  test("a ChainBlocked renders an explanation of the no-self-retrigger law", () => {
    // A unit that damages itself whenever it is hurt: the second firing sits
    // in its own causal chain and must be suppressed — and explained.
    const selfHurterAbilities: AbilityRegistry = {
      ...stressAbilities,
      SelfHurt: {
        name: "SelfHurt",
        family: "Strike",
        whens: [{ kind: "trigger", on: { on: "Hurt", unit: "holder" } }],
        selectors: [{ kind: "holder" }],
        effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }],
      },
    };
    const selfHurter: UnitDef = {
      name: "SelfHurter",
      base: { hp: 100, pwr: 0 },
      ability: "SelfHurt",
    };
    const log = battle({
      teamA: [selfHurter],
      teamB: [{ name: "Attacker", base: { hp: 100, pwr: 2 }, ability: "Strike" }],
      seed: 1,
      abilities: selfHurterAbilities,
    });
    expect(log.some((e) => e.type === "ChainBlocked")).toBe(true);

    const text = renderReplay(log);
    expect(text).toContain("chain stopped: SelfHurter's ability stayed quiet after SelfHurter was hurt");
    expect(text).toContain("an ability never triggers itself");
  });

  test("an Intercepted strike shows what cancelled what (Freeze)", () => {
    const log = battle({
      teamA: [{ name: "Attacker", base: { hp: 20, pwr: 1 }, ability: "Strike" }],
      teamB: [{ name: "FrozenOne", base: { hp: 20, pwr: 2 }, ability: "Strike", statuses: [{ status: "Freeze", stacks: 1 }] }],
      seed: 1,
      statuses: stressRegistry,
  abilities: stressAbilities,
    });
    const text = renderReplay(log);
    expect(text).toContain("FrozenOne tries to strike, but Freeze on FrozenOne cancels it");
  });

  test("an intercepted death shows the Blessing refusing it, and Shield absorption is visible", () => {
    const log = battle({
      teamA: [{ name: "Slayer", base: { hp: 100, pwr: 99 }, ability: "Strike" }],
      teamB: [
        {
          name: "Blessed",
          base: { hp: 5, pwr: 0 },
          ability: "Strike",
          statuses: [
            { status: "Blessing", stacks: 2 },
            { status: "Shield", stacks: 3 },
          ],
        },
      ],
      seed: 1,
      statuses: stressRegistry,
  abilities: stressAbilities,
    });
    const text = renderReplay(log);
    expect(text).toContain("Blessed should die, but Blessing on Blessed refuses the death.");
    expect(text).toContain("absorbed by Shield");
  });

  test("a resurrection reads as a return from the grave at the right hp", () => {
    const fodder: UnitDef = { name: "Fodder", base: { hp: 3, pwr: 1 }, ability: "Strike", statuses: [{ status: "Poison", stacks: 3 }] };
    const log = battle({
      teamA: [fodder, Necromancer],
      teamB: [{ name: "Ogre", base: { hp: 3, pwr: 1 }, ability: "Strike" }],
      seed: 1,
      statuses: stressRegistry,
  abilities: stressAbilities,
    });
    const text = renderReplay(log);
    expect(text).toContain("Fodder rises from the grave at 1 hp");
    expect(text).toContain("Necromancer's doing");
  });
});
