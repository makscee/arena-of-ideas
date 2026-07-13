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
import { assertValidContent, assertValidPool, ValidationError } from "./validate.js";
import {
  REROLL_COST,
  STACK_THRESHOLD,
  STARTING_GOLD,
  STARTING_LIVES,
  UNIT_COST,
  incomeForRound,
  shopSizeForRound,
  synthClimbTeam,
} from "./tunables.js";
import type { AbilityRegistry, Side, Stats, StatusRegistry, UnitDef } from "./types.js";

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
  /** Ability registry every unit's `ability` ref resolves through (PRD #081),
   * threaded alongside `statuses` into battles and the content gate. */
  abilities?: AbilityRegistry;
}

/** A unit on the run's line: the drafted def plus everything the shop grew on it. */
export type AwakeningPath = "trigger" | "selector";
export type BaseProgression = "Base" | "Awakened";

export interface FusionParent {
  name: string;
  def: UnitDef;
}

export interface FusionProgress {
  /** Ordered provenance. Parent 0 supplies the initial Trigger set and Ability 0. */
  parents: [FusionParent, FusionParent];
  /** Fresh copies bought after fusion; copies of either parent feed this meter. */
  meter: number;
  /** Permanent once chosen. Absence at meter 3 is the blocking choice state. */
  awakening?: AwakeningPath;
  /** Guards the snapshot double independently of presentation/state. */
  doubled: boolean;
}

/** A drafted combatant. Run progression is deliberately separate from the
 * battle DSL's optional content `level`: persisted run units never carry the
 * removed level/stacks/absorbed ladder. */
export interface RunUnit {
  name: string;
  /** Current PWR/HP after every literal duplicate increment and any one-time double. */
  base: Stats;
  kind: "base" | "composite";
  /** Total copies for a base Unit. A composite instead uses fusion.meter. */
  copies: number;
  progression: BaseProgression;
  /** Canonical battle recipe. A composite holds its ordered merged recipe here. */
  def: UnitDef;
  fusion?: FusionProgress;
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

export const RUN_PERSISTENCE_VERSION = 2;

export interface RunState {
  /** Explicit boundary: v1 level/stacks/absorbed payloads are never reinterpreted. */
  runVersion: typeof RUN_PERSISTENCE_VERSION;
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
  /** Held by reference from RunInput, never cloned — see initRun's caller contract. */
  abilities: AbilityRegistry;
}

// ---------- decisions & the run log ----------

export type RunDecision =
  | { kind: "buy"; offer: number }
  | { kind: "reroll" }
  | { kind: "reorder"; from: number; to: number }
  | { kind: "fuse"; primary: number; secondary: number }
  | { kind: "awakenFusion"; path: AwakeningPath }
  | { kind: "fight"; opponent: UnitDef[] }
  | { kind: "challengeBoss" };

export type RunEventBody =
  | { type: "RunStart"; seed: number; runId: string; gold: number; lives: number }
  | { type: "RoundStarted"; round: number; income: number; gold: number }
  | { type: "ShopRolled"; offers: string[] }
  | { type: "Bought"; offer: number; unit: string; cost: number; gold: number; copies: number; pwr: number; hp: number }
  | { type: "DuplicateGrown"; unit: string; pwrDelta: 1; hpDelta: 2; pwr: number; hp: number }
  | { type: "Awakened"; unit: string; copies: number }
  | { type: "Rerolled"; cost: number; gold: number }
  | { type: "Reordered"; from: number; to: number }
  | { type: "Fused"; first: string; second: string; name: string; abilities: [string, string]; pwr: number; hp: number }
  | { type: "FusionCopyAdded"; unit: string; parent: string; meter: number; pwrDelta: 1; hpDelta: 2; pwr: number; hp: number }
  | { type: "FusionAwakeningRequired"; unit: string; meter: 3 }
  | { type: "FusionAwakened"; unit: string; path: AwakeningPath; pwr: number; hp: number }
  | { type: "FightFought"; battleSeed: number; winner: Side | "draw"; turns: number; lives: number }
  // ladder events — the run's side of every store mutation and draw, so the
  // run log alone explains a ladder fight (the envelope's round = the pool).
  | { type: "Snapshotted"; seq: number }
  | { type: "OpponentDrawn"; opponent: string; seq: number; candidates: number }
  // Cold-start fallback (PRD #085): the floor's ghost pool held no live candidate,
  // so the climb opponent was SYNTHESIZED from the seed units (synthClimbTeam) off
  // the run's RNG. `floor` is the floor fought at; `seq` is where the run's own
  // ghost landed in that floor's pool (0 on a fresh floor). Mirrors OpponentDrawn
  // so the run log alone explains the fight; fires only when candidates is empty.
  | { type: "OpponentSynthesized"; floor: number; seq: number }
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
  // Founded (PRD #085 genesis) — the FIRST completed run on an EMPTY tower founds
  // the champion at floor 1, the bottom (the only free seat). `floor` is always 1,
  // never the challenged floor — founding is capped at the bottom. A crown-class
  // end (endedBy "crown"), but a distinct event so a genesis reads apart from a
  // #075 ascend-crown. Emitted only when challengeBoss lands on a vacant floor
  // AND no champion is seated anywhere; once a champion exists, a vacant challenge
  // is an Overshot instead.
  | { type: "Founded"; floor: number }
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
  const abilities = input.abilities ?? {};
  assertValidPool(input.pool, statuses, abilities, "pool");
  const s: RunState = {
    runVersion: RUN_PERSISTENCE_VERSION,
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
    abilities,
  };
  emit(s, { type: "RunStart", seed: input.seed, runId: s.runId, gold: s.gold, lives: s.lives });
  rollOffers(s);
  return s;
}

/** Buy an offer. First copy joins at 1/3. Every later copy immediately grants
 * +1 PWR/+2 HP. Copies of either ordered parent route into its composite. */
export function buy(state: RunState, offer: number): RunState {
  assertActionable(state, "buy");
  const def = state.offers[offer];
  if (def === undefined) throw new InvalidDecisionError("buy", `no offer at index ${offer} (shop has ${state.offers.length})`);
  if (state.gold < UNIT_COST) throw new InvalidDecisionError("buy", `${def.name} costs ${UNIT_COST} gold, have ${state.gold}`);

  const compositeTarget = state.team.findIndex((u) =>
    u.kind === "composite" && u.fusion!.parents.some((p) => p.name === def.name),
  );
  const baseTarget = state.team.findIndex((u) => u.kind === "base" && u.name === def.name);
  const target = compositeTarget >= 0 ? compositeTarget : baseTarget;
  if (target < 0 && state.team.length >= TEAM_SIZE) {
    throw new InvalidDecisionError("buy", `the line is full (${TEAM_SIZE}) and there is no ${def.name} to stack onto`);
  }

  const s = clone(state);
  s.offers.splice(offer, 1);
  s.gold -= UNIT_COST;
  if (target < 0) {
    const u: RunUnit = { name: def.name, base: { ...def.base }, kind: "base", copies: 1, progression: "Base", def };
    s.team.push(u);
    emit(s, { type: "Bought", offer, unit: def.name, cost: UNIT_COST, gold: s.gold, copies: 1, pwr: u.base.pwr, hp: u.base.hp });
    return s;
  }

  const u = s.team[target]!;
  u.base.pwr += 1;
  u.base.hp += 2;
  if (u.kind === "composite") {
    const fusion = u.fusion!;
    if (fusion.meter < STACK_THRESHOLD) fusion.meter += 1;
    emit(s, { type: "Bought", offer, unit: u.name, cost: UNIT_COST, gold: s.gold, copies: fusion.meter, pwr: u.base.pwr, hp: u.base.hp });
    emit(s, { type: "FusionCopyAdded", unit: u.name, parent: def.name, meter: fusion.meter, pwrDelta: 1, hpDelta: 2, pwr: u.base.pwr, hp: u.base.hp });
    if (fusion.meter === STACK_THRESHOLD && fusion.awakening === undefined) {
      emit(s, { type: "FusionAwakeningRequired", unit: u.name, meter: 3 });
    }
    return s;
  }

  u.copies += 1;
  emit(s, { type: "Bought", offer, unit: u.name, cost: UNIT_COST, gold: s.gold, copies: u.copies, pwr: u.base.pwr, hp: u.base.hp });
  emit(s, { type: "DuplicateGrown", unit: u.name, pwrDelta: 1, hpDelta: 2, pwr: u.base.pwr, hp: u.base.hp });
  if (u.copies === STACK_THRESHOLD) {
    u.progression = "Awakened";
    emit(s, { type: "Awakened", unit: u.name, copies: u.copies });
  }
  return s;
}

/** Refresh the shop for REROLL_COST gold — a fresh seeded draw, same round size. */
export function reroll(state: RunState): RunState {
  assertActionable(state, "reroll");
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
  assertActionable(state, "reorder");
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

/** Fuse an explicit ordered pair. Only Awakened, unfused bases qualify. */
export function fuse(state: RunState, primary: number, secondary: number): RunState {
  assertActionable(state, "fuse");
  for (const [label, i] of [["primary", primary], ["secondary", secondary]] as const) {
    if (!Number.isInteger(i) || i < 0 || i >= state.team.length) {
      throw new InvalidDecisionError("fuse", `${label} ${i} is outside the line (0..${state.team.length - 1})`);
    }
  }
  if (primary === secondary) throw new InvalidDecisionError("fuse", "a unit cannot fuse with itself — pick two distinct line positions");
  const first = state.team[primary]!;
  const second = state.team[secondary]!;
  if (first.kind !== "base" || second.kind !== "base") {
    throw new InvalidDecisionError("fuse", "a composite cannot fuse again — choose two unfused base Units");
  }
  if (first.progression !== "Awakened" || second.progression !== "Awakened") {
    throw new InvalidDecisionError("fuse", "both base Units must be Awakened (3 total copies) before fusion");
  }
  const firstAbilities = draftAbilityIds(first.def);
  const secondAbilities = draftAbilityIds(second.def);
  if (firstAbilities.length !== 1 || secondAbilities.length !== 1) {
    const invalid = firstAbilities.length !== 1 ? first : second;
    const count = invalid === first ? firstAbilities.length : secondAbilities.length;
    throw new InvalidDecisionError(
      "fuse",
      `${invalid.name} carries ${count} Abilities; each fusion parent must carry exactly one Ability. Fusion was refused without changing or dropping either parent's data`,
    );
  }
  const firstAbility = firstAbilities[0]!;
  const secondAbility = secondAbilities[0]!;
  if (firstAbility === secondAbility) {
    throw new InvalidDecisionError("fuse", `${first.name} and ${second.name} share Ability "${firstAbility}" — fusion needs different Abilities`);
  }

  const name = `${first.name} + ${second.name}`;
  const mergedStatuses = mergeIntrinsicStatuses(first.def.statuses, second.def.statuses);
  const compositeDef: UnitDef = {
    name,
    base: { hp: first.base.hp + second.base.hp, pwr: first.base.pwr + second.base.pwr },
    triggers: structuredClone(first.def.triggers ?? []),
    selectors: structuredClone(second.def.selectors ?? []),
    abilities: [firstAbility, secondAbility],
    ...(first.def.condition !== undefined ? { condition: structuredClone(first.def.condition) } : {}),
    ...(mergedStatuses.length > 0 ? { statuses: mergedStatuses } : {}),
  };
  const composite: RunUnit = {
    name,
    base: { ...compositeDef.base },
    kind: "composite",
    copies: 0,
    progression: "Base",
    def: compositeDef,
    fusion: {
      parents: [
        { name: first.name, def: structuredClone(first.def) },
        { name: second.name, def: structuredClone(second.def) },
      ],
      meter: 0,
      doubled: false,
    },
  };
  const s = clone(state);
  const low = Math.min(primary, secondary);
  const high = Math.max(primary, secondary);
  s.team.splice(high, 1);
  s.team.splice(low, 1, composite);
  emit(s, { type: "Fused", first: first.name, second: second.name, name, abilities: [firstAbility, secondAbility], pwr: composite.base.pwr, hp: composite.base.hp });
  return s;
}

/** Resolve the compulsory fusion Awakening. Existing axis entries stay first,
 * then the complete unused parent set is appended; current stats snapshot-double. */
export function awakenFusion(state: RunState, path: AwakeningPath): RunState {
  assertActive(state, "awakenFusion");
  const index = pendingFusionIndex(state);
  if (index < 0) throw new InvalidDecisionError("awakenFusion", "no fusion Awakening choice is pending");
  if (path !== "trigger" && path !== "selector") throw new InvalidDecisionError("awakenFusion", `unknown path "${String(path)}"`);
  const s = clone(state);
  const u = s.team[index]!;
  const fusion = u.fusion!;
  if (fusion.awakening !== undefined || fusion.doubled) throw new InvalidDecisionError("awakenFusion", `${u.name} already chose its permanent Awakening path`);
  const [first, second] = fusion.parents;
  if (path === "trigger") u.def.triggers = [...(u.def.triggers ?? []), ...structuredClone(second.def.triggers ?? [])];
  else u.def.selectors = [...(u.def.selectors ?? []), ...structuredClone(first.def.selectors ?? [])];
  fusion.awakening = path;
  fusion.doubled = true;
  u.progression = "Awakened";
  u.base.pwr *= 2;
  u.base.hp *= 2;
  emit(s, { type: "FusionAwakened", unit: u.name, path, pwr: u.base.pwr, hp: u.base.hp });
  return s;
}

/** Fight the given opponent (the run's team is side A — the attacker) and turn
 * the round: a loss costs a life (the last one ends the run), income lands on
 * the carryover, the shop rerolls free. Delegates to battle(); only the outcome
 * enters the run log — the full battle log is reproducible from the logged
 * battleSeed and the teams. */
export function fight(state: RunState, opponent: UnitDef[]): RunState {
  assertActionable(state, "fight");
  if (state.team.length === 0) {
    throw new InvalidDecisionError("fight", "the line is empty — buy a unit first");
  }
  assertValidContent(opponent, state.statuses, state.abilities, "opponent"); // the same gate every battle input passes
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
 * An empty draw means no live ghost stands at this floor — a cold-start, unplayed
 * tower (PRD #085). Rather than stall the climb, the opponent is SYNTHESIZED from
 * the run's seed pool (synthClimbTeam) off the run's own RNG stream, scaled to the
 * floor. Live ghosts always win: synthesis is the fallback, fired ONLY when
 * candidates is empty, never overriding a real ghost — and as runs play, their
 * snapshots accumulate in the pool and supersede it. The synthesized team passes
 * the same content gate every opponent passes, and (gate first, persist after) a
 * gate failure aborts before the run's ghost is snapshotted, so a retried climb
 * does not grow the pool with an aborted attempt.
 *
 * Depends only on the LadderStore interface — any backing serves. The store is
 * the run layer's one mutable boundary: it gains the ghost even though the
 * returned RunState is a fresh value as always. */
export function ladderFight(state: RunState, ladder: LadderStore): RunState {
  assertActionable(state, "fight");
  if (state.team.length === 0) {
    throw new InvalidDecisionError("fight", "the line is empty — buy a unit first");
  }
  const s = clone(state);
  // Draw and gate first (own ghosts are excluded from candidates, so the draw
  // is the same whether or not the ghost is in the pool yet); persist after.
  const pool = ladder.poolAt(s.round);
  const candidates = pool.filter((g) => g.runId !== s.runId);
  let opponent: UnitDef[];
  let drawEvent: RunEventBody;
  if (candidates.length === 0) {
    // Cold start: no live ghost here. Synthesize a floor-sized seed-unit team off
    // the run's RNG so the climb never stalls. (Live ghosts win — this branch is
    // reached only when there is no candidate to draw.)
    const made = synthClimbTeam(s.round, s.rng, s.pool);
    s.rng = made.rng;
    opponent = made.team;
    drawEvent = { type: "OpponentSynthesized", floor: s.round, seq: pool.length };
  } else {
    const draw = rngStep(s.rng);
    s.rng = draw.state;
    const pick = candidates[Math.floor(draw.value * candidates.length)]!;
    opponent = pick.team;
    drawEvent = { type: "OpponentDrawn", opponent: pick.runId, seq: pick.seq, candidates: candidates.length };
  }
  assertValidContent(opponent, s.statuses, s.abilities, "opponent"); // a synthesized or stored team passes the same gate as any opponent
  const ghost: TeamSnapshot = { runId: s.runId, round: s.round, seq: pool.length, team: toBattleTeam(s.team) };
  ladder.addSnapshot(ghost);
  emit(s, { type: "Snapshotted", seq: ghost.seq });
  emit(s, drawEvent);
  resolveFight(s, opponent);
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
 *  - A vacant floor (boss === null) splits on whether the tower is EMPTY (#085):
 *      FOUND (empty tower, ladder.champion() === null) — the first completed run
 *        FOUNDS the champion at floor 1, the bottom (the only free seat). The seat
 *        is always floor 1, never s.round (founding is capped at the bottom), end
 *        reason "crown" with a distinct Founded event. No fight, so no
 *        snapshot-before-fight ghost; the seat itself is left in floor 1's pool.
 *      OVERSHOOT (a champion exists) — the run climbed past the tower's top into a
 *        dead end: no boss to fight, nothing to claim. The run ends "overshoot"
 *        with NO crown and — because no fight happened — NO ghost snapshotted.
 *        (Reversing slice 2's "vacant floor auto-seats" edge: that edge let a run
 *        climb past every boss for a free crown — the trivial-crown degeneracy.
 *        Once any champion is seated, vacant-challenge overshoots, so the top is
 *        gated by a real fight against the reigning champion, not an empty slot.)
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
  assertActionable(state, "challengeBoss");
  if (state.team.length === 0) {
    throw new InvalidDecisionError("challengeBoss", "the line is empty — buy a unit first");
  }
  const s = clone(state);
  const boss = ladder.bossAt(s.round);
  if (boss === null) {
    emit(s, { type: "BossChallenged", floor: s.round, boss: null });
    if (ladder.champion() === null) {
      // Found-floor-1 genesis (#085): the tower is EMPTY — no champion seated
      // anywhere — so this vacant challenge FOUNDS the champion at floor 1, the
      // only free seat, the bottom. The seat is ALWAYS floor 1, never s.round:
      // founding is capped at the bottom, so a run that climbs an empty tower to
      // floor 7 and challenges still founds at floor 1, not 7. No fight happened,
      // so there is no snapshot-before-fight ghost; instead the seat itself is
      // left in floor 1's pool as a ghost (boss-in-its-own-pool, mirroring the
      // bootstrap/ascend pattern) so the demote-keeps-ghost invariant holds when
      // it is later dethroned. The seat carries round = 1, so the server shim's
      // `snap.round === floor` invariant holds. A founding IS the first crown — a
      // crown-class end — but a distinct Founded event records it so the run log
      // (and the UI) read a genesis apart from a #075 ascend-crown.
      const seat: TeamSnapshot = { runId: s.runId, round: 1, seq: ladder.poolAt(1).length, team: toBattleTeam(s.team) };
      ladder.addSnapshot(seat);
      ladder.setBoss(1, seat);
      emit(s, { type: "Founded", floor: 1 });
      endRun(s, "crown");
      return s;
    }
    // A champion is already seated: a vacant challenge is an OVERSHOOT, exactly as
    // #075 ships it — the run climbed past the tower's top into a dead end. No
    // fight, no seat, no crown, and (no fight) NO snapshot. The genesis above never
    // fires once any champion exists, so the #075 no-free-(high-)crown degeneracy
    // stays dead: a weak team can found the bottom seat ONCE, but can never ride
    // climb-losses to a free HIGH crown.
    emit(s, { type: "Overshot", floor: s.round });
    endRun(s, "overshoot");
    return s;
  }
  assertValidContent(boss.team, s.statuses, s.abilities, "boss"); // a seated boss passes the same gate as any opponent
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
    case "fuse":
      return fuse(state, d.primary, d.secondary);
    case "awakenFusion":
      return awakenFusion(state, d.path);
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
  if (state.runVersion !== RUN_PERSISTENCE_VERSION) {
    throw new Error(`cannot serialize run version ${String(state.runVersion)}; expected ${RUN_PERSISTENCE_VERSION}`);
  }
  return JSON.stringify(state);
}

/** Revive a serialized run. Structure is checked loudly (a corrupt store must
 * never become a silently wrong run) and the pool re-passes the content gate,
 * the initRun way — a revived run holds initRun's guarantees. */
const REVIVE_GUIDANCE = "Start a fresh run or use an explicit migration tool.";

function reviveError(reason: string): Error {
  return new Error(`${reason} ${REVIVE_GUIDANCE}`);
}

/** Revive only an intact v2 run. A stored payload is hostile input: every
 * progression object and embedded UnitDef is checked before it can reach a
 * transition. Old level/stacks/absorbed state is never guessed into v2. */
export function deserializeRun(raw: string): RunState {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (err) {
    throw reviveError(`stored run is not valid JSON: ${(err as Error).message}.`);
  }
  if (typeof parsed !== "object" || parsed === null) {
    throw reviveError("stored run is not a RunState — refusing to revive it.");
  }
  const s = parsed as Partial<RunState> & { team?: unknown[] };
  const legacyFields: string[] = [];
  if (Array.isArray(s.team)) {
    s.team.forEach((unit, i) => {
      if (typeof unit !== "object" || unit === null) return;
      for (const field of ["level", "stacks", "absorbed"] as const) {
        if (field in unit) legacyFields.push(`team[${i}].${field}`);
      }
    });
  }
  if (legacyFields.length > 0) {
    throw reviveError(
      `stored run contains legacy team progression field${legacyFields.length === 1 ? "" : "s"} ${legacyFields.join(", ")}. ` +
      "Legacy level/stacks/absorbed state cannot be reinterpreted as Awakening.",
    );
  }
  if (s.runVersion !== RUN_PERSISTENCE_VERSION) {
    throw reviveError(
      `stored run version is ${s.runVersion === undefined ? "missing/unversioned" : String(s.runVersion)}; expected ${RUN_PERSISTENCE_VERSION}.`,
    );
  }
  const intact =
    (s.status === "active" || s.status === "over") &&
    [s.seed, s.round, s.gold, s.lives, s.rng].every((n) => typeof n === "number" && Number.isFinite(n)) &&
    typeof s.runId === "string" && s.runId.length > 0 &&
    [s.team, s.offers, s.log, s.pool].every(Array.isArray) &&
    typeof s.statuses === "object" && s.statuses !== null && !Array.isArray(s.statuses) &&
    typeof s.abilities === "object" && s.abilities !== null && !Array.isArray(s.abilities);
  if (!intact) throw reviveError("stored run is not a structurally intact version-2 RunState — refusing to revive it.");

  const statuses = s.statuses as StatusRegistry;
  const abilities = s.abilities as AbilityRegistry;
  try {
    assertValidPool(s.pool, statuses, abilities, "pool");
    for (const [i, offer] of (s.offers as unknown[]).entries()) {
      assertValidContent([offer], statuses, abilities, `offers[${i}]`);
      if (draftAbilityIds(offer as UnitDef).length !== 1) {
        throw new Error(`offers[${i}] must be a singleton-Ability draft Unit`);
      }
    }
    for (const [i, value] of (s.team as unknown[]).entries()) {
      const problem = runUnitProblem(value);
      if (problem !== undefined) throw new Error(`team[${i}] ${problem}`);
      const unit = value as RunUnit;
      assertValidContent([unit.def], statuses, abilities, `team[${i}].def`);
      for (const [j, parent] of (unit.fusion?.parents ?? []).entries()) {
        assertValidContent([parent.def], statuses, abilities, `team[${i}].fusion.parents[${j}].def`);
      }
    }
  } catch (err) {
    if (err instanceof ValidationError) {
      throw new ValidationError([
        ...err.issues,
        { path: "stored run", message: `refusing to revive invalid v2 content. ${REVIVE_GUIDANCE}` },
      ]);
    }
    throw reviveError(`stored run v2 content/progression is invalid: ${(err as Error).message}.`);
  }
  return s as RunState;
}

/** The line as battle() input: the drafted def with the grown base and level on it.
 * This projection is the run layer's whole interface to the battle kernel. */
export function toBattleTeam(team: readonly RunUnit[]): UnitDef[] {
  return team.map((u) => ({ ...u.def, name: u.name, base: { ...u.base } }));
}

// ---------- internals ----------

/** Every transition's first check: an over run accepts no further decisions. */
function assertActive(state: RunState, kind: RunDecision["kind"]): void {
  if (state.status === "over") throw new InvalidDecisionError(kind, `the run is over (${state.endedBy})`);
}

/** The third post-fusion copy blocks every other direct/store-taking action. */
function assertActionable(state: RunState, kind: Exclude<RunDecision["kind"], "awakenFusion">): void {
  assertActive(state, kind);
  const pending = state.team[pendingFusionIndex(state)];
  if (pending !== undefined) {
    throw new InvalidDecisionError(kind, `${pending.name} must choose Trigger path or Selector path before any other action`);
  }
}

function pendingFusionIndex(state: RunState): number {
  return state.team.findIndex((u) => u.kind === "composite" && u.fusion!.meter >= STACK_THRESHOLD && u.fusion!.awakening === undefined);
}

/** Run one battle against `opponent` and record it: the battle seed comes off
 * the run's own stream (one seed drives the whole run), a loss costs a life.
 * Returns the winner; ending or turning the round is turnRound's job. */
function resolveFight(s: RunState, opponent: UnitDef[]): Side | "draw" {
  const draw = rngStep(s.rng);
  s.rng = draw.state;
  const battleSeed = Math.floor(draw.value * 4294967296);
  const log = battle({ teamA: toBattleTeam(s.team), teamB: opponent, seed: battleSeed, statuses: s.statuses, abilities: s.abilities });
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
  return { ...s, team: s.team.map((u) => structuredClone(u)), offers: [...s.offers], log: [...s.log] };
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

function primaryAbilityId(def: UnitDef): string {
  const id = def.abilities?.[0] ?? def.ability;
  if (!id) throw new Error(`unit "${def.name}" has no Ability`);
  return id;
}

/** The only legal draft/base shape: one canonical ref, or the accepted
 * grammar-v1 singular ref. Mixed or malformed representations return none so
 * fusion can refuse without choosing an Ability and silently dropping data. */
function draftAbilityIds(def: UnitDef): string[] {
  if (def.ability !== undefined) {
    return def.abilities === undefined && typeof def.ability === "string" && def.ability.length > 0 ? [def.ability] : [];
  }
  return Array.isArray(def.abilities) && def.abilities.every((id) => typeof id === "string" && id.length > 0)
    ? [...def.abilities]
    : [];
}

function mergeIntrinsicStatuses(a: UnitDef["statuses"], b: UnitDef["statuses"]): NonNullable<UnitDef["statuses"]> {
  const merged: NonNullable<UnitDef["statuses"]> = [];
  const byName = new Map<string, number>();
  for (const source of [a ?? [], b ?? []]) {
    for (const item of source) {
      const at = byName.get(item.status);
      if (at === undefined) {
        byName.set(item.status, merged.length);
        merged.push({ ...item });
      } else {
        merged[at]!.stacks += item.stacks;
      }
    }
  }
  return merged;
}

function sameStrings(actual: readonly string[] | undefined, expected: readonly string[]): boolean {
  return actual !== undefined && actual.length === expected.length && actual.every((value, i) => value === expected[i]);
}

/** Return the first structural/invariant failure without ever dereferencing an
 * unchecked parent. Semantic UnitDef validation follows in deserializeRun. */
function runUnitProblem(value: unknown): string | undefined {
  if (typeof value !== "object" || value === null || Array.isArray(value)) return "must be an object";
  const u = value as Partial<RunUnit> & Record<string, unknown>;
  if (typeof u.name !== "string" || u.name.length === 0) return "needs a non-empty name";
  if (typeof u.base !== "object" || u.base === null || Array.isArray(u.base)) return "needs current PWR/HP stats";
  if (typeof u.def !== "object" || u.def === null || Array.isArray(u.def)) return "needs a UnitDef object";
  const def = u.def as UnitDef;
  if (def.name !== u.name) return "name must match def.name";
  if (u.kind !== "base" && u.kind !== "composite") return "kind must be base or composite";
  if (u.progression !== "Base" && u.progression !== "Awakened") return "progression must be Base or Awakened";
  if (!Number.isInteger(u.copies) || (u.copies as number) < 0) return "copies must be a non-negative integer";
  const base = u.base as Partial<Stats>;
  if (![base.pwr, base.hp].every((n) => typeof n === "number" && Number.isInteger(n) && n >= 0)) {
    return "current PWR/HP must be non-negative integers";
  }

  if (u.kind === "base") {
    if (u.fusion !== undefined) return "a base Unit cannot carry FusionProgress";
    if ((u.copies as number) < 1) return "a base Unit must have at least one copy";
    const expected: BaseProgression = (u.copies as number) >= STACK_THRESHOLD ? "Awakened" : "Base";
    if (u.progression !== expected) return `progression ${u.progression} is inconsistent with ${String(u.copies)} copies`;
    if (draftAbilityIds(def).length !== 1) return "a base UnitDef must carry exactly one Ability";
    return undefined;
  }

  if (u.copies !== 0) return "a composite must keep copies 0";
  if (typeof u.fusion !== "object" || u.fusion === null || Array.isArray(u.fusion)) return "needs FusionProgress";
  const f = u.fusion as Partial<FusionProgress> & Record<string, unknown>;
  if (!Array.isArray(f.parents) || f.parents.length !== 2) return "fusion.parents must contain exactly two ordered parents";
  const parentAbilities: string[] = [];
  for (const [i, parentValue] of f.parents.entries()) {
    if (typeof parentValue !== "object" || parentValue === null || Array.isArray(parentValue)) return `fusion.parents[${i}] must be an object`;
    const parent = parentValue as Partial<FusionParent>;
    if (typeof parent.name !== "string" || parent.name.length === 0) return `fusion.parents[${i}] needs a non-empty name`;
    if (typeof parent.def !== "object" || parent.def === null || Array.isArray(parent.def)) return `fusion.parents[${i}] needs a UnitDef object`;
    if ((parent.def as UnitDef).name !== parent.name) return `fusion.parents[${i}] name must match def.name`;
    const ids = draftAbilityIds(parent.def as UnitDef);
    if (ids.length !== 1) return `fusion.parents[${i}] UnitDef must carry exactly one Ability`;
    parentAbilities.push(ids[0]!);
  }
  if (f.parents[0]!.name === f.parents[1]!.name) return "fusion parents must be two distinct ordered Units";
  if (parentAbilities[0] === parentAbilities[1]) return "fusion parents must carry different Abilities";
  if (!Number.isInteger(f.meter) || (f.meter as number) < 0 || (f.meter as number) > STACK_THRESHOLD) {
    return `fusion meter must be an integer from 0 to ${STACK_THRESHOLD}`;
  }
  if (f.awakening !== undefined && f.awakening !== "trigger" && f.awakening !== "selector") {
    return "fusion awakening path must be trigger or selector";
  }
  if (typeof f.doubled !== "boolean") return "fusion doubled guard must be boolean";
  if (f.awakening === undefined && f.doubled) return "fusion cannot be doubled before a path is chosen";
  if (f.awakening !== undefined && ((f.meter as number) !== STACK_THRESHOLD || !f.doubled)) {
    return `a chosen fusion path requires meter ${STACK_THRESHOLD} and doubled true`;
  }
  const expectedProgression: BaseProgression = f.awakening === undefined ? "Base" : "Awakened";
  if (u.progression !== expectedProgression) {
    return `composite progression must be ${expectedProgression} for its meter/path state`;
  }
  if (def.name !== `${f.parents[0]!.name} + ${f.parents[1]!.name}`) return "composite identity must preserve ordered parent names";
  if (!sameStrings(def.abilities, parentAbilities)) return "composite UnitDef must carry exactly both parent Abilities in order";
  return undefined;
}
