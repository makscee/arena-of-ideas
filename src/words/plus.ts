import { example, plain, rule } from "./examples.js";
import type { AmountWord } from "./types.js";
import type { AmountV2 } from "../types.js";

type A = Extract<AmountV2, { kind: "plus" }>;

/** The sum of two amounts (§3.3). */
export const plus: AmountWord<A> = {
  kind: "plus",
  class: "counter",
  value(ctx, a, at) {
    return at.eval(a.a) + at.eval(a.b);
  },
  en: (a, say) => `${say(a.a)} plus ${say(a.b)}`,
  ru: (a, say) => `${say(a.a)} плюс ${say(a.b)}`,
  example: {
    input: example([plain("Squire", 9, 1), rule("Nautilus", 9, 1, { on: "Hurt", unit: "ally" }, { kind: "holder" }, "BraceAll")], [plain("Brute", 12, 2)]),
    trace: "Nautilus → self (Nautilus) → Ward 3",
  },
};
