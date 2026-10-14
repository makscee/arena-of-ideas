import { EXAMPLE_ABILITIES, example, plain, rule } from "./examples.js";
import { ruRuleOf, type EffectWord } from "./types.js";
import type { EffectV2 } from "../types.js";

type E = Extract<EffectV2, { kind: "keepRule" }>;

/** The holder keeps the target's rule as a second rule of its own, with its
 * own When, for the rest of the battle (§3.5; #1063, Raven awoken). */
export const keepRule: EffectWord<E> = {
  kind: "keepRule",
  class: "ruleMod",
  context: "trigger",
  en: (_e, _say, target) => `keep the rule of ${target === "it" ? "that unit" : target} as a second rule of its own`,
  ru: (_e, _say, target) => `оставить себе ${ruRuleOf(target)} как второе правило`,
  example: {
    input: { ...example([rule("Archer", 1, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap"), rule("Raven", 6, 1, { on: "Death", unit: "otherAlly" }, { kind: "eventUnit" }, "Keep")], [plain("Brute", 12, 2)]), abilities: { ...EXAMPLE_ABILITIES, Keep: { name: "Keep", family: "Arcane", effects: [{ kind: "keepRule" }] } } },
    trace: "Raven keeps the rule of Archer",
  },
};
