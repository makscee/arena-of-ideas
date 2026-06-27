// Public API of the v5 kernel — everything a CLI or browser client imports.

export const KERNEL_VERSION = "5.0.0-alpha.0";

export { battle, toJSONL, winnerOf, TEAM_SIZE, FATIGUE_START, FATIGUE_RAMP, TURN_CAP, fatigueAmount } from "./battle.js";
export { initRun, buy, reroll, reorder, fuse, fight, ladderFight, challengeBoss, applyDecision, playRun, runToJSONL, serializeRun, deserializeRun, toBattleTeam, InvalidDecisionError } from "./run.js";
export type { RunInput, RunState, RunStatus, RunEndReason, RunUnit, RunDecision, RunEvent, RunEventBody, RunEventType } from "./run.js";
// The file backing (FileLadderStore) lives in ladder-file.ts, off this index:
// it needs node:fs and the browser client imports this module. Its engine
// (PersistedLadderStore + the LadderData shape) is medium-free and exported
// here — the web's localStorage backing builds on it.
export { InMemoryLadderStore, PersistedLadderStore, openEmptyLadder, seedBootstrapTower, emptyLadderData, parseLadderData, BOOTSTRAP_RUN_ID } from "./ladder.js";
export type { LadderStore, LadderData, TeamSnapshot } from "./ladder.js";
// The season archive — immutable, append-only history of finished seasons. Like
// the ladder, the file backing (FileSeasonArchiveStore) lives off this index in
// season-archive-file.ts (it needs node:fs); its medium-free engine
// (PersistedSeasonArchiveStore + the SeasonArchiveData shape) is exported here.
export {
  InMemorySeasonArchiveStore,
  PersistedSeasonArchiveStore,
  emptySeasonArchiveData,
  parseSeasonArchiveData,
  serializeSeasonArchive,
  emptyFinalTower,
  assertSeasonInOrder,
  FIRST_CONTENT_VERSION,
} from "./season-archive.js";
export type { SeasonArchiveStore, SeasonArchiveData, SeasonRecord, ContentVersion } from "./season-archive.js";
// The season clock (#077 slice 2) — the live season pointer and the
// archive→reset→bump→open transition. Same off-index split as the ladder/archive:
// the file backing (FileSeasonPointerStore) needs node:fs and lives in
// season-file.ts; the medium-free engine + the transition op are exported here.
export {
  InMemorySeasonPointerStore,
  PersistedSeasonPointerStore,
  emptySeasonPointer,
  emptySeasonPointerData,
  parseSeasonPointerData,
  serializeSeasonPointer,
  snapshotLadder,
  transitionSeason,
  FIRST_SEASON,
} from "./season.js";
export type {
  SeasonPointer,
  SeasonPointerStore,
  SeasonPointerData,
  SeasonTransitionOps,
  SeasonTransitionResult,
} from "./season.js";
// The season-history READ surface (#077 slice 3) over the archive store: list
// summaries (season + version + champion) and a single season's final tower,
// rendered. Pure presentation, shared by the CLI and the web history view.
export {
  seasonChampion,
  summarizeSeason,
  summarizeSeasons,
  championLabel,
  formatSeasonLine,
  formatHistoryList,
  formatFinalTower,
} from "./season-history.js";
export type { SeasonSummary } from "./season-history.js";
// The ideas table (#076) — the same store-interface pattern as the ladder: one
// IdeaStore interface, an in-memory backing and a persisted engine, a serialized
// IdeasData shape that round-trips. The web's localStorage backing builds on it.
export { InMemoryIdeaStore, PersistedIdeaStore, emptyIdeasData, parseIdeasData, assertSubmittableText, castVote, voteScore, votedCount, rankIdeas } from "./ideas.js";
export type { IdeaStore, IdeasData, Idea, VoteDir, VoteMap } from "./ideas.js";
// The season selection rule (#083) — a pure partition of the ideas table into
// the build slate and the carry-over, read off #082's directional vote data.
export { isEligible, selectSeason, tallyOf, talliesOf, DEFAULT_SELECTION_TUNABLES } from "./selection.js";
export type { IdeaTally, RankedIdea, SelectionResult, SelectionTunables } from "./selection.js";
export * from "./tunables.js";
export { sweep, sweepSeeds, sweepOutcome, summarizeSweep, winRate } from "./sweep.js";
export type { SweepInput, SweepOutcome, SweepStats, SweepResult } from "./sweep.js";
export { runGate, formatGateReport } from "./gate.js";
export type { GateConfig, GateReport, MatchupResult, ReferenceTeam } from "./gate.js";
export { REFERENCE_META } from "./content/reference-meta.js";
export { renderReplay } from "./replay.js";
export { displayNames, ancestry, abilityRefDesc, shortDesc, deathCauseChain } from "./trace.js";
export {
  abilityPartRefs,
  abilityStatusRefs,
  describeAbility,
  describeAbilitySegments,
  describeAmount,
  describeCondition,
  describeConditionSegments,
  describeEffect,
  describeEffectSegments,
  describeSelector,
  describeSelectorSegments,
  describeStatus,
  describeStatusSegments,
  describeWhen,
  describeWhenSegments,
} from "./describe.js";
export type { DescribeOpts, DescribeSegment, PartRef } from "./describe.js";
export type { NameOf } from "./trace.js";
export { boardAt } from "./board.js";
export type { BoardState, BoardUnit } from "./board.js";
export { beatsOf, beatAtStep, depthInBeat, isRootKind, overlaysAt, overlayHasContent, badgeValues, newBadgeKeysAt, coinHolderAt } from "./beats.js";
export type { Beat, BeatOverlay } from "./beats.js";
export { assertValidContent, assertValidPool, validateTeam, validatePool, validateRegistry, ValidationError } from "./validate.js";
export type { ValidationIssue } from "./validate.js";
export type * from "./types.js";
export * from "./content/stress.js";
export { buildCodex, codexUnits } from "./codex.js";
export type { CodexData, CodexStatusEntry, CodexAbilityEntry, CodexUnitEntry, CodexPartEntry, CodexRuleEntry } from "./codex.js";
export { partAtoms } from "./parts.js";
export type { PartAtom } from "./parts.js";
export { mergePool, parseApprovedRegistry, creditsOf } from "./registry.js";
export type { ApprovedRegistry, ApprovedUnit } from "./registry.js";
