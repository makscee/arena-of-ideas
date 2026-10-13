import { example, plain, rule } from "./examples.js";
import { compare, opEn, opRu } from "./compare.js";
import type { ConditionWord } from "./types.js";
import type { ConditionV2 } from "../types.js";

type C = Extract<ConditionV2, { kind: "enemiesAlive" }>;

/** Counts the living enemies (§3.2). */
export const enemiesAlive: ConditionWord<C> = {
  kind: "enemiesAlive",
  class: "condition",
  check(ctx, c, at) {
    return compare(ctx.line(at.holder.side === "A" ? "B" : "A").length, c.op, c.n);
  },
  en: (c, say) => `if ${opEn(c.op)} ${c.n} ${c.n === 1 ? "enemy stands" : "enemies stand"}`,
  ru: (c, say) => `если врагов ${opRu(c.op)} ${c.n}`,
  example: {
    input: example([rule("Duelist", 6, 2, { on: "TurnStart" }, { kind: "holder" }, "Empower", { kind: "enemiesAlive", op: "<=", n: 1 })], [plain("Imp", 1, 1), plain("Brute", 9, 1)]),
    trace: "Duelist (if at most 1 enemy stands) → self (Duelist)",
  },
};
