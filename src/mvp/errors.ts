// A decision the run rules refuse (mission #574). Its own module so the run
// (./run.ts) and the unit rules (./forms.ts) can both throw it.
import type { Decision } from "./contract.js";

export class MvpDecisionError extends Error {
  constructor(readonly kind: Decision["kind"], reason: string) {
    super(`invalid ${kind}: ${reason}`);
    this.name = "MvpDecisionError";
  }
}
