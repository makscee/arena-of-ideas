// Each tier's stat range (Maks, 10-17, after playing batch 3, #1048): a
// unit's sleeping PWR+HP stays inside its tier's range, which rises with the
// tier and widens with it. "Should be possible to have very different stats
// depending on rule, but maybe not in tier one and not to such an extent."
// The tune (scripts/tune-v2.ts) moves numbers only inside the range; a unit
// that still misses the band at the range's edge gets its rule's numbers
// changed (amounts, which turns), not its stats.

import type { Tier } from "./contract.js";

export const TIER_STATS: Record<Tier, { min: number; max: number }> = {
  1: { min: 8, max: 12 },
  2: { min: 11, max: 17 },
  3: { min: 13, max: 23 },
  4: { min: 15, max: 29 },
};

/** Units whose rule is built around 0 power: they keep PWR 0, every other unit
 * has PWR at least 1. The report says why. */
export const ZERO_PWR: Record<string, string> = {
  Seed: "a seed that sleeps two turns, then grows: it doesn't strike until it has grown",
  Chrysalis: "a cocoon: it doesn't strike, it becomes a Butterfly that does",
};

export const totalOf = (u: { pwr: number; hp: number }) => u.pwr + u.hp;
export const inTierRange = (u: { tier: Tier; pwr: number; hp: number }) => {
  const r = TIER_STATS[u.tier];
  return totalOf(u) >= r.min && totalOf(u) <= r.max;
};
