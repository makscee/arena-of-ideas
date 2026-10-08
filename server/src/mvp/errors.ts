// Server-side MVP errors (mission #574).

/** A route whose slice hasn't landed yet: the API answers 501 with the
 * message. A slice replaces the stub that throws it. */
export class MvpNotYet extends Error {
  override name = "MvpNotYet";
}
