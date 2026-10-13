import { EXAMPLE_ABILITIES, example, plain, rule } from "./examples.js";
import type { EffectWord } from "./types.js";
import type { EffectV2 } from "../types.js";

type E = Extract<EffectV2, { kind: "actAgain" }>;

/** The target's rule fires now, as if its trigger had fired. It counts against the step cap, passes the once-per-chain law, and a unit acts again at most once per event (§3.5). */
export const actAgain: EffectWord<E> = {
  kind: "actAgain",
  class: "ruleMod",
  context: "trigger",
  en: (_e, _say, target) => `${target} acts again`,
  ru: (_e, _say, target) => `заставить ${target} действовать ещё раз`,
  example: {
    input: { ...example([rule("Hourglass", 6, 1, { on: "TurnEnd" }, { kind: "allyBehind" }, "Again"), rule("Archer", 4, 1, { on: "TurnEnd" }, { kind: "frontEnemy" }, "Zap")], [plain("Brute", 12, 1)]), abilities: { ...EXAMPLE_ABILITIES, Again: { name: "Again", family: "Arcane", effects: [{ kind: "actAgain" }] } } },
    trace: "Hourglass: Archer acts again",
  },
};
