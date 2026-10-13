import { EXAMPLE_ABILITIES, example, plain, rule } from "./examples.js";
import type { EffectWord } from "./types.js";
import type { EffectV2 } from "../types.js";

type E = Extract<EffectV2, { kind: "transform" }>;

/** The target becomes another unit in place: HP carries over proportionally (Seed at 2/4 HP becomes Tree at 4/8), its rule is the new one's; once per battle (§3.6). */
export const transform: EffectWord<E> = {
  kind: "transform",
  class: "timing",
  context: "trigger",
  en: (e, _say, target) => `${target === "self" ? "become" : `${target} becomes`} ${e.into.name} (${e.into.base.pwr}/${e.into.base.hp})`,
  ru: (e, _say, target) => `${target === "себя" ? "превратиться" : `превратить ${target}`} в ${e.into.name} (${e.into.base.pwr}/${e.into.base.hp})`,
  example: {
    input: { ...example([rule("Seed", 4, 0, { on: "BattleStart" }, { kind: "holder" }, "Sprout"), plain("Archer", 4, 1)], [plain("Brute", 14, 1)]), abilities: { ...EXAMPLE_ABILITIES, Sprout: { name: "Sprout", family: "Summon", effects: [{ kind: "delay", turns: 2, effect: { kind: "transform", into: { name: "Tree", base: { hp: 8, pwr: 0 }, triggers: [{ kind: "trigger", on: { on: "TurnStart" } }], selectors: [{ kind: "allyBehind" }], abilities: ["Empower"] } } }] } } },
    trace: "Seed becomes Tree (0/4)",
  },
};
