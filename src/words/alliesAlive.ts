import { example, plain, rule } from "./examples.js";
import { compare, opEn, opRu } from "./compare.js";
import type { ConditionWord } from "./types.js";
import type { ConditionV2 } from "../types.js";

type C = Extract<ConditionV2, { kind: "alliesAlive" }>;

/** Counts the holder's living allies, itself included (§3.2). */
export const alliesAlive: ConditionWord<C> = {
  kind: "alliesAlive",
  class: "condition",
  check(ctx, c, at) {
    return compare(ctx.line(at.holder.side).length, c.op, c.n);
  },
  en: (c, say) => `if ${opEn(c.op)} ${c.n} ${c.n === 1 ? "ally stands" : "allies stand"}`,
  ru: (c, say) => `если союзников ${opRu(c.op)} ${c.n}`,
  example: {
    input: example([plain("Squire", 1, 1), rule("Last Stand", 3, 1, { on: "Death", unit: "otherAlly" }, { kind: "holder" }, "Empower", { kind: "alliesAlive", op: "=", n: 1 })], [plain("Brute", 9, 2)]),
    trace: "Last Stand (if exactly 1 ally stands) → self (Last Stand)",
  },
};
