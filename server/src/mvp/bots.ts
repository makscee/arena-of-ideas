// Bots and the world (mission #574, slice 6).
// - seedChampion: at start, when currentChampion() is missing or stale
//   (contract.ts, "day, champion, rating"), writes a strong bot team as the
//   champion for rt.today().seq. "Strong" is measured: bot runs on a scratch
//   store, then a round-robin of their round-12 lines through fightLines.
// - topUpGhosts: bot players (PlayerRef bot: true) play whole runs in-process
//   through startRun and decide (./runs.ts) on `rt`, so they share the store,
//   the day and the hooks with HTTP runs, until every round's pool holds
//   BOT_TARGET ghosts on the live content. Bots stop after round 12 (no Crown:
//   only players slay and change the champion), never claim fusion credit
//   (slice 10's namer gives a bot's pair discoveredBy null) and move no rating.
//   The pool is bounded: bots only fill a round below the target, so they add
//   at most BOT_TARGET ghosts per round and content version.
// - botWorld: the job (./jobs.ts) that seeds, tops up at start and every
//   BOT_TOPUP_MS, a few runs per tick so requests keep flowing.
import { randomUUID } from "node:crypto";
import type { Champion, Decision, LineUnit, MvpContent, MvpRules, PlayerRef, RunView } from "../../../src/mvp/contract.js";
import { fightLines } from "../../../src/mvp/fight.js";
import { fuseCheck, mergeTarget } from "../../../src/mvp/forms.js";
import type { MvpRunState } from "../../../src/mvp/run.js";
import { decide, startRun, type RunDeps } from "./runs.js";
import type { MvpJob, MvpRuntime } from "./runtime.js";
import { MemoryMvpStore } from "./store.js";

/** Ghosts per round (live content) the bots keep. Tunable. */
export const BOT_TARGET = 24;
/** How often the top-up looks at the pool. */
export const BOT_TOPUP_MS = 60_000;
/** Bot runs per event-loop turn while topping up. */
const RUNS_PER_TURN = 2;
/** Bot runs per top-up, so a round bots rarely reach can't spin forever. */
const MAX_RUNS_PER_TOPUP = 400;
/** The day-1 champion search: bot runs simulated, finishers kept, seeds per pairing. */
const CHAMPION_RUNS = 48;
const CHAMPION_FINALISTS = 16;
const CHAMPION_SEEDS = 2;

const BOT_NAMES = ["Ash", "Bram", "Cleo", "Dov", "Esk", "Fyn", "Gale", "Hux", "Ives", "Juno", "Kip", "Lux", "Mira", "Nox", "Orla", "Pike"];

/** A fresh bot identity for one run: a roster name, a unique id (so a bot's
 * run left before the Crown never blocks its next run). */
export function botPlayer(seed: number): PlayerRef {
  return { id: `bot-${randomUUID()}`, name: `bot-${BOT_NAMES[(seed >>> 0) % BOT_NAMES.length]}`, bot: true };
}

// ---------- the bot's play ----------

function score(u: LineUnit): number {
  return u.stats.pwr * 2 + u.stats.hp + (u.form === "awoken" ? 6 : 0) + (u.kind === "fused" ? 10 : 0);
}

/** The bot's next decision in the shop: fuse two Awoken units, buy copies
 * first (they awaken), fill the line with the highest tier, swap the weakest
 * single copy for a better tier, reroll once a round while gold allows, put
 * the toughest unit in front, then fight. `rerolled` counts this round's
 * rerolls (the caller resets it). Pure. */
export function botDecision(run: RunView, content: MvpContent, rules: MvpRules, rerolled: number): Decision {
  if (run.phase !== "shop") return { kind: "fight" };
  const line = run.line;
  // Fuse: the two strongest fusable units, the stronger first.
  const awoken = line.map((u, i) => ({ u, i })).filter(({ u }) => u.kind === "unit" && u.form === "awoken").sort((a, b) => score(b.u) - score(a.u));
  if (awoken.length >= 2 && fuseCheck(awoken[0]!.u, awoken[1]!.u) === null) return { kind: "fuse", first: awoken[0]!.i, second: awoken[1]!.i };

  const tierOf = (id: string) => content.units.find((u) => u.id === id)?.tier ?? 1;
  const affordable = run.offers.filter((o) => o.cost <= run.gold);
  const copy = affordable.find((o) => mergeTarget(line, o.unitId) >= 0);
  if (copy) return { kind: "buy", slot: copy.slot };
  const best = [...affordable].sort((a, b) => b.tier - a.tier)[0];
  if (best && line.length < rules.lineSize) return { kind: "buy", slot: best.slot };
  if (best && line.length >= rules.lineSize) {
    // Swap: sell the weakest single, sleeping copy for a higher tier.
    const weak = line.map((u, i) => ({ u, i })).filter(({ u }) => u.kind === "unit" && u.copies === 1).sort((a, b) => tierOf(a.u.unitId) - tierOf(b.u.unitId) || score(a.u) - score(b.u))[0];
    if (weak && tierOf(weak.u.unitId) < best.tier) return { kind: "sell", index: weak.i };
  }
  if (rerolled < 2 && run.gold >= rules.rerollCost + rules.unitCost) return { kind: "reroll" };
  // The toughest unit goes in front.
  if (line.length > 1) {
    let front = 0;
    for (let i = 1; i < line.length; i++) if (line[i]!.stats.hp > line[front]!.stats.hp) front = i;
    if (front !== 0) return { kind: "reorder", from: front, to: 0 };
  }
  return { kind: "fight" };
}

/** Plays one bot run through startRun and decide on `deps` until it is over
 * or reaches the Crown, which bots leave alone. Returns the run as it stopped. */
export function playBotRun(deps: RunDeps, player: PlayerRef = botPlayer(deps.seed())): MvpRunState {
  deps.store.addPlayer(player);
  let run = startRun(deps, player);
  let round = run.round;
  let rerolled = 0;
  for (let guard = 0; run.phase === "shop" && guard < 1000; guard++) {
    if (run.round !== round) {
      round = run.round;
      rerolled = 0;
    }
    const d = botDecision(run, deps.content, deps.rules, rerolled);
    if (d.kind === "reroll") rerolled++;
    decide(deps, run, d);
    run = deps.store.run(run.runId)!;
  }
  return run;
}

// ---------- the pool ----------

/** Rounds whose live-content pool holds fewer than `target` ghosts. */
export function thinRounds(rt: RunDeps, target = BOT_TARGET): number[] {
  const out: number[] = [];
  for (let r = 1; r <= rt.rules.rounds; r++) {
    if (rt.store.ghosts(r, { excludePlayerId: "", contentVersion: rt.content.version, limit: target }).length < target) out.push(r);
  }
  return out;
}

/** Plays bot runs until every round holds `target` ghosts or `maxRuns` ran.
 * Synchronous; botWorld spreads the same work over event-loop turns. */
export function topUpGhosts(rt: RunDeps, opts: { target?: number; maxRuns?: number } = {}): { runs: number; thin: number[] } {
  const target = opts.target ?? BOT_TARGET;
  const maxRuns = opts.maxRuns ?? MAX_RUNS_PER_TOPUP;
  let runs = 0;
  let thin = thinRounds(rt, target);
  while (thin.length > 0 && runs < maxRuns) {
    playBotRun(rt);
    runs++;
    thin = thinRounds(rt, target);
  }
  return { runs, thin };
}

// ---------- the day-1 champion ----------

/** A strong bot line, measured: CHAMPION_RUNS bot runs on a scratch store
 * (their own ghosts as opponents; no real writes, no hooks), then the
 * survivors of all 12 rounds are ranked by power and the
 * strongest lines play a round-robin, both sides, CHAMPION_SEEDS
 * seeds each; the most wins is the champion. */
export function strongBotLine(rt: RunDeps): { player: PlayerRef; line: LineUnit[] } {
  const scratch: RunDeps = { ...rt, store: new MemoryMvpStore(), hooks: [], nameFusion: rt.peekFusionName };
  const finishers: { player: PlayerRef; line: LineUnit[] }[] = [];
  for (let i = 0; i < CHAMPION_RUNS; i++) {
    const run = playBotRun(scratch);
    // Survived all rounds: the Crown is next, or (no champion yet) "no-champion".
    if ((run.phase === "crown" || run.endedBy === "no-champion") && run.line.length > 0) finishers.push({ player: run.player, line: run.line });
  }
  if (finishers.length === 0) throw new Error("no bot run reached the Crown; can't seed a champion");
  const power = (l: LineUnit[]) => l.reduce((s, u) => s + score(u), 0);
  const finalists = finishers.sort((a, b) => power(b.line) - power(a.line)).slice(0, CHAMPION_FINALISTS);
  const wins = finalists.map(() => 0);
  const at = rt.now().toISOString();
  for (let a = 0; a < finalists.length; a++) {
    for (let b = 0; b < finalists.length; b++) {
      if (a === b) continue;
      for (let s = 0; s < CHAMPION_SEEDS; s++) {
        const rec = fightLines(finalists[a]!, finalists[b]!, { battleId: "sim", seed: rt.seed(), kind: "playoff", round: 0, runId: null, at, content: rt.content, rules: rt.rules });
        if (rec.winner === "A") wins[a]!++;
        else if (rec.winner === "B") wins[b]!++;
      }
    }
  }
  let top = 0;
  for (let i = 1; i < wins.length; i++) if (wins[i]! > wins[top]!) top = i;
  return finalists[top]!;
}

/** Writes a strong bot team as the champion for today().seq when there is no
 * champion or it was built with other content. Returns the one written, or
 * undefined when the current one stays. */
export function seedChampion(rt: RunDeps): Champion | undefined {
  const cur = rt.store.currentChampion();
  if (cur && cur.contentVersion === rt.content.version) return undefined;
  const day = rt.today();
  const { player, line } = strongBotLine(rt);
  const champ: Champion = {
    seq: day.seq,
    day: day.day,
    player: { ...player, name: `${player.name} the First` },
    line,
    since: rt.now().toISOString(),
    contentVersion: rt.content.version,
  };
  rt.store.addPlayer(champ.player);
  rt.store.putChampion(champ);
  return champ;
}

// ---------- the job ----------

/** Seeds the champion now, tops the pool up at once and every BOT_TOPUP_MS,
 * RUNS_PER_TURN bot runs per event-loop turn. Returns the stop function. */
export const botWorld: MvpJob = (rt: MvpRuntime) => {
  let stopped = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  let busy = false;
  const log = (msg: string) => console.log(`[bots] ${msg}`);
  try {
    const champ = seedChampion(rt);
    if (champ) log(`champion for day ${champ.seq}: ${champ.player.name} (${champ.line.map((u) => u.name).join(", ")})`);
  } catch (e) {
    console.error(`[bots] champion seed failed: ${(e as Error).message}`);
  }
  const tick = () => {
    if (stopped || busy) return;
    busy = true;
    let runs = 0;
    const turn = () => {
      if (stopped) return;
      try {
        for (let i = 0; i < RUNS_PER_TURN; i++) {
          if (runs >= MAX_RUNS_PER_TOPUP || thinRounds(rt).length === 0) {
            if (runs > 0) log(`topped up: ${runs} bot runs`);
            return done();
          }
          playBotRun(rt);
          runs++;
        }
      } catch (e) {
        console.error(`[bots] top-up failed: ${(e as Error).message}`);
        return done();
      }
      timer = setTimeout(turn, 0);
    };
    const done = () => {
      busy = false;
      if (!stopped) timer = setTimeout(tick, BOT_TOPUP_MS);
    };
    turn();
  };
  timer = setTimeout(tick, 0);
  return () => {
    stopped = true;
    if (timer) clearTimeout(timer);
  };
};
