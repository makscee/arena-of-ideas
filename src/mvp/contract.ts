// Arena MVP contract (mission #574, slice 1). Every later slice codes against
// these types and the HTTP API below: the kernel (forms, fusion, chains), the
// run server, the day, bots, content, the phone client and the battle viewer.
//
// Change it consciously: a field added here is a field every slice may rely
// on. Battle-level types (UnitDef, BattleEvent, …) stay in ../types.ts; this
// file only adds the MVP's run, day and rating layer on top.

import type { AbilityRegistry, BattleEvent, Condition, Selector, Side, Stats, StatusRegistry, When } from "../types.js";

export const MVP_API_VERSION = 1;
/** Every API path hangs off this prefix, relative to the app's base URL
 * (the test instance serves the app at /arena/, so /arena/api/v1/...). */
export const MVP_API_PREFIX = "/api/v1";

// ---------- rules (tunables; agents may change the numbers, not the shape) ----------

export interface MvpRules {
  /** Shop rounds before the Crown fight against today's champion. */
  rounds: number;
  /** Hearts at run start; a lost fight costs one; 0 ends the run before the Crown. */
  hearts: number;
  /** Gold each round, no carry-over. */
  goldPerRound: number;
  unitCost: number;
  rerollCost: number;
  /** Gold back for selling a unit. */
  sellRefund: number;
  offers: number;
  lineSize: number;
  /** The Nth copy of a unit swaps it to its awoken form (slice 2). */
  copiesToAwaken: number;
  /** Stats each extra copy adds. */
  copyGrowth: Stats;
  /** tierOpensAt[t-1] = the round tier t enters the shop. */
  tierOpensAt: number[];
  /** Cascade step cap; hitting it logs a visible "chain capped" event (slice 3). */
  chainStepCap: number;
  /** Elo K and the start rating (slice 4). */
  ratingK: number;
  ratingStart: number;
  /** Day rollover, "HH:MM" in dayTimeZone (slice 5). */
  dayEndsAt: string;
  dayTimeZone: string;
}

export const MVP_RULES: MvpRules = {
  rounds: 12,
  hearts: 5,
  goldPerRound: 10,
  unitCost: 3,
  rerollCost: 1,
  sellRefund: 1,
  offers: 5,
  lineSize: 5,
  copiesToAwaken: 3,
  copyGrowth: { pwr: 1, hp: 2 },
  tierOpensAt: [1, 3, 6, 9],
  chainStepCap: 64,
  ratingK: 32,
  ratingStart: 1000,
  dayEndsAt: "04:00",
  dayTimeZone: "Europe/Moscow",
};

// ---------- content: units with two forms ----------

export type UnitId = string;
export type Tier = 1 | 2 | 3 | 4;
export type FormKey = "sleeping" | "awoken";

/** One form of a unit, shown to players as When → Who → Does. */
export interface UnitForm {
  when: When[];
  condition?: Condition;
  who: Selector[];
  /** Ordered ability ids (AbilityRegistry keys). A base form has exactly one. */
  does: string[];
  /** Optional authored one-liner; clients fall back to describe(). */
  text?: string;
}

/** A unit in the live pool. The awoken form keeps the same When and upgrades
 * the Who and/or the Does (slice 2 enforces, slice 7 authors). */
export interface UnitContent {
  id: UnitId;
  name: string;
  emoji: string;
  tier: Tier;
  base: Stats;
  forms: Record<FormKey, UnitForm>;
}

/** Everything a run plays with, carried with a version so logs stay replayable. */
export interface MvpContent {
  version: string;
  units: UnitContent[];
  abilities: AbilityRegistry;
  statuses: StatusRegistry;
}

// ---------- the line: owned units, fused units ----------

/** Ordered fusion: the When of `first`, the Who of `second`, the Does of both
 * (first's, then second's), stats summed. Fused units are final. */
export interface FusionParts {
  first: UnitId;
  second: UnitId;
  /** Generated name for this ordered pair (slice 10); a portmanteau until then. */
  name: string;
  /** Player name credited with the first discovery, or null for bots/unknown. */
  discoveredBy: string | null;
}

/** What the server hands a fuse: the pair's name and who is credited.
 * Slice 10 looks it up (MvpDeps.nameFusion); the pure run only copies it. */
export interface FuseContext {
  name: string;
  discoveredBy: PlayerRef | null;
}

export interface LineUnit {
  /** Stable within a run; survives reorder, merge and fusion (the fused unit keeps first's uid). */
  uid: string;
  kind: "unit" | "fused";
  /** The unit id; for a fused unit, the first part's id. */
  unitId: UnitId;
  name: string;
  emoji: string;
  /** Copies merged in (1 = just bought). A fused unit counts copies of either part. */
  copies: number;
  form: FormKey;
  /** Current stats, every copy and fusion included. */
  stats: Stats;
  /** The recipe that fights: the current form, or the fused recipe. */
  recipe: UnitForm;
  fusion?: FusionParts;
}

export interface Offer {
  /** Index in the offers list; what `buy` takes. */
  slot: number;
  unitId: UnitId;
  tier: Tier;
  cost: number;
}

// ---------- decisions ----------

export type Decision =
  | { kind: "buy"; slot: number }
  | { kind: "sell"; index: number }
  | { kind: "reroll" }
  | { kind: "reorder"; from: number; to: number }
  /** Two Awoken units, in tap order (slice 2). */
  | { kind: "fuse"; first: number; second: number }
  /** Ends the shop phase: fight this round's opponent (or the Crown after the last round). */
  | { kind: "fight" };

export type DecisionKind = Decision["kind"];

// ---------- opponents, fights, battles ----------

export interface PlayerRef {
  id: string;
  name: string;
  bot: boolean;
}

/** A saved team from another run at the same round. */
export interface Ghost {
  ghostId: string;
  runId: string;
  player: PlayerRef;
  round: number;
  line: LineUnit[];
}

export type FightKind = "round" | "crown";
export type Outcome = "win" | "loss" | "draw";

export interface FightResult {
  battleId: string;
  kind: FightKind;
  round: number;
  opponent: Pick<Ghost, "ghostId" | "player" | "round">;
  outcome: Outcome;
  heartsLost: number;
  heartsAfter: number;
}

/** A unit as it entered a battle: enough for the viewer to draw cards. */
export interface BattleUnit {
  name: string;
  emoji: string;
  stats: Stats;
  form: FormKey;
  fused: boolean;
}

/** Full battle record. Side A is always the run's own line. Every event
 * carries `causedBy`, so tap-to-trace is a walk up `log` (slice 9). */
export interface BattleRecord {
  battleId: string;
  runId: string;
  seed: number;
  contentVersion: string;
  kind: FightKind;
  round: number;
  teamA: BattleUnit[];
  teamB: BattleUnit[];
  opponent: PlayerRef;
  winner: Side | "draw";
  log: BattleEvent[];
}

// ---------- run ----------

export type RunPhase = "shop" | "over";
export type RunEndReason = "out-of-hearts" | "crown-won" | "crown-lost" | "no-champion";

export interface RunView {
  runId: string;
  player: PlayerRef;
  contentVersion: string;
  phase: RunPhase;
  /** 1..rules.rounds; rules.rounds + 1 means the Crown fight is next. */
  round: number;
  hearts: number;
  gold: number;
  wins: number;
  losses: number;
  line: LineUnit[];
  offers: Offer[];
  /** Who the next fight is against, when known before fighting (may be null). */
  nextOpponent: Pick<Ghost, "player" | "round"> | null;
  fights: FightResult[];
  endedBy?: RunEndReason;
  /** Present once the run is over (slice 4 computes it; slice 1 leaves it null). */
  rating?: RatingChange | null;
}

export interface DecisionResponse {
  run: RunView;
  /** Present when the decision was a fight. */
  fight?: FightResult;
}

// ---------- day, champion, rating ----------

export interface Champion {
  /** The day this team holds the throne for (YYYY-MM-DD in the rules' zone). */
  day: string;
  player: PlayerRef;
  line: LineUnit[];
  since: string;
}

export interface DayView {
  day: string;
  /** ISO time of the next rollover. */
  endsAt: string;
  champion: Champion | null;
  /** Slayers' teams stay hidden until the day ends; only the count shows. */
  slayers: number;
  /** Last day's playoff, once one ran (slice 5). */
  lastPlayoff?: PlayoffResult | null;
}

export interface PlayoffResult {
  day: string;
  entrants: PlayerRef[];
  winner: PlayerRef | null;
  /** Round-robin battle ids, for the viewer. */
  battleIds: string[];
}

export interface Rating {
  player: PlayerRef;
  rating: number;
  runs: number;
  slays: number;
  daysAsChampion: number;
  playoffWins: number;
}

/** Rating moves once per run: wins plus a slay bonus vs. the expected result. */
export interface RatingChange {
  before: number;
  after: number;
  expected: number;
  actual: number;
}

export interface HomeView {
  rules: MvpRules;
  day: DayView;
  /** Null before the player's first run. */
  rating: Rating | null;
  activeRunId: string | null;
}

// ---------- HTTP API ----------
//
// Identity tonight is a name kept on the device: POST /players returns an id
// the client stores and sends as the X-Arena-Player header on every call.
// Invite tokens replace it in slice 13. Errors are { error: string } with 4xx.
//
//   GET  /api/v1/health                      → { ok: true, api, contentVersion }
//   GET  /api/v1/content                     → MvpContent
//   POST /api/v1/players       { name }      → PlayerRef
//   GET  /api/v1/home                        → HomeView
//   POST /api/v1/runs                        → RunView            (starts a run)
//   GET  /api/v1/runs/:runId                 → RunView
//   POST /api/v1/runs/:runId/decisions  Decision → DecisionResponse
//   GET  /api/v1/battles/:battleId           → BattleRecord
//   GET  /api/v1/day                         → DayView            (slice 5)
//   POST /api/v1/dev/end-day                 → DayView            (slice 5; dev only)
//   GET  /api/v1/stats                       → StatsView          (slice 11)

export const PLAYER_HEADER = "X-Arena-Player";

export interface StatsView {
  units: { unitId: UnitId; winRate: number; pickRate: number; runs: number }[];
  champions: Champion[];
  fusions: FusionParts[];
}

export interface ApiError {
  error: string;
}
