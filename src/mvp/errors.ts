// A decision the run rules refuse (mission #574). Its own module so the run
// (./run.ts) and the unit rules (./forms.ts) can both throw it.
import type { Decision } from "./contract.js";

export class MvpDecisionError extends Error {
  constructor(readonly kind: Decision["kind"] | "abandon", reason: string) {
    super(`invalid ${kind}: ${reason}`);
    this.name = "MvpDecisionError";
  }
}

/** A decision the API can't read (an unknown kind, or an index that isn't one): a bad request, not a
 * refusal, so not an MvpDecisionError. The server maps it to a 400 (and an
 * MvpDecisionError to a 409); src/ knows nothing of HTTP. */
export class MvpBadDecision extends Error {
  constructor(
    readonly kind: unknown,
    reason?: string,
  ) {
    super(reason ? `bad ${String(kind)}: ${reason}` : `unknown decision kind ${JSON.stringify(kind) ?? String(kind)}`);
    this.name = "MvpBadDecision";
  }
}
