// Arena MVP run (mission #574, slices 1 and 4): 12 shop rounds, then the
// Crown fight against today's champion; 5 hearts, gold per round, shop tiers,
// buy/reroll/sell/reorder/fuse, and the run-end rating. Pure: every
// transition returns a new state, and the server (server/src/mvp/runs.ts)
// supplies what needs the store: each round's opponent (setOpponent, picked at
// round start), the champion for the Crown, the rating before the run. Copies,
// awakening and fusion are slice 2's, in ./forms.ts, and a fight is
// ./fight.ts's fightLines: this file only calls them. The shapes come from
// ./contract.ts.

import { rngStep } from "../rng.js";
import {
  MVP_RULES,
  type BattleRecord,
  type Champion,
  type Decision,
  type DecisionKind,
  type FightResult,
  type FuseContext,
  type Ghost,
  type LineUnit,
  type MvpContent,
  type MvpRules,
  type Offer,
  type Outcome,
  type PlayerRef,
  type RatingChange,
  type RunEndReason,
  type RunView,
  type UnitContent,
} from "./contract.js";
import { MvpBadDecision, MvpDecisionError } from "./errors.js";
import { fightLines } from "./fight.js";
import { addCopy, fuseCheck, fuseUnits, lineUnitOf, mergeTarget } from "./forms.js";

export interface MvpRunState extends RunView {
  /** The next fight's opponent, picked at round start (setOpponent): a saved
   * ghost in the shop, the champion as a ghost in the crown phase. The fight
   * uses this one. Null until the server picks it. */
  opponent: Ghost | null;
  /** The seq of the champion the Crown fights (Slay.seq); null before the crown phase. */
  crownSeq: number | null;
  seed: number;
  /** rngStep state — the run's one seeded stream. */
  rng: number;
  nextUid: number;
  rules: MvpRules;
}

export { MvpBadDecision, MvpDecisionError };

/** Every Decision kind (the compiler checks the list against the contract). */
const DECISION_KINDS: Record<DecisionKind, true> = { buy: true, sell: true, reroll: true, reorder: true, fuse: true, fight: true };
/** Moved to fight.ts; re-exported for scripts that import it from here (slice 7's meta report). */
export { toBattleDef } from "./fight.js";

function draw(s: MvpRunState, n: number): number {
  const { value, state } = rngStep(s.rng);
  s.rng = state;
  return Math.floor(value * n);
}

function openUnits(content: MvpContent, rules: MvpRules, round: number): UnitContent[] {
  const open = content.units.filter((u) => (rules.tierOpensAt[u.tier - 1] ?? Infinity) <= round);
  return open.length > 0 ? open : content.units;
}

function rollOffers(s: MvpRunState, content: MvpContent): void {
  const open = openUnits(content, s.rules, s.round);
  s.offers = Array.from({ length: s.rules.offers }, (_, slot): Offer => {
    const u = open[draw(s, open.length)]!;
    return { slot, unitId: u.id, tier: u.tier, cost: s.rules.unitCost };
  });
}

/** A new run. The kernel has no clock: the caller passes the day (DayView.seq)
 * and the start time. */
export function initMvpRun(args: { runId: string; player: PlayerRef; seed: number; content: MvpContent; day: number; startedAt: string; rules?: MvpRules }): MvpRunState {
  const rules = args.rules ?? MVP_RULES;
  const s: MvpRunState = {
    runId: args.runId,
    player: args.player,
    contentVersion: args.content.version,
    phase: "shop",
    round: 1,
    hearts: rules.hearts,
    gold: rules.goldPerRound,
    wins: 0,
    losses: 0,
    line: [],
    offers: [],
    nextOpponent: null,
    opponent: null,
    crownSeq: null,
    fights: [],
    day: args.day,
    startedAt: args.startedAt,
    seed: args.seed >>> 0,
    rng: args.seed >>> 0,
    nextUid: 1,
    rules,
  };
  rollOffers(s, args.content);
  return s;
}

export function unitById(content: MvpContent, id: string): UnitContent {
  const u = content.units.find((x) => x.id === id);
  if (!u) throw new Error(`unknown unit ${id}`);
  return u;
}

function clone(s: MvpRunState): MvpRunState {
  return structuredClone(s);
}

export interface FightContext {
  ghost: Ghost;
  battleId: string;
  battleSeed: number;
  /** ISO time of the fight (BattleRecord.at). */
  at: string;
}

/** What the server supplies for decisions the pure run can't decide alone. */
export interface DecisionContext {
  /** Required for a fight: the opponent and the ids. */
  fight?: FightContext;
  /** Required for a fuse: the pair's name and credit (slice 10). */
  fuse?: FuseContext;
}

export interface MvpStep {
  state: MvpRunState;
  fight?: FightResult;
  battle?: BattleRecord;
}

/** Apply one decision. A fight needs its opponent and ids, a fuse its name,
 * from the caller (the server picks the ghost and names the pair; the kernel
 * stays pure). Throws MvpBadDecision for a kind it doesn't know (the API
 * answers 400) and MvpDecisionError when the rules refuse it (409). */
export function applyMvpDecision(state: MvpRunState, d: Decision, content: MvpContent, ctx?: DecisionContext): MvpStep {
  if (!Object.hasOwn(DECISION_KINDS, d.kind)) throw new MvpBadDecision(d.kind);
  if (state.phase === "over") throw new MvpDecisionError(d.kind, `the run is over (${state.endedBy})`);
  if (state.phase === "crown" && d.kind !== "fight") throw new MvpDecisionError(d.kind, "only the Crown fight is left");
  const s = clone(state);
  switch (d.kind) {
    case "buy": {
      const offer = s.offers[d.slot];
      if (!offer) throw new MvpDecisionError("buy", `no offer in slot ${d.slot}`);
      if (s.gold < offer.cost) throw new MvpDecisionError("buy", `costs ${offer.cost}, have ${s.gold}`);
      const u = unitById(content, offer.unitId);
      const at = mergeTarget(s.line, u.id);
      if (at >= 0) s.line[at] = addCopy(s.line[at]!, content, s.rules);
      else {
        if (s.line.length >= s.rules.lineSize) throw new MvpDecisionError("buy", "the line is full");
        s.line.push(lineUnitOf(u, `u${s.nextUid++}`));
      }
      s.gold -= offer.cost;
      s.offers = s.offers.filter((o) => o.slot !== d.slot).map((o, i) => ({ ...o, slot: i }));
      return { state: s };
    }
    case "reroll": {
      if (s.gold < s.rules.rerollCost) throw new MvpDecisionError("reroll", `costs ${s.rules.rerollCost}, have ${s.gold}`);
      s.gold -= s.rules.rerollCost;
      rollOffers(s, content);
      return { state: s };
    }
    case "sell": {
      if (!s.line[d.index]) throw new MvpDecisionError("sell", `no unit at ${d.index}`);
      s.line.splice(d.index, 1);
      s.gold += s.rules.sellRefund;
      return { state: s };
    }
    case "reorder": {
      const u = s.line[d.from];
      if (!u || d.to < 0 || d.to >= s.line.length) throw new MvpDecisionError("reorder", `bad move ${d.from}→${d.to}`);
      s.line.splice(d.from, 1);
      s.line.splice(d.to, 0, u);
      return { state: s };
    }
    case "fuse": {
      const first = s.line[d.first];
      const second = s.line[d.second];
      if (!first || !second || d.first === d.second) throw new MvpDecisionError("fuse", `no pair at ${d.first} and ${d.second}`);
      const why = fuseCheck(first, second);
      if (why) throw new MvpDecisionError("fuse", why);
      if (!ctx?.fuse) throw new MvpDecisionError("fuse", "no fusion name");
      // The fused unit takes first's slot (and uid); second leaves the line.
      s.line[d.first] = fuseUnits(first, second, ctx.fuse, content);
      s.line.splice(d.second, 1);
      return { state: s };
    }
    case "fight": {
      // An empty line still fights, and loses (fightLines' walkover): the run
      // always moves on, so a player who sold everything or went broke isn't stuck.
      if (!ctx?.fight) throw new MvpDecisionError("fight", "no opponent");
      const crown = s.phase === "crown";
      const kind = crown ? "crown" : "round";
      const { ghost, battleId, battleSeed, at } = ctx.fight;
      const record = fightLines(
        { player: s.player, line: s.line },
        { player: ghost.player, line: ghost.line },
        { battleId, seed: battleSeed, kind, round: s.round, runId: s.runId, at, content, rules: s.rules },
      );
      const winner = record.winner;
      const outcome: Outcome = winner === "A" ? "win" : winner === "B" ? "loss" : "draw";
      const heartsLost = outcome === "loss" ? 1 : 0;
      s.hearts -= heartsLost;
      if (outcome === "win") s.wins += 1;
      if (outcome === "loss") s.losses += 1;
      const fight: FightResult = {
        battleId,
        kind,
        round: s.round,
        opponent: { ghostId: ghost.ghostId, player: ghost.player, round: ghost.round },
        outcome,
        heartsLost,
        heartsAfter: s.hearts,
      };
      s.fights.push(fight);
      s.opponent = null;
      s.nextOpponent = null;
      if (crown) endIn(s, outcome === "win" ? "crown-won" : "crown-lost");
      else if (s.hearts <= 0) endIn(s, "out-of-hearts");
      else if (s.round >= s.rules.rounds) {
        // The Crown is next: no shop, the server sets the champion (setOpponent).
        s.phase = "crown";
        s.round = s.rules.rounds + 1;
        s.gold = 0;
        s.offers = [];
      } else {
        s.round += 1;
        s.gold = s.rules.goldPerRound;
        rollOffers(s, content);
      }
      return { state: s, fight, battle: record };
    }
    default:
      throw new MvpBadDecision((d as { kind: unknown }).kind);
  }
}

function endIn(s: MvpRunState, why: RunEndReason): void {
  s.phase = "over";
  s.endedBy = why;
  s.gold = 0;
  s.offers = [];
  s.opponent = null;
  s.nextOpponent = null;
}

/** Sets the next fight's opponent: a round's ghost in the shop, or in the
 * crown phase the champion as a ghost (championGhost) with its seq. */
export function setOpponent(state: MvpRunState, ghost: Ghost, crownSeq?: number): MvpRunState {
  if (state.phase === "over") throw new MvpDecisionError("fight", `the run is over (${state.endedBy})`);
  const s = clone(state);
  s.opponent = structuredClone(ghost);
  s.nextOpponent = { player: ghost.player, round: ghost.round };
  if (s.phase === "crown") s.crownSeq = crownSeq ?? null;
  return s;
}

/** Ends a run that can't go on: "no-champion" (the Crown has no live
 * champion) or "content-changed" (the live content isn't the run's). */
export function endRun(state: MvpRunState, why: "no-champion" | "content-changed"): MvpRunState {
  const s = clone(state);
  endIn(s, why);
  return s;
}

/** Today's champion as the Crown's opponent. */
export function championGhost(c: Champion, rules: MvpRules = MVP_RULES): Ghost {
  return {
    ghostId: `champion-${c.seq}`,
    runId: `champion-${c.seq}`,
    player: c.player,
    round: rules.rounds + 1,
    line: structuredClone(c.line),
    contentVersion: c.contentVersion,
    createdAt: c.since,
  };
}

/** What slaying the champion adds to the run's score (0..1). Tunable. */
export const SLAY_BONUS = 0.25;
/** What a drawn round fight counts as. Tunable. */
export const DRAW_SCORE = 0.5;

/** True when the run slew the champion: it won the Crown. Beating your own
 * champion team counts too (Maks, round 2): the reigning champion can slay
 * and be crowned again with another team. */
export function slewChampion(run: Pick<RunView, "endedBy">): boolean {
  return run.endedBy === "crown-won";
}

/** The run-end rating, Elo-style. The run's score is its share of round
 * fights won (draws count DRAW_SCORE; the Crown isn't a round fight), plus
 * SLAY_BONUS for a slay (slewChampion), clamped to 0..1; the expected score
 * is a player rated `before` against the field (rules.ratingStart). So
 * winning half the rounds holds a start rating, and a slay lifts it. */
export function ratingChange(before: number, run: Pick<RunView, "fights" | "endedBy">, rules: MvpRules = MVP_RULES): RatingChange {
  const rounds = run.fights.filter((f) => f.kind === "round");
  const points = rounds.reduce((n, f) => n + (f.outcome === "win" ? 1 : f.outcome === "draw" ? DRAW_SCORE : 0), 0);
  const share = rounds.length > 0 ? points / rounds.length : 0;
  const actual = Math.min(1, Math.max(0, share + (slewChampion(run) ? SLAY_BONUS : 0)));
  const expected = 1 / (1 + 10 ** ((rules.ratingStart - before) / 400));
  return { before, after: Math.round(before + rules.ratingK * (actual - expected)), expected, actual };
}

/** The public view of a run (drops the rng and bookkeeping). */
export function runView(s: MvpRunState): RunView {
  const { seed: _seed, rng: _rng, nextUid: _n, rules: _r, opponent: _o, crownSeq: _c, ...view } = s;
  return view;
}

/** A bot ghost for a round with no saved teams: `size` seeded picks from the
 * units open at that round, copies merged. Slice 6's bots replace it.
 * `createdAt` comes from the caller (the kernel has no clock). */
export function synthGhost(args: { content: MvpContent; round: number; seed: number; ghostId: string; createdAt: string; rules?: MvpRules }): Ghost {
  const rules = args.rules ?? MVP_RULES;
  const s = initMvpRun({ runId: `bot-${args.ghostId}`, player: { id: "bot", name: "bot", bot: true }, seed: args.seed, content: args.content, day: 0, startedAt: args.createdAt, rules });
  s.round = args.round;
  const size = Math.min(rules.lineSize, 1 + Math.floor(args.round / 2));
  const open = openUnits(args.content, rules, args.round);
  const copiesBudget = Math.floor(args.round / 3);
  for (let i = 0; i < size; i++) s.line.push(lineUnitOf(open[draw(s, open.length)]!, `g${i}`));
  for (let i = 0; i < copiesBudget; i++) {
    const k = draw(s, s.line.length);
    s.line[k] = addCopy(s.line[k]!, args.content, rules);
  }
  return {
    ghostId: args.ghostId,
    runId: s.runId,
    player: { id: "bot", name: botName(args.seed), bot: true },
    round: args.round,
    line: s.line,
    contentVersion: args.content.version,
    createdAt: args.createdAt,
  };
}

const BOT_NAMES = ["Ash", "Bram", "Cleo", "Dov", "Esk", "Fyn", "Gale", "Hux", "Ives", "Juno", "Kip", "Lux"];
function botName(seed: number): string {
  return `bot-${BOT_NAMES[(seed >>> 0) % BOT_NAMES.length]}`;
}
