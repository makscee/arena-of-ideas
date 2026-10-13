import { example, plain, rule } from "./examples.js";
import type { AmountWord } from "./types.js";
import type { AmountV2 } from "../types.js";

type A = Extract<AmountV2, { kind: "missingHp" }>;

/** The holder's missing hp (§3.3). */
export const missingHp: AmountWord<A> = {
  kind: "missingHp",
  class: "counter",
  value(ctx, a, at) {
    return ctx.missingHp(at.holder);
  },
  en: (a, say) => "its missing HP",
  ru: (a, say) => "недостающим ОЗ",
  example: {
    input: example([rule("Berserker", 9, 1, { on: "Hurt", unit: "holder" }, { kind: "frontEnemy" }, "Rage")], [plain("Brute", 20, 2)]),
    trace: "Berserker → front enemy (Brute) → 2 damage",
  },
};
