// PRD #081 — the migration determinism gate.
//
// The Ability-entity migration must be BEHAVIOR-PRESERVING: resolving a unit's
// `ability` ref through the registry has to produce the SAME event log the old
// inline `abilities[]` did, for the same (teams, seed). This file pins that two
// ways:
//
//   1. A golden JSONL fixture of representative stress battles, captured in
//      slice 1 against the (then mostly-inline) shipped corpus. The SAME battles
//      re-run after every later slice migrates content + drops the back-compat
//      read; the snapshot must stay byte-identical. A diverging snapshot is the
//      slice-3 STOP condition — the migration changed behavior and reverts.
//   2. A focused round-trip: one unit (Venomancer) battled via an `ability` ref
//      vs. via the legacy inline ability, asserted byte-identical — the slice-1
//      check that the resolver's ref lookup is correct (must-fail-first: break
//      the lookup and this diverges).

import { describe, expect, it } from "vitest";
import { battle, toJSONL } from "./battle.js";
import { Venomancer, stressAbilities, stressRegistry } from "./content/stress.js";
import { REFERENCE_META } from "./content/reference-meta.js";
import { BOOTSTRAP_TEAMS, BOSS_TEAMS, TOWER_HEIGHT } from "./tunables.js";
import type { UnitDef } from "./types.js";

// Explicit bodies exercising the interceptor statuses (Shield/Freeze/Blessing/
// Curse) that the shipped teams don't field. Built ref-shaped from the start
// (ability "Strike", the inert vanilla ability) so they are stable across every
// migration slice — only the imported shipped content changes shape, never this.
const shieldTank: UnitDef = { name: "ShieldTank", base: { hp: 10, pwr: 3 }, ability: "Strike", statuses: [{ status: "Shield", stacks: 5 }] };
const cursed: UnitDef = { name: "Cursed", base: { hp: 9, pwr: 5 }, ability: "Strike", statuses: [{ status: "Curse", stacks: 2 }] };
const frozen: UnitDef = { name: "Frozen", base: { hp: 8, pwr: 4 }, ability: "Strike", statuses: [{ status: "Freeze", stacks: 2 }] };
const blessed: UnitDef = { name: "Blessed", base: { hp: 6, pwr: 2 }, ability: "Strike", statuses: [{ status: "Blessing", stacks: 5 }] };

interface GoldenBattle {
  label: string;
  teamA: UnitDef[];
  teamB: UnitDef[];
  seed: number;
}

/** Representative stress battles — Poison/Summon/Silence/Resurrect/Strength/
 * Vitality via the shipped meta, and Shield/Freeze/Blessing/Curse via the
 * explicit interceptor bodies. The set spans all ten §7 abilities. */
const GOLDEN_BATTLES: GoldenBattle[] = [
  { label: "aggro-vs-sustain s1", teamA: [...REFERENCE_META[0]!.units], teamB: [...REFERENCE_META[1]!.units], seed: 1 },
  { label: "aggro-vs-sustain s7", teamA: [...REFERENCE_META[0]!.units], teamB: [...REFERENCE_META[1]!.units], seed: 7 },
  { label: "statstack-vs-champion s2", teamA: [...REFERENCE_META[2]!.units], teamB: [...BOSS_TEAMS[TOWER_HEIGHT - 1]!], seed: 2 },
  { label: "bootstrap-top-vs-boss s1", teamA: [...BOOTSTRAP_TEAMS[TOWER_HEIGHT - 1]![0]!], teamB: [...BOSS_TEAMS[2]!], seed: 1 },
  { label: "interceptors s1", teamA: [shieldTank, cursed], teamB: [frozen, blessed], seed: 1 },
  { label: "interceptors s3", teamA: [blessed, shieldTank], teamB: [cursed, frozen], seed: 3 },
];

function renderGolden(): string {
  return GOLDEN_BATTLES.map((b) => {
    const log = battle({ teamA: b.teamA, teamB: b.teamB, seed: b.seed, statuses: stressRegistry, abilities: stressAbilities });
    return `# ${b.label}\n${toJSONL(log)}`;
  }).join("\n");
}

describe("PRD #081 migration determinism", () => {
  it("representative stress battles match the golden log (byte-identical across migration)", async () => {
    await expect(renderGolden()).toMatchFileSnapshot("./__fixtures__/golden-battles.jsonl");
  });

  it("Venomancer via ability-ref produces a byte-identical log to the inline ability", () => {
    const victim: UnitDef = { name: "Victim", base: { hp: 12, pwr: 1 }, ability: "Strike" };
    const venomRef: UnitDef = { name: "Venomancer", base: { hp: 6, pwr: 1 }, ability: "Venom" };
    const refLog = toJSONL(battle({ teamA: [venomRef], teamB: [victim], seed: 7, statuses: stressRegistry, abilities: stressAbilities }));
    const inlineLog = toJSONL(battle({ teamA: [Venomancer], teamB: [victim], seed: 7, statuses: stressRegistry, abilities: stressAbilities }));
    expect(refLog).toBe(inlineLog);
    // And it actually exercises the ability — a no-op resolver would also match.
    expect(refLog).toContain('"status":"Poison"');
  });
});
