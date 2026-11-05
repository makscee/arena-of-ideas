// A fusion's portrait (mission arena#34, slice F): unit A's body with unit B's
// mark, the duel's `Sigil.fuse`. Visual only: the fusion's rule, name and stats
// come from the fusion itself and are never read from here.
import { FALLBACK_SPEC } from "./fallback";
import { validate, type Spec } from "./spec";

/** The portrait of the fusion of `a` and `b` (each a unit's spec; one that is missing or invalid counts as the generic sigil).
 * A pair whose body and mark together draw more than the shape budget keeps A's whole portrait instead, so a fusion is never refused a picture. */
export function fuse(a: unknown, b: unknown): Spec {
  const first = validate(a).ok ? (a as Spec) : FALLBACK_SPEC;
  const second = validate(b).ok ? (b as Spec) : FALLBACK_SPEC;
  const fused: Spec = { body: first.body, ...(second.mark?.length ? { mark: second.mark } : {}) };
  return validate(fused).ok ? fused : first;
}
