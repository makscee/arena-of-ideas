import { example, plain, rule } from "./examples.js";
import type { ConditionWord } from "./types.js";
import type { ConditionV2 } from "../types.js";

type C = Extract<ConditionV2, { kind: "flip" }>;

/** One seeded coin per firing: heads with probability p. The engine logs
 * each coin as a Flipped event, heads or tails (§3.2, guardrails). */
export const flip: ConditionWord<C> = {
  kind: "flip",
  class: "condition",
  check(ctx, c, at) {
    const heads = ctx.draw() < c.p;
    at.flips.push(heads);
    return heads;
  },
  en: (c) => (c.p === 0.5 ? "on heads" : `with a ${Math.round(c.p * 100)}% chance`),
  ru: (c) => (c.p === 0.5 ? "если выпал орёл" : `с шансом ${Math.round(c.p * 100)}%`),
  example: {
    input: example([rule("Gambler", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap", { kind: "flip", p: 0.5 })], [plain("Brute", 9, 1)]),
    trace: "Gambler flips: heads",
  },
};
