// Tiers as players see them everywhere (R3-3): card corners, sheet heads, the
// Codex chips, the legend and the rules. Its own file so a node test can load it.
const ROMAN = ["I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X"];

/** 1-10 as I-X; anything else as digits. */
export function roman(n: number): string {
  return ROMAN[n - 1] ?? `${n}`;
}
