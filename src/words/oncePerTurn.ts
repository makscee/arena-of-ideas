import { example, plain, rule } from "./examples.js";
import type { ConditionWord } from "./types.js";
import type { ConditionV2 } from "../types.js";

type C = Extract<ConditionV2, { kind: "oncePerTurn" }>;

/** The rule fires at most once per turn (§3.2). */
export const oncePerTurn: ConditionWord<C> = {
  kind: "oncePerTurn",
  class: "condition",
  check(ctx, _c, at) {
    const mark = `turn${ctx.turn}:${at.key}`;
    if (at.marked(mark)) return false;
    at.marks.push(mark);
    return true;
  },
  en: () => "once per turn",
  ru: () => "раз за ход",
  example: {
    input: example([plain("Archer", 9, 1), rule("Medic", 6, 1, { on: "Hurt", unit: "ally" }, { kind: "eventUnit" }, "Mend", { kind: "oncePerTurn" })], [plain("Brute", 9, 1), plain("Imp", 3, 1)]),
    trace: "Medic (once per turn) → it (Archer)",
  },
};
