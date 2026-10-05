// Arena MVP run, thin path (mission #574, slice 1): start → buy → fight a
// ghost → result. Pure: every transition returns a new state. Slice 4 grows
// this into the full run (tiers, sell/reorder rules, Crown, rating); slice 2
// adds awakening and fusion. The shapes come from ./contract.ts.

import { battle, winnerOf } from "../battle.js";
import { rngStep } from "../rng.js";
import type { UnitDef } from "../types.js";
import {
  MVP_RULES,
  type BattleRecord,
  type BattleUnit,
  type Decision,
  type FightResult,
  type Ghost,
  type LineUnit,
  type MvpContent,
  type MvpRules,
  type Offer,
  type Outcome,
  type PlayerRef,
  type RunView,
  type UnitContent,
} from "./contract.js";

export interface MvpRunState extends RunView {
  seed: number;
  /** rngStep state — the run's one seeded stream. */
  rng: number;
  nextUid: number;
  rules: MvpRules;
}

export class MvpDecisionError extends Error {
  constructor(readonly kind: Decision["kind"], reason: string) {
    super(`invalid ${kind}: ${reason}`);
    this.name = "MvpDecisionError";
  }
}

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

export function initMvpRun(args: { runId: string; player: PlayerRef; seed: number; content: MvpContent; rules?: MvpRules }): MvpRunState {
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
    fights: [],
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

function lineUnitOf(u: UnitContent, uid: string): LineUnit {
  return { uid, kind: "unit", unitId: u.id, name: u.name, emoji: u.emoji, copies: 1, form: "sleeping", stats: { ...u.base }, recipe: u.forms.sleeping };
}

/** A line unit as battle() input. */
export function toBattleDef(u: LineUnit): UnitDef {
  return {
    name: u.name,
    base: { ...u.stats },
    triggers: u.recipe.when,
    selectors: u.recipe.who,
    abilities: u.recipe.does,
    ...(u.recipe.condition ? { condition: u.recipe.condition } : {}),
  };
}

function battleUnit(u: LineUnit): BattleUnit {
  return { name: u.name, emoji: u.emoji, stats: { ...u.stats }, form: u.form, fused: u.kind === "fused" };
}

function clone(s: MvpRunState): MvpRunState {
  return structuredClone(s);
}

export interface FightContext {
  ghost: Ghost;
  battleId: string;
  battleSeed: number;
}

export interface MvpStep {
  state: MvpRunState;
  fight?: FightResult;
  battle?: BattleRecord;
}

/** Apply one decision. A fight needs its opponent and ids from the caller
 * (the server picks the ghost; the kernel stays pure). */
export function applyMvpDecision(state: MvpRunState, d: Decision, content: MvpContent, fightCtx?: FightContext): MvpStep {
  if (state.phase === "over") throw new MvpDecisionError(d.kind, `the run is over (${state.endedBy})`);
  const s = clone(state);
  switch (d.kind) {
    case "buy": {
      const offer = s.offers[d.slot];
      if (!offer) throw new MvpDecisionError("buy", `no offer in slot ${d.slot}`);
      if (s.gold < offer.cost) throw new MvpDecisionError("buy", `costs ${offer.cost}, have ${s.gold}`);
      const u = unitById(content, offer.unitId);
      const same = s.line.find((x) => x.kind === "unit" && x.unitId === u.id);
      if (same) {
        same.copies += 1;
        same.stats.pwr += s.rules.copyGrowth.pwr;
        same.stats.hp += s.rules.copyGrowth.hp;
      } else {
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
    case "fuse":
      throw new MvpDecisionError("fuse", "fusion arrives in slice 2");
    case "fight": {
      if (s.line.length === 0) throw new MvpDecisionError("fight", "the line is empty, buy a unit first");
      if (!fightCtx) throw new MvpDecisionError("fight", "no opponent");
      const { ghost, battleId, battleSeed } = fightCtx;
      const teamA = s.line.map(toBattleDef);
      const teamB = ghost.line.map(toBattleDef);
      const log = battle({ teamA, teamB, seed: battleSeed, abilities: content.abilities, statuses: content.statuses });
      const winner = winnerOf(log);
      const outcome: Outcome = winner === "A" ? "win" : winner === "B" ? "loss" : "draw";
      const heartsLost = outcome === "loss" ? 1 : 0;
      s.hearts -= heartsLost;
      if (outcome === "win") s.wins += 1;
      if (outcome === "loss") s.losses += 1;
      const fight: FightResult = {
        battleId,
        kind: "round",
        round: s.round,
        opponent: { ghostId: ghost.ghostId, player: ghost.player, round: ghost.round },
        outcome,
        heartsLost,
        heartsAfter: s.hearts,
      };
      s.fights.push(fight);
      const record: BattleRecord = {
        battleId,
        runId: s.runId,
        seed: battleSeed,
        contentVersion: s.contentVersion,
        kind: "round",
        round: s.round,
        teamA: s.line.map(battleUnit),
        teamB: ghost.line.map(battleUnit),
        opponent: ghost.player,
        winner,
        log,
      };
      if (s.hearts <= 0) {
        s.phase = "over";
        s.endedBy = "out-of-hearts";
        s.rating = null;
      } else if (s.round >= s.rules.rounds) {
        // Slice 4/5: the Crown fight against today's champion goes here.
        s.phase = "over";
        s.endedBy = "no-champion";
        s.rating = null;
      } else {
        s.round += 1;
        s.gold = s.rules.goldPerRound;
        rollOffers(s, content);
      }
      return { state: s, fight, battle: record };
    }
  }
}

/** The public view of a run (drops the rng and bookkeeping). */
export function runView(s: MvpRunState): RunView {
  const { seed: _seed, rng: _rng, nextUid: _n, rules: _r, ...view } = s;
  return view;
}

/** A bot ghost for a round with no saved teams: `size` seeded picks from the
 * units open at that round, copies merged. Slice 6's bots replace it. */
export function synthGhost(args: { content: MvpContent; round: number; seed: number; ghostId: string; rules?: MvpRules }): Ghost {
  const rules = args.rules ?? MVP_RULES;
  const s = initMvpRun({ runId: `bot-${args.ghostId}`, player: { id: "bot", name: "bot", bot: true }, seed: args.seed, content: args.content, rules });
  s.round = args.round;
  const size = Math.min(rules.lineSize, 1 + Math.floor(args.round / 2));
  const open = openUnits(args.content, rules, args.round);
  const copiesBudget = Math.floor(args.round / 3);
  for (let i = 0; i < size; i++) s.line.push(lineUnitOf(open[draw(s, open.length)]!, `g${i}`));
  for (let i = 0; i < copiesBudget; i++) {
    const u = s.line[draw(s, s.line.length)]!;
    u.copies += 1;
    u.stats.pwr += rules.copyGrowth.pwr;
    u.stats.hp += rules.copyGrowth.hp;
  }
  return { ghostId: args.ghostId, runId: s.runId, player: { id: "bot", name: botName(args.seed), bot: true }, round: args.round, line: s.line };
}

const BOT_NAMES = ["Ash", "Bram", "Cleo", "Dov", "Esk", "Fyn", "Gale", "Hux", "Ives", "Juno", "Kip", "Lux"];
function botName(seed: number): string {
  return `bot-${BOT_NAMES[(seed >>> 0) % BOT_NAMES.length]}`;
}
