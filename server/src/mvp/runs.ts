// Arena MVP run engine on the server (mission #574): the bookkeeping around
// the pure run (src/mvp/run.ts). Routes only parse and call these. Slice 6's
// bots play in-process through them, and slices 5, 10 and 11 observe runs
// through RunHooks instead of editing decide().
import { randomUUID } from "node:crypto";
import type { BattleRecord, DayState, Decision, DecisionResponse, FightResult, FuseContext, Ghost, LineUnit, MvpContent, MvpRules, PlayerRef } from "../../../src/mvp/contract.js";
import { fuseCheck } from "../../../src/mvp/forms.js";
import { applyMvpDecision, initMvpRun, runView, synthGhost, unitById, type DecisionContext, type MvpRunState, type MvpStep } from "../../../src/mvp/run.js";
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

/** Starts a run for `player` on today's day and stores it. */
export function startRun(deps: RunDeps, player: PlayerRef): MvpRunState {
  const run = initMvpRun({ runId: randomUUID(), player, seed: deps.seed(), content: deps.content, day: deps.today().seq, startedAt: deps.now().toISOString(), rules: deps.rules });
  deps.store.putRun(run);
  return run;
}

/** Applies one decision to a stored run and records what it produced: the
 * snapshot ghost and the battle of a fight, the new run state, then the hooks.
 * Synchronous on purpose: read the run and call this with no await in between,
 * so two decisions on one run can't start from the same state. Throws
 * MvpDecisionError when the rules refuse the decision. */
export function decide(deps: RunDeps, run: MvpRunState, d: Decision): DecisionResponse {
  const { store, content, seed, now } = deps;
  const ctx: DecisionContext = {};
  const fuse = fuseContext(deps.nameFusion, content, run, d);
  if (fuse) ctx.fuse = fuse;
  if (d.kind === "fight") {
    // Never your own team, only teams built with the content that fights now;
    // a seeded bot team when the round has none (slice 6's bots fill it).
    const candidates = store.ghosts(run.round, { excludePlayerId: run.player.id, contentVersion: content.version });
    const pick = seed();
    const ghost =
      candidates.length > 0
        ? candidates[pick % candidates.length]!
        : synthGhost({ content, round: run.round, seed: pick, ghostId: `bot-${randomUUID()}`, createdAt: now().toISOString(), rules: deps.rules });
    ctx.fight = { ghost, battleId: randomUUID(), battleSeed: seed(), at: now().toISOString() };
  }
  const step = applyMvpDecision(run, d, content, ctx);
  // The line as it went into the fight becomes someone's ghost, win or lose;
  // only after the rules accepted the fight, so a refused one adds nothing.
  if (d.kind === "fight" && run.line.length > 0) store.addGhost(ghostOf(run, now()));
  if (step.state.phase === "over") step.state.endedAt = now().toISOString();
  store.putRun(step.state);
  if (step.battle) store.putBattle(step.battle);
  notify(deps.hooks, run, d, step, ctx);
  return { run: runView(step.state), ...(step.fight ? { fight: step.fight } : {}) };
}

/** What `d` would do to `run`: the same rules on a copy, with no store
 * writes and no hooks (slice 8 shows the awakening and the fusion result card
 * with it). A fuse is named by peekFusionName: a preview never records a
 * discovery or calls a model. Not for fights: the route answers 400. */
export function preview(deps: RunDeps, run: MvpRunState, d: Decision): DecisionResponse {
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
