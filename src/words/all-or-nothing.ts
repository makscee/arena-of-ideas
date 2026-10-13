// Gambler's rule as one effect, apart from branch.ts: fixtures.ts and
// units-v2.ts import it, and importing branch.ts first would load the word
// index (./index.js) before branch is defined, leaving the validator without
// the branch word.
import type { Effect } from "../types.js";

/** Gambler's rule: heads deals double, tails none. */
export const ALL_OR_NOTHING: Effect = {
  kind: "branch",
  if: { kind: "flip", p: 0.5 },
  then: [{ kind: "replaceAmount", amount: { kind: "twice", of: { kind: "stat", stat: "pwr", of: "holder" } } }],
  else: [{ kind: "replaceAmount", amount: { kind: "const", value: 0 } }],
};
