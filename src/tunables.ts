// Run/ladder tunables — every knob in the shop/run/ladder layer, in one place.
// All of these are sim-tunable knobs, not design pins (SPEC §6 spirit): the
// simulation farm tunes them; content and the run kernel never hardcode them.
// Battle-side constants (TEAM_SIZE, FATIGUE_*, TURN_CAP) stay in battle.ts.

import { Necromancer, Silencer, Summoner, Venomancer } from "./content/stress.js";
import type { UnitDef } from "./types.js";

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

/** How many rounds of an empty ladder get bootstrap ghosts. Depth 1 made a
 * first-ever run crown at round 2 — no climb, no game. Seeding rounds 1..DEPTH
 * gives the first session a real ladder to outclimb before the champion. */
export const BOOTSTRAP_DEPTH = 3;

/** Teams seeding rounds 1..BOOTSTRAP_DEPTH of an empty ladder (openLadder):
 * BOOTSTRAP_TEAMS[r-1] is round r's pool, so a first-ever run has opponents
 * at every bootstrap round. Composed from the shipped stress units (SPEC §7)
 * the way the example team files are; composition is a knob like any other.
 * Strength escalates with the round to track what a played run fields there —
 * round 1 ≈ one 10-gold shop phase (3 bodies), each later round adds a body
 * and fatter vanilla stats, round 3 opens with status stacks. Status
 * references (Poison via Venomancer, Strength/Vitality below) resolve in any
 * registry containing the stress statuses (the CLI and tests use
 * stressRegistry); openLadder gates every team at seed time. */
export const BOOTSTRAP_TEAMS: readonly (readonly UnitDef[][])[] = [
  // round 1 — three bodies, the scale of a first shop phase
  [
    [Venomancer, Summoner, { name: "Brawler", base: { hp: 12, pwr: 2 } }],
    [Silencer, Necromancer, { name: "Bulwark", base: { hp: 10, pwr: 3 } }],
  ],
  // round 2 — a fourth body, vanilla stats grown a notch
  [
    [Venomancer, Summoner, Necromancer, { name: "Brawler", base: { hp: 14, pwr: 3 } }],
    [Silencer, Venomancer, { name: "Bulwark", base: { hp: 13, pwr: 4 } }, { name: "Squire", base: { hp: 8, pwr: 2 } }],
  ],
  // round 3 — full lines; status openers stand in for a level-up's worth of growth
  [
    [
      Venomancer,
      Summoner,
      Silencer,
      { name: "Brawler", base: { hp: 16, pwr: 4 }, statuses: [{ status: "Strength", stacks: 2 }] },
      { name: "Bulwark", base: { hp: 14, pwr: 4 }, statuses: [{ status: "Vitality", stacks: 3 }] },
    ],
    [
      Necromancer,
      Summoner,
      Venomancer,
      { name: "Warden", base: { hp: 15, pwr: 5 } },
      { name: "Squire", base: { hp: 10, pwr: 3 } },
    ],
  ],
];

/** The team seated in the champion spot when an empty ladder opens — the
 * strongest shipped bootstrap team, one notch past round BOOTSTRAP_DEPTH's
 * pool. Sweeps showed a fresh ladder crowning every run at round
 * BOOTSTRAP_DEPTH+1 through the vacant spot (20/20, 13 with losing records):
 * no spot to take, no game. With this team seated, a crown is always earned
 * by beating someone; the kernel's vacant-spot rule stays only for the
 * truly-vacant edge (unreachable on a fresh ladder). Gated at open like
 * every bootstrap team. */
export const BOOTSTRAP_CHAMPION: readonly UnitDef[] = [
  Venomancer,
  Summoner,
  Necromancer,
  { name: "Warlord", base: { hp: 18, pwr: 5 }, statuses: [{ status: "Strength", stacks: 3 }] },
  { name: "Bulwark", base: { hp: 16, pwr: 5 }, statuses: [{ status: "Vitality", stacks: 4 }] },
];

/** The draftable pool a run's shop rolls from while no curated pool exists:
 * the stress casters (SPEC §7) plus vanilla bodies, the bootstrap-team
 * composition. Shared by CLI autoplay and the web run screen, so both fill
 * the same ladder with the same meta. A knob, not a pin. */
export const DEFAULT_RUN_POOL: UnitDef[] = [
  Venomancer,
  Summoner,
  Silencer,
  Necromancer,
  { name: "Brawler", base: { hp: 12, pwr: 2 } },
  { name: "Bulwark", base: { hp: 10, pwr: 3 } },
  { name: "Squire", base: { hp: 8, pwr: 2 } },
];

/** The creation sim-gate's default win-rate band and sweep depth — the
 * empirical "balanced" definition a candidate must clear (src/gate.ts). A
 * candidate's overall win-rate vs the reference meta must land in
 * [GATE_BAND_MIN, GATE_BAND_MAX] inclusive: below is filler, above is
 * overtuned. Empirical for now — the budget formula slots in later as one more
 * check (vision). A knob, never prose in a README; a per-task gate config may
 * override these. */
export const GATE_BAND_MIN = 0.35;
export const GATE_BAND_MAX = 0.65;

/** Per-matchup win-rate floor — every matchup vs the reference meta must clear
 * it, or the candidate is "counter-folded" even when its pooled win-rate sits
 * in-band. The pooled band alone is gameable: a candidate that hard-counters
 * one reference team to 100% and folds to another at 0% averages into the band
 * yet is unplayable — the exact line an AI magnitude-tuner steers into. The
 * floor forces broad viability. A knob, overridable per task via gate.json. */
export const GATE_MATCHUP_FLOOR = 0.25;

/** Seeds the gate sweeps per matchup. Enough to make a win-rate estimate
 * stable across the band edges without making a hand-run slow. */
export const GATE_SEEDS = 50;

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
