// Beat segmentation — the replay's narrative spine. A *beat* is one root event
// plus everything it causally spawns: the kernel fully resolves a strike's (or
// a turn-end's) cascade before the next root, so the linear log already
// partitions cleanly — a beat runs from a root up to (not including) the next
// root. Pure projection over the log, DOM-free, sat beside boardAt/ancestry so
// the viewer reads it instead of re-deriving turn structure (SPEC §0.3:
// observability is structural).

import type { BattleEvent, EventType } from "./types.js";

/** The "spontaneous"/structural events that open a beat. Everything else
 * (Hurt, Heal, Death, status/stat changes, Summon, Silenced, Intercepted,
 * ChainBlocked) is a *caused* consequence and belongs to its root's beat.
 * Classification is by kind, not by `causedBy === null`: the kernel hangs a
 * turn's strikes off the TurnStart, but each strike is its own beat. */
const ROOT_KINDS: ReadonlySet<EventType> = new Set<EventType>([
  "BattleStart",
  "TurnStart",
  "TurnEnd",
  "PairFaced",
  "Strike",
  "Fatigue",
  "BattleEnd",
]);

/** Caused events that actually touch a hero — their presence makes a beat
 * worth a card rather than a divider. Turn structure and pure trace events
 * (ChainBlocked, Intercepted) leave the heroes untouched. */
const HERO_EFFECT_KINDS: ReadonlySet<EventType> = new Set<EventType>([
  "Hurt",
  "Heal",
  "Death",
  "Summon",
  "StatusApplied",
  "StatusRemoved",
  "StatChanged",
  "Silenced",
]);

export function isRootKind(type: EventType): boolean {
  return ROOT_KINDS.has(type);
}

export interface Beat {
  /** Index of this beat in the segmentation (0-based). */
  index: number;
  /** The root event that opens the beat. */
  root: BattleEvent;
  /** The root event's kind — what classifies the beat (Strike, TurnStart, …). */
  kind: EventType;
  /** First event id in the beat (the root's id). */
  start: number;
  /** Last event id in the beat (inclusive) — the event before the next root,
   * or the final event of the log for the last beat. */
  end: number;
  /** The caused events of this beat, in log order (everything after the root
   * up to `end`). Empty for a beat whose root spawned nothing. */
  caused: BattleEvent[];
  /** No caused event touches a hero → render a "turn N" divider, not a card.
   * A BattleStart that only summons, a Strike that lands a Hurt, a TurnEnd
   * poison tick are all hero-affecting; a bare TurnStart/TurnEnd/PairFaced is
   * structural-only. */
  structural: boolean;
}

/**
 * Partition the log into beats. Pure: same log in, same beats out — scrubbing
 * stays a function of `(log, step)` because the segmentation never depends on
 * the playhead. A log with no events yields no beats; a log whose first event
 * is not a root still opens a beat at index 0 (defensive — real kernel logs
 * always start with BattleStart).
 */
export function beatsOf(log: BattleEvent[]): Beat[] {
  const beats: Beat[] = [];
  let i = 0;
  while (i < log.length) {
    const root = log[i]!;
    // Walk forward over caused (non-root) events until the next root.
    let j = i + 1;
    while (j < log.length && !ROOT_KINDS.has(log[j]!.type)) j++;
    const caused = log.slice(i + 1, j);
    const structural = !caused.some((e) => HERO_EFFECT_KINDS.has(e.type));
    beats.push({
      index: beats.length,
      root,
      kind: root.type,
      start: root.id,
      end: j - 1,
      caused,
      structural,
    });
    i = j;
  }
  return beats;
}

/**
 * The beat containing event `step`, and the event's position within it. Used
 * by the viewer to drive the card: which beat is open, and how many of its
 * lines have revealed (every beat event with id ≤ step). Returns undefined
 * for an out-of-range step or an empty log.
 */
export function beatAtStep(beats: Beat[], step: number): { beat: Beat; revealedThrough: number } | undefined {
  for (const beat of beats) {
    if (step >= beat.start && step <= beat.end) return { beat, revealedThrough: step };
  }
  return undefined;
}

/**
 * Within-beat causal nesting depth of an event: how many `causedBy` hops back
 * to the beat root (0 = the root itself, 1 = directly caused by the root, …).
 * Derived purely from `causedBy`; the chain is followed only while it stays
 * inside the beat, so a stray cross-beat link can never deepen a line. An
 * event not in the beat returns 0.
 */
export function depthInBeat(beat: Beat, log: BattleEvent[], id: number): number {
  if (id < beat.start || id > beat.end) return 0;
  let depth = 0;
  let cur: number | null = id;
  // Guard against a malformed cycle with a hop cap of the beat's own length.
  const cap = beat.end - beat.start + 1;
  while (cur !== null && cur !== beat.root.id && depth <= cap) {
    const e: BattleEvent | undefined = log[cur];
    if (!e) break;
    cur = e.causedBy;
    depth++;
    // A parent outside the beat means we've left the local tree — stop here so
    // the depth is the distance to the beat boundary, not across it.
    if (cur !== null && (cur < beat.start || cur > beat.end)) break;
  }
  return depth;
}
