// Derived descriptions — part/ability/status in, one human-readable sentence out.
// Everything is computed from the DSL data itself (when + condition + selector +
// effect), never hand-written per unit: player-created content describes itself
// exactly like the shipped stress set does. Display-only; no rule lives here —
// the wording mirrors SPEC semantics but the kernel never reads it back.

import type {
  Ability,
  AbilityDef,
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

/** A Part-card coordinate: the codex card a Part term deep-links to
 * (#codex/part/<family>/<kind>, #078 slice 3). `family` is the atom family,
 * `kind` its union discriminant (an EventPattern `on` tag for trigger/
 * interceptor, the `kind` field otherwise) — the same pair src/parts.ts keys a
 * Part card on. */
export interface PartRef {
  family: "trigger" | "interceptor" | "condition" | "selector" | "effect";
  kind: string;
}

/** One run of a described sentence. A term that names a Part atom carries a
 * `partRef` (every Trigger / Interceptor / Condition / Selector / Effect — the
 * codex is the complete, tappable vocabulary, #078). A term that names a status
 * carries `statusRef` (the registry defines it; an applyStatus/consumeStacks
 * status is BOTH a status and an effect's payload, so it may carry both — the UI
 * resolves the status). Joining every segment's `text` reproduces the plain
 * describe* string exactly; the refs are metadata only. */
export interface DescribeSegment {
  text: string;
  /** Set when `text` is a status name (applyStatus / consumeStacks). */
  statusRef?: string;
  /** Set when `text` names a Part atom — its codex Part card. */
  partRef?: PartRef;
}

const seg = (text: string): DescribeSegment => ({ text });

const joinSegments = (segs: DescribeSegment[]): string => segs.map((s) => s.text).join("");

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
  return joinSegments(describeWhenSegments(w, opts));
}

/** describeWhen as segments — identical text, with an explicit status pattern
 * (StatusApplied/StatusRemoved naming a status) marked as a ref, so
 * editor-made content like "After Poison lands on an ally" gets a tappable
 * Poison exactly like effect clauses do. */
export function describeWhenSegments(w: When, opts: DescribeOpts = {}): DescribeSegment[] {
  const holder = opts.holder ?? HOLDER_DEFAULT;
  const p: EventPattern = w.on;
  const intercept = w.kind === "interceptor";
  // The when clause names a Trigger or Interceptor Part (keyed on the event
  // pattern's `on` tag) — the clause's lead phrasing carries the codex ref.
  const ref: PartRef = { family: intercept ? "interceptor" : "trigger", kind: p.on };
  // A whole-clause segment that IS the trigger term: text + the Part ref.
  const whenSeg = (text: string): DescribeSegment => ({ text, partRef: ref });
  switch (p.on) {
    case "BattleStart":
      return [whenSeg("when the battle begins")];
    case "TurnStart":
      return [whenSeg("at the start of each turn")];
    case "TurnEnd":
      return [whenSeg("at the end of each turn")];
    case "Strike": {
      const who = filterPhrase(p.striker, holder);
      return [whenSeg(intercept ? `when ${who} would strike` : `after ${who} strikes`)];
    }
    case "Hurt": {
      const who = filterPhrase(p.unit, holder);
      return [whenSeg(intercept ? `when ${who} would be hurt` : `after ${who} is hurt`)];
    }
    case "Heal": {
      const who = filterPhrase(p.unit, holder);
      return [whenSeg(intercept ? `when ${who} would be healed` : `after ${who} is healed`)];
    }
    case "Death": {
      const who = filterPhrase(p.unit, holder);
      return [whenSeg(intercept ? `when ${who} would die` : `after ${who} dies`)];
    }
    case "Summon": {
      const who = filterPhrase(p.unit, holder);
      return [whenSeg(intercept ? `when ${who} would be summoned` : `after ${who} is summoned`)];
    }
    case "StatusApplied": {
      const who = filterPhrase(p.unit, holder);
      if (p.status === undefined)
        return [whenSeg(intercept ? `when a status would land on ${who}` : `after a status lands on ${who}`)];
      // The status name carries statusRef; the surrounding trigger phrasing
      // carries the Part ref, so both the status and the trigger are tappable.
      const sref: DescribeSegment = { text: p.status, statusRef: p.status };
      return intercept
        ? [whenSeg("when "), sref, whenSeg(` would land on ${who}`)]
        : [whenSeg("after "), sref, whenSeg(` lands on ${who}`)];
    }
    case "StatusRemoved": {
      const who = filterPhrase(p.unit, holder);
      if (p.status === undefined)
        return [whenSeg(intercept ? `when a status would leave ${who}` : `after a status leaves ${who}`)];
      const sref: DescribeSegment = { text: p.status, statusRef: p.status };
      return intercept
        ? [whenSeg("when "), sref, whenSeg(` would leave ${who}`)]
        : [whenSeg("after "), sref, whenSeg(` leaves ${who}`)];
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

/** describeCondition as one segment, carrying its Condition Part ref. */
export function describeConditionSegments(c: Condition, opts: DescribeOpts = {}): DescribeSegment[] {
  return [{ text: describeCondition(c, opts), partRef: { family: "condition", kind: c.kind } }];
}

/** describeSelector as one segment, carrying its Selector Part ref — the noun
 * phrase a UI makes tappable to the selector's codex card. */
export function describeSelectorSegments(s: Selector, opts: DescribeOpts = {}): DescribeSegment[] {
  return [{ text: describeSelector(s, opts), partRef: { family: "selector", kind: s.kind } }];
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

/** An effect as segments: the same verb phrase describeEffect yields, with
 * status names (applyStatus / consumeStacks) marked as refs a UI can wire to
 * the registry's definition, and every effect-text run carrying the Effect's
 * own Part ref. `target` arrives as segments (the selectors' own ref-bearing
 * segments) so a selector term inside the sentence stays tappable to its
 * Selector card; a bare string target is lifted to one plain segment. */
export function describeEffectSegments(
  e: Effect,
  target: string | DescribeSegment[],
  opts: DescribeOpts = {},
): DescribeSegment[] {
  // The selected-target phrase as segments — keeps selector refs intact.
  const tgt: DescribeSegment[] = typeof target === "string" ? [seg(target)] : target;
  // Effect-text runs carry this effect's Part ref (the codex Effect card).
  const ref: PartRef = { family: "effect", kind: e.kind };
  const e0 = (text: string): DescribeSegment => ({ text, partRef: ref });
  switch (e.kind) {
    case "damage":
      return e.amount.kind === "const"
        ? [e0(`deal ${e.amount.value} damage to `), ...tgt]
        : [e0(`deal damage equal to ${describeAmount(e.amount, opts)} to `), ...tgt];
    case "heal":
      return [e0("heal "), ...tgt, e0(` for ${amountClause(e.amount, opts)}`)];
    case "applyStatus": {
      const sref: DescribeSegment = { text: e.status, statusRef: e.status };
      return e.stacks.kind === "const"
        ? [e0(`apply ${e.stacks.value} `), sref, e0(" to "), ...tgt]
        : [e0("apply "), sref, e0(` equal to ${describeAmount(e.stacks, opts)} to `), ...tgt];
    }
    case "consumeStacks": {
      const which: DescribeSegment = e.status !== undefined ? { text: e.status, statusRef: e.status } : e0("this status");
      return e.stacks.kind === "const"
        ? [e0(`consume ${plural(e.stacks.value, "stack")} of `), which]
        : [e0("consume stacks of "), which, e0(` equal to ${describeAmount(e.stacks, opts)}`)];
    }
    case "summon":
      return [
        e0(`summon ${e.unit.name} (${e.unit.base.hp} hp, ${e.unit.base.pwr} pwr) at the back of `),
        ...tgt,
        e0("'s side"),
      ];
    case "silence":
      return [e0("silence "), ...tgt, e0(" — strip its statuses and disable its abilities for the battle")];
    case "resurrect": {
      // "hp" leads the derived form ("at hp equal to its level"), the
      // preventDeathHeal pattern — trailing it ("at … its level hp") is not English.
      const at = e.hp.kind === "const" ? `${e.hp.value} hp` : `hp equal to ${describeAmount(e.hp, opts)}`;
      return [e0("return "), ...tgt, e0(` to the back of the line at ${at}`)];
    }
    case "cancel":
      return [e0(`cancel it${e.consumeSelf !== undefined ? `, consuming ${plural(e.consumeSelf, "stack")}` : ""}`)];
    case "absorbHurt":
      return [e0("absorb the damage up to its stacks, consuming what it absorbs")];
    case "preventDeathHeal": {
      const to = e.toHp.kind === "const" ? `${e.toHp.value} hp` : `hp equal to ${describeAmount(e.toHp, opts)}`;
      return [e0("cancel the death and heal "), ...tgt, e0(` to ${to}${e.removeSelf ? ", spending this status" : ""}`)];
    }
  }
}

/** An effect as a verb phrase against a target phrase. */
export function describeEffect(e: Effect, target: string, opts: DescribeOpts = {}): string {
  return joinSegments(describeEffectSegments(e, target, opts));
}

/**
 * One sentence for an ability: whens, condition, then the effect sequence
 * against the selected targets. "After this unit strikes: apply 2 Poison to
 * the front enemy."
 */
export function describeAbility(ab: Ability, opts: DescribeOpts = {}): string {
  return joinSegments(describeAbilitySegments(ab, opts));
}

/** One sentence for a named AbilityDef (PRD #081) — the same derived body
 * sentence describeAbility yields. The name and family (the colour axis) are
 * presented separately by the codex; the description is the mechanic, derived
 * from the DSL like everything else. An inert ability (the vanilla Strike,
 * whose effects emit nothing) reads as a plain basic attack. */
export function describeAbilityDef(def: AbilityDef, opts: DescribeOpts = {}): string {
  return describeAbility(def, opts);
}

/** describeAbility as segments — identical text, with status AND Part refs
 * marked: every term (trigger/interceptor when, condition, selector, effect)
 * carries the codex card it links to (#078 slice 3). */
export function describeAbilitySegments(ab: Ability, opts: DescribeOpts = {}): DescribeSegment[] {
  // The selected targets as segments — each selector its own tappable term,
  // joined by plain " and " text. Reused for every effect in the sequence.
  const target: DescribeSegment[] = [];
  ab.selectors.forEach((s, i) => {
    if (i > 0) target.push(seg(" and "));
    target.push(...describeSelectorSegments(s, opts));
  });
  // The when clauses keep their segment shape (a status-pattern when carries a
  // ref); every clause opens with plain lead text ("after"/"when"/"at"), so
  // capitalizing the first segment is capitalizing the sentence.
  const segs: DescribeSegment[] = [];
  ab.whens.forEach((w, i) => {
    if (i > 0) segs.push(seg(", or "));
    segs.push(...describeWhenSegments(w, opts));
  });
  if (segs.length > 0) segs[0] = { ...segs[0]!, text: capitalize(segs[0]!.text) };
  // The condition is its own tappable Part term, framed by the comma and colon.
  if (ab.condition !== undefined) {
    segs.push(seg(", "));
    segs.push(...describeConditionSegments(ab.condition, opts));
  }
  segs.push(seg(": "));
  ab.effects.forEach((e, i) => {
    if (i > 0) segs.push(seg(", then "));
    segs.push(...describeEffectSegments(e, target, opts));
  });
  segs.push(seg("."));
  return segs;
}

// ---------- Terse chip line for the B·Arena card (PRD #082) ----------
// Where describeAbility yields one prose sentence ("after this unit strikes:
// apply 2 Poison to the front enemy"), the card wants three glance-able chips:
// `<glyph> trigger ▸ target ▸ action`. Same DSL data, read short. Display-only,
// like the rest of this file; the action's GLYPH is the unit's family glyph, so
// the card derives it from the colour axis it already holds — only the trigger's
// glyph (which depends on the event kind) travels with the chips here.

/** The terse ability line as three short labels + the trigger's glyph. Any
 * field may be absent (an ability with no when/selector/effect); the card drops
 * an absent chip and its separator. */
export interface AbilityChips {
  trigger?: string | undefined;
  /** Glyph for the trigger chip, by event kind (⚔ strike, ⚑ battle-start, …). */
  triggerGlyph?: string | undefined;
  target?: string | undefined;
  action?: string | undefined;
}

/** Trigger label + glyph per event kind (mockup trigger legend). Terse: "On
 * strike", not "after this unit strikes". An interceptor reuses its event's
 * label — the chip line names the moment, not the trigger/interceptor split. */
const TRIGGER_CHIP: Record<EventPattern["on"], { label: string; glyph: string }> = {
  BattleStart: { label: "Battle start", glyph: "⚑" },
  TurnStart: { label: "Turn start", glyph: "⟳" },
  TurnEnd: { label: "Turn end", glyph: "⟲" },
  Strike: { label: "On strike", glyph: "⚔" },
  Hurt: { label: "On damaged", glyph: "✸" },
  Heal: { label: "On heal", glyph: "✚" },
  Death: { label: "On death", glyph: "☠" },
  Summon: { label: "On summon", glyph: "✦" },
  StatusApplied: { label: "Status gained", glyph: "✦" },
  StatusRemoved: { label: "Status lost", glyph: "✦" },
};

/** Terse target label per selector (mockup target legend). "Front enemy", not
 * "the front enemy". */
const SELECTOR_CHIP: Record<Selector["kind"], string> = {
  holder: "Self",
  eventUnit: "Trigger unit",
  frontEnemy: "Front enemy",
  allEnemies: "All enemies",
  allAllies: "All allies",
  randomEnemy: "Random enemy",
  lastDeadAlly: "Last dead ally",
};

/** A magnitude as a terse chip token: a const reads as its number, anything
 * derived as the short name of what it scales on. */
function terseAmount(a: Amount): string {
  switch (a.kind) {
    case "const":
      return String(a.value);
    case "stat":
      return a.stat;
    case "level":
      return "level";
    case "stacks":
      return "stacks";
  }
}

/** An effect as a terse action chip: verb + magnitude, the target folded out
 * (it has its own chip). "Poison 2", "Deal 3", "Summon Imp", "Silence". */
function terseAction(e: Effect): string {
  switch (e.kind) {
    case "damage":
      return `Deal ${terseAmount(e.amount)}`;
    case "heal":
      return `Heal ${terseAmount(e.amount)}`;
    case "applyStatus":
      return e.stacks.kind === "const" ? `${e.status} ${e.stacks.value}` : e.status;
    case "consumeStacks":
      return `Spend ${e.status ?? "stacks"}`;
    case "summon":
      return `Summon ${e.unit.name}`;
    case "silence":
      return "Silence";
    case "resurrect":
      return "Revive";
    case "cancel":
      return "Cancel";
    case "absorbHurt":
      return "Absorb";
    case "preventDeathHeal":
      return "Cheat death";
  }
}

/** The card's terse 3-chip ability line for an ability — the first when/
 * selector/effect, each as a short label (PRD #082). The verbose
 * describeAbility sentence stays the inspector's; this is the at-a-glance read. */
export function abilityChips(ab: Ability): AbilityChips {
  const w0 = ab.whens[0];
  const t = w0 !== undefined ? TRIGGER_CHIP[w0.on.on] : undefined;
  const s0 = ab.selectors[0];
  const e0 = ab.effects[0];
  return {
    trigger: t?.label,
    triggerGlyph: t?.glyph,
    target: s0 !== undefined ? SELECTOR_CHIP[s0.kind] : undefined,
    action: e0 !== undefined ? terseAction(e0) : undefined,
  };
}

/** Status names an ability's effects reference (applyStatus, and
 * consumeStacks with an explicit status), deduped in encounter order — the
 * refs a UI renders tappable, and the codex resolves in the registry. */
export function abilityStatusRefs(ab: Ability): string[] {
  const refs: string[] = [];
  for (const s of describeAbilitySegments(ab)) {
    if (s.statusRef !== undefined && !refs.includes(s.statusRef)) refs.push(s.statusRef);
  }
  return refs;
}

/** The Part cards an ability's sentence links to (every trigger/interceptor/
 * condition/selector/effect term), deduped on family+kind in encounter order —
 * the codex Part cards a UI renders tappable (#078 slice 3). */
export function abilityPartRefs(ab: Ability): PartRef[] {
  const refs: PartRef[] = [];
  for (const s of describeAbilitySegments(ab)) {
    if (s.partRef !== undefined && !refs.some((r) => r.family === s.partRef!.family && r.kind === s.partRef!.kind))
      refs.push(s.partRef);
  }
  return refs;
}

/**
 * One description for a status bundle: the per-stack stat contribution, then
 * each ability sentence with "the holder" as the subject. Stack semantics
 * (decay, consumption) surface from the content itself — consumeStacks,
 * absorbHurt, removeSelf all say what they spend.
 */
export function describeStatus(def: StatusDef): string {
  return joinSegments(describeStatusSegments(def));
}

/** describeStatus as segments — identical text, with status refs marked. */
export function describeStatusSegments(def: StatusDef): DescribeSegment[] {
  const segs: DescribeSegment[] = [];
  if (def.statMods !== undefined) {
    const mods: string[] = [];
    for (const stat of ["hp", "pwr"] as const) {
      const v = def.statMods[stat];
      if (v !== undefined && v !== 0) mods.push(`${v > 0 ? "+" : ""}${v} ${stat} per stack`);
    }
    if (mods.length > 0) segs.push(seg(`${capitalize(mods.join(", "))}.`));
  }
  for (const ab of def.abilities) {
    if (segs.length > 0) segs.push(seg(" "));
    segs.push(...describeAbilitySegments(ab, { holder: "the holder" }));
  }
  if (segs.length === 0) segs.push(seg("No effect."));
  return segs;
}
