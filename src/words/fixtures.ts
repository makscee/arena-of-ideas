// The eight units of rules v2 §4 and the fusions of §5, written in the v2
// words as TEST FIXTURES ONLY: they are not in the pool and no shop offers
// them (unit batch 1, #1043 slice 3, designs and tunes the real ones). Each
// fixture battle has a golden in goldens/fixture-<name>.jsonl and a trace line
// it must show (fixtures.test.ts). Sleeping forms first, then each awoken form
// (§4's "awoken" lines), unit batch 1 (#1046).

import type { AbilityRegistry, BattleInput, StatusRegistry, UnitDef } from "../types.js";
import { ALL_OR_NOTHING } from "./all-or-nothing.js";
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
  // Awoken Hourglass's counter: the same, but every ally behind acts again.
  Sands: {
    name: "Sands",
    names: { en: "Sands", ru: { name: "Пески", acc: "Пески", count: ["Песок", "Песка", "Песков"] } },
    counter: true,
    triggers: [{ kind: "trigger", on: { on: "TurnEnd" } }],
    condition: { kind: "stacksAtLeast", status: "Sands", n: 3, of: "holder" },
    abilities: [
      { selectors: [{ kind: "alliesBehind" }], effects: [{ kind: "actAgain" }] },
      { selectors: [{ kind: "holder" }], effects: [{ kind: "consumeStacks", stacks: { kind: "const", value: 3 } }] },
    ],
  },
  // An enemy's random poison: 1 damage at each turn end.
  Poison: {
    name: "Poison",
    names: { en: "Poison", ru: { name: "Яд", acc: "Яд", count: ["Яд", "Яда", "Ядов"] } },
    triggers: [{ kind: "trigger", on: { on: "TurnEnd" } }],
    selectors: [{ kind: "holder" }],
    abilities: [{ effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }] }],
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
  Memories: {
    name: "Memories",
    names: { en: "Memories", ru: { name: "Воспоминания", acc: "Воспоминания", count: ["Воспоминание", "Воспоминания", "Воспоминаний"] } },
    counter: true,
    triggers: [{ kind: "trigger", on: { on: "TurnStart" } }],
    selectors: [{ kind: "holder" }],
    abilities: [{ effects: [{ kind: "performRemembered", mode: "all" }] }],
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
  // heads: double damage; tails: none (the branch word)
  AllIn: { name: "AllIn", family: "Strike", effects: [ALL_OR_NOTHING] },
  // awoken: on heads it also strikes again
  AllInAgain: {
    name: "AllInAgain",
    family: "Strike",
    effects: [{ kind: "branch", if: { kind: "flip", p: 0.5 }, then: [{ kind: "replaceAmount", amount: { kind: "twice", of: { kind: "stat", stat: "pwr", of: "holder" } } }, { kind: "strikeAgain" }], else: [{ kind: "replaceAmount", amount: c(0) }] }],
  },
  Sand: { name: "Sand", family: "Arcane", effects: [{ kind: "applyStatus", status: "Sands", stacks: c(1) }] },
  CopyPower: { name: "CopyPower", family: "Arcane", effects: [{ kind: "copyRule", withPower: true }] },
  Glow: { name: "Glow", family: "Heal", effects: [{ kind: "heal", amount: { kind: "stat", stat: "pwr", of: "holder" } }] },
  ShellAll: { name: "ShellAll", family: "Shield", effects: [{ kind: "applyStatus", status: "Shield", stacks: { kind: "plus", a: { kind: "count", fact: "hitsTaken" }, b: { kind: "count", fact: "hitsTakenByAllies" } } }] },
  Charge: { name: "Charge", family: "Control", effects: [{ kind: "applyStatus", status: "Strength", stacks: c(1) }] },
  Venom: { name: "Venom", family: "Poison", effects: [{ kind: "applyStatus", status: "Poison", stacks: c(1) }] },
  Guard: { name: "Guard", family: "Shield", effects: [{ kind: "applyStatus", status: "Shield", stacks: c(1) }] },
  SproutAwoken: {
    name: "SproutAwoken",
    family: "Summon",
    effects: [{ kind: "delay", turns: 2, effect: { kind: "transform", into: { ...rule("Tree", 8, 0, { on: "TurnStart" }, { kind: "allyBehind" }, "Grow"), also: [{ selectors: [{ kind: "neighbours" }], abilities: ["Guard"] }] } } }],
  },
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
export const Gambler: UnitDef = { ...rule("Gambler", 8, 2, { on: "Strike", striker: "holder" }, { kind: "holder" }, "AllIn"), triggers: [{ kind: "interceptor", on: { on: "Strike", striker: "holder" } }] };
export const Nautilus: UnitDef = rule("Nautilus", 10, 1, { on: "Hurt", unit: "holder" }, { kind: "holder" }, "Shell");
// Only what an enemy aimed at a random ally (its randomEnemy Who): Hurt or a status.
export const Magnet: UnitDef = {
  ...rule("Magnet", 12, 1, { on: "Hurt", unit: "otherAlly" }, { kind: "holder" }, "Pull"),
  triggers: [
    { kind: "interceptor", on: { on: "Hurt", unit: "otherAlly", aim: "random" } },
    { kind: "interceptor", on: { on: "StatusApplied", unit: "otherAlly", aim: "random" } },
  ],
};
export const Raven: UnitDef = { ...rule("Raven", 8, 1, { on: "Death", unit: "otherAlly" }, { kind: "eventUnit" }, "Recall"), statuses: [{ status: "Memory", stacks: 1 }] };
export const Seed: UnitDef = rule("Seed", 4, 0, { on: "BattleStart" }, { kind: "holder" }, "Sprout");
export const DejaVu: UnitDef = { ...rule("Déjà Vu", 10, 2, { on: "TurnStart" }, { kind: "frontEnemy" }, "Echo"), triggers: [{ kind: "trigger", on: { on: "Acted", unit: "enemy", front: true } }] };
export const GraveTree: UnitDef = rule("Grave Tree", 8, 1, { on: "Death", unit: "otherAlly" }, { kind: "eventUnit" }, "Sapling");

// The awoken forms (§4): the same When, one clause upgraded.
export const HourglassAwoken: UnitDef = rule("Hourglass", 8, 1, { on: "TurnEnd" }, { kind: "holder" }, "Sand");
export const MirrorAwoken: UnitDef = rule("Mirror", 6, 1, { on: "BattleStart" }, { kind: "frontEnemy" }, "CopyPower");
// Each time it survives this way, it heals its neighbours by its power (the
// "and" clause runs once the death is cancelled and the power paid).
export const CandleAwoken: UnitDef = { ...Candle, also: [{ selectors: [{ kind: "neighbours" }], abilities: ["Glow"] }] };
export const GamblerAwoken: UnitDef = { ...Gambler, abilities: ["AllInAgain"] };
export const NautilusAwoken: UnitDef = rule("Nautilus", 10, 1, { on: "Hurt", unit: "holder" }, { kind: "holder" }, "ShellAll");
export const MagnetAwoken: UnitDef = { ...Magnet, also: [{ selectors: [{ kind: "holder" }], abilities: ["Charge"] }] };
export const RavenAwoken: UnitDef = { ...Raven, statuses: [{ status: "Memories", stacks: 1 }] };
export const SeedAwoken: UnitDef = rule("Seed", 4, 0, { on: "BattleStart" }, { kind: "holder" }, "SproutAwoken");

const Hedgehog: UnitDef = rule("Hedgehog", 8, 1, { on: "Hurt", unit: "holder" }, { kind: "frontEnemy" }, "Zap");

/** An enemy that poisons a random enemy at each turn start. */
const Spitter = (hp = 30): UnitDef => rule("Spitter", hp, 1, { on: "TurnStart" }, { kind: "randomEnemy" }, "Venom");
/** An enemy that shoots a random enemy at each turn start (a randomly aimed Hurt). */
const Slinger = (hp = 30): UnitDef => rule("Slinger", hp, 1, { on: "TurnStart" }, { kind: "randomEnemy" }, "Shoot");

const SEED_MAGNET = 2;
const fight = (teamA: UnitDef[], teamB: UnitDef[], seed = 1): BattleInput => ({ teamA, teamB, seed, statuses: FIXTURE_STATUSES, abilities: FIXTURE_ABILITIES });

/** name → its battle and the trace line the battle shows. */
export const FIXTURES: Record<string, { input: BattleInput; trace: string }> = {
  hourglass: { input: fight([Hourglass, Archer(9)], [plain("Brute", 30, 1)]), trace: "Hourglass: Archer acts again" },
  mirror: { input: fight([Mirror], [Hedgehog]), trace: "Mirror copies the rule of Hedgehog" },
  candle: { input: fight([Candle], [plain("Brute", 30, 2)]), trace: "Candle intercepts: Candle survives at 1 HP, loses 1 PWR" },
  gambler: { input: fight([Gambler], [plain("Brute", 30, 1)]), trace: "Gambler: 50% → double" },
  nautilus: { input: fight([Nautilus], [plain("Brute", 30, 3)]), trace: "Nautilus → self (Nautilus) → Shield 3" },
  // A randomly aimed shot at Archer lands on Magnet; Brute's front strikes on Archer pass (no Redirected in the golden).
  magnet: { input: fight([Archer(6), Magnet], [Slinger()], SEED_MAGNET), trace: "Magnet intercepts: the hit on Archer lands on Magnet" },
  raven: { input: fight([Archer(1), Raven], [plain("Brute", 30, 2)]), trace: "Raven performs the rule of Archer" },
  seed: { input: fight([Seed, Archer()], [plain("Brute", 30, 1)]), trace: "Seed becomes Tree (0/4)" },
  gamblerTails: { input: fight([Gambler], [plain("Brute", 30, 1)], 2), trace: "Gambler's strike on Brute hits for 0" },
  magnetRandom: { input: fight([Archer(6), Magnet], [Spitter()], 3), trace: "Magnet intercepts: the status on Archer lands on Magnet" },
  hourglassAwoken: { input: fight([HourglassAwoken, Archer(9), rule("Bowman", 9, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Shoot")], [plain("Brute", 40, 1)]), trace: "Hourglass: Bowman acts again" },
  mirrorAwoken: { input: fight([MirrorAwoken], [Hedgehog]), trace: "Mirror copies the rule of Hedgehog" },
  candleAwoken: { input: fight([CandleAwoken, plain("Squire", 6, 1)], [plain("Brute", 30, 2)]), trace: "Candle intercepts: Candle survives at 1 HP, loses 1 PWR" },
  gamblerAwoken: { input: fight([GamblerAwoken], [plain("Brute", 30, 1)]), trace: "Gambler strikes again" },
  nautilusAwoken: { input: fight([plain("Squire", 4, 1), NautilusAwoken], [plain("Brute", 30, 3)]), trace: "Nautilus → self (Nautilus) → Shield 3" },
  magnetAwoken: { input: fight([Archer(6), MagnetAwoken], [Spitter()], 3), trace: "Magnet intercepts: the status on Archer lands on Magnet" },
  ravenAwoken: { input: fight([Archer(1), rule("Bowman", 1, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Shoot"), RavenAwoken], [plain("Brute", 30, 2)]), trace: "Raven performs the rule of Bowman" },
  seedAwoken: { input: fight([plain("Squire", 6, 1), SeedAwoken, Archer()], [plain("Brute", 30, 1)]), trace: "Seed becomes Tree (0/8)" },
  dejaVu: { input: fight([DejaVu], [Archer(12)]), trace: "Déjà Vu performs the rule of Archer" },
  graveTree: { input: fight([Archer(1), GraveTree], [plain("Brute", 30, 2)]), trace: "Grave Tree grows Sapling with the rule of Archer" },
};
