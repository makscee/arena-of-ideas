import { example, plain, rule } from "./examples.js";
import type { AmountWord } from "./types.js";
import { statusRu } from "../status-names.js";
import type { AmountV2 } from "../types.js";

type A = Extract<AmountV2, { kind: "stacksOf" }>;

/** A named counter or status on the holder or the target (§3.3). */
export const stacksOf: AmountWord<A> = {
  kind: "stacksOf",
  class: "counter",
  value(ctx, a, at) {
    const u = a.of === "holder" ? at.holder : at.target;
    return u ? ctx.stacks(u, a.status) : 0;
  },
  en: (a, say) => `${a.of === "holder" ? "its" : "the target's"} ${a.status}`,
  ru: (a, say) => `числу ${statusRu(a.status).count[2]} ${a.of === "holder" ? "у себя" : "у цели"}`,
  example: {
    input: example([rule("Drummer", 6, 1, { on: "TurnStart" }, { kind: "holder" }, "Empower"), rule("Echo", 6, 1, { on: "TurnStart" }, { kind: "allyAhead" }, "Mirror")], [plain("Brute", 12, 1)]),
    trace: "Echo → the ally ahead of it (Drummer) → Strength 1",
  },
};
