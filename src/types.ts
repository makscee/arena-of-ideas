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
  /** Player-facing authorship snapshot for governed content. The battle kernel
   * ignores it; it travels by value so cards can credit the entity itself. */
  _creator?: string;
  /** A unit a word makes (a transform's body): its Russian name, as a label
   * and in the accusative ("превратиться в Бабочку"). Display only. */
  names?: { ru: { name: string; acc: string } };
  base: Stats;
  level?: number;
  /** Canonical v2 behavior recipe: Trigger is when. */
  triggers?: When[];
  condition?: Condition;
  /** Canonical v2 behavior recipe: Selector is who/what receives the action. */
  selectors?: Selector[];
  /** Canonical v2 behavior recipe: ordered refs to what happens. Base Units
   * carry exactly one; the array reserves the accepted two-Ability fused shape. */
  abilities?: string[];
  /** "And" clauses (MVP round 4): more abilities on the same triggers and
   * condition, each clause with selectors of its own, firing after
   * `abilities`, in order. Canonical recipes only. */
  also?: { selectors: Selector[]; abilities: string[] }[];
  /** v1 compatibility only. Parsers migrate this field deterministically and
   * reject it when mixed with any canonical recipe field. */
  ability?: string;
  /** Initial statuses, applied (as events) right after BattleStart. Names resolve via the status registry. */
  statuses?: { status: string; stacks: number }[];
}

/** Ability is only the action — what happens. Trigger and Selector context
 * lives on the Unit/Status recipe, never in this entity. */
export interface Ability {
  effects: Effect[]; // ≥1; run in sequence order
  /** v1 compatibility only. Canonical AbilityDef validation rejects these. */
  whens?: When[];
  condition?: Condition;
  selectors?: Selector[];
}

/** Runtime reaction assembled from a recipe's context plus one Ability action. */
export interface Reaction extends Ability {
  triggers: When[];
  condition?: Condition;
  selectors: Selector[];
}

/** Every Ability declares one family; a Unit's visual identity derives from its
 * ordered Ability refs. The family→hex palette is centralized in tunables.ts. */
export type Family = "Poison" | "Strike" | "Shield" | "Summon" | "Arcane" | "Control" | "Heal";

/** A named, referenceable action. Context is deliberately absent. */
export interface AbilityDef extends Ability {
  name: string;
  family: Family;
  /** v1 read compatibility only; canonical validators reject context on Ability. */
  whens?: When[];
  condition?: Condition;
  selectors?: Selector[];
}

export type AbilityRegistry = Record<string, AbilityDef>;

/** Canonical ordered Ability ids, with the v1 singleton read isolated here. */
export function abilityIdsOf(unit: UnitDef): string[] {
  return unit.abilities ?? (unit.ability !== undefined ? [unit.ability] : []);
}

export function primaryAbilityIdOf(unit: UnitDef): string | undefined {
  return abilityIdsOf(unit)[0];
}

/** Bind a canonical Unit's when/who recipe to its referenced action(s) for
 * execution/description. Legacy context is read only on the compatibility path. */
export function unitActionsOf(unit: UnitDef, registry: AbilityRegistry): Ability[] {
  const clauses = [
    { ids: abilityIdsOf(unit), selectors: unit.selectors },
    ...(unit.also ?? []).map((c) => ({ ids: c.abilities, selectors: c.selectors as Selector[] | undefined })),
  ];
  return clauses.flatMap(({ ids, selectors }) => ids.flatMap((id) => {
    const action = registry[id];
    if (!action) return [];
    return [{
      ...action,
      whens: unit.triggers ?? action.whens ?? [],
      selectors: selectors ?? action.selectors ?? [],
      ...(unit.condition ?? action.condition ? { condition: unit.condition ?? action.condition } : {}),
    }];
  }));
}

/** Bind canonical Status context to each action for description tooling. */
export function statusActionsOf(status: StatusDef): Ability[] {
  return status.abilities.map((action) => ({
    ...action,
    whens: status.triggers ?? action.whens ?? [],
    selectors: status.selectors ?? action.selectors ?? [],
    ...(status.condition ?? action.condition ? { condition: status.condition ?? action.condition } : {}),
  }));
}

export interface When {
  kind: "trigger" | "interceptor";
  on: EventPattern;
}

/** Unit filters are relative to the ability's holder. "ally" includes the holder's side (and the holder);
 * "otherAlly" is the holder's side minus the holder ("after an ally dies" never fires on its own death). */
export type UnitFilter = "holder" | "ally" | "otherAlly" | "enemy" | "any";

export type EventPattern = EventPatternV1 | ActedPattern;

/** Rules v2 (#1043 §5, Déjà Vu): "whenever <unit> acts". It matches a unit's
 * own rule firing (not its statuses'), with no event of its own in the log:
 * the reaction is caused by the event the actor reacted to. `front`: only
 * while the actor is at the front of its line. Trigger-only. */
export type ActedPattern = { on: "Acted"; unit?: UnitFilter; front?: true };

export type EventPatternV1 =
  | { on: "BattleStart" | "TurnStart" | "TurnEnd" }
  | { on: "Strike"; striker?: UnitFilter }
  // aim "random": only a Hurt or status an enemy's rule aimed by a random
  // pick (its randomEnemy Who) — Magnet, rules v2 §4
  | { on: "Hurt"; unit?: UnitFilter; aim?: "random" }
  | { on: "Heal" | "Death" | "Summon"; unit?: UnitFilter }
  | { on: "StatusApplied"; unit?: UnitFilter; status?: string; aim?: "random" }
  | { on: "StatusRemoved"; unit?: UnitFilter; status?: string }
  // A stat moved (a statMod status landed or left). Trigger-only: the change
  // follows from its status event, so there is nothing to intercept.
  | { on: "StatChanged"; unit?: UnitFilter; stat?: StatName; sign?: "gain" | "loss" };

export type ConditionV1 = { kind: "holderHpAtMost"; value: number };

/** Rules v2 conditions (#1043 §3.2), all composable with and / not. Each kind
 * is a word in src/words/. */
export type CompareOp = "<=" | "=" | ">=";
export type ConditionV2 =
  | { kind: "and"; all: Condition[] }
  | { kind: "not"; cond: Condition }
  | { kind: "turnIs"; n: number }
  | { kind: "turnAtLeast"; n: number }
  | { kind: "turnEvery"; n: number }
  // of: the holder, or the unit the triggering event is about
  | { kind: "stacksAtLeast"; status: string; n: number; of: "holder" | "eventUnit" }
  | { kind: "alliesAlive"; op: CompareOp; n: number }
  | { kind: "enemiesAlive"; op: CompareOp; n: number }
  // one seeded coin per firing, heads with probability p; a Flipped event logs it
  | { kind: "flip"; p: number }
  | { kind: "onceHolder" }
  | { kind: "oncePerTurn" }
  | { kind: "unhurt" }
  | { kind: "outnumbered" }
  | { kind: "targetHas"; statuses: string[] }
  | { kind: "killedByHolder" };

export type Condition = ConditionV1 | ConditionV2;

/** Rules v2 position selectors (#1043 §3.1); each kind is a word in src/words/. */
export type PositionSelector =
  | { kind: "allyBehind" } // the living ally one slot behind the holder
  | { kind: "allyAhead" } // … one slot ahead
  | { kind: "neighbours" } // both of the above, ahead first
  | { kind: "alliesBehind" } // every living ally behind the holder, in line order
  | { kind: "lastEnemy" } // the enemy at the back
  | { kind: "randomAlly" } // one seeded draw among the other living allies
  | { kind: "eventTarget" } // the unit an event was aimed at (a Strike's defender)
  | { kind: "applier" } // the unit whose rule gave the triggering status (#1063, Prism)
  | { kind: "poisonedEnemies" };

export type Selector = SelectorV1 | PositionSelector;

export type SelectorV1 =
  | { kind: "holder" } // the owning unit; resolves even if it just died (on-death abilities)
  | { kind: "eventUnit" } // the subject unit of the triggering event (striker for Strike)
  | { kind: "attacker" } // who dealt the triggering Hurt, while alive; none for status damage
  | { kind: "frontEnemy" }
  | { kind: "allEnemies" }
  | { kind: "allAllies" } // living, includes holder
  | { kind: "randomEnemy" } // one seeded draw whenever ≥1 candidate
  | { kind: "lastDeadAlly" }; // most recent graveyard entry still dead

/** Magnitude expressions. Stat scaling is opt-in content, priced by the budget (not v1). */
export type Amount = AmountV1 | AmountV2;

/** Rules v2 scaling amounts (#1043 §3.3); each kind is a word in src/words/. */
export type BattleFact = "hitsTaken" | "hitsTakenByAllies" | "deadAllies" | "alliesAlive" | "enemiesAlive" | "turn";
export type AmountV2 =
  // a named counter or status on the holder or the effect's target
  | { kind: "stacksOf"; status: string; of: "holder" | "target" }
  // a battle fact the kernel keeps
  | { kind: "count"; fact: BattleFact }
  // someone else's effective stat
  | { kind: "statOf"; stat: StatName; of: "target" | "eventUnit" }
  // the holder's missing hp
  | { kind: "missingHp" }
  | { kind: "half"; of: Amount } // rounded down
  | { kind: "twice"; of: Amount }
  | { kind: "plus"; a: Amount; b: Amount };

export type AmountV1 =
  | { kind: "const"; value: number }
  | { kind: "stat"; stat: StatName; of: "holder" } // holder's effective stat
  | { kind: "level"; of: "holder" } // holder's level — shop-layer growth, readable like a stat
  | { kind: "stacks" }; // current stacks of the owning status instance (status content only)

export type Effect = EffectV1 | EffectV2 | MacroEffect;

/** A composed word (#1043 §7, src/words/macros.ts): the compiler expands it to its primitives before the kernel sees it. */
export type MacroEffect = { kind: "macro"; name: string };

/** Rules v2 effects (#1043 §3.4–3.6); each kind is a word in src/words/. */
export type EffectV2 =
  // interceptor: the proposed Hurt, Heal, Strike or status lands on another unit; once per event
  | { kind: "redirect"; to: Selector }
  // interceptor: cancel a Death, stay at `at` hp and pay `cost` from a stat (Candle)
  | { kind: "preventDeath"; at: number; cost: { stat: StatName; amount: number } }
  // interceptor: the proposed Hurt or Heal (or a Strike's hit) becomes this amount
  | { kind: "replaceAmount"; amount: Amount }
  // rule modification (§3.5): the holder's rule becomes the target's for the battle
  // withPower (Mirror awoken): the holder also gains Strength equal to the target's power
  | { kind: "copyRule"; withPower?: true }
  // the target's rule fires now, as if its trigger had fired; once per event per target
  | { kind: "actAgain" }
  // the holder remembers the target's rule
  | { kind: "remember" }
  // the holder performs its remembered rules: the oldest once (then forgets it), or all (kept)
  | { kind: "performRemembered"; mode: "once" | "all" }
  // the holder and the target trade rules
  | { kind: "swapRules" }
  // timing and forms (§3.6): the effect runs on the target at the end of the turn `turns` turns from now
  | { kind: "delay"; turns: number; effect: Effect }
  // the target becomes another unit in place; HP carries over proportionally; once per battle
  | { kind: "transform"; into: UnitDef }
  // summon `unit` carrying the target's rule, at the target's slot when it has fallen (else the front)
  | { kind: "summonWithRule"; unit: UnitDef }
  // either context: check `if` (a flip logs its coin) and run `then` or `else` (Gambler's tails)
  | { kind: "branch"; if: Condition; then: Effect[]; else: Effect[] }
  // interceptor: the proposed Strike's striker strikes the same defender again
  // after it; a strike made this way never strikes again (Gambler awoken)
  | { kind: "strikeAgain" }
  | { kind: "takeRule" }
  | { kind: "reflectStatus"; to: Selector }; // the proposed status also lands on `to`, same amount (#1063, Prism)

export type EffectV1 =
  // trigger-context atoms
  | { kind: "damage"; amount: Amount }
  | { kind: "heal"; amount: Amount }
  | { kind: "applyStatus"; status: string; stacks: Amount }
  | { kind: "consumeStacks"; status?: string; stacks: Amount } // status omitted = the owning status
  | { kind: "summon"; unit: UnitDef } // at the front of the target's team; a full line logs NoRoom instead
  | { kind: "silence" } // remove all statuses, disable the unit's own abilities for the battle
  | { kind: "resurrect"; hp: Amount } // revive the (dead) target at N hp, back of line (BattleInput.reviveInPlace: where it fell); a full line logs NoRoom
  // interceptor-context atoms — transform/cancel the proposed event
  | { kind: "cancel"; consumeSelf?: number } // cancel the event; optionally consume own stacks
  | { kind: "absorbHurt" } // reduce a proposed Hurt by min(own stacks, amount); consume = absorbed
  | { kind: "preventDeathHeal"; toHp: Amount; removeSelf?: boolean }; // cancel a Death, heal to N hp

export interface StatusDef {
  name: string;
  /** A counter (rules v2 §3.3): tracked like stacks, drawn as a small
   * number on the card rather than as a buff. Display only. */
  counter?: true;
  /** A coined status's names (rules v2: a rule the renderer can't say is not
   * legal): the night batch refuses a coined status without them. Shipped
   * statuses take their RU names from src/status-names.ts. */
  names?: { en: string; ru: { name: string; acc: string; count: readonly [one: string, few: string, many: string] } };
  /** Contribution per stack while attached. Effective stat = max(0, base + Σ contributions) — computed, never baked. */
  statMods?: Partial<Stats>;
  /** Status recipe context. Empty for modifier-only statuses. */
  triggers?: When[];
  condition?: Condition;
  selectors?: Selector[];
  abilities: Ability[];
}

export type StatusRegistry = Record<string, StatusDef>;

// ---------- Events: the causal log ----------

export interface AbilityRef {
  unit: string; // holding unit instance id
  status?: string; // present when the ability lives on a status
  ability: number; // index within the ability list
  /** Which of the reactor's Whens (index in its triggers) set this firing off
   * (or would have, on a ChainBlocked). Stamped by the kernel; absent on logs
   * from before round 2 (R2-15). */
  when?: number;
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
  // amount: set by a replaceAmount interceptor (rules v2); absent = the striker's pwr
  | { type: "Strike"; striker: string; defender: string; again?: true; amount?: number }
  // hpAfter = the unit's current hp after the event applied; stamped at apply time
  // (optional only because proposals are drafted without it — every logged event carries it).
  // pierced: sudden death's fatigue (R4-1), which no interceptor (Shield) can stop.
  | { type: "Hurt"; unit: string; amount: number; hpAfter?: number; absorbed?: number; pierced?: true }
  | { type: "Heal"; unit: string; amount: number; hpAfter?: number }
  // pierced: dealt by a pierced Hurt, so no interceptor (Blessing) can stop it.
  | { type: "Death"; unit: string; pierced?: true }
  | {
      type: "Summon";
      unit: string;
      name: string;
      side: Side;
      hp: number;
      pwr: number;
      resurrected?: boolean;
      atHp?: number;
      // the unit entered at the front of its line; absent = at the back (revives, and summons in logs before R3)
      front?: boolean;
      // a revive's index in its line: where it fell, clamped to the line's length (#918);
      // absent on revives in older logs, which returned at the back
      slot?: number;
    }
  | { type: "StatusApplied"; unit: string; status: string; stacks: number; total: number }
  | { type: "StatusRemoved"; unit: string; status: string; stacks: number; remaining: number }
  // hpAfter is present on hp StatChanged events only (a pwr change moves no hp).
  | { type: "StatChanged"; unit: string; stat: StatName; delta: number; now: number; hpAfter?: number }
  | { type: "Silenced"; unit: string }
  // suddenDeath: from BattleInput.suddenDeathAt on (R4-1): doubled, piercing.
  | { type: "Fatigue"; amount: number; suddenDeath?: true }
  | { type: "ChainBlocked"; ability: AbilityRef; at: number; spent?: true }
  // A cascade hit the step cap: `root` is the event its first firing reacted
  // to, `steps` the firings it ran; the firings still queued were dropped.
  | { type: "ChainCapped"; root: number; steps: number }
  // A Summon or Revive found its line full and did nothing (R4-2): `unit` is
  // the holder whose ability tried, `side` the full line, `name` who found no
  // room; `revive` is the dead unit's id when it was a revive.
  | { type: "NoRoom"; unit: string; side: Side; name: string; revive?: string }
  // A Summon (or, with revive, a Revive of `unit`) that did nothing, and why:
  // in sudden death (R4-1) nobody enters the line; with reviveInPlace a unit
  // returns once per battle, so its second revive fails ("revivedOnce", #918).
  | { type: "SummonFailed"; name: string; side: Side; unit?: string; revive?: true; reason: "suddenDeath" | "revivedOnce" }
  | { type: "Intercepted"; by: AbilityRef; original: string; unit?: string }
  // A flip condition (rules v2, §3.2) drew its coin for `unit`'s firing.
  | { type: "Flipped"; unit: string; by: AbilityRef; heads: boolean }
  // A redirect interceptor (rules v2, §3.4) moved a proposed event from one unit to another.
  | { type: "Redirected"; by: AbilityRef; original: string; from: string; to: string }
  // A rule-modifying effect (rules v2, §3.5) acted: `unit` is the holder, `other` the unit whose rule it took, ran or traded.
  | { type: "RuleMod"; unit: string; op: "copyRule" | "takeRule" | "actAgain" | "remember" | "performRemembered" | "swapRules"; other?: string }
  // A delay (rules v2, §3.6) was set on `unit`: its effect lands at the end of turn `at`.
  | { type: "Delayed"; unit: string; by: AbilityRef; at: number }
  // A transform (rules v2, §3.6): `unit` is now `name` with these stats.
  | { type: "Transformed"; unit: string; name: string; hp: number; pwr: number }
  // timeUp: the turn cap ran out with both sides standing (a draw); absent otherwise.
  | { type: "BattleEnd"; winner: Side | "draw"; turns: number; timeUp?: true };

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
  /** Registry resolving the Unit recipe's ordered `abilities` action refs. */
  abilities?: AbilityRegistry;
  /** Most trigger firings one cascade may run before it stops with a
   * ChainCapped event. Omitted = DEFAULT_CHAIN_STEP_CAP (battle.ts). */
  chainStepCap?: number;
  /** Turns a battle may last; past them, both sides standing is a draw and
   * BattleEnd says timeUp. Omitted = TURN_CAP (battle.ts). */
  turnCap?: number;
  /** Sudden death (R4-1): from this turn on, Fatigue doubles every turn and
   * pierces Shield and Blessing, and Summon and Revive do nothing (a
   * SummonFailed event says why). Omitted = no sudden death. */
  suddenDeathAt?: number;
  /** Most units a side's battle line holds (R4-10): Summon and Revive find no
   * room (a NoRoom event) past it. Teams still enter with at most TEAM_SIZE.
   * Omitted = TEAM_SIZE. */
  lineCap?: number;
  /** A revive returns the unit to its index in the line at death, clamped to
   * the line's length (#918), and its Summon event says `slot`. Omitted =
   * the back of the line, as fights before it had (the desktop ladder and
   * its goldens re-sim those). It also limits a unit to one revive a
   * battle: a second fails with SummonFailed "revivedOnce". */
  reviveInPlace?: boolean;
}
