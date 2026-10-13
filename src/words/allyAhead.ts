import { example, plain, rule } from "./examples.js";
import { slotOf } from "./position.js";
import type { SelectorWord } from "./types.js";

/** The living ally one slot ahead of the holder (§3.1). */
export const allyAhead: SelectorWord = {
  kind: "allyAhead",
  class: "position",
  side: "ally",
  select(ctx, holder) {
    const i = slotOf(ctx, holder);
    const u = i > 0 ? ctx.line(holder.side)[i - 1] : undefined;
    return u ? [u] : [];
  },
  en: { phrase: "the ally ahead", chip: "Ally ahead", trace: "the ally ahead of it" },
  ru: { dat: "союзнику впереди", acc: "союзника впереди", chip: "Союзник впереди", trace: "союзник впереди" },
  example: {
    input: example([plain("Archer", 3, 2), rule("Banner", 4, 1, { on: "BattleStart" }, { kind: "allyAhead" }, "Empower")], [plain("Brute", 6, 2)]),
    trace: "Banner → the ally ahead of it (Archer) → Strength 1",
  },
};
