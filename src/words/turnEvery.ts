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
  // the turns themselves, so "every third turn" never reads as "every third time" (Maks, 10-14)
  en: (c) => `on turns ${c.n}, ${c.n * 2}, ${c.n * 3}…`,
  ru: (c) => `на ${c.n}-м, ${c.n * 2}-м, ${c.n * 3}-м… ходу`,
  example: {
    input: example([rule("Clock", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap", { kind: "turnEvery", n: 2 })], [plain("Brute", 9, 1)]),
    trace: "Clock (on turns 2, 4, 6…) → front enemy (Brute) → 1 damage",
  },
};
