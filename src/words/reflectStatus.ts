import { EXAMPLE_ABILITIES, example, plain, rule } from "./examples.js";
import type { EffectWord } from "./types.js";
import type { EffectV2, UnitDef } from "../types.js";

type E = Extract<EffectV2, { kind: "reflectStatus" }>;

const Prism: UnitDef = {
  name: "Prism",
  base: { hp: 8, pwr: 1 },
  triggers: [{ kind: "interceptor", on: { on: "StatusApplied", unit: "holder", status: "Strength" } }],
  selectors: [{ kind: "applier" }],
  abilities: ["Reflect"],
};

/** The proposed status lands as proposed, and `to` gets the same status in
 * the same amount too (#1063, Prism asleep; awoken Prism is a redirect). */
export const reflectStatus: EffectWord<E> = {
  kind: "reflectStatus",
  class: "replacement",
  context: "interceptor",
  intercept(_ctx, e, at) {
    const d = at.draft;
    if (d.type !== "StatusApplied") return "pass";
    const to = at.select(e.to)[0];
    if (to && to.id !== d.unit) at.alsoApply(to.id, d.status, d.stacks);
    return "pass";
  },
  en: (e, say) => `${say.who(e.to)} gets the same amount too`,
  ru: (e, say) => `${say.who(e.to)} тоже получает столько же`,
  example: {
    input: {
      ...example([Prism], [rule("Witch", 6, 1, { on: "BattleStart" }, { kind: "frontEnemy" }, "Empower"), plain("Brute", 6, 2)]),
      abilities: { ...EXAMPLE_ABILITIES, Reflect: { name: "Reflect", family: "Control", effects: [{ kind: "reflectStatus", to: { kind: "applier" } }] } },
    },
    trace: "Prism → the giver (Witch) → Strength 1",
  },
};
