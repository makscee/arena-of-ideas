import { example, plain, rule } from "./examples.js";
import type { ConditionWord } from "./types.js";
import type { ConditionV2 } from "../types.js";

type T = Extract<ConditionV2, { kind: "outnumbered" }>;

/** True while the enemy line has more living units than the holder's (§3.2;
 * #1063, Fortune Teller). */
export const outnumbered: ConditionWord<T> = {
  kind: "outnumbered",
  class: "condition",
  check(ctx, _c, at) {
    const mine = ctx.line(at.holder.side).length;
    const theirs = ctx.line(at.holder.side === "A" ? "B" : "A").length;
    return theirs > mine;
  },
  en: () => "if the enemy has more units",
  ru: () => "если у врага больше бойцов",
  example: {
    input: example([rule("Seer", 6, 1, { on: "BattleStart" }, { kind: "holder" }, "Empower", { kind: "outnumbered" })], [plain("Brute", 9, 1), plain("Thug", 3, 1)]),
    trace: "Seer (if the enemy has more units) → self (Seer) → Strength 1",
  },
};
