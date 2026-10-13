import { EXAMPLE_ABILITIES, example, plain, rule } from "./examples.js";
import type { EffectWord, Say } from "./types.js";
import type { Effect } from "../types.js";
import { describeEffect } from "../describe.js";
import type { EffectV2 } from "../types.js";

type E = Extract<EffectV2, { kind: "delay" }>;

/** The effect lands at the end of the turn `turns` turns from now; the trace shows when it was set (§3.6). */
export const delay: EffectWord<E> = {
  kind: "delay",
  class: "timing",
  context: "trigger",
  en: (e, say, target) => `in ${e.turns} ${e.turns === 1 ? "turn" : "turns"}: ${describeLater(e.effect, say, target)}`,
  ru: (e, say, target) => `через ${e.turns} ${e.turns === 1 ? "ход" : e.turns < 5 ? "хода" : "ходов"}: ${describeLaterRu(e.effect, say, target)}`,
  example: {
    input: { ...example([rule("Fuse", 6, 1, { on: "BattleStart" }, { kind: "frontEnemy" }, "Later")], [plain("Brute", 12, 1)]), abilities: { ...EXAMPLE_ABILITIES, Later: { name: "Later", family: "Arcane", effects: [{ kind: "delay", turns: 1, effect: { kind: "damage", amount: { kind: "const", value: 3 } } }] } } },
    trace: "Fuse sets a delay on Brute: lands at the end of turn 1",
  },
};

function describeLater(e: Effect, _say: Say, target: string): string {
  return describeEffect(e, target);
}

function describeLaterRu(e: Effect, say: Say, target: string): string {
  if (e.kind === "damage") return `${say.amount(e.amount)} урона`;
  if (e.kind === "transform") return `превращение в ${e.into.name}`;
  return describeEffect(e, target, { lang: "ru" });
}
