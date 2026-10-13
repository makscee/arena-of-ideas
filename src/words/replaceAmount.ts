import { EXAMPLE_ABILITIES, example, plain } from "./examples.js";
import type { EffectWord } from "./types.js";
import type { EffectV2, UnitDef } from "../types.js";

type E = Extract<EffectV2, { kind: "replaceAmount" }>;

const Gambler: UnitDef = {
  name: "Gambler",
  base: { hp: 9, pwr: 2 },
  triggers: [{ kind: "interceptor", on: { on: "Strike", striker: "holder" } }],
  condition: { kind: "flip", p: 0.5 },
  selectors: [{ kind: "holder" }],
  abilities: ["AllIn"],
};

/** The proposed Hurt or Heal, or a Strike's hit, becomes this amount (§3.4). */
export const replaceAmount: EffectWord<E> = {
  kind: "replaceAmount",
  class: "replacement",
  context: "interceptor",
  intercept(_ctx, e, at) {
    const d = at.draft;
    if (d.type === "Hurt" || d.type === "Heal" || d.type === "Strike") d.amount = at.eval(e.amount);
    return "pass";
  },
  en: (e, say) => `it becomes ${say.amount(e.amount)}`,
  ru: (e, say) => `он становится равным ${say.amount(e.amount)}`,
  example: {
    input: { ...example([Gambler], [plain("Brute", 30, 1)], 3), abilities: { ...EXAMPLE_ABILITIES, AllIn: { name: "AllIn", family: "Strike", effects: [{ kind: "replaceAmount", amount: { kind: "twice", of: { kind: "stat", stat: "pwr", of: "holder" } } }] } } },
    trace: "Gambler's strike on Brute hits for 4",
  },
};
