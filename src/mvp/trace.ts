// Battle viewer logic (mission #574, slice 9): playback steps with one-line
// captions, tap-a-change-to-trace-its-chain, and "why I lost". Pure functions
// over the causal log; the phone client (mobile/screens/battle.ts) only draws
// what these return. "Why did this happen" is always a walk up `causedBy`.

import { beatsOf, isRootKind } from "../beats.js";
import { displayNames, type NameOf } from "../trace.js";
import type { BattleEvent, Side } from "../types.js";

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
  /** Nearest cause first. Consecutive acts by the same unit collapse to one link. */
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
      const prev = links.at(-1);
      if (!prev || prev.unit !== a.unit) {
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
 * Until the kernel stamps the matched When (R2-15), the trigger is read off
 * the cause's event type, which is what a unit's When matched. */
export function firingOf(log: BattleEvent[], step: Pick<Step, "eventIds" | "changes">): Firing | null {
  const first = log[step.eventIds[0]!];
  if (!first || first.source === "kernel" || first.source.status) return null;
  const c = step.changes[0];
  if (!c) return null;
  const cause = first.causedBy !== null ? log[first.causedBy] : undefined;
  const trigger = cause && TRIGGER_EVENTS.has(cause.type) ? (`trigger:${cause.type}` as const) : null;
  const triggerStatus = cause && (cause.type === "StatusApplied" || cause.type === "StatusRemoved") ? cause.status : undefined;
  const e = log[c.eventId];
  let effect: string;
  let effectStatus: string | undefined;
  switch (e?.type) {
    case "Hurt": effect = "effect:damage"; break;
    case "Heal": effect = "effect:heal"; break;
    case "StatusApplied": effect = `status:${e.status}`; effectStatus = e.status; break;
    case "StatChanged": effect = `stat:${e.stat}`; break;
    case "Summon": effect = e.resurrected ? "effect:resurrect" : "effect:summon"; break;
    case "Silenced": effect = "effect:silence"; break;
    case "Death": effect = "effect:damage"; break;
    default: return null;
  }
  return { unit: first.source.unit, trigger, ...(triggerStatus ? { triggerStatus } : {}), effect, ...(effectStatus ? { effectStatus } : {}) };
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
      parent.caption += `, ${first.status} −${first.stacks}`;
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
      return `Chain capped after ${e.steps} steps`;
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
