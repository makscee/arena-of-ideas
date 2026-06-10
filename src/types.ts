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
  | { type: "Hurt"; unit: string; amount: number; absorbed?: number }
  | { type: "Heal"; unit: string; amount: number }
  | { type: "Death"; unit: string }
  | { type: "Summon"; unit: string; name: string; side: Side; hp: number; pwr: number; resurrected?: boolean }
  | { type: "StatusApplied"; unit: string; status: string; stacks: number; total: number }
  | { type: "StatusRemoved"; unit: string; status: string; stacks: number; remaining: number }
  | { type: "StatChanged"; unit: string; stat: StatName; delta: number; now: number }
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
}
