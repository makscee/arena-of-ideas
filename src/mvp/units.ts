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
  "Call Imp": body("Imp", 1, 3),
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

export interface Row {
  name: string;
  emoji: string;
  tier: Tier;
  pwr: number;
  hp: number;
  when: WhenKey;
  who: WhoKey;
  does: string;
  /** One sentence on what the unit is about (ARCHETYPE below). */
  archetype: string;
  /** The awoken form, built on top of the sleeping one (round 4, note 8):
   * Awoken only adds to or enhances, never drops (`awokenKeeps` checks it). */
  awoken: AwokenRow;
}

/** What Awoken adds to the sleeping form. Each part is optional; with none
 * but `more`, it is a numbers-only Awoken (allowed since round 4). */
export interface AwokenRow {
  /** A wider Who for the sleeping Does (front → all enemies, me → all allies). */
  who?: WhoKey;
  /** The sleeping Does, enhanced: the same parts with numbers no lower, or
   * a bigger summoned body ("Hit 1" → "Hit 2", "Call Imp" → "Call Treant"). */
  more?: string;
  /** Does added before the sleeping one, on the same Who (Mesmerist silences
   * first, so the Silence doesn't strip its own Freeze and Curse). */
  before?: string[];
  /** Does added after the sleeping one, on the same Who. */
  add?: string[];
  /** An "and" clause: one more Does with a Who of its own ("…, and Silence
   * the front enemy"). */
  also?: { who: WhoKey; does: string };
}

/** Each unit's archetype (round 4, note 3; docs/round4/units.md): one
 * sentence on what the unit is about, unique across the pool. */
/* eslint-disable prettier/prettier */
const ARCHETYPE: Record<string, string> = {
  Fighter:         "A plain brawler whose every swing adds a jab.",
  Fodder:          "A sacrificial body that shields the team when it falls.",
  Squire:          "A starter who powers itself up before the fight.",
  Gnat:            "A pest that pings a random enemy every turn.",
  Spore:           "Its own death weakens the whole enemy line.",
  Planter:         "Opens by planting a small body, and grows if there's no room.",
  Nurse:           "A medic who patches each ally right after they're hit.",
  Prepper:         "Hands the team a little armour before the fight.",
  Coach:           "A pep talk: team-wide Strength at the start.",
  Bat:             "A flier whose strikes land on a random enemy.",
  Taser:           "Opens by freezing the front enemy.",
  Wire:            "A wire that numbs the front enemy whenever an ally powers up.",
  Rose:            "Thorns: hits back at the front when struck.",
  Victim:          "A taunting tank whose pain powers the team.",
  Saboteur:        "Opens by weakening the front enemy.",
  Distractor:      "A big decoy that swells each time it's hit.",
  Henchman:        "Gets stronger each time a comrade falls (revenge).",
  Sniper:          "One big opening shot at a random enemy.",
  Medic:           "Steady team heal at every turn end.",
  Guardian:        "Puts a Shield on each ally the moment it's hit.",
  Almsgiver:       "Turns Shields into heals for the shielded ally.",
  Sanctifier:      "Each heal also strengthens the healed ally.",
  Enhancer:        "Each Shield also strengthens the shielded ally.",
  Battery:         "Charges itself on every ally heal.",
  Injector:        "Poisons the front enemy when hit.",
  Venomancer:      "Strikes that stack Poison on the front enemy.",
  "Plague Rat":    "Its own death poisons the whole enemy line.",
  Duelist:         "Lunges at the front enemy with full power at the start.",
  Berserker:       "Gets stronger with every wound.",
  Emberling:       "Explodes on death, hurting every enemy.",
  Icebinder:       "Frostbite: every cursed enemy also freezes.",
  Summoner:        "Leaves a wolf behind when it dies.",
  Gardener:        "Toughens every newly summoned ally.",
  Fruiter:         "Drops a team-wide heal when it dies.",
  Leech:           "Heals itself by its damage on every strike.",
  Bloodthinner:    "Makes each poisoned enemy bleed at once.",
  Hag:             "Each Poison it sees also curses the target.",
  Trickster:       "Each Curse sets off damage.",
  Custodian:       "Shields the team every time an ally falls.",
  Silencer:        "Opens by shutting off the front enemy's ability.",
  Scavenger:       "Heals itself off fallen allies.",
  Syren:           "A lullaby that freezes a random enemy each turn.",
  Bulwark:         "A wall that starts heavily shielded.",
  Stoneskin:       "Hardens with every hit it takes.",
  Commander:       "A captain whose every strike shields the line.",
  "War Drummer":   "A team Strength ramp every turn.",
  Physician:       "Saps the front enemy every turn.",
  Crusader:        "A heavy hit at the front every turn.",
  Lightning:       "Ally power-ups call down random strikes.",
  "Battle Mage":   "Ally Shields fire random bolts.",
  Pathologist:     "A numbing toxin: every poisoned enemy also freezes.",
  "Plague Doctor": "Poisons the whole enemy line at the start.",
  Virus:           "Each kill spreads Poison to all enemies.",
  Necromancer:     "Raises fallen allies.",
  Sexton:          "Digs a fresh body for every ally that falls.",
  Fungoid:         "Sprouts an Imp every turn.",
  Mesmerist:       "Opens by locking down the front enemy.",
  Keeper:          "Shields the team every turn.",
  Wane:            "Each ally heal saps the front enemy.",
  Harvest:         "Each kill feeds the team a heal.",
  Robber:          "Steals HP whenever an enemy is cursed.",
  Ritualist:       "Each ally death becomes damage to all enemies.",
  King:            "Grants max HP to the whole team at the start.",
  Priest:          "Avenges the fallen by silencing an enemy.",
  Divinity:        "Each ally death blesses the team against dying.",
  Phoenix:         "Comes back from death, many times over.",
  Lilith:          "Grows on every kill.",
  Famin:           "Slowly poisons random enemies.",
  Mentalist:       "Each ally death freezes all enemies.",
  Equalizer:       "Ally power-ups sap the front enemy.",
  Ruin:            "Opens with damage to the whole enemy line.",
  Fertilizer:      "Arms and armours every new summon.",
  Morbid:          "Each ally death weakens all enemies.",
};
/* eslint-enable prettier/prettier */

function archetypeOf(name: string): string {
  const a = ARCHETYPE[name];
  if (!a) throw new Error(`unit "${name}" has no archetype`);
  return a;
}

const r = (name: string, emoji: string, tier: Tier, pwr: number, hp: number, when: WhenKey, who: WhoKey, does: string, awoken: Row["awoken"]): Row =>
  ({ name, emoji, tier, pwr, hp, when, who, does, archetype: archetypeOf(name), awoken });

/* eslint-disable prettier/prettier */
export const ROWS: Row[] = [
  // ---- tier 1: openers and first links ----
  r("Fighter",       "🥊", 1, 2, 5, "strike",     "front",   "Hit 1",       { who: "enemies" }),
  r("Fodder",        "🥔", 1, 1, 6, "die",        "allies",  "Shield 1",    { add: ["Heal 2"] }),
  r("Squire",        "🗡️", 1, 2, 4, "start",      "me",      "Strength 2",  { add: ["Shield 2"] }),
  r("Gnat",          "🦟", 1, 1, 4, "turnEnd",    "random",  "Hit 1",       { who: "enemies" }),
  r("Spore",         "🍄", 1, 1, 5, "die",        "enemies", "Curse 1",     { also: { who: "front", does: "Freeze 1" } }),
  // A summon into a full line is skipped, so Planter also grows: in a full
  // line it is a sturdier body instead of a blank.
  r("Planter",       "🌱", 1, 1, 5, "start",      "me",      "Call Imp + Vitality 2", { more: "Call Treant + Vitality 2", add: ["Shield 2"] }),
  r("Nurse",         "💉", 1, 1, 5, "allyHurt",   "it",      "Heal 1",      { add: ["Shield 1"] }),
  r("Prepper",       "🎒", 1, 1, 5, "start",      "allies",  "Shield 1",    { add: ["Bless 1"] }),
  r("Coach",         "📣", 1, 1, 5, "start",      "allies",  "Strength 1",  { add: ["Shield 1"] }),
  r("Bat",           "🦇", 1, 2, 4, "strike",     "random",  "Hit 1",       { add: ["Curse 1"] }),
  r("Taser",         "⚡", 1, 2, 4, "start",      "front",   "Freeze 1",    { add: ["Hit 2"] }),
  r("Wire",          "🔌", 1, 1, 5, "allyPower",  "front",   "Freeze 1",    { add: ["Hit 1"] }),
  r("Rose",          "🌹", 1, 2, 6, "hurt",       "front",   "Hit 2",       { add: ["Poison 1"] }),
  r("Victim",        "😵", 1, 1, 7, "hurt",       "allies",  "Strength 1",  { add: ["Heal 1"] }),
  r("Saboteur",      "🧨", 1, 2, 4, "start",      "front",   "Curse 1",     { who: "enemies" }),
  r("Distractor",    "🤡", 1, 1, 9, "hurt",       "me",      "Vitality 1",  { also: { who: "front", does: "Curse 1" } }),
  r("Henchman",      "🦹", 1, 2, 5, "allyDies",   "me",      "Strength 2",  { also: { who: "front", does: "Silence" } }),
  r("Sniper",        "🎯", 1, 2, 4, "start",      "random",  "Hit 4",       { add: ["Poison 2"] }),
  r("Medic",         "⛑️", 1, 1, 6, "turnEnd",    "allies",  "Heal 1",      { add: ["Shield 1"] }),

  // ---- tier 2: links that react to links ----
  r("Guardian",      "🛡️", 2, 2, 6, "allyHurt",   "it",      "Shield 1",    { also: { who: "front", does: "Poison 1" } }),
  r("Almsgiver",     "🪙", 2, 2, 7, "allyShield", "it",      "Heal 1",      { also: { who: "random", does: "Poison 1" } }),
  r("Sanctifier",    "✨", 2, 2, 6, "allyHealed", "it",      "Strength 1",  { also: { who: "front", does: "Smite" } }),
  r("Enhancer",      "🔋", 2, 1, 6, "allyShield", "it",      "Strength 1",  { add: ["Heal 1"] }),
  r("Battery",       "🪫", 2, 2, 5, "allyHealed", "me",      "Strength 1",  { also: { who: "random", does: "Freeze 1" } }),
  r("Injector",      "🧪", 2, 2, 5, "hurt",       "front",   "Poison 1",    { add: ["Freeze 1"] }),
  r("Venomancer",    "🐍", 2, 2, 7, "strike",     "front",   "Poison 2",    { add: ["Curse 1"] }),
  r("Plague Rat",    "🐁", 2, 1, 5, "die",        "enemies", "Poison 2",    { add: ["Freeze 1"] }),
  // Duelist opens with a lunge; Fighter is the one that hits on every strike.
  r("Duelist",       "🤺", 2, 3, 7, "start",      "front",   "Smite",       { add: ["Curse 1"] }),
  r("Berserker",     "🪓", 2, 3, 6, "hurt",       "me",      "Strength 1",  { add: ["Heal 1"] }),
  r("Emberling",     "🔥", 2, 3, 6, "die",        "enemies", "Hit 2",       { add: ["Freeze 1"] }),
  r("Icebinder",     "🧊", 2, 2, 5, "enemyCursed", "it",     "Freeze 1",    { add: ["Hit 1"] }),
  r("Summoner",      "🔮", 2, 1, 6, "die",        "me",      "Call Wolf",   { more: "Call Warg" }),
  r("Gardener",      "🪴", 2, 1, 7, "allySummoned", "it",  "Vitality 2",  { also: { who: "random", does: "Poison 2" } }),
  r("Fruiter",       "🍎", 2, 2, 6, "die",        "allies",  "Heal 3",      { add: ["Bless 1"] }),
  r("Leech",         "🩸", 2, 2, 6, "strike",     "me",      "Mend",        { add: ["Shield 1"] }),
  r("Bloodthinner",  "💧", 2, 2, 5, "enemyPoisoned", "it",   "Hit 1",       { also: { who: "random", does: "Curse 1" } }),
  r("Hag",           "🧙", 2, 1, 6, "enemyPoisoned", "it",   "Curse 1",     { add: ["Hit 1"] }),
  r("Trickster",     "🃏", 2, 2, 5, "enemyCursed", "random", "Hit 2",       { also: { who: "it", does: "Hit 1" } }),
  r("Custodian",     "🗝️", 2, 2, 9, "allyDies",   "allies",  "Shield 2",    { add: ["Heal 1"] }),
  r("Silencer",      "🤫", 2, 2, 6, "start",      "front",   "Silence",     { add: ["Hit 2"] }),
  r("Scavenger",     "🦅", 2, 2, 6, "allyDies",   "me",      "Heal 3",      { add: ["Bless 1"] }),
  r("Syren",         "🧜", 2, 2, 5, "turnStart",  "random",  "Freeze 1",    { add: ["Curse 1"] }),
  r("Bulwark",       "🧱", 2, 2, 11, "start",     "me",      "Shield 3",    { add: ["Bless 1"] }),
  r("Stoneskin",     "🪨", 2, 2, 7, "hurt",       "me",      "Shield 1",    { who: "allies" }),

  // ---- tier 3: engines ----
  r("Commander",     "🎖️", 3, 2, 8, "strike",     "allies",  "Shield 1",    { add: ["Strength 1"] }),
  r("War Drummer",   "🥁", 3, 1, 7, "turnStart",  "allies",  "Strength 1",  { add: ["Heal 1"] }),
  // Physician treats the cause, not the wound: once a turn it saps the front
  // enemy. On "ally hit" it cursed ~18 times a battle (R3-9).
  r("Physician",     "🩺", 3, 2, 8, "turnEnd",    "front",   "Curse 1",     { add: ["Poison 1"] }),
  r("Crusader",      "⚔️", 3, 3, 9, "turnStart",  "front",   "Hit 2",       { add: ["Curse 1"] }),
  r("Lightning",     "🌩️", 3, 3, 7, "allyPower",  "random",  "Hit 2",       { add: ["Curse 1"] }),
  r("Battle Mage",   "🪄", 3, 3, 7, "allyShield", "random", "Hit 2",       { add: ["Poison 1"] }),
  r("Pathologist",   "🔬", 3, 2, 7, "enemyPoisoned", "it",   "Freeze 1",    { add: ["Curse 1"] }),
  r("Plague Doctor", "🦤", 3, 2, 7, "start",      "enemies", "Poison 2",    { add: ["Curse 1"] }),
  r("Virus",         "🧫", 3, 2, 6, "enemyDies",  "enemies", "Poison 2",    { add: ["Curse 1"] }),
  // Awoken, each death raises two: the fallen ally, and an Imp at the front
  // (R3-26, Maks's note 1: Awoken does something new, not a stat rider).
  r("Necromancer",   "💀", 3, 1, 5, "allyDies",   "fallen",  "Revive 1",    { more: "Revive 1 + Call Imp" }),
  r("Sexton",        "⚰️", 3, 3, 9, "allyDies",   "me",      "Call Wraith", { more: "Call Ghoul" }),
  r("Fungoid",       "🪸", 3, 1, 6, "turnEnd",    "me",      "Call Imp",    { more: "Call Puffball" }),
  r("Mesmerist",     "🌀", 3, 2, 7, "start",      "front",   "Freeze 1 + Curse 1", { before: ["Silence"] }),
  r("Keeper",        "🏰", 3, 1, 6, "turnStart",  "allies",  "Shield 1",    { add: ["Heal 1"] }),
  r("Wane",          "🌘", 3, 2, 7, "allyHealed", "front",   "Curse 1",     { add: ["Hit 1"] }),
  r("Harvest",       "🌾", 3, 2, 8, "enemyDies",  "allies",  "Heal 2",      { add: ["Shield 1"] }),
  r("Robber",        "💰", 3, 3, 6, "enemyCursed", "me",     "Vitality 1",  { also: { who: "front", does: "Hit 1" } }),
  r("Ritualist",     "🕯️", 3, 2, 7, "allyDies",   "enemies", "Hit 2",       { add: ["Poison 1"] }),

  // ---- tier 4: payoffs ----
  r("King",          "👑", 4, 3, 12, "start",     "allies",  "Vitality 2",  { add: ["Bless 1"] }),
  // Priest avenges the fallen: Silence strips an enemy's statuses (Shield, Blessing)
  // and its ability. Its old team blessing was Divinity's job (R3-9); King's
  // Awoken form keeps a one-off team blessing at battle start.
  r("Priest",        "⛪", 4, 1, 6,  "allyDies",  "front",   "Silence",     { add: ["Curse 1"] }),
  r("Divinity",      "😇", 4, 2, 8, "allyDies",  "allies",  "Bless 1",     { add: ["Shield 2"] }),
  r("Phoenix",       "🐦", 4, 4, 9, "start",     "me",      "Bless 8",     { add: ["Call Chick"] }),
  r("Lilith",        "🧛", 4, 4, 8, "enemyDies",  "me",      "Strength 2",  { add: ["Mend"] }),
  r("Famin",         "☠️", 4, 3, 9, "turnEnd",   "random",  "Poison 1",    { more: "Poison 1 + Curse 1" }),
  r("Mentalist",     "🧠", 4, 3, 8, "allyDies",  "enemies", "Freeze 1",    { add: ["Curse 1"] }),
  r("Equalizer",     "⚖️", 4, 3, 9, "allyPower", "front",   "Curse 1",     { add: ["Hit 1"] }),
  r("Ruin",          "🌋", 4, 4, 8, "start",     "enemies", "Hit 2",       { add: ["Curse 1"] }),
  r("Fertilizer",    "🌻", 4, 2, 10, "allySummoned", "it",  "Strength 2 + Shield 2", { add: ["Bless 1"] }),
  r("Morbid",        "🦴", 4, 2, 8, "allyDies",  "enemies", "Curse 1",     { add: ["Poison 1"] }),
];
/* eslint-enable prettier/prettier */

/** The most words an archetype may have (round 4, note 3: "one sentence"). */
export const ARCHETYPE_MAX_WORDS = 15;

/** What's wrong with the units' archetypes: each is one sentence (a capital
 * first, one "." at the end, no other sentence end), at most
 * ARCHETYPE_MAX_WORDS words, and no two are the same, case ignored. */
export function archetypeProblems(units: Pick<UnitContent, "name" | "archetype">[]): string[] {
  const out: string[] = [];
  const seen = new Map<string, string>();
  for (const u of units) {
    const a = (u.archetype ?? "").trim();
    if (!a) {
      out.push(`${u.name}: no archetype`);
      continue;
    }
    if (!/^[A-Z]/.test(a)) out.push(`${u.name}: archetype starts with a capital`);
    if (!a.endsWith(".")) out.push(`${u.name}: archetype ends with "."`);
    if (/[.!?]\s/.test(a)) out.push(`${u.name}: archetype is more than one sentence`);
    const words = a.split(/\s+/).length;
    if (words > ARCHETYPE_MAX_WORDS) out.push(`${u.name}: archetype has ${words} words (max ${ARCHETYPE_MAX_WORDS})`);
    const key = a.toLowerCase();
    const other = seen.get(key);
    if (other) out.push(`${u.name}: same archetype as ${other}`);
    else seen.set(key, u.name);
  }
  return out;
}

function slug(name: string): string {
  return name.toLowerCase().replace(/[^a-z0-9]+/g, "-").replace(/^-|-$/g, "");
}

function unitOf(row: Row): UnitContent {
  const sleeping: UnitForm = { when: WHEN[row.when], who: [WHO[row.who]], does: [row.does] };
  const a = row.awoken;
  const awoken: UnitForm = {
    when: WHEN[row.when],
    who: [WHO[a.who ?? row.who]],
    does: [...(a.before ?? []), a.more ?? row.does, ...(a.add ?? [])],
    ...(a.also ? { also: [{ who: [WHO[a.also.who]], does: [a.also.does] }] } : {}),
  };
  return {
    id: slug(row.name),
    name: row.name,
    emoji: row.emoji,
    archetype: row.archetype,
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

/** Every Does of a form: its own, then its "and" clauses'. */
export function allDoes(form: UnitForm): string[] {
  return [...form.does, ...(form.also ?? []).flatMap((c) => c.does)];
}

/** A form's shape: When · Who kind · its set of effect kinds. Two units with
 * the same shape in the same form are the same hero, whatever the numbers.
 * An "and" clause adds its own "& Who kind · effect kinds". */
export function sig(form: UnitForm): string {
  const part = (who: UnitForm["who"], does: string[]) => [who.map((w) => w.kind).join("+"), shapeKinds(does).join("+")];
  const also = (form.also ?? []).map((c) => `& ${part(c.who, c.does).join(" · ")}`);
  return [whenKeyOf(form), ...part(form.who, form.does), ...also].join(" · ");
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
  const kinds = (f: UnitForm) => new Set([...effectKinds(allDoes(f)), ...calledJobs(allDoes(f))].map((k) => FAMILY[k] ?? k));
  const had = kinds(sleeping);
  const added = [...kinds(awoken)].filter((k) => !had.has(k) && !RIDERS.includes(k));
  return added.length ? `adds ${added.join(", ")}` : null;
}

/** Who kinds, narrowest to widest within a side: a wider Who still reaches
 * the unit the narrower one did (front or random enemy → all enemies; me or
 * that ally → all allies). Anything else is a different Who, not a wider one. */
const WIDER: Record<string, string[]> = {
  frontEnemy: ["allEnemies"],
  randomEnemy: ["allEnemies"],
  holder: ["allAllies"],
  eventUnit: [],
  lastDeadAlly: [],
};

/** A Does part's word and number: "Hit 2" → ["Hit", 2], "Silence" → ["Silence", 0]. */
function partOf(part: string): [string, number] {
  const m = /^(.*?) (\d+)$/.exec(part);
  return m ? [m[1]!, Number(m[2])] : [part, 0];
}

/** A summoned body's PWR and HP, by its Ability ("Call Imp"). */
const bodyOf = (part: string) => SUMMONS[part]?.base;

/** Why an Awoken form drops or weakens something its sleeping form does
 * (round 4, note 8: Awoken only adds to or enhances), or null when it keeps
 * it all. Every sleeping part must still be there, on the same or a wider Who,
 * with a number no lower; a summon may become a body at least as big. The
 * parts may sit in the main Does or in an "and" clause. */
export function awokenKeeps(sleeping: UnitForm, awoken: UnitForm): string | null {
  const who = (f: { who: UnitForm["who"] }) => f.who.map((w) => w.kind).join("+");
  const sw = who(sleeping);
  const clauses = [{ who: awoken.who, does: awoken.does }, ...(awoken.also ?? [])];
  const reaches = (c: { who: UnitForm["who"] }) => who(c) === sw || (WIDER[sw] ?? []).includes(who(c));
  for (const part of sleeping.does.flatMap((d) => d.split(" + "))) {
    const [word, n] = partOf(part);
    const kept = clauses.some((c) => reaches(c) && c.does.flatMap((d) => d.split(" + ")).some((p) => {
      if (word === "Call" || part.startsWith("Call ")) {
        const a = bodyOf(part), b = bodyOf(p);
        return !!a && !!b && b.pwr >= a.pwr && b.hp >= a.hp;
      }
      const [w2, n2] = partOf(p);
      return w2 === word && n2 >= n;
    }));
    if (!kept) return `drops or weakens "${part}" on ${sw}`;
  }
  return null;
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
  return effectKinds(allDoes(form)).flatMap((k) => (EMITS[k] ? [[from, EMITS[k]!] as [string, string]] : []));
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
  for (const u of units) for (const f of [u.forms.sleeping, u.forms.awoken]) for (const d of allDoes(f)) names.add(d);
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
