// The rules v2 pool (mission #1043, unit batch 1, #1046): units written as
// signature sentences in the v2 words, not as the name grammar of units.ts.
// A V2Row carries its two forms as UnitForms outright (interceptor Whens, a
// condition, position selectors, named abilities whose effects are words, a
// starting status), plus the shape class the design tags it with (page §4).
// Numbers (pwr, hp) come from the tune, never from the designer.
//
// The eight units of the design's §4 come first; their battles are the
// fixtures in src/words/fixtures.ts (goldens there).

import type { AbilityRegistry, StatusRegistry } from "../types.js";
import { ALL_OR_NOTHING } from "../words/all-or-nothing.js";
import type { Tier, UnitForm } from "./contract.js";
import { V2_NUMBERS } from "./units-v2-numbers.js";

/** The six shape classes of rules v2 §3, as the design tags each unit. */
export type Shape = "counter" | "copy" | "replacement" | "swing" | "scaling" | "redirect" | "memory" | "form" | "position" | "condition" | "rule";

export interface V2Row {
  v2: true;
  name: string;
  emoji: string;
  tier: Tier;
  shape: Shape;
  pwr: number;
  hp: number;
  /** One sentence on what the unit is about (unique across the pool). */
  archetype: string;
  forms: { sleeping: UnitForm; awoken: UnitForm };
}

export const isV2Row = (row: object): row is V2Row => (row as { v2?: true }).v2 === true;

const c = (value: number) => ({ kind: "const" as const, value });

/** Statuses the v2 units need beyond the stress registry: counters carrying
 * a rule of their own (Hourglass's grains, Raven's memory). */
export const V2_STATUSES: StatusRegistry = {
  // Hourglass's counter: at 3 grains, spend them and the ally behind acts
  // again (awoken: Sands, every ally behind). The rule lives on the counter so
  // its condition reads the grain just added at the same turn end.
  Grains: {
    name: "Grains",
    names: { en: "Grains", ru: { name: "Песчинки", acc: "Песчинки", count: ["Песчинка", "Песчинки", "Песчинок"] } },
    counter: true,
    triggers: [{ kind: "trigger", on: { on: "TurnEnd" } }],
    condition: { kind: "stacksAtLeast", status: "Grains", n: 3, of: "holder" },
    abilities: [
      // each status ability is its own reactor checking the condition, so the grains are spent last
      { selectors: [{ kind: "allyBehind" }], effects: [{ kind: "actAgain" }] },
      { selectors: [{ kind: "holder" }], effects: [{ kind: "consumeStacks", stacks: c(3) }] },
    ],
  },
  Sands: {
    name: "Sands",
    names: { en: "Sands", ru: { name: "Пески", acc: "Пески", count: ["Песок", "Песка", "Песков"] } },
    counter: true,
    triggers: [{ kind: "trigger", on: { on: "TurnEnd" } }],
    condition: { kind: "stacksAtLeast", status: "Sands", n: 3, of: "holder" },
    abilities: [
      { selectors: [{ kind: "alliesBehind" }], effects: [{ kind: "actAgain" }] },
      { selectors: [{ kind: "holder" }], effects: [{ kind: "consumeStacks", stacks: c(3) }] },
    ],
  },
  // Drummer's counter: at 3 beats, spend them and every ally behind gains 1
  // power (awoken: Drums, the whole line).
  Beats: {
    name: "Beats",
    names: { en: "Beats", ru: { name: "Такты", acc: "Такты", count: ["Такт", "Такта", "Тактов"] } },
    counter: true,
    triggers: [{ kind: "trigger", on: { on: "TurnEnd" } }],
    condition: { kind: "stacksAtLeast", status: "Beats", n: 3, of: "holder" },
    abilities: [
      { selectors: [{ kind: "alliesBehind" }], effects: [{ kind: "applyStatus", status: "Strength", stacks: c(1) }] },
      { selectors: [{ kind: "holder" }], effects: [{ kind: "consumeStacks", stacks: c(3) }] },
    ],
  },
  Drums: {
    name: "Drums",
    names: { en: "Drums", ru: { name: "Барабаны", acc: "Барабаны", count: ["Барабан", "Барабана", "Барабанов"] } },
    counter: true,
    triggers: [{ kind: "trigger", on: { on: "TurnEnd" } }],
    condition: { kind: "stacksAtLeast", status: "Drums", n: 3, of: "holder" },
    abilities: [
      { selectors: [{ kind: "allAllies" }], effects: [{ kind: "applyStatus", status: "Strength", stacks: c(1) }] },
      { selectors: [{ kind: "holder" }], effects: [{ kind: "consumeStacks", stacks: c(3) }] },
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
  Memories: {
    name: "Memories",
    names: { en: "Memories", ru: { name: "Воспоминания", acc: "Воспоминания", count: ["Воспоминание", "Воспоминания", "Воспоминаний"] } },
    counter: true,
    triggers: [{ kind: "trigger", on: { on: "TurnStart" } }],
    selectors: [{ kind: "holder" }],
    abilities: [{ effects: [{ kind: "performRemembered", mode: "all" }] }],
  },
};

const pwrOfHolder = { kind: "stat" as const, stat: "pwr" as const, of: "holder" as const };

/** The Tree Seed becomes: gives the ally behind it 1 power each turn (awoken: and shields its neighbours). */
const tree = (awoken: boolean) => ({
  name: "Tree",
  names: { ru: { name: "Дерево", acc: "Дерево" } },
  base: { hp: 8, pwr: 0 },
  triggers: [{ kind: "trigger" as const, on: { on: "TurnStart" as const } }],
  selectors: [{ kind: "allyBehind" as const }],
  abilities: ["Grow"],
  ...(awoken ? { also: [{ selectors: [{ kind: "neighbours" as const }], abilities: ["Guard"] }] } : {}),
});

/** The Chrysalis becomes: a Butterfly that strikes every enemy each turn
 * (awoken: a Monarch, which also powers every ally). */
const butterfly = (awoken: boolean) => ({
  name: awoken ? "Monarch" : "Butterfly",
  names: { ru: awoken ? { name: "Монарх", acc: "Монарха" } : { name: "Бабочка", acc: "Бабочку" } },
  base: { hp: 6, pwr: 2 },
  triggers: [{ kind: "trigger" as const, on: { on: "TurnStart" as const } }],
  selectors: [{ kind: "allEnemies" as const }],
  abilities: ["Smite"],
  ...(awoken ? { also: [{ selectors: [{ kind: "allAllies" as const }], abilities: ["Charge"] }] } : {}),
});

/** Named abilities of the v2 units: each is a word sentence, not a "Hit 1". */
export const V2_ABILITIES: AbilityRegistry = {
  // batch 1, the twelve after the eight (#1046)
  Sip: { name: "Sip", family: "Heal", effects: [{ kind: "heal", amount: { kind: "half", of: pwrOfHolder } }] },
  Fury: { name: "Fury", family: "Strike", effects: [{ kind: "replaceAmount", amount: { kind: "twice", of: pwrOfHolder } }] },
  FuryAgain: { name: "FuryAgain", family: "Strike", effects: [{ kind: "replaceAmount", amount: { kind: "twice", of: pwrOfHolder } }, { kind: "strikeAgain" }] },
  Beat: { name: "Beat", family: "Arcane", effects: [{ kind: "applyStatus", status: "Beats", stacks: c(1) }] },
  Drum: { name: "Drum", family: "Arcane", effects: [{ kind: "applyStatus", status: "Drums", stacks: c(1) }] },
  Emerge: { name: "Emerge", family: "Summon", effects: [{ kind: "transform", into: butterfly(false) }] },
  EmergeMonarch: { name: "EmergeMonarch", family: "Summon", effects: [{ kind: "transform", into: butterfly(true) }] },
  Swap: { name: "Swap", family: "Arcane", effects: [{ kind: "swapRules" }] },
  Toll: { name: "Toll", family: "Strike", effects: [{ kind: "damage", amount: { kind: "plus", a: pwrOfHolder, b: { kind: "count", fact: "deadAllies" } } }] },
  Encore: { name: "Encore", family: "Arcane", effects: [{ kind: "actAgain" }] },
  Grain: { name: "Grain", family: "Arcane", effects: [{ kind: "applyStatus", status: "Grains", stacks: c(1) }] },
  Sand: { name: "Sand", family: "Arcane", effects: [{ kind: "applyStatus", status: "Sands", stacks: c(1) }] },
  Copy: { name: "Copy", family: "Arcane", effects: [{ kind: "copyRule" }] },
  CopyPower: { name: "CopyPower", family: "Arcane", effects: [{ kind: "copyRule", withPower: true }] },
  Gutter: { name: "Gutter", family: "Shield", effects: [{ kind: "preventDeath", at: 1, cost: { stat: "pwr", amount: 1 } }] },
  Glow: { name: "Glow", family: "Heal", effects: [{ kind: "heal", amount: pwrOfHolder }] },
  // heads: double damage; tails: none (the branch word)
  AllIn: { name: "AllIn", family: "Strike", effects: [ALL_OR_NOTHING] },
  // awoken: on heads it also strikes again
  AllInAgain: {
    name: "AllInAgain",
    family: "Strike",
    effects: [{ kind: "branch", if: { kind: "flip", p: 0.5 }, then: [{ kind: "replaceAmount", amount: { kind: "twice", of: pwrOfHolder } }, { kind: "strikeAgain" }], else: [{ kind: "replaceAmount", amount: c(0) }] }],
  },
  Shell: { name: "Shell", family: "Shield", effects: [{ kind: "applyStatus", status: "Shield", stacks: { kind: "count", fact: "hitsTaken" } }] },
  ShellAll: { name: "ShellAll", family: "Shield", effects: [{ kind: "applyStatus", status: "Shield", stacks: { kind: "plus", a: { kind: "count", fact: "hitsTaken" }, b: { kind: "count", fact: "hitsTakenByAllies" } } }] },
  Pull: { name: "Pull", family: "Control", effects: [{ kind: "redirect", to: { kind: "holder" } }] },
  Charge: { name: "Charge", family: "Control", effects: [{ kind: "applyStatus", status: "Strength", stacks: c(1) }] },
  Recall: { name: "Recall", family: "Arcane", effects: [{ kind: "remember" }] },
  Sprout: { name: "Sprout", family: "Summon", effects: [{ kind: "delay", turns: 2, effect: { kind: "transform", into: tree(false) } }] },
  SproutAwoken: { name: "SproutAwoken", family: "Summon", effects: [{ kind: "delay", turns: 2, effect: { kind: "transform", into: tree(true) } }] },
  Grow: { name: "Grow", family: "Control", effects: [{ kind: "applyStatus", status: "Strength", stacks: c(1) }] },
  Guard: { name: "Guard", family: "Shield", effects: [{ kind: "applyStatus", status: "Shield", stacks: c(1) }] },
};

type Form = Omit<UnitForm, "who" | "does"> & { who?: UnitForm["who"]; does: string };
const sleeping = (when: UnitForm["when"], who: UnitForm["who"], does: string, rest: Partial<UnitForm> = {}): UnitForm => ({ when, who, does: [does], ...rest });
const trig = (on: UnitForm["when"][number]["on"]): UnitForm["when"] => [{ kind: "trigger", on }];
const intercept = (on: UnitForm["when"][number]["on"]): UnitForm["when"] => [{ kind: "interceptor", on }];
const me: UnitForm["who"] = [{ kind: "holder" }];

/** The awoken form: the sleeping one with one clause upgraded. */
const awake = (s: UnitForm, up: Partial<Form>): UnitForm => ({ ...s, ...(up.does !== undefined ? { does: [up.does] } : {}), ...(up.who ? { who: up.who } : {}), ...(up.also ? { also: up.also } : {}), ...(up.statuses ? { statuses: up.statuses } : {}) });

const v2 = (name: string, emoji: string, tier: Tier, shape: Shape, pwr: number, hp: number, archetype: string, s: UnitForm, up: Partial<Form>): V2Row =>
  ({ v2: true, name, emoji, tier, shape, pwr, hp, ...V2_NUMBERS[name], archetype, forms: { sleeping: s, awoken: awake(s, up) } });

// The pwr and hp columns are the designer's placeholders; the tune's numbers
// (units-v2-numbers.ts, scripts/tune-v2.ts) replace them. A V2Row's knobs are
// pwr and hp only.
const Hourglass = sleeping(trig({ on: "TurnEnd" }), me, "Grain");
const Mirror = sleeping(trig({ on: "BattleStart" }), [{ kind: "frontEnemy" }], "Copy");
const Candle = sleeping(intercept({ on: "Death", unit: "holder" }), me, "Gutter");
const Gambler = sleeping(intercept({ on: "Strike", striker: "holder" }), me, "AllIn");
const Nautilus = sleeping(trig({ on: "Hurt", unit: "holder" }), me, "Shell");
const Magnet = sleeping(
  [{ kind: "interceptor", on: { on: "Hurt", unit: "otherAlly", aim: "random" } }, { kind: "interceptor", on: { on: "StatusApplied", unit: "otherAlly", aim: "random" } }],
  me,
  "Pull",
);
const Raven = sleeping(trig({ on: "Death", unit: "otherAlly" }), [{ kind: "eventUnit" }], "Recall", { statuses: [{ status: "Memory", stacks: 1 }] });
const Seed = sleeping(trig({ on: "BattleStart" }), me, "Sprout");

// The twelve after the eight: every shape class and tier again, higher tiers
// with more clauses (page §8).
const Sniper = sleeping(trig({ on: "TurnStart" }), [{ kind: "lastEnemy" }], "Smite");
const Medic = sleeping(trig({ on: "Hurt", unit: "otherAlly" }), [{ kind: "eventUnit" }], "Mend");
const Leech = sleeping(trig({ on: "Strike", striker: "holder" }), me, "Sip");
const Metronome = sleeping(trig({ on: "TurnStart" }), [{ kind: "allyBehind" }], "Charge", { condition: { kind: "turnEvery", n: 2 } });
const LoneWolf = sleeping(intercept({ on: "Strike", striker: "holder" }), me, "Fury", { condition: { kind: "alliesAlive", op: "=", n: 0 } });
const Drummer = sleeping(trig({ on: "Strike", striker: "otherAlly" }), me, "Beat");
const Bodyguard = sleeping(intercept({ on: "Hurt", unit: "otherAlly" }), me, "Pull", { condition: { kind: "oncePerTurn" } });
const Chrysalis = sleeping(trig({ on: "TurnStart" }), me, "Emerge", { condition: { kind: "turnIs", n: 3 } });
const Trickster = sleeping(trig({ on: "BattleStart" }), [{ kind: "frontEnemy" }], "Swap");
const Understudy = sleeping(trig({ on: "Death", unit: "otherAlly" }), [{ kind: "eventUnit" }], "Copy", { condition: { kind: "onceHolder" } });
const Gravedigger = sleeping(trig({ on: "TurnStart" }), [{ kind: "frontEnemy" }], "Toll");
const Conductor = sleeping(trig({ on: "TurnStart" }), [{ kind: "neighbours" }], "Encore", { condition: { kind: "turnEvery", n: 2 } });

/* eslint-disable prettier/prettier */
export const ROWS_V2: V2Row[] = [
  v2("Hourglass", "⏳", 2, "counter",     1,  8, "Keeps time: every third grain, the ally behind it acts again.",            Hourglass, { does: "Sand" }),
  v2("Mirror",    "🪞", 2, "copy",        1,  6, "Becomes whatever stands in front of it.",                                 Mirror,    { does: "CopyPower" }),
  v2("Candle",    "🕯️", 3, "replacement", 3,  3, "Burns down instead of dying, one power at a time.",                       Candle,    { also: [{ who: [{ kind: "neighbours" }], does: ["Glow"] }] }),
  v2("Gambler",   "🎲", 1, "swing",       2,  8, "Every strike is a coin: double or nothing.",                               Gambler,   { does: "AllInAgain" }),
  v2("Nautilus",  "🐚", 1, "scaling",     1, 10, "Grows a shell from every hit it takes.",                                  Nautilus,  { does: "ShellAll" }),
  v2("Magnet",    "🧲", 3, "redirect",    1, 12, "Pulls the enemy's random shots onto itself.",                             Magnet,    { also: [{ who: me, does: ["Charge"] }] }),
  v2("Raven",     "🐦‍⬛", 4, "memory",      1,  8, "Remembers the rules of the fallen and performs them.",                   Raven,     { statuses: [{ status: "Memories", stacks: 1 }] }),
  v2("Seed",      "🌱", 2, "form",        0,  4, "Waits two turns, then grows into a Tree that feeds the ally behind it.",  Seed,      { does: "SproutAwoken" }),
  // the twelve
  v2("Sniper",      "🎯", 1, "position",    2,  6, "Shoots the enemy at the back every turn.",                              Sniper,      { who: [{ kind: "lastEnemy" }, { kind: "frontEnemy" }] }),
  v2("Medic",       "💉", 1, "position",    1,  8, "Heals whichever ally just got hurt.",                                   Medic,       { does: "Mend + Guard" }),
  v2("Leech",       "🧛", 1, "scaling",     2,  7, "Drinks from every strike it makes.",                                    Leech,       { does: "Glow" }),
  v2("Metronome",   "🎵", 2, "condition",   1,  8, "Every second turn, powers the ally behind it.",                         Metronome,   { who: [{ kind: "alliesBehind" }] }),
  v2("Lone Wolf",   "🐺", 2, "replacement", 3,  7, "Alone, its strikes deal double.",                                       LoneWolf,    { does: "FuryAgain" }),
  v2("Drummer",     "🥁", 2, "counter",     1,  9, "Every third ally strike, the line behind it beats stronger.",           Drummer,     { does: "Drum" }),
  v2("Bodyguard",   "🕴️", 2, "redirect",    1, 11, "Once a turn, takes a hit meant for an ally.",                           Bodyguard,   { also: [{ who: [{ kind: "eventUnit" }], does: ["Guard"] }] }),
  v2("Chrysalis",   "🐛", 3, "form",        0,  7, "On the third turn, becomes a Butterfly that strikes every enemy.",      Chrysalis,   { does: "EmergeMonarch" }),
  v2("Trickster",   "🃏", 3, "rule",        2,  8, "Trades rules with the front enemy before the first blow.",             Trickster,   { who: [{ kind: "frontEnemy" }, { kind: "lastEnemy" }] }),
  v2("Understudy",  "🎭", 3, "copy",        1,  8, "When an ally falls, takes over its rule for good.",                     Understudy,  { does: "CopyPower" }),
  v2("Gravedigger", "⚰️", 4, "scaling",     2, 10, "Hits the front enemy harder for every fallen ally.",                    Gravedigger, { who: [{ kind: "allEnemies" }] }),
  v2("Conductor",   "🎼", 4, "rule",        1,  9, "Every second turn, its neighbours act again.",                          Conductor,   { also: [{ who: [{ kind: "neighbours" }], does: ["Charge"] }] }),
];
/* eslint-enable prettier/prettier */
