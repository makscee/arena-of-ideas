// Battle viewer logic (mission #574, slice 9): playback steps with one-line
// captions, tap-a-change-to-trace-its-chain, and "why I lost". Pure functions
// over the causal log; the phone client (mobile/screens/battle.ts) only draws
// what these return. "Why did this happen" is always a walk up `causedBy`.

import { beatsOf, isRootKind } from "../beats.js";
import { displayNames, type NameOf } from "../trace.js";
import type { AbilityRef, BattleEvent, Side, UnitFilter, When } from "../types.js";
import type { Lang } from "../describe.js";
import { ruCount, ruStatusName, RU_STAT } from "../describe-ru.js";

// ---------- language (M4-3) ----------

/** The language captions, change labels and chain roots are written in. The
 * battle viewer is the only user and a page reads one language, so it is set
 * once (setTraceLang) rather than threaded through every caller; unset is
 * English, exactly as before. */
let LANG: Lang | undefined;

export function setTraceLang(lang: Lang | undefined): void {
  LANG = lang;
}

const ru = (): boolean => LANG === "ru";
/** A status's name in the caption's language. */
const st = (status: string): string => (ru() ? ruStatusName(status) : status);
/** A stat word in the caption's language: "PWR" / "АТК". */
const statWord = (stat: string): string => (ru() && (stat === "pwr" || stat === "hp") ? RU_STAT[stat] : stat.toUpperCase());

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

export type ChangeKind = "damage" | "heal" | "buff" | "debuff" | "status" | "death" | "summon" | "silence" | "noRoom";

/** A change you can tap: a number (or mark) on a unit card. */
export interface Change {
  eventId: number;
  unit: string;
  kind: ChangeKind;
  /** What the card shows: "−5", "+2", "+1 PWR", "Poison ×2", "✝". */
  label: string;
}

const CHANGE_TYPES = new Set(["Hurt", "Heal", "StatChanged", "StatusApplied", "Death", "Summon", "Silenced", "NoRoom"]);

/** A hit as a change reads: "−n", or for a hit that did nothing, what
 * stopped it ("3 blocked" by Shield, or "no damage"), never "−0". */
export function hurtLabel(amount: number, absorbed?: number): string {
  if (amount > 0) return `−${amount}`;
  if (ru()) return absorbed ? `${absorbed} заблокировано` : "без урона";
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
      return { eventId: e.id, unit: e.unit, kind: e.delta >= 0 ? "buff" : "debuff", label: `${e.delta >= 0 ? "+" : "−"}${Math.abs(e.delta)} ${statWord(e.stat)}` };
    case "StatusApplied":
      return { eventId: e.id, unit: e.unit, kind: "status", label: `${st(e.status)} ×${e.stacks}` };
    case "Death":
      return { eventId: e.id, unit: e.unit, kind: "death", label: "✝" };
    case "Summon":
      return { eventId: e.id, unit: e.unit, kind: "summon", label: ru() ? (e.resurrected ? "возвращается" : "новый") : e.resurrected ? "returns" : "new" };
    case "Silenced":
      return { eventId: e.id, unit: e.unit, kind: "silence", label: ru() ? "заглушён" : "silenced" };
    // On the unit that tried, so it can be tapped for its trace (R4-2).
    case "NoRoom":
      return { eventId: e.id, unit: e.unit, kind: "noRoom", label: ru() ? "нет места" : "no room" };
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
  return l.via === "strike" || l.via === "ability" ? l.name : `${l.name} (${st(l.via)})`;
}

/** What a trace with no acting unit came from: fatigue, or the battle's start. */
function rootText(log: BattleEvent[], eventId: number): string {
  let cur = log[eventId];
  for (let hops = 0; cur && hops < MAX_HOPS; hops++) {
    if (cur.type === "Fatigue") return ru() ? "Усталость" : "Fatigue";
    if (cur.causedBy === null) break;
    cur = log[cur.causedBy];
  }
  if (ru()) return cur?.type === "BattleStart" ? "начало боя" : "правила";
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
  if (ru())
    switch (e.type) {
      case "TurnStart": return `Ход ${e.turn}`;
      case "TurnEnd": return `Конец хода ${e.turn}`;
      case "BattleStart": return "Начало боя";
      case "Fatigue": return `Усталость (ход ${e.turn})`;
      case "PairFaced": return `Ход ${e.turn}`;
      default: return e.type;
    }
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

/** The end of a battle the turn cap stopped (BattleEnd.timeUp): its caption
 * leads with the glossary's "Time's up" term (battle:timeUp). */
export const TIME_UP_CAPTION = "Time's up: draw";
export const TIME_UP_CAPTION_RU = "Время вышло: ничья";

/** A summon or revive that found its line full (R4-2): the glossary's
 * battle:noRoom term, as its caption reads it. */
export const NO_ROOM = "No room";
export const NO_ROOM_RU = "Нет места";

/** How the battle's end reads from `p`. */
export function endCaption(winner: Side | "draw", p: Perspective = {}): string {
  if (ru()) {
    if (winner === "draw") return "Ничья";
    if (p.you) return winner === p.you ? "Вы победили" : "Они победили";
    return `${p.sideName ? p.sideName(winner) : `Сторона ${winner}`} побеждает`;
  }
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

/** Milliseconds between waves at 1×, the shortest a beat lasts, and the cap
 * (round 3, note 14: about 1.5× round 2's, so every hit can be followed). */
export const WAVE_MS = 220;
export const BEAT_MS = 1300;
export const BEAT_MAX_MS = 2200;
/** A quiet beat (one wave, a plain hit or a status tick, nothing dies) is
 * shorter: a −1 trade shouldn't take as long as a kill (pacing by weight). */
export const QUIET_BEAT_MS = 900;
/** A quiet beat of a few waves (a −1 hit and the +1 heal it sets off) holds
 * its last wave this long, and lasts at least QUIET_MULTI_MS (R3-26). */
const QUIET_HOLD_MS = 650;
export const QUIET_MULTI_MS = 1000;
/** A quiet beat that repeats one of the turn before (the same lines again:
 * Medic heals itself +1 each turn against a 1-PWR striker) plays at this
 * share of its time (R3-26: early fights were 40+ s of the same lines). */
export const REPEAT_SHARE = 0.7;
/** How long the last wave stays before the next beat. */
const BEAT_HOLD_MS = 800;
/** How long the line-up shows before the first beat, at 1×. */
export const LINEUP_MS = 900;
/** The empty beat after the deciding blow (the battle's end, no change on
 * the board): short, so the end card follows the blow's own hold. */
export const END_BEAT_MS = 150;
/** The time a beat's big moments add at 1× (note 14): a kill, a hit of
 * BIG_HIT_MIN or more, a summon or revive, the first fatigue beat, and the
 * battle's last beat (the deciding blow, before the end card). */
export const EMPHASIS_MS = { kill: 350, big: 200, summon: 250, fatigue: 500, last: 600 } as const;
/** How long a card stands still after its killing blow lands, before it pops. */
export const KILL_FREEZE_MS = 120;
/** A hit counts as big at this much damage (the timeline's marks also scale it to the battle). */
export const BIG_HIT_MIN = 4;

/** What a beat holds that earns it more time; `quiet` (only small changes)
 * and `repeat` (a quiet beat the turn before already played) earn it less. */
export type BeatWeight = Partial<Record<keyof typeof EMPHASIS_MS | "end" | "quiet" | "repeat", boolean>>;

/** Only small changes: hits under BIG_HIT_MIN, heals, statuses, PWR / HP, a silence; no death or summon. */
function quietBeat(log: BattleEvent[], b: PlayBeat): boolean {
  return b.waves.every((w) => w.changes.every((c) => {
    if (c.kind === "death" || c.kind === "summon") return false;
    const e = log[c.eventId];
    return !(e?.type === "Hurt" && e.amount >= BIG_HIT_MIN);
  }));
}
/** A beat's lines, to tell a repeat: its waves' captions. */
const linesOf = (b: PlayBeat) => b.waves.map((w) => w.caption).join("\n");

/** When each wave lands (ms from the beat's start, at 1×) and how long the
 * beat lasts: waves 220 ms apart, squeezed so the last lands by 1.4 s; the
 * beat lasts 1.3 s, up to 2.2 s for a long cascade, and 0.9 s when quiet. */
export function beatTiming(waves: number, quiet = false): { at: number[]; ms: number } {
  if (quiet && waves <= 1) return { at: [0], ms: QUIET_BEAT_MS };
  const span = BEAT_MAX_MS - BEAT_HOLD_MS;
  const gap = waves > 1 ? Math.min(WAVE_MS, span / (waves - 1)) : 0;
  const at = Array.from({ length: waves }, (_, i) => Math.round(i * gap));
  if (quiet) return { at, ms: Math.min(BEAT_MAX_MS, Math.max(QUIET_MULTI_MS, (at.at(-1) ?? 0) + QUIET_HOLD_MS)) };
  return { at, ms: Math.min(BEAT_MAX_MS, Math.max(BEAT_MS, (at.at(-1) ?? 0) + BEAT_HOLD_MS)) };
}

/** Each beat's weight: a kill, a big hit, a summon or revive, the first
 * fatigue beat (the one holding the battle's first Fatigue event, else the
 * first beat of its turn after it), and the last beat that changes the
 * board (the deciding blow); `end` marks an empty beat after it. */
export function weightsOf(log: BattleEvent[], beats: PlayBeat[]): BeatWeight[] {
  const fatigue = log.find((e) => e.type === "Fatigue");
  const firstFatigue = fatigue ? beats.findIndex((b) => b.waves.some((w) => w.eventIds.includes(fatigue.id)) || (b.turn === fatigue.turn && b.end >= fatigue.id)) : -1;
  // The deciding blow's beat, not the empty BattleEnd beat after it: its
  // hold lands while the killed card is still on the board.
  const decisive = beats.map((b) => b.waves.some((w) => w.changes.length)).lastIndexOf(true);
  return beats.map((b, i) => {
    const events = b.waves.flatMap((w) => w.eventIds.map((id) => log[id]));
    const w: BeatWeight = {};
    if (events.some((e) => e?.type === "Death")) w.kill = true;
    if (events.some((e) => e?.type === "Hurt" && e.amount >= BIG_HIT_MIN)) w.big = true;
    if (events.some((e) => e?.type === "Summon")) w.summon = true;
    if (i === firstFatigue) w.fatigue = true;
    if (i === (decisive >= 0 ? decisive : beats.length - 1)) w.last = true;
    if (decisive >= 0 && i > decisive && b.waves.every((x) => !x.changes.length)) w.end = true;
    // Empty time (R3-26): a beat with only small changes and nothing to
    // emphasise is quiet; one that replays a line of the turn before, a repeat.
    if (!w.kill && !w.big && !w.summon && !w.fatigue && !w.last && !w.end && b.waves.some((x) => x.changes.length) && quietBeat(log, b)) {
      w.quiet = true;
      const lines = linesOf(b);
      if (b.turn >= 1 && beats.some((o) => o.turn === b.turn - 1 && linesOf(o) === lines)) w.repeat = true;
    }
    return w;
  });
}

/** A beat's timing at 1×, by its weight: one wave of plain hits, status
 * changes or nothing at all is quiet; anything more plays full length. Each
 * big moment then adds its EMPHASIS_MS after the last wave (past the cap: a
 * long cascade that kills still gets its beat of stillness). An empty beat
 * after the deciding blow lasts END_BEAT_MS. */
export function timingOf(beat: PlayBeat, weight: BeatWeight = {}): { at: number[]; ms: number } {
  if (weight.end) return { at: beat.waves.map(() => 0), ms: END_BEAT_MS };
  const quiet = weight.quiet || (beat.waves.length === 1 && beat.waves[0]!.changes.every((c) => c.kind === "damage" || c.kind === "status"));
  const t = beatTiming(beat.waves.length, quiet);
  const extra = (Object.keys(EMPHASIS_MS) as (keyof typeof EMPHASIS_MS)[]).reduce((n, k) => n + (weight[k] ? EMPHASIS_MS[k] : 0), 0);
  // A repeat keeps its waves' spacing and cuts its hold (never below its last wave plus 0.4 s).
  if (weight.repeat) return { at: t.at, ms: Math.max((t.at.at(-1) ?? 0) + 400, Math.round(t.ms * REPEAT_SHARE)) + extra };
  return { at: t.at, ms: t.ms + extra };
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
    case "NoRoom": return { effect: "battle:noRoom" };
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
  // A status a hit or a heal used up (Shield spent on the blow) leaves
  // because of that hit: its cause is the hit's, not the holder's own status,
  // so a unit killed through its Shield reads as the striker's (R3-21).
  const hit = e.type === "StatusRemoved" && e.causedBy !== null && e.causedBy < e.id ? log[e.causedBy] : undefined;
  if (e.type === "StatusRemoved" && hit && (hit.type === "Hurt" || hit.type === "Heal") && hit.unit === e.unit && hops < MAX_HOPS) {
    const up = causeOfEvent(log, hit, whenOf, hops + 1);
    if (up) return withEffect(up, eff);
  }
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
  return `${cause} → ${label} ${ru() ? "на" : "on"} ${names.join(", ")}`;
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

// ---------- the Log: a beat's waves grouped into rows (round 4, R4-3) ----------

/** A row of the battle Log: one or more waves of one beat, read as one line.
 * Long fights repeat themselves (a Shield on every ally after every hit, a
 * Poison tick on each unit, a Blessing save as three lines), so the Log
 * groups what playback shows wave by wave. */
export interface LogRow {
  beat: number;
  /** The waves of `beat` this row stands for, in order. */
  waves: number[];
  turn: number;
  subjectSide: Side | null;
  caption: string;
  /** Every change of every wave in it: the Why panel reaches each one. */
  changes: Change[];
  eventIds: number[];
}

/** What one target took in a grouped row, summed. */
interface Took {
  amount: number;
  absorbed: number;
  /** Stat follow-ups of a status (Vitality's +HP), by stat. */
  stats: Record<string, number>;
  /** HP a Blessing save gave back after the hit, or null. */
  saved: number | null;
  /** How many times it landed on this target. */
  hits: number;
}

type FxType = "Hurt" | "Heal" | "StatChanged" | "StatusApplied";

/** A row while it is being grouped. `fx` is set when the row is plain
 * effects (one source, one kind) and so can be summed and re-captioned;
 * other rows keep their wave's caption. */
interface LogDraft {
  waves: number[];
  first: Step;
  eventIds: number[];
  changes: Change[];
  caption: string;
  sides: Set<Side | null>;
  times: number;
  fx: {
    type: FxType;
    /** The status or stat it is about ("" for hits and heals). */
    what: string;
    /** "Coach", "Fatigue", or the ticking status ("Poison"). */
    cause: string;
    /** The firing's source and kind; with `parent`, one firing. */
    key: string;
    parent: number | null;
    actorSide: Side | null;
    /** A status acting on its own holder at a turn's edge (a Poison tick). */
    tick: boolean;
    fades: number;
    targets: string[];
    took: Map<string, Took>;
  } | null;
}

const FX_TYPES = new Set<string>(["Hurt", "Heal", "StatChanged", "StatusApplied"]);

/** Who acts, as the Log reads it: by name and side, not ability or instance
 * (a fused unit's two Shield abilities, or two War Drummers, are one effect). */
function sourceKey(log: BattleEvent[], e: BattleEvent, plain: NameOf, sides: Map<string, Side>): string {
  if (e.source === "kernel") return `kernel:${e.causedBy !== null ? log[e.causedBy]?.type ?? "" : ""}`;
  return `${plain(e.source.unit)}:${sides.get(e.source.unit) ?? ""}/${e.source.status ?? ""}`;
}

/** The effect part of a wave, when it is plain enough to sum: every event is
 * the same kind from the same source, plus the follow-ups its caption
 * already covers (a status's stat change, the Shield an absorbed hit spent). */
function fxOf(log: BattleEvent[], s: Step, plain: NameOf, sides: Map<string, Side>): LogDraft["fx"] {
  const e = log[s.eventIds[0]!]!;
  if (!FX_TYPES.has(e.type)) return null;
  const type = e.type as FxType;
  const parent = e.causedBy !== null ? log[e.causedBy] : undefined;
  // A strike's hit reads "X strikes Y → −n": its own beat, nothing to group.
  if (type === "Hurt" && e.source === "kernel" && parent?.type === "Strike") return null;
  const what = e.type === "StatusApplied" ? e.status : e.type === "StatChanged" ? e.stat : "";
  const src = sourceKey(log, e, plain, sides);
  const took = new Map<string, Took>();
  const targets: string[] = [];
  for (const id of s.eventIds) {
    const m = log[id]!;
    if (m.type === e.type && m.causedBy === e.causedBy && sameSource(m, e) && "unit" in m) {
      if ((m.type === "StatusApplied" && m.status !== what) || (m.type === "StatChanged" && m.stat !== what)) return null;
      const unit = m.unit as string;
      let t = took.get(unit);
      if (!t) {
        t = { amount: 0, absorbed: 0, stats: {}, saved: null, hits: 0 };
        took.set(unit, t);
        targets.push(unit);
      }
      t.hits++;
      if (m.type === "Hurt") {
        t.amount += m.amount;
        t.absorbed += m.absorbed ?? 0;
      } else if (m.type === "Heal") t.amount += m.amount;
      else if (m.type === "StatChanged") t.amount += m.delta;
      else if (m.type === "StatusApplied") t.amount += m.stacks;
      continue;
    }
    if (m.type === "StatChanged" && isStatusFollowUp(log, m)) {
      const t = took.get(m.unit);
      if (!t || type !== "StatusApplied") return null;
      t.stats[m.stat] = (t.stats[m.stat] ?? 0) + m.delta;
      continue;
    }
    const spent = m.causedBy !== null ? log[m.causedBy] : undefined;
    if (m.type === "StatusRemoved" && m.status === "Shield" && spent?.type === "Hurt" && spent.absorbed) continue;
    return null;
  }
  const own = e.source !== "kernel" && e.source.status !== undefined && e.source.status !== "Blessing" && targets.length === 1 && targets[0] === e.source.unit;
  const tick = own && (type === "Hurt" || type === "Heal") && (parent?.type === "TurnEnd" || parent?.type === "TurnStart");
  return {
    type,
    what,
    cause: tick && e.source !== "kernel" ? e.source.status! : s.caption.split(" → ")[0]!,
    key: tick && e.source !== "kernel" ? `tick:${e.source.status}:${type}` : `${src}|${type}|${what}`,
    parent: e.causedBy,
    actorSide: s.actorSide,
    tick,
    fades: 0,
    targets,
    took,
  };
}

function sameSource(a: BattleEvent, b: BattleEvent): boolean {
  if (a.source === "kernel" || b.source === "kernel") return a.source === b.source;
  return a.source.unit === b.source.unit && a.source.status === b.source.status;
}

/** The units alive on each side just before event `id`. */
function aliveBefore(log: BattleEvent[], id: number): Map<Side, Set<string>> {
  const alive = new Map<Side, Set<string>>([["A", new Set()], ["B", new Set()]]);
  for (let i = 0; i < id && i < log.length; i++) {
    const e = log[i]!;
    if (e.type === "BattleStart") for (const s of ["A", "B"] as const) for (const r of e.teams[s]) alive.get(s)!.add(r.id);
    else if (e.type === "Summon") alive.get(e.side)!.add(e.unit);
    else if (e.type === "Death") for (const set of alive.values()) set.delete(e.unit);
  }
  return alive;
}

function sumStats(a: Record<string, number>, b: Record<string, number>): Record<string, number> {
  const out = { ...a };
  for (const [k, v] of Object.entries(b)) out[k] = (out[k] ?? 0) + v;
  return out;
}

function signed(n: number): string {
  return `${n >= 0 ? "+" : "−"}${Math.abs(n)}`;
}

/** Groups each beat's waves into Log rows (round 4, note 4):
 * - R1: rows of one actor and one effect within a beat merge: one firing's
 *   targets join up again, and a firing that repeats on the same targets
 *   sums its amounts and says "(×N)";
 * - R2: a status ticking on many units is one row, its fades folded in
 *   ("Poison ticks 5 units, −4 each, 4 blocked");
 * - R3: a Blessing save folds into the hit it stopped
 *   ("Poison → Rose −112 → Blessing saves Rose (+113)");
 * - R4: a target list that is a whole side reads "all allies" / "all enemies".
 * Rows keep every change they merged, so the Why panel reaches each event. */
export function logRowsOf(log: BattleEvent[], beats: PlayBeat[], name: NameOf = displayNames(log)): LogRow[] {
  const rows: LogRow[] = [];
  const plain = displayNames(log);
  const sides = sidesOf(log);
  for (const pb of beats) {
    let drafts: LogDraft[] = pb.waves.map((w, i) => ({
      waves: [i],
      first: w,
      eventIds: [...w.eventIds],
      changes: [...w.changes],
      caption: w.caption,
      sides: new Set([w.subjectSide]),
      times: 1,
      fx: fxOf(log, w, plain, sides),
    }));
    const absorb = (into: LogDraft, d: LogDraft) => {
      into.waves.push(...d.waves);
      into.eventIds.push(...d.eventIds);
      into.changes.push(...d.changes);
      for (const s of d.sides) into.sides.add(s);
    };
    const holding = (id: number) => drafts.find((d) => d.eventIds.includes(id));
    const gone = new Set<LogDraft>();
    // R3: a Blessing save (the stopped death, its heal) joins the hit.
    for (const d of drafts) {
      const e = log[d.first.eventIds[0]!]!;
      if (e.type !== "Intercepted" || e.original !== "Death" || e.by.status !== "Blessing" || !e.unit) continue;
      const heal = drafts.find((h) => !gone.has(h) && h.eventIds.length === 1 && log[h.eventIds[0]!]?.type === "Heal" && log[h.eventIds[0]!]?.causedBy === e.id);
      const amount = heal ? (log[heal.eventIds[0]!] as { amount: number }).amount : 0;
      const hit = e.causedBy !== null ? holding(e.causedBy) : undefined;
      const t = hit?.fx?.type === "Hurt" ? hit.fx.took.get(e.unit) : undefined;
      if (hit && t) {
        t.saved = (t.saved ?? 0) + amount;
        absorb(hit, d);
        gone.add(d);
      } else {
        d.fx = null;
        d.caption = `Blessing saves ${name(e.unit)}${heal ? ` (+${amount})` : ""}`;
      }
      if (heal) {
        absorb(t && hit ? hit : d, heal);
        gone.add(heal);
      }
    }
    drafts = drafts.filter((d) => !gone.has(d));
    // R2: a status's fade at the turn's edge folds into its tick.
    for (const d of drafts) {
      const e = log[d.first.eventIds[0]!]!;
      if (e.type !== "StatusRemoved" || d.eventIds.length !== 1 || e.source === "kernel") continue;
      const tick = drafts.find((t) => !gone.has(t) && t.fx?.tick && t.fx.parent === e.causedBy && t.fx.key.startsWith(`tick:${e.status}:`) && t.fx.took.has(e.unit));
      if (!tick) continue;
      absorb(tick, d);
      tick.fx!.fades++;
      gone.add(d);
    }
    drafts = drafts.filter((d) => !gone.has(d));
    // R1 and R2: one actor's same effect is one row, summed per target (a
    // tick is one effect across its holders).
    {
      const out: LogDraft[] = [];
      for (const d of drafts) {
        const into = d.fx ? out.find((o) => o.fx && o.fx.key === d.fx!.key) : undefined;
        if (!into) {
          out.push(d);
          continue;
        }
        absorb(into, d);
        const f = into.fx!;
        for (const u of d.fx!.targets) {
          const a = f.took.get(u), b = d.fx!.took.get(u)!;
          if (!a) {
            f.targets.push(u);
            f.took.set(u, { ...b });
          } else {
            a.amount += b.amount;
            a.absorbed += b.absorbed;
            a.stats = sumStats(a.stats, b.stats);
            a.saved = a.saved === null && b.saved === null ? null : (a.saved ?? 0) + (b.saved ?? 0);
            a.hits += b.hits;
          }
        }
        f.fades += d.fx!.fades;
      }
      drafts = out;
    }
    // R1 for the rest (a summon, a stopped strike): the same line again is "(×N)".
    const out: LogDraft[] = [];
    for (const d of drafts) {
      const into = !d.fx ? out.find((o) => !o.fx && o.caption === d.caption) : undefined;
      if (into) {
        absorb(into, d);
        into.times += d.times;
      } else out.push(d);
    }
    for (const d of out) {
      const order = [...d.waves].sort((x, y) => x - y);
      const side = d.sides.size === 1 ? [...d.sides][0]! : null;
      const caption = d.fx ? fxCaption(log, d.fx, name, d.eventIds) : d.caption;
      rows.push({ beat: pb.index, waves: order, turn: pb.turn, subjectSide: side, caption: d.times > 1 ? `${caption} (×${d.times})` : caption, changes: d.changes, eventIds: d.eventIds });
    }
  }
  return rows;
}

// ---------- the Log: repeated turns fold (round 4, R5, R4-13) ----------

/** A turn of the Log that repeats the turn before it, read as one line:
 * what it did in sum (damage, heals, deaths), its rows behind a tap. */
export interface LogFold {
  /** The first turn of the run of repeated turns, and the last (the same
   * for a single turn): T2–T4 each repeating the turn before read as one. */
  turn: number;
  turnTo: number;
  caption: string;
  /** HP lost on each side (Shield's blocks aside), and HP healed. */
  damage: Record<Side, number>;
  /** Damage Shield took instead. */
  blocked: number;
  heals: Record<Side, number>;
  /** Who fell, by name. */
  deaths: string[];
  rows: LogRow[];
}

/** A Log entry: a row, or a folded turn of rows. */
export type LogItem = LogRow | LogFold;

export function isLogFold(x: LogItem): x is LogFold {
  return "rows" in x;
}

/** What a row does, amounts and targets aside: each event's kind, who acts
 * (by name and side) and with what status or stat; for a death, who fell. A
 * Shield spent by a hit is left out (how much a hit takes is an amount), and
 * so are targets: a random pick or a Shield running out moves them, while
 * the loop stays the same. */
function rowShape(log: BattleEvent[], r: LogRow, plain: NameOf, sides: Map<string, Side>): string {
  const who = (id: string) => `${plain(id)}:${sides.get(id) ?? ""}`;
  const keys = new Set<string>();
  for (const id of r.eventIds) {
    const e = log[id]!;
    const parent = e.causedBy !== null ? log[e.causedBy] : undefined;
    if (e.type === "StatusRemoved" && parent?.type === "Hurt") continue;
    // A status acts on its holder: Poison ticking is one actor, whoever holds it.
    const src = e.source === "kernel" ? `kernel<${parent?.type ?? ""}` : e.source.status !== undefined ? `/${e.source.status}` : who(e.source.unit);
    const parts: string[] = [e.type, src];
    const b = e as unknown as Record<string, unknown>;
    if (e.type === "Death") parts.push(who(e.unit));
    for (const f of ["status", "stat", "name", "original", "side"]) if (typeof b[f] === "string") parts.push(`${f}=${b[f]}`);
    keys.add(parts.join(" "));
  }
  return [...keys].sort().join("|");
}

/** Folds each turn whose rows repeat the previous turn's (the same rows in
 * the same order, the same actors and effects, amounts and targets aside)
 * into one summary row (round 4, note 4, R5): late fights loop, and the Log
 * then reads one line per run of repeated turns (T2–T4, summed). A turn of
 * one row stays a row. Every row is kept, in order, inside its fold. */
export function foldTurnsOf(log: BattleEvent[], rows: LogRow[], name: NameOf = displayNames(log)): LogItem[] {
  const plain = displayNames(log);
  const sides = sidesOf(log);
  const turns: { turn: number; rows: LogRow[]; shape: string }[] = [];
  for (const r of rows) {
    let t = turns.at(-1);
    if (!t || t.turn !== r.turn) turns.push((t = { turn: r.turn, rows: [], shape: "" }));
    t.rows.push(r);
  }
  for (const t of turns) t.shape = t.rows.map((r) => rowShape(log, r, plain, sides)).join("\n");
  const out: LogItem[] = [];
  turns.forEach((t, i) => {
    const prev = turns[i - 1];
    if (t.turn < 1 || t.rows.length < 2 || !prev || prev.turn !== t.turn - 1 || prev.shape !== t.shape) {
      out.push(...t.rows);
      return;
    }
    // A run of repeated turns is one fold: T3 repeating T2, itself folded, joins it.
    const last = out.at(-1);
    const run = last && isLogFold(last) && last.turnTo === t.turn - 1 ? last : undefined;
    const damage: Record<Side, number> = run?.damage ?? { A: 0, B: 0 };
    const heals: Record<Side, number> = run?.heals ?? { A: 0, B: 0 };
    const deaths: string[] = run?.deaths ?? [];
    let blocked = run?.blocked ?? 0;
    for (const r of t.rows)
      for (const id of r.eventIds) {
        const e = log[id]!;
        const side = "unit" in e && typeof e.unit === "string" ? sides.get(e.unit) : undefined;
        if (e.type === "Hurt" && side) {
          damage[side] += e.amount;
          blocked += e.absorbed ?? 0;
        }
        else if (e.type === "Heal" && side) heals[side] += e.amount;
        else if (e.type === "Death") deaths.push(name(e.unit));
      }
    const hurt = damage.A + damage.B, healed = heals.A + heals.B;
    const parts = [
      hurt ? `−${hurt} damage` : "no damage",
      ...(blocked ? [`${blocked} blocked`] : []),
      ...(healed ? [`+${healed} healed`] : []),
      ...(deaths.length ? [`${deaths.join(", ")} ${deaths.length > 1 ? "fall" : "falls"}`] : []),
    ];
    const rows = [...(run?.rows ?? []), ...t.rows];
    const first = run?.turn ?? t.turn;
    const times = t.turn - first + 1;
    const fold: LogFold = { turn: first, turnTo: t.turn, caption: `Same as last turn${times > 1 ? ` ×${times}` : ""}: ${parts.join(", ")} · ${rows.length} rows`, damage, blocked, heals, deaths, rows };
    if (run) out[out.length - 1] = fold;
    else out.push(fold);
  });
  return out;
}

function fxCaption(log: BattleEvent[], fx: NonNullable<LogDraft["fx"]>, name: NameOf, eventIds: number[]): string {
  const effect = (t: Took): string => {
    switch (fx.type) {
      case "Hurt":
        if (ru()) return t.amount > 0 ? `−${t.amount}${t.absorbed ? ` (${t.absorbed} поглощено)` : ""}` : t.absorbed ? `${st("Shield")} блокирует ${t.absorbed}` : "без урона";
        return t.amount > 0 ? `−${t.amount}${t.absorbed ? ` (${t.absorbed} absorbed)` : ""}` : t.absorbed ? `Shield blocks ${t.absorbed}` : "no damage";
      case "Heal":
        return `+${t.amount}`;
      case "StatChanged":
        return `${signed(t.amount)} ${statWord(fx.what)}`;
      case "StatusApplied": {
        const stats = Object.entries(t.stats).map(([k, v]) => `${signed(v)} ${statWord(k)}`);
        return `${st(fx.what)} ×${t.amount}${stats.length ? ` (${stats.join(", ")})` : ""}`;
      }
    }
  };
  const start = Math.min(...eventIds);
  let alive: Map<Side, Set<string>> | null = null;
  const list = (units: string[]): string => {
    if (units.length >= 2 && fx.actorSide) {
      alive ??= aliveBefore(log, start);
      for (const [side, set] of alive) {
        if (set.size === units.length && units.every((u) => set.has(u))) return ru() ? (side === fx.actorSide ? "всех союзников" : "всех врагов") : side === fx.actorSide ? "all allies" : "all enemies";
      }
    }
    return units.map(name).join(", ");
  };
  const saved = fx.targets.filter((u) => fx.took.get(u)!.saved !== null);
  const saves = saved.length ? ` → ${ru() ? `${st("Blessing")} спасает` : "Blessing saves"} ${saved.map(name).join(", ")} (+${saved.reduce((s, u) => s + fx.took.get(u)!.saved!, 0)})` : "";
  if (fx.tick) {
    // What each tick dealt, Shield's share included: "−4 each, 4 blocked".
    const dealt = fx.targets.map((u) => fx.took.get(u)!.amount + fx.took.get(u)!.absorbed);
    const total = dealt.reduce((s, x) => s + x, 0);
    const blocked = fx.targets.reduce((s, u) => s + fx.took.get(u)!.absorbed, 0);
    const n = fx.targets.length;
    const sign = fx.type === "Heal" ? "+" : "−";
    if (ru()) {
      const cause = st(fx.cause);
      const partsRu = [n === 1 ? `${cause} действует на ${name(fx.targets[0]!)}` : `${cause} действует на ${n} ${ruCount(n, ["юнита", "юнита", "юнитов"])}`];
      partsRu.push(n === 1 ? `${sign}${total}` : dealt.every((x) => x === dealt[0]) ? `${sign}${dealt[0]} каждому` : `${sign}${total} всего`);
      if (blocked) partsRu.push(`${blocked} заблокировано`);
      if (fx.fades) partsRu.push(n === 1 ? "спадает" : `спадает у ${fx.fades}`);
      return partsRu.join(", ") + saves;
    }
    const parts = [n === 1 ? `${fx.cause} ticks ${name(fx.targets[0]!)}` : `${fx.cause} ticks ${n} units`];
    parts.push(n === 1 ? `${sign}${total}` : dealt.every((x) => x === dealt[0]) ? `${sign}${dealt[0]} each` : `${sign}${total} in all`);
    if (blocked) parts.push(`${blocked} blocked`);
    if (fx.fades) parts.push(n === 1 ? "fades" : `${fx.fades} fade${fx.fades === 1 ? "s" : ""}`);
    return parts.join(", ") + saves;
  }
  // Targets that took the same, together: "−2 on all enemies", "Rose −5; −2 on Ace, Kit".
  // "(×N)": it landed N times on each (amounts summed); per group when that differs.
  const hits = new Set(fx.targets.map((u) => fx.took.get(u)!.hits));
  const times = hits.size === 1 ? [...hits][0]! : 1;
  const groups: { text: string; units: string[] }[] = [];
  for (const u of fx.targets) {
    const t = fx.took.get(u)!;
    const text = effect(t) + (hits.size > 1 && t.hits > 1 ? ` (×${t.hits})` : "");
    const g = groups.find((x) => x.text === text);
    if (g) g.units.push(u);
    else groups.push({ text, units: [u] });
  }
  const body = groups
    .map((g) => (fx.type === "StatusApplied" || g.units.length > 1 ? `${g.text} ${ru() ? "на" : "on"} ${list(g.units)}` : `${name(g.units[0]!)} ${g.text}`))
    .join("; ");
  return `${ru() && fx.cause === "Fatigue" ? "Усталость" : st(fx.cause)} → ${body}${saves}${times > 1 ? ` (×${times})` : ""}`;
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
    case "Death":
      return fellToFatigue(log, e) ? e.unit : (causeUnit(log, e, name) ?? e.unit);
    case "Heal":
    case "StatusApplied":
    case "StatChanged":
    case "Silenced":
      return causeUnit(log, e, name) ?? e.unit;
    case "Summon":
    case "NoRoom":
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

/** The hit that felled a unit: its Death's cause, else the last hit on it. */
function fellBy(log: BattleEvent[], death: BattleEvent & { type: "Death" }): BattleEvent | undefined {
  const cause = death.causedBy !== null ? log[death.causedBy] : undefined;
  if (cause?.type === "Hurt") return cause;
  for (let i = death.id - 1; i >= 0; i--) {
    const e = log[i];
    if (e?.type === "Hurt" && e.unit === death.unit) return e;
  }
  return undefined;
}
/** Fatigue dealt the blow that felled the unit (R3-26: its "falls" line named
 * the unit whose Shield it last held, as if that unit killed it). */
export function fellToFatigue(log: BattleEvent[], death: BattleEvent): boolean {
  if (death.type !== "Death") return false;
  const hit = fellBy(log, death);
  return hit?.causedBy !== null && hit?.causedBy !== undefined && log[hit.causedBy]?.type === "Fatigue";
}

/** One line, cause → effect, for the event `id` (plus merged follow-ups). */
export function captionOf(log: BattleEvent[], id: number, name: NameOf = displayNames(log), merged: number[] = [id], p: Perspective = {}): string {
  const e = log[id];
  if (!e) return "";
  if (ru()) return captionOfRu(log, e, name, merged, p);
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
      return `${fellToFatigue(log, e) ? "Fatigue" : causeName(log, e, name)} → ${name(e.unit)} falls`;
    case "Summon":
      return e.resurrected
        ? `${causeName(log, e, name)} → ${name(e.unit)} returns at ${e.atHp ?? e.hp} HP`
        : `${causeName(log, e, name)} → ${e.name} appears (${e.pwr}/${e.hp})`;
    case "Silenced":
      return `${causeName(log, e, name)} → ${name(e.unit)} is silenced`;
    case "Fatigue":
      // Sudden death (R4-1): the caption says why nothing blocks it.
      return e.suddenDeath ? `Sudden death: Fatigue → everyone takes ${e.amount}, nothing blocks it` : `Fatigue → everyone takes ${e.amount}`;
    case "SummonFailed":
      return `${causeName(log, e, name)} → ${e.revive && e.unit ? name(e.unit) : e.name} can't ${e.revive ? "return" : "appear"}: sudden death`;
    case "ChainCapped":
      return `Chain stopped after ${e.steps} steps`;
    case "NoRoom":
      return `${causeName(log, e, name)} → ${NO_ROOM} ${e.revive !== undefined ? `to revive ${name(e.revive)}` : `for ${e.name}`}`;
    case "Intercepted":
      // A status on the unit stopping its own act reads from the unit:
      // "Freeze on Rose → stops its strike", not "Rose (Freeze) → … on Rose".
      if (e.by.status && e.unit === e.by.unit) return `${e.by.status} on ${name(e.unit)} → stops ${stoppedText(e.original, e.unit, true, name)}`;
      return `${name(e.by.unit)}${e.by.status ? ` (${e.by.status})` : ""} → stops ${stoppedText(e.original, e.unit, false, name)}`;
    case "BattleEnd":
      // The turn cap ran out (R3-26): say why it is a draw.
      return e.timeUp ? TIME_UP_CAPTION : endCaption(e.winner, p);
    default:
      return e.type;
  }
}

/** captionOf in Russian (M4-3): the same captions, the same "→" and numbers,
 * Russian words. Unit names stay as they are (M4-4). */
function captionOfRu(log: BattleEvent[], e: BattleEvent, name: NameOf, merged: number[], p: Perspective): string {
  switch (e.type) {
    case "Hurt": {
      const by = e.causedBy !== null ? log[e.causedBy] : undefined;
      const absorbed = e.absorbed ? ` (${e.absorbed} поглощено)` : "";
      const effect = e.amount > 0 ? `−${e.amount}${absorbed}` : e.absorbed ? `${st("Shield")} блокирует ${e.absorbed}` : "без урона";
      if (e.source === "kernel" && by?.type === "Strike") return `${name(by.striker)} бьёт ${name(e.unit)} → ${effect}`;
      return `${causeName(log, e, name)} → ${name(e.unit)} ${effect}`;
    }
    case "Heal":
      return `${causeName(log, e, name)} → ${name(e.unit)} +${e.amount}`;
    case "StatusApplied": {
      const stat = merged
        .map((i) => log[i])
        .flatMap((m) => (m && m.type === "StatChanged" ? [`${m.delta >= 0 ? "+" : "−"}${Math.abs(m.delta)} ${statWord(m.stat)}`] : []));
      return `${causeName(log, e, name)} → ${st(e.status)} ×${e.stacks} на ${name(e.unit)}${stat.length ? ` (${stat.join(", ")})` : ""}`;
    }
    case "StatusRemoved":
      return `${st(e.status)} спадает с ${name(e.unit)}${e.remaining > 0 ? ` (осталось ${e.remaining})` : ""}`;
    case "StatChanged":
      return `${causeName(log, e, name)} → ${name(e.unit)} ${e.delta >= 0 ? "+" : "−"}${Math.abs(e.delta)} ${statWord(e.stat)}`;
    case "Death":
      return `${fellToFatigue(log, e) ? "Усталость" : causeName(log, e, name)} → ${name(e.unit)} падает`;
    case "Summon":
      return e.resurrected
        ? `${causeName(log, e, name)} → ${name(e.unit)} возвращается с ${e.atHp ?? e.hp} ${RU_STAT.hp}`
        : `${causeName(log, e, name)} → ${e.name} появляется (${e.pwr}/${e.hp})`;
    case "Silenced":
      return `${causeName(log, e, name)} → ${name(e.unit)} заглушён`;
    case "Fatigue":
      return e.suddenDeath ? `Внезапная смерть: Усталость → все получают ${e.amount}, ничто не блокирует` : `Усталость → все получают ${e.amount}`;
    case "SummonFailed":
      return `${causeName(log, e, name)} → ${e.revive && e.unit ? name(e.unit) : e.name} не может ${e.revive ? "вернуться" : "появиться"}: внезапная смерть`;
    case "ChainCapped":
      return `Цепь прервана после ${e.steps} ${ruCount(e.steps, ["шага", "шагов", "шагов"])}`;
    case "NoRoom":
      return `${causeName(log, e, name)} → ${NO_ROOM_RU} ${e.revive !== undefined ? `для воскрешения ${name(e.revive)}` : `для ${e.name}`}`;
    case "Intercepted":
      if (e.by.status && e.unit === e.by.unit) return `${st(e.by.status)} на ${name(e.unit)} → останавливает ${stoppedTextRu(e.original, e.unit, true, name)}`;
      return `${name(e.by.unit)}${e.by.status ? ` (${st(e.by.status)})` : ""} → останавливает ${stoppedTextRu(e.original, e.unit, false, name)}`;
    case "BattleEnd":
      return e.timeUp ? TIME_UP_CAPTION_RU : endCaption(e.winner, p);
    default:
      return e.type;
  }
}

/** stoppedText in Russian: "атаку", "удар по Rose", "атаку Rose". */
function stoppedTextRu(original: string, unit: string | undefined, own: boolean, name: NameOf): string {
  const n = unit ? name(unit) : "";
  const of = unit && !own ? ` ${n}` : "";
  switch (original) {
    case "Strike":
      return `атаку${of}`;
    case "Death":
      return `смерть${of}`;
    case "Hurt":
      return unit && !own ? `удар по ${n}` : "удар";
    case "Heal":
      return unit && !own ? `лечение для ${n}` : "лечение";
    case "StatusApplied":
      return unit && !own ? `статус на ${n}` : "статус";
    default:
      return `${original}${unit && !own ? ` на ${n}` : ""}`;
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
 * units behind it, and ranked by impact. Your own units' acts are not enemy
 * chains. Fatigue is a row of its own ("Fatigue") once it killed one of your
 * units, listed even when three chains did more (R3-26: it felled 3 of 5 and
 * the card didn't say so). */
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
  // Fatigue's hits on your units, and the kills they made.
  const fatigue: LossChain = { names: ["Fatigue"], units: [], text: "Fatigue", impact: 0, damage: 0, heal: 0, kills: 0, times: 0, hits: 0, heals: 0, sampleEventId: -1, sampleKind: "none" };
  const byFatigue = (e: BattleEvent) => e.causedBy !== null && log[e.causedBy]?.type === "Fatigue";
  const add = (e: BattleEvent, dmg: number, heal: number, kill: boolean) => {
    if (e.type === "Hurt" && byFatigue(e)) {
      if (dmg > 0) offer(fatigue, e.id, "hit", dmg);
      dealt.set(e.id, dmg);
      fatigue.damage += dmg;
      fatigue.impact += dmg;
      fatigue.times++;
      fatigue.hits++;
      return;
    }
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
      if (fellToFatigue(log, e)) {
        const hit = fellBy(log, e as BattleEvent & { type: "Death" })!;
        fatigue.kills++;
        offer(fatigue, hit.id, "kill", dealt.get(hit.id) ?? 0);
      } else if (cause) {
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
  const ranked = all.sort((a, b) => b.impact - a.impact || b.kills - a.kills);
  if (!fatigue.kills) return ranked.slice(0, top);
  const out = [...ranked, fatigue].sort((a, b) => b.impact - a.impact || b.kills - a.kills).slice(0, top);
  return out.includes(fatigue) ? out : [...out.slice(0, Math.max(0, top - 1)), fatigue];
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
    if (k.fatigue) return `Fatigue kills ${n(k.victim)}`;
    if (!k.killer) return `${n(k.victim)} falls`;
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
              ? `${summoned[0]!.back ? "brings back" : "summons"} ${n(summoned[0]!.unit)}${summoned.length > 1 ? `, +${summoned.length - 1} more` : ""}`
              : blocked
                ? `${blocked} blocked by Shield`
                : null;
      if (what(plain) === null) return { kind: "combo", beat: b.index, label: fit((n) => `${n(actor)}: ${steps} steps`), unit: actor };
      return { kind: "combo", beat: b.index, label: fit((n) => `${n(actor)}: ${what(n)} in ${steps} steps`, (n) => `${n(actor)}: ${what(n)}`), unit: actor };
    }
    const [u, d] = top;
    const victims = (n: NameOf) => `kills ${n(d.kills[0]!)}${d.kills.length > 1 ? `, +${d.kills.length - 1} more` : ""}`;
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
    // "+3" alone read as a heal (R3-26): "+3 more fell".
    const label = fk.length ? fit((n) => `Fatigue kills ${n(fk[0]!.victim)}${fk.length > 1 ? `, +${fk.length - 1} more fell` : ""}`) : `Fatigue sets in: ${total} dmg, no kills`;
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
export function turnLabel(turn: number, to = turn): string {
  return turn >= 1 ? (to > turn ? `T${turn}–T${to}` : `T${turn}`) : "Start";
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


/** The turn timeline: one block per turn, each with its beats and its marks
 * (deaths, big hits, fatigue setting in). Pure, like boardAt: scrubbing to a
 * block's beat is go(beat). Beats before the first turn (battle start) get a
 * block of their own, turn 0, which the viewer labels "Start" (R2-17). */
export function timelineOf(log: BattleEvent[], beats: PlayBeat[], sides = sidesOf(log)): TimelineTurn[] {
  const hits = log.flatMap((e) => (e.type === "Hurt" ? [e.amount] : []));
  // A mark for a big hit: BIG_HIT_MIN, or 60% of the battle's biggest when that is more.
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

// ---------- per-turn totals (round 4, R4-6) ----------

/** What a turn did to one unit, summed over every beat of that turn. */
export interface UnitTurnTotals {
  unit: string;
  side: Side | null;
  /** HP lost to hits, as the hits read ("−n"), Shield's share not counted. */
  damage: number;
  healed: number;
  /** Net PWR and HP stat changes (StatChanged), signed. */
  pwr: number;
  hp: number;
  /** What Shield took off hits. */
  blocked: number;
  /** Net stacks gained (+) or lost (−) per status, in the order they first
   * moved; a status that came and went to net 0 is left out. */
  statuses: { status: string; stacks: number }[];
  died: boolean;
  /** The events summed, in log order: the totals' traces. */
  eventIds: number[];
}

/** One turn's totals (turn 0: the battle's start, as turnLabel reads it). */
export interface TurnSummary {
  turn: number;
  /** The turn's beats (PlayBeat.index), in order; the summary shows after the last. */
  beats: number[];
  /** Each unit the turn changed, in the order it was first changed. */
  units: UnitTurnTotals[];
}

/** Per unit per turn: damage taken, healing, PWR/HP change, Shield blocked,
 * net status stacks, and who died (R4-6, the turn-end summary's numbers).
 * Pure over the log and its playback beats: a turn is the events its beats
 * reveal (start..end), grouped as the timeline groups them. */
export function turnSummaryOf(log: BattleEvent[], beats: PlayBeat[], sides = sidesOf(log)): TurnSummary[] {
  const turns: TurnSummary[] = [];
  const seen = new Set<number>();
  let byUnit = new Map<string, UnitTurnTotals>();
  for (const b of beats) {
    const turn = Math.max(0, b.turn);
    let t = turns.at(-1);
    if (!t || turn > t.turn) {
      t = { turn, beats: [], units: [] };
      turns.push(t);
      byUnit = new Map();
    }
    t.beats.push(b.index);
    for (let id = b.start; id <= b.end; id++) {
      const e = log[id];
      if (!e || seen.has(id)) continue;
      seen.add(id);
      if (e.type !== "Hurt" && e.type !== "Heal" && e.type !== "StatChanged" && e.type !== "StatusApplied" && e.type !== "StatusRemoved" && e.type !== "Death") continue;
      let u = byUnit.get(e.unit);
      if (!u) {
        u = { unit: e.unit, side: sides.get(e.unit) ?? null, damage: 0, healed: 0, pwr: 0, hp: 0, blocked: 0, statuses: [], died: false, eventIds: [] };
        byUnit.set(e.unit, u);
        t.units.push(u);
      }
      u.eventIds.push(id);
      if (e.type === "Hurt") { u.damage += e.amount; u.blocked += e.absorbed ?? 0; }
      else if (e.type === "Heal") u.healed += e.amount;
      else if (e.type === "StatChanged") u[e.stat] += e.delta;
      else if (e.type === "Death") u.died = true;
      else {
        let st = u.statuses.find((x) => x.status === e.status);
        if (!st) u.statuses.push((st = { status: e.status, stacks: 0 }));
        st.stacks += e.type === "StatusApplied" ? e.stacks : -e.stacks;
      }
    }
  }
  for (const t of turns) {
    for (const u of t.units) u.statuses = u.statuses.filter((x) => x.stacks !== 0);
    t.units = t.units.filter((u) => u.damage || u.healed || u.pwr || u.hp || u.blocked || u.statuses.length || u.died);
  }
  return turns;
}

// ---------- the turn-end summary (round 4, R4-12) ----------

/** How long playback holds on a turn's totals at 1×, after the turn's last
 * beat (R4-12, viewer.md (6)): divided by the speed, and none from
 * TURN_END_SKIP_SPEED up. */
export const TURN_END_MS = 1200;
export const TURN_END_SKIP_SPEED = 4;

/** The hold at `speed`, in real ms. */
export function turnEndHoldMs(speed: number): number {
  return speed >= TURN_END_SKIP_SPEED ? 0 : Math.round(TURN_END_MS / speed);
}

/** The beats playback holds after, by beat index, each with its turn's
 * totals: a turn's last beat, when the turn changed a unit and the battle
 * goes on after it (the last beat opens the end card instead). */
export function turnEndsOf(turns: TurnSummary[], beatCount: number): Map<number, TurnSummary> {
  const out = new Map<number, TurnSummary>();
  for (const t of turns) {
    const last = t.beats.at(-1);
    if (last !== undefined && last < beatCount - 1 && t.units.length) out.set(last, t);
  }
  return out;
}

/** One piece of a unit's turn-end label: a number ("−7", "+2", "+1/+2",
 * "+1 PWR"), what Shield blocked, or a status's net stacks. */
export interface TotalsPart {
  kind: "damage" | "heal" | "buff" | "debuff" | "blocked" | "status";
  text: string;
  status?: string;
  stacks?: number;
}

/** A unit's turn totals as its label reads them, in order: damage, healing,
 * PWR/HP, Shield blocked, then each status's net stacks. Shield spent on the
 * blocked hits is not listed again. */
export function totalsPartsOf(u: UnitTurnTotals): TotalsPart[] {
  const sign = (n: number) => (n > 0 ? `+${n}` : `−${-n}`);
  const parts: TotalsPart[] = [];
  if (u.damage) parts.push({ kind: "damage", text: `−${u.damage}` });
  if (u.healed) parts.push({ kind: "heal", text: `+${u.healed}` });
  if (u.pwr || u.hp) {
    const kind = u.pwr + u.hp >= 0 ? "buff" : "debuff";
    parts.push({ kind, text: u.pwr && u.hp ? `${sign(u.pwr)}/${sign(u.hp)}` : u.pwr ? `${sign(u.pwr)} PWR` : `${sign(u.hp)} HP` });
  }
  if (u.blocked) parts.push({ kind: "blocked", text: `${u.blocked}` });
  for (const st of u.statuses) {
    if (st.status === "Shield" && st.stacks < 0 && u.blocked) continue;
    parts.push({ kind: "status", text: st.stacks > 0 ? `×${st.stacks}` : `−${-st.stacks}`, status: st.status, stacks: st.stacks });
  }
  return parts;
}

/** A unit's turn totals in words, for reduced motion's caption and the
 * label's aria-label: "−7 · +2 · Shield ×3", "✝" first for the fallen. */
export function totalsText(u: UnitTurnTotals): string {
  const words = totalsPartsOf(u).map((p) => (p.kind === "blocked" ? `${p.text} blocked` : p.kind === "status" ? `${p.status} ${p.text}` : p.text));
  return [...(u.died ? ["✝"] : []), ...words].join(" · ");
}

// ---------- running totals above each unit (R4-22) ----------

/** One row above a unit: what one kind of change has summed to so far this
 * turn. Its key fixes its place: rows keep the order their kinds first
 * showed in, and a row stays for the rest of the turn once shown. */
export interface RunRow {
  /** "damage", "heal", "pwr", "hp", "blocked", or "status:<name>". */
  key: string;
  kind: "damage" | "heal" | "buff" | "debuff" | "blocked" | "status";
  /** The signed sum: damage and blocked as positive amounts, a status's net stacks. */
  value: number;
  status?: string;
  /** The events summed, in log order (a row's tap traces the latest). */
  eventIds: number[];
}

/** Each unit's running rows over `ids` (one turn's events landed so far, any
 * order), in the order units were first changed. Shield taken off by a hit
 * it blocked counts in "blocked", not again in Shield's row. */
export function runningTotalsOf(log: BattleEvent[], ids: Iterable<number>): Map<string, RunRow[]> {
  const out = new Map<string, RunRow[]>();
  const add = (unit: string, key: string, kind: RunRow["kind"], delta: number, id: number, status?: string) => {
    let rows = out.get(unit);
    let r = rows?.find((x) => x.key === key);
    if (!r) {
      if (!delta) return;
      if (!rows) out.set(unit, (rows = []));
      rows.push((r = { key, kind, value: 0, ...(status ? { status } : {}), eventIds: [] }));
    }
    r.value += delta;
    r.eventIds.push(id);
    if (key === "pwr" || key === "hp") r.kind = r.value >= 0 ? "buff" : "debuff";
  };
  for (const id of [...new Set(ids)].sort((a, b) => a - b)) {
    const e = log[id];
    if (!e) continue;
    if (e.type === "Hurt") {
      add(e.unit, "damage", "damage", e.amount, id);
      add(e.unit, "blocked", "blocked", e.absorbed ?? 0, id);
    } else if (e.type === "Heal") add(e.unit, "heal", "heal", e.amount, id);
    else if (e.type === "StatChanged") add(e.unit, e.stat, e.delta >= 0 ? "buff" : "debuff", e.delta, id);
    else if (e.type === "StatusApplied") add(e.unit, `status:${e.status}`, "status", e.stacks, id, e.status);
    else if (e.type === "StatusRemoved") {
      const by = e.causedBy !== null ? log[e.causedBy] : undefined;
      if (by?.type === "Hurt" && by.unit === e.unit && (by.absorbed ?? 0) > 0 && e.status === "Shield") continue;
      add(e.unit, `status:${e.status}`, "status", -e.stacks, id, e.status);
    }
  }
  return out;
}

/** A running row's number as it reads: "−7", "+2", "+1 PWR", "−1 HP", "3" (blocked), "×2" / "−1" (a status). */
export function runRowText(kind: RunRow["kind"], key: string, value: number): string {
  const sign = (n: number) => (n > 0 ? `+${n}` : n < 0 ? `−${-n}` : "0");
  if (kind === "damage") return `−${value}`;
  if (kind === "heal") return `+${value}`;
  if (key === "pwr") return `${sign(value)} PWR`;
  if (key === "hp") return `${sign(value)} HP`;
  if (kind === "blocked") return `${value}`;
  return value > 0 ? `×${value}` : value < 0 ? `−${-value}` : "0";
}

/** Each beat's own events (R4-22): those its waves show, and the rest of
 * its range that no beat's wave shows. A wave can show an event from outside
 * its beat's range (a death folded into the hit before it): that event is
 * the showing beat's, so a running total or a fallen unit never waits for,
 * or outlives, the beat that showed it. */
export function beatIdsOf(beats: PlayBeat[]): number[][] {
  const owner = new Map<number, number>();
  for (const b of beats) for (let id = b.start; id <= b.end; id++) if (!owner.has(id)) owner.set(id, b.index);
  for (const b of beats) for (const w of b.waves) for (const id of w.eventIds) owner.set(id, b.index);
  const out: number[][] = beats.map(() => []);
  for (const [id, i] of [...owner].sort((p, q) => p[0] - q[0])) out[i]?.push(id);
  return out;
}

/** The ids of a turn's events landed by beat `at`: every earlier beat of the
 * turn whole, and those of beat `at` that `landed` says are on screen. */
export function turnSoFarIds(beatIds: number[][], turn: TurnSummary | undefined, at: number, landed: (id: number) => boolean): number[] {
  const ids: number[] = [];
  for (const i of turn?.beats ?? []) {
    if (i > at) break;
    for (const id of beatIds[i] ?? []) if (i < at || landed(id)) ids.push(id);
  }
  return ids;
}

/** What a beam shows (round 3, R3-21, battle.md (11)): the effect's kind, for its colour. */
export type BeamKind = "damage" | "heal" | "buff" | "debuff" | "status" | "summon" | "silence";

/** One beam of a wave: from the unit that acted (or the clash, for fatigue
 * and a capped chain) to a unit it changed. from === to is a self-target
 * (a Poison tick, a unit buffing itself): drawn as a ring on the card. */
export interface Beam {
  from: string;
  to: string;
  kind: BeamKind;
  /** The status put on or taken off, for its colour. */
  status?: string;
}

/** The beams a wave draws: one per unit it changed, from the unit its
 * cause sits on (causeOf().at), in the kind of that unit's first change.
 * A group effect is a fan from one source; a death wave draws none (the
 * blow that killed already drew its beam); the battle's end has none. */
export function beamsOf(log: BattleEvent[], step: Pick<Step, "eventIds" | "changes">, whenOf?: WhenOf): Beam[] {
  const first = log[step.eventIds[0]!];
  if (!first || first.type === "Death") return [];
  const cause = causeOf(log, step, whenOf);
  if (!cause) return [];
  const beams: Beam[] = [];
  const seen = new Set<string>();
  for (const c of step.changes) {
    if (c.kind === "death" || seen.has(c.unit)) continue;
    seen.add(c.unit);
    const e = log[c.eventId];
    const status = e?.type === "StatusApplied" || e?.type === "StatusRemoved" ? e.status : undefined;
    beams.push({ from: cause.at, to: c.unit, kind: c.kind as BeamKind, ...(status ? { status } : {}) });
  }
  return beams;
}
