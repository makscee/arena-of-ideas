// The reference meta — the fixed set of opponents a creation candidate is
// gated against. All built from the shipped stress content (SPEC §7) the same
// way the example team files are, so the bar is the game's own material, not a
// bespoke balance fixture. Pure data; a knob, not a kernel rule.
//
// A candidate's overall win-rate across these teams is what the sim gate
// (src/gate.ts) judges against the band. Three archetypes span the strategy
// space the stress set affords — aggro tempo, attrition/sustain, and a fat
// status-stacked frontline. The pooled band alone does NOT stop a candidate
// from beating one shape while folding to another (those average out); the
// gate's per-matchup floor (GATE_MATCHUP_FLOOR) is what forces viability
// against every archetype here, so spanning the space only bites with the
// floor on.

import {
  Necromancer,
  Silencer,
  Summoner,
  Venomancer,
} from "./stress.js";
import type { ReferenceTeam } from "../gate.js";

/** Aggro tempo — poison pressure plus a value body that trades up on death. */
const AggroVenom: ReferenceTeam = {
  name: "AggroVenom",
  units: [
    Venomancer,
    Summoner,
    { name: "Brawler", base: { hp: 12, pwr: 3 }, ability: "Strike" },
  ],
};

/** Attrition — silence the threat, recur the dead, grind to fatigue. */
const SustainControl: ReferenceTeam = {
  name: "SustainControl",
  units: [
    Silencer,
    Necromancer,
    { name: "Warden", base: { hp: 14, pwr: 2 }, ability: "Strike" },
  ],
};

/** A fat status-stacked frontline — what a mid-run leveled board looks like. */
const StatStack: ReferenceTeam = {
  name: "StatStack",
  units: [
    Venomancer,
    { name: "Warlord", base: { hp: 16, pwr: 4 }, ability: "Strike", statuses: [{ status: "Strength", stacks: 2 }] },
    { name: "Bulwark", base: { hp: 14, pwr: 3 }, ability: "Strike", statuses: [{ status: "Vitality", stacks: 3 }] },
  ],
};

/** The reference meta, in a fixed order (the gate sweeps each in turn; order
 * does not affect the pooled win-rate but keeps the report stable). */
export const REFERENCE_META: readonly ReferenceTeam[] = [AggroVenom, SustainControl, StatStack];
