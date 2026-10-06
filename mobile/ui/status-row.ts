// A battle card's status rows (R2-17): how many status chips fit, measured
// in the client's font and pixels, so it lives here, not in the pure kernel.

/** A status chip's width on a battle card: an 11 px icon plus 5.6 px a digit
 * (10 px IBM Plex Mono, −0.04em), 2 px between chips; "+n" is its digits plus 6 px (padding and border). */
const chipWidth = (stacks: number) => 11 + 5.6 * String(stacks).length;
const CHIP_GAP = 2;
const moreWidth = (n: number) => 5.6 * (1 + String(n).length) + 6;

/** How many of a card's statuses (their stacks, in order) its status rows
 * show, `rows` rows of `width` px filled in order as flex-wrap fills them;
 * when not all fit, the shown ones leave room for a last "+n" chip. A
 * two-digit stack is a wider chip, so a row may hold 2 where it held 3. */
export function statusesShown(stacks: number[], width: number, rows = 2): number {
  const fits = (widths: number[]) => {
    let row = 1;
    let x = 0;
    let n = 0;
    for (const cw of widths) {
      const need = x ? x + CHIP_GAP + cw : cw;
      if (need <= width + 0.5) x = need;
      else if (++row <= rows && cw <= width + 0.5) x = cw;
      else break;
      n++;
    }
    return n;
  };
  const widths = stacks.map(chipWidth);
  if (fits(widths) >= stacks.length) return stacks.length;
  let shown = stacks.length - 1;
  while (shown > 0 && fits([...widths.slice(0, shown), moreWidth(stacks.length - shown)]) < shown + 1) shown--;
  return shown;
}

/** A status row's width when none was measured yet: three one-digit chips. */
export const STATUS_ROW_FALLBACK = 3 * chipWidth(1) + 2 * CHIP_GAP;
