import { example, plain, rule } from "./examples.js";
import type { AmountWord } from "./types.js";
import type { AmountV2 } from "../types.js";

type A = Extract<AmountV2, { kind: "statOf" }>;

/** Someone else's effective stat (§3.3). */
export const statOf: AmountWord<A> = {
  kind: "statOf",
  class: "counter",
  value(ctx, a, at) {
    const u = a.of === "target" ? at.target : at.eventUnit;
    return u ? ctx.stat(u, a.stat) : 0;
  },
  en: (a, say) => `${a.of === "target" ? "the target's" : "its"} ${a.stat.toUpperCase()}`,
  ru: (a, say) => `${a.stat === "pwr" ? "АТК" : "ОЗ"} ${a.of === "target" ? "цели" : "его"}`,
  example: {
    input: example([rule("Riposter", 9, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Riposte")], [plain("Brute", 12, 2)]),
    trace: "Riposter → front enemy (Brute) → 2 damage",
  },
};
