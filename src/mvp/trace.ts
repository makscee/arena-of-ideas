// Battle viewer logic (mission #574, slice 9): playback steps with one-line
// captions, tap-a-change-to-trace-its-chain, and "why I lost". Pure functions
// over the causal log; the phone client (mobile/screens/battle.ts) only draws
// what these return. "Why did this happen" is always a walk up `causedBy`.

import { isRootKind } from "../beats.js";
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

export function changeOf(e: BattleEvent): Change | null {
  if (!CHANGE_TYPES.has(e.type)) return null;
  switch (e.type) {
    case "Hurt":
      return { eventId: e.id, unit: e.unit, kind: "damage", label: `−${e.amount}` };
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
      const absorbed = e.absorbed ? ` (${e.absorbed} absorbed)` : "";
      if (e.source === "kernel" && p?.type === "Strike") return `${name(p.striker)} strikes ${name(e.unit)} → −${e.amount}${absorbed}`;
      return `${causeName(log, e, name)} → ${name(e.unit)} −${e.amount}${absorbed}`;
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
