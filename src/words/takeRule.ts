import { EXAMPLE_ABILITIES, example, rule } from "./examples.js";
import { ruRuleOf, type EffectWord } from "./types.js";
import type { EffectV2 } from "../types.js";

type E = Extract<EffectV2, { kind: "takeRule" }>;

/** The holder's rule becomes the target's and the target is left with none,
 * for the rest of the battle (§3.5; #1063, Trickster awoken). */
export const takeRule: EffectWord<E> = {
  kind: "takeRule",
  class: "ruleMod",
  context: "trigger",
  en: (_e, _say, target) => `take the rule of ${target} and leave it none`,
  ru: (_e, _say, target) => `забрать ${ruRuleOf(target)}, оставив его без правила`,
  example: {
    input: { ...example([rule("Thief", 6, 1, { on: "BattleStart" }, { kind: "frontEnemy" }, "Take")], [rule("Hedgehog", 8, 1, { on: "Hurt", unit: "holder" }, { kind: "frontEnemy" }, "Zap")]), abilities: { ...EXAMPLE_ABILITIES, Take: { name: "Take", family: "Arcane", effects: [{ kind: "takeRule" }] } } },
    trace: "Thief takes the rule of Hedgehog",
  },
};
