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

/** How the describing context names the holder: "self" on a unit's own
 * ability, "holder" on a status's (the unit the status is attached to). */
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

const HOLDER_DEFAULT = "self";

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
  /** On target runs: which side the target is on. On a unit-ref run, the
   * side the summoned unit joins. */
  side?: Side;
  /** A summoned unit's name and numbers ("Imp (1/2)", R3-5): its summon id
   * (MvpContent.summons), which the client opens as that unit's card. */
  unitRef?: string;
  /** On every run of a trigger clause, so "After [Shield] lands on an ally"
   * draws as one pill. */
  clause?: "when";
  /** On a trigger clause's runs: whose event it is ("an enemy" in "after an
   * enemy dies"), so the rule shown matches the scope (glossary scopedTip).
   * Unset for the turn and battle triggers, which have no unit. */
  scope?: UnitFilter;
  /** On the number of a damage or heal ("deal [2] damage", "heal it for
   * [2]"), a status's stacks ("apply [2] Poison") or an HP amount ("[2] HP"):
   * the client draws it bold with its term's icon, so the amount carries the
   * icon and the word is only coloured. */
  amount?: true;
}

const seg = (text: string): DescribeSegment => ({ text });

const joinSegments = (segs: DescribeSegment[]): string => segs.map((s) => s.text).join("");

const plural = (n: number, word: string): string => `${n} ${word}${n === 1 ? "" : "s"}`;

const capitalize = (s: string): string => (s.length > 0 ? s[0]!.toUpperCase() + s.slice(1) : s);

/** A trigger's subject, card-speak: none for the holder itself ("Hit"),
 * "ally" (always another ally in the MVP pool), "enemy", "any unit". */
function subject(f: UnitFilter | undefined): string {
  switch (f) {
    case "holder":
      return "";
    case "ally":
    case "otherAlly":
      return "ally";
    case "enemy":
      return "enemy";
    default:
      return "any unit"; // "any" and an omitted filter mean the same thing
  }
}

/** "ally hit", or "hit" when the subject is the holder. */
const subj = (f: UnitFilter | undefined, rest: string): string => {
  const who = subject(f);
  return who ? `${who} ${rest}` : rest;
};

/** An amount as a noun phrase: "3", "PWR", "stacks". */
export function describeAmount(a: Amount, opts: DescribeOpts = {}): string {
  return joinSegments(amountSegments(a, opts));
}

/** describeAmount as segments: a stat or "stacks" word carries its term. */
export function amountSegments(a: Amount, _opts: DescribeOpts = {}): DescribeSegment[] {
  switch (a.kind) {
    case "const":
      return [seg(String(a.value))];
    case "stat":
      // An Amount stat is always the holder's (types.ts), so it needs no owner.
      return [statSeg(a.stat)];
    case "level":
      return [seg("level")];
    case "stacks":
      return [{ text: "stacks", term: "term:stacks" }];
  }
}

/** A stat word, upper case, carrying its term: "PWR", "HP". */
const statSeg = (stat: "pwr" | "hp"): DescribeSegment => ({ text: stat.toUpperCase(), term: `stat:${stat}` });

/** "1 stack" / "2 stacks", the word carrying its term. */
const stacksSegs = (n: number): DescribeSegment[] => [seg(`${n} `), { text: n === 1 ? "stack" : "stacks", term: "term:stacks" }];

/** An HP amount: "2 HP", or "HP equal to stacks" for a derived one. The
 * number is its own run, marked as the amount (with HP's icon, R2-17). */
const hpSegs = (a: Amount, opts: DescribeOpts): DescribeSegment[] =>
  a.kind === "const" ? [{ text: String(a.value), term: "stat:hp", amount: true }, seg(" "), statSeg("hp")] : [statSeg("hp"), seg(" equal to "), ...amountSegments(a, opts)];

/** A when as a clause, card-speak: triggers read "ally dies", interceptors
 * "would be hit". */
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
export function describeWhenSegments(w: When, _opts: DescribeOpts = {}): DescribeSegment[] {
  const p: EventPattern = w.on;
  const intercept = w.kind === "interceptor";
  // The when clause names a Trigger or Interceptor Part (keyed on the event
  // pattern's `on` tag) — the clause's lead phrasing carries the codex ref.
  const ref: PartRef = { family: intercept ? "interceptor" : "trigger", kind: p.on };
  const term: TermId = `trigger:${p.on}`;
  // A clause run: text + the Part ref + the trigger's term; " would " splits
  // out as its own run.
  const filter = p.on === "Strike" ? p.striker : "unit" in p ? p.unit : undefined;
  const scope: { scope?: UnitFilter } = p.on === "BattleStart" || p.on === "TurnStart" || p.on === "TurnEnd" ? {} : { scope: filter ?? "any" };
  const whenSeg = (text: string): DescribeSegment[] =>
    text.split(/(?<=^| )(would)(?= )/).filter((t) => t !== "").map((t) =>
      t === "would" ? { text: t, partRef: ref, term: "term:would", clause: "when" } : { text: t, partRef: ref, term, clause: "when", ...scope },
    );
  const statusSeg = (status: string): DescribeSegment => ({ text: status, statusRef: status, term: `status:${status}`, clause: "when" });
  // Interceptors say "would …" before the verb: "would be hit", "ally would die".
  const verb = (trig: string, icpt: string): string => subj(filter, intercept ? icpt : trig);
  switch (p.on) {
    case "BattleStart":
      return whenSeg("battle start");
    case "TurnStart":
      return whenSeg("turn start");
    case "TurnEnd":
      return whenSeg("turn end");
    case "Strike":
      return whenSeg(verb("strikes", "would strike"));
    case "Hurt":
      return whenSeg(verb("hit", "would be hit"));
    case "Heal":
      return whenSeg(verb("healed", "would be healed"));
    case "Death":
      return whenSeg(verb("dies", "would die"));
    case "Summon":
      return whenSeg(verb("summoned", "would be summoned"));
    case "StatusApplied":
    case "StatusRemoved": {
      const v = p.on === "StatusApplied" ? (intercept ? "would get" : "gets") : intercept ? "would lose" : "loses";
      if (p.status === undefined) return whenSeg(subj(filter, `${v} a status`));
      // The status name carries statusRef; the surrounding trigger phrasing
      // carries the Part ref, so both the status and the trigger are tappable.
      return [...whenSeg(subj(filter, `${v} `)), statusSeg(p.status)];
    }
    case "StatChanged": {
      // Trigger-only (validate rejects an interceptor): "ally gains PWR".
      const stat = p.stat !== undefined ? p.stat.toUpperCase() : "a stat";
      if (p.sign === "gain" || p.sign === "loss") return whenSeg(subj(filter, `${p.sign === "gain" ? "gains" : "loses"} ${stat}`));
      const what = p.stat !== undefined ? p.stat.toUpperCase() : "stat";
      const who = subject(filter);
      return whenSeg(who ? `${who}'s ${what} changes` : `${what} changes`);
    }
  }
}

/** The unit an event is about, as the trigger's filter names it: an eventUnit
 * selector reads "it", tinted by side ("ally dies" → an ally, "enemy hit" → an
 * enemy), or the holder's word when the event is the holder's own. Undefined
 * when the whens disagree or name no one (plain "it"). */
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
      return { text: "it", side: "ally" };
    case "enemy":
      return { text: "it", side: "enemy" };
    default:
      return undefined;
  }
}

/** A condition as a clause: "at 5 HP or less". */
export function describeCondition(c: Condition, opts: DescribeOpts = {}): string {
  return joinSegments(describeConditionSegments(c, opts));
}

/** describeCondition as segments, carrying its Condition Part ref and term. */
export function describeConditionSegments(c: Condition, _opts: DescribeOpts = {}): DescribeSegment[] {
  const partRef: PartRef = { family: "condition", kind: c.kind };
  switch (c.kind) {
    case "holderHpAtMost":
      return [{ text: `at ${c.value} HP or less`, partRef, term: `condition:${c.kind}` }];
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
      return opts.eventUnit?.text ?? "it";
    case "frontEnemy":
      return "front enemy";
    case "allEnemies":
      return "all enemies";
    case "allAllies":
      return "all allies";
    case "randomEnemy":
      return "random enemy";
    case "lastDeadAlly":
      return "fallen ally";
  }
}

/** A summoned body's id (MvpContent.summons): its name, lower-cased ("Imp" → "imp"). */
export const summonId = (name: string): string => name.toLowerCase().replace(/[^a-z0-9]+/g, "-");

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
  // The effect's number: its own run, marked as the amount.
  const eN = (n: number): DescribeSegment => ({ ...eT(String(n)), amount: true });
  const head = shortEffectHead(e);
  if (head) return [...head, e0(" to "), ...tgt];
  switch (e.kind) {
    case "damage":
      // "PWR damage to T"; any other derived amount: "damage equal to stacks to T".
      return e.amount.kind === "stat"
        ? [...amountSegments(e.amount, opts), e0(" "), eT("damage"), e0(" to "), ...tgt]
        : [eT("damage"), e0(" equal to "), ...amountSegments(e.amount, opts), e0(" to "), ...tgt];
    case "heal":
      return e.amount.kind === "const"
        ? [eT("heal"), e0(" "), ...tgt, e0(" for "), eN(e.amount.value)]
        : [eT("heal"), e0(" "), ...tgt, e0(" for "), ...amountSegments(e.amount, opts)];
    case "applyStatus":
      return [statusSeg(e.status), e0(" equal to "), ...amountSegments(e.stacks, opts), e0(" to "), ...tgt];
    case "consumeStacks": {
      // "spend 2 Shield"; a status's own stacks: "spend 1 stack".
      if (e.stacks.kind === "const")
        return e.status !== undefined ? [e0(`spend ${e.stacks.value} `), statusSeg(e.status)] : [e0("spend "), ...stacksSegs(e.stacks.value)];
      const which: DescribeSegment = e.status !== undefined ? statusSeg(e.status) : { text: "stacks", term: "term:stacks" };
      return [e0("spend "), which, e0(" equal to "), ...amountSegments(e.stacks, opts)];
    }
    case "summon": {
      // "summon Imp (1/2)": the numbers are its PWR / HP, which the term's
      // tip says. The kernel summons at the front of the target's line.
      // "Imp (1/2)" is its own run, a ref to the summoned unit's card (R3-5).
      const unitRun = (side: Side): DescribeSegment => ({
        text: `${e.unit.name} (${e.unit.base.pwr}/${e.unit.base.hp})`,
        partRef: ref,
        unitRef: summonId(e.unit.name),
        side,
      });
      const summon = (side: Side): DescribeSegment[] => [eT("summon "), unitRun(side)];
      // All allies / all enemies: the kernel summons once per target, at the
      // front of that target's line, skipping it once the line is full.
      const kinds = tgt.flatMap((t) => (t.partRef?.family === "selector" ? [t.partRef.kind] : []));
      const side: Side = tgt.find((t) => t.side)?.side ?? "ally";
      if (kinds.length === 1 && (kinds[0] === "allAllies" || kinds[0] === "allEnemies")) {
        const ally = kinds[0] === "allAllies";
        // "for each ally": the selector's run, worded per target.
        const each = tgt.map((t) => (t.partRef?.family === "selector" ? { ...t, text: ally ? "each ally" : "each enemy" } : t));
        return [
          ...summon(ally ? "ally" : "enemy"),
          e0(" for "),
          ...each,
          e0(`${ally && !opts.holderGone ? `, ${opts.holder ?? HOLDER_DEFAULT} included` : ""}, if there's room`),
        ];
      }
      if (kinds.length === 1 && kinds[0] === "holder") return summon("ally");
      return [...summon(side), e0(" for "), ...tgt];
    }
    case "silence":
      return [eT("silence"), e0(" "), ...tgt];
    case "resurrect":
      return [eT("revive"), e0(" "), ...tgt, e0(" at "), ...hpSegs(e.hp, opts)];
    case "cancel":
      return e.consumeSelf !== undefined ? [eT("cancel it"), e0(", spend "), ...stacksSegs(e.consumeSelf)] : [eT("cancel it")];
    case "absorbHurt":
      return [eT("block damage"), e0(" up to "), { text: "stacks", term: "term:stacks" }, e0(", spending them")];
    case "preventDeathHeal":
      return [eT("cancel death"), e0(", set "), ...tgt, e0(" to "), ...hpSegs(e.toHp, opts), e0(e.removeSelf ? ", spend this status" : "")];
  }
}

/** The "amount, word" head of an effect that reads "amount, word, to T"
 * (a constant damage or status: "2 damage", "1 Freeze"), else undefined.
 * Consecutive such effects on one target merge: "1 Freeze and 2 damage to
 * front enemy". */
function shortEffectHead(e: Effect): DescribeSegment[] | undefined {
  const ref: PartRef = { family: "effect", kind: e.kind };
  if (e.kind === "damage" && e.amount.kind === "const")
    return [
      { text: String(e.amount.value), partRef: ref, term: "effect:damage", amount: true },
      { text: " ", partRef: ref },
      { text: "damage", partRef: ref, term: "effect:damage" },
    ];
  if (e.kind === "applyStatus" && e.stacks.kind === "const")
    // The stacks are the amount, drawn with the status's icon like a damage number (R2-17).
    return [
      { text: String(e.stacks.value), partRef: ref, term: `status:${e.status}`, amount: true },
      { text: " ", partRef: ref },
      { text: e.status, statusRef: e.status, term: `status:${e.status}` },
    ];
  return undefined;
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
  // Every effect shares the ability's targets, so a run of "amount, word, to T"
  // effects names T once: "1 Freeze and 2 damage to front enemy", and with
  // three or more, commas then a final "and" ("1 Strength, 1 Vitality and 4
  // damage to …"). Any other pair keeps ", then"; the order stays left to right.
  ab.effects.forEach((e, i) => {
    const head = shortEffectHead(e);
    const nextHead = i + 1 < ab.effects.length && head !== undefined && shortEffectHead(ab.effects[i + 1]!) !== undefined;
    const prevHead = i > 0 && head !== undefined && shortEffectHead(ab.effects[i - 1]!) !== undefined;
    if (i > 0) segs.push(seg(prevHead ? (nextHead ? ", " : " and ") : ", then "));
    if (nextHead) segs.push(...head!);
    else segs.push(...describeEffectSegments(e, target, gone ? { ...opts, holderGone: true } : opts));
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

/** Trigger label + glyph per event kind (mockup trigger legend), in the card's
 * words (the glossary's labels): "Strikes", not "after this unit strikes". An
 * interceptor reuses its event's label — the chip line names the moment, not
 * the trigger/interceptor split. */
const TRIGGER_CHIP: Record<EventPattern["on"], { label: string; glyph: string }> = {
  BattleStart: { label: "Battle start", glyph: "⚑" },
  TurnStart: { label: "Turn start", glyph: "⟳" },
  TurnEnd: { label: "Turn end", glyph: "⟲" },
  Strike: { label: "Strikes", glyph: "⚔" },
  Hurt: { label: "Hit", glyph: "✸" },
  Heal: { label: "Healed", glyph: "✚" },
  Death: { label: "Dies", glyph: "☠" },
  Summon: { label: "Summoned", glyph: "✦" },
  StatusApplied: { label: "Gets status", glyph: "✦" },
  StatusRemoved: { label: "Loses status", glyph: "✦" },
  StatChanged: { label: "Stat changes", glyph: "▲" },
};

/** Terse target label per selector, in the card's words (the glossary's
 * labels). "Front enemy", not "the front enemy". */
const SELECTOR_CHIP: Record<Selector["kind"], string> = {
  holder: "Self",
  eventUnit: "It",
  frontEnemy: "Front enemy",
  allEnemies: "All enemies",
  allAllies: "All allies",
  randomEnemy: "Random enemy",
  lastDeadAlly: "Fallen ally",
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
    segs.push(...describeAbilitySegments(ab, { holder: "holder" }));
  }
  if (segs.length === 0) segs.push(seg("No effect."));
  return segs;
}
