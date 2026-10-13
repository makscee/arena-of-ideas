import { example, plain, rule } from "./examples.js";
import type { AmountWord } from "./types.js";
import type { AmountV2 } from "../types.js";

type A = Extract<AmountV2, { kind: "half" }>;

/** Half an amount, rounded down (§3.3). */
export const half: AmountWord<A> = {
  kind: "half",
  class: "counter",
  value(ctx, a, at) {
    return Math.floor(at.eval(a.of) / 2);
  },
  en: (a, say) => `half ${say(a.of)}`,
  ru: (a, say) => `половине ${say(a.of)}`,
  example: {
    input: example([rule("Apprentice", 9, 4, { on: "TurnStart" }, { kind: "frontEnemy" }, "HalfHit")], [plain("Brute", 12, 1)]),
    trace: "Apprentice → front enemy (Brute) → 2 damage",
  },
};
