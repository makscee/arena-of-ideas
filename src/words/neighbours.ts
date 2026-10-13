import { allyAhead } from "./allyAhead.js";
import { allyBehind } from "./allyBehind.js";
import { example, plain, rule } from "./examples.js";
import type { SelectorWord } from "./types.js";

/** Both neighbours of the holder: the ally ahead, then the ally behind (§3.1). */
export const neighbours: SelectorWord = {
  kind: "neighbours",
  class: "position",
  side: "ally",
  select(ctx, holder, event) {
    return [...allyAhead.select(ctx, holder, event), ...allyBehind.select(ctx, holder, event)];
  },
  en: { phrase: "its neighbours", chip: "Neighbours", trace: "its neighbours" },
  ru: { dat: "соседям", acc: "соседей", chip: "Соседи", trace: "соседи" },
  example: {
    input: example([plain("Archer", 3, 2), rule("Candle", 4, 1, { on: "BattleStart" }, { kind: "neighbours" }, "Empower"), plain("Squire", 3, 1)], [plain("Brute", 6, 2)]),
    trace: "Candle → its neighbours (Archer, Squire)",
  },
};
