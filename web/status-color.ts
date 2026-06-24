// Per-status colour (#065 item 2): each status name gets its OWN stable colour
// so "Poison" and "Shield" are always told apart at a glance, and a given status
// is the SAME colour everywhere it shows — the chip on a card, the ±N status
// badge in the overlay, and the card lines that mention it. Reuses the unit
// card's `nameHue` (a hash-stable hue per name), so the scheme is one shared
// hash, never two drifting palettes.
//
// Legibility is the constraint, not the hue: a status renders on three very
// different backdrops (the near-black overlay pill, the dim card chip bg, and
// the dark card line). So we emit a BRIGHT, desaturated-enough text hue that
// reads on dark, plus a matching translucent ring — never a saturated fill that
// would clash with the dark pill. Same hue, two tuned lightnesses for the two
// jobs (a coloured number on a dark pill vs a coloured chip).

import { nameHue } from "./unit-card.js";

/** The hash-stable hue (0–360) for a status name — the shared identity. */
export function statusHue(status: string): number {
  return nameHue(status);
}

/** Inline style for an overlay `.ov-status` badge: a bright per-status number
 * colour + a tinted ring, on the badge family's dark pill. Mirrors how the
 * built-in `.ov-*` families set `--ov-fg`/`--ov-ring`, so a status badge sits
 * consistently beside the damage/heal/stat badges, only hued by its name. */
export function statusColorStyle(status: string): string {
  const h = statusHue(status).toFixed(0);
  // 78% lightness reads bright on the near-black pill; the ring is the same hue,
  // translucent, so the pill edge carries the status colour without a hard line.
  return `--ov-fg: hsl(${h} 70% 78%); --ov-ring: hsl(${h} 65% 60% / 0.85);`;
}

/** Inline style for a status CHIP (on a unit card) and for a card-LINE mention:
 * the chip's text takes the per-status hue (kept bright so it reads on the dim
 * chip bg and the dark line), with a matching translucent border so two chips
 * of different statuses are told apart by colour, not just text. */
export function statusChipStyle(status: string): string {
  const h = statusHue(status).toFixed(0);
  return `color: hsl(${h} 68% 74%); border-color: hsl(${h} 55% 50% / 0.6);`;
}
