import { example, plain, rule } from "./examples.js";
import type { ConditionWord } from "./types.js";
import type { ConditionV2 } from "../types.js";

type C = Extract<ConditionV2, { kind: "turnIs" }>;

/** Fires only on turn n (§3.2). */
export const turnIs: ConditionWord<C> = {
  kind: "turnIs",
  class: "condition",
  check(ctx, c, at) {
    return ctx.turn === c.n;
  },
  en: (c, say) => `on turn ${c.n}`,
  ru: (c, say) => `на ${c.n}-м ходу`,
  example: {
    input: example([rule("Clock", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap", { kind: "turnIs", n: 2 })], [plain("Brute", 9, 1)]),
    trace: "Clock (on turn 2) → front enemy (Brute) → 1 damage",
  },
};
