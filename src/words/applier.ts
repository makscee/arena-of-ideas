import { example, plain, rule } from "./examples.js";
import type { SelectorWord } from "./types.js";

/** The unit whose rule gave the triggering status: a StatusApplied's source
 * (#1063, Prism). None when the kernel itself applied it. */
export const applier: SelectorWord = {
  kind: "applier",
  class: "position",
  select(ctx, _holder, event) {
    if (event.type !== "StatusApplied" || event.source === "kernel") return [];
    const u = ctx.unit(event.source.unit);
    return u && u.alive ? [u] : [];
  },
  en: { phrase: "the unit that gave it", chip: "Giver", trace: "the giver" },
  ru: { dat: "тому, кто дал", acc: "того, кто дал", chip: "Давший", trace: "давшего" },
  example: {
    input: example(
      [rule("Prism", 8, 1, { on: "StatusApplied", unit: "holder", status: "Strength" }, { kind: "applier" }, "Zap")],
      [rule("Witch", 6, 1, { on: "BattleStart" }, { kind: "frontEnemy" }, "Empower"), plain("Brute", 6, 2)],
    ),
    trace: "Prism → the giver (Witch) → 1 damage",
  },
};
