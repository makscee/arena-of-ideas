// The run kernel — a pure sibling of battle.ts: a run is seed + decision
// sequence → RunState + run log. Every transition is a pure function of its
// inputs (the RNG stream is carried as plain state, never a closure), so the
// same seed and decisions reproduce the run byte-for-byte — the run log is
// the determinism artifact, exactly as the battle log is (SPEC §3).
//
// fight() takes its opponent as a parameter and delegates to battle();
// ladderFight() draws the opponent from a Ladder (ladder.ts) — pools of
// ghosts built out of played runs, a champion spot at the top — and owns the
// run-end states: out of lives, or crowned. It depends only on the LadderStore
// interface, never a backing. Invalid decisions are rejected loudly, the
// validate.ts manner — a silently ignored buy would desync the stored
// decision sequence from the state it claims to reproduce.

import { battle, TEAM_SIZE } from "./battle.js";
import type { LadderStore, TeamSnapshot } from "./ladder.js";
import { rngStep } from "./rng.js";
import { assertValidContent, assertValidPool } from "./validate.js";
import {
  LEVEL_HP_GROWTH,
  LEVEL_PWR_GROWTH,
  REROLL_COST,
  STACK_THRESHOLD,
  STARTING_GOLD,
  STARTING_LIVES,
  UNIT_COST,
  incomeForRound,
  shopSizeForRound,
} from "./tunables.js";
import type { Side, Stats, StatusRegistry, UnitDef } from "./types.js";

// ---------- input & state ----------

export interface RunInput {
  seed: number;
  /** The run's identity on the ladder — its ghosts carry it, and its own
   * ghosts are excluded from its opponent draws. Defaults to `run-<seed>`;
   * runs sharing a ladder must use distinct ids. */
  runId?: string;
  /** The draftable units offers are drawn from (seeded, with replacement).
   * Names must be unique — the shop stacks copies by name. */
  pool: UnitDef[];
  /** Registry the run's battles run with (and the gate fight() validates opponents against). */
  statuses?: StatusRegistry;
}

/** A unit on the run's line: the drafted def plus everything the shop grew on it. */
export interface RunUnit {
  name: string;
  /** Grown base — level growth bakes in here; the content def stays untouched. */
  base: Stats;
  level: number;
  /** Copies absorbed since the last level-up (1..STACK_THRESHOLD−1). */
  stacks: number;
  /** The def as drafted — abilities and initial statuses come from it. */
  def: UnitDef;
}

/** A run accepts decisions while "active"; an "over" run rejects them all. */
export type RunStatus = "active" | "over";

/** The ways a run ends. A climb death — the last life lost — is "out-of-lives".
 * A boss challenge is always terminal and splits on WHICH boss fell and the
 * outcome:
 *
 *  - "crown" — beat the CHAMPION (the boss of the highest occupied floor). The
 *    challenger ASCENDS: it seats one floor higher (f+1) as the new champion and
 *    the tower grows by a floor; the old champion stays seated at f. This is the
 *    only end that grows the tower, and the only one that crowns — a crown means
 *    you out-topped the reigning summit, not merely seized a seat.
 *  - "seated" — beat a LOWER boss (a floor below the champion). A cash-out: the
 *    challenger seats at f IN PLACE, demoting the old boss to a pool-ghost as a
 *    climb does; the tower does NOT grow and no crown is taken. Distinct from
 *    "crown" because seizing a mid-tower seat is targeting depth, not the summit.
 *  - "challenge-lost" — a challenge that does not win (loss or draw). No seat
 *    happened and the old boss still stands; terminal regardless of lives.
 *  - "overshoot" — challengeBoss on a VACANT floor (above the tower's top): no
 *    boss to fight, no seat, no crown. The guard that keeps a run from climbing
 *    past the champion into a free seat. */
export type RunEndReason = "out-of-lives" | "crown" | "seated" | "challenge-lost" | "overshoot";

export interface RunState {
  seed: number;
  runId: string;
  status: RunStatus;
  /** Why the run ended; present only once status is "over". */
  endedBy?: RunEndReason;
  round: number;
  gold: number;
  lives: number;
  /** The line — ordered, index 0 front, at most TEAM_SIZE units. */
  team: RunUnit[];
  /** Current shop offers; buying removes one, rerolling replaces them all. */
  offers: UnitDef[];
  /** mulberry32 state — the run's single seeded RNG stream, as plain data. */
  rng: number;
  /** Append-only decision/outcome log — the run's determinism artifact. */
  log: RunEvent[];
  /** Held by reference from RunInput, never cloned — see initRun's caller contract. */
  pool: UnitDef[];
  /** Held by reference from RunInput, never cloned — see initRun's caller contract. */
  statuses: StatusRegistry;
}

// ---------- decisions & the run log ----------

export type RunDecision =
  | { kind: "buy"; offer: number } // index into the current offers
  | { kind: "reroll" }
  | { kind: "reorder"; from: number; to: number } // line positions
  | { kind: "fight"; opponent: UnitDef[] } // an explicit opponent; ladderFight draws its own from the store
  | { kind: "challengeBoss" }; // a store-taking, terminal move — like ladderFight, invoked directly, not through applyDecision

export type RunEventBody =
  | { type: "RunStart"; seed: number; runId: string; gold: number; lives: number }
  | { type: "RoundStarted"; round: number; income: number; gold: number }
  | { type: "ShopRolled"; offers: string[] }
  | { type: "Bought"; offer: number; unit: string; cost: number; gold: number; stacks: number }
  | { type: "LeveledUp"; unit: string; level: number; hp: number; pwr: number }
  | { type: "Rerolled"; cost: number; gold: number }
  | { type: "Reordered"; from: number; to: number }
  | { type: "FightFought"; battleSeed: number; winner: Side | "draw"; turns: number; lives: number }
  // ladder events — the run's side of every store mutation and draw, so the
  // run log alone explains a ladder fight (the envelope's round = the pool).
  | { type: "Snapshotted"; seq: number }
  | { type: "OpponentDrawn"; opponent: string; seq: number; candidates: number }
  // boss challenge — the terminal move (challengeBoss). BossChallenged names
  // the floor fought at and the boss it faced.
  //
  // The two winning ends carry distinct events so the run log alone says what
  // the tower did:
  //   Crowned — beat the champion and ASCENDED. `floor` is the SEAT taken (f+1,
  //     one above the fight), `dethroned` the champion that was out-topped (which
  //     STAYS seated below — a crown adds a floor, it does not remove the old
  //     summit). The seat at `floor` carries round = `floor`, so the server shim's
  //     `snap.round === floor` invariant holds for the ascended champion.
  //   Seated — beat a lower boss and CASHED OUT. `floor` is the floor fought at
  //     (= the seat, replaced in place), `dethroned` the boss demoted to a
  //     pool-ghost. The tower height is unchanged; this is not a crown.
  // Overshot is the dual end: challengeBoss landed on a floor with NO boss — the
  // run climbed past the tower's top — so there is nothing to fight or seat. No
  // ghost is snapshotted (no fight happened); the run ends with no crown.
  | { type: "BossChallenged"; floor: number; boss: string | null }
  | { type: "Crowned"; floor: number; dethroned: string | null }
  | { type: "Seated"; floor: number; dethroned: string | null }
  | { type: "Overshot"; floor: number }
  | { type: "RunEnded"; reason: RunEndReason; lives: number };

export type RunEvent = { id: number; round: number } & RunEventBody;

export type RunEventType = RunEventBody["type"];

/** An impossible decision — rejected loudly, never silently ignored. */
export class InvalidDecisionError extends Error {
  readonly decision: RunDecision["kind"];
  readonly reason: string;
  constructor(decision: RunDecision["kind"], reason: string) {
    super(`invalid decision "${decision}": ${reason}`);
    this.name = "InvalidDecisionError";
    this.decision = decision;
    this.reason = reason;
  }
}

// ---------- transitions ----------

/** Start a run: the pool through the content gate (every unit valid, names
 * unique — the shop stacks copies by name), then starting gold and lives,
 * round 1's shop rolled from the seed.
 *
 * Caller contract: `pool` and `statuses` are held by reference in RunState —
 * never mutate them after initRun, or every derived state and the replay of
 * the stored decision sequence silently diverge from this run. */
export function initRun(input: RunInput): RunState {
  const statuses = input.statuses ?? {};
  assertValidPool(input.pool, statuses, "pool");
  const s: RunState = {
    seed: input.seed,
    runId: input.runId ?? `run-${input.seed}`,
    status: "active",
    round: 1,
    gold: STARTING_GOLD,
    lives: STARTING_LIVES,
    team: [],
    offers: [],
    rng: input.seed >>> 0,
    log: [],
    pool: input.pool,
    statuses,
  };
  emit(s, { type: "RunStart", seed: input.seed, runId: s.runId, gold: s.gold, lives: s.lives });
  rollOffers(s);
  return s;
}

/** Buy an offer: a new name joins the line; a copy of an owned unit stacks onto it,
 * and at STACK_THRESHOLD copies the stacks fuse into a level. */
export function buy(state: RunState, offer: number): RunState {
  assertActive(state, "buy");
  const def = state.offers[offer];
  if (def === undefined) {
    throw new InvalidDecisionError("buy", `no offer at index ${offer} (shop has ${state.offers.length})`);
  }
  if (state.gold < UNIT_COST) {
    throw new InvalidDecisionError("buy", `${def.name} costs ${UNIT_COST} gold, have ${state.gold}`);
  }
  const target = state.team.findIndex((u) => u.name === def.name);
  if (target < 0 && state.team.length >= TEAM_SIZE) {
    throw new InvalidDecisionError("buy", `the line is full (${TEAM_SIZE}) and there is no ${def.name} to stack onto`);
  }
  const s = clone(state);
  s.offers.splice(offer, 1);
  s.gold -= UNIT_COST;
  if (target < 0) {
    s.team.push({ name: def.name, base: { ...def.base }, level: def.level ?? 1, stacks: 1, def });
    emit(s, { type: "Bought", offer, unit: def.name, cost: UNIT_COST, gold: s.gold, stacks: 1 });
  } else {
    const u = s.team[target]!;
    u.stacks += 1;
    emit(s, { type: "Bought", offer, unit: u.name, cost: UNIT_COST, gold: s.gold, stacks: u.stacks });
    if (u.stacks >= STACK_THRESHOLD) {
      // The copies fuse: one level, stat growth baked into the unit's grown base.
      u.stacks = 1;
      u.level += 1;
      u.base.hp += LEVEL_HP_GROWTH;
      u.base.pwr += LEVEL_PWR_GROWTH;
      emit(s, { type: "LeveledUp", unit: u.name, level: u.level, hp: u.base.hp, pwr: u.base.pwr });
    }
  }
  return s;
}

/** Refresh the shop for REROLL_COST gold — a fresh seeded draw, same round size. */
export function reroll(state: RunState): RunState {
  assertActive(state, "reroll");
  if (state.gold < REROLL_COST) {
    throw new InvalidDecisionError("reroll", `a reroll costs ${REROLL_COST} gold, have ${state.gold}`);
  }
  const s = clone(state);
  s.gold -= REROLL_COST;
  emit(s, { type: "Rerolled", cost: REROLL_COST, gold: s.gold });
  rollOffers(s);
  return s;
}

/** Move a unit to a new line position (index 0 is the front in battle). */
export function reorder(state: RunState, from: number, to: number): RunState {
  assertActive(state, "reorder");
  for (const [label, i] of [["from", from], ["to", to]] as const) {
    if (!Number.isInteger(i) || i < 0 || i >= state.team.length) {
      throw new InvalidDecisionError("reorder", `${label} ${i} is outside the line (0..${state.team.length - 1})`);
    }
  }
  const s = clone(state);
  const [u] = s.team.splice(from, 1);
  s.team.splice(to, 0, u!);
  emit(s, { type: "Reordered", from, to });
  return s;
}

/** Fight the given opponent (the run's team is side A — the attacker) and turn
 * the round: a loss costs a life (the last one ends the run), income lands on
 * the carryover, the shop rerolls free. Delegates to battle(); only the outcome
 * enters the run log — the full battle log is reproducible from the logged
 * battleSeed and the teams. */
export function fight(state: RunState, opponent: UnitDef[]): RunState {
  assertActive(state, "fight");
  if (state.team.length === 0) {
    throw new InvalidDecisionError("fight", "the line is empty — buy a unit first");
  }
  assertValidContent(opponent, state.statuses, "opponent"); // the same gate every battle input passes
  const s = clone(state);
  resolveFight(s, opponent);
  turnRound(s);
  return s;
}

/** Fight on the ladder — a pure same-floor ghost climb. Snapshot-before-fight:
 * the fielded team enters the round's pool as a ghost before any outcome is
 * known, so even a run about to die leaves an opponent behind. The opponent is
 * a seeded draw from that pool, own ghosts excluded — deterministic given the
 * run's RNG state and the pool contents — and it passes the content gate
 * BEFORE the run's own ghost persists: a gate-failing opponent aborts the whole
 * fight, and a retried fight must not grow the pool with the aborted attempt's
 * ghost on every try.
 *
 * An empty draw means the run has outrun every ghost at this floor — but a
 * climb is not a boss challenge any more (that is challengeBoss(), the explicit
 * terminal move). So an empty draw is rejected loudly, and rejected BEFORE the
 * snapshot, so a rejected climb does not grow the pool (the same no-grow-on-
 * retry property the gate-failing draw above preserves). The caller's move is
 * then challengeBoss: the only legal play at a floor with no climb opponent.
 *
 * Depends only on the LadderStore interface — any backing serves. The store is
 * the run layer's one mutable boundary: it gains the ghost even though the
 * returned RunState is a fresh value as always. */
export function ladderFight(state: RunState, ladder: LadderStore): RunState {
  assertActive(state, "fight");
  if (state.team.length === 0) {
    throw new InvalidDecisionError("fight", "the line is empty — buy a unit first");
  }
  const s = clone(state);
  // Draw and gate first (own ghosts are excluded from candidates, so the draw
  // is the same whether or not the ghost is in the pool yet); persist after.
  const pool = ladder.poolAt(s.round);
  const candidates = pool.filter((g) => g.runId !== s.runId);
  if (candidates.length === 0) {
    // No climb opponent at this floor — reject BEFORE snapshotting, so the pool
    // does not grow on the rejected attempt. The boss is the move now.
    throw new InvalidDecisionError("fight", `no climb opponent at floor ${s.round} — challenge the boss instead`);
  }
  const draw = rngStep(s.rng);
  s.rng = draw.state;
  const pick = candidates[Math.floor(draw.value * candidates.length)]!;
  assertValidContent(pick.team, s.statuses, "opponent"); // a stored ghost passes the same gate as any opponent
  const ghost: TeamSnapshot = { runId: s.runId, round: s.round, seq: pool.length, team: toBattleTeam(s.team) };
  ladder.addSnapshot(ghost);
  emit(s, { type: "Snapshotted", seq: ghost.seq });
  emit(s, { type: "OpponentDrawn", opponent: pick.runId, seq: pick.seq, candidates: candidates.length });
  resolveFight(s, pick.team);
  turnRound(s);
  return s;
}

/** Challenge the current floor's boss — the run's explicit, terminal endgame.
 * The "current floor" is s.round; the floor's boss is ladder.bossAt(s.round).
 * The run always ends here, win or lose:
 *
 *  - Snapshot-before-fight, exactly as ladderFight does: the fielded team is
 *    ghosted into the floor-s.round pool before any outcome is known, so the
 *    challenger leaves an opponent behind — and on a win this same ghost is the
 *    team that takes the boss seat.
 *  - A vacant floor (boss === null) is an OVERSHOOT: the run climbed past the
 *    seeded tower's top, so there is no boss to fight and nothing to claim. The
 *    run ends "overshoot" with NO crown and — because no fight happened — NO
 *    ghost snapshotted. This is the dual of a crown: the tower is a fixed height
 *    (the bootstrap seeds floors 1..TOWER_HEIGHT and nothing above), and a floor
 *    above it is not a free seat but a dead end. (Reversing slice 2's "vacant
 *    floor auto-seats" edge: that edge let a run climb past every boss for a free
 *    crown — the trivial-crown degeneracy. With the overshoot rule the top is
 *    gated by a real fight at floor TOWER_HEIGHT, not by an empty slot above it.)
 *  - A boss present is gated (like any opponent) and fought off the run's
 *    stream (the same battle-seed draw as a climb, and only that draw — no
 *    climb draw happens, so the RNG order stays deterministic). Snapshot-before-
 *    fight, exactly as ladderFight does: the fielded team is ghosted into the
 *    floor pool before any outcome — so even a lost challenge leaves an opponent
 *    behind. A loss or draw does not seat and ends the run "challenge-lost"
 *    (terminal regardless of lives — a lost challenge never loops to a climb).
 *
 *    A WIN splits on whether the boss fought IS the champion — i.e. whether the
 *    challenged floor f is the highest occupied floor (no boss seated above it):
 *      ASCEND (champion case) — beating the reigning summit grows the tower. The
 *        challenger seats at f+1 as the NEW champion; the old champion STAYS
 *        seated at f (a crown adds a floor above, it does not demote the summit).
 *        The seat snapshot at f+1 carries round = f+1 (its seated floor, NOT f) so
 *        the server shim's `snap.round === floor` invariant holds; the same team
 *        is ALSO left in pool@f+1 as a ghost, mirroring the bootstrap's
 *        boss-in-its-own-pool pattern, so the demote-keeps-ghost invariant holds
 *        when this champion is itself later dethroned. End reason "crown".
 *      CASH OUT (lower-boss case) — beating a boss below the champion seizes a
 *        mid-tower seat without growing the tower. The challenger seats at f in
 *        place; the demoted boss drops to a pool-ghost (its ghosts stay in the
 *        pool), exactly as the climb-demote does. End reason "seated", NOT a crown.
 *    Either way the snapshot-before-fight ghost (round = f) is what the climb
 *    pool keeps; the SEAT is a distinct write (round = its seated floor).
 *
 * Like ladderFight, this is a store-taking transition (NOT part of
 * applyDecision): the store is its one mutable boundary — it gains the
 * challenger's ghost and, on a win, the new seat (and, on an ascend, the new
 * champion's pool-ghost) — while the returned RunState is a fresh value. */
export function challengeBoss(state: RunState, ladder: LadderStore): RunState {
  assertActive(state, "challengeBoss");
  if (state.team.length === 0) {
    throw new InvalidDecisionError("challengeBoss", "the line is empty — buy a unit first");
  }
  const s = clone(state);
  const boss = ladder.bossAt(s.round);
  if (boss === null) {
    // Overshoot: climbed past the seeded tower's top. No boss, no fight, no seat,
    // no crown — and crucially NO snapshot, because no fight happened (snapshot-
    // before-fight only makes sense when there is a fight to leave a ghost for).
    emit(s, { type: "BossChallenged", floor: s.round, boss: null });
    emit(s, { type: "Overshot", floor: s.round });
    endRun(s, "overshoot");
    return s;
  }
  assertValidContent(boss.team, s.statuses, "boss"); // a seated boss passes the same gate as any opponent
  // Snapshot-before-fight: the challenger's ghost enters the floor's pool
  // before any outcome, so even a lost challenge leaves an opponent behind —
  // and on a win this is exactly the ghost that takes the seat.
  const pool = ladder.poolAt(s.round);
  const ghost: TeamSnapshot = { runId: s.runId, round: s.round, seq: pool.length, team: toBattleTeam(s.team) };
  ladder.addSnapshot(ghost);
  emit(s, { type: "Snapshotted", seq: ghost.seq });
  emit(s, { type: "BossChallenged", floor: s.round, boss: boss.runId });
  const winner = resolveFight(s, boss.team);
  if (winner !== "A") {
    // Not a win (loss or draw): the challenger does not seat, the boss stands.
    // Terminal whether or not a life remains — a lost challenge does not loop
    // back to a climb.
    endRun(s, "challenge-lost");
    return s;
  }
  // Won the challenge. Whether this is a CROWN (ascend) or a cash-out SEAT turns
  // on whether the boss just beaten is the champion — i.e. whether the challenged
  // floor is the highest occupied floor. The champion is the boss of the highest
  // occupied floor (ladder.champion()), so the test is "no boss sits above f":
  // the challenged boss IS the champion exactly when champion().round === f.
  const champ = ladder.champion();
  const atChampionFloor = champ !== null && champ.round === s.round;
  if (atChampionFloor) {
    // Ascend: the tower grows. Seat the challenger ONE floor higher as the new
    // champion; the old champion stays seated at f (do not demote it). The seat
    // snapshot carries round = f+1 (its seated floor) — distinct from the
    // snapshot-before-fight ghost (round = f) already in pool@f — so the server
    // shim's `snap.round === floor` invariant holds for the ascended champion.
    const seatFloor = s.round + 1;
    const seat: TeamSnapshot = { runId: s.runId, round: seatFloor, seq: ladder.poolAt(seatFloor).length, team: toBattleTeam(s.team) };
    // Mirror the bootstrap's "boss also lives in its floor's pool": leave the new
    // champion's team in pool@f+1 as a ghost too, so the demote-keeps-ghost
    // invariant holds when IT is later dethroned. (Pool write before the seat, so
    // seat.seq pins the pool length the ghost just took.)
    ladder.addSnapshot(seat);
    ladder.setBoss(seatFloor, seat);
    emit(s, { type: "Crowned", floor: seatFloor, dethroned: boss.runId });
    endRun(s, "crown");
    return s;
  }
  // Cash out: a lower seat, the tower does not grow. The challenger seats at its
  // own floor (the round it was fielded at), demoting the old boss in the slot —
  // its ghosts stay in the pool, exactly as a climb-demote leaves them. The seat
  // is the snapshot-before-fight ghost itself (round = f), already in pool@f.
  ladder.setBoss(ghost.round, ghost);
  emit(s, { type: "Seated", floor: ghost.round, dethroned: boss.runId });
  endRun(s, "seated");
  return s;
}

// ---------- the decision sequence ----------

/** Apply one decision — the dispatch a stored decision sequence replays
 * through. The store-taking moves (ladderFight, challengeBoss) are invoked
 * directly with their ladder, never here: applyDecision is pure/store-free, so
 * a challengeBoss arriving here has no boss to fight and is rejected loudly —
 * the caller must route it through challengeBoss(state, ladder) instead. */
export function applyDecision(state: RunState, d: RunDecision): RunState {
  switch (d.kind) {
    case "buy":
      return buy(state, d.offer);
    case "reroll":
      return reroll(state);
    case "reorder":
      return reorder(state, d.from, d.to);
    case "fight":
      return fight(state, d.opponent);
    case "challengeBoss":
      throw new InvalidDecisionError("challengeBoss", "a boss challenge needs the ladder — call challengeBoss(state, ladder) directly, not applyDecision");
  }
}

/** A whole run at once: seed + decision sequence → final state (SPEC §3 ladder practice). */
export function playRun(input: RunInput, decisions: readonly RunDecision[]): RunState {
  return decisions.reduce(applyDecision, initRun(input));
}

/** The run log as JSONL — byte-comparable, like the battle log. */
export function runToJSONL(log: readonly RunEvent[]): string {
  return log.map((e) => JSON.stringify(e)).join("\n") + "\n";
}

/** RunState as JSON — the persistence shape for an abandoned run.
 *
 * Everything serializes BY VALUE, `pool` and `statuses` included: the DSL is
 * data (RunState holds no functions anywhere — the RNG stream is a plain
 * number), so JSON captures the whole state and a revived run is
 * self-contained. It continues against exactly the pool and registry it
 * started with, even if the shipped content drifts between sessions; the cost
 * is one copy of the pool per stored run, and pools are a handful of UnitDefs.
 *
 * Identity note: in a live state `offers[i]` and `team[i].def` alias pool
 * entries; a round-trip splits those aliases into equal-but-distinct objects.
 * No transition compares by identity (the shop stacks copies by *name*), so a
 * revived run continues byte-identically — pinned by test. */
export function serializeRun(state: RunState): string {
  return JSON.stringify(state);
}

/** Revive a serialized run. Structure is checked loudly (a corrupt store must
 * never become a silently wrong run) and the pool re-passes the content gate,
 * the initRun way — a revived run holds initRun's guarantees. */
export function deserializeRun(raw: string): RunState {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (err) {
    throw new Error(`stored run is not valid JSON: ${(err as Error).message}`);
  }
  const s = parsed as Partial<RunState>;
  const intact =
    typeof s === "object" &&
    s !== null &&
    (s.status === "active" || s.status === "over") &&
    [s.seed, s.round, s.gold, s.lives, s.rng].every((n) => typeof n === "number") &&
    typeof s.runId === "string" &&
    [s.team, s.offers, s.log, s.pool].every(Array.isArray) &&
    typeof s.statuses === "object" &&
    s.statuses !== null;
  if (!intact) throw new Error("stored run is not a RunState — refusing to revive it");
  assertValidPool(s.pool!, s.statuses!, "pool");
  return s as RunState;
}

/** The line as battle() input: the drafted def with the grown base and level on it.
 * This projection is the run layer's whole interface to the battle kernel. */
export function toBattleTeam(team: readonly RunUnit[]): UnitDef[] {
  return team.map((u) => ({ ...u.def, name: u.name, base: { ...u.base }, level: u.level }));
}

// ---------- internals ----------

/** Every transition's first check: an over run accepts no further decisions. */
function assertActive(state: RunState, kind: RunDecision["kind"]): void {
  if (state.status === "over") {
    throw new InvalidDecisionError(kind, `the run is over (${state.endedBy})`);
  }
}

/** Run one battle against `opponent` and record it: the battle seed comes off
 * the run's own stream (one seed drives the whole run), a loss costs a life.
 * Returns the winner; ending or turning the round is turnRound's job. */
function resolveFight(s: RunState, opponent: UnitDef[]): Side | "draw" {
  const draw = rngStep(s.rng);
  s.rng = draw.state;
  const battleSeed = Math.floor(draw.value * 4294967296);
  const log = battle({ teamA: toBattleTeam(s.team), teamB: opponent, seed: battleSeed, statuses: s.statuses });
  const end = log[log.length - 1]!;
  if (end.type !== "BattleEnd") throw new Error("battle log has no BattleEnd");
  if (end.winner === "B") s.lives -= 1; // a draw costs no life
  emit(s, { type: "FightFought", battleSeed, winner: end.winner, turns: end.turns, lives: s.lives });
  return end.winner;
}

/** After a fight: at 0 lives the run ends; otherwise the round turns —
 * income lands on the carryover and the shop rerolls free. */
function turnRound(s: RunState): void {
  if (s.lives <= 0) {
    endRun(s, "out-of-lives");
    return;
  }
  s.round += 1;
  const income = incomeForRound(s.round);
  s.gold += income;
  emit(s, { type: "RoundStarted", round: s.round, income, gold: s.gold });
  rollOffers(s);
}

/** End the run: the state goes "over" and every later decision throws. */
function endRun(s: RunState, reason: RunEndReason): void {
  s.status = "over";
  s.endedBy = reason;
  emit(s, { type: "RunEnded", reason, lives: s.lives });
}

/** Clone the layers a transition may touch, so it never mutates its input. */
function clone(s: RunState): RunState {
  return { ...s, team: s.team.map((u) => ({ ...u, base: { ...u.base } })), offers: [...s.offers], log: [...s.log] };
}

function emit(s: RunState, body: RunEventBody): void {
  s.log.push({ id: s.log.length, round: s.round, ...body });
}

/** Roll the round's offers: shopSizeForRound seeded draws from the pool, with replacement. */
function rollOffers(s: RunState): void {
  const size = shopSizeForRound(s.round);
  const offers: UnitDef[] = [];
  for (let i = 0; i < size; i++) {
    const draw = rngStep(s.rng);
    s.rng = draw.state;
    offers.push(s.pool[Math.floor(draw.value * s.pool.length)]!);
  }
  s.offers = offers;
  emit(s, { type: "ShopRolled", offers: offers.map((u) => u.name) });
}
