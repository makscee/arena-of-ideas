// M3-6 (mission #800): new versions of a Library unit on the vote cards, the
// Library version "unchanged" beside them, and one entry per archetype.
import { describe, expect, it } from "vitest";
import type { CandidateScore, Idea, PlayerRef, UnitId, VoteCard } from "../../../src/mvp/contract.js";
import type { Row } from "../../../src/mvp/units.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { seedUnits } from "./pool.js";
import { describePlan, rotationPlan } from "./rotation.js";
import { mvpRuntime } from "./runtime.js";
import { MemoryMvpStore } from "./store.js";
import { candidateScores, qualified } from "./votes.js";

const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };
const eva: PlayerRef = { id: "p2", name: "Eva", bot: false };
const lev: PlayerRef = { id: "p3", name: "Lev", bot: false };

function world() {
  let n = 7;
  const store = new MemoryMvpStore();
  const now = new Date("2026-10-08T08:00:00.000Z");
  seedUnits(store, now);
  const rt = mvpRuntime({ content: mvpContent(), store, seed: () => (n = (n * 1103515245 + 12345) >>> 0), now: () => now, dev: true });
  for (const p of [maks, eva, lev]) store.addPlayer(p);
  const app = createMvpApp(rt);
  const call = async <T>(method: string, path: string, player: PlayerRef, body?: unknown): Promise<{ status: number; json: T }> => {
    const headers = { "content-type": "application/json", "X-Arena-Player": player.id };
    const res = await app.request(`/api/v1${path}`, { method, headers, ...(body !== undefined ? { body: JSON.stringify(body) } : {}) });
    return { status: res.status, json: (await res.json()) as T };
  };
  const library = store.units({ status: "library" });
  return { rt, store, call, library };
}

type W = ReturnType<typeof world>;

/** `author`'s version of `target` (a Library unit), through the check: its
 * idea in `voting`, its unit a candidate with the target's name. */
function version(w: W, author: PlayerRef, target: UnitId, k: number, change: Partial<Row> = {}): UnitId {
  const t = w.store.unit(target)!;
  const unitId = `${target}-v${k}`;
  w.store.putUnit({ unitId, status: "candidate", row: { ...t.row, pwr: t.row.pwr + k, ...change }, authorId: author.id, origin: "evolution", parentId: target, rootId: t.rootId ?? t.unitId, createdAt: "" });
  const idea: Idea = { ideaId: `idea-${unitId}`, playerId: author.id, text: "make it hit every enemy", state: "voting", createdAt: "", data: { kind: "evolve", target, unitId } };
  w.store.putIdea(idea);
  return unitId;
}

/** A new idea's candidate in `voting`. */
function idea(w: W, author: PlayerRef, from: UnitId, name: string): UnitId {
  const row = { ...w.store.unit(from)!.row, name };
  const unitId = name.toLowerCase();
  w.store.putUnit({ unitId, status: "candidate", row, authorId: author.id, origin: "idea", parentId: null, createdAt: "" });
  w.store.putIdea({ ideaId: `idea-${unitId}`, playerId: author.id, text: "a new one", state: "voting", createdAt: "", data: { unitId } });
  return unitId;
}

/** `won` of `of` votes for the candidate, against live units, from fresh voters. */
function votes(w: W, candidateId: UnitId, won: number, of: number) {
  const others = w.rt.content.units.map((u) => u.id);
  for (let i = 0; i < of; i++) w.store.addVote({ playerId: `voter-${candidateId}-${i}`, candidateId, otherId: others[i]!, pick: i < won ? candidateId : others[i]!, createdAt: "" });
}

const byId = (scores: CandidateScore[]) => new Map(scores.map((s) => [s.unitId, s]));

describe("versions on the vote cards (M3-6)", () => {
  it("puts a version and its Library version, unchanged, on cards; none for an archetype with no proposal", async () => {
    const w = world();
    const [rat, other] = [w.library[0]!.unitId, w.library[1]!.unitId];
    // No proposal: no candidate at all, the Library unit included.
    expect(candidateScores(w.rt)).toEqual([]);
    const v = version(w, maks, rat, 1);
    const scores = candidateScores(w.rt);
    expect(scores.map((s) => [s.unitId, s.kind, s.rootId]).sort()).toEqual([[rat, "unchanged", rat], [v, "version", rat]].sort());
    expect(scores.some((s) => s.unitId === other)).toBe(false);

    // Eva gets both on cards, each tagged.
    const seen = new Map<UnitId, VoteCard["candidateKind"]>();
    let { card } = (await w.call<{ card: VoteCard | null }>("GET", "/votes/next", eva)).json;
    for (let i = 0; card && i < 400; i++) {
      seen.set(card.candidateId, card.candidateKind);
      card = (await w.call<{ card: VoteCard | null }>("POST", "/votes", eva, { candidateId: card.candidateId, otherId: card.otherId, pick: card.candidateId })).json.card;
    }
    expect(Object.fromEntries(seen)).toEqual({ [v]: "version", [rat]: "unchanged" });
  });

  it("never counts an author's vote on their own version; unchanged is open to anyone", async () => {
    const w = world();
    const rat = w.library[0]!.unitId;
    const v = version(w, maks, rat, 1);
    // Maks only ever gets "unchanged".
    for (let i = 0; i < 5; i++) {
      const { card } = (await w.call<{ card: VoteCard | null }>("GET", "/votes/next", maks)).json;
      expect(card).toMatchObject({ candidateId: rat, candidateKind: "unchanged" });
      const res = await w.call("POST", "/votes", maks, { candidateId: card!.candidateId, otherId: card!.otherId, pick: rat });
      expect(res.status).toBe(200);
    }
    const other = w.rt.content.units[0]!.id;
    expect((await w.call("POST", "/votes", maks, { candidateId: v, otherId: other, pick: v })).status).toBe(409);
    expect(w.store.votes({ candidateId: v })).toEqual([]);
    // The Library unit's own author (none for a seed unit) or anyone else may vote on unchanged.
    w.store.putUnit({ ...w.store.unit(rat)!, authorId: lev.id });
    const { card } = (await w.call<{ card: VoteCard | null }>("GET", "/votes/next", lev)).json;
    expect(card?.candidateId).toBeDefined();
  });

  it("two versions and unchanged: the best qualified version is the entry, one per archetype", () => {
    const w = world();
    const rat = w.library[0]!.unitId;
    const a = version(w, maks, rat, 1);
    const b = version(w, eva, rat, 2);
    votes(w, a, 4, 5);
    votes(w, b, 5, 5);
    votes(w, rat, 3, 5);
    const s = byId(candidateScores(w.rt));
    expect([s.get(a)!.qualified, s.get(b)!.qualified, s.get(rat)!.qualified]).toEqual([true, true, true]);
    expect(s.get(b)!.score).toBeGreaterThan(s.get(a)!.score);
    expect([...s.values()].filter((x) => x.entry).map((x) => x.unitId)).toEqual([b]);
    expect(qualified([...s.values()]).map((x) => x.unitId)).toEqual([b]);
  });

  it("unchanged enters only by scoring above every qualified version", () => {
    const w = world();
    const rat = w.library[0]!.unitId;
    const a = version(w, maks, rat, 1);
    const b = version(w, eva, rat, 2);
    votes(w, a, 3, 5); // 0.6 + novelty: qualified
    votes(w, b, 1, 5); // not qualified
    votes(w, rat, 5, 5);
    const s = byId(candidateScores(w.rt));
    expect(s.get(a)!.qualified).toBe(true);
    expect(s.get(b)!.qualified).toBe(false);
    expect(s.get(rat)!.score).toBeGreaterThan(s.get(a)!.score);
    expect(qualified([...s.values()]).map((x) => [x.unitId, x.kind])).toEqual([[rat, "unchanged"]]);
  });

  it("on even votes the version enters: unchanged gets no novelty bonus", () => {
    const w = world();
    const rat = w.library[0]!.unitId;
    // A version close to the Library unit (a little less novel than it):
    // even votes, and unchanged still doesn't win on novelty (#800).
    const a = version(w, maks, rat, 1);
    votes(w, a, 4, 5);
    votes(w, rat, 4, 5);
    const s = byId(candidateScores(w.rt));
    expect(s.get(rat)!.novelty).toBe(0);
    expect(s.get(rat)!.score).toBe(0.8);
    expect(s.get(a)!.score).toBeGreaterThan(s.get(rat)!.score);
    expect(qualified([...s.values()]).map((x) => x.unitId)).toEqual([a]);
  });

  it("keeps each archetype to one entry beside new ideas, every idea its own entry", () => {
    const w = world();
    const [rat, other] = [w.library[0]!.unitId, w.library[1]!.unitId];
    const live = w.rt.content.units[0]!.id;
    const a = version(w, maks, rat, 1);
    const b = version(w, eva, rat, 2);
    const c = version(w, maks, other, 1);
    const x = idea(w, lev, live, "Hedgehog");
    const y = idea(w, eva, live, "Porcupine");
    for (const id of [a, b, c, x, y, rat, other]) votes(w, id, 5, 5);
    const entries = qualified(candidateScores(w.rt));
    const roots = entries.map((e) => e.rootId);
    expect(new Set(roots).size).toBe(roots.length);
    expect(entries.filter((e) => e.kind === "idea").map((e) => e.unitId).sort()).toEqual([x, y].sort());
    expect(entries.filter((e) => e.kind !== "idea").map((e) => e.rootId).sort()).toEqual([rat, other].sort());
  });

  it("has no unchanged while a version of the archetype is live", () => {
    const w = world();
    const rat = w.library[0]!.unitId;
    const a = version(w, maks, rat, 1);
    w.store.putUnit({ ...w.store.unit(a)!, status: "live" });
    w.store.putIdea({ ...w.store.idea(`idea-${a}`)!, state: "live" });
    version(w, eva, rat, 2);
    expect(candidateScores(w.rt).map((s) => s.kind)).toEqual(["version"]);
  });

  it("lists each contest in the dry run, the entry first", () => {
    const w = world();
    const rat = w.library[0]!.unitId;
    const a = version(w, maks, rat, 1);
    const b = version(w, eva, rat, 2);
    votes(w, a, 4, 5);
    votes(w, b, 5, 5);
    votes(w, rat, 2, 5);
    const plan = rotationPlan(w.rt);
    expect(plan.contests).toHaveLength(1);
    expect(plan.contests[0]!.rootId).toBe(rat);
    expect(plan.contests[0]!.candidates.map((s) => s.unitId)).toEqual([b, a, rat]);
    expect(plan.swaps.map((s) => s.entrant.unitId)).toEqual([b]);
    const text = describePlan(plan, w.rt.rules).join("\n");
    expect(text).toContain(`contest ${w.store.unit(rat)!.row.emoji} ${w.store.unit(rat)!.row.name} (archetype ${rat}): ${b} is its entry`);
    expect(text).toContain(`    ${b} version by ${eva.id}: score`);
    expect(text).toMatch(new RegExp(`    ${a} version by ${maks.id}: .*; qualified, beaten`));
    expect(text).toMatch(new RegExp(`    ${rat} unchanged: score .* = 2/5 votes .*; not qualified`));
    expect(text).toContain(`enters ${w.store.unit(rat)!.row.emoji} ${w.store.unit(rat)!.row.name} (${b}, version by ${eva.id}; evolution slot)`);
  });
});
