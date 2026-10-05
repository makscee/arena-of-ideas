// One MVP line against another (mission #574). Every fight between two lines
// goes through fightLines: the run's round and Crown fights (slice 4), the
// playoff games and the strongest-team pick (slice 5), the day-1 champion
// (slice 6) and the tuning sims (slice 7). So the rules (chainStepCap and the
// rest) reach sims and real fights alike; nobody calls battle() on MVP lines
// directly. Pure: the caller supplies the ids, the seed and the time.

import { battle, winnerOf } from "../battle.js";
import type { BattleEvent, Side, UnitDef } from "../types.js";
import type { BattleRecord, BattleUnit, FightKind, LineUnit, MvpContent, MvpRules, PlayerRef } from "./contract.js";

/** A line unit as battle() input. */
export function toBattleDef(u: LineUnit): UnitDef {
  return {
    name: u.name,
    base: { ...u.stats },
    triggers: u.recipe.when,
    selectors: u.recipe.who,
    abilities: u.recipe.does,
    ...(u.recipe.condition ? { condition: u.recipe.condition } : {}),
  };
}

/** A line as it entered the battle, each unit tagged with its kernel instance
 * id from the log's BattleStart roster. */
function battleTeam(line: LineUnit[], side: Side, log: BattleEvent[]): BattleUnit[] {
  const start = log.find((e) => e.type === "BattleStart");
  const roster = start?.type === "BattleStart" ? start.teams[side] : [];
  return line.map((u, i) => ({ ...structuredClone(u), id: roster[i]?.id ?? `${side}${i + 1}:${u.name}` }));
}

/** One side of a fight: whose line it is, front first. */
export interface FightSide {
  player: PlayerRef;
  line: LineUnit[];
}

export interface FightOptions {
  battleId: string;
  seed: number;
  kind: FightKind;
  /** The run's round; 0 for a playoff game. */
  round: number;
  /** The run that fights; null for a playoff game or a sim. */
  runId: string | null;
  /** ISO time of the fight (the kernel has no clock). */
  at: string;
  content: MvpContent;
  rules: MvpRules;
}

/** Fights `a` (side A) against `b` (side B) and returns the whole record:
 * the causal log, the winner and both teams as they entered. */
export function fightLines(a: FightSide, b: FightSide, o: FightOptions): BattleRecord {
  const log = battle({
    teamA: a.line.map(toBattleDef),
    teamB: b.line.map(toBattleDef),
    seed: o.seed,
    abilities: o.content.abilities,
    statuses: o.content.statuses,
    chainStepCap: o.rules.chainStepCap,
  });
  return {
    battleId: o.battleId,
    runId: o.runId,
    player: a.player,
    seed: o.seed,
    contentVersion: o.content.version,
    kind: o.kind,
    round: o.round,
    at: o.at,
    teamA: battleTeam(a.line, "A", log),
    teamB: battleTeam(b.line, "B", log),
    opponent: b.player,
    winner: winnerOf(log),
    log,
  };
}
