// Rotation at the day end (mission 2, M2-10, makscee/void-board#795). The
// pool changes a little each day: the qualified candidates (M2-8) enter, best
// first, at most rules.rotationEntrants a day, and for each one a unit that
// has been live at least rules.rotationMinStay days leaves, least played
// first. The pool never shrinks: with fewer leavers than entrants, fewer
// enter and the rest wait for the next day.
//
// endDay (./day.ts) runs it after the playoff and the champion and before the
// next day is stored, only with the switch on (MVP_ROTATION=1, rt.rotation).
// Every write is keyed by the new day's seq, and a pool already stored for
// that day means the rotation ran: a retried day end does nothing twice.
// Runs in progress keep the pool they started on (M2-2).
//
// M3-7 (mission #800): the entrants split into two queues. Of the
// rules.rotationEntrants slots, rules.rotationEvolveShare are for evolutions
// and returns (each archetype's entry, M3-6), the rest for ideas; a slot one
// queue can't fill goes to the other. A version enters `live` (stint reason
// `evolution`); a return reopens the Library unit itself (a new stint,
// reason `return`). Either way the archetype's other proposals on the cards
// close to the Library, not refunded, and the old version stays there.
//
// `npm run mvp:rotate -- --db <path> --dry-run` prints the plan for the
// coming day end and writes nothing (./rotate-cli.ts).
import type { CandidateScore, MvpRules, UnitId } from "../../../src/mvp/contract.js";
import type { Row } from "../../../src/mvp/units.js";
import { contentOf } from "./content.js";
import type { MvpStore, PoolSnapshot } from "./store.js";
import { candidateScores, contests, qualified } from "./votes.js";

/** The rotation tunables, defaults filled in. */
export function rotationRules(rules: Pick<MvpRules, "rotationEntrants" | "rotationEvolveShare" | "rotationMinStay" | "rotationWindow">): { entrants: number; evolveShare: number; minStay: number; window: number } {
  const entrants = Math.max(0, Math.floor(rules.rotationEntrants ?? 3));
  return {
    entrants,
    evolveShare: Math.min(entrants, Math.max(0, Math.floor(rules.rotationEvolveShare ?? 1))),
    minStay: Math.max(1, Math.floor(rules.rotationMinStay ?? 14)),
    window: Math.max(1, Math.floor(rules.rotationWindow ?? 14)),
  };
}

/** How much a leaver's coolness and meta health weigh against how much it is
 * played (0–1 each): keep = play + COOL_WEIGHT × coolness + HEALTH_WEIGHT × health. */
export const COOL_WEIGHT = 0.25;
export const HEALTH_WEIGHT = 0.25;

/** A live unit that may leave, and why it ranks where it does. */
export interface LeaverScore {
  unitId: UnitId;
  name: string;
  emoji: string;
  /** Days live in its current stay, the ending day included. */
  days: number;
  /** Picks (bought or gifted) over the window. */
  picks: number;
  /** picks / the most picks among the units that may leave (0–1). */
  play: number;
  /** The share of either/or votes it won where it was on a card (M2-8); 0.5 with none. */
  coolness: number;
  /** Votes behind coolness (skips not counted). */
  votes: number;
  /** 1 − 2·|its team's win rate − ½| over the window; 1 with no fights. */
  health: number;
  /** play + COOL_WEIGHT × coolness + HEALTH_WEIGHT × health: the lowest leaves first. */
  keep: number;
}

/** The two queues the entrants come from (M3-7): new ideas, and evolutions
 * and returns. */
export type RotationSlot = "idea" | "evolution";

/** Which queue an entrant comes from. */
export const slotOf = (s: CandidateScore): RotationSlot => (s.kind === "idea" ? "idea" : "evolution");

/** What a day end would do: who enters, who leaves in their place, and why. */
export interface RotationPlan {
  /** The day the new pool starts (the ending day's seq + 1). */
  daySeq: number;
  /** The pool it changes. */
  from: string;
  /** Each entrant with the unit whose place it takes, in pairs, and the slot
   * it took: its own queue's, or (`passed`) one the other queue couldn't fill. */
  swaps: { entrant: CandidateScore; leaver: LeaverScore; slot: RotationSlot; passed: boolean }[];
  /** The day's slots by queue (M3-7): ideas' and evolutions'. */
  slots: Record<RotationSlot, number>;
  /** Qualified candidates with no leaver for them today. */
  waiting: CandidateScore[];
  /** M3-6: each archetype whose versions are on the cards: its versions and
   * "unchanged", the entry (if any) first. */
  contests: { rootId: UnitId; candidates: CandidateScore[] }[];
  /** Every live unit that may leave, the first to leave first. */
  leavers: LeaverScore[];
  /** Live units that haven't stayed long enough to leave. */
  tooNew: number;
  /** The first day a unit that stays too short today may leave, or null. */
  nextLeaverSeq: number | null;
  /** Why nothing changes, when nothing does. */
  none?: string;
}

const r3 = (x: number) => Math.round(x * 1000) / 1000;

/** The plan for ending day `endingSeq` (default: the current day), on the
 * current pool. Reads only. */
export function rotationPlan(rt: { store: MvpStore; rules: MvpRules }, endingSeq = rt.store.currentDay()?.seq ?? 1): RotationPlan {
  const { store } = rt;
  const tun = rotationRules(rt.rules);
  const pool = store.currentPool();
  const daySeq = endingSeq + 1;
  const plan: RotationPlan = { daySeq, from: pool?.version ?? "", swaps: [], slots: { idea: tun.entrants - tun.evolveShare, evolution: tun.evolveShare }, waiting: [], contests: [], leavers: [], tooNew: 0, nextLeaverSeq: null };
  if (!pool) return { ...plan, none: "no pool in the store" };
  if (pool.daySeq >= daySeq) return { ...plan, none: `the pool for day ${daySeq} is already stored` };

  // Leavers: open stints old enough, scored over the window.
  const eligible: { unitId: UnitId; row: Row; days: number }[] = [];
  for (const unitId of pool.unitIds) {
    const open = store.stints(unitId).filter((s) => s.leftSeq === null).sort((a, b) => b.enteredSeq - a.enteredSeq)[0];
    const unit = store.unit(unitId);
    if (!unit) continue;
    const entered = open?.enteredSeq ?? pool.daySeq;
    const days = endingSeq - entered + 1;
    // The minimum stay gives a new unit time to be tried. The seed's units were
    // tried from the game's first day, so their first stay never holds them
    // (orchestrator's call on #735: ideas can enter from the first rotation).
    const seedStay = unit.origin === "seed" && open?.reason === "seed";
    if (seedStay || days >= tun.minStay) eligible.push({ unitId, row: unit.row, days });
    else {
      plan.tooNew++;
      const may = entered + tun.minStay;
      plan.nextLeaverSeq = plan.nextLeaverSeq === null ? may : Math.min(plan.nextLeaverSeq, may);
    }
  }
  const tally = new Map<UnitId, { picks: number; fights: number; wins: number }>(eligible.map((e) => [e.unitId, { picks: 0, fights: 0, wins: 0 }]));
  for (let d = Math.max(1, endingSeq - tun.window + 1); d <= endingSeq; d++) {
    for (const u of store.dayTallies(d).units) {
      const t = tally.get(u.unitId);
      if (!t) continue;
      t.picks += u.picks;
      t.fights += u.fights;
      t.wins += u.wins;
    }
  }
  const cool = new Map<UnitId, { won: number; votes: number }>();
  for (const v of store.votes()) {
    if (v.pick === null) continue;
    for (const id of [v.candidateId, v.otherId]) {
      const c = cool.get(id) ?? { won: 0, votes: 0 };
      c.votes++;
      if (v.pick === id) c.won++;
      cool.set(id, c);
    }
  }
  const most = Math.max(1, ...[...tally.values()].map((t) => t.picks));
  plan.leavers = eligible
    .map((e): LeaverScore => {
      const t = tally.get(e.unitId)!;
      const c = cool.get(e.unitId);
      const play = t.picks / most;
      const coolness = c?.votes ? c.won / c.votes : 0.5;
      const health = t.fights ? 1 - 2 * Math.abs(t.wins / t.fights - 0.5) : 1;
      const keep = play + COOL_WEIGHT * coolness + HEALTH_WEIGHT * health;
      return { unitId: e.unitId, name: e.row.name, emoji: e.row.emoji, days: e.days, picks: t.picks, play: r3(play), coolness: r3(coolness), votes: c?.votes ?? 0, health: r3(health), keep: r3(keep) };
    })
    .sort((a, b) => a.keep - b.keep || a.picks - b.picks || b.days - a.days || (a.unitId < b.unitId ? -1 : 1));

  // Entrants: the qualified candidates, best first, one per archetype, never
  // one already live, nor one whose archetype has a live version (M3-7).
  const live = new Set(pool.unitIds);
  const liveRoots = new Set(pool.unitIds.map((id) => store.unit(id)?.rootId ?? id));
  const scores = candidateScores(rt);
  plan.contests = contests(scores);
  const ready = qualified(scores).filter((s) => !live.has(s.unitId) && (s.kind === "idea" || !liveRoots.has(s.rootId)));
  // M3-7: each queue fills its own slots, best first; a slot its queue can't
  // fill goes to the other. With fewer leavers than slots, the best scores
  // of those chosen enter.
  const queue = { idea: ready.filter((s) => slotOf(s) === "idea"), evolution: ready.filter((s) => slotOf(s) === "evolution") };
  const own = { idea: Math.min(plan.slots.idea, queue.idea.length), evolution: Math.min(plan.slots.evolution, queue.evolution.length) };
  const spare = plan.slots.idea - own.idea + plan.slots.evolution - own.evolution;
  const passed = { idea: Math.min(spare, queue.idea.length - own.idea), evolution: 0 };
  passed.evolution = Math.min(spare - passed.idea, queue.evolution.length - own.evolution);
  const other = { idea: "evolution", evolution: "idea" } as const;
  const chosen = (["idea", "evolution"] as const).flatMap((q) => queue[q].slice(0, own[q] + passed[q]).map((entrant, i) => ({ entrant, slot: i < own[q] ? q : other[q], passed: i >= own[q] })));
  chosen.sort((a, b) => b.entrant.score - a.entrant.score);
  const k = Math.min(chosen.length, plan.leavers.length);
  plan.swaps = chosen.slice(0, k).map((c, i) => ({ ...c, leaver: plan.leavers[i]! }));
  const entering = new Set(plan.swaps.map((s) => s.entrant.unitId));
  plan.waiting = ready.filter((s) => !entering.has(s.unitId));
  if (!k) {
    plan.none = !tun.entrants
      ? "rules.rotationEntrants is 0"
      : !ready.length
        ? "no candidate has qualified"
        : `no live unit has stayed ${tun.minStay} days yet${plan.nextLeaverSeq !== null ? ` (the first may leave at the end of day ${plan.nextLeaverSeq - 1})` : ""}`;
  }
  return plan;
}

/** Rotates the pool at the end of day `endingSeq`: each entrant takes its
 * leaver's place in a new pool snapshot for day endingSeq + 1. Entrants go
 * `live` with a stint from the new day and their idea `live`: an idea's unit
 * (origin `idea`, stint `idea`), a version (origin `evolution`, stint
 * `evolution`) or a Library unit returning unchanged (its origin kept, stint
 * `return`). For a version or a return, the archetype's other proposals on
 * the cards close: their units and ideas to the `library`, not refunded. Leavers go to the `library` (their stint closed "rotated", their idea
 * `library`). Writes nothing when the plan swaps nothing or the new day's
 * pool is already stored (a retried day end). Builds the new pool before any
 * write, so a unit that doesn't build throws with nothing written. */
export function rotate(rt: { store: MvpStore; rules: MvpRules; now: () => Date }, endingSeq: number): RotationPlan {
  return rt.store.atomically(() => {
    const plan = rotationPlan(rt, endingSeq);
    if (!plan.swaps.length) return plan;
    const { store } = rt;
    const pool = store.currentPool()!;
    const oldRows = pool.rows ?? pool.unitIds.map((id) => store.unit(id)!.row);
    const into = new Map(plan.swaps.map((s) => [s.leaver.unitId, s.entrant.unitId]));
    const unitIds = pool.unitIds.map((id) => into.get(id) ?? id);
    const rows = unitIds.map((id, i) => (into.has(pool.unitIds[i]!) ? store.unit(id)!.row : oldRows[i]!));
    const version = contentOf(rows, unitIds).version;
    const at = rt.now().toISOString();
    const { daySeq } = plan;

    // Pin the old rows first: runs on the old pool keep buying what they bought.
    if (!pool.rows) store.putPool({ ...pool, rows: oldRows });
    const ideaOf = new Map(store.ideas().filter((i) => i.data.unitId).map((i) => [i.data.unitId!, i]));
    for (const { entrant, leaver } of plan.swaps) {
      const out = store.unit(leaver.unitId)!;
      store.putUnit({ ...out, status: "library" });
      for (const s of store.stints(leaver.unitId)) if (s.leftSeq === null) store.putStint({ ...s, leftSeq: daySeq, reason: "rotated" });
      const outIdea = ideaOf.get(leaver.unitId);
      if (outIdea && outIdea.state === "live") store.putIdea({ ...outIdea, state: "library" });

      const inUnit = store.unit(entrant.unitId)!;
      const reason = entrant.kind === "unchanged" ? "return" : entrant.kind === "version" ? "evolution" : "idea";
      store.putUnit({ ...inUnit, status: "live", origin: reason === "return" ? inUnit.origin : reason });
      store.putStint({ unitId: entrant.unitId, enteredSeq: daySeq, leftSeq: null, reason });
      const inIdea = ideaOf.get(entrant.unitId);
      if (inIdea) store.putIdea({ ...inIdea, state: "live" });
      if (entrant.kind === "idea") continue;
      // The archetype's other proposals competed and lost.
      for (const idea of store.ideas({ state: "voting" })) {
        const u = idea.data.unitId ? store.unit(idea.data.unitId) : undefined;
        if (!u || u.unitId === entrant.unitId || (u.rootId ?? u.unitId) !== entrant.rootId) continue;
        store.putUnit({ ...u, status: "library" });
        store.putIdea({ ...idea, state: "library" });
      }
    }
    const next: PoolSnapshot = { version, daySeq, unitIds, createdAt: at, rows };
    store.putPool(next);
    return plan;
  });
}

/** The plan as lines for a person (the dry-run CLI, the server log). */
export function describePlan(plan: RotationPlan, rules: MvpRules): string[] {
  const tun = rotationRules(rules);
  const out = [`rotation for day ${plan.daySeq} (pool ${plan.from}; at most ${tun.entrants} a day: ${plan.slots.idea} for ideas, ${plan.slots.evolution} for evolutions and returns; leavers live ≥ ${tun.minStay} days, played over the last ${tun.window} days)`];
  if (plan.none) out.push(`nothing changes: ${plan.none}`);
  for (const { entrant: e, leaver: l, slot, passed } of plan.swaps) {
    const by = e.kind === "unchanged" ? "returns unchanged" : `${e.kind === "version" ? "version" : "idea"} by ${e.authorId ?? "?"}`;
    const why = passed ? `${slot} slot, passed on: no ${slot === "idea" ? "idea" : "evolution or return"} qualified for it` : `${slot} slot`;
    out.push(`  enters ${e.emoji} ${e.name} (${e.unitId}, ${by}; ${why}): score ${e.score} = ${e.won}/${e.votes} votes + novelty ${e.novelty}`);
    out.push(`  leaves ${l.emoji} ${l.name} (${l.unitId}): keep ${l.keep} = play ${l.play} (${l.picks} picks) + ${COOL_WEIGHT}×coolness ${l.coolness} (${l.votes} votes) + ${HEALTH_WEIGHT}×health ${l.health}; live ${l.days} days`);
  }
  const full = plan.swaps.length < plan.leavers.length;
  for (const e of plan.waiting) out.push(`  waits ${e.emoji} ${e.name} (${e.unitId}): qualified, score ${e.score}, ${full ? `today's slots are taken (${slotOf(e)} queue)` : "no leaver for it today"}`);
  for (const c of plan.contests) {
    const entry = c.candidates.find((s) => s.entry);
    const first = c.candidates[0]!;
    out.push(`  contest ${first.emoji} ${first.name} (archetype ${c.rootId}): ${entry ? `${entry.unitId} is its entry` : "no entry yet"}`);
    for (const s of c.candidates) {
      const what = s.kind === "unchanged" ? "unchanged" : `version by ${s.authorId ?? "?"}`;
      const verdict = s.entry ? "entry" : s.qualified ? "qualified, beaten" : `not qualified`;
      out.push(`    ${s.unitId} ${what}: score ${s.score} = ${s.won}/${s.votes} votes + novelty ${s.novelty}; ${verdict}`);
    }
  }
  const stays = plan.leavers.slice(plan.swaps.length, plan.swaps.length + 5);
  if (stays.length) out.push(`  next to leave: ${stays.map((l) => `${l.name} (keep ${l.keep})`).join(", ")}`);
  out.push(`  ${plan.leavers.length} live units may leave, ${plan.tooNew} stayed too short${plan.nextLeaverSeq !== null ? ` (the first of them may leave at the end of day ${plan.nextLeaverSeq - 1})` : ""}`);
  return out;
}
