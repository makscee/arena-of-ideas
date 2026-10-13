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
import { canonicalDesign, claimCheck, forgeView, pairState, sortPair, type CodexClaim, type FusionDesign, type ForgeView } from "../../../src/mvp/codex.js";
import { fuseCheck } from "../../../src/mvp/forms.js";
import { MvpDecisionError } from "../../../src/mvp/errors.js";
import type { MvpRunState } from "../../../src/mvp/run.js";
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

/** One fusable pair on a run's board, as the forge shows it. */
export interface ForgeSlot extends ForgeView {
  /** Board slots of the two units (the fuse's first and second). */
  first: number;
  second: number;
  /** A ready pair this run holds: the three designs to claim from. */
  candidates?: FusionDesign[];
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
