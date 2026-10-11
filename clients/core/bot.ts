// The reference bot's shop play (#957: moved from server/src/mvp/bots.ts): a
// pure function of what a client sees (RunView, content, rules), so any
// client plays it: the server's bots, `npm run mvp:bot`, an agent's practice
// run. DOM-free and Node-free, like all of clients/core.
import { benchSizeOf, lockedFull, type Decision, type LineUnit, type MvpContent, type MvpRules, type RunView } from "../../src/mvp/contract.js";
import { fuseCheck, lineUnitOf, mergeTarget } from "../../src/mvp/forms.js";

/** How strong the bot thinks a unit is. */
export function botScore(u: LineUnit): number {
  return u.stats.pwr * 2 + u.stats.hp + (u.form === "awoken" ? 6 : 0) + (u.kind === "fused" ? 10 : 0);
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
  const awoken = board.filter(({ u }) => u.kind === "unit" && u.form === "awoken").sort((a, b) => botScore(b.u) - botScore(a.u));
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
    const weak = board.filter(({ u }) => u.kind === "unit" && u.copies === 1).sort((a, b) => tierOf(a.u.unitId) - tierOf(b.u.unitId) || botScore(a.u) - botScore(b.u))[0];
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
  const bestBench = bench.map((u, i) => ({ u, i: rules.lineSize + i })).sort((a, b) => botScore(b.u) - botScore(a.u))[0];
  if (bestBench && line.length < rules.lineSize) return { kind: "reorder", from: bestBench.i, to: line.length };
  if (bestBench) {
    const weakest = line.map((u, i) => ({ u, i })).sort((a, b) => botScore(a.u) - botScore(b.u))[0];
    if (weakest && botScore(bestBench.u) > botScore(weakest.u)) return { kind: "reorder", from: bestBench.i, to: weakest.i };
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
    const s = botScore(lineUnitOf(u, "gift", 1, rules));
    if (s > best) [pick, best] = [i, s];
  });
  return { kind: "gift", pick };
}
