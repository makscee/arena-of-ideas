import { EXAMPLE_ABILITIES, example, plain } from "./examples.js";
import type { EffectWord } from "./types.js";
import type { EffectV2, UnitDef } from "../types.js";

type E = Extract<EffectV2, { kind: "preventDeath" }>;

const Candle: UnitDef = {
  name: "Candle",
  base: { hp: 3, pwr: 2 },
  triggers: [{ kind: "interceptor", on: { on: "Death", unit: "holder" } }],
  selectors: [{ kind: "holder" }],
  abilities: ["Gutter"],
};

/** Cancel a Death: the unit stays at `at` HP and pays `cost` from a stat.
 * When the stat can't pay, the unit dies (Candle: at 0 power) (§3.4). */
export const preventDeath: EffectWord<E> = {
  kind: "preventDeath",
  class: "replacement",
  context: "interceptor",
  intercept(ctx, e, at) {
    if (at.draft.type !== "Death") return "pass";
    const u = ctx.unit(at.draft.unit);
    if (!u || ctx.stat(u, e.cost.stat) < e.cost.amount) return "pass";
    at.survive(u.id, e.at, e.cost.stat, e.cost.amount);
    return "cancel";
  },
  en: (e) => `stays at ${e.at} HP and loses ${e.cost.amount} ${e.cost.stat.toUpperCase()} instead`,
  ru: (e) => `вместо смерти остаётся с ${e.at} ОЗ и теряет ${e.cost.amount} ${e.cost.stat === "pwr" ? "АТК" : "ОЗ"}`,
  example: {
    input: { ...example([Candle], [plain("Brute", 30, 3)]), abilities: { ...EXAMPLE_ABILITIES, Gutter: { name: "Gutter", family: "Shield", effects: [{ kind: "preventDeath", at: 1, cost: { stat: "pwr", amount: 1 } }] } } },
    trace: "Candle intercepts: Candle survives at 1 HP, loses 1 PWR",
  },
};
