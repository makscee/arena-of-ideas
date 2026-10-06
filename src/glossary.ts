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
import type { Condition, Effect, EventPattern, Selector, StatusRegistry } from "./types.js";

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
    tip: "Blocks damage: each point blocked uses up 1 Shield.",
    more: "It also blocks Poison and Fatigue damage. A fully blocked hit still counts as being hit.",
  },
  Vitality: { label: "Vitality", icon: "heart-plus", tone: "vital", tip: "+1 HP for each Vitality, for the rest of the battle." },
  Strength: { label: "Strength", icon: "biceps", tone: "str", tip: "+1 PWR for each Strength, for the rest of the battle." },
  Curse: { label: "Curse", icon: "cursed-star", tone: "curse", tip: "-1 PWR for each Curse, for the rest of the battle (PWR stops at 0)." },
  Poison: { label: "Poison", icon: "drop", tone: "poison", tip: "At the end of each turn: takes damage equal to its Poison, then Poison drops by 1." },
  Freeze: { label: "Freeze", icon: "snowflake-2", tone: "freeze", tip: "Skips its next strike; each skipped strike uses up 1 Freeze." },
  Blessing: {
    label: "Blessing", icon: "angel-wings", tone: "bless",
    tip: "The next time it would die, it lives on with HP equal to its Blessing instead. Then the Blessing is gone.",
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
    label: "Is hit", icon: "broken-heart", tone: "when",
    tip: "When damage comes at it: a strike, an ability, Poison or Fatigue. It counts even if Shield blocks all of it.",
  },
  "trigger:Heal": { label: "Is healed", icon: "health-normal", tone: "when", tip: "When it gets HP back. A unit at full HP can't be healed, so this doesn't fire." },
  "trigger:Death": { label: "Dies", icon: "death-skull", tone: "when", tip: "When it dies. Its own death ability still fires as it leaves the line." },
  "trigger:Summon": { label: "Is summoned", icon: "magic-portal", tone: "when", tip: "When a new unit joins the line: summoned, or revived." },
  "trigger:StatusApplied": { label: "Status lands", tone: "when", tip: "When that status is put on it." },
  "trigger:StatusRemoved": { label: "Status leaves", tone: "when", tip: "When that status is used up or removed." },
  "trigger:StatChanged": { label: "Gains PWR", icon: "upgrade", tone: "when", tip: "When its PWR goes up, for example from Strength." },
  "term:would": {
    label: "would", tone: "when",
    tip: "\"When X would …\" happens just before X, and can change or stop it (how Shield, Freeze and Blessing work).",
  },

  // Conditions
  "condition:holderHpAtMost": { label: "Low HP", icon: "hearts", tone: "hp", tip: "Fires only while this unit's HP is at or below the number." },

  // Targets (Who)
  "target:holder": { label: "This unit", icon: "person", tone: "ally", tip: "The unit that has this ability." },
  "target:eventUnit": {
    label: "That unit", icon: "pointing", tone: "plain",
    tip: "The unit the trigger was about: the ally who got hit, the enemy who got poisoned.",
  },
  "target:frontEnemy": { label: "Front enemy", icon: "targeted", tone: "enemy", tip: "The first enemy in line, the one fighting right now." },
  "target:randomEnemy": { label: "Random enemy", icon: "perspective-dice-six-faces-random", tone: "enemy", tip: "One living enemy, picked at random." },
  "target:allEnemies": { label: "Every enemy", icon: "minions", tone: "enemy", tip: "All living enemies." },
  "target:allAllies": { label: "Every ally", icon: "three-friends", tone: "ally", tip: "All living allies, this unit included." },
  "target:lastDeadAlly": { label: "Fallen ally", icon: "tombstone", tone: "ally", tip: "The ally who died most recently and is still dead." },

  // Effects (Does)
  "effect:damage": { label: "Damage", icon: "spiky-explosion", tone: "dmg", tip: "Takes away HP. Shield blocks it first." },
  "effect:heal": { label: "Heal", icon: "health-normal", tone: "heal", tip: "Gives back lost HP, never above the unit's max." },
  "effect:applyStatus": { label: "Apply status", tone: "plain", tip: "Puts stacks of a status on the target." },
  "effect:consumeStacks": { label: "Consume stacks", tone: "plain", tip: "Removes stacks of a status from the unit that has it." },
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
    tip: "A chain of reactions ran 64 steps and was cut off, so a battle can't loop forever.",
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
