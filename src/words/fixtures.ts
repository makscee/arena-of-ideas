// The eight units of rules v2 §4 and the fusions of §5, written in the v2
// words as TEST FIXTURES ONLY: they are not in the pool and no shop offers
// them (unit batch 1, #1043 slice 3, designs and tunes the real ones). Each
// fixture battle has a golden in goldens/fixture-<name>.jsonl and a trace line
// it must show (fixtures.test.ts). Sleeping forms; the awoken clause is noted.

import type { AbilityRegistry, BattleInput, StatusRegistry, UnitDef } from "../types.js";
import { plain, rule } from "./examples.js";

export const FIXTURE_STATUSES: StatusRegistry = {
  Strength: { name: "Strength", statMods: { pwr: 1 }, abilities: [] },
  Shield: {
    name: "Shield",
    triggers: [{ kind: "interceptor", on: { on: "Hurt", unit: "holder" } }],
    selectors: [{ kind: "holder" }],
    abilities: [{ effects: [{ kind: "absorbHurt" }] }],
  },
  // Hourglass's counter: at 3 grains, spend them and the ally behind acts again
  // (awoken: alliesBehind). The rule lives on the counter so its condition
  // reads the grain just added at the same turn end.
  Grains: {
    name: "Grains",
    names: { en: "Grains", ru: { name: "Песчинки", acc: "Песчинки", count: ["Песчинка", "Песчинки", "Песчинок"] } },
    counter: true,
    triggers: [{ kind: "trigger", on: { on: "TurnEnd" } }],
    condition: { kind: "stacksAtLeast", status: "Grains", n: 3, of: "holder" },
    abilities: [
      // each status ability is its own reactor checking the condition, so the
      // grains are spent last
      { selectors: [{ kind: "allyBehind" }], effects: [{ kind: "actAgain" }] },
      { selectors: [{ kind: "holder" }], effects: [{ kind: "consumeStacks", stacks: { kind: "const", value: 3 } }] },
    ],
  },
  // Raven's second rule: at turn start, perform a remembered rule once (awoken: all).
  Memory: {
    name: "Memory",
    names: { en: "Memory", ru: { name: "Память", acc: "Память", count: ["Память", "Памяти", "Памяти"] } },
    counter: true,
    triggers: [{ kind: "trigger", on: { on: "TurnStart" } }],
    selectors: [{ kind: "holder" }],
    abilities: [{ effects: [{ kind: "performRemembered", mode: "once" }] }],
  },
};

const c = (value: number) => ({ kind: "const" as const, value });

export const FIXTURE_ABILITIES: AbilityRegistry = {
  Zap: { name: "Zap", family: "Strike", effects: [{ kind: "damage", amount: c(1) }] },
  Shoot: { name: "Shoot", family: "Strike", effects: [{ kind: "damage", amount: c(3) }] },
  Mend: { name: "Mend", family: "Heal", effects: [{ kind: "heal", amount: c(1) }] },
  Grain: { name: "Grain", family: "Arcane", effects: [{ kind: "applyStatus", status: "Grains", stacks: c(1) }] },
  Copy: { name: "Copy", family: "Arcane", effects: [{ kind: "copyRule" }] },
  Gutter: { name: "Gutter", family: "Shield", effects: [{ kind: "preventDeath", at: 1, cost: { stat: "pwr", amount: 1 } }] },
  AllIn: { name: "AllIn", family: "Strike", effects: [{ kind: "replaceAmount", amount: { kind: "twice", of: { kind: "stat", stat: "pwr", of: "holder" } } }] },
  Shell: { name: "Shell", family: "Shield", effects: [{ kind: "applyStatus", status: "Shield", stacks: { kind: "count", fact: "hitsTaken" } }] },
  Pull: { name: "Pull", family: "Control", effects: [{ kind: "redirect", to: { kind: "holder" } }] },
  Recall: { name: "Recall", family: "Arcane", effects: [{ kind: "remember" }] },
  Sprout: {
    name: "Sprout",
    family: "Summon",
    effects: [{ kind: "delay", turns: 2, effect: { kind: "transform", into: rule("Tree", 8, 0, { on: "TurnStart" }, { kind: "allyBehind" }, "Grow") } }],
  },
  Grow: { name: "Grow", family: "Control", effects: [{ kind: "applyStatus", status: "Strength", stacks: c(1) }] },
  // Déjà Vu: remember what the front enemy just did, do it one turn later.
  Echo: {
    name: "Echo",
    family: "Arcane",
    // The composed word "echo" (macros.ts): remember + delay 1 + performRemembered once.
    effects: [{ kind: "macro", name: "echo" }],
  },
  Sapling: {
    name: "Sapling",
    family: "Summon",
    effects: [{ kind: "summonWithRule", unit: plain("Sapling", 4, 0) }],
  },
};

/** The archer every fixture leans on: at turn start, 3 damage to the front enemy. */
const Archer = (hp = 4): UnitDef => rule("Archer", hp, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Shoot");

export const Hourglass: UnitDef = rule("Hourglass", 8, 1, { on: "TurnEnd" }, { kind: "holder" }, "Grain");
export const Mirror: UnitDef = rule("Mirror", 6, 1, { on: "BattleStart" }, { kind: "frontEnemy" }, "Copy");
export const Candle: UnitDef = { ...rule("Candle", 3, 3, { on: "Death", unit: "holder" }, { kind: "holder" }, "Gutter"), triggers: [{ kind: "interceptor", on: { on: "Death", unit: "holder" } }] };
// Deviation: tails strikes normally; "tails: none" needs an else word (next attempt).
export const Gambler: UnitDef = { ...rule("Gambler", 8, 2, { on: "Strike", striker: "holder" }, { kind: "holder" }, "AllIn", { kind: "flip", p: 0.5 }), triggers: [{ kind: "interceptor", on: { on: "Strike", striker: "holder" } }] };
export const Nautilus: UnitDef = rule("Nautilus", 10, 1, { on: "Hurt", unit: "holder" }, { kind: "holder" }, "Shell");
// Deviation: every hit on another ally, not only randomly aimed ones (no event says how it was aimed yet).
export const Magnet: UnitDef = { ...rule("Magnet", 12, 1, { on: "Hurt", unit: "otherAlly" }, { kind: "holder" }, "Pull"), triggers: [{ kind: "interceptor", on: { on: "Hurt", unit: "otherAlly" } }] };
export const Raven: UnitDef = { ...rule("Raven", 8, 1, { on: "Death", unit: "otherAlly" }, { kind: "eventUnit" }, "Recall"), statuses: [{ status: "Memory", stacks: 1 }] };
export const Seed: UnitDef = rule("Seed", 4, 0, { on: "BattleStart" }, { kind: "holder" }, "Sprout");
export const DejaVu: UnitDef = { ...rule("Déjà Vu", 10, 2, { on: "TurnStart" }, { kind: "frontEnemy" }, "Echo"), triggers: [{ kind: "trigger", on: { on: "Acted", unit: "enemy", front: true } }] };
export const GraveTree: UnitDef = rule("Grave Tree", 8, 1, { on: "Death", unit: "otherAlly" }, { kind: "eventUnit" }, "Sapling");

const Hedgehog: UnitDef = rule("Hedgehog", 8, 1, { on: "Hurt", unit: "holder" }, { kind: "frontEnemy" }, "Zap");

const fight = (teamA: UnitDef[], teamB: UnitDef[], seed = 1): BattleInput => ({ teamA, teamB, seed, statuses: FIXTURE_STATUSES, abilities: FIXTURE_ABILITIES });

/** name → its battle and the trace line the battle shows. */
export const FIXTURES: Record<string, { input: BattleInput; trace: string }> = {
  hourglass: { input: fight([Hourglass, Archer(9)], [plain("Brute", 30, 1)]), trace: "Hourglass: Archer acts again" },
  mirror: { input: fight([Mirror], [Hedgehog]), trace: "Mirror copies the rule of Hedgehog" },
  candle: { input: fight([Candle], [plain("Brute", 30, 2)]), trace: "Candle intercepts: Candle survives at 1 HP, loses 1 PWR" },
  gambler: { input: fight([Gambler], [plain("Brute", 30, 1)]), trace: "Gambler flips: heads" },
  nautilus: { input: fight([Nautilus], [plain("Brute", 30, 3)]), trace: "Nautilus → self (Nautilus) → Shield 3" },
  magnet: { input: fight([Archer(), Magnet], [plain("Brute", 30, 2)]), trace: "Magnet intercepts: the hit on Archer lands on Magnet" },
  raven: { input: fight([Archer(1), Raven], [plain("Brute", 30, 2)]), trace: "Raven performs the rule of Archer" },
  seed: { input: fight([Seed, Archer()], [plain("Brute", 30, 1)]), trace: "Seed becomes Tree (0/4)" },
  dejaVu: { input: fight([DejaVu], [Archer(12)]), trace: "Déjà Vu performs the rule of Archer" },
  graveTree: { input: fight([Archer(1), GraveTree], [plain("Brute", 30, 2)]), trace: "Grave Tree grows Sapling with the rule of Archer" },
};
