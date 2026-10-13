import { EXAMPLE_ABILITIES, example, plain, rule } from "./examples.js";
import type { EffectWord } from "./types.js";
import type { EffectV2 } from "../types.js";

type E = Extract<EffectV2, { kind: "delay" }>;

/** The effect lands at the end of the turn `turns` turns from now; the trace shows when it was set (§3.6). */
export const delay: EffectWord<E> = {
  kind: "delay",
  class: "timing",
  context: "trigger",
  en: (e, say) => `in ${e.turns} ${e.turns === 1 ? "turn" : "turns"}: ${say.effect(e.effect)}`,
  ru: (e, say) => `через ${e.turns} ${e.turns === 1 ? "ход" : e.turns < 5 ? "хода" : "ходов"}: ${say.effect(e.effect)}`,
  example: {
    input: { ...example([rule("Fuse", 6, 1, { on: "BattleStart" }, { kind: "frontEnemy" }, "Later")], [plain("Brute", 12, 1)]), abilities: { ...EXAMPLE_ABILITIES, Later: { name: "Later", family: "Arcane", effects: [{ kind: "delay", turns: 1, effect: { kind: "damage", amount: { kind: "const", value: 3 } } }] } } },
    trace: "Fuse sets a delay on Brute: lands at the end of turn 1",
  },
};

