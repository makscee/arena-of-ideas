// The sigil engine (mission arena#34): a unit's portrait is a small JSON spec
// drawn on a canvas. Nothing in the client mounts one yet.
import { FALLBACK_SPEC } from "./fallback";
import { validate, type Spec } from "./spec";
export { compile, evalNum, expressionProblem, ExprError, FUNCTION_NAMES, VARS, type Env } from "./grammar";
export { PALETTE_NAMES, isPaletteName, type Team } from "./palette";
export { MAX_COPIES, MAX_SHAPES, SHAPES, shapeCount, validate, type Layer, type Spec, type Validation } from "./spec";
export { beatAt, drawSigil, type DrawOptions, type DrawState } from "./draw";
export { mountSigil, mountedCount, resetSigils, type LiveOptions, type Pulse, type SigilHandle } from "./live";
export { FALLBACK_SPEC };

/** A unit's spec when it validates, else the generic one. */
export function specOrFallback(spec: unknown): Spec {
  return validate(spec).ok ? (spec as Spec) : FALLBACK_SPEC;
}
