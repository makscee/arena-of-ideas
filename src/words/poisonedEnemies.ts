import { EXAMPLE_ABILITIES, EXAMPLE_STATUSES, example, plain, rule } from "./examples.js";
import type { SelectorWord } from "./types.js";

/** Every living enemy carrying Poison, in line order (§3.1; #1063,
 * Alchemist). */
export const poisonedEnemies: SelectorWord = {
  kind: "poisonedEnemies",
  class: "position",
  side: "enemy",
  select(ctx, holder) {
    return ctx.line(holder.side === "A" ? "B" : "A").filter((u) => ctx.stacks(u, "Poison") > 0);
  },
  en: { phrase: "every poisoned enemy", chip: "Poisoned enemies", trace: "every poisoned enemy" },
  ru: { dat: "всем отравленным врагам", acc: "всех отравленных врагов", chip: "Отравленные враги", trace: "все отравленные враги" },
  example: {
    input: {
      ...example([rule("Alchemist", 6, 1, { on: "TurnStart" }, { kind: "poisonedEnemies" }, "Zap")], [plain("Brute", 9, 1), rule("Ogre", 9, 1, { on: "BattleStart" }, { kind: "holder" }, "Taint")]),
      statuses: { ...EXAMPLE_STATUSES, Poison: { name: "Poison", counter: true, abilities: [] } },
      abilities: { ...EXAMPLE_ABILITIES, Taint: { name: "Taint", family: "Poison", effects: [{ kind: "applyStatus", status: "Poison", stacks: { kind: "const", value: 1 } }] } },
    },
    trace: "Alchemist → every poisoned enemy (Ogre) → 1 damage",
  },
};
