import { example, plain, rule } from "./examples.js";
import type { SelectorWord } from "./types.js";

/** The unit the triggering event was aimed at: a Strike's defender, the unit
 * a Hurt, Heal or status landed on (§3.1). */
export const eventTarget: SelectorWord = {
  kind: "eventTarget",
  class: "position",
  select(ctx, _holder, event) {
    const id = event.type === "Strike" ? event.defender : "unit" in event && typeof event.unit === "string" ? event.unit : undefined;
    const u = id !== undefined ? ctx.unit(id) : undefined;
    return u && u.alive ? [u] : [];
  },
  en: { phrase: "its target", chip: "Target", trace: "the target" },
  ru: { dat: "цели", acc: "цель", chip: "Цель", trace: "цель" },
  example: {
    input: example([plain("Archer", 5, 2), rule("Medic", 4, 1, { on: "Strike", striker: "enemy" }, { kind: "eventTarget" }, "Mend")], [plain("Brute", 6, 2)]),
    trace: "Medic → the target (Archer) → heal 1",
  },
};
