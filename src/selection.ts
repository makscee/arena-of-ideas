// The season selection rule (PRD #083) — the one place the Govern loop hands the
// Play loop its content. At the season roll, this partitions the live ideas
// table into the build slate and the carry-over, and it does so as a PURE,
// decision-sourced function: it reads the directional vote data #082's store
// already holds, names what the new content version should contain, and mutates
// nothing. The season-transition op (season.ts) calls it; the actual content
// version is what Maks ships. Selection is candidacy, never a guarantee.
//
// Two gates, ANDed (Outcome): an idea is ELIGIBLE only when it cleared an
// ENGAGEMENT FLOOR (total votes ≥ SELECTION_VOTE_FLOOR) AND an APPROVAL ratio
// (up / total ≥ SELECTION_APPROVAL_RATIO). The floor kills single-vote noise;
// the ratio kills the divisive 52/48. Eligible ideas rank by approval ratio
// (seq tiebreak, mirroring rankIdeas); the top SELECTION_BUILD_CAPACITY are
// SELECTED, the rest — eligible-but-unbuilt and below-floor alike — are CARRIED
// with their votes intact, doing the priority-queue work with no daily machinery.
//
// No store mutation, no wall-clock, no node built-ins (index.ts re-exports this
// for the browser): a tally in, a partition out, fully unit-testable.

import { jsonClone, type Idea } from "./ideas.js";
import {
  SELECTION_APPROVAL_RATIO,
  SELECTION_BUILD_CAPACITY,
  SELECTION_VOTE_FLOOR,
} from "./tunables.js";

/** One idea's vote tally — derived from #082's directional vote map (read its
 * up/total, not recomputed from some other source). `up` is the up-vote count,
 * `total` is up + down (both are participation), `ratio` is the approval ratio
 * (up / total), 0 when nothing has been voted (an unvoted idea is not approved). */
export interface IdeaTally {
  ideaId: string;
  /** Up-votes — the positive support. */
  up: number;
  /** Total votes, up + down: the engagement the floor measures. */
  total: number;
  /** Approval ratio, up / total; 0 when total is 0 (no votes is no approval). */
  ratio: number;
}

/** Read an idea's tally straight off its directional vote map (#082's store) —
 * the up/total/ratio the gates read. This is a read of the store, not a recount:
 * the vote map IS the directional vote data. */
export function tallyOf(idea: Idea): IdeaTally {
  let up = 0;
  let total = 0;
  for (const dir of Object.values(idea.votes)) {
    total++;
    if (dir === "up") up++;
  }
  return { ideaId: idea.id, up, total, ratio: total === 0 ? 0 : up / total };
}

/** Every idea's tally, keyed by id — the directional data selectSeason reads. */
export function talliesOf(ideas: readonly Idea[]): Map<string, IdeaTally> {
  const out = new Map<string, IdeaTally>();
  for (const idea of ideas) out.set(idea.id, tallyOf(idea));
  return out;
}

/** The three selection knobs, as one bag — defaults from tunables.ts, overridable
 * per call (tests, a future per-season config). Mirrors the flat tunables so the
 * rule reads them in one place rather than re-importing each const everywhere. */
export interface SelectionTunables {
  /** Total votes an idea needs before it is eligible at all (the floor). */
  voteFloor: number;
  /** The minimum approval ratio (up / total) for eligibility. */
  approvalRatio: number;
  /** How many top-ranked eligible ideas are marked selected/building. */
  buildCapacity: number;
}

/** The shipped defaults — the tunables.ts knobs as one bag. */
export const DEFAULT_SELECTION_TUNABLES: SelectionTunables = {
  voteFloor: SELECTION_VOTE_FLOOR,
  approvalRatio: SELECTION_APPROVAL_RATIO,
  buildCapacity: SELECTION_BUILD_CAPACITY,
};

/** Eligibility: FLOOR and APPROVAL, both gates ANDed. An idea below the floor is
 * ineligible even at 100% approval (single-vote noise); an idea past the floor
 * but below the ratio is ineligible (the divisive 52/48). Only both → eligible. */
export function isEligible(tally: IdeaTally, tunables: SelectionTunables): boolean {
  return tally.total >= tunables.voteFloor && tally.ratio >= tunables.approvalRatio;
}

/** An eligible idea with its tally — the ranked slate carries the numbers the
 * roll (and Maks) read, so the season op need not re-derive them. */
export interface RankedIdea {
  idea: Idea;
  tally: IdeaTally;
}

/** What the rule produces at a roll: the full ranked eligible set, the SELECTED
 * top-N (the build slate), and the CARRIED rest (eligible-but-unbuilt ∪
 * below-floor), every carried idea with its votes intact. `selected` is exactly
 * `eligible.slice(0, capacity)`; `carried` is the complement that stays on the
 * table. All entries are detached clones — selection never aliases the store. */
export interface SelectionResult {
  /** Every eligible idea, ranked by approval ratio (seq tiebreak). */
  eligible: RankedIdea[];
  /** The build slate: the top `buildCapacity` of `eligible`. */
  selected: RankedIdea[];
  /** Everything that stays on the table — eligible-but-unbuilt (ranked) then the
   * ineligible (seq order) — votes intact, nothing destroyed. */
  carried: Idea[];
}

/** Run the season selection over the live ideas table. Pure and deterministic:
 * partition the ideas by `isEligible` against their tallies, rank the eligible
 * by approval ratio (seq tiebreak — the same total, stable order rankIdeas uses),
 * mark the top `buildCapacity` selected, and carry the rest (eligible-but-unbuilt
 * ∪ below-floor) with their votes intact. Mutates nothing — the returned ideas
 * are detached clones, so a caller can read or even mutate the slate without
 * reaching the store. An idea with no tally in the map is treated as zero votes
 * (below-floor, carried). */
export function selectSeason(
  ideas: readonly Idea[],
  tallies: ReadonlyMap<string, IdeaTally>,
  tunables: SelectionTunables = DEFAULT_SELECTION_TUNABLES,
): SelectionResult {
  const zero = (ideaId: string): IdeaTally => ({ ideaId, up: 0, total: 0, ratio: 0 });

  const eligible: RankedIdea[] = [];
  const belowFloor: Idea[] = [];
  for (const idea of ideas) {
    const tally = tallies.get(idea.id) ?? zero(idea.id);
    if (isEligible(tally, tunables)) eligible.push({ idea: jsonClone(idea), tally });
    else belowFloor.push(jsonClone(idea));
  }

  // Rank eligible by approval ratio descending, ties broken by submission order
  // (lower seq first) — the same seq tiebreak rankIdeas uses, so the order is
  // total and stable across rolls.
  eligible.sort((a, b) => b.tally.ratio - a.tally.ratio || a.idea.seq - b.idea.seq);

  const selected = eligible.slice(0, Math.max(0, tunables.buildCapacity));
  const unbuilt = eligible.slice(Math.max(0, tunables.buildCapacity)).map((r) => r.idea);
  // Carry-over = eligible-but-unbuilt (ranked) ∪ below-floor (seq order), every
  // idea's votes intact: the priority queue survives the roll with no daily work.
  const carried = [...unbuilt, ...belowFloor.sort((a, b) => a.seq - b.seq)];

  return { eligible, selected, carried };
}
