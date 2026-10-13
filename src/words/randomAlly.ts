import { example, plain, rule } from "./examples.js";
import type { SelectorWord } from "./types.js";

/** One seeded draw among the holder's other living allies (§3.1). */
export const randomAlly: SelectorWord = {
  kind: "randomAlly",
  class: "position",
  side: "ally",
  select(ctx, holder) {
    const pool = ctx.line(holder.side).filter((u) => u.id !== holder.id);
    if (pool.length === 0) return [];
    return [pool[Math.floor(ctx.draw() * pool.length)]!];
  },
  en: { phrase: "a random ally", chip: "Random ally", trace: "a random ally" },
  ru: { dat: "случайному союзнику", acc: "случайного союзника", chip: "Случайный союзник", trace: "случайный союзник" },
  example: {
    input: example([rule("Jester", 4, 1, { on: "BattleStart" }, { kind: "randomAlly" }, "Empower"), plain("Archer", 3, 2), plain("Squire", 3, 1)], [plain("Brute", 6, 2)]),
    trace: "Jester → a random ally (Squire)",
  },
};
