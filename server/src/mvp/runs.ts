// Arena MVP run engine on the server (mission #574): the bookkeeping around
// the pure run (src/mvp/run.ts). Routes only parse and call these. Slice 6's
// bots play in-process through them, and slices 5, 10 and 11 observe runs
// through RunHooks instead of editing decide().
import { randomUUID } from "node:crypto";
import type { BattleRecord, DayState, Decision, DecisionResponse, FightResult, FuseContext, Ghost, LineUnit, MvpContent, MvpRules, PlayerRef, Rating } from "../../../src/mvp/contract.js";
import { fuseCheck } from "../../../src/mvp/forms.js";
import { applyMvpDecision, championGhost, endRun, initMvpRun, MvpDecisionError, ratingChange, runView, setOpponent, synthGhost, unitById, type DecisionContext, type MvpRunState, type MvpStep } from "../../../src/mvp/run.js";
import type { NameFusion } from "./fusions.js";
import type { MvpStore } from "./store.js";

/** Observers of the run engine (slice 10's fusion names, slice 11's stats,
 * whatever slice 5 needs). decide() calls them after its store writes,
 * synchronously: every onDecision first, then onFuse, onFight and onRunEnd as
 * they apply, each in registration order. A preview calls none. A hook must
 * not throw: the decision is already saved. */
export interface RunHooks {
  /** Any decision went through: the run before it and after it. Slice 10
   * prefetches fusion names here. */
  onDecision?(before: MvpRunState, d: Decision, after: MvpRunState): void;
  /** A fuse went through: `fused` is the new unit on `run`'s line, `ctx` the
   * name and credit it got. Slice 10 records the FusionDiscovery here. */
  onFuse?(run: MvpRunState, fused: LineUnit, ctx: FuseContext): void;
  /** A fight was fought; `run` is the state after it. */
  onFight?(run: MvpRunState, fight: FightResult, battle: BattleRecord): void;
  /** The decision ended the run. */
  onRunEnd?(run: MvpRunState): void;
}

/** What the run engine needs. MvpRuntime (./runtime.ts) is one, and
 * mvpRuntime() is the only place that fills it. */
export interface RunDeps {
  store: MvpStore;
  content: MvpContent;
  rules: MvpRules;
  /** Seed source; tests pin it. */
  seed(): number;
  now(): Date;
  /** The current day (./day.ts today(), slice 5's): a run starts on it and
   * the Crown and the Slay read it. */
  today(): DayState;
  hooks: RunHooks[];
  /** decide()'s namer for a fuse; peekFusionName unless a test overrides it. */
  nameFusion: NameFusion;
  /** The read-only namer (slice 10, ./fusions.ts): the stored name, else a
   * prefetched model name, else the portmanteau. It never records or waits.
   * preview() uses it, so a preview names a fusion as the fuse will. */
  peekFusionName: NameFusion;
}

/** Starts a run for `player` on today's day, with round 1's opponent picked,
 * and stores it. One active run per player: if one is going, that one comes
 * back instead (a run on content that is no longer live is ended first, and a
 * new one starts). Synchronous, so parallel starts can't create two. */
export function startRun(deps: RunDeps, player: PlayerRef): MvpRunState {
  const active = deps.store.activeRun(player.id);
  if (active) {
    if (active.contentVersion === deps.content.version) return active;
    const ended = finish(deps, endRun(active, "content-changed"));
    for (const h of deps.hooks) h.onRunEnd?.(ended);
  }
  const fresh = initMvpRun({ runId: randomUUID(), player, seed: deps.seed(), content: deps.content, day: deps.today().seq, startedAt: deps.now().toISOString(), rules: deps.rules });
  const run = setOpponent(fresh, pickGhost(deps, fresh));
  deps.store.putRun(run);
  return run;
}

/** Applies one decision to a stored run and records what it produced: the
 * snapshot ghost and the battle of a fight, the next opponent (the round's
 * ghost, or the champion for the Crown), the Slay, the rating at the run's
 * end, the new run state, then the hooks. Synchronous on purpose: read the
 * run and call this with no await in between, so two decisions on one run
 * can't start from the same state. Throws MvpDecisionError when the rules
 * refuse the decision. A run on content that is no longer live is ended
 * ("content-changed") instead of applying `d`. */
export function decide(deps: RunDeps, run: MvpRunState, d: Decision): DecisionResponse {
  const { store, content, seed, now } = deps;
  if (run.phase !== "over" && run.contentVersion !== content.version) {
    const ended = finish(deps, endRun(run, "content-changed"));
    for (const h of deps.hooks) h.onRunEnd?.(ended);
    return { run: runView(ended) };
  }
  // A rollover since round 12 ended crowned a new champion: fight that one.
  if (d.kind === "fight" && run.phase === "crown") run = currentCrown(deps, run);
  const ctx: DecisionContext = {};
  const fuse = fuseContext(deps.nameFusion, content, run, d);
  if (fuse) ctx.fuse = fuse;
  if (d.kind === "fight") {
    // The opponent picked at round start; a run stored without one picks now.
    const ghost = run.opponent ?? pickGhost(deps, run);
    ctx.fight = { ghost, battleId: randomUUID(), battleSeed: seed(), at: now().toISOString() };
  }
  const step = applyMvpDecision(run, d, content, ctx);
  // The line as it went into a round's fight becomes someone's ghost, win or
  // lose; only after the rules accepted the fight, so a refused one adds
  // nothing. An empty line (a walkover) is nobody's opponent.
  if (step.fight?.kind === "round" && run.line.length > 0) store.addGhost(ghostOf(run, now()));
  if (step.fight?.kind === "crown" && step.fight.outcome === "win" && !run.player.bot && run.crownSeq !== null) {
    store.addSlay({ seq: run.crownSeq, player: run.player, runId: run.runId, battleId: step.fight.battleId, line: structuredClone(run.line), contentVersion: run.contentVersion, at: ctx.fight!.at });
  }
  if (step.fight) step.state = nextOpponent(deps, step.state);
  if (step.state.phase === "over") step.state = finish(deps, step.state);
  else store.putRun(step.state);
  if (step.battle) store.putBattle(step.battle);
  notify(deps.hooks, run, d, step, ctx);
  return { run: runView(step.state), ...(step.fight ? { fight: step.fight } : {}) };
}

/** After a fight: the next round's ghost, or the champion for the Crown; a
 * Crown with no live champion ends the run ("no-champion"). */
function nextOpponent(deps: RunDeps, run: MvpRunState): MvpRunState {
  if (run.phase === "shop") return setOpponent(run, pickGhost(deps, run));
  if (run.phase !== "crown") return run;
  const champ = deps.store.currentChampion();
  if (!champ || champ.contentVersion !== deps.content.version) return endRun(run, "no-champion");
  return setOpponent(run, championGhost(champ, run.rules), champ.seq);
}

/** The Crown's opponent as of now: when the stored champion's seq is not
 * the one picked when round 12 ended (a rollover in between) and it is on the
 * live content, the run fights it instead, and a slay counts for its seq. */
function currentCrown(deps: RunDeps, run: MvpRunState): MvpRunState {
  const champ = deps.store.currentChampion();
  if (!champ || champ.seq === run.crownSeq || champ.contentVersion !== deps.content.version) return run;
  return setOpponent(run, championGhost(champ, run.rules), champ.seq);
}

/** How many of a round's newest saved teams a ghost pick draws from. Tunable. */
export const GHOST_PICK_POOL = 200;

/** A saved team at the run's round, never the player's own, built with the
 * live content; a seeded bot team when the round has none (slice 6's bots
 * fill the pool). */
function pickGhost(deps: RunDeps, run: MvpRunState): Ghost {
  const { store, content, now } = deps;
  const candidates = store.ghosts(run.round, { excludePlayerId: run.player.id, contentVersion: content.version, limit: GHOST_PICK_POOL });
  const pick = deps.seed();
  return candidates.length > 0
    ? candidates[pick % candidates.length]!
    : synthGhost({ content, round: run.round, seed: pick, ghostId: `bot-${randomUUID()}`, createdAt: now().toISOString(), rules: deps.rules });
}

/** Stamps a run that just ended, writes a human's rating (once per run) and
 * stores the run. Bots and a "content-changed" end move no rating. */
function finish(deps: RunDeps, run: MvpRunState): MvpRunState {
  const { store, rules } = deps;
  const r = structuredClone(run);
  r.endedAt = deps.now().toISOString();
  r.rating = null;
  if (!r.player.bot && r.endedBy !== "content-changed") {
    const prev: Rating = store.rating(r.player.id) ?? { player: r.player, rating: rules.ratingStart, runs: 0, slays: 0, daysAsChampion: 0, playoffWins: 0 };
    r.rating = ratingChange(prev.rating, r, rules);
    store.putRating({ ...prev, player: r.player, rating: r.rating.after, runs: prev.runs + 1, slays: prev.slays + (r.endedBy === "crown-won" ? 1 : 0) });
  }
  store.putRun(r);
  return r;
}

/** What `d` would do to `run`: the same rules on a copy, with no store
 * writes and no hooks (slice 8 shows the awakening and the fusion result card
 * with it). A fuse is named by peekFusionName: a preview never records a
 * discovery or calls a model. Not for fights: the route answers 400. */
export function preview(deps: RunDeps, run: MvpRunState, d: Decision): DecisionResponse {
  // decide() would end such a run instead of applying `d`; its line may name
  // units the live content no longer has.
  if (run.phase !== "over" && run.contentVersion !== deps.content.version) throw new MvpDecisionError(d.kind, "the run's content is no longer live: start a new run");
  const ctx: DecisionContext = {};
  const fuse = fuseContext(deps.peekFusionName, deps.content, run, d);
  if (fuse) ctx.fuse = fuse;
  return { run: runView(applyMvpDecision(run, d, deps.content, ctx).state) };
}

function notify(hooks: RunHooks[], before: MvpRunState, d: Decision, step: MvpStep, ctx: DecisionContext): void {
  const after = step.state;
  for (const h of hooks) h.onDecision?.(before, d, after);
  if (d.kind === "fuse" && ctx.fuse) {
    // The fused unit keeps first's uid.
    const uid = before.line[d.first]!.uid;
    const fused = after.line.find((u) => u.uid === uid)!;
    for (const h of hooks) h.onFuse?.(after, fused, ctx.fuse);
  }
  if (step.fight && step.battle) for (const h of hooks) h.onFight?.(after, step.fight, step.battle);
  if (after.phase === "over") for (const h of hooks) h.onRunEnd?.(after);
}

/** The name and credit for a fuse the rules allow; undefined otherwise (the
 * run then refuses the fuse with the reason). */
function fuseContext(nameFusion: NameFusion, content: MvpContent, run: MvpRunState, d: Decision): FuseContext | undefined {
  if (d.kind !== "fuse" || d.first === d.second) return undefined;
  const first = run.line[d.first];
  const second = run.line[d.second];
  if (!first || !second || fuseCheck(first, second) !== null) return undefined;
  return nameFusion(unitById(content, first.unitId), unitById(content, second.unitId), run.player);
}

function ghostOf(r: MvpRunState, at: Date): Ghost {
  return {
    ghostId: `${r.runId}-r${r.round}`,
    runId: r.runId,
    player: r.player,
    round: r.round,
    line: structuredClone(r.line),
    contentVersion: r.contentVersion,
    createdAt: at.toISOString(),
  };
}
