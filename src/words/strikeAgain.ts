import { EXAMPLE_ABILITIES, example, plain } from "./examples.js";
import type { EffectWord } from "./types.js";
import type { EffectV2, UnitDef } from "../types.js";

type E = Extract<EffectV2, { kind: "strikeAgain" }>;

const Twin: UnitDef = {
  name: "Twin",
  base: { hp: 9, pwr: 2 },
  triggers: [{ kind: "interceptor", on: { on: "Strike", striker: "holder" } }],
  selectors: [{ kind: "holder" }],
  abilities: ["Again"],
};

/** The proposed Strike's striker strikes the same defender once more, right
 * after; a strike made this way never strikes again (Gambler awoken, §4).
 * In trigger context (#1063, Executioner) the holder strikes the rule's
 * target once more, right now. */
export const strikeAgain: EffectWord<E> = {
  kind: "strikeAgain",
  class: "ruleMod",
  context: "either",
  en: () => "it strikes again",
  ru: () => "он бьёт ещё раз",
  example: {
    input: { ...example([Twin], [plain("Brute", 30, 1)]), abilities: { ...EXAMPLE_ABILITIES, Again: { name: "Again", family: "Strike", effects: [{ kind: "strikeAgain" }] } } },
    trace: "Twin strikes again",
  },
};
