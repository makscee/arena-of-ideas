import { example, plain, rule } from "./examples.js";
import { slotOf } from "./position.js";
import type { SelectorWord } from "./types.js";

/** Every living ally behind the holder, in line order (§3.1). */
export const alliesBehind: SelectorWord = {
  kind: "alliesBehind",
  class: "position",
  side: "ally",
  select(ctx, holder) {
    const i = slotOf(ctx, holder);
    return i >= 0 ? ctx.line(holder.side).slice(i + 1) : [];
  },
  en: { phrase: "every ally behind", chip: "Allies behind", trace: "every ally behind it" },
  ru: { dat: "всем союзникам позади", acc: "всех союзников позади", chip: "Союзники позади", trace: "все союзники позади" },
  example: {
    input: example([rule("Hourglass", 4, 1, { on: "BattleStart" }, { kind: "alliesBehind" }, "Empower"), plain("Archer", 3, 2), plain("Squire", 3, 1)], [plain("Brute", 6, 2)]),
    trace: "Hourglass → every ally behind it (Archer, Squire)",
  },
};
