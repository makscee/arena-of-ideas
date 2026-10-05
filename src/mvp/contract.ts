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
  /** The pair's name, fixed by its first fuse (see FusionDiscovery). */
  name: string;
  /** Who is credited with discovering the pair (see FusionDiscovery); null
   * while only bots have made it. */
  discoveredBy: PlayerRef | null;
}

/** What the server hands a fuse: the pair's name and who is credited.
 * Slice 10's namer looks it up (RunDeps.nameFusion, and peekFusionName for a
 * preview); the pure run only copies it into the fused unit. */
export interface FuseContext {
  name: string;
  discoveredBy: PlayerRef | null;
}

/** A discovered ordered pair, stored once per (first, second); the same pair
 * always gets the same name. Slice 10 owns this storage (MvpStore.fusion,
 * putFusion, fusions), the naming and GET /fusions. It writes discoveries from
 * its RunHooks.onFuse (server/src/mvp/fusions.ts), never from decide() or a
 * preview. Slice 11's stats page and StatsView.fusions only read them.
 *
 * Name rule: the first fuse of a pair stores its name: the model's name when
 * slice 10 prepared one ahead (it prefetches every fusable pair on a human's
 * line), else the deterministic portmanteau. That name is copied into the
 * fused LineUnit, and so into runs, ghosts, champions and battles. The model's
 * name may replace the stored name only while no human has fused the pair
 * (discoveredBy null, bot fusions only), and only once. Once a human fuses a
 * pair, its name is fixed for good.
 *
 * Credit rule: a bot's fusion stores the pair with discoveredBy null; the
 * first human to fuse a pair whose discoveredBy is null claims it (the name
 * stays). */
export interface FusionDiscovery {
  first: UnitId;
  second: UnitId;
  name: string;
  discoveredBy: PlayerRef | null;
  /** ISO time the pair was first fused. */
  discoveredAt: string;
  /** The local model's name, or the deterministic portmanteau when it was down. */
  nameSource: "model" | "fallback";
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
  /** The content its line was built with; matchmaking serves only the current one. */
  contentVersion: string;
  /** ISO time it was saved. */
  createdAt: string;
}

/** round: a shop round's ghost. crown: the run's last fight, against today's
 * champion (slice 4). playoff: a day-end round-robin game between two
 * slayers' teams, outside any run (slice 5). */
export type FightKind = "round" | "crown" | "playoff";
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

/** A unit as it entered a battle: the whole line unit (unitId, copies, form,
 * recipe, fusion), so the viewer can name the ability that fired and open both
 * forms, plus `id`, its kernel instance id in the log (BattleStart's roster,
 * e.g. "A1:Brawler"). */
export type BattleUnit = LineUnit & { id: string };

/** Full battle record. Side A is the run's own line, owned by `player`; side
 * B is `opponent`'s. Every event carries `causedBy`, so tap-to-trace is a walk
 * up `log` (slice 9).
 *
 * A playoff game (kind "playoff", slice 5) belongs to no run: `runId` is null,
 * `round` is 0, side A is the pairing's first entrant (`player`) and side B
 * the second (`opponent`). */
export interface BattleRecord {
  battleId: string;
  /** The run that fought it; null for a playoff game. */
  runId: string | null;
  /** Side A's owner. */
  player: PlayerRef;
  seed: number;
  contentVersion: string;
  kind: FightKind;
  round: number;
  /** ISO time it was fought. */
  at: string;
  teamA: BattleUnit[];
  teamB: BattleUnit[];
  opponent: PlayerRef;
  winner: Side | "draw";
  log: BattleEvent[];
}

// ---------- run ----------

/** shop: buy, sell, reroll, reorder, fuse, then fight the round's ghost.
 * crown: after the last round with hearts left; offers [], gold 0, the only
 * decision is { kind: "fight" } and nextOpponent is today's champion. With no
 * live champion (none, or a stale contentVersion) the run ends right after
 * the last round instead ("no-champion").
 * over: nothing more to do; endedBy says why.
 *
 * Content changes (slice 7 retunes it): a run started on other content than
 * the live one can't fight, so the server ends it cleanly ("content-changed",
 * rating null, no rating change) on its next decision, or when its player
 * starts a run. Its line is not rebuilt. */
export type RunPhase = "shop" | "crown" | "over";
export type RunEndReason = "out-of-hearts" | "crown-won" | "crown-lost" | "no-champion" | "content-changed";

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
  /** Who the next fight is against: the server picks the round's ghost at
   * round start and the fight uses that ghost; in the crown phase it is the
   * champion. Null once the run is over. */
  nextOpponent: Pick<Ghost, "player" | "round"> | null;
  fights: FightResult[];
  /** The day (DayView.seq) the run started on. */
  day: number;
  /** ISO time the run started. */
  startedAt: string;
  /** ISO time the run ended; the server stamps it. */
  endedAt?: string;
  endedBy?: RunEndReason;
  /** Present once the run is over: a human's rating change (once per run),
   * null for a bot's run and a "content-changed" end. */
  rating?: RatingChange | null;
}

export interface DecisionResponse {
  run: RunView;
  /** Present when the decision was a fight. */
  fight?: FightResult;
}

// ---------- day, champion, rating ----------
//
// A day is numbered by `seq`: 1 is day 1, and every rollover adds 1, the dev
// "end day now" included, so several days can share one calendar date.
// Storage is keyed by seq; `day` (YYYY-MM-DD) is only a label. The current day
// is a DayState row (MvpStore.currentDay); everyone reads it through
// RunDeps.today(), which is server/src/mvp/day.ts today(), slice 5's.
//
// Who writes what (MvpStore):
// - Slice 5 owns putDay, putPlayoff and the rollover. Every rollover writes a
//   Champion row for the new seq: the playoff winner, or, on a day with no
//   slayers, the kept champion copied with the new seq.
// - Slice 6 seeds at startup: when currentChampion() is missing or stale
//   (below), it writes a strong bot team as the champion for today().seq.
// - Slice 4's Crown fights currentChampion() as it is at the fight (a
//   rollover between round 12 and the Crown switches to the new champion). A
//   win writes the Slay, with Slay.seq = that champion's seq; the run's end
//   writes its Rating.
// - Bots fight the Crown too, but slice 4 writes no Slay and no Rating for a
//   player.bot: slayers, playoffs and ratings are humans only.
//
// Stale content: a Champion or Slay whose contentVersion isn't the live
// content's may name abilities that no longer exist. Slice 6's seeder replaces
// a stale champion for today().seq at startup, slice 4's Crown treats a stale
// champion as none (endedBy "no-champion"), and slice 5's playoff skips slays
// of another contentVersion.

/** The current day as the server keeps it (MvpStore.currentDay). */
export interface DayState {
  seq: number;
  /** That day's date (YYYY-MM-DD), a label. */
  day: string;
  /** ISO time the day began: its rollover, or the first call for day 1. */
  startedAt: string;
  /** ISO time of the next rollover (slice 5 computes it from rules.dayEndsAt). */
  endsAt: string;
}

export interface Champion {
  /** The day this team holds the throne for. */
  seq: number;
  /** That day's date (YYYY-MM-DD in the rules' zone), a label. */
  day: string;
  player: PlayerRef;
  line: LineUnit[];
  since: string;
  /** The content its line was built with. */
  contentVersion: string;
}

/** A Crown fight won by a human (bots write none): the slayer's team that
 * day. Hidden until the day ends; slice 5 picks each slayer's strongest one
 * for the playoff, by simulation (fightLines). */
export interface Slay {
  /** The seq of the champion it beat. */
  seq: number;
  player: PlayerRef;
  runId: string;
  battleId: string;
  line: LineUnit[];
  /** The content its line was built with; the playoff skips other versions. */
  contentVersion: string;
  /** ISO time of the Crown fight. */
  at: string;
}

export interface DayView {
  seq: number;
  day: string;
  /** ISO time of the next rollover. */
  endsAt: string;
  champion: Champion | null;
  /** Slayers' teams stay hidden until the day ends; only the count shows. */
  slayers: number;
  /** The previous day's playoff, once one ran (MvpStore.playoff(seq - 1), slice 5). */
  lastPlayoff?: PlayoffResult | null;
}

export interface PlayoffResult {
  /** The day whose slayers played it. */
  seq: number;
  day: string;
  entrants: PlayerRef[];
  winner: PlayerRef | null;
  /** Round-robin battle ids, for the viewer. */
  battleIds: string[];
  /** The round-robin table, best first. */
  standings: { player: PlayerRef; wins: number; draws: number; losses: number }[];
  /** Every game, in play order; battleId opens it in the viewer. */
  games: { a: PlayerRef; b: PlayerRef; battleId: string; winner: Side | "draw" }[];
}

export interface Rating {
  player: PlayerRef;
  rating: number;
  runs: number;
  slays: number;
  daysAsChampion: number;
  playoffWins: number;
}

/** Rating moves once per run: round wins plus a slay bonus vs. the expected
 * result (src/mvp/run.ts ratingChange). actual = clamp(round fights won /
 * round fights fought, a draw counting DRAW_SCORE, + SLAY_BONUS if slayed,
 * 0, 1); expected = 1 / (1 + 10^((ratingStart - before)/400));
 * after = before + ratingK * (actual - expected), rounded. */
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
//   POST /api/v1/runs                        → RunView            (starts a run; the player's active run if one is going)
//   GET  /api/v1/runs/:runId                 → RunView
//   POST /api/v1/runs/:runId/decisions  Decision → DecisionResponse
//   POST /api/v1/runs/:runId/preview    Decision → DecisionResponse  (dry run, no writes; 400 for a fight)
//   GET  /api/v1/battles/:battleId           → BattleRecord
//   GET  /api/v1/fusions                     → FusionDiscovery[]  (slice 10)
//   GET  /api/v1/day                         → DayView            (slice 5)
//   POST /api/v1/dev/end-day                 → DayView            (slice 5; 404 unless MVP_DEV=1)
//   GET  /api/v1/stats                       → StatsView          (slice 11)

export const PLAYER_HEADER = "X-Arena-Player";

export interface StatsView {
  units: { unitId: UnitId; winRate: number; pickRate: number; runs: number }[];
  champions: Champion[];
  /** Read from slice 10's store (MvpStore.fusions); slice 11 never writes it. */
  fusions: FusionDiscovery[];
}

export interface ApiError {
  error: string;
}
