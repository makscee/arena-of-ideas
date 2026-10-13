import { example, plain, rule } from "./examples.js";
import type { ConditionWord } from "./types.js";
import type { ConditionV2 } from "../types.js";

type C = Extract<ConditionV2, { kind: "turnAtLeast" }>;

/** Fires from turn n on (§3.2). */
export const turnAtLeast: ConditionWord<C> = {
  kind: "turnAtLeast",
  class: "condition",
  check(ctx, c, at) {
    return ctx.turn >= c.n;
  },
  en: (c, say) => `from turn ${c.n}`,
  ru: (c, say) => `начиная с ${c.n}-го хода`,
  example: {
    input: example([rule("Clock", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap", { kind: "turnAtLeast", n: 2 })], [plain("Brute", 9, 1)]),
    trace: "Clock (from turn 2) → front enemy (Brute) → 1 damage",
  },
};
