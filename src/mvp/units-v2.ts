// The rules v2 pool (mission #1043, unit batch 1, #1046): units written as
// signature sentences in the v2 words, not as the name grammar of units.ts.
// A V2Row carries its two forms as UnitForms outright (interceptor Whens, a
// condition, position selectors, named abilities whose effects are words, a
// starting status), plus the shape class the design tags it with (page §4).
// Numbers (pwr, hp) come from the tune, never from the designer.
//
// The eight units of the design's §4 come first; their battles are the
// fixtures in src/words/fixtures.ts (goldens there).

import type { AbilityRegistry, Effect, StatusRegistry } from "../types.js";
import { ALL_OR_NOTHING } from "../words/all-or-nothing.js";
import type { Tier, UnitForm } from "./contract.js";
import { V2_NUMBERS } from "./units-v2-numbers.js";
import { V2_SAY } from "./units-v2-say.js";

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
  /** PWR and HP the unit gains when it awakens (the tune's awoken knob, so
   * Awoken always adds to the team; units-v2-numbers.ts). */
  awoken?: { pwr: number; hp: number };
  /** One sentence on what the unit is about (unique across the pool). */
  archetype: string;
  forms: { sleeping: UnitForm; awoken: UnitForm };
}

export const isV2Row = (row: object): row is V2Row => (row as { v2?: true }).v2 === true;

const c = (value: number) => ({ kind: "const" as const, value });

/** Statuses the v2 units need beyond the stress registry: counters carrying
 * a rule of their own (Hourglass's grains, Raven's memory). */
export const V2_STATUSES: StatusRegistry = {
  // Charge ⚡ (#1063): the one shared counter of the pool. Any unit may build
  // it up and spend it; the counter itself has no rule. A Charge unit gains 1
  // in its Does and spends 3 in its "and" clauses the moment it holds 3
  // (V2_ABILITIES: Spark, Spend, and the charged payloads), so every card and
  // log row shows the same counter.
  Charge: {
    name: "Charge",
    names: { en: "Charge", ru: { name: "Заряд", acc: "Заряд", count: ["Заряд", "Заряда", "Зарядов"] } },
    counter: true,
    abilities: [],
  },
  // Raven's second rule (awoken, #1063): the rule it took from a fallen ally,
  // performed at each turn start for the rest of the battle.
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

/** The Chrysalis becomes: a Butterfly that strikes every enemy each turn
 * (awoken: a Monarch, which also powers every ally). */
const butterfly = (awoken: boolean) => ({
  name: awoken ? "Monarch" : "Butterfly",
  names: { ru: awoken ? { name: "Монарх", acc: "Монарха" } : { name: "Бабочка", acc: "Бабочку" } },
  base: { hp: 4, pwr: 2 }, // #1047 pass 2: 4 HP, was 6
  triggers: [{ kind: "trigger" as const, on: { on: "TurnStart" as const } }],
  selectors: [{ kind: "allEnemies" as const }],
  abilities: ["Smite"],
  ...(awoken ? { also: [{ selectors: [{ kind: "allAllies" as const }], abilities: ["Rally"] }] } : {}),
});

/** Hydra's heads: each a plain 2/3 with no rule (#1063). The validator wants
 * a recipe, so it carries the examples' dead-letter one (a Mend on its own
 * death, which never shows). */
const head = {
  name: "Head",
  names: { ru: { name: "Голова", acc: "Голову" } },
  base: { hp: 3, pwr: 2 },
  triggers: [{ kind: "trigger" as const, on: { on: "Death" as const, unit: "holder" as const } }],
  selectors: [{ kind: "holder" as const }],
  abilities: ["Mend"],
};

/** "It holds 3 Charge": the moment a Charge unit reaches 3, its charged
 * clauses fire and Spend takes the 3 back (#1063). */
const charged = { kind: "stacksAtLeast" as const, status: "Charge", n: 3, of: "holder" as const };
const whenCharged = (then: Effect[]) => ({ kind: "branch" as const, if: charged, then, else: [] });

/** Named abilities of the v2 units: each is a word sentence, not a "Hit 1". */
export const V2_ABILITIES: AbilityRegistry = {
  // batch 1, the twelve after the eight (#1046)
  Sip: { name: "Sip", family: "Heal", effects: [{ kind: "heal", amount: { kind: "half", of: pwrOfHolder } }] },
  Fury: { name: "Fury", family: "Strike", effects: [{ kind: "replaceAmount", amount: { kind: "twice", of: pwrOfHolder } }] },
  FuryAgain: { name: "FuryAgain", family: "Strike", effects: [{ kind: "replaceAmount", amount: { kind: "twice", of: pwrOfHolder } }, { kind: "strikeAgain" }] },
  Emerge: { name: "Emerge", family: "Summon", effects: [{ kind: "transform", into: butterfly(false) }] },
  EmergeMonarch: { name: "EmergeMonarch", family: "Summon", effects: [{ kind: "transform", into: butterfly(true) }] },
  Swap: { name: "Swap", family: "Arcane", effects: [{ kind: "swapRules" }] },
  // #1047 (Maks's (b)): 1 per fallen ally, no longer its power on top
  Toll: { name: "Toll", family: "Strike", effects: [{ kind: "damage", amount: { kind: "count", fact: "deadAllies" } }] },
  Encore: { name: "Encore", family: "Arcane", effects: [{ kind: "actAgain" }] },
  Copy: { name: "Copy", family: "Arcane", effects: [{ kind: "copyRule" }] },
  CopyPower: { name: "CopyPower", family: "Arcane", effects: [{ kind: "copyRule", withPower: true }] },
  Gutter: { name: "Gutter", family: "Shield", effects: [{ kind: "preventDeath", at: 1, cost: { stat: "pwr", amount: 1 } }] },
  Glow: { name: "Glow", family: "Heal", effects: [{ kind: "heal", amount: pwrOfHolder }] },
  // heads: double damage; tails: none (the branch word)
  AllIn: { name: "AllIn", family: "Strike", effects: [ALL_OR_NOTHING] },
  // awoken: on a win it also strikes again
  AllInAgain: {
    name: "AllInAgain",
    family: "Strike",
    effects: [{ kind: "branch", if: { kind: "flip", p: 0.5 }, then: [{ kind: "replaceAmount", amount: { kind: "twice", of: pwrOfHolder } }, { kind: "strikeAgain" }], else: [{ kind: "replaceAmount", amount: c(0) }] }],
  },
  // #1047 (Maks's (b)): a flat shell per hit, no longer the running count of hits
  Shell: { name: "Shell", family: "Shield", effects: [{ kind: "applyStatus", status: "Shield", stacks: c(1) }] },
  ShellAll: { name: "ShellAll", family: "Shield", effects: [{ kind: "applyStatus", status: "Shield", stacks: c(2) }] },
  Pull: { name: "Pull", family: "Control", effects: [{ kind: "redirect", to: { kind: "holder" } }] },
  Rally: { name: "Rally", family: "Control", effects: [{ kind: "applyStatus", status: "Strength", stacks: c(1) }] },
  Recall: { name: "Recall", family: "Arcane", effects: [{ kind: "remember" }] },
  // Raven (#1063): take the fallen ally's rule and use it once, right away
  RecallNow: { name: "RecallNow", family: "Arcane", effects: [{ kind: "remember" }, { kind: "performRemembered", mode: "once" }] },
  Grow: { name: "Grow", family: "Control", effects: [{ kind: "applyStatus", status: "Strength", stacks: c(1) }] },
  Guard: { name: "Guard", family: "Shield", effects: [{ kind: "applyStatus", status: "Shield", stacks: c(1) }] },
  // Seed (#1063): grows 2 HP a turn, no Tree
  Grow2: { name: "Grow2", family: "Heal", effects: [{ kind: "heal", amount: c(2) }] },
  // batch 2 (#1047)
  Prick: { name: "Prick", family: "Strike", effects: [{ kind: "damage", amount: c(1) }] },
  Spine: { name: "Spine", family: "Strike", effects: [{ kind: "damage", amount: pwrOfHolder }] },
  Feast: { name: "Feast", family: "Control", effects: [{ kind: "applyStatus", status: "Strength", stacks: c(1) }, { kind: "heal", amount: pwrOfHolder }] },
  Sting: { name: "Sting", family: "Poison", effects: [{ kind: "applyStatus", status: "Poison", stacks: c(1) }] },
  Bounce: { name: "Bounce", family: "Control", effects: [{ kind: "redirect", to: { kind: "randomEnemy" } }] },
  Hush2: { name: "Hush2", family: "Control", effects: [{ kind: "silence" }] },
  Fortune: { name: "Fortune", family: "Control", effects: [{ kind: "branch", if: { kind: "flip", p: 0.5 }, then: [{ kind: "applyStatus", status: "Strength", stacks: c(1) }], else: [{ kind: "applyStatus", status: "Shield", stacks: c(1) }] }] }, // #1047 pass 2: 1, was 2
  FortuneAwoken: { name: "FortuneAwoken", family: "Control", effects: [{ kind: "branch", if: { kind: "flip", p: 0.5 }, then: [{ kind: "applyStatus", status: "Strength", stacks: c(2) }], else: [{ kind: "applyStatus", status: "Shield", stacks: c(2) }] }] },
  Block: { name: "Block", family: "Shield", effects: [{ kind: "cancel" }] },
  Raise: { name: "Raise", family: "Summon", effects: [{ kind: "resurrect", hp: c(1) }] },
  RaiseAwoken: { name: "RaiseAwoken", family: "Summon", effects: [{ kind: "resurrect", hp: pwrOfHolder }] },
  Heads: { name: "Heads", family: "Summon", effects: [{ kind: "summon", unit: head }, { kind: "summon", unit: head }] },
  HeadsAwoken: { name: "HeadsAwoken", family: "Summon", effects: [{ kind: "summon", unit: head }, { kind: "summon", unit: head }, { kind: "summon", unit: head }] },
  Hex: { name: "Hex", family: "Control", effects: [{ kind: "applyStatus", status: "Curse", stacks: c(1) }] },
  Rage: { name: "Rage", family: "Strike", effects: [{ kind: "replaceAmount", amount: { kind: "plus", a: pwrOfHolder, b: { kind: "missingHp" } } }] },
  // the rethink (#1063): Charge ⚡, one shared counter
  Spark: { name: "Spark", family: "Arcane", effects: [{ kind: "applyStatus", status: "Charge", stacks: c(1) }] },
  Spend: { name: "Spend", family: "Arcane", effects: [whenCharged([{ kind: "consumeStacks", status: "Charge", stacks: c(3) }])] },
  EncoreCharged: { name: "EncoreCharged", family: "Arcane", effects: [whenCharged([{ kind: "actAgain" }])] },
  RallyCharged: { name: "RallyCharged", family: "Control", effects: [whenCharged([{ kind: "applyStatus", status: "Strength", stacks: c(1) }])] },
  GuardCharged: { name: "GuardCharged", family: "Shield", effects: [whenCharged([{ kind: "applyStatus", status: "Shield", stacks: c(1) }])] },
  GuardMendCharged: { name: "GuardMendCharged", family: "Shield", effects: [whenCharged([{ kind: "applyStatus", status: "Shield", stacks: c(1) }, { kind: "heal", amount: c(1) }])] },
  BlastCharged: { name: "BlastCharged", family: "Strike", effects: [whenCharged([{ kind: "damage", amount: pwrOfHolder }])] },
  BlastHexCharged: { name: "BlastHexCharged", family: "Strike", effects: [whenCharged([{ kind: "damage", amount: pwrOfHolder }, { kind: "applyStatus", status: "Curse", stacks: c(1) }])] },
  // Freeze and Blessing, used at last
  Chill: { name: "Chill", family: "Control", effects: [{ kind: "applyStatus", status: "Freeze", stacks: c(1) }] },
  Chill2: { name: "Chill2", family: "Control", effects: [{ kind: "applyStatus", status: "Freeze", stacks: c(2) }] },
  Bless: { name: "Bless", family: "Shield", effects: [{ kind: "applyStatus", status: "Blessing", stacks: pwrOfHolder }] },
};

type Form = Omit<UnitForm, "who" | "does"> & { who?: UnitForm["who"]; does: string };
const sleeping = (when: UnitForm["when"], who: UnitForm["who"], does: string, rest: Partial<UnitForm> = {}): UnitForm => ({ when, who, does: [does], ...rest });
const trig = (on: UnitForm["when"][number]["on"]): UnitForm["when"] => [{ kind: "trigger", on }];
const intercept = (on: UnitForm["when"][number]["on"]): UnitForm["when"] => [{ kind: "interceptor", on }];
const me: UnitForm["who"] = [{ kind: "holder" }];

/** The awoken form: the sleeping one with one clause upgraded. */
const awake = (s: UnitForm, up: Partial<Form>): UnitForm => ({ ...s, ...(up.does !== undefined ? { does: [up.does] } : {}), ...(up.who ? { who: up.who } : {}), ...(up.also ? { also: up.also } : {}), ...(up.statuses ? { statuses: up.statuses } : {}), ...(up.condition ? { condition: up.condition } : {}) });

/** Each form carries its card sentence (units-v2-say.ts), set per form so
 * the awoken one never reads the sleeping one's. */
const v2 = (name: string, emoji: string, tier: Tier, shape: Shape, pwr: number, hp: number, archetype: string, s: UnitForm, up: Partial<Form>): V2Row => {
  const say = V2_SAY[name]?.();
  const awoken = awake(s, up);
  return {
    v2: true, name, emoji, tier, shape, pwr, hp, ...V2_NUMBERS[name], archetype,
    forms: say ? { sleeping: { ...s, say: { en: say[0], ru: say[2] } }, awoken: { ...awoken, say: { en: say[1], ru: say[3] } } } : { sleeping: s, awoken },
  };
};

// The pwr and hp columns are the designer's placeholders; the tune's numbers
// (units-v2-numbers.ts, scripts/tune-v2.ts) replace them. A V2Row's knobs are
// pwr and hp only.
// Charge units (#1063): the Does gains 1 Charge, the "and" clauses fire once
// it holds 3, and Spend (last) takes the 3 back.
const Hourglass = sleeping(trig({ on: "TurnEnd" }), me, "Spark", { also: [{ who: [{ kind: "allyBehind" }], does: ["EncoreCharged"] }, { who: me, does: ["Spend"] }] });
const Mirror = sleeping(trig({ on: "BattleStart" }), [{ kind: "frontEnemy" }], "Copy");
const Candle = sleeping(intercept({ on: "Death", unit: "holder" }), me, "Gutter");
const Gambler = sleeping(intercept({ on: "Strike", striker: "holder" }), me, "AllIn");
const Nautilus = sleeping(trig({ on: "Hurt", unit: "holder" }), me, "Shell");
// #1063: Poison, Curse and Freeze aimed at its allies, no longer "random hits"
const Magnet = sleeping(
  [
    { kind: "interceptor", on: { on: "StatusApplied", unit: "otherAlly", status: "Poison" } },
    { kind: "interceptor", on: { on: "StatusApplied", unit: "otherAlly", status: "Curse" } },
    { kind: "interceptor", on: { on: "StatusApplied", unit: "otherAlly", status: "Freeze" } },
  ],
  me,
  "Pull",
);
// #1063: no memory list; uses the fallen ally's rule once, right away (awoken: and keeps it)
const Raven = sleeping(trig({ on: "Death", unit: "otherAlly" }), [{ kind: "eventUnit" }], "RecallNow");
// #1063: no Tree; from turn 3 on it grows and feeds the ally behind
const Seed = sleeping(trig({ on: "TurnStart" }), me, "Grow2", { condition: { kind: "turnAtLeast", n: 3 }, also: [{ who: [{ kind: "allyBehind" }], does: ["Grow"] }] });

// The twelve after the eight: every shape class and tier again, higher tiers
// with more clauses (page §8).
const Sniper = sleeping(trig({ on: "TurnStart" }), [{ kind: "lastEnemy" }], "Smite");
const Medic = sleeping(trig({ on: "Hurt", unit: "otherAlly" }), [{ kind: "eventUnit" }], "Mend");
const Leech = sleeping(trig({ on: "Strike", striker: "holder" }), me, "Sip");
// #1063: alliesAlive counts the holder, so "no allies left" is 1 (it was 0 and never fired)
const LoneWolf = sleeping(intercept({ on: "Strike", striker: "holder" }), me, "Fury", { condition: { kind: "alliesAlive", op: "=", n: 1 } });
const Drummer = sleeping(trig({ on: "Strike", striker: "otherAlly" }), me, "Spark", { also: [{ who: [{ kind: "alliesBehind" }], does: ["RallyCharged"] }, { who: me, does: ["Spend"] }] });
const Bodyguard = sleeping(intercept({ on: "Hurt", unit: "otherAlly" }), me, "Pull", { condition: { kind: "oncePerTurn" } });
const Chrysalis = sleeping(trig({ on: "TurnStart" }), me, "Emerge", { condition: { kind: "turnIs", n: 3 } });
const Trickster = sleeping(trig({ on: "BattleStart" }), [{ kind: "frontEnemy" }], "Swap");
const Gravedigger = sleeping(trig({ on: "TurnStart" }), [{ kind: "frontEnemy" }], "Toll");
const Conductor = sleeping(trig({ on: "TurnStart" }), [{ kind: "neighbours" }], "Encore", { condition: { kind: "turnEvery", n: 3 } }); // #1047 pass 2: every 3rd turn, was 2nd

// Batch 2 (#1047): twenty more, every shape class and tier again.
const Thornback = sleeping(trig({ on: "Hurt", unit: "holder" }), [{ kind: "attacker" }], "Prick");
// #1063: a thief that steals power every turn, no coin
const CoinThief = sleeping(trig({ on: "TurnStart" }), [{ kind: "randomEnemy" }], "Hex", { also: [{ who: me, does: ["Rally"] }] });
const Lantern = sleeping(trig({ on: "TurnStart" }), [{ kind: "allyAhead" }], "Guard", { condition: { kind: "unhurt" } });
const Hornet = sleeping(trig({ on: "Strike", striker: "holder" }), [{ kind: "eventTarget" }], "Sting");
const Prism = sleeping(intercept({ on: "Hurt", unit: "holder", aim: "random" }), me, "Bounce");
// #1047: from 3 allies standing (was 2), War Horn sat low at the PWR/HP cap
const Horn = sleeping(trig({ on: "TurnStart" }), [{ kind: "allAllies" }], "Rally", { condition: { kind: "alliesAlive", op: "<=", n: 3 } });
// #1063: any unit falling, ally or enemy (takes in Scavenger)
const Vulture = sleeping(trig({ on: "Death", unit: "any" }), me, "Rally");
// #1047 (Maks's (b)): on turn 2, so the back enemy's battle-start rule still fires
const Hexer = sleeping(trig({ on: "TurnStart" }), [{ kind: "lastEnemy" }], "Hush2", { condition: { kind: "turnIs", n: 2 } });
const Fortune = sleeping(trig({ on: "BattleStart" }), [{ kind: "allAllies" }], "Fortune");
const Aegis = sleeping(intercept({ on: "Hurt", unit: "holder" }), me, "Block", { condition: { kind: "oncePerTurn" } });
const Gravecaller = sleeping(trig({ on: "Death", unit: "otherAlly" }), [{ kind: "lastDeadAlly" }], "Raise", { condition: { kind: "onceHolder" } });
const Hydra = sleeping(trig({ on: "Death", unit: "holder" }), me, "Heads");
const Archmage = sleeping(trig({ on: "TurnEnd" }), me, "Spark", { also: [{ who: [{ kind: "allEnemies" }], does: ["BlastCharged"] }, { who: me, does: ["Spend"] }] });
const Oracle = sleeping(trig({ on: "TurnStart" }), [{ kind: "frontEnemy" }], "Hex", { condition: { kind: "turnAtLeast", n: 3 } });
const Berserker = sleeping(intercept({ on: "Strike", striker: "holder" }), me, "Rage");

// The rethink (#1063): seven new units replace the cut duplicates.
const Frostling = sleeping(trig({ on: "Strike", striker: "holder" }), [{ kind: "eventTarget" }], "Chill");
const Priest = sleeping(trig({ on: "BattleStart" }), [{ kind: "allyAhead" }], "Bless");
const Battery = sleeping(trig({ on: "Hurt", unit: "otherAlly" }), me, "Spark", { also: [{ who: [{ kind: "allAllies" }], does: ["GuardCharged"] }, { who: me, does: ["Spend"] }] });
const FrostWitch = sleeping(trig({ on: "TurnStart" }), [{ kind: "allEnemies" }], "Chill", { condition: { kind: "turnIs", n: 2 } });

/* eslint-disable prettier/prettier */
export const ROWS_V2: V2Row[] = [
  v2("Hourglass", "⏳", 2, "counter",     1,  8, "Keeps time: every third turn end, the ally behind it acts again.",            Hourglass, { also: [{ who: [{ kind: "alliesBehind" }], does: ["EncoreCharged"] }, { who: me, does: ["Spend"] }] }),
  v2("Mirror",    "🪞", 2, "copy",        1,  6, "Becomes whatever stands in front of it.",                                 Mirror,    { does: "CopyPower" }),
  v2("Candle",    "🕯️", 3, "replacement", 3,  3, "Burns down instead of dying, one power at a time.",                       Candle,    { also: [{ who: [{ kind: "neighbours" }], does: ["Glow"] }] }),
  v2("Gambler",   "🎲", 1, "swing",       2,  8, "Every strike is a gamble: double or nothing.",                               Gambler,   { does: "AllInAgain" }),
  v2("Nautilus",  "🐚", 1, "scaling",     1, 10, "Grows a shell from every hit it takes.",                                  Nautilus,  { does: "ShellAll" }),
  v2("Magnet",    "🧲", 3, "redirect",    1, 12, "Draws the Poison, Curse and Freeze aimed at its allies onto itself.",    Magnet,    { also: [{ who: me, does: ["Guard"] }] }),
  v2("Raven",     "🐦‍⬛", 4, "memory",      1,  8, "Uses the rule of each ally that falls.",                                  Raven,     { statuses: [{ status: "Memories", stacks: 1 }] }),
  v2("Seed",      "🌱", 2, "form",        0,  4, "Sleeps two turns, then grows and feeds the ally behind it.",              Seed,      { condition: { kind: "turnAtLeast", n: 2 } }),
  // the twelve
  v2("Sniper",      "🎯", 1, "position",    2,  6, "Shoots the enemy at the back every turn.",                              Sniper,      { who: [{ kind: "lastEnemy" }, { kind: "frontEnemy" }] }),
  v2("Medic",       "💉", 1, "position",    1,  8, "Heals whichever ally just got hurt.",                                   Medic,       { does: "Mend + Guard" }),
  v2("Leech",       "🧛", 1, "scaling",     2,  7, "Drinks from every strike it makes.",                                    Leech,       { does: "Glow" }),
  v2("Lone Wolf",   "🐺", 2, "replacement", 3,  7, "Alone, its strikes deal double.",                                       LoneWolf,    { does: "FuryAgain" }),
  v2("Drummer",     "🥁", 2, "counter",     1,  9, "Ally strikes build its Charge; at 3, the line behind it grows stronger.",  Drummer,     { also: [{ who: [{ kind: "allAllies" }], does: ["RallyCharged"] }, { who: me, does: ["Spend"] }] }),
  v2("Bodyguard",   "🕴️", 2, "redirect",    1, 11, "Once a turn, takes a hit meant for an ally.",                           Bodyguard,   { also: [{ who: [{ kind: "eventUnit" }], does: ["Guard"] }] }),
  v2("Chrysalis",   "🐛", 3, "form",        0,  7, "On the third turn, becomes a Butterfly that strikes every enemy.",      Chrysalis,   { does: "EmergeMonarch" }),
  v2("Trickster",   "🃏", 3, "rule",        2,  8, "Trades rules with the front enemy before the first blow.",             Trickster,   { who: [{ kind: "frontEnemy" }, { kind: "lastEnemy" }] }),
  v2("Gravedigger", "⚰️", 4, "scaling",     2, 10, "Hits the front enemy for every fallen ally.",                    Gravedigger, { who: [{ kind: "allEnemies" }] }),
  v2("Conductor",   "🎼", 4, "rule",        1,  9, "On turns 3, 6, 9…, its neighbours act again.",                          Conductor,   { also: [{ who: [{ kind: "neighbours" }], does: ["Rally"] }] }),
  // batch 2 (#1047)
  v2("Thornback",   "🦔", 1, "position",    1,  8, "Pricks whoever hits it.",                                               Thornback,   { does: "Spine" }),
  v2("Coin Thief",  "🪙", 1, "swing",       2,  6, "Robs a random enemy of power every turn, and grows on it.",            CoinThief,   { also: [{ who: me, does: ["Rally", "Guard"] }] }),
  v2("Lantern",     "🏮", 1, "condition",   1,  7, "While untouched, shields the ally ahead of it.",                        Lantern,     { who: [{ kind: "neighbours" }] }),
  v2("Hornet",      "🐝", 1, "position",    2,  5, "Poisons whatever it strikes.",                                          Hornet,      { who: [{ kind: "allEnemies" }] }),
  v2("Prism",       "🔮", 2, "redirect",    1,  9, "Random shots aimed at it bend onto a random enemy.",                    Prism,       { also: [{ who: me, does: ["Guard"] }] }),
  v2("War Horn",    "📯", 2, "condition",   1,  8, "When the line runs thin, it rallies everyone left.",                    Horn,        { also: [{ who: [{ kind: "allAllies" }], does: ["Guard"] }] }),
  v2("Vulture",     "🦅", 2, "scaling",     2,  7, "Feeds on every unit that falls, friend or foe.",                        Vulture,     { also: [{ who: me, does: ["Glow"] }] }),
  v2("Hexer",       "🧿", 3, "rule",        1,  7, "On turn 2, silences the enemy at the back.",               Hexer,       { who: [{ kind: "lastEnemy" }, { kind: "frontEnemy" }] }),
  v2("Fortune Teller", "🪬", 3, "swing",    1,  7, "At the start, an even chance: the whole line strikes harder, or else stands sturdier.", Fortune,  { does: "FortuneAwoken" }),
  v2("Aegis",       "🛡️", 3, "replacement", 1,  9, "Shrugs off the first hit each turn.",                                   Aegis,       { also: [{ who: [{ kind: "allyBehind" }], does: ["Guard"] }] }),
  v2("Gravecaller", "💀", 3, "memory",      1,  7, "Calls the first fallen ally back to the line.",                         Gravecaller, { does: "RaiseAwoken" }),
  v2("Hydra",       "🐉", 4, "form",        2,  8, "Cut it down and two Heads take its place.",                             Hydra,       { does: "HeadsAwoken" }),
  v2("Archmage",    "🧙", 4, "counter",     2,  7, "Builds Charge each turn end; at 3, it strikes every enemy.",           Archmage,    { also: [{ who: [{ kind: "allEnemies" }], does: ["BlastHexCharged"] }, { who: me, does: ["Spend"] }] }),
  v2("Oracle",      "👁️", 4, "condition",   1,  8, "From the third turn on, curses the enemy in front.",                    Oracle,      { who: [{ kind: "frontEnemy" }, { kind: "lastEnemy" }] }),
  v2("Berserker",   "🪓", 4, "scaling",     2,  9, "The more it bleeds, the harder it strikes.",                            Berserker,   { also: [{ who: me, does: ["Sip"] }] }),
  // the rethink (#1063): new units on the shared vocabulary
  v2("Frostling",   "❄️", 1, "position",    2,  6, "Freezes whatever it strikes.",                                          Frostling,   { does: "Chill2" }),
  v2("Priest",      "✨", 2, "condition",   2,  7, "Blesses the ally ahead: it survives its death once.",                   Priest,      { who: [{ kind: "neighbours" }] }),
  v2("Battery",     "🔋", 2, "counter",     1,  9, "Hits on allies build its Charge; at 3, every ally gains Shield.",      Battery,     { also: [{ who: [{ kind: "allAllies" }], does: ["GuardMendCharged"] }, { who: me, does: ["Spend"] }] }),
  v2("Frost Witch", "🧊", 3, "condition",   1,  7, "On turn 2, freezes every enemy.",                                       FrostWitch,  { also: [{ who: [{ kind: "frontEnemy" }], does: ["Chill"] }] }),
];
/* eslint-enable prettier/prettier */
