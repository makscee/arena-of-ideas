import { EXAMPLE_ABILITIES, example, plain } from "./examples.js";
import type { EffectWord } from "./types.js";
import type { EffectV2, UnitDef } from "../types.js";

type E = Extract<EffectV2, { kind: "redirect" }>;

const Magnet: UnitDef = {
  name: "Magnet",
  base: { hp: 9, pwr: 1 },
  triggers: [{ kind: "interceptor", on: { on: "Hurt", unit: "otherAlly" } }],
  selectors: [{ kind: "holder" }],
  abilities: ["Pull"],
};

/** The proposed Hurt, Heal, Strike or status lands on another unit instead;
 * an event is redirected at most once (§3.4, guardrails). */
export const redirect: EffectWord<E> = {
  kind: "redirect",
  class: "replacement",
  context: "interceptor",
  intercept(_ctx, e, at) {
    const to = at.select(e.to)[0];
    if (to) at.redirect(to);
    return "pass";
  },
  en: (e, say) => `it lands on ${say.who(e.to)} instead`,
  ru: (e, say) => `вместо этого достаётся ${say.who(e.to)}`,
  example: {
    input: { ...example([plain("Archer", 3, 2), Magnet], [plain("Brute", 12, 2)]), abilities: { ...EXAMPLE_ABILITIES, Pull: { name: "Pull", family: "Control", effects: [{ kind: "redirect", to: { kind: "holder" } }] } } },
    trace: "Magnet intercepts: the hit on Archer lands on Magnet",
  },
};
