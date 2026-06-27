// Ideas-table tests (#076 slice 1, evolved to directional votes in #082 slice 1)
// — the store interface and its in-memory / localStorage-backed backings.
// Storage is injected, so a Map-backed stub drives the exact code main.ts wires
// to window.localStorage. Parity bar (the ladder's): an ideas table behind
// localStorage and an InMemory one, given the same drives, hold the same ranked
// list. Votes are now DIRECTIONAL and SWITCH-ONLY (up/down, never back to
// neutral) — the toggle-off case is gone, replaced by switch + no-remove cases.

import { describe, expect, test } from "vitest";
import {
  InMemoryIdeaStore,
  emptyIdeasData,
  parseIdeasData,
  type IdeaStore,
} from "../src/index.js";
import { openLocalIdeas, serializeIdeas, type KVStorage } from "./ideas-store.js";

function fakeStorage(): KVStorage {
  const m = new Map<string, string>();
  return {
    getItem: (k) => m.get(k) ?? null,
    setItem: (k, v) => void m.set(k, v),
    removeItem: (k) => void m.delete(k),
  };
}

/** The top-ranked idea, asserted present — the store under test is never empty
 * where this is called, so an absent top is a test bug, surfaced loudly. */
function top(store: IdeaStore) {
  const first = store.list()[0];
  if (first === undefined) throw new Error("expected at least one idea on the table");
  return first;
}

// Every contract test runs against both backings, so they can never silently
// disagree on the IdeaStore semantics.
const backings: Array<[string, () => IdeaStore]> = [
  ["InMemoryIdeaStore", () => new InMemoryIdeaStore()],
  ["openLocalIdeas (localStorage)", () => openLocalIdeas(fakeStorage())],
];

describe.each(backings)("IdeaStore: %s", (_name, open) => {
  test("submit: a submitted idea appears in the list, trimmed, with no votes", () => {
    const store = open();
    const idea = store.submit("  add a spectate mode  ", "ada");
    expect(idea.text).toBe("add a spectate mode"); // trimmed
    expect(idea.authorId).toBe("ada");
    expect(idea.votes).toEqual({}); // an empty directional vote map
    expect(store.list().map((i) => i.id)).toEqual([idea.id]);
  });

  test("submit: empty / whitespace-only text is rejected", () => {
    const store = open();
    expect(() => store.submit("", "ada")).toThrow(/must not be empty/);
    expect(() => store.submit("   ", "ada")).toThrow(/must not be empty/);
    expect(store.list()).toEqual([]); // nothing slipped onto the table
  });

  test("cast up then switch to down: one directional vote, flipped — never two", () => {
    const store = open();
    const idea = store.submit("ranked seasons", "ada");
    store.castVote(idea.id, "bob", "up");
    expect(top(store).votes).toEqual({ bob: "up" }); // voted up
    store.castVote(idea.id, "bob", "down");
    expect(top(store).votes).toEqual({ bob: "down" }); // FLIPPED to down, still one vote
  });

  test("cast same direction twice: a no-op — never a second entry", () => {
    const store = open();
    const idea = store.submit("new keyword: cleave", "ada");
    store.castVote(idea.id, "bob", "up");
    store.castVote(idea.id, "bob", "up"); // same direction → no-op
    expect(top(store).votes).toEqual({ bob: "up" }); // bob counted exactly once
  });

  test("switch, NEVER remove: there is no operation that returns a player to neutral", () => {
    const store = open();
    const idea = store.submit("permanent participation", "ada");
    store.castVote(idea.id, "bob", "up");
    store.castVote(idea.id, "bob", "down");
    store.castVote(idea.id, "bob", "up"); // flip back — still present, never gone
    // Every cast leaves bob in the map. The store surface offers no un-vote:
    // castVote only adds or flips, so a vote, once cast, is permanent.
    expect("bob" in top(store).votes).toBe(true);
    expect(Object.keys(top(store).votes)).toEqual(["bob"]); // one vote, switch-only
  });

  test("two distinct voters each hold their own directional vote", () => {
    const store = open();
    const idea = store.submit("two voters", "ada");
    store.castVote(idea.id, "bob", "up");
    store.castVote(idea.id, "cleo", "down");
    expect(top(store).votes).toEqual({ bob: "up", cleo: "down" }); // keys sorted, each its own dir
  });

  test("ranked ordering: up-votes raise, down-votes lower; ties break by submission order", () => {
    const store = open();
    const low = store.submit("idea-low", "ada"); // seq 0
    const high = store.submit("idea-high", "ada"); // seq 1
    const tieEarly = store.submit("idea-tie-early", "ada"); // seq 2
    const tieLate = store.submit("idea-tie-late", "ada"); // seq 3

    store.castVote(high.id, "bob", "up");
    store.castVote(high.id, "cleo", "up"); // high → score +2
    store.castVote(low.id, "bob", "up"); // low → score +1
    store.castVote(tieEarly.id, "cleo", "up"); // tieEarly → score +1
    store.castVote(tieLate.id, "dan", "down"); // tieLate → score −1 (a down LOWERS it)

    // high (+2) first; then the two +1 ideas by seq (low seq0 before tieEarly
    // seq2); tieLate (−1) ranks LAST — a down-vote pushes it below the unvoted-
    // equivalent neutral line.
    expect(store.list().map((i) => i.id)).toEqual([high.id, low.id, tieEarly.id, tieLate.id]);
  });

  test("down-vote lowers an idea below a neutral one", () => {
    const store = open();
    const neutral = store.submit("neutral idea", "ada"); // seq 0, no votes → score 0
    const downed = store.submit("downed idea", "ada"); // seq 1
    store.castVote(downed.id, "bob", "down"); // score −1
    // Despite its earlier-or-later seq, a negative score ranks behind score 0.
    expect(store.list().map((i) => i.id)).toEqual([neutral.id, downed.id]);
  });

  test("votedCount: the currency counts distinct ideas voted, unchanged by a flip or re-cast", () => {
    const store = open();
    const a = store.submit("idea A", "ada");
    const b = store.submit("idea B", "ada");
    const c = store.submit("idea C", "ada");
    expect(store.votedCount("bob")).toBe(0); // voted on nothing yet
    store.castVote(a.id, "bob", "up");
    store.castVote(b.id, "bob", "down");
    expect(store.votedCount("bob")).toBe(2); // two distinct ideas
    store.castVote(a.id, "bob", "down"); // flip a's direction
    expect(store.votedCount("bob")).toBe(2); // a flip does not change the count
    store.castVote(b.id, "bob", "down"); // re-cast the same direction
    expect(store.votedCount("bob")).toBe(2); // a no-op re-cast does not change it
    store.castVote(c.id, "bob", "up"); // a third distinct idea
    expect(store.votedCount("bob")).toBe(3);
    // Another player's footprint is independent.
    expect(store.votedCount("cleo")).toBe(0);
  });

  test("removeOwn: the author removes their idea; a non-author is refused", () => {
    const store = open();
    const idea = store.submit("kept", "ada");
    expect(() => store.removeOwn(idea.id, "bob")).toThrow(/only its author/);
    expect(store.list().map((i) => i.id)).toEqual([idea.id]); // refusal left it on the table
    store.removeOwn(idea.id, "ada");
    expect(store.list()).toEqual([]); // the author's own removal took it off
  });

  test("unknown ideaId is refused loudly on vote and remove", () => {
    const store = open();
    expect(() => store.castVote("idea-999", "bob", "up")).toThrow(/no idea with id/);
    expect(() => store.removeOwn("idea-999", "ada")).toThrow(/no idea with id/);
  });

  test("list() returns detached copies — mutating a returned idea never corrupts the store", () => {
    const store = open();
    const a = store.submit("idea A", "ada"); // seq 0, 0 votes
    const b = store.submit("idea B", "bob"); // seq 1, 0 votes
    // A reader (the renderer) reaches into a returned idea's votes and mutates
    // the map. If list() leaked the store's own maps, this would give B a vote
    // inside the store and swap the ranking on the next read.
    const returned = store.list();
    returned[1]!.votes["x"] = "up";
    const after = store.list();
    expect(after.map((i) => i.id)).toEqual([a.id, b.id]); // order unchanged (tie → seq)
    expect(after.map((i) => i.votes)).toEqual([{}, {}]); // votes unchanged
  });
});

describe("openLocalIdeas (localStorage backing)", () => {
  test("write-through: a reopened table holds every idea and vote", () => {
    const storage = fakeStorage();
    const first = openLocalIdeas(storage);
    const a = first.submit("idea A", "ada");
    const b = first.submit("idea B", "bob");
    first.castVote(b.id, "ada", "up"); // B outranks A

    const reopened = openLocalIdeas(storage); // a page reload
    expect(reopened.list().map((i) => i.id)).toEqual([b.id, a.id]);
    expect(top(reopened).votes).toEqual({ ada: "up" });
  });

  test("parity: localStorage and InMemory, same drives → same ranked list", () => {
    const local = openLocalIdeas(fakeStorage());
    const inMemory = new InMemoryIdeaStore();
    for (const store of [local, inMemory]) {
      const x = store.submit("alpha", "ada");
      const y = store.submit("beta", "bob");
      store.castVote(y.id, "cleo", "up");
      store.castVote(x.id, "cleo", "down");
      store.castVote(y.id, "dan", "up"); // beta: +2, alpha: −1
    }
    expect(local.list()).toEqual(inMemory.list());
    expect(local.votedCount("cleo")).toBe(inMemory.votedCount("cleo"));
  });

  test("corrupt stored ideas JSON throws loudly, never a silent fresh table", () => {
    const storage = fakeStorage();
    storage.setItem("aoi.ideas.v2", "not json");
    expect(() => openLocalIdeas(storage)).toThrow(/not valid JSON/);
  });

  test("a present-but-malformed table (no ideas array) is refused", () => {
    const storage = fakeStorage();
    storage.setItem("aoi.ideas.v2", JSON.stringify({ nextSeq: 0 }));
    expect(() => openLocalIdeas(storage)).toThrow(/not an ideas table/);
  });

  test("a stale v1 blob is ignored (the key bumped to v2) — a fresh table opens", () => {
    const storage = fakeStorage();
    // A v1 blob (flat array votes) lives under the OLD key; v2 reads a new key,
    // so the stale shape is simply not loaded — no throw, a fresh table.
    storage.setItem("aoi.ideas.v1", JSON.stringify({ ideas: [{ id: "idea-0", authorId: "ada", text: "old", seq: 0, votes: ["bob"] }], nextSeq: 1 }));
    const store = openLocalIdeas(storage);
    expect(store.list()).toEqual([]); // v1 dropped; v2 starts fresh
  });
});

describe("serialized shape round-trips", () => {
  test("serialize → parse deep-equals the original table, with votes and counter intact", () => {
    const storage = fakeStorage();
    const store = openLocalIdeas(storage);
    const a = store.submit("first idea", "ada");
    const b = store.submit("second idea", "bob");
    store.castVote(a.id, "bob", "up");
    store.castVote(a.id, "cleo", "down");
    store.castVote(b.id, "ada", "up");

    // The stored JSON is exactly what the backing wrote; round-trip it.
    const raw = storage.getItem("aoi.ideas.v2")!;
    const data = parseIdeasData(raw, "round-trip test");
    expect(serializeIdeas(data)).toBe(raw); // re-serialize is byte-equal
    expect(parseIdeasData(serializeIdeas(data), "round-trip test")).toEqual(data); // deep-equal

    // And the counter survived: a reopened store mints the next id, never a collision.
    const reopened = openLocalIdeas(storage);
    const c = reopened.submit("third idea", "cleo");
    expect(c.id).not.toBe(a.id);
    expect(c.id).not.toBe(b.id);
  });

  test("emptyIdeasData is the fresh-table baseline and round-trips", () => {
    const data = emptyIdeasData();
    expect(data).toEqual({ ideas: [], nextSeq: 0 });
    expect(parseIdeasData(serializeIdeas(data), "empty")).toEqual(data);
  });
});
