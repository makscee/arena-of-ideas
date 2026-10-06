// What changes between two forms' texts (the sheet's "See Awoken" underline),
// worked out over describe.ts's pieces so the underline survives highlighting.
// Pure: no DOM, so it is tested under node.
import type { DescribeSegment } from "../../src/describe";

export interface Token {
  tok: string;
  /** The index of the piece of `next` the token sits in. */
  piece: number;
  /** Not in `was`: underlined. Whitespace is never marked. */
  changed: boolean;
}

/** `next`'s words, each marked changed unless it is in `was`, by a word-level
 * LCS over the whole sentence. Punctuation is its own token, so "ally." and
 * "ally, then …" share "ally". */
export function changedTokens(was: DescribeSegment[], next: DescribeSegment[]): Token[] {
  const split = (t: string) => t.split(/(\s+|[.,;:!?])/).filter((x) => x !== "");
  const a = was.flatMap((s) => split(s.text));
  const b = next.flatMap((s, piece) => split(s.text).map((tok) => ({ tok, piece })));
  const n = a.length;
  const m = b.length;
  const lcs = Array.from({ length: n + 1 }, () => new Array<number>(m + 1).fill(0));
  for (let i = n - 1; i >= 0; i--) for (let j = m - 1; j >= 0; j--) lcs[i]![j] = a[i] === b[j]!.tok ? lcs[i + 1]![j + 1]! + 1 : Math.max(lcs[i + 1]![j]!, lcs[i]![j + 1]!);
  const out: Token[] = [];
  let i = 0;
  let j = 0;
  while (j < m) {
    const { tok, piece } = b[j]!;
    if (i < n && a[i] === tok) {
      out.push({ tok, piece, changed: false });
      i++;
      j++;
    } else if (i < n && lcs[i + 1]![j]! >= lcs[i]![j + 1]!) i++;
    else {
      out.push({ tok, piece, changed: !/^\s*$/.test(tok) });
      j++;
    }
  }
  return out;
}
