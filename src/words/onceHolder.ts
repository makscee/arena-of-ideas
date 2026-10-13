import { example, plain, rule } from "./examples.js";
import type { ConditionWord } from "./types.js";
import type { ConditionV2 } from "../types.js";

type C = Extract<ConditionV2, { kind: "onceHolder" }>;

/** The rule fires at most once per battle (§3.2). */
export const onceHolder: ConditionWord<C> = {
  kind: "onceHolder",
  class: "condition",
  check(_ctx, _c, at) {
    const mark = `battle:${at.key}`;
    if (at.marked(mark)) return false;
    at.marks.push(mark);
    return true;
  },
  en: () => "once per battle",
  ru: () => "раз за бой",
  example: {
    input: example([rule("Clock", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap", { kind: "onceHolder" })], [plain("Brute", 9, 1)]),
    trace: "Clock (once per battle) → front enemy (Brute) → 1 damage",
  },
};
