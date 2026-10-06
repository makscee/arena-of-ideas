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
import { summonId } from "../describe.js";
import type { AbilityDef, AbilityRegistry, Effect, EventPattern, Family, Selector, StatusRegistry, UnitDef, When } from "../types.js";
import type { SummonContent, Tier, UnitContent, UnitForm } from "./contract.js";

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
  allyShield: trig({ on: "StatusApplied", unit: "otherAlly", status: "Shield" }),
  allyHealed: trig({ on: "Heal", unit: "otherAlly" }),
  allyPower: trig({ on: "StatChanged", unit: "otherAlly", stat: "pwr", sign: "gain" }),
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

/** The summoned bodies, by Ability name. Their emoji is in SUMMON_EMOJI. */
const SUMMONS: Record<string, UnitDef> = {
  "Call Imp": body("Imp", 1, 2),
  "Call Wolf": body("Wolf", 2, 3),
  "Call Golem": body("Golem", 2, 6),
  "Call Wraith": body("Wraith", 3, 3),
  "Call Treant": body("Treant", 1, 8),
  // Awoken Summoner's body (R3-8): it has a job of its own, so the Awoken
  // form does something new and not just a bigger body.
  "Call Warg": { name: "Warg", base: { pwr: 2, hp: 4 }, triggers: WHEN.strike, selectors: [WHO.front], abilities: ["Poison 1"] },
  // Awoken Phoenix hatches a Chick; Awoken Sexton's and Fungoid's bodies (R3-9), with jobs of their own.
  "Call Chick": body("Chick", 1, 3),
  "Call Ghoul": { name: "Ghoul", base: { pwr: 3, hp: 3 }, triggers: WHEN.strike, selectors: [WHO.front], abilities: ["Curse 1"] },
  "Call Puffball": { name: "Puffball", base: { pwr: 1, hp: 3 }, triggers: WHEN.die, selectors: [WHO.enemies], abilities: ["Poison 1"] },
};

/** Each summoned body's emoji, by its name (R3-5). */
const SUMMON_EMOJI: Record<string, string> = { Imp: "👺", Wolf: "🐺", Golem: "🗿", Wraith: "👻", Treant: "🌳", Warg: "🐕", Ghoul: "🧟", Puffball: "💨", Chick: "🐣" };


/** A summoned body as content: its emoji, and its form unless it only strikes. */
export function summonContentOf(def: UnitDef, emoji: string): SummonContent {
  const does = def.abilities ?? [];
  const form: UnitForm | null = does.every((x) => x === "Strike")
    ? null
    : { when: def.triggers ?? [], who: def.selectors ?? [], does, ...(def.condition ? { condition: def.condition } : {}) };
  return { id: summonId(def.name), name: def.name, emoji, base: { ...def.base }, form };
}

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
  r("Fighter",       "🥊", 1, 2, 5, "strike",     "front",   "Hit 1",       { who: "enemies" }),
  r("Fodder",        "🥔", 1, 1, 6, "die",        "allies",  "Shield 1",    { does: ["Shield 1", "Heal 2"] }),
  r("Squire",        "🗡️", 1, 2, 4, "start",      "me",      "Strength 2",  { does: ["Strength 2", "Shield 2"] }),
  r("Gnat",          "🦟", 1, 1, 4, "turnEnd",    "random",  "Hit 1",       { who: "enemies" }),
  r("Spore",         "🍄", 1, 1, 5, "die",        "enemies", "Curse 1",     { does: ["Curse 1", "Poison 1"] }),
  r("Rat",           "🐀", 1, 2, 4, "die",        "front",   "Poison 2",    { who: "enemies" }),
  // A summon into a full line is skipped, so Planter also grows: in a full
  // line it is a sturdier body instead of a blank.
  r("Planter",       "🌱", 1, 1, 5, "start",      "me",      "Call Imp + Vitality 2", { does: ["Call Treant + Vitality 2", "Shield 2"] }),
  r("Nurse",         "💉", 1, 1, 5, "allyHurt",   "it",      "Heal 1",      { does: ["Heal 1", "Shield 1"] }),
  r("Prepper",       "🎒", 1, 1, 5, "start",      "allies",  "Shield 1",    { does: ["Shield 1", "Bless 1"] }),
  r("Coach",         "📣", 1, 1, 5, "start",      "allies",  "Strength 1",  { does: ["Strength 1", "Shield 1"] }),
  r("Bat",           "🦇", 1, 2, 4, "strike",     "random",  "Hit 1",       { does: ["Hit 1", "Curse 1"] }),
  r("Taser",         "⚡", 1, 2, 4, "start",      "front",   "Freeze 1",    { does: ["Freeze 1", "Hit 2"] }),
  r("Wire",          "🔌", 1, 1, 5, "allyPower",  "front",   "Hit 1",       { does: ["Hit 1", "Silence"] }),
  r("Rose",          "🌹", 1, 2, 6, "hurt",       "front",   "Hit 2",       { does: ["Hit 2", "Curse 1"] }),
  r("Victim",        "😵", 1, 1, 7, "hurt",       "allies",  "Strength 1",  { does: ["Strength 1", "Heal 1"] }),
  r("Saboteur",      "🧨", 1, 2, 4, "start",      "front",   "Curse 1",     { who: "enemies" }),
  r("Spike",         "🌵", 1, 1, 6, "allyShield", "front",   "Hit 1",       { does: ["Hit 1", "Poison 1"] }),
  r("Wither",        "🥀", 1, 2, 4, "enemyDies",  "allies",  "Strength 1",  { does: ["Strength 1", "Shield 1"] }),
  r("Distractor",    "🤡", 1, 1, 9, "hurt",       "me",      "Vitality 1",  { does: ["Vitality 1", "Shield 2"] }),
  r("Henchman",      "🦹", 1, 2, 5, "allyDies",   "me",      "Strength 2",  { does: ["Strength 2", "Shield 2"] }),
  r("Sniper",        "🎯", 1, 2, 4, "start",      "random",  "Hit 4",       { does: ["Hit 4", "Poison 2"] }),
  r("Medic",         "⛑️", 1, 1, 6, "turnEnd",    "allies",  "Heal 1",      { does: ["Heal 1", "Shield 1"] }),

  // ---- tier 2: links that react to links ----
  r("Guardian",      "🛡️", 2, 2, 6, "allyHurt",   "it",      "Shield 1",    { who: "me", does: ["Shield 2"] }),
  r("Almsgiver",     "🪙", 2, 2, 7, "allyShield", "it",      "Heal 1",      { who: "me", does: ["Heal 2", "Strength 1"] }),
  r("Sanctifier",    "✨", 2, 2, 6, "allyHealed", "it",      "Strength 1",  { who: "front", does: ["Smite"] }),
  r("Enhancer",      "🔋", 2, 1, 6, "allyShield", "it",      "Strength 1",  { does: ["Strength 1", "Heal 1"] }),
  r("Battery",       "🪫", 2, 2, 5, "allyHealed", "me",      "Strength 1",  { who: "random", does: ["Hit 2"] }),
  r("Injector",      "🧪", 2, 2, 5, "hurt",       "front",   "Poison 1",    { who: "enemies" }),
  r("Venomancer",    "🐍", 2, 2, 7, "strike",     "front",   "Poison 2",    { does: ["Poison 2", "Curse 1"] }),
  r("Plague Rat",    "🐁", 2, 1, 5, "die",        "enemies", "Poison 2",    { does: ["Poison 2", "Freeze 1"] }),
  // Duelist opens with a lunge; Fighter is the one that hits on every strike.
  r("Duelist",       "🤺", 2, 3, 7, "start",      "front",   "Smite",       { does: ["Smite", "Curse 1"] }),
  r("Berserker",     "🪓", 2, 3, 6, "hurt",       "me",      "Strength 1",  { does: ["Strength 1", "Heal 1"] }),
  r("Emberling",     "🔥", 2, 3, 6, "die",        "enemies", "Hit 2",       { does: ["Hit 2", "Freeze 1"] }),
  r("Icebinder",     "🧊", 2, 2, 5, "start",      "random",  "Freeze 2",    { who: "enemies" }),
  r("Summoner",      "🔮", 2, 1, 6, "die",        "me",      "Call Wolf",   { does: ["Call Warg"] }),
  r("Gardener",      "🪴", 2, 1, 7, "allySummoned", "it",  "Vitality 2",  { who: "me", does: ["Vitality 2", "Shield 2"] }),
  r("Fruiter",       "🍎", 2, 2, 6, "die",        "allies",  "Heal 3",      { does: ["Heal 2", "Bless 1"] }),
  r("Leech",         "🩸", 2, 2, 6, "strike",     "me",      "Mend",        { does: ["Mend", "Shield 1"] }),
  r("Bloodthinner",  "💧", 2, 2, 5, "enemyPoisoned", "it",   "Hit 1",       { who: "front", does: ["Hit 2"] }),
  r("Hag",           "🧙", 2, 1, 6, "enemyPoisoned", "it",   "Curse 1",     { does: ["Curse 1", "Hit 1"] }),
  r("Trickster",     "🃏", 2, 2, 5, "enemyCursed", "random", "Hit 2",       { who: "it", does: ["Smite"] }),
  r("Custodian",     "🗝️", 2, 2, 9, "allyDies",   "allies",  "Shield 2",    { does: ["Shield 2", "Heal 1"] }),
  r("Silencer",      "🤫", 2, 2, 6, "start",      "front",   "Silence",     { who: "random", does: ["Silence", "Hit 2"] }),
  r("Scavenger",     "🦅", 2, 2, 6, "allyDies",   "me",      "Heal 3",      { does: ["Heal 3", "Bless 1"] }),
  r("Syren",         "🧜", 2, 2, 5, "turnStart",  "random",  "Curse 1",     { who: "enemies" }),
  r("Rot",           "🦠", 2, 1, 6, "turnEnd",    "front",   "Poison 1",    { who: "enemies" }),
  r("Bulwark",       "🧱", 2, 2, 11, "start",     "me",      "Shield 3",    { does: ["Shield 3", "Bless 1"] }),
  r("Stoneskin",     "🪨", 2, 2, 7, "hurt",       "me",      "Shield 1",    { who: "allies" }),

  // ---- tier 3: engines ----
  r("Commander",     "🎖️", 3, 2, 8, "strike",     "allies",  "Strength 1",  { does: ["Strength 1", "Heal 1"] }),
  r("War Drummer",   "🥁", 3, 1, 7, "turnStart",  "allies",  "Strength 1",  { does: ["Strength 1", "Heal 1"] }),
  // Physician treats the cause, not the wound: once a turn it saps the front
  // enemy. On "ally hit" it cursed ~18 times a battle (R3-9).
  r("Physician",     "🩺", 3, 2, 8, "turnEnd",    "front",   "Curse 1",     { does: ["Curse 1", "Poison 1"] }),
  r("Pediatrician",  "🍼", 2, 2, 8, "allySummoned", "it",   "Strength 1",  { does: ["Strength 1", "Bless 1"] }),
  r("Crusader",      "⚔️", 3, 3, 9, "turnStart",  "front",   "Hit 2",       { does: ["Hit 2", "Curse 1"] }),
  r("Lightning",     "🌩️", 3, 3, 7, "allyPower",  "random",  "Hit 2",       { does: ["Hit 1", "Curse 1"] }),
  r("Battle Mage",   "🪄", 3, 3, 7, "allyShield", "random", "Hit 2",       { does: ["Hit 2", "Poison 1"] }),
  r("Pathologist",   "🔬", 3, 2, 7, "enemyPoisoned", "me",   "Strength 1",  { does: ["Strength 1", "Heal 1"] }),
  r("Plague Doctor", "🦤", 3, 2, 7, "start",      "enemies", "Poison 2",    { does: ["Poison 2", "Curse 1"] }),
  r("Virus",         "🧫", 3, 2, 6, "enemyDies",  "enemies", "Poison 2",    { does: ["Poison 2", "Freeze 1"] }),
  // Awoken, the dead return as undead glass cannons: 1 HP, +3 PWR (1a).
  r("Necromancer",   "💀", 3, 2, 7, "allyDies",   "fallen",  "Revive 2",    { does: ["Revive 1 + Strength 3"] }),
  r("Sexton",        "⚰️", 3, 3, 9, "allyDies",   "me",      "Call Wraith", { does: ["Call Ghoul"] }),
  r("Fungoid",       "🪸", 3, 2, 9, "turnEnd",    "me",      "Call Imp",    { does: ["Call Puffball"] }),
  r("Mesmerist",     "🌀", 3, 2, 7, "start",      "front",   "Freeze 1 + Curse 1", { who: "enemies", does: ["Freeze 1 + Curse 1"] }),
  r("Redirector",    "🪞", 3, 2, 8, "hurt",       "random",  "Hit 2",       { does: ["Hit 2", "Freeze 1"] }),
  r("Keeper",        "🏰", 3, 1, 6, "turnStart",  "allies",  "Shield 1",    { does: ["Shield 1", "Heal 1"] }),
  r("Wane",          "🌘", 3, 2, 7, "allyHealed", "front",   "Curse 1",     { does: ["Curse 1", "Hit 1"] }),
  r("Harvest",       "🌾", 3, 2, 8, "enemyDies",  "allies",  "Heal 2",      { does: ["Heal 2", "Shield 1"] }),
  r("Robber",        "💰", 3, 3, 6, "enemyCursed", "me",     "Vitality 1",  { who: "front", does: ["Hit 1"] }),
  r("Ritualist",     "🕯️", 3, 2, 7, "allyDies",   "enemies", "Hit 2",       { does: ["Hit 2", "Poison 1"] }),

  // ---- tier 4: payoffs ----
  r("King",          "👑", 4, 3, 12, "start",     "allies",  "Vitality 2",  { does: ["Vitality 2", "Bless 1"] }),
  // Priest avenges the fallen: Silence strips an enemy's statuses (Shield, Blessing)
  // and its ability. Its old team blessing was Divinity's job (R3-9); King's
  // Awoken form keeps a one-off team blessing at battle start.
  r("Priest",        "⛪", 4, 2, 10, "allyDies",  "random",  "Silence",     { does: ["Silence", "Curse 1"] }),
  r("Divinity",      "😇", 4, 2, 8, "allyDies",  "allies",  "Bless 1",     { does: ["Bless 1", "Shield 2"] }),
  r("Phoenix",       "🐦", 4, 4, 9, "start",     "me",      "Bless 8",     { does: ["Bless 8", "Call Chick"] }),
  r("Lilith",        "🧛", 4, 4, 8, "enemyDies",  "me",      "Strength 2",  { does: ["Strength 2", "Mend"] }),
  r("Famin",         "☠️", 4, 3, 9, "turnEnd",   "random",  "Poison 1",    { does: ["Poison 1 + Curse 1"] }),
  r("Mentalist",     "🧠", 4, 3, 8, "allyDies",  "enemies", "Freeze 1",    { does: ["Freeze 1", "Curse 1"] }),
  r("Equalizer",     "⚖️", 4, 3, 9, "allyPower", "front",   "Curse 1",     { does: ["Curse 1", "Hit 1"] }),
  r("Director",      "🎬", 4, 3, 9, "allyDies",  "allies",  "Strength 1",  { does: ["Strength 1", "Heal 2"] }),
  r("Doctor",        "🥼", 4, 2, 10, "hurt",     "allies",  "Heal 1",      { does: ["Heal 1", "Shield 1"] }),
  r("Ruin",          "🌋", 4, 4, 8, "start",     "enemies", "Hit 2",       { does: ["Hit 2", "Curse 1"] }),
  r("Fertilizer",    "🌻", 4, 2, 10, "allySummoned", "it",  "Strength 2 + Shield 2", { does: ["Strength 2 + Shield 2", "Bless 1"] }),
  r("Morbid",        "🦴", 4, 3, 9, "allyDies",  "enemies", "Curse 1",     { does: ["Curse 1", "Poison 1"] }),
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

// ---------- shape: one hero per signature (round 3, docs/round3/units.md 1b) ----------

/** The kinds of effect a Does list has, numbers ignored: "Hit 3" → Hit,
 * "Call Imp" → Call, "Freeze 1 + Curse 1" → Curse, Freeze. */
export function effectKinds(does: string[]): string[] {
  return [...new Set(does.flatMap((d) => d.split(" + ")).map((part) => part.split(" ")[0]!))].sort();
}

/** The WHEN key a form listens to ("allyDies", …). */
export function whenKeyOf(form: UnitForm): string {
  const json = JSON.stringify(form.when);
  return (Object.keys(WHEN) as WhenKey[]).find((k) => JSON.stringify(WHEN[k]) === json) ?? json;
}

/** A form's shape: When · Who kind · its set of effect kinds. Two units with
 * the same shape in the same form are the same hero, whatever the numbers. */
export function sig(form: UnitForm): string {
  return [whenKeyOf(form), form.who.map((w) => w.kind).join("+"), shapeKinds(form.does).join("+")].join(" · ");
}

/** The effect kinds that make a hero's job: the heal family (Heal, Mend) is
 * one kind, so is the damage family (Hit, Smite), and Strength and Vitality riders don't count next to another
 * kind ("Call Golem + Strength 1" is a summoner). A form that only grows
 * stats keeps them, since that is its job. */
export function shapeKinds(does: string[]): string[] {
  const kinds = [...new Set(effectKinds(does).map((k) => FAMILY[k] ?? k))].sort();
  const job = kinds.filter((k) => !RIDERS.includes(k));
  return job.length ? job : kinds;
}
const FAMILY: Record<string, string> = { Mend: "Heal", Smite: "Hit" };

/** The effect kinds the bodies a Does list summons act with ("Call Warg" →
 * Poison); a body that only strikes adds none. */
function calledJobs(does: string[]): string[] {
  const parts = does.flatMap((d) => d.split(" + "));
  return effectKinds(parts.flatMap((p) => (SUMMONS[p]?.abilities ?? []).filter((a) => a !== "Strike")));
}
const RIDERS = ["Strength", "Vitality"];

/** What an Awoken form does that its sleeping form doesn't (round 3, R3):
 * a new Who kind, or an effect kind the sleeping form lacks. Families count
 * as one kind (Hit = Smite, Heal = Mend), and a Strength or Vitality rider
 * never counts as the new part. Null when Awoken only has bigger numbers. */
export function awokenNewPart(sleeping: UnitForm, awoken: UnitForm): string | null {
  const who = (f: UnitForm) => f.who.map((w) => w.kind).join("+");
  if (who(awoken) !== who(sleeping)) return `Who ${who(sleeping)} → ${who(awoken)}`;
  const kinds = (f: UnitForm) => new Set([...effectKinds(f.does), ...calledJobs(f.does)].map((k) => FAMILY[k] ?? k));
  const had = kinds(sleeping);
  const added = [...kinds(awoken)].filter((k) => !had.has(k) && !RIDERS.includes(k));
  return added.length ? `adds ${added.join(", ")}` : null;
}

/** The link event each listening When reacts to. The other Whens are roots
 * (battle start, turns, strike, hurt, death): damage only takes HP away, so a
 * cascade through hurt or death ends on its own. */
export const LISTENS: Partial<Record<WhenKey, string>> = {
  allyShield: "Shield",
  allyHealed: "Heal",
  allyPower: "Power",
  enemyPoisoned: "Poison",
  enemyCursed: "Curse",
  allySummoned: "Summon",
};
export const ROOT_WHENS: WhenKey[] = ["start", "turnStart", "turnEnd", "strike", "hurt", "allyHurt", "die", "allyDies", "enemyDies"];

/** The link event each effect kind emits; the kinds no When listens to emit none.
 * Bless emits a Heal: a Blessing's death save heals (stress.ts Blessing). */
export const EMITS: Record<string, string | null> = {
  Shield: "Shield", Heal: "Heal", Mend: "Heal", Strength: "Power", Poison: "Poison", Curse: "Curse", Call: "Summon", Revive: "Summon",
  Bless: "Heal",
  Hit: null, Smite: null, Vitality: null, Freeze: null, Silence: null,
};

/** A form's edges in the listen → emit graph: the link it reacts to, to each
 * link it emits. A cycle in this graph is a loop only the chain cap stops. */
export function linkEdges(form: UnitForm): [string, string][] {
  const from = LISTENS[whenKeyOf(form) as WhenKey];
  if (!from) return [];
  return effectKinds(form.does).flatMap((k) => (EMITS[k] ? [[from, EMITS[k]!] as [string, string]] : []));
}

export interface MvpPool {
  units: UnitContent[];
  abilities: AbilityRegistry;
  statuses: StatusRegistry;
  summons: SummonContent[];
}

/** The pool: every unit plus exactly the Abilities and statuses they use. */
export function mvpPool(rows: Row[] = ROWS): MvpPool {
  const units = rows.map(unitOf);
  const names = new Set<string>(["Strike"]);
  for (const u of units) for (const f of [u.forms.sleeping, u.forms.awoken]) for (const d of f.does) names.add(d);
  // …and what the summoned bodies act with.
  for (const n of [...names]) for (const part of n.split(" + ")) for (const a of SUMMONS[part]?.abilities ?? []) names.add(a);
  const abilities: AbilityRegistry = {};
  for (const n of [...names].sort()) abilities[n] = abilityOf(n);
  // Every body a summon effect in the pool makes, once each. Names are unique
  // among summons, so a Summon event's name finds exactly one body.
  const bodies = new Map<string, UnitDef>();
  for (const ab of Object.values(abilities))
    for (const e of ab.effects) {
      if (e.kind !== "summon") continue;
      const seen = bodies.get(e.unit.name);
      if (seen && JSON.stringify(seen) !== JSON.stringify(e.unit)) throw new Error(`two summoned bodies are named "${e.unit.name}"`);
      bodies.set(e.unit.name, e.unit);
    }
  const summons = [...bodies.values()].map((d) => {
    const emoji = SUMMON_EMOJI[d.name];
    if (!emoji) throw new Error(`summon "${d.name}" has no emoji`);
    return summonContentOf(d, emoji);
  });
  return { units, abilities, statuses: { ...stressRegistry }, summons };
}
