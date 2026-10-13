import { EXAMPLE_ABILITIES, example, plain, rule } from "./examples.js";
import { ruRuleOf, type EffectWord } from "./types.js";
import type { EffectV2 } from "../types.js";

type E = Extract<EffectV2, { kind: "remember" }>;

/** The holder remembers the target's rule, to perform later (§3.5). */
export const remember: EffectWord<E> = {
  kind: "remember",
  class: "ruleMod",
  context: "trigger",
  en: (_e, _say, target) => `remember the rule of ${target === "it" ? "that unit" : target}`,
  ru: (_e, _say, target) => `запомнить ${ruRuleOf(target)}`,
  example: {
    input: { ...example([rule("Archer", 1, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap"), { ...rule("Raven", 6, 1, { on: "Death", unit: "otherAlly" }, { kind: "eventUnit" }, "Recall"), triggers: [{ kind: "trigger", on: { on: "Death", unit: "otherAlly" } }, { kind: "trigger", on: { on: "TurnStart" } }], also: [{ selectors: [{ kind: "holder" }], abilities: ["Perform"] }] }], [plain("Brute", 12, 2)]), abilities: { ...EXAMPLE_ABILITIES, Recall: { name: "Recall", family: "Arcane", effects: [{ kind: "remember" }] }, Perform: { name: "Perform", family: "Arcane", effects: [{ kind: "performRemembered", mode: "once" }] } } },
    trace: "Raven remembers the rule of Archer",
  },
};
