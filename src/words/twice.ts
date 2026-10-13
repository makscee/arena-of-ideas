import { example, plain, rule } from "./examples.js";
import type { AmountWord } from "./types.js";
import type { AmountV2 } from "../types.js";

type A = Extract<AmountV2, { kind: "twice" }>;

/** Twice an amount (§3.3). */
export const twice: AmountWord<A> = {
  kind: "twice",
  class: "counter",
  value(ctx, a, at) {
    return 2 * at.eval(a.of);
  },
  en: (a, say) => `twice ${say(a.of)}`,
  ru: (a, say) => `удвоенному ${say(a.of)}`,
  example: {
    input: example([rule("Champion", 9, 2, { on: "TurnStart" }, { kind: "frontEnemy" }, "DoubleHit")], [plain("Brute", 12, 1)]),
    trace: "Champion → front enemy (Brute) → 4 damage",
  },
};
