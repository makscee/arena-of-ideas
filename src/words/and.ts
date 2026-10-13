import { example, plain, rule } from "./examples.js";
import type { ConditionWord } from "./types.js";
import type { ConditionV2 } from "../types.js";

type C = Extract<ConditionV2, { kind: "and" }>;

/** Every condition holds, checked in order; the first that fails stops the
 * check, so a later flip draws no coin (§3.2). */
export const and: ConditionWord<C> = {
  kind: "and",
  class: "condition",
  check: (_ctx, c, at) => c.all.every((x) => at.check(x)),
  en: (c, say) => c.all.map(say).join(" and "),
  ru: (c, say) => c.all.map(say).join(" и "),
  example: {
    input: example([rule("Clock", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap", { kind: "and", all: [{ kind: "turnAtLeast", n: 2 }, { kind: "enemiesAlive", op: ">=", n: 1 }] })], [plain("Brute", 9, 1)]),
    trace: "Clock (from turn 2 and if at least 1 enemy stands) → front enemy (Brute)",
  },
};
