// Part atoms — the creator-facing vocabulary (SPEC §1 "Part"): a single
// trigger, interceptor, condition, selector, or effect. Creation assembles
// abilities from these atoms; the codex must show every one as its own card.
//
// COVERAGE IS DERIVED FROM THE TYPE SPACE, not a hand-maintained list. Each
// atom family below is keyed by its discriminated-union `kind` (Effect["kind"],
// Selector["kind"], Condition["kind"]) or by the EventPattern `on` tag, through
// a `Record<…Tag, …>` whose key set is the FULL union. Add a new Effect/Selector/
// Condition/EventPattern kind to types.ts and these objects stop type-checking
// until a sample atom is supplied — so a new Part kind cannot ship without a
// card. The runtime list is `Object.keys()` over those same records, so the
// derivation and the compile-time guard read off one source.
//
// Each atom carries a representative value; the codex turns it into a card whose
// one-line meaning is produced by the describe.ts helpers (kernel untouched —
// presentation only). The sample's numbers/targets are illustrative; the card
// shows the atom's SHAPE, not a specific tuned instance.

import {
  describeCondition,
  describeEffect,
  describeSelector,
  describeWhen,
} from "./describe.js";
import type {
  Condition,
  Effect,
  EventPattern,
  Selector,
  UnitDef,
  When,
} from "./types.js";

/** A creator atom, ready for a uniform card. `family` groups the codex section;
 * `kind` is the atom's discriminant (the deep-link key within its family);
 * `name` is its display label; `meaning` is the describe-derived one-liner. */
export interface PartAtom {
  family: "trigger" | "interceptor" | "condition" | "selector" | "effect";
  kind: string;
  name: string;
  meaning: string;
}

// A throwaway unit for the `summon` sample — the card shows the effect's shape,
// not this unit. Kept tiny so the meaning sentence stays about the atom.
const SAMPLE_SUMMON_UNIT: UnitDef = { name: "a token", base: { hp: 1, pwr: 1 } };

// ---------------------------------------------------------------------------
// Sample atoms, one per union kind. The Record key sets ARE the union: TS
// rejects a missing or misspelled kind, so the enumeration tracks types.ts.
// ---------------------------------------------------------------------------

/** Every Effect kind → a representative atom. Key set = Effect["kind"]. */
const EFFECT_SAMPLES: Record<Effect["kind"], Effect> = {
  damage: { kind: "damage", amount: { kind: "const", value: 3 } },
  heal: { kind: "heal", amount: { kind: "const", value: 3 } },
  applyStatus: { kind: "applyStatus", status: "a status", stacks: { kind: "const", value: 1 } },
  consumeStacks: { kind: "consumeStacks", stacks: { kind: "const", value: 1 } },
  summon: { kind: "summon", unit: SAMPLE_SUMMON_UNIT },
  silence: { kind: "silence" },
  resurrect: { kind: "resurrect", hp: { kind: "const", value: 1 } },
  cancel: { kind: "cancel" },
  absorbHurt: { kind: "absorbHurt" },
  preventDeathHeal: { kind: "preventDeathHeal", toHp: { kind: "const", value: 1 } },
};

/** Every Selector kind → a representative atom. Key set = Selector["kind"]. */
const SELECTOR_SAMPLES: Record<Selector["kind"], Selector> = {
  holder: { kind: "holder" },
  eventUnit: { kind: "eventUnit" },
  frontEnemy: { kind: "frontEnemy" },
  allEnemies: { kind: "allEnemies" },
  allAllies: { kind: "allAllies" },
  randomEnemy: { kind: "randomEnemy" },
  lastDeadAlly: { kind: "lastDeadAlly" },
};

/** Every Condition kind → a representative atom. Key set = Condition["kind"]. */
const CONDITION_SAMPLES: Record<Condition["kind"], Condition> = {
  holderHpAtMost: { kind: "holderHpAtMost", value: 3 },
};

/** Every EventPattern `on` tag → a representative pattern. Key set =
 * EventPattern["on"] (the union of the `on` discriminants). A Trigger fires
 * after the event; an Interceptor fires instead of a proposed one — so each tag
 * appears once per `When.kind` it is valid in (below). */
const EVENT_PATTERN_SAMPLES: Record<EventPattern["on"], EventPattern> = {
  BattleStart: { on: "BattleStart" },
  TurnStart: { on: "TurnStart" },
  TurnEnd: { on: "TurnEnd" },
  Strike: { on: "Strike" },
  Hurt: { on: "Hurt" },
  Heal: { on: "Heal" },
  Death: { on: "Death" },
  Summon: { on: "Summon" },
  StatusApplied: { on: "StatusApplied" },
  StatusRemoved: { on: "StatusRemoved" },
};

// Which event patterns make sense as INTERCEPTORS: a kernel-emitted lifecycle
// beat (BattleStart/TurnStart/TurnEnd) is not a *proposed* event, so it cannot
// be intercepted — only transformable/cancellable proposals can. This split is
// itself derived from the type space: every `on` tag is a trigger; the
// proposable subset (those that go through the propose→settle path) are also
// interceptors. Listed as a typed subset so a new proposable event must be
// classified here consciously.
const INTERCEPTABLE: ReadonlySet<EventPattern["on"]> = new Set<EventPattern["on"]>([
  "Strike",
  "Hurt",
  "Heal",
  "Death",
  "Summon",
  "StatusApplied",
  "StatusRemoved",
]);

// ---------------------------------------------------------------------------
// Enumeration — Object.keys over the sample records, so the list is the union.
// ---------------------------------------------------------------------------

/** A human title for an atom kind: the kind tag de-camelCased ("applyStatus" →
 * "Apply status", "allEnemies" → "All enemies"). Presentation only. */
function titleOf(kind: string): string {
  const spaced = kind.replace(/([a-z])([A-Z])/g, "$1 $2");
  return spaced.charAt(0).toUpperCase() + spaced.slice(1);
}

/** Every Part atom the type space defines, each ready for a uniform card. The
 * order is families in creation order (triggers, interceptors, conditions,
 * selectors, effects); within a family, the union's declaration order. */
export function partAtoms(): PartAtom[] {
  const atoms: PartAtom[] = [];

  // Triggers + interceptors: one card per (When.kind, EventPattern) the type
  // space allows. The meaning is describeWhen of a sample When carrying that
  // pattern — kernel-derived, the same sentence an ability's clause would read.
  for (const on of Object.keys(EVENT_PATTERN_SAMPLES) as EventPattern["on"][]) {
    const pattern = EVENT_PATTERN_SAMPLES[on];
    const trigger: When = { kind: "trigger", on: pattern };
    atoms.push({
      family: "trigger",
      kind: on,
      name: titleOf(on),
      meaning: describeWhen(trigger),
    });
    if (INTERCEPTABLE.has(on)) {
      const interceptor: When = { kind: "interceptor", on: pattern };
      atoms.push({
        family: "interceptor",
        kind: on,
        name: titleOf(on),
        meaning: describeWhen(interceptor),
      });
    }
  }

  for (const kind of Object.keys(CONDITION_SAMPLES) as Condition["kind"][]) {
    atoms.push({
      family: "condition",
      kind,
      name: titleOf(kind),
      meaning: describeCondition(CONDITION_SAMPLES[kind]),
    });
  }

  for (const kind of Object.keys(SELECTOR_SAMPLES) as Selector["kind"][]) {
    atoms.push({
      family: "selector",
      kind,
      name: titleOf(kind),
      // A selector is a noun phrase ("the front enemy"); shown as "Targets …".
      meaning: `Targets ${describeSelector(SELECTOR_SAMPLES[kind])}.`,
    });
  }

  for (const kind of Object.keys(EFFECT_SAMPLES) as Effect["kind"][]) {
    atoms.push({
      family: "effect",
      kind,
      // describeEffect is a verb phrase against a target; "the target" keeps the
      // sentence about the atom, not a specific selector.
      name: titleOf(kind),
      meaning: `${capitalize(describeEffect(EFFECT_SAMPLES[kind], "the target"))}.`,
    });
  }

  return atoms;
}

const capitalize = (s: string): string => (s.length > 0 ? s[0]!.toUpperCase() + s.slice(1) : s);
