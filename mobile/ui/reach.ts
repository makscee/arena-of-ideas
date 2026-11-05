// How far a piece reaches around its face box: pure arithmetic, shared by the piece (which draws it)
// and the layouts that must leave room for it.

/** How far an M piece with its name reaches above its face box (the tab) and
 * below the box's top (rim numbers and the name), px, for a face of side `s`;
 * the same sums piece() makes (Home's champion sizes its card from them). */
export function pieceReach(s: number, floor = 0): { above: number; below: number } {
  const tabH = Math.round(Math.max(20, s * 0.27, floor * 1.45));
  const d = Math.round(Math.max(22, s * 0.37, floor * 1.5));
  const nf = Math.round(Math.max(12, s * 0.2, floor));
  return { above: Math.round(tabH * 0.62), below: s - Math.round(d * 0.7) + d + 3 + nf + 2 };
}

