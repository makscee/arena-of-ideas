import { example, plain, rule } from "./examples.js";
import type { ConditionWord } from "./types.js";
import type { ConditionV2 } from "../types.js";

type C = Extract<ConditionV2, { kind: "not" }>;

/** The condition does not hold (§3.2). */
export const not: ConditionWord<C> = {
  kind: "not",
  class: "condition",
  check: (_ctx, c, at) => !at.check(c.cond),
  en: (c, say) => `unless ${say(c.cond).replace(/^if /, "")}`,
  ru: (c, say) => `если не ${say(c.cond).replace(/^если /, "")}`,
  example: {
    input: example([rule("Clock", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap", { kind: "not", cond: { kind: "turnIs", n: 1 } })], [plain("Brute", 9, 1)]),
    trace: "Clock (unless on turn 1) → front enemy (Brute)",
  },
};
