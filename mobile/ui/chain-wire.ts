// The chain on the field (arena#30, mission #26 move 2): one step at a time. The field draws only the current
// step's wire, with its number on it; the tray's ladder numbers the steps in words with CHAIN ×N above them (arena#52).
// Pure: no DOM. battle.ts and the shop's coach draw it.

/** Whether a beat's steps read as a chain: two or more links, one setting off the next. */
export const chainShown = (steps: number): boolean => steps >= 2;

/** CHAIN ×N: the links landed so far. */
export const chainCount = (wave: number): number => wave + 1;

export interface Pt {
  x: number;
  y: number;
}

/** Where the step's number sits on a quadratic wire: its middle (t = ½). */
export const wireMiddle = (p0: Pt, c: Pt, p1: Pt): Pt => ({ x: (p0.x + 2 * c.x + p1.x) / 4, y: (p0.y + 2 * c.y + p1.y) / 4 });
