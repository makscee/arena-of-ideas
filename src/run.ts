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

/** The two ways a run ends: its last life lost, or the champion spot taken. */
export type RunEndReason = "out-of-lives" | "crown";

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
  | { kind: "fight"; opponent: UnitDef[] }; // the ladder picks opponents in slice 2

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
  | { type: "ChampionChallenged"; champion: string }
  | { type: "Crowned"; dethroned: string | null }
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

/** Fight on the ladder. Snapshot-before-fight: the fielded team enters the
 * round's pool as a ghost before any outcome is known, so even a run about to
 * die leaves an opponent behind. Then a seeded draw from that pool, own ghosts
 * excluded — deterministic given the run's RNG state and the pool contents.
 * An empty draw means the run has outrun every ghost at this round and
 * challenges the champion: win (or find the spot vacant) and the fielded team
 * takes the spot — the run ends crowned; a loss is a normal loss.
 *
 * Depends only on the LadderStore interface — any backing serves. The store is
 * the run layer's one mutable boundary: it gains the ghost (and possibly a new
 * champion) even though the returned RunState is a fresh value as always. */
export function ladderFight(state: RunState, ladder: LadderStore): RunState {
  assertActive(state, "fight");
  if (state.team.length === 0) {
    throw new InvalidDecisionError("fight", "the line is empty — buy a unit first");
  }
  const s = clone(state);
  const ghost: TeamSnapshot = { runId: s.runId, round: s.round, seq: ladder.poolAt(s.round).length, team: toBattleTeam(s.team) };
  ladder.addSnapshot(ghost);
  emit(s, { type: "Snapshotted", seq: ghost.seq });
  const candidates = ladder.poolAt(s.round).filter((g) => g.runId !== s.runId);
  if (candidates.length > 0) {
    const draw = rngStep(s.rng);
    s.rng = draw.state;
    const pick = candidates[Math.floor(draw.value * candidates.length)]!;
    emit(s, { type: "OpponentDrawn", opponent: pick.runId, seq: pick.seq, candidates: candidates.length });
    assertValidContent(pick.team, s.statuses, "opponent"); // a stored ghost passes the same gate as any opponent
    resolveFight(s, pick.team);
    turnRound(s);
    return s;
  }
  const champion = ladder.champion();
  if (champion !== null) {
    emit(s, { type: "ChampionChallenged", champion: champion.runId });
    assertValidContent(champion.team, s.statuses, "champion");
    const winner = resolveFight(s, champion.team);
    if (winner !== "A") {
      // A loss is a normal loss (a draw costs nothing) — the run carries on:
      // next round may hold ghosts again, or this challenge repeats.
      turnRound(s);
      return s;
    }
  }
  // Won the challenge, or the spot was vacant: the fielded team takes it. The
  // dethroned champion loses only the spot — its ghosts stay in their pools.
  ladder.setChampion(ghost);
  emit(s, { type: "Crowned", dethroned: champion === null ? null : champion.runId });
  endRun(s, "crown");
  return s;
}

// ---------- the decision sequence ----------

/** Apply one decision — the dispatch a stored decision sequence replays through. */
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
