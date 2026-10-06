// Battle viewer logic (mission #574, slice 9): playback steps with one-line
// captions, tap-a-change-to-trace-its-chain, and "why I lost". Pure functions
// over the causal log; the phone client (mobile/screens/battle.ts) only draws
// what these return. "Why did this happen" is always a walk up `causedBy`.

import { beatsOf, isRootKind } from "../beats.js";
import { displayNames, type NameOf } from "../trace.js";
import type { AbilityRef, BattleEvent, Side, UnitFilter, When } from "../types.js";

// ---------- who acted ----------

/** The unit that made an event happen, and how. Events with no actor of their
 * own (a Death, a StatChanged that follows a status) return null and the
 * trace walks on to their cause. A status's tick jumps to where the status
 * was applied, so "Poison −2" traces back to whoever poisoned. */
interface ActorInfo {
  unit: string | null;
  /** "strike", "ability", or a status name. */
  via: string;
  /** Continue the walk here instead of `causedBy` (status origin). */
  jumpTo?: number;
}

function statusOriginId(log: BattleEvent[], unit: string, status: string, beforeId: number): number | undefined {
  for (let i = beforeId - 1; i >= 0; i--) {
    const e = log[i]!;
    if (e.type === "StatusApplied" && e.unit === unit && e.status === status) return i;
  }
  return undefined;
}

function actorOf(log: BattleEvent[], e: BattleEvent): ActorInfo | null {
  if (e.type === "Strike") return { unit: e.striker, via: "strike" };
  if (e.source !== "kernel") {
    if (e.source.status === undefined) return { unit: e.source.unit, via: "ability" };
    const originId = statusOriginId(log, e.source.unit, e.source.status, e.id);
    const origin = originId !== undefined ? log[originId] : undefined;
    const originActor = origin ? actorOf(log, origin) : null;
    return { unit: originActor?.unit ?? null, via: e.source.status, ...(originId !== undefined ? { jumpTo: originId } : {}) };
  }
  if (e.type === "Hurt" && e.causedBy !== null) {
    const parent = log[e.causedBy];
    if (parent?.type === "Strike") return { unit: parent.striker, via: "strike", jumpTo: parent.id };
  }
  return null;
}

/** Side of every unit instance in the log (rosters plus summons). */
export function sidesOf(log: BattleEvent[]): Map<string, Side> {
  const sides = new Map<string, Side>();
  for (const e of log) {
    if (e.type === "BattleStart") for (const s of ["A", "B"] as const) for (const r of e.teams[s]) sides.set(r.id, s);
    else if (e.type === "Summon") sides.set(e.unit, e.side);
  }
  return sides;
}

// ---------- changes and their traces ----------

export type ChangeKind = "damage" | "heal" | "buff" | "debuff" | "status" | "death" | "summon" | "silence";

/** A change you can tap: a number (or mark) on a unit card. */
export interface Change {
  eventId: number;
  unit: string;
  kind: ChangeKind;
  /** What the card shows: "−5", "+2", "+1 PWR", "Poison ×2", "✝". */
  label: string;
}

const CHANGE_TYPES = new Set(["Hurt", "Heal", "StatChanged", "StatusApplied", "Death", "Summon", "Silenced"]);

/** A hit as a change reads: "−n", or for a hit that did nothing, what
 * stopped it ("3 blocked" by Shield, or "no damage"), never "−0". */
export function hurtLabel(amount: number, absorbed?: number): string {
  if (amount > 0) return `−${amount}`;
  return absorbed ? `${absorbed} blocked` : "no damage";
}

export function changeOf(e: BattleEvent): Change | null {
  if (!CHANGE_TYPES.has(e.type)) return null;
  switch (e.type) {
    case "Hurt":
      return { eventId: e.id, unit: e.unit, kind: "damage", label: hurtLabel(e.amount, e.absorbed) };
    case "Heal":
      return { eventId: e.id, unit: e.unit, kind: "heal", label: `+${e.amount}` };
    case "StatChanged":
      return { eventId: e.id, unit: e.unit, kind: e.delta >= 0 ? "buff" : "debuff", label: `${e.delta >= 0 ? "+" : "−"}${Math.abs(e.delta)} ${e.stat.toUpperCase()}` };
    case "StatusApplied":
      return { eventId: e.id, unit: e.unit, kind: "status", label: `${e.status} ×${e.stacks}` };
    case "Death":
      return { eventId: e.id, unit: e.unit, kind: "death", label: "✝" };
    case "Summon":
      return { eventId: e.id, unit: e.unit, kind: "summon", label: e.resurrected ? "returns" : "new" };
    case "Silenced":
      return { eventId: e.id, unit: e.unit, kind: "silence", label: "silenced" };
    default:
      return null;
  }
}

export interface TraceLink {
  /** The event this unit acted through. */
  eventId: number;
  unit: string;
  name: string;
  side: Side | null;
  /** "strike", "ability", or the status it acted through ("Poison"). */
  via: string;
}

export interface Trace {
  eventId: number;
  change: Change | null;
  /** Nearest cause first. Each unit appears once: consecutive acts by one
   * unit collapse to one link, and a loop (Virus's Poison sets off
   * Guardian's Shield, which sets off Virus again, turn after turn) reads
   * once, the walk going on past it to whoever started it (R2-17). */
  links: TraceLink[];
  /** "−5 ← Archer ← Smith ← Shieldbearer" */
  text: string;
}

const MAX_HOPS = 64;

/** Walk an event's causes up `causedBy` (and status origins) to the turn's
 * beat, naming each unit that acted along the way. */
export function traceOf(log: BattleEvent[], eventId: number, name: NameOf = displayNames(log), sides = sidesOf(log)): Trace {
  const links: TraceLink[] = [];
  let cur: BattleEvent | undefined = log[eventId];
  for (let hops = 0; cur && hops < MAX_HOPS; hops++) {
    if (cur.type !== "Strike" && isRootKind(cur.type)) break; // turn structure ends the story
    const a = actorOf(log, cur);
    if (a?.unit) {
      if (!links.some((l) => l.unit === a.unit)) {
        links.push({ eventId: cur.id, unit: a.unit, name: name(a.unit), side: sides.get(a.unit) ?? null, via: a.via });
      }
    }
    const next: number | null = a?.jumpTo ?? cur.causedBy;
    cur = next !== null && next < cur.id ? log[next] : undefined;
  }
  const change = log[eventId] ? changeOf(log[eventId]!) : null;
  const head = change?.label ?? log[eventId]?.type ?? "?";
  const text = links.length ? [head, ...links.map(linkText)].join(" ← ") : `${head} ← ${rootText(log, eventId)}`;
  return { eventId, change, links, text };
}

function linkText(l: TraceLink): string {
  return l.via === "strike" || l.via === "ability" ? l.name : `${l.name} (${l.via})`;
}

/** What a trace with no acting unit came from: fatigue, or the battle's start. */
function rootText(log: BattleEvent[], eventId: number): string {
  let cur = log[eventId];
  for (let hops = 0; cur && hops < MAX_HOPS; hops++) {
    if (cur.type === "Fatigue") return "Fatigue";
    if (cur.causedBy === null) break;
    cur = log[cur.causedBy];
  }
  return cur?.type === "BattleStart" ? "battle start" : "the rules";
}

// ---------- Why: the chain from a change back to the turn (round 2, R2-15) ----------

/** The When a firing answered, looked up by its stamped ref (AbilityRef.when)
 * in the holder's recipe or the status's def. The client builds it from the
 * battle's units and content; without it the trigger is read off the event
 * that set the firing off, which is what the When matched. */
export type WhenOf = (ref: AbilityRef) => When | undefined;

/** One step of a Why chain.
 * - change: the clicked change ("Victim → Strength ×1 on Victim (+1 PWR)");
 * - firing: the ability that made the step below it (its holder, the When it
 *   answered, which ability); the client draws its When → Does from the unit;
 * - event: the event that set the firing below it off ("Medic strikes Victim → −1");
 * - root: where it all started ("Turn 6", "Battle start", "Fatigue").
 * Every step has an eventId, so a click can move the playhead there. */
export interface ChainNode {
  kind: "change" | "firing" | "event" | "root";
  /** The log event's type, for an icon when no trigger answers it (Fatigue, Intercepted). */
  event: BattleEvent["type"];
  eventId: number;
  /** One line: a caption for a change or an event, the reactor for a firing, the turn for a root. */
  text: string;
  /** The unit it is about: the firing's holder, else the caption's subject. */
  unit: string | null;
  side: Side | null;
  /** The trigger icon the step shows: a firing's When ("trigger:Hurt"), the
   * kind of an event or root ("trigger:Strike", "trigger:TurnStart"). */
  trigger: `trigger:${string}` | null;
  /** The status a StatusApplied / StatusRemoved trigger is about; an
   * Intercepted step's, the status that stopped it (Freeze). */
  triggerStatus?: string;
  /** firing only: whose event the When watched ("otherAlly" reads "Ally dies"). */
  triggerScope?: UnitFilter;
  /** firing only: the reactor, its `when` the index of the When that fired. */
  ref?: AbilityRef;
  /** firing of a status (a Poison tick): the status, and where it was put on. */
  status?: string;
  origin?: number;
}

export interface Chain {
  eventId: number;
  change: Change | null;
  /** The change first, then back to the root (or as far as the log goes). */
  nodes: ChainNode[];
}

/** The trigger an event is, for its icon; null for one no When can answer. */
function eventTrigger(e: BattleEvent): Pick<ChainNode, "trigger" | "triggerStatus"> {
  if (!TRIGGER_EVENTS.has(e.type)) return { trigger: null };
  return { trigger: `trigger:${e.type}`, ...(e.type === "StatusApplied" || e.type === "StatusRemoved" ? { triggerStatus: e.status } : {}) };
}

/** The trigger a firing answered: its stamped When when there is one to look
 * up, else the event that set it off (a log from before the stamp). */
function firedTrigger(log: BattleEvent[], e: BattleEvent, whenOf?: WhenOf): Pick<ChainNode, "trigger" | "triggerStatus" | "triggerScope"> {
  if (e.source !== "kernel" && e.source.when !== undefined) {
    const w = whenOf?.(e.source);
    if (w) {
      const status = "status" in w.on ? w.on.status : undefined;
      const scope = "unit" in w.on ? w.on.unit : "striker" in w.on ? w.on.striker : undefined;
      const cause = e.causedBy !== null ? log[e.causedBy] : undefined;
      const seen = cause && (cause.type === "StatusApplied" || cause.type === "StatusRemoved") ? cause.status : undefined;
      return { trigger: `trigger:${w.on.on}`, ...((status ?? seen) ? { triggerStatus: (status ?? seen)! } : {}), ...(scope ? { triggerScope: scope } : {}) };
    }
  }
  const cause = e.causedBy !== null ? log[e.causedBy] : undefined;
  return cause ? eventTrigger(cause) : { trigger: null };
}

function rootNodeText(e: BattleEvent): string {
  switch (e.type) {
    case "TurnStart": return `Turn ${e.turn}`;
    case "TurnEnd": return `Turn ${e.turn} ends`;
    case "BattleStart": return "Battle start";
    case "Fatigue": return `Fatigue (turn ${e.turn})`;
    case "PairFaced": return `Turn ${e.turn}`;
    default: return e.type;
  }
}

/** Why `eventId` happened, as steps from the change back to the turn: the
 * change ← the ability that fired (holder, When) ← the event that set it off
 * ← … ← the turn or the battle's start. A stat change a status brought folds
 * into that status's step; a strike's hit reads as one step with its strike;
 * a status's tick (Poison) continues where the status was put on. */
export function chainOf(log: BattleEvent[], eventId: number, o: { name?: NameOf; sides?: Map<string, Side>; whenOf?: WhenOf } = {}): Chain {
  const name = o.name ?? displayNames(log);
  const sides = o.sides ?? sidesOf(log);
  const side = (u: string | null) => (u ? sides.get(u) ?? null : null);
  const nodes: ChainNode[] = [];
  const seen = new Set<number>();
  let cur: BattleEvent | undefined = log[eventId];
  for (let hops = 0; cur && hops < MAX_HOPS && !seen.has(cur.id); hops++) {
    seen.add(cur.id);
    if (cur.type !== "Strike" && isRootKind(cur.type)) {
      nodes.push({ event: cur.type, kind: "root", eventId: cur.id, text: rootNodeText(cur), unit: null, side: null, ...eventTrigger(cur) });
      break;
    }
    const kind = nodes.length ? "event" : "change";
    if (cur.type === "Strike") {
      // A strike's own hit (the kernel's Hurt) already reads "X strikes Y →
      // −n"; a strike that set a firing off reads on its own, even when that
      // firing's change is a Hurt too (a Strike-When unit's damage, R2-17).
      const prev = nodes.at(-1);
      const hit = prev && prev.kind !== "firing" ? log[prev.eventId] : undefined;
      if (!(hit && hit.causedBy === cur.id && hit.type === "Hurt" && hit.source === "kernel")) {
        nodes.push({ event: cur.type, kind, eventId: cur.id, text: `${name(cur.striker)} strikes ${name(cur.defender)}`, unit: cur.striker, side: side(cur.striker), ...eventTrigger(cur) });
      }
    } else if (isStatusFollowUp(log, cur)) {
      // "+1 PWR" is the stat side of a status landing: one step, the status's caption.
      const parent = log[cur.causedBy!]!;
      const subject = captionSubject(log, parent.id, name);
      nodes.push({ event: cur.type, kind, eventId: cur.id, text: captionOf(log, parent.id, name, [parent.id, cur.id]), unit: subject, side: side(subject), ...eventTrigger(cur) });
      seen.add(parent.id);
      cur = parent;
    } else {
      const subject = captionSubject(log, cur.id, name);
      nodes.push({ event: cur.type, kind, eventId: cur.id, text: captionOf(log, cur.id, name), unit: subject, side: side(subject), ...eventTrigger(cur), ...(cur.type === "Intercepted" && cur.by.status ? { triggerStatus: cur.by.status } : {}) });
    }
    let next: number | null = cur.causedBy;
    if (cur.source !== "kernel") {
      const src: AbilityRef = cur.source;
      const fired = firedTrigger(log, cur, o.whenOf);
      if (src.status === undefined) {
        nodes.push({ event: cur.type, kind: "firing", eventId: cur.id, text: `${name(src.unit)}'s ability`, unit: src.unit, side: side(src.unit), ...fired, ref: src });
      } else {
        const origin = statusOriginId(log, src.unit, src.status, cur.id);
        nodes.push({ event: cur.type, kind: "firing", eventId: cur.id, text: `${src.status} on ${name(src.unit)}`, unit: src.unit, side: side(src.unit), ...fired, ref: src, status: src.status, ...(origin !== undefined ? { origin } : {}) });
        if (origin !== undefined) next = origin;
      }
    }
    cur = next !== null && next < cur.id ? log[next] : undefined;
  }
  const change = log[eventId] ? changeOf(log[eventId]!) : null;
  return { eventId, change, nodes };
}

// ---------- playback steps and captions ----------

export interface Step {
  /** Events this step reveals; the board after it is boardAt(log, last id). */
  eventIds: number[];
  turn: number;
  /** The unit that lights up while this step plays. */
  actor: string | null;
  actorSide: Side | null;
  /** The unit the caption is about: the first one it names ("Rose" in
   * "Freeze on Rose → stops its strike"), which may not be the actor (the
   * Freeze came from an enemy). Null when it names none (fatigue, the end). */
  subject: string | null;
  subjectSide: Side | null;
  /** One line, cause → effect. */
  caption: string;
  changes: Change[];
}

/** Events that are pure bookkeeping for playback: turn structure, and the
 * strike itself (its Hurt carries the caption "X strikes Y → −n"). */
const SILENT = new Set(["BattleStart", "TurnStart", "TurnEnd", "PairFaced", "Strike", "ChainBlocked"]);

/** A StatChanged the kernel logs as the follow-up of a status landing or
 * leaving belongs to that status's step. */
function isStatusFollowUp(log: BattleEvent[], e: BattleEvent): boolean {
  if (e.type !== "StatChanged" || e.source !== "kernel" || e.causedBy === null) return false;
  const p = log[e.causedBy];
  return p?.type === "StatusApplied" || p?.type === "StatusRemoved";
}

/** Whose eyes the captions use: `you` reads the end as "You win" / "They
 * win"; without it (a playoff game, a champion's battle) the end names the
 * winning side, by `sideName` when given ("@alice wins"), else "Side A wins". */
export interface Perspective {
  you?: Side;
  sideName?: (side: Side) => string;
}

/** How the battle's end reads from `p`. */
export function endCaption(winner: Side | "draw", p: Perspective = {}): string {
  if (winner === "draw") return "Draw";
  if (p.you) return winner === p.you ? "You win" : "They win";
  return `${p.sideName ? p.sideName(winner) : `Side ${winner}`} wins`;
}

export function stepsOf(log: BattleEvent[], name: NameOf = displayNames(log), sides = sidesOf(log), p: Perspective = {}): Step[] {
  const steps: Step[] = [];
  const byEvent = new Map<number, Step>();
  for (const e of log) {
    if (SILENT.has(e.type)) continue;
    if (isStatusFollowUp(log, e)) {
      const parent = byEvent.get(e.causedBy!);
      if (parent) {
        parent.eventIds.push(e.id);
        const c = changeOf(e);
        if (c) parent.changes.push(c);
        parent.caption = captionOf(log, e.causedBy!, name, parent.eventIds, p);
        byEvent.set(e.id, parent);
        continue;
      }
    }
    const a = actorOf(log, e);
    const actor = a?.unit ?? traceOf(log, e.id, name, sides).links[0]?.unit ?? null;
    const c = changeOf(e);
    const subject = captionSubject(log, e.id, name);
    const step: Step = {
      eventIds: [e.id],
      turn: e.turn,
      actor,
      actorSide: actor ? sides.get(actor) ?? null : null,
      subject,
      subjectSide: subject ? sides.get(subject) ?? null : null,
      caption: captionOf(log, e.id, name, [e.id], p),
      changes: c ? [c] : [],
    };
    steps.push(step);
    byEvent.set(e.id, step);
  }
  return steps;
}

// ---------- beats: one strike (or turn end) and everything it sets off ----------

/** A beat as the viewer plays it (round 2, R2-12): one root event (a strike,
 * a turn end, fatigue, …) plus its whole cascade, in quick waves. A wave is a
 * Step: one firing's same-kind effects landing together, so a buff on all
 * allies is one wave, and a status that leaves because of a hit ("Shield
 * absorbs 1") folds into that hit. Beats with nothing to show are dropped. */
export interface PlayBeat {
  index: number;
  turn: number;
  /** The waves, in log order; never empty. */
  waves: Step[];
  /** Every event id the beat reveals; the board after it is boardAt(log, end). */
  end: number;
  /** The board before the beat: boardAt(log, start - 1). */
  start: number;
}

/** Milliseconds between waves at 1×, the shortest a beat lasts, and the cap. */
export const WAVE_MS = 150;
export const BEAT_MS = 1000;
export const BEAT_MAX_MS = 1400;
/** A quiet beat (one wave, a plain hit or a status tick, nothing dies) is
 * shorter: a −1 trade shouldn't take as long as a kill (pacing by weight). */
export const QUIET_BEAT_MS = 700;
/** How long the last wave stays before the next beat. */
const BEAT_HOLD_MS = 700;

/** When each wave lands (ms from the beat's start, at 1×) and how long the
 * beat lasts: waves 150 ms apart, squeezed so the last lands by 700 ms; the
 * beat lasts 1 s, up to 1.4 s for a long cascade, and 0.7 s when quiet
 * (tightened from 1.2/1.5 s so a median battle plays in about 25 s). */
export function beatTiming(waves: number, quiet = false): { at: number[]; ms: number } {
  if (quiet && waves <= 1) return { at: [0], ms: QUIET_BEAT_MS };
  const span = BEAT_MAX_MS - BEAT_HOLD_MS;
  const gap = waves > 1 ? Math.min(WAVE_MS, span / (waves - 1)) : 0;
  const at = Array.from({ length: waves }, (_, i) => Math.round(i * gap));
  return { at, ms: Math.min(BEAT_MAX_MS, Math.max(BEAT_MS, (at.at(-1) ?? 0) + BEAT_HOLD_MS)) };
}

/** A beat's timing at 1×, by its weight: one wave of plain hits, status
 * changes or nothing at all is quiet; anything more plays full length. */
export function timingOf(beat: PlayBeat): { at: number[]; ms: number } {
  const quiet = beat.waves.length === 1 && beat.waves[0]!.changes.every((c) => c.kind === "damage" || c.kind === "status");
  return beatTiming(beat.waves.length, quiet);
}

/** What a trigger badge shows (R2-13): the trigger a unit's ability answered
 * and what the ability did, as glossary terms for their icons. */
export interface Firing {
  unit: string;
  /** "trigger:Hurt", or null when the cause isn't a trigger event (a status's own tick). */
  trigger: `trigger:${string}` | null;
  /** The status a StatusApplied / StatusRemoved trigger was about. */
  triggerStatus?: string;
  /** Whose event the When watched, for its scoped label ("Ally dies"). */
  triggerScope?: UnitFilter;
  /** "effect:damage", "status:Strength", "stat:pwr", … */
  effect: string;
  /** The status the effect put on, for its icon and colour. */
  effectStatus?: string;
}

/** The kernel events a unit trigger can answer (glossary trigger:<type>). */
const TRIGGER_EVENTS = new Set(["BattleStart", "TurnStart", "TurnEnd", "Strike", "Hurt", "Heal", "Death", "Summon", "StatusApplied", "StatusRemoved", "StatChanged"]);

/** The unit ability that made this step, if one did: its holder, the event
 * that set it off and the step's first change. A status's own ability
 * (Shield blocking, Poison ticking) and the kernel's strikes have none.
 * The trigger is the When the kernel stamped (AbilityRef.when, R2-15), looked
 * up with `whenOf`; without one, the cause's event type, which it matched. */
export function firingOf(log: BattleEvent[], step: Pick<Step, "eventIds" | "changes">, whenOf?: WhenOf): Firing | null {
  const first = log[step.eventIds[0]!];
  if (!first || first.source === "kernel" || first.source.status) return null;
  const c = step.changes[0];
  if (!c) return null;
  const fired = firedTrigger(log, first, whenOf);
  const eff = effectOf(log[c.eventId]);
  if (!eff) return null;
  return { unit: first.source.unit, ...fired, ...eff };
}

/** What a change event did, as a glossary term for its icon. */
function effectOf(e: BattleEvent | undefined): Pick<Firing, "effect" | "effectStatus"> | null {
  switch (e?.type) {
    case "Hurt": return { effect: "effect:damage" };
    case "Heal": return { effect: "effect:heal" };
    case "StatusApplied": return { effect: `status:${e.status}`, effectStatus: e.status };
    case "StatusRemoved": return { effect: `status:${e.status}`, effectStatus: e.status };
    case "StatChanged": return { effect: `stat:${e.stat}` };
    case "Summon": return { effect: e.resurrected ? "effect:resurrect" : "effect:summon" };
    case "Silenced": return { effect: "effect:silence" };
    case "Death": return { effect: "effect:damage" };
    case "Fatigue": return { effect: "effect:damage" };
    case "Intercepted": return { effect: "effect:cancel" };
    case "ChainCapped": return { effect: "battle:chainCapped" };
    default: return null;
  }
}

/** Why a wave happens, for its badge (round 3, R3-19, battle.md (10)):
 * where the badge sits and the cause → effect it shows.
 * - a unit's ability: on the unit, the When it answered (as firingOf);
 * - a strike's hit: on the striker, crossed swords (trigger:Strike);
 * - a status acting (a Poison tick, Freeze stopping a strike, Blessing
 *   saving a unit): on its holder, the status's own icon (status:Poison);
 * - fatigue, a capped chain: at the clash, its battle term;
 * - a death, or any other follow-up the rules log: the cause of what led to
 *   it, so the killing wave's badge stays.
 * Only the battle's end has none. */
export interface Cause {
  /** The unit the badge sits on, or "clash" (fatigue, a capped chain). */
  at: string;
  kind: "ability" | "strike" | "status" | "battle";
  /** "trigger:Hurt", "trigger:Strike", "status:Poison", "battle:fatigue". */
  cause: `trigger:${string}` | `status:${string}` | `battle:${string}`;
  /** The status a StatusApplied / StatusRemoved trigger was about. */
  causeStatus?: string;
  /** Whose event the When watched (an ability's), for its scoped label. */
  causeScope?: UnitFilter;
  /** What the wave did: "effect:damage", "status:Strength", "effect:cancel", … */
  effect: string;
  effectStatus?: string;
}

export function causeOf(log: BattleEvent[], step: Pick<Step, "eventIds" | "changes">, whenOf?: WhenOf): Cause | null {
  const first = log[step.eventIds[0]!];
  if (!first) return null;
  const c = step.changes[0];
  const why = causeOfEvent(log, first, whenOf);
  if (!why) return null;
  // A death reads as the wave that killed (its own change is only the ✝).
  const eff = first.type === "Death" ? null : effectOf(c ? log[c.eventId] : first);
  return eff ? withEffect(why, eff) : why;
}

/** A cause with another effect: the wave's own, not the one up its chain. */
function withEffect(c: Cause, eff: Pick<Firing, "effect" | "effectStatus">): Cause {
  const { effect: _e, effectStatus: _s, ...rest } = c;
  return { ...rest, effect: eff.effect, ...(eff.effectStatus ? { effectStatus: eff.effectStatus } : {}) };
}

function causeOfEvent(log: BattleEvent[], e: BattleEvent, whenOf?: WhenOf, hops = 0): Cause | null {
  if (e.type === "BattleEnd") return null;
  const eff = effectOf(e) ?? { effect: "effect:damage" };
  const base = { effect: eff.effect, ...(eff.effectStatus ? { effectStatus: eff.effectStatus } : {}) };
  if (e.source !== "kernel") {
    if (e.source.status !== undefined) return { at: e.source.unit, kind: "status", cause: `status:${e.source.status}`, ...base };
    const f = firedTrigger(log, e, whenOf);
    return {
      at: e.source.unit,
      kind: "ability",
      cause: f.trigger ?? "trigger:Strike",
      ...(f.triggerStatus ? { causeStatus: f.triggerStatus } : {}),
      ...(f.triggerScope ? { causeScope: f.triggerScope } : {}),
      ...base,
    };
  }
  if (e.type === "Strike") return { at: e.striker, kind: "strike", cause: "trigger:Strike", effect: "effect:damage" };
  if (e.type === "Fatigue") return { at: "clash", kind: "battle", cause: "battle:fatigue", ...base };
  if (e.type === "ChainCapped") return { at: "clash", kind: "battle", cause: "battle:chainCapped", ...base };
  const parent = e.causedBy !== null && e.causedBy < e.id ? log[e.causedBy] : undefined;
  if (parent && hops < MAX_HOPS) {
    if (parent.type === "Strike") return { at: parent.striker, kind: "strike", cause: "trigger:Strike", ...base };
    if (parent.type === "Fatigue") return { at: "clash", kind: "battle", cause: "battle:fatigue", ...base };
    if (!isRootKind(parent.type)) {
      const up = causeOfEvent(log, parent, whenOf, hops + 1);
      if (up) return e.type === "Death" ? up : withEffect(up, eff);
    }
    // The rules acting on a unit at a turn's start or end (a status wearing off).
    if (TRIGGER_EVENTS.has(parent.type)) {
      const unit = "unit" in e && typeof e.unit === "string" ? e.unit : "clash";
      return { at: unit, kind: "battle", cause: `trigger:${parent.type}`, ...base };
    }
  }
  return { at: "unit" in e && typeof e.unit === "string" ? e.unit : "clash", kind: "battle", cause: "trigger:TurnEnd", ...base };
}

/** Which firing made a step: its first event's parent and source. Steps of
 * one firing and one kind land in the same wave. */
function waveKey(log: BattleEvent[], s: Step): string {
  const e = log[s.eventIds[0]!]!;
  const src = e.source === "kernel" ? "kernel" : `${e.source.unit}/${e.source.status ?? ""}/${e.source.ability}`;
  return `${e.causedBy}|${src}|${e.type}|${s.changes[0]?.label ?? ""}`;
}

/** One caption for several targets of one firing: "Coach → Strength ×1 on Rose, Ace". */
function groupCaption(first: Step, names: string[], label: string): string {
  const cause = first.caption.split(" → ")[0];
  return `${cause} → ${label} on ${names.join(", ")}`;
}

export function beatPlayOf(log: BattleEvent[], steps: Step[], name: NameOf = displayNames(log)): PlayBeat[] {
  // 1. A status leaving because of a hit or a heal folds into that step.
  const byEvent = new Map<number, Step>();
  const kept: Step[] = [];
  for (const s of steps) {
    const first = log[s.eventIds[0]!]!;
    const parent = first.type === "StatusRemoved" && first.causedBy !== null ? byEvent.get(first.causedBy) : undefined;
    if (parent && first.type === "StatusRemoved") {
      parent.eventIds.push(...s.eventIds);
      parent.changes.push(...s.changes);
      // A Shield spent on a hit is already in its caption ("Shield blocks n", "(n absorbed)").
      const spent = log[first.causedBy!];
      if (!(first.status === "Shield" && spent?.type === "Hurt" && spent.absorbed)) parent.caption += `, ${first.status} −${first.stacks}`;
      for (const id of s.eventIds) byEvent.set(id, parent);
      continue;
    }
    const copy: Step = { ...s, eventIds: [...s.eventIds], changes: [...s.changes] };
    kept.push(copy);
    for (const id of copy.eventIds) byEvent.set(id, copy);
  }
  // 2. Steps fall into the log's beats; one firing's same-kind effects form one wave.
  const beats = beatsOf(log);
  const beatOf = new Map<number, number>();
  for (const b of beats) for (let id = b.start; id <= b.end; id++) beatOf.set(id, b.index);
  const out: PlayBeat[] = [];
  let cur: { b: number; waves: Step[]; keys: string[]; names: string[][] } | null = null;
  const flush = () => {
    if (!cur) return;
    const b = beats[cur.b]!;
    out.push({ index: out.length, turn: cur.waves[0]!.turn, waves: cur.waves, start: b.start, end: b.end });
  };
  for (const s of kept) {
    const b = beatOf.get(s.eventIds[0]!) ?? -1;
    if (!cur || cur.b !== b) {
      flush();
      cur = { b, waves: [], keys: [], names: [] };
    }
    const key = waveKey(log, s);
    const last = cur.waves.length - 1;
    const prev = cur.waves[last];
    if (prev && cur.keys[last] === key && s.changes.length && prev.changes[0]?.unit !== s.changes[0]?.unit) {
      prev.eventIds.push(...s.eventIds);
      prev.changes.push(...s.changes);
      cur.names[last]!.push(name(s.changes[0]!.unit));
      prev.caption = groupCaption(prev, cur.names[last]!, s.changes[0]!.label);
      // A wave whose captions are about units of both sides (fatigue hitting
      // every front) has no one side: its tag would mislabel half the names,
      // which carry their own side's colour anyway (R2-17).
      if (s.subjectSide !== prev.subjectSide) {
        prev.subject = null;
        prev.subjectSide = null;
      }
      continue;
    }
    cur.waves.push(s);
    cur.keys.push(key);
    cur.names.push(s.changes[0] ? [name(s.changes[0].unit)] : []);
  }
  flush();
  return out;
}

function causeName(log: BattleEvent[], e: BattleEvent, name: NameOf): string {
  const a = actorOf(log, e);
  if (a?.unit) return a.via === "strike" || a.via === "ability" ? name(a.unit) : a.via;
  const t = traceOf(log, e.id, name);
  return t.links[0] ? linkText(t.links[0]) : rootText(log, e.id);
}

/** The unit causeName() names, or null when it names a status or the rules. */
function causeUnit(log: BattleEvent[], e: BattleEvent, name: NameOf): string | null {
  const a = actorOf(log, e);
  if (a?.unit) return a.via === "strike" || a.via === "ability" ? a.unit : null;
  if (a) return null;
  return traceOf(log, e.id, name).links[0]?.unit ?? null;
}

/** The first unit captionOf(id) names: the caption's side tag is that unit's side. */
export function captionSubject(log: BattleEvent[], id: number, name: NameOf = displayNames(log)): string | null {
  const e = log[id];
  if (!e) return null;
  switch (e.type) {
    case "Hurt": {
      const p = e.causedBy !== null ? log[e.causedBy] : undefined;
      if (e.source === "kernel" && p?.type === "Strike") return p.striker;
      return causeUnit(log, e, name) ?? e.unit;
    }
    case "Heal":
    case "StatusApplied":
    case "StatChanged":
    case "Death":
    case "Silenced":
      return causeUnit(log, e, name) ?? e.unit;
    case "Summon":
      return causeUnit(log, e, name) ?? e.unit;
    case "StatusRemoved":
      return e.unit;
    case "Intercepted":
      return e.by.status && e.unit === e.by.unit ? e.unit : e.by.unit;
    default:
      return null;
  }
}

/** What an interceptor stopped, as words: "its strike", "Rose's strike", "a hit on Rose". */
function stoppedText(original: string, unit: string | undefined, own: boolean, name: NameOf): string {
  const n = unit ? name(unit) : "";
  const whose = own ? "its" : `${n}'s`;
  switch (original) {
    case "Strike":
      return unit ? `${whose} strike` : "a strike";
    case "Death":
      return unit ? `${whose} death` : "a death";
    case "Hurt":
      return unit ? (own ? "the hit" : `a hit on ${n}`) : "a hit";
    case "Heal":
      return unit ? (own ? "the heal" : `a heal on ${n}`) : "a heal";
    case "StatusApplied":
      return unit ? (own ? "the status" : `a status on ${n}`) : "a status";
    default:
      return `a ${original}${unit && !own ? ` on ${n}` : ""}`;
  }
}

/** One line, cause → effect, for the event `id` (plus merged follow-ups). */
export function captionOf(log: BattleEvent[], id: number, name: NameOf = displayNames(log), merged: number[] = [id], p: Perspective = {}): string {
  const e = log[id];
  if (!e) return "";
  switch (e.type) {
    case "Hurt": {
      const p = e.causedBy !== null ? log[e.causedBy] : undefined;
      // A hit Shield took whole reads "Shield blocks n"; one that did nothing else, "no damage"; never "−0".
      const absorbed = e.absorbed ? ` (${e.absorbed} absorbed)` : "";
      const effect = e.amount > 0 ? `−${e.amount}${absorbed}` : e.absorbed ? `Shield blocks ${e.absorbed}` : "no damage";
      if (e.source === "kernel" && p?.type === "Strike") return `${name(p.striker)} strikes ${name(e.unit)} → ${effect}`;
      return `${causeName(log, e, name)} → ${name(e.unit)} ${effect}`;
    }
    case "Heal":
      return `${causeName(log, e, name)} → ${name(e.unit)} +${e.amount}`;
    case "StatusApplied": {
      const stat = merged
        .map((i) => log[i])
        .flatMap((m) => (m && m.type === "StatChanged" ? [`${m.delta >= 0 ? "+" : "−"}${Math.abs(m.delta)} ${m.stat.toUpperCase()}`] : []));
      return `${causeName(log, e, name)} → ${e.status} ×${e.stacks} on ${name(e.unit)}${stat.length ? ` (${stat.join(", ")})` : ""}`;
    }
    case "StatusRemoved":
      return `${e.status} fades from ${name(e.unit)}${e.remaining > 0 ? ` (${e.remaining} left)` : ""}`;
    case "StatChanged":
      return `${causeName(log, e, name)} → ${name(e.unit)} ${e.delta >= 0 ? "+" : "−"}${Math.abs(e.delta)} ${e.stat.toUpperCase()}`;
    case "Death":
      return `${causeName(log, e, name)} → ${name(e.unit)} falls`;
    case "Summon":
      return e.resurrected
        ? `${causeName(log, e, name)} → ${name(e.unit)} returns at ${e.atHp ?? e.hp} HP`
        : `${causeName(log, e, name)} → ${e.name} appears (${e.pwr}/${e.hp})`;
    case "Silenced":
      return `${causeName(log, e, name)} → ${name(e.unit)} is silenced`;
    case "Fatigue":
      return `Fatigue → everyone takes ${e.amount}`;
    case "ChainCapped":
      return `Chain stopped after ${e.steps} steps`;
    case "Intercepted":
      // A status on the unit stopping its own act reads from the unit:
      // "Freeze on Rose → stops its strike", not "Rose (Freeze) → … on Rose".
      if (e.by.status && e.unit === e.by.unit) return `${e.by.status} on ${name(e.unit)} → stops ${stoppedText(e.original, e.unit, true, name)}`;
      return `${name(e.by.unit)}${e.by.status ? ` (${e.by.status})` : ""} → stops ${stoppedText(e.original, e.unit, false, name)}`;
    case "BattleEnd":
      return endCaption(e.winner, p);
    default:
      return e.type;
  }
}

// ---------- why I lost ----------

export interface LossChain {
  /** Unit names, nearest cause first: ["Archer", "Smith"]. */
  names: string[];
  /** The unit instances behind it, nearest first: ["B3:Archer", "B2:Smith"].
   * Chains group by instance, so two Medics are two chains. */
  units: string[];
  /** "Archer ← Smith" (with the status when one carried it). When two chains
   * would read the same (two Medics), each first name gets its slot: "Medic #2". */
  text: string;
  /** Damage it dealt to your units (overkill not counted), plus healing it gave theirs. */
  impact: number;
  damage: number;
  heal: number;
  kills: number;
  /** How many changes it made (hits on your units plus heals on theirs). */
  times: number;
  hits: number;
  heals: number;
  /** The change that mattered most, to replay its trace: its biggest killing
   * blow, else its biggest hit, else its biggest heal; never a "−0" hit a
   * Shield took whole. Only when every change came to nothing, the first one. */
  sampleEventId: number;
  /** What sampleEventId is: "kill", "hit", "heal", or "none" (all came to nothing). */
  sampleKind: "kill" | "hit" | "heal" | "none";
}

/** "B2:Medic" → 2: the slot a unit started in; null for a summon or another id. */
function slotOf(id: string): number | null {
  const m = /^[AB](\d+):/.exec(id);
  return m ? Number(m[1]) : null;
}

/** The enemy chains that did the most against `you` (default side A): every
 * Hurt on your units and Heal on theirs is traced, grouped by the chain of
 * units behind it, and ranked by impact. Fatigue and your own units' acts
 * are not enemy chains. */
export function whyILost(log: BattleEvent[], you: Side = "A", top = 3): LossChain[] {
  const name = displayNames(log);
  const sides = sidesOf(log);
  const them: Side = you === "A" ? "B" : "A";
  const hp = new Map<string, number>();
  for (const e of log) if (e.type === "BattleStart") for (const s of ["A", "B"] as const) for (const r of e.teams[s]) hp.set(r.id, r.hp);
  const groups = new Map<string, LossChain>();
  // Each group's best sample so far: a kill beats a hit beats a heal, then the bigger one.
  const best = new Map<LossChain, { rank: number; amount: number }>();
  const RANK = { none: 0, heal: 1, hit: 2, kill: 3 } as const;
  const offer = (g: LossChain, id: number, kind: LossChain["sampleKind"], amount: number) => {
    const b = best.get(g);
    if (b && (RANK[kind] < b.rank || (RANK[kind] === b.rank && amount <= b.amount))) return;
    best.set(g, { rank: RANK[kind], amount });
    g.sampleEventId = id;
    g.sampleKind = kind;
  };
  // The damage each traced hit dealt, so a killing blow is ranked by its size.
  const dealt = new Map<number, number>();
  // By instance and how it reads: a unit's strikes and abilities are one row
  // ("Medic"), its status ticks another ("Zealot (Poison)").
  const keyOf = (t: Trace) => t.links.map((l) => `${l.unit}|${linkText(l)}`).join(" ← ");
  const add = (e: BattleEvent, dmg: number, heal: number, kill: boolean) => {
    const t = traceOf(log, e.id, name, sides);
    if (t.links[0]?.side !== them) return;
    const key = keyOf(t);
    let g = groups.get(key);
    if (!g) {
      g = { names: t.links.map((l) => l.name), units: t.links.map((l) => l.unit), text: t.links.map(linkText).join(" ← "), impact: 0, damage: 0, heal: 0, kills: 0, times: 0, hits: 0, heals: 0, sampleEventId: e.id, sampleKind: "none" };
      groups.set(key, g);
    }
    if (dmg > 0) offer(g, e.id, "hit", dmg);
    else if (heal > 0) offer(g, e.id, "heal", heal);
    else offer(g, e.id, "none", 0);
    dealt.set(e.id, dmg);
    g.damage += dmg;
    g.heal += heal;
    g.impact += dmg + heal;
    g.kills += kill ? 1 : 0;
    g.times++;
    if (heal > 0) g.heals++;
    else g.hits++;
  };
  for (const e of log) {
    if (e.type === "Hurt") {
      const before = hp.get(e.unit) ?? e.amount;
      const after = e.hpAfter ?? before - e.amount;
      hp.set(e.unit, after);
      if (sides.get(e.unit) === you) add(e, Math.max(0, Math.min(e.amount, before)), 0, false);
    } else if (e.type === "Heal") {
      const before = hp.get(e.unit) ?? 0;
      hp.set(e.unit, e.hpAfter ?? before + e.amount);
      if (sides.get(e.unit) === them) add(e, 0, e.amount, false);
    } else if (e.type === "StatChanged" && e.hpAfter !== undefined) {
      hp.set(e.unit, e.hpAfter);
    } else if (e.type === "Summon") {
      hp.set(e.unit, e.atHp ?? e.hp);
    } else if (e.type === "Death" && sides.get(e.unit) === you) {
      // the kill belongs to the chain of the change that dropped the unit
      const cause = e.causedBy !== null ? log[e.causedBy] : undefined;
      if (cause) {
        const t = traceOf(log, cause.id, name, sides);
        const g = groups.get(keyOf(t));
        if (g && t.links[0]?.side === them) {
          g.kills++;
          offer(g, cause.id, "kill", dealt.get(cause.id) ?? 0);
        }
      }
    }
  }
  const all = [...groups.values()];
  // Two instances with one name (two Medics) read the same: name the first by its slot.
  const seen = new Map<string, number>();
  for (const g of all) seen.set(g.text, (seen.get(g.text) ?? 0) + 1);
  const summons = new Map<string, number>();
  for (const g of all) {
    if (seen.get(g.text)! < 2) continue;
    const slot = slotOf(g.units[0]!);
    const k = summons.get(g.text) ?? 0;
    if (slot === null) summons.set(g.text, k + 1);
    g.text = g.text.replace(g.names[0]!, slot !== null ? `${g.names[0]} #${slot}` : `${g.names[0]} (summon ${k + 1})`);
  }
  return all.sort((a, b) => b.impact - a.impact || b.kills - a.kills).slice(0, top);
}

// ---------- the end card (round 2, R2-14) ----------

export interface UnitDamage {
  unit: string;
  name: string;
  side: Side;
  /** HP it took from the other side's units, overkill not counted. */
  damage: number;
}

/** The damage each unit dealt to the other side over the battle, credited to
 * the nearest unit in the hit's chain (a status tick to the unit that put the
 * status on). Fatigue and self-harm count for nobody. Every unit that entered
 * the battle is listed, a summon once it dealt damage; biggest first per side. */
export function damageByUnit(log: BattleEvent[], name: NameOf = displayNames(log), sides = sidesOf(log)): UnitDamage[] {
  const hp = new Map<string, number>();
  const dealt = new Map<string, number>();
  for (const e of log) if (e.type === "BattleStart") for (const s of ["A", "B"] as const) for (const r of e.teams[s]) { hp.set(r.id, r.hp); dealt.set(r.id, 0); }
  for (const e of log) {
    if (e.type === "Hurt") {
      const before = hp.get(e.unit) ?? e.amount;
      hp.set(e.unit, e.hpAfter ?? before - e.amount);
      const by = traceOf(log, e.id, name, sides).links[0];
      const side = sides.get(e.unit);
      if (by?.side && side && by.side !== side) dealt.set(by.unit, (dealt.get(by.unit) ?? 0) + Math.max(0, Math.min(e.amount, before)));
    } else if (e.type === "Heal") hp.set(e.unit, e.hpAfter ?? (hp.get(e.unit) ?? 0) + e.amount);
    else if (e.type === "StatChanged" && e.hpAfter !== undefined) hp.set(e.unit, e.hpAfter);
    else if (e.type === "Summon") hp.set(e.unit, e.atHp ?? e.hp);
  }
  const out: UnitDamage[] = [];
  for (const [unit, damage] of dealt) {
    const side = sides.get(unit);
    if (side) out.push({ unit, name: name(unit), side, damage });
  }
  return out.sort((p, q) => (p.side === q.side ? q.damage - p.damage : p.side < q.side ? -1 : 1));
}

export interface KeyMoment {
  /** "summon": a turning point that is a unit joining ("Imp joins"), no kill (R2-17 batch F). */
  kind: "kill" | "summon" | "combo" | "fatigue" | "hit";
  /** The playback beat it happens in (PlayBeat.index). */
  beat: number;
  /** One short line about this fight (R2-17): "Turning point: Rat kills Bat",
   * "Virus's Poison kills Knight (−3)", "Redirector: 14 dmg, kills Bat",
   * "Fatigue kills Wall +3", "Archer hits Knight (−4)". */
  label: string;
  /** The unit it is about (the killer, the combo's top dealer), when there is one. */
  unit?: string;
}

/** Combos shorter than this aren't a key moment. */
const COMBO_MIN = 3;
/** How many key moments the end card lists, at most. */
const MOMENTS = 3;
/** A label longer than this (in plain names) drops its extras: the phone's row holds about this much on one line. */
const MOMENT_CHARS = 40;

/** The 2–3 moments worth replaying, in battle order, each saying who did
 * what in this fight (R2-17: "Combo: 14 steps, King first" and "Fatigue
 * sets in" said little). First the turning point: the death (or summon)
 * after which the winner led in units standing for good, when the lead
 * changed hands at all; then the biggest killing blow (by the hit, as its
 * chip showed it, overkill included), the longest combo (the beat with the
 * most waves, at least 3) by its top dealer, and fatigue by what it did. A
 * battle with fewer fills up to 3: its other kills, biggest first, then its
 * biggest hits that killed nobody, then its shorter combos (2 waves). Never
 * two on one beat. */
export function keyMomentsOf(log: BattleEvent[], beats: PlayBeat[], name: NameOf = displayNames(log), sides = sidesOf(log)): KeyMoment[] {
  const plain = displayNames(log);
  const beatOfEvent = (id: number) => beats.find((b) => b.waves.some((w) => w.eventIds.includes(id)))?.index ?? beats.find((b) => b.end >= id)?.index ?? beats.length - 1;
  // Who dealt a hurt, and through what: the nearest unit in its chain (a status tick's, whoever put the status on).
  const dealer = (id: number) => traceOf(log, id, plain, sides).links[0] ?? null;
  type Kill = { id: number; killer: string | null; via: string; victim: string; dmg: number; hit?: number; fatigue: boolean };
  const kills: Kill[] = [];
  const lastHit = new Map<string, { id: number; dmg: number }>();
  /** The hurts each Fatigue dealt, so a death they caused is fatigue's. */
  const fatigueOf = new Map<number, number>();
  for (const e of log) {
    if (e.type === "Hurt") {
      lastHit.set(e.unit, { id: e.id, dmg: e.amount });
      if (e.causedBy !== null && log[e.causedBy]?.type === "Fatigue") fatigueOf.set(e.id, e.causedBy);
    } else if (e.type === "Death") {
      const cause = e.causedBy !== null ? log[e.causedBy] : undefined;
      const hit = cause?.type === "Hurt" ? { id: cause.id, dmg: cause.amount } : lastHit.get(e.unit);
      const by = hit ? dealer(hit.id) : null;
      kills.push({ id: e.id, killer: by?.unit ?? null, via: by?.via ?? "", victim: e.unit, dmg: hit?.dmg ?? 0, ...(hit ? { hit: hit.id } : {}), fatigue: hit ? fatigueOf.has(hit.id) : false });
    }
  }
  /** The first label that fits MOMENT_CHARS in plain names, else the last (the shortest). */
  const fit = (...ways: ((n: NameOf) => string)[]) => ways.find((w) => w(plain).length <= MOMENT_CHARS)?.(name) ?? ways.at(-1)!(name);
  const killText = (k: Kill, n: NameOf) => {
    if (!k.killer) return k.fatigue ? `Fatigue kills ${n(k.victim)}` : `${n(k.victim)} falls`;
    // A status's tick says whose status it was: "Virus's Poison kills Knight".
    const who = k.via === "strike" || k.via === "ability" ? n(k.killer) : `${n(k.killer)}'s ${k.via}`;
    return `${who} kills ${n(k.victim)}`;
  };
  const killMoment = (k: Kill): KeyMoment => ({
    kind: "kill",
    beat: beatOfEvent(k.id),
    label: fit((n) => `${killText(k, n)}${k.dmg ? ` (−${k.dmg})` : ""}`, (n) => killText(k, n)),
    ...(k.killer ? { unit: k.killer } : {}),
  });

  // The turning point: units standing per side after each event; the last
  // time the winner's lead went from none to some, if it held to the end.
  const end = log.at(-1);
  const winner = end?.type === "BattleEnd" && end.winner !== "draw" ? end.winner : null;
  let turning: KeyMoment | null = null;
  if (winner) {
    const standing = { A: 0, B: 0 };
    for (const e of log) if (e.type === "BattleStart") { standing.A = e.teams.A.length; standing.B = e.teams.B.length; }
    const loser: Side = winner === "A" ? "B" : "A";
    let lead = standing[winner] - standing[loser];
    let at: BattleEvent | null = null;
    for (const e of log) {
      if (e.type !== "Death" && e.type !== "Summon") continue;
      const side = e.type === "Summon" ? e.side : sides.get(e.unit);
      if (!side) continue;
      standing[side] += e.type === "Death" ? -1 : 1;
      const now = standing[winner] - standing[loser];
      if (lead <= 0 && now > 0) at = e;
      lead = now;
    }
    if (at && lead > 0) {
      const k = at.type === "Death" ? kills.find((x) => x.id === at!.id) : undefined;
      const what = (n: NameOf) => (k ? killText(k, n) : at!.type === "Summon" ? `${n(at!.unit)} joins` : `${n((at as { unit: string }).unit)} falls`);
      const kind = at.type === "Summon" ? "summon" : "kill";
      turning = { kind, beat: beatOfEvent(at.id), label: fit((n) => `Turning point: ${what(n)}`, (n) => `Decisive: ${what(n)}`, what), ...(k?.killer ? { unit: k.killer } : at.type === "Summon" ? { unit: at.unit } : {}) };
    }
  }

  // A combo by what it did: the unit that did the most in the beat, its damage, kills and healing.
  const comboMoment = (b: PlayBeat): KeyMoment | null => {
    const ids = new Set(b.waves.flatMap((w) => w.eventIds));
    const did = new Map<string, { dmg: number; heal: number; kills: string[] }>();
    const of = (u: string) => did.get(u) ?? (did.set(u, { dmg: 0, heal: 0, kills: [] }), did.get(u)!);
    for (const id of ids) {
      const e = log[id];
      if (e?.type === "Hurt" && e.amount > 0) {
        const by = dealer(id)?.unit;
        if (by && sides.get(by) !== sides.get(e.unit)) of(by).dmg += e.amount;
      } else if (e?.type === "Heal" && e.amount > 0) {
        const by = dealer(id)?.unit;
        if (by) of(by).heal += e.amount;
      }
    }
    for (const k of kills) if (ids.has(k.id) && k.killer && sides.get(k.killer) !== sides.get(k.victim)) of(k.killer).kills.push(k.victim);
    const score = (d: { dmg: number; heal: number; kills: string[] }) => d.dmg + d.heal + 5 * d.kills.length;
    const top = [...did].sort((p, q) => score(q[1]) - score(p[1]))[0];
    const steps = b.waves.length;
    if (!top || score(top[1]) === 0) {
      // No unit dealt damage or healing. In fatigue's beat, fatigue did it:
      // its own moment says so, not a unit that only reacted (R2-17 batch F).
      if ([...ids].some((id) => log[id]?.type === "Fatigue")) return null;
      // Else its first actor and what the beat did: the status it spread most
      // ("Coach: Strength on 4 units"), else its PWR / HP changes, summons,
      // or what Shield blocked; never just "8 steps".
      const actor = b.waves.find((w) => w.actor)?.actor;
      // Nobody acted: nothing to name, no moment.
      if (!actor) return null;
      const on = new Map<string, Set<string>>();
      const stat = { pwr: 0, hp: 0, units: new Set<string>() };
      const summoned: { unit: string; back: boolean }[] = [];
      let blocked = 0;
      for (const id of ids) {
        const e = log[id];
        if (e?.type === "StatusApplied") on.set(e.status, (on.get(e.status) ?? new Set()).add(e.unit));
        else if (e?.type === "StatChanged") { stat[e.stat === "pwr" ? "pwr" : "hp"] += e.delta; stat.units.add(e.unit); }
        else if (e?.type === "Summon") summoned.push({ unit: e.unit, back: !!e.resurrected });
        else if (e?.type === "Hurt" && e.amount === 0) blocked += e.absorbed ?? 0;
      }
      const st = [...on].sort((p, q) => q[1].size - p[1].size)[0];
      const signed = (v: number, what: string) => `${v > 0 ? "+" : "−"}${Math.abs(v)} ${what}`;
      const stats = [stat.pwr ? signed(stat.pwr, "PWR") : "", stat.hp ? signed(stat.hp, "HP") : ""].filter(Boolean).join(", ");
      const mine = [...stat.units].every((u) => sides.get(u) === sides.get(actor));
      const to = (n: NameOf) => (stat.units.size === 1 ? ` ${mine ? "to" : "on"} ${n([...stat.units][0]!)}` : mine ? " to the line" : ` on ${stat.units.size} units`);
      const what = (n: NameOf): string | null =>
        st
          ? `${st[0]} on ${st[1].size === 1 ? "1 unit" : `${st[1].size} units`}`
          : stats
            ? `${stats}${to(n)}`
            : summoned.length
              ? `${summoned[0]!.back ? "brings back" : "summons"} ${n(summoned[0]!.unit)}${summoned.length > 1 ? ` +${summoned.length - 1}` : ""}`
              : blocked
                ? `${blocked} blocked by Shield`
                : null;
      if (what(plain) === null) return { kind: "combo", beat: b.index, label: fit((n) => `${n(actor)}: ${steps} steps`), unit: actor };
      return { kind: "combo", beat: b.index, label: fit((n) => `${n(actor)}: ${what(n)} in ${steps} steps`, (n) => `${n(actor)}: ${what(n)}`), unit: actor };
    }
    const [u, d] = top;
    const victims = (n: NameOf) => `kills ${n(d.kills[0]!)}${d.kills.length > 1 ? ` +${d.kills.length - 1}` : ""}`;
    const label = d.kills.length
      ? fit((n) => `${n(u)}: ${d.dmg} dmg, ${victims(n)}`, (n) => `${n(u)} ${victims(n)}`)
      : d.dmg
        ? fit((n) => `${n(u)}: ${d.dmg} dmg in ${steps} steps`, (n) => `${n(u)}: ${d.dmg} dmg`)
        : fit((n) => `${n(u)}: +${d.heal} HP in ${steps} steps`, (n) => `${n(u)}: +${d.heal} HP`);
    return { kind: "combo", beat: b.index, label, unit: u };
  };

  // Fatigue by what it did over the fight: who it killed, else how much it took.
  const fatigueMoment = (f: BattleEvent): KeyMoment => {
    const fk = kills.filter((k) => k.fatigue);
    const total = [...fatigueOf.keys()].reduce((t, id) => t + ((log[id] as { amount: number }).amount ?? 0), 0);
    const label = fk.length ? fit((n) => `Fatigue kills ${n(fk[0]!.victim)}${fk.length > 1 ? ` +${fk.length - 1}` : ""}`) : `Fatigue sets in: ${total} dmg, no kills`;
    return { kind: "fatigue", beat: beatOfEvent(f.id), label };
  };

  const out: KeyMoment[] = [];
  const add = (m: KeyMoment | null) => { if (m && out.length < MOMENTS && !out.some((o) => o.beat === m.beat)) out.push(m); };
  if (turning) add(turning);
  const fatigue = log.find((e) => e.type === "Fatigue");
  // Fatigue's kills are its own moment's ("Fatigue kills Wall +3"), not a row each.
  const byDamage = kills.filter((k) => !(fatigue && k.fatigue)).sort((p, q) => q.dmg - p.dmg || p.id - q.id);
  if (byDamage[0]) add(killMoment(byDamage[0]));
  const combos = [...beats].sort((p, q) => q.waves.length - p.waves.length || p.index - q.index);
  if (combos[0] && combos[0].waves.length >= COMBO_MIN) add(comboMoment(combos[0]));
  if (fatigue) add(fatigueMoment(fatigue));
  for (const k of byDamage.slice(1)) add(killMoment(k));
  const killing = new Set(kills.flatMap((k) => (k.hit !== undefined ? [k.hit] : [])));
  const hits = log.flatMap((e) => (e.type === "Hurt" && e.amount >= 2 && !killing.has(e.id) ? [{ id: e.id, unit: e.unit, amount: e.amount }] : [])).sort((p, q) => q.amount - p.amount || p.id - q.id);
  for (const e of hits) {
    const by = dealer(e.id)?.unit;
    if (!by) continue;
    add({ kind: "hit", beat: beatOfEvent(e.id), label: `${name(by)} hits ${name(e.unit)} (−${e.amount})`, unit: by });
  }
  for (const b of combos) if (b.waves.length >= 2) add(comboMoment(b));
  // Last, a fight fatigue ended: its other kills ("Fatigue kills Wall2 (−11)").
  for (const k of kills.filter((x) => fatigue && x.fatigue).sort((p, q) => q.dmg - p.dmg || p.id - q.id)) add(killMoment(k));
  return out.sort((p, q) => p.beat - q.beat);
}

/** A turn as the viewer labels it: "T3", and the battle's start (turn 0) "Start". */
export function turnLabel(turn: number): string {
  return turn >= 1 ? `T${turn}` : "Start";
}

// ---------- the timeline under the desktop battle (R2-16) ----------

/** What a turn's block marks: a death (in the fallen unit's side colour), a
 * big hit, and the turn fatigue set in. Each mark sits on the beat it plays in. */
export interface TimelineMark {
  kind: "death" | "big" | "fatigue";
  beat: number;
  /** The fallen unit's side (deaths), the struck unit's side (big hits). */
  side?: Side;
  unit?: string;
}

/** One block per turn that has beats: its beats (playback indexes, in order) and its marks. */
export interface TimelineTurn {
  turn: number;
  beats: number[];
  marks: TimelineMark[];
}

/** A hit counts as big at this much damage, or at 60% of the battle's biggest when that is more. */
const BIG_HIT_MIN = 4;

/** The turn timeline: one block per turn, each with its beats and its marks
 * (deaths, big hits, fatigue setting in). Pure, like boardAt: scrubbing to a
 * block's beat is go(beat). Beats before the first turn (battle start) get a
 * block of their own, turn 0, which the viewer labels "Start" (R2-17). */
export function timelineOf(log: BattleEvent[], beats: PlayBeat[], sides = sidesOf(log)): TimelineTurn[] {
  const hits = log.flatMap((e) => (e.type === "Hurt" ? [e.amount] : []));
  const big = Math.max(BIG_HIT_MIN, Math.ceil(Math.max(0, ...hits) * 0.6));
  const fatigue = log.find((e) => e.type === "Fatigue");
  const turns: TimelineTurn[] = [];
  for (const b of beats) {
    const last = turns.at(-1);
    const t = last && Math.max(0, b.turn) <= last.turn ? last : { turn: Math.max(0, b.turn), beats: [], marks: [] };
    if (t !== last) turns.push(t);
    t.beats.push(b.index);
    const ids = new Set(b.waves.flatMap((w) => w.eventIds));
    let bigHit = false;
    for (const id of [...ids].sort((p, q) => p - q)) {
      const e = log[id];
      if (e?.type === "Death") t.marks.push({ kind: "death", beat: b.index, unit: e.unit, ...(sides.get(e.unit) ? { side: sides.get(e.unit)! } : {}) });
      else if (e?.type === "Hurt" && e.amount >= big && !bigHit) {
        bigHit = true;
        t.marks.push({ kind: "big", beat: b.index, unit: e.unit, ...(sides.get(e.unit) ? { side: sides.get(e.unit)! } : {}) });
      }
    }
    if (fatigue && (ids.has(fatigue.id) || (b.turn === fatigue.turn && b.end >= fatigue.id)) && !turns.some((x) => x.marks.some((m) => m.kind === "fatigue")) && !t.marks.some((m) => m.kind === "fatigue")) t.marks.push({ kind: "fatigue", beat: b.index });
  }
  return turns;
}
