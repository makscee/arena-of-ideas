import { example, plain, rule } from "./examples.js";
import type { ConditionWord } from "./types.js";
import { ruStatusCount } from "../status-names.js";
import type { ConditionV2 } from "../types.js";

type C = Extract<ConditionV2, { kind: "stacksAtLeast" }>;

/** A counter or status threshold, on the holder or the event's unit (§3.2). */
export const stacksAtLeast: ConditionWord<C> = {
  kind: "stacksAtLeast",
  class: "condition",
  check(ctx, c, at) {
    const u = c.of === "holder" ? at.holder : "unit" in at.event && typeof at.event.unit === "string" ? ctx.unit(at.event.unit) : undefined;
    return u !== undefined && ctx.stacks(u, c.status) >= c.n;
  },
  en: (c, say) => `if ${c.of === "holder" ? "it has" : "the unit has"} ${c.n}+ ${c.status}`,
  ru: (c, say) => `если у ${c.of === "holder" ? "него" : "цели"} ${ruStatusCount(c.n, c.status)} или больше`,
  example: {
    input: example([rule("Hoarder", 6, 1, { on: "TurnStart" }, { kind: "holder" }, "Empower", { kind: "not", cond: { kind: "stacksAtLeast", status: "Strength", n: 2, of: "holder" } })], [plain("Brute", 9, 1)]),
    trace: "Hoarder (unless it has 2+ Strength) → self (Hoarder) → Strength 1",
  },
};
