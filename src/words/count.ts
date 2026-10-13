import { example, plain, rule } from "./examples.js";
import { FACT_EN, FACT_RU } from "./facts.js";
import type { AmountWord } from "./types.js";
import type { AmountV2 } from "../types.js";

type A = Extract<AmountV2, { kind: "count" }>;

/** A battle fact the kernel keeps (§3.3). */
export const count: AmountWord<A> = {
  kind: "count",
  class: "counter",
  value(ctx, a, at) {
    return ctx.fact(a.fact, at.holder);
  },
  en: (a, say) => FACT_EN[a.fact],
  ru: (a, say) => FACT_RU[a.fact],
  example: {
    input: example([rule("Nautilus", 9, 1, { on: "Hurt", unit: "holder" }, { kind: "holder" }, "Brace")], [plain("Brute", 12, 2)]),
    trace: "Nautilus → self (Nautilus) → Ward 3",
  },
};
