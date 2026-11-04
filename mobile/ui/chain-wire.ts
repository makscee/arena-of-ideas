// The chain on the field (arena#30, mission #26 move 2): one step at a time. The field draws only the current
// step's wire, with its number on it; the tray lists the steps so far in words with CHAIN ×N beside them.
// Pure: no DOM. battle.ts and the shop's coach draw it.

/** A step the tray lists: its number from 1 and its caption (battle.ts turns the caption into words). */
export interface ChainRow<S> {
  n: number;
  step: S;
  /** The step on screen: its wire is the one drawn. */
  now: boolean;
}

/** Whether a beat's steps read as a chain: two or more links, one setting off the next. */
export const chainShown = (steps: number): boolean => steps >= 2;

/** The tray's list for a beat of `waves` with `wave` (from 0) on screen: the steps so far, never one still to come. */
export function chainRows<S>(waves: readonly S[], wave: number): ChainRow<S>[] {
  if (!chainShown(waves.length) || wave < 0) return [];
  return waves.slice(0, wave + 1).map((step, i) => ({ n: i + 1, step, now: i === wave }));
}

/** CHAIN ×N: the links landed so far. */
export const chainCount = (wave: number): number => wave + 1;

export interface Pt {
  x: number;
  y: number;
}

/** Where the step's number sits on a quadratic wire: its middle (t = ½). */
export const wireMiddle = (p0: Pt, c: Pt, p1: Pt): Pt => ({ x: (p0.x + 2 * c.x + p1.x) / 4, y: (p0.y + 2 * c.y + p1.y) / 4 });
