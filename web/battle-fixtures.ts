// Harness-owned deterministic board-first scenarios. These are available only
// through the dev server query seam in main.ts; they use committed Unit defs,
// the production battle kernel, projections, viewer and controls.

import { battle, DEFAULT_RUN_POOL, stressAbilities, stressRegistry, type BattleEvent, type UnitDef } from "../src/index.js";
import type { BattleContent } from "./viewer.js";

export type BoardFirstScenarioName = "board-first-strike-status-chain" | "board-first-summon-death-advance";

const named = (names: string[]): UnitDef[] => names.map((name) => {
  const def = DEFAULT_RUN_POOL.find((unit) => unit.name === name);
  if (def === undefined) throw new Error(`board-first fixture requires committed Unit ${name}`);
  return def;
});

const definitions: Record<BoardFirstScenarioName, { teamA: string[]; teamB: string[]; seed: number }> = {
  "board-first-strike-status-chain": {
    teamA: ["Venomancer", "Necromancer", "Summoner", "Brawler", "Bulwark"],
    teamB: ["Brawler", "Squire", "Bulwark", "Summoner", "Necromancer"],
    seed: 0,
  },
  "board-first-summon-death-advance": {
    teamA: ["Summoner", "Necromancer", "Brawler", "Squire", "Bulwark"],
    teamB: ["Brawler", "Squire", "Bulwark", "Venomancer", "Necromancer"],
    seed: 0,
  },
};

export interface BoardFirstFixture { name: BoardFirstScenarioName; log: BattleEvent[]; content: BattleContent }

export function boardFirstFixture(name: string): BoardFirstFixture {
  if (!(name in definitions)) throw new Error(`unknown board-first fixture: ${name}`);
  const scenarioName = name as BoardFirstScenarioName;
  const spec = definitions[scenarioName];
  const teamA = named(spec.teamA);
  const teamB = named(spec.teamB);
  const log = battle({ teamA, teamB, seed: spec.seed, statuses: stressRegistry, abilities: stressAbilities });
  const has = (type: BattleEvent["type"]) => log.some((event) => event.type === type);
  if (scenarioName === "board-first-strike-status-chain") {
    const statusTick = log.some((event) => event.type === "Hurt" && event.source !== "kernel" && event.source.status === "Poison");
    const multiHop = log.some((event) => {
      let parent = event.causedBy;
      let hops = 0;
      while (parent !== null && hops < log.length) { hops++; parent = log[parent]?.causedBy ?? null; }
      return hops >= 3;
    });
    if (!["Strike", "Hurt", "StatusApplied", "StatusRemoved"].every((type) => has(type as BattleEvent["type"])) || !statusTick || !multiHop)
      throw new Error("board-first-strike-status-chain no longer produces its named event families");
  } else {
    const deathSummon = log.some((event) => event.type === "Summon" && event.causedBy !== null && log[event.causedBy]?.type === "Death");
    if (!has("Death") || !has("Summon") || !deathSummon) throw new Error("board-first-summon-death-advance no longer produces its named event families");
  }
  return { name: scenarioName, log, content: { teams: { A: teamA, B: teamB }, registry: stressRegistry, abilities: stressAbilities, meta: { opponent: scenarioName, seed: spec.seed } } };
}
