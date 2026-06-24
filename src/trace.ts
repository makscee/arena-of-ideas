// Causal trace helpers — shared narration over the log's `causedBy` links.
// The text replay and the web viewer both answer "why did this happen" with
// these functions; the walking/narration logic lives here once (SPEC §0.3:
// observability is structural). Pure functions over the log, no state.

import type { AbilityRef, BattleEvent } from "./types.js";

/** Resolves a unit instance id to its display name. */
export type NameOf = (unitId: string) => string;

/**
 * Display-name resolver for a log: the unit's bare NAME, never the instance-id
 * prefix (#065 item 3). An instance id is `A1:Brawler` / `B3:Silencer` (battle.ts
 * stamps `${side}${i}:${name}`); the leading `Xn:` is a DATA-layer disambiguator
 * (the kernel event unit ids, the board cards' `data-unit`, probe selectors, the
 * cause-trace keying) and must never leak into player-facing text. The player
 * reads just "Brawler" / "Silencer"; when two units share a name the TEAM TINT
 * (board card + battle log) is what tells the two sides apart, not an id prefix.
 *
 * So this strips the prefix unconditionally: a name claimed by the log resolves
 * to that bare name (`A1:Brawler` → "Brawler"); an unknown id still passes
 * through verbatim (it is no longer a player-facing unit, e.g. a stale ref).
 */
export function displayNames(log: BattleEvent[]): NameOf {
  const display = new Map<string, string>(); // instance id → bare name
  const claim = (id: string, name: string) => display.set(id, name);
  for (const e of log) {
    if (e.type === "BattleStart") {
      for (const side of ["A", "B"] as const) for (const r of e.teams[side]) claim(r.id, r.name);
    } else if (e.type === "Summon") {
      claim(e.unit, e.name);
    }
  }
  return (id) => display.get(id) ?? id;
}

/** The `causedBy` ancestor chain of an event: direct parent first, root (kernel beat) last. */
export function ancestry(log: BattleEvent[], id: number): BattleEvent[] {
  const chain: BattleEvent[] = [];
  let cur = log[id]?.causedBy ?? null;
  while (cur !== null) {
    const e = log[cur];
    if (!e) break;
    chain.push(e);
    cur = e.causedBy;
  }
  return chain;
}

/** "Poison on Orc" for status abilities, "Witch's ability" for unit abilities. */
export function abilityRefDesc(ref: AbilityRef, name: NameOf): string {
  return ref.status !== undefined ? `${ref.status} on ${name(ref.unit)}` : `${name(ref.unit)}'s ability`;
}

/** One-clause description of an event, for chain narration. */
export function shortDesc(e: BattleEvent | undefined, name: NameOf): string {
  if (!e) return "an earlier event";
  switch (e.type) {
    case "Hurt":
      return `${name(e.unit)} was hurt`;
    case "Heal":
      return `${name(e.unit)} healed`;
    case "Death":
      return `${name(e.unit)} died`;
    case "Strike":
      return `${name(e.striker)}'s strike`;
    case "Summon":
      return `${name(e.unit)} appeared`;
    case "StatusApplied":
      return `${e.status} landed on ${name(e.unit)}`;
    case "StatusRemoved":
      return `${e.status} left ${name(e.unit)}`;
    case "TurnStart":
      return "the turn began";
    case "TurnEnd":
      return "the turn ended";
    case "Fatigue":
      return "fatigue struck";
    default:
      return `a ${e.type} event`;
  }
}

/**
 * Walk `causedBy` ancestry from a death's proximate cause and narrate it
 * compactly: "Poison tick (3 dmg) ← Poison applied turn 4 by Witch".
 */
export function deathCauseChain(log: BattleEvent[], startId: number, name: NameOf): string[] {
  const parts: string[] = [];
  let id: number | null = startId;
  for (let hops = 0; id !== null && hops < 6; hops++) {
    const e: BattleEvent | undefined = log[id];
    if (!e) break;
    switch (e.type) {
      case "Hurt": {
        if (e.source !== "kernel" && e.source.status !== undefined) {
          parts.push(`${e.source.status} tick (${e.amount} dmg)`);
          const origin = statusOrigin(log, e.unit, e.source.status, e.id, name);
          if (origin) parts.push(origin);
          return parts;
        }
        if (e.source !== "kernel") {
          parts.push(`hit by ${name(e.source.unit)}'s ability (${e.amount} dmg)`);
          return parts;
        }
        const parent = e.causedBy !== null ? log[e.causedBy] : undefined;
        if (parent?.type === "Strike") {
          parts.push(`struck by ${name(parent.striker)} for ${e.amount}`);
          return parts;
        }
        if (parent?.type === "Fatigue") {
          parts.push(`fatigue (${e.amount} dmg)`);
          return parts;
        }
        parts.push(`${e.amount} damage`);
        id = e.causedBy;
        continue;
      }
      case "StatChanged":
        parts.push(`max ${e.stat} ${e.delta >= 0 ? "rose" : "fell"} to ${e.now}`);
        id = e.causedBy;
        continue;
      case "StatusRemoved": {
        const by = e.source !== "kernel" && e.source.unit !== e.unit ? ` by ${name(e.source.unit)}` : "";
        parts.push(`${e.status} stripped${by}`);
        id = e.causedBy;
        continue;
      }
      case "Heal":
        parts.push(`after healing ${e.amount}`);
        id = e.causedBy;
        continue;
      default:
        return parts; // turn structure and the rest add no story to a death
    }
  }
  return parts;
}

/** Where did this status come from? The most recent application before the given event. */
function statusOrigin(
  log: BattleEvent[],
  unitId: string,
  status: string,
  beforeId: number,
  name: NameOf,
): string | undefined {
  for (let i = beforeId - 1; i >= 0; i--) {
    const e = log[i];
    if (e && e.type === "StatusApplied" && e.unit === unitId && e.status === status) {
      if (e.turn === 0) return `${status} carried from the start`;
      const by = e.source !== "kernel" ? ` by ${name(e.source.unit)}` : "";
      return `${status} applied turn ${e.turn}${by}`;
    }
  }
  return undefined;
}
