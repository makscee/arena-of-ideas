// Building blocks for the words' example battles: a tiny registry and a unit
// builder, so each word file states its example in a few lines.

import type { AbilityRegistry, BattleInput, Condition, EventPattern, Selector, StatusRegistry, UnitDef } from "../types.js";

export const EXAMPLE_STATUSES: StatusRegistry = {
  Strength: { name: "Strength", statMods: { pwr: 1 }, abilities: [] },
};

export const EXAMPLE_ABILITIES: AbilityRegistry = {
  Empower: { name: "Empower", family: "Control", effects: [{ kind: "applyStatus", status: "Strength", stacks: { kind: "const", value: 1 } }] },
  Zap: { name: "Zap", family: "Strike", effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }] },
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
