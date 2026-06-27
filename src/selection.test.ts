import { describe, expect, test } from "vitest";
import type { Idea, VoteMap } from "./ideas.js";
import {
  DEFAULT_SELECTION_TUNABLES,
  isEligible,
  selectSeason,
  talliesOf,
  tallyOf,
  type SelectionTunables,
} from "./selection.js";
import { SELECTION_APPROVAL_RATIO, SELECTION_BUILD_CAPACITY, SELECTION_VOTE_FLOOR } from "./tunables.js";

// ---------------------------------------------------------------------------
// Helpers — build an idea with `up` up-votes and `down` down-votes. Voter keys
// are unique per idea so the vote map carries exactly up+down distinct voters.
// ---------------------------------------------------------------------------

function idea(id: string, seq: number, up: number, down: number): Idea {
  const votes: VoteMap = {};
  for (let i = 0; i < up; i++) votes[`${id}-u${i}`] = "up";
  for (let i = 0; i < down; i++) votes[`${id}-d${i}`] = "down";
  return { id, authorId: "author", text: `idea ${id}`, seq, votes };
}

// ---------------------------------------------------------------------------
// 1. Tally — read up/total/ratio straight off the directional vote map
// ---------------------------------------------------------------------------

describe("tallyOf / talliesOf", () => {
  test("reads up, total, and approval ratio off the vote map", () => {
    expect(tallyOf(idea("a", 0, 3, 1))).toEqual({ ideaId: "a", up: 3, total: 4, ratio: 0.75 });
  });

  test("an unvoted idea is zero/zero, ratio 0 (no votes is no approval)", () => {
    expect(tallyOf(idea("a", 0, 0, 0))).toEqual({ ideaId: "a", up: 0, total: 0, ratio: 0 });
  });

  test("talliesOf keys every idea by id", () => {
    const tallies = talliesOf([idea("a", 0, 2, 0), idea("b", 1, 1, 1)]);
    expect(tallies.get("a")).toEqual({ ideaId: "a", up: 2, total: 2, ratio: 1 });
    expect(tallies.get("b")).toEqual({ ideaId: "b", up: 1, total: 2, ratio: 0.5 });
  });
});

// ---------------------------------------------------------------------------
// 2. Eligibility = FLOOR and APPROVAL, both gates ANDed — each must-fail-first
// ---------------------------------------------------------------------------

describe("isEligible — both gates ANDed", () => {
  const T = DEFAULT_SELECTION_TUNABLES;

  test("both gates clear → eligible", () => {
    // 5 votes (== floor), 4/5 = 0.8 approval (>= 0.6).
    expect(isEligible(tallyOf(idea("a", 0, 4, 1)), T)).toBe(true);
  });

  test("FLOOR must-fail-first: a 100%-approval idea BELOW the floor is ineligible — drop the floor and it wrongly qualifies", () => {
    const noise = tallyOf(idea("a", 0, 1, 0)); // one up-vote: 100% approval, 1 total
    expect(noise.ratio).toBe(1);
    // With the real floor (5) the single-vote noise is rejected …
    expect(isEligible(noise, T)).toBe(false);
    // … and dropping the floor to 0 is exactly the mutation that lets it through,
    // proving the floor gate is load-bearing, not decorative.
    expect(isEligible(noise, { ...T, voteFloor: 0 })).toBe(true);
  });

  test("APPROVAL must-fail-first: a divisive idea ABOVE the floor is ineligible — drop the ratio and it wrongly qualifies", () => {
    const divisive = tallyOf(idea("a", 0, 5, 5)); // 10 votes (>= floor), 0.5 approval (< 0.6)
    expect(divisive.total).toBeGreaterThanOrEqual(T.voteFloor);
    expect(isEligible(divisive, T)).toBe(false);
    // Dropping the ratio gate to 0 lets the divisive 50/50 through — the gate bites.
    expect(isEligible(divisive, { ...T, approvalRatio: 0 })).toBe(true);
  });

  test("the floor is inclusive (total == floor) and the ratio is inclusive (ratio == approvalRatio)", () => {
    // exactly the floor in votes, exactly the ratio threshold (3/5 = 0.6)
    expect(isEligible(tallyOf(idea("a", 0, 3, 2)), T)).toBe(true);
  });

  test("defaults match the shipped tunables", () => {
    expect(DEFAULT_SELECTION_TUNABLES).toEqual({
      voteFloor: SELECTION_VOTE_FLOOR,
      approvalRatio: SELECTION_APPROVAL_RATIO,
      buildCapacity: SELECTION_BUILD_CAPACITY,
    });
  });
});

// ---------------------------------------------------------------------------
// 3. selectSeason — ranking, capacity cut, carry-over preserves votes
// ---------------------------------------------------------------------------

describe("selectSeason", () => {
  const T = DEFAULT_SELECTION_TUNABLES;

  test("eligible ideas rank by approval ratio, seq tiebreak — a higher-ratio idea outranks a higher-vote-count one", () => {
    const fewerButHigher = idea("hi", 0, 5, 0); // 5 votes, ratio 1.0
    const moreButLower = idea("lo", 1, 8, 4); // 12 votes, ratio 0.667
    const result = selectSeason([moreButLower, fewerButHigher], talliesOf([moreButLower, fewerButHigher]), T);
    expect(result.eligible.map((r) => r.idea.id)).toEqual(["hi", "lo"]); // ratio wins over count
  });

  test("seq breaks a ratio tie (lower seq ranks higher)", () => {
    const later = idea("later", 5, 4, 1); // ratio 0.8
    const earlier = idea("earlier", 2, 8, 2); // ratio 0.8 — same ratio, lower seq
    const result = selectSeason([later, earlier], talliesOf([later, earlier]), T);
    expect(result.eligible.map((r) => r.idea.id)).toEqual(["earlier", "later"]);
  });

  test("capacity cut is exact: selected = top buildCapacity of the ranked eligible", () => {
    const ideas = [
      idea("a", 0, 10, 0), // 1.0
      idea("b", 1, 9, 1), // 0.9
      idea("c", 2, 8, 2), // 0.8
      idea("d", 3, 7, 3), // 0.7
    ];
    const result = selectSeason(ideas, talliesOf(ideas), { ...T, buildCapacity: 2 });
    expect(result.selected.map((r) => r.idea.id)).toEqual(["a", "b"]);
    expect(result.selected.length).toBe(2);
    // The rest of the eligible carry over (ranked), votes intact.
    expect(result.carried.map((i) => i.id)).toEqual(["c", "d"]);
  });

  test("carry-over preserves votes: eligible-but-unbuilt AND below-floor ideas stay on the table, vote maps byte-equal", () => {
    const built = idea("built", 0, 9, 0); // eligible, will be selected
    const unbuilt = idea("unbuilt", 1, 6, 1); // eligible, below capacity
    const noise = idea("noise", 2, 1, 0); // below floor
    const divisive = idea("divisive", 3, 3, 3); // above floor, below ratio
    const ideas = [built, unbuilt, noise, divisive];

    const result = selectSeason(ideas, talliesOf(ideas), { ...T, buildCapacity: 1 });

    expect(result.selected.map((r) => r.idea.id)).toEqual(["built"]);
    // every non-selected idea is carried, none destroyed
    expect(new Set(result.carried.map((i) => i.id))).toEqual(new Set(["unbuilt", "noise", "divisive"]));
    // votes intact — each carried idea's vote map deep-equals the original
    for (const original of [unbuilt, noise, divisive]) {
      const carried = result.carried.find((i) => i.id === original.id)!;
      expect(carried.votes).toEqual(original.votes);
    }
  });

  test("pure: the input ideas are untouched (returned entries are detached clones)", () => {
    const original = idea("a", 0, 6, 0);
    const before = JSON.stringify(original);
    const result = selectSeason([original], talliesOf([original]), T);
    result.eligible[0]!.idea.votes["injected"] = "down";
    result.eligible[0]!.idea.text = "mutated";
    expect(JSON.stringify(original)).toBe(before); // the store-side idea is untouched
  });

  test("an idea with no tally entry is treated as below-floor and carried", () => {
    const a = idea("a", 0, 9, 0);
    const orphan = idea("orphan", 1, 9, 0); // would be eligible, but no tally supplied
    const tallies = talliesOf([a]); // orphan deliberately absent
    const result = selectSeason([a, orphan], tallies, T);
    expect(result.selected.map((r) => r.idea.id)).toEqual(["a"]);
    expect(result.carried.map((i) => i.id)).toContain("orphan");
  });

  test("no eligible ideas → empty slate, everything carried", () => {
    const ideas = [idea("a", 0, 1, 0), idea("b", 1, 2, 3)];
    const result = selectSeason(ideas, talliesOf(ideas), T);
    expect(result.eligible).toEqual([]);
    expect(result.selected).toEqual([]);
    expect(result.carried.map((i) => i.id).sort()).toEqual(["a", "b"]);
  });
});
