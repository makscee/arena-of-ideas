// The glossary: one source for every term a player meets in unit text, status
// badges, battle captions and the Codex (docs/round2/icons.md). Pure data,
// shared by the client, the Codex and the server. Display-only, like
// describe.ts: no rule lives here, but every tip must say what the kernel does.
//
// Each term has a label, an icon (an SVG in mobile/icons/, credited in
// mobile/icons/CREDITS.txt), a tone (the colour class the client paints it
// with) and a one-line rule. The tables are Records over the kernel's unions,
// like parts.ts: a new trigger, selector or effect kind can't ship without an
// entry.

import { describeStatus } from "./describe.js";
import { MVP_RULES } from "./mvp/contract.js";
import type { Condition, Effect, EventPattern, Selector, StatusRegistry, UnitFilter } from "./types.js";

/** Every term id. `status:` takes a registry name ("Shield", "Poison", …). */
export type TermId =
  | `stat:${"pwr" | "hp"}`
  | `status:${string}`
  | `trigger:${EventPattern["on"]}`
  | `condition:${Condition["kind"]}`
  | `target:${Selector["kind"]}`
  | `effect:${Effect["kind"]}`
  | `state:${"sleeping" | "awoken" | "fused"}`
  | `battle:${"fatigue" | "chainCapped"}`
  | `term:${"stacks" | "would"}`;

/** The terms with a fixed entry (every TermId but the open `status:` set). */
export type FixedTermId = Exclude<TermId, `status:${string}`>;

/** The colour a term is painted with: a CSS class `tone-<tone>`, one value
 * each for dark and light (docs/round2/mockups/icons.html). */
export type Tone =
  | "pwr" | "hp" | "shield" | "vital" | "str" | "curse" | "poison" | "freeze" | "bless" | "silence"
  | "dmg" | "heal" | "summon" | "when" | "ally" | "enemy" | "gold" | "plain";

/** The icon ids: one SVG each in mobile/icons/ (`icon(id)` draws it). */
export type IconId =
  | "angel-wings" | "biceps" | "breaking-chain" | "broadsword" | "broken-heart" | "crossed-swords"
  | "cursed-star" | "death-skull" | "drop" | "flying-flag" | "health-normal" | "heart-plus" | "hearts"
  | "hourglass" | "linked-rings" | "magic-portal" | "minions" | "moon" | "perspective-dice-six-faces-random"
  | "person" | "pointing" | "raise-zombie" | "shield" | "silence" | "sleepy" | "snowflake-2"
  | "spiky-explosion" | "sunrise" | "targeted" | "third-eye" | "three-friends" | "tombstone" | "upgrade";

export const ICON_IDS: readonly IconId[] = [
  "angel-wings", "biceps", "breaking-chain", "broadsword", "broken-heart", "crossed-swords",
  "cursed-star", "death-skull", "drop", "flying-flag", "health-normal", "heart-plus", "hearts",
  "hourglass", "linked-rings", "magic-portal", "minions", "moon", "perspective-dice-six-faces-random",
  "person", "pointing", "raise-zombie", "shield", "silence", "sleepy", "snowflake-2",
  "spiky-explosion", "sunrise", "targeted", "third-eye", "three-friends", "tombstone", "upgrade",
];

export type TermGroup = "stat" | "status" | "trigger" | "condition" | "target" | "effect" | "state" | "battle" | "term";

export interface TermDef {
  label: string;
  /** No icon: the term reads as words only (Stacks: the number is the badge). */
  icon?: IconId;
  tone: Tone;
  /** The one-line rule a tooltip shows. */
  tip: string;
  /** A longer line for the Codex, where useful. */
  more?: string;
}

/** The shipped statuses (src/content/stress.ts). A status not listed here
 * (player-made content) still resolves through termDef, from its definition. */
export const STATUS_TERMS: Record<string, TermDef> = {
  Shield: {
    label: "Shield", icon: "shield", tone: "shield",
    tip: "Blocks damage of any kind: each point blocked uses up 1 stack.",
  },
  Vitality: { label: "Vitality", icon: "heart-plus", tone: "vital", tip: "+1 HP per stack, for the rest of the battle." },
  Strength: { label: "Strength", icon: "biceps", tone: "str", tip: "+1 PWR per stack, for the rest of the battle." },
  Curse: { label: "Curse", icon: "cursed-star", tone: "curse", tip: "-1 PWR per stack, for the rest of the battle (PWR stops at 0)." },
  Poison: { label: "Poison", icon: "drop", tone: "poison", tip: "At the end of every turn, takes damage equal to its stacks, then loses 1 stack." },
  Freeze: { label: "Freeze", icon: "snowflake-2", tone: "freeze", tip: "Skips its next strike; each skipped strike uses up 1 stack." },
  Blessing: {
    label: "Blessing", icon: "angel-wings", tone: "bless",
    tip: "The next time it would die, it lives on with HP equal to its stacks instead, and loses them all.",
  },
};

export const GLOSSARY: Record<FixedTermId, TermDef> = {
  // Stats
  "stat:pwr": { label: "PWR", icon: "broadsword", tone: "pwr", tip: "Power: the damage this unit deals with each strike." },
  "stat:hp": { label: "HP", icon: "hearts", tone: "hp", tip: "Health. When it reaches 0 the unit dies." },

  // Triggers (When). A trigger shares its event's icon on purpose: in a
  // chain, the cause and the reaction show the same picture.
  "trigger:BattleStart": { label: "Battle start", icon: "flying-flag", tone: "when", tip: "Once, when the battle begins, before anyone strikes." },
  "trigger:TurnStart": { label: "Turn start", icon: "sunrise", tone: "when", tip: "At the start of every turn." },
  "trigger:TurnEnd": { label: "Turn end", icon: "moon", tone: "when", tip: "At the end of every turn, after the front units have struck." },
  "trigger:Strike": { label: "Strikes", icon: "crossed-swords", tone: "when", tip: "When it makes its normal attack. Each turn the two front units strike each other." },
  "trigger:Hurt": {
    label: "Hit", icon: "broken-heart", tone: "when",
    tip: "When any damage comes at it, from anything. It counts even if all of it is blocked.",
  },
  "trigger:Heal": { label: "Healed", icon: "health-normal", tone: "when", tip: "When it gets HP back. A unit at full HP can't be healed, so this doesn't fire." },
  "trigger:Death": { label: "Dies", icon: "death-skull", tone: "when", tip: "When it dies. Its own death ability still fires as it leaves the line." },
  "trigger:Summon": { label: "Summoned", icon: "magic-portal", tone: "when", tip: "When it joins the line mid-battle, new or brought back." },
  "trigger:StatusApplied": { label: "Gets status", tone: "when", tip: "When that status is put on it." },
  "trigger:StatusRemoved": { label: "Loses status", tone: "when", tip: "When that status is used up or removed." },
  "trigger:StatChanged": { label: "Gains PWR", icon: "upgrade", tone: "when", tip: "When its PWR goes up, from anything." },
  "term:would": {
    label: "would", tone: "when",
    tip: "\"Would …\" happens just before the thing, and can change or stop it.",
  },

  // Conditions
  "condition:holderHpAtMost": { label: "Low HP", icon: "hearts", tone: "hp", tip: "Fires only while its own HP is at or below the number." },

  // Targets (Who)
  "target:holder": { label: "Self", icon: "person", tone: "ally", tip: "The unit that has this ability." },
  "target:eventUnit": { label: "It", icon: "pointing", tone: "plain", tip: "The unit the trigger was about." },
  "target:frontEnemy": { label: "Front enemy", icon: "targeted", tone: "enemy", tip: "The first enemy in line, the one fighting right now." },
  "target:randomEnemy": { label: "Random enemy", icon: "perspective-dice-six-faces-random", tone: "enemy", tip: "One living enemy, picked at random." },
  "target:allEnemies": { label: "All enemies", icon: "minions", tone: "enemy", tip: "Every living enemy." },
  "target:allAllies": { label: "All allies", icon: "three-friends", tone: "ally", tip: "Every living ally, self included: the unit with this ability counts too." },
  "target:lastDeadAlly": { label: "Fallen ally", icon: "tombstone", tone: "ally", tip: "The ally who died most recently and is still dead." },

  // Effects (Does)
  "effect:damage": { label: "Damage", icon: "spiky-explosion", tone: "dmg", tip: "Takes away that much HP." },
  "effect:heal": { label: "Heal", icon: "health-normal", tone: "heal", tip: "Gives back lost HP, never above the unit's max." },
  "effect:applyStatus": { label: "Apply status", tone: "plain", tip: "Puts stacks of a status on the target." },
  "effect:consumeStacks": { label: "Spend", tone: "plain", tip: "Removes stacks of a status from the unit that has it." },
  "effect:summon": {
    label: "Summon", icon: "magic-portal", tone: "summon",
    tip: "Adds a new unit at the back of the line, if the line has room (5 max). The numbers are its PWR / HP.",
  },
  "effect:resurrect": { label: "Revive", icon: "raise-zombie", tone: "bless", tip: "Brings a fallen ally back at the back of the line with that much HP, if there's room." },
  "effect:silence": { label: "Silence", icon: "silence", tone: "silence", tip: "Removes all its statuses and turns off its abilities for the rest of the battle." },
  "effect:cancel": { label: "Cancel", tone: "plain", tip: "Stops the thing that was about to happen." },
  "effect:absorbHurt": { label: "Absorb", icon: "shield", tone: "shield", tip: "Blocks damage up to its stacks, using up what it blocks." },
  "effect:preventDeathHeal": { label: "Cheat death", icon: "angel-wings", tone: "bless", tip: "Stops the death and sets the unit's HP instead." },

  // Unit states
  "state:sleeping": { label: "Sleeping", icon: "sleepy", tone: "plain", tip: "A unit's first form. Owning 3 copies awakens it." },
  "state:awoken": { label: "Awoken", icon: "third-eye", tone: "gold", tip: "The stronger form, unlocked by the 3rd copy." },
  "state:fused": { label: "Fused", icon: "linked-rings", tone: "summon", tip: "Two awoken units made into one: When of the 1st, Who of the 2nd, Does of both." },

  // Battle
  "battle:fatigue": {
    label: "Fatigue", icon: "hourglass", tone: "dmg",
    tip: "From turn 10, every unit takes damage at each turn's end: 1, then 2, 3 … so battles always end.",
  },
  "battle:chainCapped": {
    label: "Chain stopped", icon: "breaking-chain", tone: "plain",
    tip: `A chain of reactions ran ${MVP_RULES.chainStepCap} steps and was cut off, so a battle can't loop forever.`,
  },

  // Words
  "term:stacks": { label: "Stacks", tone: "plain", tip: "The number next to a status. More stacks, stronger effect; some statuses use stacks up." },
};

/** The group a term belongs to: the part before the colon. */
export const termGroup = (id: TermId): TermGroup => id.slice(0, id.indexOf(":")) as TermGroup;

/** A term's entry. A status the glossary doesn't list (player-made content)
 * gets its label from its name and its tip from its own definition; a status
 * the registry doesn't know either has no entry (undefined). */
export function termDef(id: TermId, statuses: StatusRegistry = {}): TermDef | undefined {
  if (id.startsWith("status:")) {
    const name = id.slice("status:".length);
    const known = STATUS_TERMS[name];
    if (known) return known;
    const def = statuses[name];
    return def ? { label: name, tone: "plain", tip: describeStatus(def) } : undefined;
  }
  return GLOSSARY[id as FixedTermId];
}

/** The icon a term shows. A status lands / apply status run shows the status's
 * own icon, so pass the status name for those. */
export function termIcon(id: TermId, status?: string): IconId | undefined {
  if ((id === "trigger:StatusApplied" || id === "trigger:StatusRemoved" || id === "effect:applyStatus") && status)
    return STATUS_TERMS[status]?.icon;
  return termDef(id)?.icon;
}

/** What a unit trigger watches for, said of someone else (its tip is said of
 * the holder: "When it dies …"). */
const SCOPED_EVENT: Partial<Record<EventPattern["on"], string>> = {
  Strike: "makes its normal attack.",
  Hurt: "is hit by any damage, from anything. It counts even if all of it is blocked.",
  Heal: "gets HP back. A unit at full HP can't be healed, so this doesn't fire.",
  Death: "dies.",
  Summon: "joins the line mid-battle, new or brought back.",
  StatusApplied: "gets that status.",
  StatusRemoved: "loses that status: used up or removed.",
  StatChanged: "gains PWR, from anything.",
};

const SCOPE_WHO: Record<Exclude<UnitFilter, "holder">, string> = {
  ally: "When any ally, self included,",
  otherAlly: "When another ally (not self)",
  enemy: "When an enemy",
  any: "When any unit, on either side,",
};

/** A trigger's rule as its sentence scopes it: "After an enemy dies" shows
 * "When an enemy dies.", not the holder's own-death rule. The holder's scope
 * (and a term that isn't a unit trigger) keeps the plain tip. */
export function scopedTip(id: TermId, scope?: UnitFilter): string | undefined {
  const def = termDef(id);
  if (!def || !scope || scope === "holder" || !id.startsWith("trigger:")) return def?.tip;
  const event = SCOPED_EVENT[id.slice("trigger:".length) as EventPattern["on"]];
  return event ? `${SCOPE_WHO[scope]} ${event}` : def.tip;
}

/** Who a scoped trigger is about, as the card's text opens it ("Ally dies"). */
const SCOPE_SUBJECT: Record<Exclude<UnitFilter, "holder">, string> = {
  ally: "Ally",
  otherAlly: "Ally",
  enemy: "Enemy",
  any: "Any unit",
};

/** A trigger's label as its sentence scopes it, in the card's words: "Ally
 * dies", "Enemy hit", "Ally gains PWR". The holder's scope (and a term that
 * isn't a unit trigger) keeps the plain label. */
export function scopedLabel(id: TermId, scope?: UnitFilter): string | undefined {
  const def = termDef(id);
  if (!def || !scope || scope === "holder" || !id.startsWith("trigger:") || !SCOPED_EVENT[id.slice("trigger:".length) as EventPattern["on"]]) return def?.label;
  return `${SCOPE_SUBJECT[scope]} ${def.label.charAt(0).toLowerCase()}${def.label.slice(1)}`;
}
