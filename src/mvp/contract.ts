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
  /** Gold back for selling a sleeping unit. */
  sellRefund: number;
  /** Gold back for selling an Awoken or fused unit. Runs stored before it
   * have no field and sell everything for `sellRefund`. */
  sellRefundAwoken?: number;
  /** Offers in a round-1 shop. */
  offers: number;
  /** +1 offer from each listed round on (3 → 6 with [2, 4, 7]). Runs stored
   * before the curve have no field and keep a fixed `offers`. */
  offersGrowAt?: number[];
  lineSize: number;
  /** Bench slots (round 3): units there don't fight, but copies still merge
   * into them and they fuse. Board slots 0..lineSize−1 are the line and
   * lineSize..lineSize+benchSize−1 the bench. Runs stored before the bench
   * have no field: benchSizeOf gives 0, no bench. */
  benchSize?: number;
  /** The Nth copy of a unit swaps it to its awoken form (slice 2). */
  copiesToAwaken: number;
  /** Stats each extra copy adds. */
  copyGrowth: Stats;
  /** tierOpensAt[t-1] = the round tier t enters the shop. */
  tierOpensAt: number[];
  /** Cascade step cap; hitting it logs a visible "chain capped" event (slice 3).
   * 32 since round 3 (note 19); a run keeps the cap it started with. */
  chainStepCap: number;
  /** Per-fight Elo (docs/round2/rating.md): K falls with runs played. A
   * player with fewer than `runsBelow` runs (the first step that fits) uses
   * its `k`; past every step, `ratingK`. Steps rather than an Infinity bound,
   * so the rules survive JSON. */
  ratingKSteps: { runsBelow: number; k: number }[];
  ratingK: number;
  ratingStart: number;
  /** The rating stamped on bots' ghosts and runs: anchors the scale. */
  botRating: number;
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
  sellRefundAwoken: 2,
  offers: 3,
  offersGrowAt: [2, 4, 7],
  lineSize: 5,
  benchSize: 3,
  copiesToAwaken: 3,
  copyGrowth: { pwr: 1, hp: 2 },
  tierOpensAt: [1, 3, 6, 9],
  chainStepCap: 32,
  ratingKSteps: [{ runsBelow: 5, k: 32 }, { runsBelow: 15, k: 16 }],
  ratingK: 10,
  ratingStart: 1000,
  botRating: 1000,
  dayEndsAt: "04:00",
  dayTimeZone: "Europe/Moscow",
};

/** Offers in the shop at `round`: `offers`, +1 for each `offersGrowAt` round reached. */
export function offersAt(rules: MvpRules, round: number): number {
  return rules.offers + (rules.offersGrowAt ?? []).filter((r) => r <= round).length;
}

/** True when a reroll would redraw nothing: locked offers fill the whole shop.
 * Empty slots (offers bought this round) still refill, so a reroll is fine then. */
export function lockedFull(s: { offers: readonly Pick<Offer, "locked">[]; rules: MvpRules; round: number }): boolean {
  return s.offers.length >= offersAt(s.rules, s.round) && s.offers.every((o) => o.locked);
}

/** The run's bench slots: 0 for a run stored before the bench. */
export function benchSizeOf(rules: Pick<MvpRules, "benchSize">): number {
  return rules.benchSize ?? 0;
}

/** Where board slot `slot` is: the line (0..lineSize−1) or the bench
 * (lineSize..lineSize+benchSize−1), and its index in that zone; null past the
 * board. The slot may be empty (index ≥ the zone's length). */
export function boardSlot(rules: Pick<MvpRules, "lineSize" | "benchSize">, slot: number): { zone: "line" | "bench"; index: number } | null {
  if (slot < rules.lineSize) return { zone: "line", index: slot };
  if (slot < rules.lineSize + benchSizeOf(rules)) return { zone: "bench", index: slot - rules.lineSize };
  return null;
}

/** The unit at board slot `slot`, or undefined when the slot is empty or off the board. */
export function boardUnit(run: { line: readonly LineUnit[]; bench?: readonly LineUnit[] }, rules: Pick<MvpRules, "lineSize" | "benchSize">, slot: number): LineUnit | undefined {
  const at = boardSlot(rules, slot);
  if (!at) return undefined;
  return at.zone === "line" ? run.line[at.index] : run.bench?.[at.index];
}

/** Gold back for selling `unit`: Awoken and fused units (form "awoken") get
 * `sellRefundAwoken`, sleeping ones `sellRefund`. */
export function sellValue(rules: MvpRules, unit: Pick<LineUnit, "form">): number {
  return unit.form === "awoken" ? (rules.sellRefundAwoken ?? rules.sellRefund) : rules.sellRefund;
}

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

/** A summoned body (round 3, R3-5): what a Summon effect puts on the line.
 * `id` is its name, lower-cased; names are unique among summons (mvpPool
 * checks), so a battle's Summon event (which carries only the name) and a
 * summon effect both resolve to one body. `form` is null when the body only
 * strikes ("No ability: it fights with its PWR / HP"). */
export interface SummonContent {
  id: string;
  name: string;
  emoji: string;
  base: Stats;
  form: UnitForm | null;
}

/** Everything a run plays with, carried with a version so logs stay replayable. */
export interface MvpContent {
  version: string;
  units: UnitContent[];
  abilities: AbilityRegistry;
  statuses: StatusRegistry;
  /** The summoned bodies (R3-5); absent in content built before round 3. */
  summons?: SummonContent[];
}

// ---------- the line: owned units, fused units ----------

/** Ordered fusion: the When of `first`, the Who of `second`, the Does of both
 * (first's, then second's); PWR and HP are the stronger part's plus one copy's
 * growth. Fused units are final. */
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
 * preview); the pure run only copies it into the fused unit. In a preview, a
 * pair nobody has fused has name "" (named when fused), and a stored pair
 * shows its stored credit (null while only bots have made it). */
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
 * Name rule: a pair's name is fixed the first time anyone, bot or human,
 * fuses it, and never changes after that: the model's name when slice 10
 * prepared one ahead (it asks for both orders of every fusable pair on any
 * line, humans' lines first), else the deterministic portmanteau. That name is
 * copied into the fused LineUnit, and so into runs, ghosts, champions and
 * battles, which therefore always agree with the store. A human never waits:
 * with no name ready, the fuse gets the portmanteau. A bot fuses a pair only
 * once its name is ready, or once the model has finally failed on it: bots
 * run in the background, so a bot waits for the name (asked at once). The
 * day-1 champion's fusions are stored as bot discoveries, named the same way.
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
  /** Kept through rerolls and new rounds until bought or unlocked (R3-12).
   * Missing on offers stored before it reads as not locked. */
  locked?: boolean;
}

// ---------- decisions ----------

export type Decision =
  | { kind: "buy"; slot: number }
  /** `index`, `from`/`to`, `first`/`second` are board slots: 0..lineSize−1 the
   * line, then the bench (boardSlot). */
  | { kind: "sell"; index: number }
  | { kind: "reroll" }
  /** Toggles the offer's lock. Free; locked offers survive rerolls and rounds. */
  | { kind: "lock"; slot: number }
  /** Within one zone: move and shift. Across line and bench: swap with the
   * unit at `to`, or move into `to` when it is that zone's empty slot. */
  | { kind: "reorder"; from: number; to: number }
  /** Two Awoken units, in tap order (slice 2). The fused unit keeps first's
   * uid and stands in the front-most line slot of the two, or the lower bench
   * slot when both are on the bench, so swapping the order changes only the
   * recipe. */
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
  /** Its owner's rating at the start of that run (a bot's: rules.botRating;
   * the champion's: Champion.rating). A row saved before round 2 has none
   * and reads as rules.ratingStart. */
  rating: number;
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
  /** `rating` is what the fight is rated against (Ghost.rating). */
  opponent: Pick<Ghost, "ghostId" | "player" | "round" | "rating">;
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
export type RunEndReason = "out-of-hearts" | "crown-won" | "crown-lost" | "no-champion" | "content-changed" | "abandoned";

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
  /** Units that don't fight (rules.benchSize slots, board slots after the
   * line), packed. Not part of ghosts, the Slay or stats. Empty for runs
   * from before the bench. */
  bench: LineUnit[];
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
  /** A run its player gave up (POST /runs/:id/abandon, endedBy "abandoned"):
   * the fights it forfeited, each rated a loss against `opponentRating`, the
   * round's already-picked opponent. Every heart left in the shop (playing on
   * could lose them all); one at the Crown, the only fight left. */
  forfeit?: { fights: number; opponentRating: number };
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
// - Slice 4's Crown fights today's champion as it is at the fight
//   (server/src/mvp/day.ts todaysChampion(), championOf(today().seq): never a
//   row a failed day end stored early for tomorrow; a rollover between round
//   12 and the Crown switches to the new champion). A win writes the Slay,
//   with Slay.seq = that champion's seq; the run's end writes its Rating.
// - Bots fight the Crown too, and a bot's win is a Slay like a human's
//   (hidden until the day ends, counted on /day, its strongest team enters
//   the playoff, so a bot can be crowned): a player alone still sees a real
//   playoff (#587). Ratings are humans only: a bot's Rating row may carry its
//   records (slays, playoffWins, daysAsChampion), never a rating change.
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
  /** What the Crown is rated against: the slayer's run-start rating
   * (Slay.rating), a bot's rules.botRating. Absent on rows from before round 2:
   * rules.ratingStart. */
  rating?: number;
}

/** A Crown fight won by a human or a bot: the slayer's team that day.
 * Hidden until the day ends; slice 5 picks each slayer's strongest one for
 * the playoff, by simulation (fightLines). */
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
  /** The slayer's rating at the start of that run; a crowned slay's champion
   * carries it (Champion.rating). */
  rating?: number;
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

/** Rating moves once per run, summed over its rated fights (every round
 * fight, the Crown, and an abandoned run's forfeits), Elo-style against each
 * opponent's stamped rating (src/mvp/run.ts ratingChange; docs/round2/rating.md):
 * E_i = 1 / (1 + 10^((opponent_i - before)/400)); S_i = 1 win, DRAW_SCORE
 * draw, 0 loss; after = round(before + k * Σ(S_i - E_i)). No slay bonus. */
export interface RatingChange {
  before: number;
  after: number;
  /** Σ E_i: the wins expected at `before`. */
  expected: number;
  /** Σ S_i: the wins got (a draw counts DRAW_SCORE). */
  actual: number;
  /** The K used: rules.ratingKSteps by the runs played before this one. */
  k: number;
}

export interface HomeView {
  rules: MvpRules;
  day: DayView;
  /** Null before the player's first run. */
  rating: Rating | null;
  activeRunId: string | null;
  /** True on a dev server (MVP_DEV=1): the title menu shows the dev tools
   * ("End day now"); every other player never sees them. */
  dev: boolean;
}

// ---------- HTTP API ----------
//
// Identity tonight is a name kept on the device: POST /players returns an id
// the client stores and sends as the X-Arena-Player header on every call.
// Invite tokens replace it in slice 13. Errors are { error: string } with 4xx:
// 400 a request the API can't read (a Decision of an unknown kind), 409 a
// decision the rules refuse, 404 any path not listed here.
//
//   GET  /api/v1/health                      → { ok: true, api, contentVersion, build }  (build: the deployed commit, or null)
//   GET  /api/v1/content                     → MvpContent
//   POST /api/v1/players       { name }      → PlayerRef
//   GET  /api/v1/home                        → HomeView
//   POST /api/v1/runs                        → RunView            (starts a run; the player's active run if one is going)
//   GET  /api/v1/runs/:runId                 → RunView
//   POST /api/v1/runs/:runId/decisions  Decision → DecisionResponse
//   POST /api/v1/runs/:runId/preview    Decision → DecisionResponse  (dry run, no writes; 400 for a fight)
//   POST /api/v1/runs/:runId/abandon             → RunView            (gives the run up: endedBy "abandoned"; 409 when over)
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
