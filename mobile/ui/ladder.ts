// The battle tray's ladder (arena#52, mission #45 move 3): four fixed rows of steps in words. The newest step (or a turn's totals)
// is the foot row, the three above it are the steps before it, across beats and turns, so a turn's last steps stay in view when the
// next turn starts. A row with no step behind it stays empty and keeps its place: nothing in the tray changes size.
// Pure: no DOM. battle.ts draws it.

/** Rows in the ladder, the foot row included. */
export const LADDER_ROWS = 4;

/** The rows above the foot, oldest first, each a step or null (empty, but present). `cur` is the index of the step on screen in `seq`
 * (-1 before the first step: the face-off); with a turn's totals on the foot row, the step on screen is the newest of the rows above it. */
export function ladderAbove<T>(seq: readonly T[], cur: number, totals: boolean): (T | null)[] {
  const slots = LADDER_ROWS - 1;
  const end = cur < 0 ? 0 : totals ? cur + 1 : cur;
  const shown = seq.slice(Math.max(0, end - slots), end);
  return [...Array<null>(slots - shown.length).fill(null), ...shown];
}
