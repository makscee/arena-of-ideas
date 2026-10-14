import { EXAMPLE_ABILITIES, example, plain, rule } from "./examples.js";
import type { EffectWord } from "./types.js";
import type { EffectV2 } from "../types.js";

type E = Extract<EffectV2, { kind: "performRemembered" }>;

/** The holder performs its remembered rules: the oldest once (then forgets it), or all of them (kept) (§3.5). */
export const performRemembered: EffectWord<E> = {
  kind: "performRemembered",
  class: "ruleMod",
  context: "trigger",
  en: (e) => (e.mode === "once" ? "perform the oldest remembered rule once and forget it" : "perform every remembered rule"),
  ru: (e) => (e.mode === "once" ? "исполнить самое старое запомненное правило и забыть его" : "исполнить все запомненные правила"),
  example: {
    input: { ...example([rule("Archer", 1, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap"), { ...rule("Raven", 6, 1, { on: "Death", unit: "otherAlly" }, { kind: "eventUnit" }, "Recall"), triggers: [{ kind: "trigger", on: { on: "Death", unit: "otherAlly" } }, { kind: "trigger", on: { on: "TurnStart" } }], also: [{ selectors: [{ kind: "holder" }], abilities: ["Perform"] }] }], [plain("Brute", 12, 2)]), abilities: { ...EXAMPLE_ABILITIES, Recall: { name: "Recall", family: "Arcane", effects: [{ kind: "remember" }] }, Perform: { name: "Perform", family: "Arcane", effects: [{ kind: "performRemembered", mode: "all" }] } } },
    trace: "Raven performs the rule of Archer",
  },
};
