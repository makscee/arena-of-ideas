// One line of units on the table (arena#29 round 4, mission #26): where its columns sit and how big its pieces
// are, for a line of `cols` units. The battle uses it; the shop's field can take the same numbers so a line
// looks the same on both screens. Pure: stage pixels, no DOM.
import type { Orient } from "./stage-fit";

/** The field a line is centred in, and its size, per stage (stage pixels). The stage's middle is the field's. */
export const FIELD: Record<Orient, { left: number; width: number }> = { L: { left: 190, width: 900 }, P: { left: 12, width: 366 } };

/** The shop line's piece (main.ts pieceSizes().line): the size a battle's piece is measured against. */
export const SHOP_PX: Record<Orient, number> = { L: 70, P: 56 };

/** A short line's pieces grow up to this multiple of the shop's, never beyond it. */
export const GROW_MAX = 1.4;

/** The columns' pitch for `cols` slots across the field: spaced like the shop's slots, a short line closer than a
 * full one so it stays one group in the field's middle; a long one (past five) shares the field. */
export function linePitch(o: Orient, cols: number): number {
  const n = Math.max(1, cols);
  const base = o === "L" ? 160 : n <= 3 ? 110 : 91;
  return Math.floor(Math.min(base, FIELD[o].width / n));
}

/** The largest piece a line of `cols` may draw: the shop's × GROW_MAX, and no wider than its column allows
 * (the piece's name and rim take 1.4× its face). */
export const pieceCap = (o: Orient, cols: number): number => Math.min(Math.floor(SHOP_PX[o] * GROW_MAX), Math.floor(linePitch(o, cols) / 1.4));

/** How far a piece's star, rim, counter and status chips reach past its face, both sides together, as a share of the
 * face (measured on the table: 30 px beside a 50 px face). Two neighbours' reaches must not meet. */
export const REACH_X = 0.6;

/** A full-width line of `cols` units (Home's TO BEAT on the phone, where the line spreads over the whole stage):
 * the column pitch, and the largest face whose reach still leaves its neighbour's clear. */
export function spreadLine(width: number, cols: number): { pitch: number; px: number } {
  const pitch = Math.floor(width / Math.max(1, cols));
  return { pitch, px: Math.floor(pitch / (1 + REACH_X)) };
}
