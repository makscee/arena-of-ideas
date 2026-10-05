// Arena MVP run engine on the server (mission #574): the bookkeeping around
// the pure run (src/mvp/run.ts). Routes only parse and call these. Slice 6's
// bots play in-process through them, and slices 5 and 11 observe fights and
// run ends through RunHooks instead of editing the routes.
import { randomUUID } from "node:crypto";
import type { BattleRecord, Decision, DecisionResponse, FightResult, FuseContext, Ghost, MvpContent, PlayerRef } from "../../../src/mvp/contract.js";
import { fuseCheck } from "../../../src/mvp/forms.js";
import { applyMvpDecision, initMvpRun, runView, synthGhost, unitById, type DecisionContext, type MvpRunState } from "../../../src/mvp/run.js";
import type { NameFusion } from "./fusions.js";
import type { MvpStore } from "./store.js";

/** Observers of the run engine. They run after the store writes, in order. */
export interface RunHooks {
  /** A fight was fought; `run` is the state after it. */
  onFight?(run: MvpRunState, fight: FightResult, battle: BattleRecord): void;
  /** The decision ended the run. */
  onRunEnd?(run: MvpRunState): void;
}

export interface RunDeps {
  store: MvpStore;
  content: MvpContent;
  /** Seed source; tests pin it. */
  seed(): number;
  now(): Date;
  hooks: RunHooks[];
  nameFusion: NameFusion;
}

/** Starts a run for `player` on day `day` (DayView.seq) and stores it. */
export function startRun(deps: RunDeps, player: PlayerRef, day: number): MvpRunState {
  const run = initMvpRun({ runId: randomUUID(), player, seed: deps.seed(), content: deps.content, day, startedAt: deps.now().toISOString() });
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
        : synthGhost({ content, round: run.round, seed: pick, ghostId: `bot-${randomUUID()}`, createdAt: now().toISOString() });
    // Snapshot before the fight, so even a losing line becomes someone's ghost.
    if (run.line.length > 0) store.addGhost(ghostOf(run, now()));
    ctx.fight = { ghost, battleId: randomUUID(), battleSeed: seed() };
  }
  const step = applyMvpDecision(run, d, content, ctx);
  if (step.state.phase === "over") step.state.endedAt = now().toISOString();
  store.putRun(step.state);
  if (step.battle) store.putBattle(step.battle);
  // Slice 10: a fuse that went through records its FusionDiscovery here.
  if (step.fight && step.battle) for (const h of deps.hooks) h.onFight?.(step.state, step.fight, step.battle);
  if (step.state.phase === "over") for (const h of deps.hooks) h.onRunEnd?.(step.state);
  return { run: runView(step.state), ...(step.fight ? { fight: step.fight } : {}) };
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
