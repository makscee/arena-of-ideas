// Pure geometry of the battle's motion (arena#50, MOVES 1): where a striker's portrait goes.

/** The lunge's reach in stage pixels: a little over a quarter of the way to the target, at most this much. */
export const LUNGE_MAX_PX = 32;
export const LUNGE_SHARE = 0.28;

/** The offset a striker travels along the real vector (dx, dy) to its target, in the vector's own units. Zero with no
 * distance; never more than LUNGE_MAX_PX, and never past the target. */
export function lungeVector(dx: number, dy: number): { x: number; y: number } {
  const dist = Math.hypot(dx, dy);
  if (!dist) return { x: 0, y: 0 };
  const reach = Math.min(LUNGE_MAX_PX, dist * LUNGE_SHARE);
  return { x: (dx / dist) * reach, y: (dy / dist) * reach };
}
