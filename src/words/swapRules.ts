import { EXAMPLE_ABILITIES, example, plain, rule } from "./examples.js";
import { ruRuleOf, type EffectWord } from "./types.js";
import type { EffectV2 } from "../types.js";

type E = Extract<EffectV2, { kind: "swapRules" }>;

/** The holder and the target trade rules for the rest of the battle (§3.5). */
export const swapRules: EffectWord<E> = {
  kind: "swapRules",
  class: "ruleMod",
  context: "trigger",
  en: (_e, _say, target) => `trade rules with ${target}`,
  ru: (_e, _say, target) => `отдать своё правило и взять ${ruRuleOf(target)}`,
  example: {
    input: { ...example([rule("Trickster", 6, 1, { on: "BattleStart" }, { kind: "frontEnemy" }, "Swap")], [rule("Brute", 8, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap")]), abilities: { ...EXAMPLE_ABILITIES, Swap: { name: "Swap", family: "Arcane", effects: [{ kind: "swapRules" }] } } },
    trace: "Trickster trades rules with Brute",
  },
};
