// Run-economy tunables — every number in the shop/run layer, in one place.
// All of these are sim-tunable knobs, not design pins (SPEC §6 spirit): the
// simulation farm tunes them; content and the run kernel never hardcode them.
// Battle-side constants (TEAM_SIZE, FATIGUE_*, TURN_CAP) stay in battle.ts.

/** Gold in hand when a run begins — round 1's shopping budget (SAP-like 10). */
export const STARTING_GOLD = 10;

/** Gold granted when a new round starts, on top of whatever carried over. */
export const INCOME_BASE = 10;

/** Income growth per round; 0 = flat SAP-like income, raise for richer late rounds. */
export const INCOME_PER_ROUND = 0;

/** Ceiling on per-round income so a growth curve can't run away. */
export const INCOME_CAP = 15;

/** Price of one shop offer; flat for v1 — per-unit pricing is the budget's job. */
export const UNIT_COST = 3;

/** Price of refreshing the shop; cheap so digging for a copy is a real line of play. */
export const REROLL_COST = 1;

/** Offers shown in round 1. */
export const SHOP_SIZE_BASE = 3;

/** One more offer every this many rounds — a slowly widening market. */
export const SHOP_SIZE_STEP = 3;

/** Offer ceiling — offers stay in the 3–6 band. */
export const SHOP_SIZE_MAX = 6;

/** Copies of a unit that fuse into a level-up (SAP: three of a kind). */
export const STACK_THRESHOLD = 3;

/** Fight losses a run survives; ending the run at 0 is the ladder's rule (slice 2). */
export const STARTING_LIVES = 5;

/** Base hp gained per level-up. */
export const LEVEL_HP_GROWTH = 2;

/** Base pwr gained per level-up. */
export const LEVEL_PWR_GROWTH = 1;

/** The income curve: what a new round adds to the carryover. */
export function incomeForRound(round: number): number {
  return Math.min(INCOME_CAP, INCOME_BASE + (round - 1) * INCOME_PER_ROUND);
}

/** The shop-size curve: 3 offers early, widening to 6 as the run goes long. */
export function shopSizeForRound(round: number): number {
  return Math.min(SHOP_SIZE_MAX, SHOP_SIZE_BASE + Math.floor((round - 1) / SHOP_SIZE_STEP));
}
