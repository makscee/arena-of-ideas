// Public API of the v5 kernel — everything a CLI or browser client imports.

export const KERNEL_VERSION = "5.0.0-alpha.0";

export { battle, toJSONL, winnerOf, TEAM_SIZE, FATIGUE_START, FATIGUE_RAMP, TURN_CAP } from "./battle.js";
export { initRun, buy, reroll, reorder, fight, applyDecision, playRun, runToJSONL, toBattleTeam, InvalidDecisionError } from "./run.js";
export type { RunInput, RunState, RunUnit, RunDecision, RunEvent, RunEventBody, RunEventType } from "./run.js";
export * from "./tunables.js";
export { sweep, sweepSeeds, sweepOutcome, summarizeSweep } from "./sweep.js";
export type { SweepInput, SweepOutcome, SweepStats, SweepResult } from "./sweep.js";
export { renderReplay } from "./replay.js";
export { displayNames, ancestry, abilityRefDesc, shortDesc, deathCauseChain } from "./trace.js";
export type { NameOf } from "./trace.js";
export { boardAt } from "./board.js";
export type { BoardState, BoardUnit } from "./board.js";
export { assertValidContent, validateTeam, validateRegistry, ValidationError } from "./validate.js";
export type { ValidationIssue } from "./validate.js";
export type * from "./types.js";
export * from "./content/stress.js";
