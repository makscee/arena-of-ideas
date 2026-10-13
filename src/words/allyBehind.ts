import { example, plain, rule } from "./examples.js";
import { slotOf } from "./position.js";
import type { SelectorWord } from "./types.js";

/** The living ally one slot behind the holder (§3.1). */
export const allyBehind: SelectorWord = {
  kind: "allyBehind",
  class: "position",
  side: "ally",
  select(ctx, holder) {
    const i = slotOf(ctx, holder);
    const u = i >= 0 ? ctx.line(holder.side)[i + 1] : undefined;
    return u ? [u] : [];
  },
  en: { phrase: "the ally behind", chip: "Ally behind", trace: "the ally behind it" },
  ru: { dat: "союзнику позади", acc: "союзника позади", chip: "Союзник позади", trace: "союзник позади" },
  example: {
    input: example([rule("Hourglass", 4, 1, { on: "BattleStart" }, { kind: "allyBehind" }, "Empower"), plain("Archer", 3, 2), plain("Squire", 3, 1)], [plain("Brute", 6, 2)]),
    trace: "Hourglass → the ally behind it (Archer)",
  },
};
