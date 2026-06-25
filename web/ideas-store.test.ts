// Ideas-table tests (#076 slice 1) — the store interface and its in-memory /
// localStorage-backed backings. Storage is injected, so a Map-backed stub
// drives the exact code main.ts wires to window.localStorage. Parity bar (the
// ladder's): an ideas table behind localStorage and an InMemory one, given the
// same drives, hold the same ranked list.

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
    expect(idea.votes).toEqual([]);
    expect(store.list().map((i) => i.id)).toEqual([idea.id]);
  });

  test("submit: empty / whitespace-only text is rejected", () => {
    const store = open();
    expect(() => store.submit("", "ada")).toThrow(/must not be empty/);
    expect(() => store.submit("   ", "ada")).toThrow(/must not be empty/);
    expect(store.list()).toEqual([]); // nothing slipped onto the table
  });

  test("vote toggle: a vote then an un-vote returns the count to zero", () => {
    const store = open();
    const idea = store.submit("ranked seasons", "ada");
    store.toggleVote(idea.id, "bob");
    expect(top(store).votes).toEqual(["bob"]); // voted
    store.toggleVote(idea.id, "bob");
    expect(top(store).votes).toEqual([]); // un-voted, count back to zero
  });

  test("idempotent re-vote: votes are a set — re-voting the same way never double-counts", () => {
    const store = open();
    const idea = store.submit("new keyword: cleave", "ada");
    store.toggleVote(idea.id, "bob"); // add
    store.toggleVote(idea.id, "cleo"); // a second, distinct voter
    store.toggleVote(idea.id, "bob"); // toggle bob OFF (not a double-add)
    store.toggleVote(idea.id, "bob"); // toggle bob back ON
    // bob counted exactly once despite three toggles; cleo once.
    expect(top(store).votes).toEqual(["bob", "cleo"]);
  });

  test("ranked ordering: more votes ranks higher; ties break by submission order", () => {
    const store = open();
    const low = store.submit("idea-low", "ada"); // seq 0, 1 vote
    const high = store.submit("idea-high", "ada"); // seq 1, 2 votes
    const tieEarly = store.submit("idea-tie-early", "ada"); // seq 2, 1 vote
    const tieLate = store.submit("idea-tie-late", "ada"); // seq 3, 1 vote

    store.toggleVote(high.id, "bob");
    store.toggleVote(high.id, "cleo"); // high → 2 votes
    store.toggleVote(low.id, "bob"); // low → 1 vote
    store.toggleVote(tieEarly.id, "cleo"); // tieEarly → 1 vote
    store.toggleVote(tieLate.id, "dan"); // tieLate → 1 vote

    // high first (2 votes); then the three 1-vote ideas in submission order;
    // the unrelated tiebreak (low seq 0 before tieEarly seq 2 before tieLate seq 3).
    expect(store.list().map((i) => i.id)).toEqual([high.id, low.id, tieEarly.id, tieLate.id]);
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
    expect(() => store.toggleVote("idea-999", "bob")).toThrow(/no idea with id/);
    expect(() => store.removeOwn("idea-999", "ada")).toThrow(/no idea with id/);
  });
});

describe("openLocalIdeas (localStorage backing)", () => {
  test("write-through: a reopened table holds every idea and vote", () => {
    const storage = fakeStorage();
    const first = openLocalIdeas(storage);
    const a = first.submit("idea A", "ada");
    const b = first.submit("idea B", "bob");
    first.toggleVote(b.id, "ada"); // B outranks A

    const reopened = openLocalIdeas(storage); // a page reload
    expect(reopened.list().map((i) => i.id)).toEqual([b.id, a.id]);
    expect(top(reopened).votes).toEqual(["ada"]);
  });

  test("parity: localStorage and InMemory, same drives → same ranked list", () => {
    const local = openLocalIdeas(fakeStorage());
    const inMemory = new InMemoryIdeaStore();
    for (const store of [local, inMemory]) {
      const x = store.submit("alpha", "ada");
      const y = store.submit("beta", "bob");
      store.toggleVote(y.id, "cleo");
      store.toggleVote(x.id, "cleo");
      store.toggleVote(y.id, "dan"); // beta: 2 votes, alpha: 1
    }
    expect(local.list()).toEqual(inMemory.list());
  });

  test("corrupt stored ideas JSON throws loudly, never a silent fresh table", () => {
    const storage = fakeStorage();
    storage.setItem("aoi.ideas.v1", "not json");
    expect(() => openLocalIdeas(storage)).toThrow(/not valid JSON/);
  });

  test("a present-but-malformed table (no ideas array) is refused", () => {
    const storage = fakeStorage();
    storage.setItem("aoi.ideas.v1", JSON.stringify({ nextSeq: 0 }));
    expect(() => openLocalIdeas(storage)).toThrow(/not an ideas table/);
  });
});

describe("serialized shape round-trips", () => {
  test("serialize → parse deep-equals the original table, with votes and counter intact", () => {
    const storage = fakeStorage();
    const store = openLocalIdeas(storage);
    const a = store.submit("first idea", "ada");
    const b = store.submit("second idea", "bob");
    store.toggleVote(a.id, "bob");
    store.toggleVote(a.id, "cleo");
    store.toggleVote(b.id, "ada");

    // The stored JSON is exactly what the backing wrote; round-trip it.
    const raw = storage.getItem("aoi.ideas.v1")!;
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
