import { example, plain, rule } from "./examples.js";
import type { ConditionWord } from "./types.js";
import type { ConditionV2 } from "../types.js";

type C = Extract<ConditionV2, { kind: "turnEvery" }>;

/** Fires every n-th turn (§3.2). */
export const turnEvery: ConditionWord<C> = {
  kind: "turnEvery",
  class: "condition",
  check(ctx, c, at) {
    return c.n > 0 && ctx.turn % c.n === 0;
  },
  en: (c, say) => `every ${c.n === 2 ? "second" : c.n === 3 ? "third" : `${c.n}th`} turn`,
  ru: (c, say) => `каждый ${c.n}-й ход`,
  example: {
    input: example([rule("Clock", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap", { kind: "turnEvery", n: 2 })], [plain("Brute", 9, 1)]),
    trace: "Clock (every second turn) → front enemy (Brute)",
  },
};
