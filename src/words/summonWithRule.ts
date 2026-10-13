import { EXAMPLE_ABILITIES, example, plain, rule } from "./examples.js";
import type { EffectWord } from "./types.js";
import type { EffectV2 } from "../types.js";

type E = Extract<EffectV2, { kind: "summonWithRule" }>;

/** Summon a body that carries the target's rule, where the target fell (else at the front) (§3.6). */
export const summonWithRule: EffectWord<E> = {
  kind: "summonWithRule",
  class: "timing",
  context: "trigger",
  en: (e, _say, target) => `summon ${e.unit.name} (${e.unit.base.pwr}/${e.unit.base.hp}) with the rule of ${target === "it" ? "that unit" : target}`,
  ru: (e, _say, target) => `призвать ${e.unit.name} (${e.unit.base.pwr}/${e.unit.base.hp}) с правилом ${target}`,
  example: {
    input: { ...example([rule("Archer", 1, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap"), rule("Grave Tree", 8, 1, { on: "Death", unit: "otherAlly" }, { kind: "eventUnit" }, "Grow")], [plain("Brute", 14, 2)]), abilities: { ...EXAMPLE_ABILITIES, Grow: { name: "Grow", family: "Summon", effects: [{ kind: "summonWithRule", unit: { name: "Sapling", base: { hp: 4, pwr: 0 }, triggers: [{ kind: "trigger", on: { on: "BattleEnd" as never } }], selectors: [{ kind: "holder" }], abilities: ["Mend"] } }] } } },
    trace: "Grave Tree grows Sapling with the rule of Archer",
  },
};
