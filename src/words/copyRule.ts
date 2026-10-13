import { EXAMPLE_ABILITIES, example, plain, rule } from "./examples.js";
import type { EffectWord } from "./types.js";
import type { EffectV2 } from "../types.js";

type E = Extract<EffectV2, { kind: "copyRule" }>;

/** The holder's rule becomes the target's for the rest of the battle (stats untouched) (§3.5). */
export const copyRule: EffectWord<E> = {
  kind: "copyRule",
  class: "ruleMod",
  context: "trigger",
  en: (e, _say, target) => `take the rule of ${target}${e.withPower ? " and its power" : ""}`,
  ru: (e, _say, target) => `взять правило ${target}${e.withPower ? " и его силу" : ""}`,
  example: {
    input: { ...example([rule("Mirror", 6, 1, { on: "BattleStart" }, { kind: "frontEnemy" }, "Copy")], [rule("Hedgehog", 8, 1, { on: "Hurt", unit: "holder" }, { kind: "frontEnemy" }, "Zap")]), abilities: { ...EXAMPLE_ABILITIES, Copy: { name: "Copy", family: "Arcane", effects: [{ kind: "copyRule" }] } } },
    trace: "Mirror copies the rule of Hedgehog",
  },
};
