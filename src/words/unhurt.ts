import { example, plain, rule } from "./examples.js";
import type { ConditionWord } from "./types.js";
import type { ConditionV2 } from "../types.js";

type T = Extract<ConditionV2, { kind: "unhurt" }>;

/** True while the holder has taken no damage. */
export const unhurt: ConditionWord<T> = {
  kind: "unhurt",
  class: "condition",
  check(ctx, c, at) {
    return ctx.missingHp(at.holder) === 0;
  },
  en: () => "while unhurt",
  ru: () => "пока не ранен",
  example: {
    input: example([rule("Holder", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap", { kind: "unhurt" })], [plain("Brute", 9, 1)]),
    trace: "Holder (while unhurt) → front enemy (Brute) → 1 damage",
  },
};
