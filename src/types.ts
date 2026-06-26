// DSL v1 — rules are data; the engine is an interpreter over this schema.
// SPEC.md is normative; where this file and SPEC disagree, one is a bug to fix consciously.

export type Side = "A" | "B";
export type StatName = "hp" | "pwr";

export interface Stats {
  hp: number;
  pwr: number;
}

// ---------- Content: units, abilities, statuses ----------

export interface UnitDef {
  name: string;
  base: Stats;
  level?: number;
  /** The unit's single ability, by id into the AbilityRegistry — the ontology's
   * spine (PRD #081): a Unit references exactly one named Ability, and that
   * Ability's `family` is the unit's color. Optional only during the #081
   * migration; `abilities?` below is the legacy inline path being retired. */
  ability?: string;
  /** @deprecated #081 — inline abilities are being migrated to a single
   * `ability` ref. The battle resolver still reads this behind a back-compat
   * read while the corpus migrates; dropped once every unit carries a ref. */
  abilities?: Ability[];
  /** Initial statuses, applied (as events) right after BattleStart. Names resolve via the status registry. */
  statuses?: { status: string; stacks: number }[];
}

export interface Ability {
  whens: When[]; // ≥1; each matching when fires independently
  condition?: Condition; // checked at fire time
  selectors: Selector[]; // ≥1; effect sequence applies once per selected target, per selector
  effects: Effect[]; // ≥1; run in sequence order
}

/** The color axis (PRD #081): every Ability declares one family, and a Unit's
 * color is its Ability's family — derived, never stored on the unit. The 7
 * families and their hexes are pinned by the mockup; the family→hex palette is
 * centralized in tunables.ts (FAMILY_HEX). */
export type Family = "Poison" | "Strike" | "Shield" | "Summon" | "Arcane" | "Control" | "Heal";

/** A named, referenceable Ability — today's inline `Ability` shape plus a `name`
 * and a `family`. Mirrors `StatusDef`/`StatusRegistry`: the AbilityRegistry is
 * the unit-ability analogue of the status registry. A Unit references exactly
 * one of these by id (`UnitDef.ability`). */
export interface AbilityDef extends Ability {
  name: string;
  family: Family;
}

export type AbilityRegistry = Record<string, AbilityDef>;

export interface When {
  kind: "trigger" | "interceptor";
  on: EventPattern;
}

/** Unit filters are relative to the ability's holder. "ally" includes the holder's side (and the holder). */
export type UnitFilter = "holder" | "ally" | "enemy" | "any";

export type EventPattern =
  | { on: "BattleStart" | "TurnStart" | "TurnEnd" }
  | { on: "Strike"; striker?: UnitFilter }
  | { on: "Hurt" | "Heal" | "Death" | "Summon"; unit?: UnitFilter }
  | { on: "StatusApplied" | "StatusRemoved"; unit?: UnitFilter; status?: string };

export type Condition = { kind: "holderHpAtMost"; value: number };

export type Selector =
  | { kind: "holder" } // the owning unit; resolves even if it just died (on-death abilities)
  | { kind: "eventUnit" } // the subject unit of the triggering event (striker for Strike)
  | { kind: "frontEnemy" }
  | { kind: "allEnemies" }
  | { kind: "allAllies" } // living, includes holder
  | { kind: "randomEnemy" } // one seeded draw whenever ≥1 candidate
  | { kind: "lastDeadAlly" }; // most recent graveyard entry still dead

/** Magnitude expressions. Stat scaling is opt-in content, priced by the budget (not v1). */
export type Amount =
  | { kind: "const"; value: number }
  | { kind: "stat"; stat: StatName; of: "holder" } // holder's effective stat
  | { kind: "level"; of: "holder" } // holder's level — shop-layer growth, readable like a stat
  | { kind: "stacks" }; // current stacks of the owning status instance (status content only)

export type Effect =
  // trigger-context atoms
  | { kind: "damage"; amount: Amount }
  | { kind: "heal"; amount: Amount }
  | { kind: "applyStatus"; status: string; stacks: Amount }
  | { kind: "consumeStacks"; status?: string; stacks: Amount } // status omitted = the owning status
  | { kind: "summon"; unit: UnitDef } // at the back of the target's team; skipped if line is full
  | { kind: "silence" } // remove all statuses, disable the unit's own abilities for the battle
  | { kind: "resurrect"; hp: Amount } // revive the (dead) target at N hp, back of line
  // interceptor-context atoms — transform/cancel the proposed event
  | { kind: "cancel"; consumeSelf?: number } // cancel the event; optionally consume own stacks
  | { kind: "absorbHurt" } // reduce a proposed Hurt by min(own stacks, amount); consume = absorbed
  | { kind: "preventDeathHeal"; toHp: Amount; removeSelf?: boolean }; // cancel a Death, heal to N hp

export interface StatusDef {
  name: string;
  /** Contribution per stack while attached. Effective stat = max(0, base + Σ contributions) — computed, never baked. */
  statMods?: Partial<Stats>;
  abilities: Ability[];
}

export type StatusRegistry = Record<string, StatusDef>;

// ---------- Events: the causal log ----------

export interface AbilityRef {
  unit: string; // holding unit instance id
  status?: string; // present when the ability lives on a status
  ability: number; // index within the ability list
}

export type SourceRef = "kernel" | AbilityRef;

export interface RosterEntry {
  id: string;
  name: string;
  hp: number;
  pwr: number;
}

export type EventBody =
  | { type: "BattleStart"; teams: { A: RosterEntry[]; B: RosterEntry[] } }
  | { type: "TurnStart" }
  | { type: "TurnEnd" }
  | { type: "PairFaced"; a: string; b: string; first: string }
  | { type: "Strike"; striker: string; defender: string }
  // hpAfter = the unit's current hp after the event applied; stamped at apply time
  // (optional only because proposals are drafted without it — every logged event carries it).
  | { type: "Hurt"; unit: string; amount: number; hpAfter?: number; absorbed?: number }
  | { type: "Heal"; unit: string; amount: number; hpAfter?: number }
  | { type: "Death"; unit: string }
  | { type: "Summon"; unit: string; name: string; side: Side; hp: number; pwr: number; resurrected?: boolean; atHp?: number }
  | { type: "StatusApplied"; unit: string; status: string; stacks: number; total: number }
  | { type: "StatusRemoved"; unit: string; status: string; stacks: number; remaining: number }
  // hpAfter is present on hp StatChanged events only (a pwr change moves no hp).
  | { type: "StatChanged"; unit: string; stat: StatName; delta: number; now: number; hpAfter?: number }
  | { type: "Silenced"; unit: string }
  | { type: "Fatigue"; amount: number }
  | { type: "ChainBlocked"; ability: AbilityRef; at: number }
  | { type: "Intercepted"; by: AbilityRef; original: string; unit?: string }
  | { type: "BattleEnd"; winner: Side | "draw"; turns: number };

export type EventType = EventBody["type"];

export type BattleEvent = {
  id: number; // ordinal; equals index in the log
  turn: number;
  causedBy: number | null; // parent event id; null only for kernel beats
  source: SourceRef;
} & EventBody;

// ---------- Battle input ----------

export interface BattleInput {
  teamA: UnitDef[];
  teamB: UnitDef[];
  seed: number;
  statuses?: StatusRegistry;
  /** The ability registry a unit's `ability` ref resolves through (PRD #081),
   * the analogue of `statuses`. Optional during the migration: a unit still on
   * the legacy inline `abilities[]` path needs no registry. */
  abilities?: AbilityRegistry;
}
