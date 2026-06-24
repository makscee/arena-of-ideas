// Per-status colour (#065 item 2): each status name gets its OWN stable colour
// so "Poison" and "Shield" are always told apart at a glance, and a given status
// is the SAME colour everywhere it shows — the chip on a card, the ±N status
// badge in the overlay, and the card lines that mention it.
//
// Built-in statuses get a CURATED hue — hand-picked so the seven shipped
// statuses are clearly distinct (no two share a neighbourhood on the hue wheel).
// Author-made statuses fall back to the hash-stable hue from the unit card's
// `nameHue`, which kept the old per-status identity for everything not in the
// curated map. The function signature is unchanged; only the source of the hue
// changed for built-ins.
//
// Legibility is the constraint, not the hue: a status renders on three very
// different backdrops (the near-black overlay pill, the dim card chip bg, and
// the dark card line). So we emit a BRIGHT, desaturated-enough text hue that
// reads on dark, plus a matching translucent ring — never a saturated fill that
// would clash with the dark pill. Same hue, two tuned lightnesses for the two
// jobs (a coloured number on a dark pill vs a coloured chip).

import { nameHue } from "./unit-card.js";

// Curated hues for the seven built-in statuses (src/content/stress.ts).
// Spread across the wheel so no two built-ins land in the same neighbourhood.
// Hue values chosen for semantic sense where possible, distinctness first:
//   Strength  20   — orange-red (power, aggression)
//   Blessing  45   — amber-gold (divine, positive)
//   Poison   120   — green (toxic, the classic poison colour)
//   Freeze   185   — cyan (cold, ice)
//   Shield   210   — steel blue (protection)
//   Curse    280   — violet (dark, negative)
//   Vitality 345   — rose (life, health)
//
// Author-made statuses get nameHue() — the original hash-stable fallback — so
// no player-authored status is ever colourless.
const CURATED_HUES: Record<string, number> = {
  Strength: 20,
  Blessing: 45,
  Poison: 120,
  Freeze: 185,
  Shield: 210,
  Curse: 280,
  Vitality: 345,
};

/** The hue (0–360) for a status name: curated for built-ins, hash-stable for
 * author-made statuses. */
export function statusHue(status: string): number {
  const curated = CURATED_HUES[status];
  return curated !== undefined ? curated : nameHue(status);
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
