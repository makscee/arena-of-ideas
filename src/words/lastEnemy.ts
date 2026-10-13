import { example, plain, rule } from "./examples.js";
import type { SelectorWord } from "./types.js";

/** The enemy at the back of its line (§3.1). */
export const lastEnemy: SelectorWord = {
  kind: "lastEnemy",
  class: "position",
  side: "enemy",
  select(ctx, holder) {
    const line = ctx.line(holder.side === "A" ? "B" : "A");
    const u = line[line.length - 1];
    return u ? [u] : [];
  },
  en: { phrase: "the back enemy", chip: "Back enemy", trace: "the enemy at the back" },
  ru: { dat: "заднему врагу", acc: "заднего врага", chip: "Задний враг", trace: "задний враг" },
  example: {
    input: example([rule("Sniper", 4, 1, { on: "TurnStart" }, { kind: "lastEnemy" }, "Zap")], [plain("Brute", 6, 2), plain("Healer", 2, 1)]),
    trace: "Sniper → the enemy at the back (Healer)",
  },
};
