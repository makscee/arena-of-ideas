// The MVP unit pool (mission #574, slice 7): ~80 units, each with a sleeping
// and an awoken form, drafted from the 2024 heroes (tag v0.10.5) and the v5
// approved units, rewritten into When → Who → Does.
//
// Rules are data: this file is the content corpus, and the numbers are tuned
// by `npm run mvp:meta` (scripts/meta-health.ts), which writes the committed
// meta-health report docs/mvp/meta-health.md.
//
// Chains: a unit's Does emits an event other units listen to (shield gained,
// healed, power gained, status applied, summoned, ally died). Listeners only
// emit events further down this order, so cascades end on their own:
//   roots (start, strike, hurt, death, summon) → shield → heal → power →
//   enemy status (poison, curse, freeze) → damage
// And a listener to a one-unit event (an ally shielded, healed, powered or
// summoned, an enemy poisoned or cursed) acts on one unit, never a group:
// a group reaction to a group event fans out n² and swamps the chain cap.

import { stressRegistry } from "../content/stress.js";
import type { AbilityDef, AbilityRegistry, Effect, EventPattern, Family, Selector, StatusRegistry, UnitDef, When } from "../types.js";
import type { Tier, UnitContent, UnitForm } from "./contract.js";

// ---------- When ----------

const trig = (on: EventPattern): When[] => [{ kind: "trigger", on }];

export const WHEN = {
  start: trig({ on: "BattleStart" }),
  turnStart: trig({ on: "TurnStart" }),
  turnEnd: trig({ on: "TurnEnd" }),
  strike: trig({ on: "Strike", striker: "holder" }),
  hurt: trig({ on: "Hurt", unit: "holder" }),
  allyHurt: trig({ on: "Hurt", unit: "otherAlly" }),
  die: trig({ on: "Death", unit: "holder" }),
  allyDies: trig({ on: "Death", unit: "otherAlly" }),
  enemyDies: trig({ on: "Death", unit: "enemy" }),
  allyShield: trig({ on: "StatusApplied", unit: "ally", status: "Shield" }),
  allyHealed: trig({ on: "Heal", unit: "ally" }),
  allyPower: trig({ on: "StatChanged", unit: "ally", stat: "pwr", sign: "gain" }),
  enemyPoisoned: trig({ on: "StatusApplied", unit: "enemy", status: "Poison" }),
  enemyCursed: trig({ on: "StatusApplied", unit: "enemy", status: "Curse" }),
  allySummoned: trig({ on: "Summon", unit: "otherAlly" }),
} satisfies Record<string, When[]>;
export type WhenKey = keyof typeof WHEN;

// ---------- Who ----------

export const WHO = {
  me: { kind: "holder" },
  it: { kind: "eventUnit" },
  front: { kind: "frontEnemy" },
  enemies: { kind: "allEnemies" },
  allies: { kind: "allAllies" },
  random: { kind: "randomEnemy" },
  fallen: { kind: "lastDeadAlly" },
} satisfies Record<string, Selector>;
export type WhoKey = keyof typeof WHO;

// ---------- Does: a small vocabulary of named Abilities ----------

const c = (value: number) => ({ kind: "const" as const, value });
const body = (name: string, pwr: number, hp: number): UnitDef => ({
  name, base: { pwr, hp }, triggers: WHEN.start, selectors: [WHO.me], abilities: ["Strike"],
});

/** The summoned bodies, by Ability name. */
const SUMMONS: Record<string, UnitDef> = {
  "Call Imp": body("Imp", 1, 2),
  "Call Wolf": body("Wolf", 2, 3),
  "Call Golem": body("Golem", 2, 6),
  "Call Wraith": body("Wraith", 3, 3),
  "Call Treant": body("Treant", 1, 8),
};

/** Ability names read as what they do: "Hit 3", "Poison 2", "Shield 2", …
 * "A + B" is one Ability doing A then B on the same target (a sleeping form
 * Does exactly one Ability); its family is A's. */
function abilityOf(name: string): AbilityDef {
  if (name.includes(" + ")) {
    const parts = name.split(" + ").map(abilityOf);
    return { name, family: parts[0]!.family, effects: parts.flatMap((p) => p.effects) };
  }
  if (name === "Strike") return { name, family: "Strike", effects: [{ kind: "heal", amount: c(0) }] };
  if (name === "Smite") return { name, family: "Strike", effects: [{ kind: "damage", amount: { kind: "stat", stat: "pwr", of: "holder" } }] };
  if (name === "Mend") return { name, family: "Heal", effects: [{ kind: "heal", amount: { kind: "stat", stat: "pwr", of: "holder" } }] };
  if (name === "Silence") return { name, family: "Control", effects: [{ kind: "silence" }] };
  const summon = SUMMONS[name];
  if (summon) return { name, family: "Summon", effects: [{ kind: "summon", unit: summon }] };
  const m = /^(Hit|Poison|Shield|Heal|Strength|Vitality|Curse|Freeze|Bless|Revive) (\d+)$/.exec(name);
  if (!m) throw new Error(`unknown ability name "${name}"`);
  const n = Number(m[2]);
  const status = (s: string, family: Family): AbilityDef => ({ name, family, effects: [{ kind: "applyStatus", status: s, stacks: c(n) }] });
  const effect: Record<string, () => AbilityDef> = {
    Hit: () => ({ name, family: "Strike", effects: [{ kind: "damage", amount: c(n) } as Effect] }),
    Heal: () => ({ name, family: "Heal", effects: [{ kind: "heal", amount: c(n) }] }),
    Revive: () => ({ name, family: "Summon", effects: [{ kind: "resurrect", hp: c(n) }] }),
    Poison: () => status("Poison", "Poison"),
    Shield: () => status("Shield", "Shield"),
    Strength: () => status("Strength", "Arcane"),
    Vitality: () => status("Vitality", "Heal"),
    Curse: () => status("Curse", "Control"),
    Freeze: () => status("Freeze", "Control"),
    Bless: () => status("Blessing", "Heal"),
  };
  return effect[m[1]!]!();
}

// ---------- the units ----------

interface Row {
  name: string;
  emoji: string;
  tier: Tier;
  pwr: number;
  hp: number;
  when: WhenKey;
  who: WhoKey;
  does: string;
  /** The awoken form: a new Who and/or Does (one or two Abilities). */
  awoken: { who?: WhoKey; does?: string[] };
}

const r = (name: string, emoji: string, tier: Tier, pwr: number, hp: number, when: WhenKey, who: WhoKey, does: string, awoken: Row["awoken"]): Row =>
  ({ name, emoji, tier, pwr, hp, when, who, does, awoken });

/* eslint-disable prettier/prettier */
export const ROWS: Row[] = [
  // ---- tier 1: openers and first links ----
  r("Fighter",       "🥊", 1, 2, 5, "strike",     "front",   "Hit 1",       { does: ["Hit 3"] }),
  r("Fodder",        "🥔", 1, 1, 6, "die",        "allies",  "Shield 1",    { does: ["Shield 2"] }),
  r("Squire",        "🗡️", 1, 2, 4, "start",      "me",      "Shield 2",    { does: ["Shield 2", "Strength 1"] }),
  r("Gnat",          "🦟", 1, 1, 4, "start",      "random",  "Hit 2",       { who: "enemies" }),
  r("Spore",         "🍄", 1, 1, 5, "die",        "enemies", "Poison 1",    { does: ["Poison 2"] }),
  r("Rat",           "🐀", 1, 2, 4, "die",        "front",   "Poison 2",    { who: "enemies" }),
  // A summon into a full line is skipped, so Planter also grows: in a full
  // line it is a sturdier body instead of a blank.
  r("Planter",       "🌱", 1, 1, 5, "start",      "me",      "Call Imp + Vitality 2", { does: ["Call Treant", "Vitality 3"] }),
  r("Nurse",         "💉", 1, 1, 5, "allyHurt",   "it",      "Heal 1",      { does: ["Heal 2"] }),
  r("Prepper",       "🎒", 1, 1, 5, "start",      "allies",  "Shield 1",    { does: ["Shield 1", "Vitality 1"] }),
  r("Coach",         "📣", 1, 1, 5, "start",      "allies",  "Strength 1",  { does: ["Strength 1", "Shield 1"] }),
  r("Bat",           "🦇", 1, 2, 4, "strike",     "me",      "Heal 1",      { does: ["Heal 2", "Strength 1"] }),
  r("Taser",         "⚡", 1, 2, 4, "start",      "front",   "Freeze 1",    { does: ["Freeze 1", "Hit 2"] }),
  r("Wire",          "🔌", 1, 1, 5, "allyPower",  "front",   "Hit 1",       { does: ["Hit 2"] }),
  r("Rose",          "🌹", 1, 2, 6, "hurt",       "front",   "Hit 2",       { does: ["Hit 3"] }),
  r("Victim",        "😵", 1, 1, 7, "hurt",       "allies",  "Strength 1",  { does: ["Strength 1", "Heal 1"] }),
  r("Saboteur",      "🧨", 1, 2, 4, "start",      "front",   "Curse 1",     { who: "enemies" }),
  r("Spike",         "🌵", 1, 1, 6, "allyShield", "front",   "Hit 1",       { does: ["Hit 2"] }),
  r("Wither",        "🥀", 1, 2, 4, "enemyDies",  "allies",  "Strength 1",  { does: ["Strength 1", "Vitality 1"] }),
  r("Distractor",    "🤡", 1, 1, 9, "start",      "me",      "Shield 4",    { does: ["Shield 4", "Vitality 2"] }),
  r("Henchman",      "🦹", 1, 2, 5, "allyDies",   "me",      "Strength 2",  { does: ["Strength 2", "Vitality 2"] }),
  r("Sniper",        "🎯", 1, 2, 4, "start",      "random",  "Hit 4",       { does: ["Hit 4", "Hit 3"] }),
  r("Medic",         "⛑️", 1, 1, 6, "turnEnd",    "allies",  "Heal 1",      { does: ["Heal 2"] }),

  // ---- tier 2: links that react to links ----
  r("Guardian",      "🛡️", 2, 2, 6, "allyHurt",   "it",      "Shield 1",    { does: ["Shield 2"] }),
  r("Almsgiver",     "🪙", 2, 2, 7, "allyShield", "it",      "Heal 1",      { does: ["Heal 1", "Strength 1"] }),
  r("Sanctifier",    "✨", 2, 2, 6, "allyHealed", "it",      "Strength 1",  { does: ["Strength 1", "Vitality 1"] }),
  r("Enhancer",      "🔋", 2, 1, 6, "allyShield", "it",      "Strength 1",  { does: ["Strength 2"] }),
  r("Battery",       "🪫", 2, 2, 5, "allyHealed", "me",      "Strength 1",  { does: ["Strength 2"] }),
  r("Injector",      "🧪", 2, 2, 5, "strike",     "front",   "Poison 1",    { does: ["Poison 2"] }),
  r("Venomancer",    "🐍", 2, 2, 7, "strike",     "front",   "Poison 2",    { does: ["Poison 2", "Curse 1"] }),
  r("Plague Rat",    "🐁", 2, 1, 5, "die",        "enemies", "Poison 2",    { does: ["Poison 3"] }),
  r("Duelist",       "🤺", 2, 3, 7, "strike",     "front",   "Hit 2",       { does: ["Hit 3"] }),
  r("Berserker",     "🪓", 2, 3, 6, "hurt",       "me",      "Strength 1",  { does: ["Strength 2"] }),
  r("Emberling",     "🔥", 2, 3, 6, "start",      "enemies", "Hit 1",       { does: ["Hit 2"] }),
  r("Icebinder",     "🧊", 2, 2, 5, "start",      "random",  "Freeze 2",    { who: "enemies" }),
  r("Summoner",      "🔮", 2, 1, 6, "die",        "me",      "Call Wolf",   { does: ["Call Wraith"] }),
  r("Gardener",      "🪴", 2, 1, 7, "allySummoned", "it",  "Vitality 2",  { does: ["Vitality 2", "Strength 1"] }),
  r("Fruiter",       "🍎", 2, 2, 6, "die",        "allies",  "Heal 3",      { does: ["Heal 3", "Strength 1"] }),
  r("Leech",         "🩸", 2, 2, 6, "strike",     "me",      "Mend",        { does: ["Mend", "Strength 1"] }),
  r("Bloodthinner",  "💧", 2, 2, 5, "enemyPoisoned", "it",   "Hit 1",       { does: ["Hit 2"] }),
  r("Hag",           "🧙", 2, 1, 6, "enemyPoisoned", "it",   "Curse 1",     { does: ["Curse 1", "Hit 1"] }),
  r("Trickster",     "🃏", 2, 2, 5, "enemyCursed", "random", "Hit 2",       { does: ["Hit 3"] }),
  r("Custodian",     "🗝️", 2, 2, 9, "allyDies",   "allies",  "Shield 2",    { does: ["Shield 2", "Heal 1"] }),
  r("Silencer",      "🤫", 2, 2, 6, "start",      "front",   "Silence",     { who: "random", does: ["Silence", "Hit 2"] }),
  r("Scavenger",     "🦅", 2, 2, 6, "allyDies",   "me",      "Heal 3",      { does: ["Heal 3", "Strength 2"] }),
  r("Syren",         "🧜", 2, 2, 5, "turnStart",  "random",  "Curse 1",     { who: "enemies" }),
  r("Rot",           "🦠", 2, 1, 6, "turnEnd",    "front",   "Poison 1",    { who: "enemies" }),
  r("Bulwark",       "🧱", 2, 2, 11, "start",     "me",      "Shield 3",    { does: ["Shield 3", "Strength 1"] }),
  r("Stoneskin",     "🪨", 2, 2, 7, "hurt",       "me",      "Shield 1",    { does: ["Shield 2"] }),

  // ---- tier 3: engines ----
  r("Commander",     "🎖️", 3, 2, 8, "start",      "allies",  "Strength 1",  { does: ["Strength 1", "Shield 2"] }),
  r("War Drummer",   "🥁", 3, 2, 8, "turnStart",  "allies",  "Strength 1",  { does: ["Strength 1", "Heal 1"] }),
  r("Physician",     "🩺", 3, 2, 7, "allyHurt",   "it",      "Heal 2",      { does: ["Heal 2", "Shield 1"] }),
  r("Pediatrician",  "🍼", 2, 2, 8, "allySummoned", "it",   "Strength 1",  { does: ["Strength 2"] }),
  r("Crusader",      "⚔️", 3, 3, 9, "allyPower",  "front",   "Hit 1",       { does: ["Hit 2"] }),
  r("Lightning",     "🌩️", 3, 3, 7, "allyPower",  "random",  "Hit 2",       { does: ["Hit 3"] }),
  r("Battle Mage",   "🪄", 3, 3, 7, "allyShield", "random", "Hit 2",       { does: ["Hit 3"] }),
  r("Pathologist",   "🔬", 3, 2, 7, "enemyPoisoned", "me",   "Strength 1",  { does: ["Strength 2"] }),
  r("Plague Doctor", "🦤", 3, 2, 7, "start",      "enemies", "Poison 2",    { does: ["Poison 2", "Curse 1"] }),
  r("Virus",         "🧫", 3, 2, 6, "enemyDies",  "enemies", "Poison 2",    { does: ["Poison 3"] }),
  r("Necromancer",   "💀", 3, 2, 7, "allyDies",   "fallen",  "Revive 2",    { does: ["Revive 3"] }),
  r("Sexton",        "⚰️", 3, 3, 9, "allyDies",   "me",      "Call Wraith", { does: ["Call Golem"] }),
  r("Fungoid",       "🪸", 3, 2, 9, "turnEnd",    "me",      "Call Imp",    { does: ["Call Wolf"] }),
  r("Mesmerist",     "🌀", 3, 2, 7, "start",     "front",   "Freeze 2",    { does: ["Freeze 2", "Curse 1"] }),
  r("Redirector",    "🪞", 3, 2, 8, "hurt",       "random",  "Hit 2",       { does: ["Hit 3"] }),
  r("Keeper",        "🏰", 3, 2, 7, "turnStart",  "allies",  "Shield 1",    { does: ["Shield 2"] }),
  r("Wane",          "🌘", 3, 2, 7, "allyHealed", "front",   "Curse 1",     { does: ["Curse 1", "Hit 1"] }),
  r("Harvest",       "🌾", 3, 2, 8, "enemyDies",  "allies",  "Heal 2",      { does: ["Heal 2", "Vitality 1"] }),
  r("Robber",        "💰", 3, 3, 6, "enemyCursed", "me",     "Strength 1",  { does: ["Strength 2"] }),
  r("Ritualist",     "🕯️", 3, 2, 7, "allyDies",   "enemies", "Hit 2",       { does: ["Hit 3"] }),

  // ---- tier 4: payoffs ----
  r("King",          "👑", 4, 3, 12, "start",     "allies",  "Vitality 2",  { does: ["Vitality 2", "Strength 1"] }),
  r("Priest",        "⛪", 4, 2, 10, "turnEnd",   "allies",  "Heal 2",      { does: ["Heal 2", "Shield 1"] }),
  r("Divinity",      "😇", 4, 2, 8, "allyDies",  "fallen",  "Revive 3",    { does: ["Revive 3", "Bless 2"] }),
  r("Phoenix",       "🐦", 4, 4, 9, "start",     "me",      "Bless 8",     { does: ["Bless 8", "Strength 2"] }),
  r("Lilith",        "🧛", 4, 4, 8, "enemyDies",  "me",      "Strength 2",  { does: ["Strength 2", "Mend"] }),
  r("Famin",         "☠️", 4, 3, 9, "turnEnd",   "random",  "Poison 1",    { who: "enemies" }),
  r("Mentalist",     "🧠", 4, 3, 8, "allyDies",  "enemies", "Freeze 1",    { does: ["Freeze 1", "Curse 1"] }),
  r("Equalizer",     "⚖️", 4, 3, 9, "allyPower", "front",   "Curse 1",     { does: ["Curse 1", "Hit 1"] }),
  r("Director",      "🎬", 4, 3, 9, "allyDies",  "allies",  "Strength 1",  { does: ["Strength 1", "Shield 1"] }),
  r("Doctor",        "🥼", 4, 2, 10, "hurt",     "allies",  "Heal 1",      { does: ["Heal 2"] }),
  r("Ruin",          "🌋", 4, 4, 8, "start",     "enemies", "Hit 2",       { does: ["Hit 3"] }),
  r("Fertilizer",    "🌻", 4, 2, 10, "allySummoned", "it",  "Strength 2",  { does: ["Strength 2", "Shield 2"] }),
  r("Morbid",        "🦴", 4, 3, 9, "allyDies",  "me",      "Call Golem",  { does: ["Call Golem", "Strength 2"] }),
];
/* eslint-enable prettier/prettier */

function slug(name: string): string {
  return name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
}

function unitOf(row: Row): UnitContent {
  const sleeping: UnitForm = { when: WHEN[row.when], who: [WHO[row.who]], does: [row.does] };
  const awoken: UnitForm = {
    when: WHEN[row.when],
    who: [WHO[row.awoken.who ?? row.who]],
    does: row.awoken.does ?? [row.does],
  };
  return {
    id: slug(row.name),
    name: row.name,
    emoji: row.emoji,
    tier: row.tier,
    base: { pwr: row.pwr, hp: row.hp },
    forms: { sleeping, awoken },
  };
}

export interface MvpPool {
  units: UnitContent[];
  abilities: AbilityRegistry;
  statuses: StatusRegistry;
}

/** The pool: every unit plus exactly the Abilities and statuses they use. */
export function mvpPool(rows: Row[] = ROWS): MvpPool {
  const units = rows.map(unitOf);
  const names = new Set<string>(["Strike"]);
  for (const u of units) for (const f of [u.forms.sleeping, u.forms.awoken]) for (const d of f.does) names.add(d);
  const abilities: AbilityRegistry = {};
  for (const n of [...names].sort()) abilities[n] = abilityOf(n);
  return { units, abilities, statuses: { ...stressRegistry } };
}
