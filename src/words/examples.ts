// Building blocks for the words' example battles: a tiny registry and a unit
// builder, so each word file states its example in a few lines.

import type { AbilityRegistry, BattleInput, Condition, EventPattern, Selector, StatusRegistry, UnitDef } from "../types.js";

export const EXAMPLE_STATUSES: StatusRegistry = {
  Strength: { name: "Strength", statMods: { pwr: 1 }, abilities: [] },
  // a counter: no stat, no rule, just a number on the card
  Ward: { name: "Ward", counter: true, abilities: [] },
};

export const EXAMPLE_ABILITIES: AbilityRegistry = {
  Empower: { name: "Empower", family: "Control", effects: [{ kind: "applyStatus", status: "Strength", stacks: { kind: "const", value: 1 } }] },
  Zap: { name: "Zap", family: "Strike", effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }] },
  Hit: { name: "Hit", family: "Strike", effects: [{ kind: "damage", amount: { kind: "stat", stat: "pwr", of: "holder" } }] },
  Mirror: { name: "Mirror", family: "Control", effects: [{ kind: "applyStatus", status: "Strength", stacks: { kind: "stacksOf", status: "Strength", of: "target" } }] },
  Brace: { name: "Brace", family: "Shield", effects: [{ kind: "applyStatus", status: "Ward", stacks: { kind: "count", fact: "hitsTaken" } }] },
  BraceAll: { name: "BraceAll", family: "Shield", effects: [{ kind: "applyStatus", status: "Ward", stacks: { kind: "plus", a: { kind: "count", fact: "hitsTaken" }, b: { kind: "count", fact: "hitsTakenByAllies" } } }] },
  Riposte: { name: "Riposte", family: "Strike", effects: [{ kind: "damage", amount: { kind: "statOf", stat: "pwr", of: "target" } }] },
  Rage: { name: "Rage", family: "Strike", effects: [{ kind: "damage", amount: { kind: "missingHp" } }] },
  HalfHit: { name: "HalfHit", family: "Strike", effects: [{ kind: "damage", amount: { kind: "half", of: { kind: "stat", stat: "pwr", of: "holder" } } }] },
  DoubleHit: { name: "DoubleHit", family: "Strike", effects: [{ kind: "damage", amount: { kind: "twice", of: { kind: "stat", stat: "pwr", of: "holder" } } }] },
  Mend: { name: "Mend", family: "Heal", effects: [{ kind: "heal", amount: { kind: "const", value: 1 } }] },
};

/** A unit with one rule: when `on`, `who` gets `does`. */
export function rule(name: string, hp: number, pwr: number, on: EventPattern, who: Selector, does: string, condition?: Condition): UnitDef {
  return {
    name,
    base: { hp, pwr },
    triggers: [{ kind: "trigger", on }],
    selectors: [who],
    abilities: [does],
    ...(condition ? { condition } : {}),
  };
}

/** A unit whose rule never matters (it mends itself as it dies). */
export const plain = (name: string, hp: number, pwr: number): UnitDef => rule(name, hp, pwr, { on: "Death", unit: "holder" }, { kind: "holder" }, "Mend");

export function example(teamA: UnitDef[], teamB: UnitDef[], seed = 1): BattleInput {
  return { teamA, teamB, seed, statuses: EXAMPLE_STATUSES, abilities: EXAMPLE_ABILITIES };
}
