// Bots and the world (mission #574, slice 6).
// - seedChampion: at start, when currentChampion() is missing or stale
//   (contract.ts, "day, champion, rating"), writes a strong bot team as the
//   champion for rt.today().seq. "Strong" is measured: bot runs on a scratch
//   store, then a round-robin of their round-12 lines through fightLines.
// - topUpGhosts: bot players (PlayerRef bot: true) play whole runs in-process
//   through startRun and decide (./runs.ts) on `rt`, so they share the store,
//   the day and the hooks with HTTP runs, until every round's pool holds
//   BOT_TARGET ghosts on the live content. Bots play whole runs, the Crown
//   included: a bot's Crown win is a Slay like a player's, so its team can
//   enter the playoff and be crowned, and Maks alone still sees a real
//   playoff (#587); a bot's rating never moves. Bots never claim fusion
//   credit (slice 10's namer gives a bot's pair discoveredBy null). A bot fuses a pair only once
//   slice 10 has its name ready (fusionNameReady): the top-up's bots run in
//   the background and wait for the name (awaitFusionName, bounded by the
//   model's final failure), so a bot never fixes the portmanteau while the
//   model's name is coming, and the pool still holds fusions.
//   Bots also play every day (#587): runs until BOT_DAILY_CROWNS of the day's
//   Crown fights are bots' and BOT_DAILY_SLAYERS bots slew (capped by
//   BOT_MAX_DAILY_CROWNS), so each day's champion meets challengers and a
//   player alone still sees slayers and a playoff on day 2 and later, not only
//   on a fresh world's first day. The pool stays bounded per day: a round
//   below BOT_TARGET is filled, and a day adds the ghosts of the runs that
//   reach its Crown quota (pickGhost draws from a round's newest
//   GHOST_PICK_POOL anyway). No live champion, no daily runs.
// - botWorld: the job (./jobs.ts) that seeds, tops up at start and every
//   BOT_TOPUP_MS (the thin rounds, then the day's Crown quota), a few runs per
//   tick so requests keep flowing. The seed waits
//   for the model's names of the champion's fusions before it writes them.
import { randomUUID } from "node:crypto";
import { benchSizeOf, boardUnit, lockedFull, type Champion, type Decision, type LineUnit, type MvpContent, type MvpRules, type PlayerRef, type RunView } from "../../../src/mvp/contract.js";
import { fightLines } from "../../../src/mvp/fight.js";
import { fuseCheck, lineUnitOf, mergeTarget } from "../../../src/mvp/forms.js";
import type { MvpRunState } from "../../../src/mvp/run.js";
import { todaysChampion } from "./day.js";
import { awaitFusionName, fusionNameReady, recordFusion } from "./fusions.js";
import { decide, startRun, type RunDeps } from "./runs.js";
import type { MvpJob, MvpRuntime } from "./runtime.js";
import { MemoryMvpStore } from "./store.js";

/** Ghosts per round (live content) the bots keep. Tunable. */
export const BOT_TARGET = 24;
/** Bot Crown fights each day, at least: bots play runs until this many of
 * the day's Crown fights are bots', so each day's champion meets challengers.
 * A bot run reaches the Crown about 1 time in 4 once the pool is full. Set it
 * to 0 for no daily bot runs at all. Tunable. */
export const BOT_DAILY_CROWNS = 12;
/** Bot slayers a day aims for, so a player alone sees a real playoff (how to
 * try, step 5): past BOT_DAILY_CROWNS, bots keep fighting the Crown until this
 * many different bots slew it, up to BOT_MAX_DAILY_CROWNS. How often a bot
 * slays varies a lot with the champion (1 Crown in 10 to 1 in 3 measured), so
 * a fixed quota left 8 of 20 fresh worlds with fewer than 2 (#587). Tunable. */
export const BOT_DAILY_SLAYERS = 2;
/** The most bot Crown fights a day (about 240 bot runs, ~60 MB of battles). */
export const BOT_MAX_DAILY_CROWNS = 60;
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

/** A fresh bot identity for one run: a unique id (so a bot's run left before
 * the Crown never blocks its next run) and a roster name none of `taken` has:
 * the seed's roster name, else the roster's next free one, else a numbered
 * one (bot-Esk-2). */
export function botPlayer(seed: number, taken: ReadonlySet<string> = new Set()): PlayerRef {
  const start = (seed >>> 0) % BOT_NAMES.length;
  for (let n = 1; ; n++) {
    for (let i = 0; i < BOT_NAMES.length; i++) {
      const name = `bot-${BOT_NAMES[(start + i) % BOT_NAMES.length]}${n > 1 ? `-${n}` : ""}`;
      if (!taken.has(name)) return { id: `bot-${randomUUID()}`, name, bot: true };
    }
  }
}

/** The names a new bot run must not take (#587): today's slayers' and every
 * champion's. Bots slay and enter the playoff, and each bot run is a new
 * player, so without this two of the day's playoff entrants, or two days'
 * champions, could share a name and read as one bot playing itself. Bots
 * play one run at a time, so a name free at the run's start is still free
 * at its Crown. */
export function takenBotNames(rt: Pick<RunDeps, "store" | "today">): Set<string> {
  const names = new Set(rt.store.slays(rt.today().seq).map((s) => s.player.name));
  for (const c of rt.store.champions()) names.add(c.player.name);
  return names;
}

// ---------- the bot's play ----------

function score(u: LineUnit): number {
  return u.stats.pwr * 2 + u.stats.hp + (u.form === "awoken" ? 6 : 0) + (u.kind === "fused" ? 10 : 0);
}

/** The bot's next decision in the shop: fuse two Awoken units whose name is
 * ready (`ready`, by default always) from anywhere on the board, buy copies
 * first (they awaken, on the bench too), fill the line with the highest tier;
 * with the line full, keep a better tier than the weakest single copy on the
 * bench while it has room, else swap that single for it; lock a copy it can't
 * afford yet, reroll once a round while gold allows (unless locked offers
 * fill the shop), field the best 5 (bench units into empty line slots, and in
 * for the weakest line unit when they score higher), put the toughest unit in
 * front, then fight. Board slots: the line, then the bench (boardSlot).
 * `rerolled` counts this round's rerolls (the caller resets it). Pure. */
export function botDecision(
  run: RunView,
  content: MvpContent,
  rules: MvpRules,
  rerolled: number,
  ready: (first: LineUnit, second: LineUnit) => boolean = () => true,
): Decision {
  if (run.phase !== "shop") return { kind: "fight" };
  const line = run.line;
  const bench = run.bench ?? [];
  const benchRoom = bench.length < benchSizeOf(rules);
  if (run.gift) return giftDecision(run.gift, line, bench, content, rules);
  // Every unit with its board slot: the line, then the bench.
  const board = [...line.map((u, i) => ({ u, i })), ...bench.map((u, i) => ({ u, i: rules.lineSize + i }))];
  // Fuse: the strongest fusable pair whose name is ready, the stronger first.
  const awoken = board.filter(({ u }) => u.kind === "unit" && u.form === "awoken").sort((a, b) => score(b.u) - score(a.u));
  for (let a = 0; a < awoken.length; a++)
    for (let b = a + 1; b < awoken.length; b++) {
      const [x, y] = [awoken[a]!, awoken[b]!];
      if (fuseCheck(x.u, y.u) === null && ready(x.u, y.u) && !fansOut(x.u, y.u)) return { kind: "fuse", first: x.i, second: y.i };
    }

  const tierOf = (id: string) => content.units.find((u) => u.id === id)?.tier ?? 1;
  const affordable = run.offers.filter((o) => o.cost <= run.gold);
  const owned = (unitId: string) => mergeTarget(line, unitId) >= 0 || mergeTarget(bench, unitId) >= 0;
  const copy = affordable.find((o) => owned(o.unitId));
  if (copy) return { kind: "buy", slot: copy.slot };
  const best = [...affordable].sort((a, b) => b.tier - a.tier)[0];
  if (best && line.length < rules.lineSize) return { kind: "buy", slot: best.slot };
  if (best && line.length >= rules.lineSize) {
    // A better tier than the weakest single, sleeping copy: onto the bench
    // while it has room, else sell that single for it.
    const weak = board.filter(({ u }) => u.kind === "unit" && u.copies === 1).sort((a, b) => tierOf(a.u.unitId) - tierOf(b.u.unitId) || score(a.u) - score(b.u))[0];
    if (weak && tierOf(weak.u.unitId) < best.tier) return benchRoom ? { kind: "buy", slot: best.slot } : { kind: "sell", index: weak.i };
  }
  // Lock a copy it can't afford yet: it waits for next round's gold. Not in
  // the last shop round: the Crown clears the offers.
  const later = run.round < rules.rounds && run.offers.find((o) => !o.locked && o.cost > run.gold && owned(o.unitId));
  if (later) return { kind: "lock", slot: later.slot };
  const rerollable = !lockedFull({ offers: run.offers, rules, round: run.round });
  if (rerolled < 2 && rerollable && run.gold >= rules.rerollCost + rules.unitCost) return { kind: "reroll" };
  // Field the best 5: the best bench unit fills an empty line slot, or comes
  // in for the weakest line unit when it scores higher (each swap raises the
  // line's score, so this ends).
  const bestBench = bench.map((u, i) => ({ u, i: rules.lineSize + i })).sort((a, b) => score(b.u) - score(a.u))[0];
  if (bestBench && line.length < rules.lineSize) return { kind: "reorder", from: bestBench.i, to: line.length };
  if (bestBench) {
    const weakest = line.map((u, i) => ({ u, i })).sort((a, b) => score(a.u) - score(b.u))[0];
    if (weakest && score(bestBench.u) > score(weakest.u)) return { kind: "reorder", from: bestBench.i, to: weakest.i };
  }
  // The toughest unit goes in front.
  if (line.length > 1) {
    let front = 0;
    for (let i = 1; i < line.length; i++) if (line[i]!.stats.hp > line[front]!.stats.hp) front = i;
    if (front !== 0) return { kind: "reorder", from: front, to: 0 };
  }
  return { kind: "fight" };
}

/** The bot's awakening-gift pick: a copy of a unit it has first (it merges,
 * so it needs no room), else with room on the line or bench the choice whose
 * fresh unit scores best; else it skips. */
export function giftDecision(gift: readonly string[], line: LineUnit[], bench: LineUnit[], content: MvpContent, rules: MvpRules): Decision {
  const copy = gift.findIndex((id) => mergeTarget(line, id) >= 0 || mergeTarget(bench, id) >= 0);
  if (copy >= 0) return { kind: "gift", pick: copy };
  if (line.length >= rules.lineSize && bench.length >= benchSizeOf(rules)) return { kind: "gift", pick: null };
  let pick: number | null = null;
  let best = -Infinity;
  gift.forEach((id, i) => {
    const u = content.units.find((x) => x.id === id);
    if (!u) return;
    const s = score(lineUnitOf(u, "gift", 1, rules));
    if (s > best) [pick, best] = [i, s];
  });
  return { kind: "gift", pick };
}

/** Plays one whole bot run through startRun and decide on `deps`, the Crown
 * included, until it is over, never waiting: a pair whose name isn't ready is
 * not fused. For stores with no namer (the champion search's scratch store,
 * tests); botWorld plays playBotRunWaiting. Returns the ended run. */
export function playBotRun(deps: RunDeps, player: PlayerRef = botPlayer(deps.seed(), takenBotNames(deps))): MvpRunState {
  deps.store.addPlayer(player);
  let run = startRun(deps, player);
  let round = run.round;
  let rerolled = 0;
  const ready = (first: LineUnit, second: LineUnit) => fusionNameReady(deps.store, first.unitId, second.unitId);
  for (let guard = 0; run.phase !== "over" && guard < 1000; guard++) {
    if (run.round !== round) {
      round = run.round;
      rerolled = 0;
    }
    const d = botDecision(run, deps.contentFor(run.contentVersion), deps.rules, rerolled, ready);
    if (d.kind === "reroll") rerolled++;
    decide(deps, run, d);
    run = deps.store.run(run.runId)!;
  }
  return run;
}

/** A fused unit whose When listens to a one-unit event (an ally shielded,
 * healed, powered, summoned) and whose Who is a group breaks the pool's chain
 * discipline (src/mvp/units.ts): it fans out n² and, with hostile Does on its
 * own allies, can stall a battle to the turn cap. Bots don't make one. */
function fansOut(first: LineUnit, second: LineUnit): boolean {
  if (first.kind !== "unit" || second.kind !== "unit") return false;
  const oneUnit = first.recipe.when.some((w) => w.kind === "trigger" && ["StatusApplied", "Heal", "StatChanged", "Summon"].includes(w.on.on));
  return oneUnit && second.recipe.who.some((w) => w.kind.startsWith("all"));
}

/** playBotRun for the background: a bot that wants to fuse a pair whose name
 * isn't ready waits for it (awaitFusionName), so it fuses as often as with no
 * model. Between waits it runs synchronously. Stops early, leaving the run
 * open, once `stopped()`. Returns the run as it ended. */
export async function playBotRunWaiting(deps: RunDeps, stopped: () => boolean = () => false, player: PlayerRef = botPlayer(deps.seed(), takenBotNames(deps))): Promise<MvpRunState> {
  deps.store.addPlayer(player);
  let run = startRun(deps, player);
  let round = run.round;
  let rerolled = 0;
  for (let guard = 0; run.phase !== "over" && guard < 1000 && !stopped(); guard++) {
    if (run.round !== round) {
      round = run.round;
      rerolled = 0;
    }
    const d = botDecision(run, deps.contentFor(run.contentVersion), deps.rules, rerolled);
    if (d.kind === "fuse") {
      const [first, second] = [boardUnit(run, run.rules, d.first)!, boardUnit(run, run.rules, d.second)!];
      if (!fusionNameReady(deps.store, first.unitId, second.unitId)) {
        await awaitFusionName(deps.store, first.unitId, second.unitId);
        run = deps.store.run(run.runId)!;
        continue;
      }
    }
    if (d.kind === "reroll") rerolled++;
    decide(deps, run, d);
    run = deps.store.run(run.runId)!;
  }
  return run;
}

// ---------- the pool ----------

/** Rounds that hold fewer than `target` ghosts (made on any pool). */
export function thinRounds(rt: RunDeps, target = BOT_TARGET): number[] {
  const out: number[] = [];
  for (let r = 1; r <= rt.rules.rounds; r++) {
    if (rt.store.ghosts(r, { excludePlayerId: "", limit: target }).length < target) out.push(r);
  }
  return out;
}

/** How many more Crown fights bots owe today: `quota` minus today's bot
 * Crown fights; past it, one more while fewer than BOT_DAILY_SLAYERS
 * different bots slew today's champion and fewer than BOT_MAX_DAILY_CROWNS
 * were fought. 0 with no champion today (a run would end "no-champion",
 * never fighting one), or a quota of 0. Asked again after every Crown fight. */
export function crownsOwed(rt: RunDeps, quota = BOT_DAILY_CROWNS): number {
  const champ = todaysChampion(rt);
  if (!champ || quota <= 0) return 0;
  const fought = rt.store.battles({ kind: "crown", since: rt.today().startedAt }).filter((b) => b.player.bot).length;
  if (fought < quota) return quota - fought;
  const slayers = new Set(rt.store.slays(rt.today().seq).filter((s) => s.player.bot).map((s) => s.player.id)).size;
  return slayers < BOT_DAILY_SLAYERS && fought < BOT_MAX_DAILY_CROWNS ? 1 : 0;
}

const foughtCrown = (run: MvpRunState) => run.endedBy === "crown-won" || run.endedBy === "crown-lost";

/** Plays bot runs until every round holds `target` ghosts and bots fought
 * today's `dailyCrowns` Crown fights (crownsOwed), or `maxRuns` ran.
 * Synchronous; botWorld spreads the same work over event-loop turns. */
export function topUpGhosts(rt: RunDeps, opts: { target?: number; maxRuns?: number; dailyCrowns?: number } = {}): { runs: number; thin: number[]; crownsOwed: number } {
  const target = opts.target ?? BOT_TARGET;
  const maxRuns = opts.maxRuns ?? MAX_RUNS_PER_TOPUP;
  let runs = 0;
  let owed = crownsOwed(rt, opts.dailyCrowns);
  let thin = thinRounds(rt, target);
  while ((thin.length > 0 || owed > 0) && runs < maxRuns) {
    if (foughtCrown(playBotRun(rt))) owed = crownsOwed(rt, opts.dailyCrowns);
    runs++;
    thin = thinRounds(rt, target);
  }
  return { runs, thin, crownsOwed: Math.max(0, owed) };
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
    // Survived all rounds: it fought the Crown, or (no champion yet) "no-champion".
    const reachedCrown = run.endedBy === "crown-won" || run.endedBy === "crown-lost" || run.endedBy === "no-champion";
    if (reachedCrown && run.line.length > 0) finishers.push({ player: run.player, line: run.line });
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
 * champion (the day-1 seed). A champion made on another pool stays: a pool
 * change never hands a human's crown to a bot (M2-2). Its fusions are stored as bot
 * discoveries (slice 10's recordFusion, discoveredBy null), each once its name
 * is ready (awaitFusionName: the model's, or the portmanteau after the model's
 * final failure); a pair already stored keeps its name, and the champion's
 * unit takes it, so a human's later fuse of the pair shows the same name and
 * claims the credit. Resolves to the one written, or undefined when the
 * current one stays. */
export async function seedChampion(rt: RunDeps): Promise<Champion | undefined> {
  if (rt.store.currentChampion()) return undefined;
  const { player, line } = strongBotLine(rt);
  await Promise.all(line.map((u) => (u.fusion ? awaitFusionName(rt.store, u.fusion.first, u.fusion.second) : undefined)));
  // Checked again: a champion may have been written while the names came.
  if (rt.store.currentChampion()) return undefined;
  const day = rt.today();
  const at = rt.now().toISOString();
  const unit = (id: string) => rt.content.units.find((c) => c.id === id)!;
  for (const u of line) {
    if (!u.fusion) continue;
    // The search named it on a scratch store: the real store's name now.
    const name = rt.peekFusionName(unit(u.fusion.first), unit(u.fusion.second), player).name;
    const stored = recordFusion(rt.store, { ...u.fusion, name }, at);
    u.name = stored.name;
    u.fusion = { ...u.fusion, name: stored.name, discoveredBy: stored.discoveredBy };
  }
  const champ: Champion = {
    seq: day.seq,
    day: day.day,
    player: { ...player, name: `${player.name} the First` },
    line,
    since: at,
    contentVersion: rt.content.version,
    rating: rt.rules.botRating,
  };
  rt.store.addPlayer(champ.player);
  rt.store.putChampion(champ);
  return champ;
}

// ---------- the job ----------

/** Seeds the champion first, then tops the pool up at once and every
 * BOT_TOPUP_MS (thin rounds, then the day's BOT_DAILY_CROWNS: a day ended
 * by the rollover or dev end-day gets its bot challengers within a tick),
 * one bot run after another, yielding to the event loop every
 * RUNS_PER_TURN runs and whenever a bot waits for a fusion's name. Returns the
 * stop function. */
export const botWorld: MvpJob = (rt: MvpRuntime) => {
  let stopped = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const log = (msg: string) => console.log(`[bots] ${msg}`);
  const isStopped = () => stopped;
  const yieldTurn = () => new Promise<void>((resolve) => setImmediate(resolve));
  const topUp = async () => {
    let runs = 0;
    let crowns = 0;
    try {
      let owed = crownsOwed(rt);
      while (!stopped && runs < MAX_RUNS_PER_TOPUP && (owed > 0 || thinRounds(rt).length > 0)) {
        if (foughtCrown(await playBotRunWaiting(rt, isStopped))) {
          owed = crownsOwed(rt);
          crowns++;
        }
        if (++runs % RUNS_PER_TURN === 0) await yieldTurn();
      }
      if (runs > 0 && !stopped) log(`topped up: ${runs} bot runs, ${crowns} Crown fights, ${rt.store.slays(rt.today().seq).filter((s) => s.player.bot).length} bot slays today`);
    } catch (e) {
      console.error(`[bots] top-up failed: ${(e as Error).message}`);
    }
    if (!stopped) timer = setTimeout(() => void topUp(), BOT_TOPUP_MS);
  };
  const start = async () => {
    try {
      const champ = await seedChampion(rt);
      if (champ) log(`champion for day ${champ.seq}: ${champ.player.name} (${champ.line.map((u) => u.name).join(", ")})`);
    } catch (e) {
      console.error(`[bots] champion seed failed: ${(e as Error).message}`);
    }
    if (!stopped) await topUp();
  };
  void start();
  return () => {
    stopped = true;
    if (timer) clearTimeout(timer);
  };
};
