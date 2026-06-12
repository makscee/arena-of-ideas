// Public API of the v5 kernel — everything a CLI or browser client imports.

export const KERNEL_VERSION = "5.0.0-alpha.0";

export { battle, toJSONL, winnerOf, TEAM_SIZE, FATIGUE_START, FATIGUE_RAMP, TURN_CAP, fatigueAmount } from "./battle.js";
export { initRun, buy, reroll, reorder, fight, ladderFight, applyDecision, playRun, runToJSONL, serializeRun, deserializeRun, toBattleTeam, InvalidDecisionError } from "./run.js";
export type { RunInput, RunState, RunStatus, RunEndReason, RunUnit, RunDecision, RunEvent, RunEventBody, RunEventType } from "./run.js";
// The file backing (FileLadderStore) lives in ladder-file.ts, off this index:
// it needs node:fs and the browser client imports this module. Its engine
// (PersistedLadderStore + the LadderData shape) is medium-free and exported
// here — the web's localStorage backing builds on it.
export { InMemoryLadderStore, PersistedLadderStore, openLadder, emptyLadderData, parseLadderData, BOOTSTRAP_RUN_ID } from "./ladder.js";
export type { LadderStore, LadderData, TeamSnapshot } from "./ladder.js";
export * from "./tunables.js";
export { sweep, sweepSeeds, sweepOutcome, summarizeSweep } from "./sweep.js";
export type { SweepInput, SweepOutcome, SweepStats, SweepResult } from "./sweep.js";
export { runGate, formatGateReport } from "./gate.js";
export type { GateConfig, GateReport, MatchupResult, ReferenceTeam } from "./gate.js";
export { REFERENCE_META } from "./content/reference-meta.js";
export { renderReplay } from "./replay.js";
export { displayNames, ancestry, abilityRefDesc, shortDesc, deathCauseChain } from "./trace.js";
export {
  abilityStatusRefs,
  describeAbility,
  describeAbilitySegments,
  describeAmount,
  describeCondition,
  describeEffect,
  describeEffectSegments,
  describeSelector,
  describeStatus,
  describeStatusSegments,
  describeWhen,
  describeWhenSegments,
} from "./describe.js";
export type { DescribeOpts, DescribeSegment } from "./describe.js";
export type { NameOf } from "./trace.js";
export { boardAt } from "./board.js";
export type { BoardState, BoardUnit } from "./board.js";
export { assertValidContent, assertValidPool, validateTeam, validatePool, validateRegistry, ValidationError } from "./validate.js";
export type { ValidationIssue } from "./validate.js";
export type * from "./types.js";
export * from "./content/stress.js";
export { buildCodex, codexUnits } from "./codex.js";
export type { CodexData, CodexStatusEntry, CodexUnitEntry, CodexRuleEntry } from "./codex.js";
export { mergePool, parseApprovedRegistry, creditsOf } from "./registry.js";
export type { ApprovedRegistry, ApprovedUnit } from "./registry.js";
