// A decision the run rules refuse (mission #574). Its own module so the run
// (./run.ts) and the unit rules (./forms.ts) can both throw it.
import type { Decision } from "./contract.js";

export class MvpDecisionError extends Error {
  constructor(readonly kind: Decision["kind"], reason: string) {
    super(`invalid ${kind}: ${reason}`);
    this.name = "MvpDecisionError";
  }
}

/** A decision the API can't read (an unknown kind): a bad request, not a
 * refusal, so not an MvpDecisionError (409). The routes don't catch it; the
 * HTTP framework's error handler (Hono's) answers with getResponse(): a JSON
 * 400 shaped like the routes' own errors. */
export class MvpBadDecision extends Error {
  readonly status = 400;
  constructor(kind: unknown) {
    super(`unknown decision kind ${JSON.stringify(kind) ?? String(kind)}`);
    this.name = "MvpBadDecision";
  }
  getResponse(): Response {
    return Response.json({ error: this.message }, { status: this.status });
  }
}
