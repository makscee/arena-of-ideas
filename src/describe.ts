// Derived descriptions — part/ability/status in, one human-readable sentence out.
// Everything is computed from the DSL data itself (when + condition + selector +
// effect), never hand-written per unit: player-created content describes itself
// exactly like the shipped stress set does. Display-only; no rule lives here —
// the wording mirrors SPEC semantics but the kernel never reads it back.

import type {
  Ability,
  Amount,
  Condition,
  Effect,
  EventPattern,
  Selector,
  StatusDef,
  UnitFilter,
  When,
} from "./types.js";

/** How the describing context names the holder: "this unit" on a unit's own
 * ability, "the holder" on a status's (the unit the status is attached to). */
export interface DescribeOpts {
  holder?: string;
}

const HOLDER_DEFAULT = "this unit";

const plural = (n: number, word: string): string => `${n} ${word}${n === 1 ? "" : "s"}`;

const capitalize = (s: string): string => (s.length > 0 ? s[0]!.toUpperCase() + s.slice(1) : s);

/** A unit filter as a noun phrase, relative to the holder. */
function filterPhrase(f: UnitFilter | undefined, holder: string): string {
  switch (f) {
    case "holder":
      return holder;
    case "ally":
      return "an ally";
    case "enemy":
      return "an enemy";
    default:
      return "any unit"; // "any" and an omitted filter mean the same thing
  }
}

/** An amount as a noun phrase: "3", "this unit's pwr", "its stacks". */
export function describeAmount(a: Amount, opts: DescribeOpts = {}): string {
  const holder = opts.holder ?? HOLDER_DEFAULT;
  switch (a.kind) {
    case "const":
      return String(a.value);
    case "stat":
      return `${holder}'s ${a.stat}`;
    case "level":
      return `${holder}'s level`;
    case "stacks":
      return "its stacks";
  }
}

/** A const amount reads inline ("deal 3 damage"); anything derived reads as
 * "equal to …" so the sentence stays grammatical for every Amount kind. */
const amountClause = (a: Amount, opts: DescribeOpts): string =>
  a.kind === "const" ? String(a.value) : `an amount equal to ${describeAmount(a, opts)}`;

/** A when as a clause: triggers read "after X", interceptors "when X would …". */
export function describeWhen(w: When, opts: DescribeOpts = {}): string {
  const holder = opts.holder ?? HOLDER_DEFAULT;
  const p: EventPattern = w.on;
  const intercept = w.kind === "interceptor";
  switch (p.on) {
    case "BattleStart":
      return "when the battle begins";
    case "TurnStart":
      return "at the start of each turn";
    case "TurnEnd":
      return "at the end of each turn";
    case "Strike": {
      const who = filterPhrase(p.striker, holder);
      return intercept ? `when ${who} would strike` : `after ${who} strikes`;
    }
    case "Hurt": {
      const who = filterPhrase(p.unit, holder);
      return intercept ? `when ${who} would be hurt` : `after ${who} is hurt`;
    }
    case "Heal": {
      const who = filterPhrase(p.unit, holder);
      return intercept ? `when ${who} would be healed` : `after ${who} is healed`;
    }
    case "Death": {
      const who = filterPhrase(p.unit, holder);
      return intercept ? `when ${who} would die` : `after ${who} dies`;
    }
    case "Summon": {
      const who = filterPhrase(p.unit, holder);
      return intercept ? `when ${who} would be summoned` : `after ${who} is summoned`;
    }
    case "StatusApplied": {
      const who = filterPhrase(p.unit, holder);
      const status = p.status ?? "a status";
      return intercept ? `when ${status} would land on ${who}` : `after ${status} lands on ${who}`;
    }
    case "StatusRemoved": {
      const who = filterPhrase(p.unit, holder);
      const status = p.status ?? "a status";
      return intercept ? `when ${status} would leave ${who}` : `after ${status} leaves ${who}`;
    }
  }
}

/** A condition as a "while …" clause. */
export function describeCondition(c: Condition, opts: DescribeOpts = {}): string {
  const holder = opts.holder ?? HOLDER_DEFAULT;
  switch (c.kind) {
    case "holderHpAtMost":
      return `while ${holder} is at ${c.value} hp or less`;
  }
}

/** A selector as the noun phrase of what it picks. */
export function describeSelector(s: Selector, opts: DescribeOpts = {}): string {
  const holder = opts.holder ?? HOLDER_DEFAULT;
  switch (s.kind) {
    case "holder":
      return holder;
    case "eventUnit":
      return "the event's unit";
    case "frontEnemy":
      return "the front enemy";
    case "allEnemies":
      return "every enemy";
    case "allAllies":
      return "every ally";
    case "randomEnemy":
      return "a random enemy";
    case "lastDeadAlly":
      return "the most recently dead ally";
  }
}

/** An effect as a verb phrase against a target phrase. */
export function describeEffect(e: Effect, target: string, opts: DescribeOpts = {}): string {
  switch (e.kind) {
    case "damage":
      return e.amount.kind === "const"
        ? `deal ${e.amount.value} damage to ${target}`
        : `deal damage equal to ${describeAmount(e.amount, opts)} to ${target}`;
    case "heal":
      return `heal ${target} for ${amountClause(e.amount, opts)}`;
    case "applyStatus":
      return e.stacks.kind === "const"
        ? `apply ${e.stacks.value} ${e.status} to ${target}`
        : `apply ${e.status} equal to ${describeAmount(e.stacks, opts)} to ${target}`;
    case "consumeStacks": {
      const which = e.status ?? "this status";
      return e.stacks.kind === "const"
        ? `consume ${plural(e.stacks.value, "stack")} of ${which}`
        : `consume stacks of ${which} equal to ${describeAmount(e.stacks, opts)}`;
    }
    case "summon":
      return `summon ${e.unit.name} (${e.unit.base.hp} hp, ${e.unit.base.pwr} pwr) at the back of ${target}'s side`;
    case "silence":
      return `silence ${target} — strip its statuses and disable its abilities for the battle`;
    case "resurrect": {
      // "hp" leads the derived form ("at hp equal to its level"), the
      // preventDeathHeal pattern — trailing it ("at … its level hp") is not English.
      const at = e.hp.kind === "const" ? `${e.hp.value} hp` : `hp equal to ${describeAmount(e.hp, opts)}`;
      return `return ${target} to the back of the line at ${at}`;
    }
    case "cancel":
      return `cancel it${e.consumeSelf !== undefined ? `, consuming ${plural(e.consumeSelf, "stack")}` : ""}`;
    case "absorbHurt":
      return "absorb the damage up to its stacks, consuming what it absorbs";
    case "preventDeathHeal": {
      const to = e.toHp.kind === "const" ? `${e.toHp.value} hp` : `hp equal to ${describeAmount(e.toHp, opts)}`;
      return `cancel the death and heal ${target} to ${to}${e.removeSelf ? ", spending this status" : ""}`;
    }
  }
}

/**
 * One sentence for an ability: whens, condition, then the effect sequence
 * against the selected targets. "After this unit strikes: apply 2 Poison to
 * the front enemy."
 */
export function describeAbility(ab: Ability, opts: DescribeOpts = {}): string {
  const whens = ab.whens.map((w) => describeWhen(w, opts)).join(", or ");
  const cond = ab.condition !== undefined ? `, ${describeCondition(ab.condition, opts)}` : "";
  const target = ab.selectors.map((s) => describeSelector(s, opts)).join(" and ");
  const effects = ab.effects.map((e) => describeEffect(e, target, opts)).join(", then ");
  return `${capitalize(whens)}${cond}: ${effects}.`;
}

/**
 * One description for a status bundle: the per-stack stat contribution, then
 * each ability sentence with "the holder" as the subject. Stack semantics
 * (decay, consumption) surface from the content itself — consumeStacks,
 * absorbHurt, removeSelf all say what they spend.
 */
export function describeStatus(def: StatusDef): string {
  const parts: string[] = [];
  if (def.statMods !== undefined) {
    const mods: string[] = [];
    for (const stat of ["hp", "pwr"] as const) {
      const v = def.statMods[stat];
      if (v !== undefined && v !== 0) mods.push(`${v > 0 ? "+" : ""}${v} ${stat} per stack`);
    }
    if (mods.length > 0) parts.push(`${capitalize(mods.join(", "))}.`);
  }
  for (const ab of def.abilities) parts.push(describeAbility(ab, { holder: "the holder" }));
  if (parts.length === 0) parts.push("No effect.");
  return parts.join(" ");
}
