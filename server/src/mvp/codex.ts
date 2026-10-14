// The Codex on the server (rules v2, slice makscee/void-board#1049; design
// page §6). With rt.codex on (main.ts: ARENA_CODEX=1), a fuse goes through
// the Codex instead of the v1 splice and its namer:
// - unforged pair: refused ("not forged yet: wish for it"); a wish puts it on
//   tonight's list (POST /codex/wish).
// - ready to claim: a human picks one of the three sealed designs (the fuse's
//   `pick`); it becomes canonical with their name, final forever. A bot fuses
//   with the design it chose (./bots.ts simulates them) and claims nothing:
//   bots are never credited, so the pair stays ready for a human.
// - discovered: the canonical design, credited to its claimer.
// The forge view (GET /runs/:id/forge) lists every pair the run can fuse now
// with its state; for a ready pair the run holds, the three designs show, so
// the claim is a choice. Elsewhere (GET /codex/:a/:b) they stay sealed.
import { benchSizeOf, boardUnit, type Decision, type FuseContext, type LineUnit, type PlayerRef, type UnitId } from "../../../src/mvp/contract.js";
import { canonicalDesign, claimCheck, forgeView, pairState, sortPair, type CodexClaim, type ForgeOffer, type ForgeSlot, type ForgeView, type RunForge } from "../../../src/mvp/codex.js";
import { fuseCheck } from "../../../src/mvp/forms.js";
import { MvpDecisionError } from "../../../src/mvp/errors.js";
import { fightLines } from "../../../src/mvp/fight.js";
import { applyMvpDecision, type MvpRunState } from "../../../src/mvp/run.js";
import { lineResolves } from "../../../src/mvp/units.js";
import type { RunDeps } from "./runs.js";

/** A fuse through the Codex: its context, and the claim to store once the
 * rules accepted it (a human's pick on a ready pair). */
export interface CodexFuse {
  ctx: FuseContext;
  claim?: { a: UnitId; b: UnitId; claim: CodexClaim };
}

/** The Codex's context for a fuse `d` the rules allow; undefined for one they
 * refuse anyway (the run then says why). Throws MvpDecisionError when the
 * Codex refuses it. */
export function codexFuse(deps: RunDeps, run: MvpRunState, d: Decision): CodexFuse | undefined {
  if (d.kind !== "fuse" || d.first === d.second) return undefined;
  const first = boardUnit(run, run.rules, d.first);
  const second = boardUnit(run, run.rules, d.second);
  if (!first || !second || fuseCheck(first, second) !== null) return undefined;
  const [a, b] = sortPair(first.unitId, second.unitId);
  const pair = deps.store.codexPair(a, b);
  const state = pairState(pair);
  if (state === "discovered") {
    const design = canonicalDesign(pair)!;
    return { ctx: { name: design.name, discoveredBy: pair!.claim!.by, design } };
  }
  if (state === "ready" && run.player.bot) {
    const design = pair!.candidates[d.pick ?? 0] ?? pair!.candidates[0]!;
    return { ctx: { name: design.name, discoveredBy: null, design } };
  }
  const why = claimCheck(pair, d.pick);
  if (why) throw new MvpDecisionError("fuse", why);
  const design = pair!.candidates[d.pick!]!;
  const claim: CodexClaim = { pick: d.pick!, by: run.player, at: deps.now().toISOString() };
  return { ctx: { name: design.name, discoveredBy: run.player, design }, claim: { a, b, claim } };
}

/** Every pair the run can fuse now (two Awoken, neither fused), front first. */
export function forgeOf(deps: RunDeps, run: MvpRunState): ForgeSlot[] {
  const board: [number, LineUnit][] = [];
  for (let i = 0; i < run.rules.lineSize + benchSizeOf(run.rules); i++) {
    const u = boardUnit(run, run.rules, i);
    if (u) board.push([i, u]);
  }
  const seq = deps.today().seq;
  const out: ForgeSlot[] = [];
  for (let x = 0; x < board.length; x++)
    for (let y = x + 1; y < board.length; y++) {
      const [i, u] = board[x]!;
      const [j, v] = board[y]!;
      if (fuseCheck(u, v) !== null) continue;
      const view = codexView(deps, u.unitId, v.unitId, run.player, seq);
      const slot: ForgeSlot = { ...view, first: i, second: j };
      const pair = view.state === "ready" ? deps.store.codexPair(view.a, view.b) : undefined;
      if (pair) slot.candidates = pair.candidates;
      out.push(slot);
    }
  return out;
}

/** The shop offers that bring a designed fusion within reach (#1049
 * reopened): an offer whose unit pairs with an Awoken unit on the board, the
 * pair discovered or ready to claim. Unforged pairs, a unit already Awoken
 * (its pairs are on the board already) and fused units never show. */
export function forgeOffers(deps: RunDeps, run: MvpRunState): ForgeOffer[] {
  const board: [number, LineUnit][] = [];
  for (let i = 0; i < run.rules.lineSize + benchSizeOf(run.rules); i++) {
    const u = boardUnit(run, run.rules, i);
    if (u) board.push([i, u]);
  }
  const out: ForgeOffer[] = [];
  for (const o of run.offers) {
    const own = board.find(([, u]) => u.kind === "unit" && u.unitId === o.unitId)?.[1];
    if (own && own.form === "awoken") continue;
    const awakens = (own?.copies ?? 0) + 1 >= run.rules.copiesToAwaken;
    for (const [j, v] of board) {
      if (v.kind !== "unit" || v.form !== "awoken" || v.unitId === o.unitId) continue;
      const [a, b] = sortPair(o.unitId, v.unitId);
      const pair = deps.store.codexPair(a, b);
      const state = pairState(pair);
      if (state === "unforged") continue;
      out.push({ slot: o.slot, partner: j, a, b, state, ...(state === "discovered" ? { design: canonicalDesign(pair)! } : {}), awakens });
    }
  }
  return out;
}

/** The forge of a run: its fusable pairs and the offers that bring one within reach. */
export function runForge(deps: RunDeps, run: MvpRunState): RunForge {
  return { pairs: forgeOf(deps, run), offers: run.phase === "shop" ? forgeOffers(deps, run) : [] };
}

/** A pair's Codex view for `viewer` today; candidates stay sealed. */
export function codexView(deps: RunDeps, x: UnitId, y: UnitId, viewer: PlayerRef | undefined, seq = deps.today().seq): ForgeView {
  const [a, b] = sortPair(x, y);
  const wishes = deps.store.wishes(seq).find((w) => w.a === a && w.b === b)?.count ?? 0;
  return forgeView(a, b, deps.store.codexPair(a, b), wishes, viewer ? deps.store.wishedBy(a, b, viewer.id, seq) : false);
}

/** A wish on an unforged pair (unlimited: one per player, pair and day counts
 * toward tonight's list). Throws MvpDecisionError on an unknown unit or a
 * pair that is already forged. */
export function wish(deps: RunDeps, player: PlayerRef, x: UnitId, y: UnitId): ForgeView {
  const known = (id: UnitId) => deps.content.units.some((u) => u.id === id) || (deps.content.left ?? []).some((u) => u.id === id);
  if (!known(x) || !known(y)) throw new MvpDecisionError("wish", `no unit ${known(x) ? y : x}`);
  if (x === y) throw new MvpDecisionError("wish", "a unit can't fuse with itself");
  const [a, b] = sortPair(x, y);
  if (pairState(deps.store.codexPair(a, b)) !== "unforged") throw new MvpDecisionError("wish", "already forged");
  const seq = deps.today().seq;
  deps.store.addWish({ a, b, playerId: player.id, seq, at: deps.now().toISOString() });
  return codexView(deps, a, b, player, seq);
}

// ---------- bots at the forge ----------

/** How many of the round's newest ghosts a bot fights each design against. */
export const BOT_FORGE_GHOSTS = 5;

/** Whether a bot may fuse the pair: forged (ready or discovered). */
export function codexForged(deps: RunDeps, first: LineUnit, second: LineUnit): boolean {
  const [a, b] = sortPair(first.unitId, second.unitId);
  return pairState(deps.store.codexPair(a, b)) !== "unforged";
}

/** A bot's fuse `d` with its pick: on a ready pair, each candidate is fused
 * into the bot's line and fought against the round's newest ghosts (fixed
 * seeds, so the pick is deterministic); the most wins takes it, a draw half,
 * ties to the earlier candidate. Bots stay heuristic everywhere else. */
export function botForge(deps: RunDeps, run: MvpRunState, d: Extract<Decision, { kind: "fuse" }>): Extract<Decision, { kind: "fuse" }> {
  const first = boardUnit(run, run.rules, d.first);
  const second = boardUnit(run, run.rules, d.second);
  if (!first || !second) return d;
  const [a, b] = sortPair(first.unitId, second.unitId);
  const pair = deps.store.codexPair(a, b);
  if (pairState(pair) !== "ready") return d;
  const content = deps.contentFor(run.contentVersion);
  const ghosts = deps.store.ghosts(run.round, { excludePlayerId: run.player.id, limit: BOT_FORGE_GHOSTS }).filter((g) => lineResolves(g.line, content.abilities));
  let pick = 0;
  let best = -1;
  pair!.candidates.forEach((design, i) => {
    let wins = 0;
    try {
      const line = applyMvpDecision(run, { kind: "fuse", first: d.first, second: d.second }, content, { fuse: { name: design.name, discoveredBy: null, design } }).state.line;
      ghosts.forEach((g, k) => {
        const r = fightLines({ player: run.player, line }, { player: g.player, line: g.line }, { battleId: "forge-sim", seed: k + 1, kind: "round", round: run.round, runId: null, at: "", content, rules: run.rules });
        wins += r.winner === "A" ? 1 : r.winner === "draw" ? 0.5 : 0;
      });
    } catch {
      return; // a design the engine can't fight is never picked
    }
    if (wins > best) [pick, best] = [i, wins];
  });
  return { ...d, pick };
}
