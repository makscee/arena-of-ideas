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
import type { TermId } from "./glossary.js";

/** How the describing context names the holder: "this unit" on a unit's own
 * ability, "the holder" on a status's (the unit the status is attached to). */
export interface DescribeOpts {
  holder?: string;
  /** The ability fires on its holder's own death: the holder has left the
   * line, so an every-ally effect doesn't include it. */
  holderGone?: boolean;
  /** How an eventUnit selector reads ("that ally", "that enemy"), worked out
   * from the ability's whens by describeAbilitySegments. */
  eventUnit?: { text: string; side?: Side };
}

/** Which side a target is on, relative to the holder: the client tints ally
 * terms teal and enemy terms pink. */
export type Side = "ally" | "enemy";

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
  /** Set on every highlighted run: its glossary term (src/glossary.ts). Glue
   * text ("to", ", then", ":") has none. */
  term?: TermId;
  /** On target runs: which side the target is on. */
  side?: Side;
  /** On every run of a trigger clause, so "After [Shield] lands on an ally"
   * draws as one pill. */
  clause?: "when";
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
    case "otherAlly":
      return "another ally";
    case "enemy":
      return "an enemy";
    default:
      return "any unit"; // "any" and an omitted filter mean the same thing
  }
}

/** An amount as a noun phrase: "3", "this unit's PWR", "its stacks". */
export function describeAmount(a: Amount, opts: DescribeOpts = {}): string {
  return joinSegments(amountSegments(a, opts));
}

/** describeAmount as segments: a stat or "stacks" word carries its term. */
export function amountSegments(a: Amount, opts: DescribeOpts = {}): DescribeSegment[] {
  const holder = opts.holder ?? HOLDER_DEFAULT;
  switch (a.kind) {
    case "const":
      return [seg(String(a.value))];
    case "stat":
      return [seg(`${holder}'s `), statSeg(a.stat)];
    case "level":
      return [seg(`${holder}'s level`)];
    case "stacks":
      return [seg("its "), { text: "stacks", term: "term:stacks" }];
  }
}

/** A stat word, upper case, carrying its term: "PWR", "HP". */
const statSeg = (stat: "pwr" | "hp"): DescribeSegment => ({ text: stat.toUpperCase(), term: `stat:${stat}` });

/** "1 stack" / "2 stacks", the word carrying its term. */
const stacksSegs = (n: number): DescribeSegment[] => [seg(`${n} `), { text: n === 1 ? "stack" : "stacks", term: "term:stacks" }];

/** An HP amount: "2 HP", or "HP equal to its stacks" for a derived one. */
const hpSegs = (a: Amount, opts: DescribeOpts): DescribeSegment[] =>
  a.kind === "const" ? [seg(`${a.value} `), statSeg("hp")] : [statSeg("hp"), seg(" equal to "), ...amountSegments(a, opts)];

/** A when as a clause: triggers read "after X", interceptors "when X would …". */
export function describeWhen(w: When, opts: DescribeOpts = {}): string {
  return joinSegments(describeWhenSegments(w, opts));
}

/** describeWhen as segments — identical text. Every run of the clause carries
 * the trigger's term and `clause: "when"`; a status the pattern names
 * (StatusApplied/StatusRemoved) is its own run with the status's term, and an
 * interceptor's "would" is its own run too.
 *
 * The Hurt event reads "is hit", not "is hurt": it fires for every damage that
 * comes at a unit, even a hit Shield blocks completely, so the wording mustn't
 * promise lost HP. "Is struck" would collide with the Strike trigger
 * ("strikes"), and Poison or Fatigue damage isn't a strike; "is damaged"
 * promises the HP loss again. */
export function describeWhenSegments(w: When, opts: DescribeOpts = {}): DescribeSegment[] {
  const holder = opts.holder ?? HOLDER_DEFAULT;
  const p: EventPattern = w.on;
  const intercept = w.kind === "interceptor";
  // The when clause names a Trigger or Interceptor Part (keyed on the event
  // pattern's `on` tag) — the clause's lead phrasing carries the codex ref.
  const ref: PartRef = { family: intercept ? "interceptor" : "trigger", kind: p.on };
  const term: TermId = `trigger:${p.on}`;
  // A clause run: text + the Part ref + the trigger's term; " would " splits
  // out as its own run.
  const whenSeg = (text: string): DescribeSegment[] =>
    text.split(/(?<= )(would)(?= )/).filter((t) => t !== "").map((t) =>
      t === "would" ? { text: t, partRef: ref, term: "term:would", clause: "when" } : { text: t, partRef: ref, term, clause: "when" },
    );
  const statusSeg = (status: string): DescribeSegment => ({ text: status, statusRef: status, term: `status:${status}`, clause: "when" });
  switch (p.on) {
    case "BattleStart":
      return whenSeg("when the battle begins");
    case "TurnStart":
      return whenSeg("at the start of each turn");
    case "TurnEnd":
      return whenSeg("at the end of each turn");
    case "Strike": {
      const who = filterPhrase(p.striker, holder);
      return whenSeg(intercept ? `when ${who} would strike` : `after ${who} strikes`);
    }
    case "Hurt": {
      const who = filterPhrase(p.unit, holder);
      return whenSeg(intercept ? `when ${who} would be hit` : `after ${who} is hit`);
    }
    case "Heal": {
      const who = filterPhrase(p.unit, holder);
      return whenSeg(intercept ? `when ${who} would be healed` : `after ${who} is healed`);
    }
    case "Death": {
      const who = filterPhrase(p.unit, holder);
      return whenSeg(intercept ? `when ${who} would die` : `after ${who} dies`);
    }
    case "Summon": {
      const who = filterPhrase(p.unit, holder);
      return whenSeg(intercept ? `when ${who} would be summoned` : `after ${who} is summoned`);
    }
    case "StatusApplied": {
      const who = filterPhrase(p.unit, holder);
      if (p.status === undefined)
        return whenSeg(intercept ? `when a status would land on ${who}` : `after a status lands on ${who}`);
      // The status name carries statusRef; the surrounding trigger phrasing
      // carries the Part ref, so both the status and the trigger are tappable.
      return intercept
        ? [...whenSeg("when "), statusSeg(p.status), ...whenSeg(` would land on ${who}`)]
        : [...whenSeg("after "), statusSeg(p.status), ...whenSeg(` lands on ${who}`)];
    }
    case "StatusRemoved": {
      const who = filterPhrase(p.unit, holder);
      if (p.status === undefined)
        return whenSeg(intercept ? `when a status would leave ${who}` : `after a status leaves ${who}`);
      return intercept
        ? [...whenSeg("when "), statusSeg(p.status), ...whenSeg(` would leave ${who}`)]
        : [...whenSeg("after "), statusSeg(p.status), ...whenSeg(` leaves ${who}`)];
    }
    case "StatChanged": {
      // Trigger-only (validate rejects an interceptor): "after an ally gains PWR".
      const who = filterPhrase(p.unit, holder);
      const verb = p.sign === "gain" ? "gains" : p.sign === "loss" ? "loses" : "changes";
      const stat = p.stat !== undefined ? p.stat.toUpperCase() : "a stat";
      return whenSeg(`after ${who} ${verb} ${stat}`);
    }
  }
}

/** The unit an event is about, as the trigger's filter names it: an eventUnit
 * selector reads "that ally" after an ally's event, "that enemy" after an
 * enemy's. Undefined when the whens disagree or name no one ("that unit"). */
function eventUnitOf(whens: When[], holder: string): { text: string; side?: Side } | undefined {
  const filters = whens.map((w) => {
    const p = w.on;
    switch (p.on) {
      case "BattleStart":
      case "TurnStart":
      case "TurnEnd":
        return undefined;
      case "Strike":
        return p.striker;
      default:
        return p.unit;
    }
  });
  const first = filters[0];
  if (first === undefined || filters.some((f) => f !== first)) return undefined;
  switch (first) {
    case "holder":
      return { text: holder, side: "ally" };
    case "ally":
    case "otherAlly":
      return { text: "that ally", side: "ally" };
    case "enemy":
      return { text: "that enemy", side: "enemy" };
    default:
      return undefined;
  }
}

/** A condition as a "while …" clause. */
export function describeCondition(c: Condition, opts: DescribeOpts = {}): string {
  return joinSegments(describeConditionSegments(c, opts));
}

/** describeCondition as segments, carrying its Condition Part ref and term. */
export function describeConditionSegments(c: Condition, opts: DescribeOpts = {}): DescribeSegment[] {
  const holder = opts.holder ?? HOLDER_DEFAULT;
  const partRef: PartRef = { family: "condition", kind: c.kind };
  switch (c.kind) {
    case "holderHpAtMost":
      return [{ text: `while ${holder} is at ${c.value} HP or less`, partRef, term: `condition:${c.kind}` }];
  }
}

/** Which side each selector kind picks from (eventUnit: from the trigger). */
const SELECTOR_SIDE: Record<Selector["kind"], Side | undefined> = {
  holder: "ally",
  eventUnit: undefined,
  frontEnemy: "enemy",
  allEnemies: "enemy",
  allAllies: "ally",
  randomEnemy: "enemy",
  lastDeadAlly: "ally",
};

/** describeSelector as one segment, carrying its Selector Part ref, term and
 * side — the noun phrase a UI makes tappable to the selector's codex card. */
export function describeSelectorSegments(s: Selector, opts: DescribeOpts = {}): DescribeSegment[] {
  const side = s.kind === "eventUnit" ? opts.eventUnit?.side : SELECTOR_SIDE[s.kind];
  return [{ text: describeSelector(s, opts), partRef: { family: "selector", kind: s.kind }, term: `target:${s.kind}`, ...(side ? { side } : {}) }];
}

/** A selector as the noun phrase of what it picks. */
export function describeSelector(s: Selector, opts: DescribeOpts = {}): string {
  const holder = opts.holder ?? HOLDER_DEFAULT;
  switch (s.kind) {
    case "holder":
      return holder;
    case "eventUnit":
      return opts.eventUnit?.text ?? "that unit";
    case "frontEnemy":
      return "the front enemy";
    case "allEnemies":
      return "every enemy";
    case "allAllies":
      return "every ally";
    case "randomEnemy":
      return "a random enemy";
    case "lastDeadAlly":
      return "the last fallen ally";
  }
}

/** An effect as segments: the same verb phrase describeEffect yields. The
 * highlighted runs carry their term (the amount with its effect, a status
 * name, a stat word); the glue runs carry the Effect's Part ref only. `target`
 * arrives as segments (the selectors' own term-bearing segments) so a selector
 * term inside the sentence stays tappable; a bare string target is lifted to
 * one plain segment. */
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
  // A highlighted effect run: the Part ref plus the effect's term.
  const eT = (text: string): DescribeSegment => ({ text, partRef: ref, term: `effect:${e.kind}` });
  const statusSeg = (status: string): DescribeSegment => ({ text: status, statusRef: status, term: `status:${status}` });
  switch (e.kind) {
    case "damage":
      return e.amount.kind === "const"
        ? [e0("deal "), eT(`${e.amount.value} damage`), e0(" to "), ...tgt]
        : [e0("deal "), eT("damage"), e0(" equal to "), ...amountSegments(e.amount, opts), e0(" to "), ...tgt];
    case "heal":
      return e.amount.kind === "const"
        ? [eT("heal"), e0(" "), ...tgt, e0(" for "), eT(String(e.amount.value))]
        : [eT("heal"), e0(" "), ...tgt, e0(" for "), ...amountSegments(e.amount, opts)];
    case "applyStatus":
      return e.stacks.kind === "const"
        ? [e0(`apply ${e.stacks.value} `), statusSeg(e.status), e0(" to "), ...tgt]
        : [e0("apply "), statusSeg(e.status), e0(" equal to "), ...amountSegments(e.stacks, opts), e0(" to "), ...tgt];
    case "consumeStacks": {
      const which: DescribeSegment = e.status !== undefined ? statusSeg(e.status) : e0("this status");
      return e.stacks.kind === "const"
        ? [e0("consume "), ...stacksSegs(e.stacks.value), e0(" of "), which]
        : [e0("consume "), { text: "stacks", term: "term:stacks" }, e0(" of "), which, e0(" equal to "), ...amountSegments(e.stacks, opts)];
    }
    case "summon": {
      // "summon an Imp (1/2)": the numbers are its PWR / HP, which the term's
      // tip says. The kernel summons at the back of the target's line.
      const unit = `${/^[aeiou]/i.test(e.unit.name) ? "an" : "a"} ${e.unit.name} (${e.unit.base.pwr}/${e.unit.base.hp})`;
      // Every ally / every enemy: the kernel summons once per target, at the
      // back of that target's line, skipping it once the line is full.
      const kinds = tgt.flatMap((t) => (t.partRef?.family === "selector" ? [t.partRef.kind] : []));
      if (kinds.length === 1 && (kinds[0] === "allAllies" || kinds[0] === "allEnemies")) {
        const ally = kinds[0] === "allAllies";
        return [
          eT(`summon ${unit}`),
          e0(" for "),
          ...tgt,
          e0(`${ally && !opts.holderGone ? `, ${opts.holder ?? HOLDER_DEFAULT} included` : ""}, while the line has room`),
        ];
      }
      if (kinds.length === 1 && kinds[0] === "holder") return [eT(`summon ${unit}`)];
      return [eT(`summon ${unit}`), e0(" on the side of "), ...tgt];
    }
    case "silence":
      return [eT("silence"), e0(" "), ...tgt];
    case "resurrect":
      return [eT("revive"), e0(" "), ...tgt, e0(" at "), ...hpSegs(e.hp, opts)];
    case "cancel":
      return e.consumeSelf !== undefined
        ? [eT("cancel it"), e0(", consuming "), ...stacksSegs(e.consumeSelf)]
        : [eT("cancel it")];
    case "absorbHurt":
      return [eT("absorb the damage"), e0(" up to its "), { text: "stacks", term: "term:stacks" }, e0(", consuming what it absorbs")];
    case "preventDeathHeal":
      return [eT("cancel the death"), e0(" and "), eT("heal"), e0(" "), ...tgt, e0(" to "), ...hpSegs(e.toHp, opts), e0(e.removeSelf ? ", spending this status" : "")];
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
export function describeAbilitySegments(ab: Ability, opts0: DescribeOpts = {}): DescribeSegment[] {
  // An eventUnit target reads as the unit the trigger is about ("that ally").
  const eventUnit = opts0.eventUnit ?? eventUnitOf(ab.whens ?? [], opts0.holder ?? HOLDER_DEFAULT);
  const opts: DescribeOpts = eventUnit ? { ...opts0, eventUnit } : opts0;
  // The selected targets as segments — each selector its own tappable term,
  // joined by plain " and " text. Reused for every effect in the sequence.
  const target: DescribeSegment[] = [];
  (ab.selectors ?? []).forEach((s, i) => {
    if (i > 0) target.push(seg(" and "));
    target.push(...describeSelectorSegments(s, opts));
  });
  // The when clauses keep their segment shape (a status-pattern when carries a
  // ref); every clause opens with plain lead text ("after"/"when"/"at"), so
  // capitalizing the first segment is capitalizing the sentence.
  const segs: DescribeSegment[] = [];
  (ab.whens ?? []).forEach((w, i) => {
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
  const gone = (ab.whens ?? []).some((w) => w.kind !== "interceptor" && w.on.on === "Death" && w.on.unit === "holder");
  ab.effects.forEach((e, i) => {
    if (i > 0) segs.push(seg(", then "));
    segs.push(...describeEffectSegments(e, target, gone ? { ...opts, holderGone: true } : opts));
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
  Hurt: { label: "On hit", glyph: "✸" },
  Heal: { label: "On heal", glyph: "✚" },
  Death: { label: "On death", glyph: "☠" },
  Summon: { label: "On summon", glyph: "✦" },
  StatusApplied: { label: "Status gained", glyph: "✦" },
  StatusRemoved: { label: "Status lost", glyph: "✦" },
  StatChanged: { label: "Stat changed", glyph: "▲" },
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
  const w0 = ab.whens?.[0];
  const t = w0 !== undefined ? TRIGGER_CHIP[w0.on.on] : undefined;
  const s0 = ab.selectors?.[0];
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
    for (const stat of ["hp", "pwr"] as const) {
      const v = def.statMods[stat];
      if (v === undefined || v === 0) continue;
      if (segs.length > 0) segs.push(seg(", "));
      segs.push(seg(`${v > 0 ? "+" : ""}${v} `), statSeg(stat), seg(" per "), { text: "stack", term: "term:stacks" });
    }
    if (segs.length > 0) segs.push(seg("."));
  }
  for (const action of def.abilities) {
    if (segs.length > 0) segs.push(seg(" "));
    const ab: Ability = {
      ...action,
      whens: def.triggers ?? action.whens ?? [],
      selectors: def.selectors ?? action.selectors ?? [],
      ...(def.condition ?? action.condition ? { condition: def.condition ?? action.condition } : {}),
    };
    segs.push(...describeAbilitySegments(ab, { holder: "the holder" }));
  }
  if (segs.length === 0) segs.push(seg("No effect."));
  return segs;
}
