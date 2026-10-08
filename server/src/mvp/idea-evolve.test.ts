// M3-4 (mission #800): proposing a new version of a Library unit and reading
// it. A proposal spends an idea, skips the archetype stage (the target's name,
// emoji and line), and its readings must differ from the target's shape and be
// true to its line; the picked one is an `evolution` candidate.
import { describe, expect, it } from "vitest";
import { MVP_RULES, type MyIdeasView, type PlayerRef } from "../../../src/mvp/contract.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { sameShape } from "./idea-checks.js";
import { fakeIdeaReader, readingsPrompt, type IdeaReader } from "./idea-reader.js";
import { COULD_NOT, pickReading, readIdeas, TARGET_GONE } from "./idea-reading.js";
import { grantIdea, ideasOf, writeIdea } from "./ideas.js";
import { seedUnits, swapUnit } from "./pool.js";
import { mvpRuntime } from "./runtime.js";
import { MemoryMvpStore } from "./store.js";
import { overnightCheck, type Tuner } from "./votes.js";

const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };
const WANT = "make it hit every enemy instead of only the front one";

const tuner = (pass: boolean): Tuner => async (row) => ({ pass, reason: pass ? "fits" : "too strong in a damage team", row: { ...row, pwr: row.pwr + 1 } });

/** A seeded world with Fighter in the Library, and the API. */
function world(pass = true) {
  const store = new MemoryMvpStore();
  const now = new Date("2026-10-08T08:00:00.000Z");
  seedUnits(store, now);
  swapUnit(store, "fighter", "rat", now);
  const rt = mvpRuntime({ content: mvpContent(), store, now: () => now, rules: MVP_RULES, dev: true, tuner: { night: tuner(pass), dev: tuner(pass) } });
  store.addPlayer(maks);
  const app = createMvpApp(rt);
  const call = async <T>(method: string, path: string, body?: unknown): Promise<{ status: number; json: T }> => {
    const headers = { "content-type": "application/json", "X-Arena-Player": maks.id };
    const res = await app.request(`/api/v1${path}`, { method, headers, ...(body !== undefined ? { body: JSON.stringify(body) } : {}) });
    return { status: res.status, json: (await res.json()) as T };
  };
  const held = () => ideasOf(rt, maks.id).held;
  return { rt, store, call, held };
}

describe("proposing a new version (M3-4)", () => {
  it("is refused for a live unit, an unknown one, without an idea, or outside the idea limits", async () => {
    const w = world();
    expect(w.store.unit("fighter")!.status).toBe("library");
    expect((await w.call("POST", "/ideas", { text: WANT, kind: "evolve", target: "fighter" })).status).toBe(409); // no idea held
    await w.call("POST", "/dev/grant-idea");
    expect(w.held()).toBe(1);
    const live = await w.call<{ error: string }>("POST", "/ideas", { text: WANT, kind: "evolve", target: "rat" });
    expect(live).toMatchObject({ status: 409, json: { error: "Only a unit in the Library can get a new version." } });
    expect((await w.call("POST", "/ideas", { text: WANT, kind: "evolve", target: "nobody" })).status).toBe(404);
    expect((await w.call("POST", "/ideas", { text: "short", kind: "evolve", target: "fighter" })).status).toBe(400);
    expect((await w.call("POST", "/ideas", { text: WANT, kind: "evolve" })).status).toBe(400);
    expect((await w.call("POST", "/ideas", { text: WANT, kind: "fuse", target: "fighter" })).status).toBe(400);
    expect(w.held()).toBe(1);
    expect(w.store.ideas()).toEqual([]);
  });

  it("spends an idea and is stored as an evolve idea for its target", async () => {
    const w = world();
    await w.call("POST", "/dev/grant-idea");
    const res = await w.call<MyIdeasView>("POST", "/ideas", { text: WANT, kind: "evolve", target: "fighter" });
    expect(res.status).toBe(200);
    expect(res.json.ideas.held).toBe(0);
    expect(res.json.sent[0]).toMatchObject({ state: "written", text: WANT, data: { kind: "evolve", target: "fighter" } });
    expect(w.store.ideas({ target: "fighter" })).toHaveLength(1);
  });

  it("goes written → pick-reading → simulating with the fake reader, its candidate a version of the target", async () => {
    const w = world();
    const fighter = w.store.unit("fighter")!;
    await w.call("POST", "/dev/grant-idea");
    const id = (await w.call<MyIdeasView>("POST", "/ideas", { text: WANT, kind: "evolve", target: "fighter" })).json.sent[0]!.ideaId;
    expect(await readIdeas(w.rt, fakeIdeaReader())).toBe(1);
    const read = w.store.idea(id)!;
    expect(read.state).toBe("pick-reading");
    expect(read.data.archetypes).toBeUndefined();
    expect(read.data.archetype).toEqual({ name: fighter.row.name, emoji: fighter.row.emoji, line: fighter.row.archetype });
    expect(read.data.readings).toHaveLength(3);
    const picked = await w.call<MyIdeasView>("POST", `/ideas/${id}/reading`, { index: 1 });
    expect(picked.json.sent[0]).toMatchObject({ state: "simulating", data: { unitId: "fighter-2" } });
    const unit = w.store.unit("fighter-2")!;
    expect(unit).toMatchObject({ status: "candidate", authorId: maks.id, origin: "evolution", parentId: "fighter", rootId: "fighter" });
    expect(unit.row).toMatchObject({ name: fighter.row.name, emoji: fighter.row.emoji, archetype: fighter.row.archetype, when: read.data.readings![1]!.when });
    expect(sameShape(unit.row, fighter.row)).toBe(false);
    expect(w.store.unit("fighter")!.status).toBe("library");
    expect(await overnightCheck(w.rt, w.rt.tuner.dev)).toBe(1);
    expect(w.store.idea(id)!.state).toBe("voting");
    expect(w.store.unit("fighter-2")).toMatchObject({ status: "candidate", parentId: "fighter", rootId: "fighter" });
  });

  it("a version of a version keeps the archetype's root", async () => {
    const w = world();
    const fighter = w.store.unit("fighter")!;
    w.store.putUnit({ ...fighter, unitId: "fighter-2", status: "library", origin: "evolution", parentId: "fighter", rootId: "fighter", row: { ...fighter.row, when: "hurt" } });
    grantIdea(w.rt, maks.id);
    const idea = writeIdea(w.rt, maks.id, WANT, { target: "fighter-2" });
    await readIdeas(w.rt, fakeIdeaReader());
    pickReading(w.rt, maks.id, idea.ideaId, 0);
    expect(w.store.unit("fighter-3")).toMatchObject({ origin: "evolution", parentId: "fighter-2", rootId: "fighter" });
  });

  it("asks the reader for a new version: the line, the current rule and what the player wants", async () => {
    const w = world();
    const seen: string[] = [];
    const fake = fakeIdeaReader();
    const reader: IdeaReader = {
      ...fake,
      async readings(text, a, opts) {
        seen.push(readingsPrompt(text, a, opts));
        return fake.readings(text, a, opts);
      },
    };
    grantIdea(w.rt, maks.id);
    writeIdea(w.rt, maks.id, WANT, { target: "fighter" });
    await readIdeas(w.rt, reader);
    const f = w.store.unit("fighter")!.row;
    expect(seen[0]).toContain(`a new version of ${f.emoji} ${f.name}, line "${f.archetype}"`);
    expect(seen[0]).toMatch(/Its current rule: sleeping "[^"]+"; awoken "[^"]+"/);
    expect(seen[0]).toContain(WANT);
  });
});

describe("a new version's checks (M3-4)", () => {
  it("an unfaithful reading is dropped and retried with the reason; the rest are kept", async () => {
    const w = world();
    const fake = fakeIdeaReader();
    const problems: string[][] = [];
    let asked = 0;
    const reader: IdeaReader = {
      ...fake,
      async readings(text, a, opts) {
        if (opts.problems) problems.push(opts.problems);
        return fake.readings(text, a, opts);
      },
      // The first reading asked about isn't true to the line.
      async faithful() {
        return asked++ === 0 ? { true: false, reason: "a fighter doesn't heal" } : { true: true, reason: "" };
      },
    };
    grantIdea(w.rt, maks.id);
    const idea = writeIdea(w.rt, maks.id, WANT, { target: "fighter" });
    await readIdeas(w.rt, reader);
    expect(problems).toHaveLength(1);
    expect(problems[0]!.join("\n")).toContain(`it isn't true to the line "${w.store.unit("fighter")!.row.archetype}": a fighter doesn't heal`);
    expect(w.store.idea(idea.ideaId)).toMatchObject({ state: "pick-reading" });
    expect(w.store.idea(idea.ideaId)!.data.readings).toHaveLength(3);
  });

  it("the fake finds an 'unfaithful' text untrue: every reading dropped, the proposal fails and is refunded", async () => {
    const w = world();
    grantIdea(w.rt, maks.id);
    const idea = writeIdea(w.rt, maks.id, `${WANT}, an unfaithful one`, { target: "fighter" });
    expect(w.held()).toBe(0);
    await readIdeas(w.rt, fakeIdeaReader());
    expect(w.store.idea(idea.ideaId)).toMatchObject({ state: "failed", data: { failure: COULD_NOT } });
    expect(w.held()).toBe(1);
    expect(w.store.units({ status: "candidate" })).toEqual([]);
  });

  it("a reading with the current version's shape is dropped", async () => {
    const w = world();
    const f = w.store.unit("fighter")!.row;
    const fake = fakeIdeaReader();
    const problems: string[][] = [];
    const reader: IdeaReader = {
      ...fake,
      async readings(text, a, opts) {
        if (opts.problems) problems.push(opts.problems);
        const out = await fake.readings(text, a, opts);
        return opts.problems ? out : { ...out, options: [{ when: f.when, who: f.who, does: f.does, awoken: f.awoken }, ...out.options.slice(0, 2)] };
      },
    };
    grantIdea(w.rt, maks.id);
    const idea = writeIdea(w.rt, maks.id, WANT, { target: "fighter" });
    await readIdeas(w.rt, reader);
    expect(problems[0]).toEqual(["reading 1: it has the current version's shape; change its When, Who or Does"]);
    expect(w.store.idea(idea.ideaId)!.data.readings).toHaveLength(3);
    for (const r of w.store.idea(idea.ideaId)!.data.readings!) expect(r.when === f.when && r.who === f.who && r.does === f.does).toBe(false);
  });

  it("a proposal that fails the overnight check is refunded, the target stays in the Library", async () => {
    const w = world(false);
    grantIdea(w.rt, maks.id);
    const idea = writeIdea(w.rt, maks.id, WANT, { target: "fighter" });
    await readIdeas(w.rt, fakeIdeaReader());
    pickReading(w.rt, maks.id, idea.ideaId, 0);
    expect(w.held()).toBe(0);
    expect(await overnightCheck(w.rt, w.rt.tuner.dev)).toBe(1);
    expect(w.store.idea(idea.ideaId)).toMatchObject({ state: "failed" });
    expect(w.store.idea(idea.ideaId)!.data.failure).toContain("too strong in a damage team");
    expect(w.store.unit("fighter-2")!.status).toBe("rejected");
    expect(w.store.unit("fighter")!.status).toBe("library");
    expect(w.held()).toBe(1);
  });

  it("a proposal whose target is gone fails and is refunded", async () => {
    const w = world();
    grantIdea(w.rt, maks.id);
    const idea = writeIdea(w.rt, maks.id, WANT, { target: "fighter" });
    w.store.putIdea({ ...idea, data: { ...idea.data, target: "nobody" } });
    await readIdeas(w.rt, fakeIdeaReader());
    expect(w.store.idea(idea.ideaId)).toMatchObject({ state: "failed", data: { failure: TARGET_GONE } });
    expect(w.held()).toBe(1);
  });

  it("the new-words log takes a proposal's missing words and near misses", async () => {
    const w = world();
    grantIdea(w.rt, maks.id);
    const idea = writeIdea(w.rt, maks.id, "let it steal gold from the weakest enemy", { target: "fighter" });
    await readIdeas(w.rt, fakeIdeaReader());
    expect(w.store.wordRequests().map((r) => r.word)).toEqual(["steal gold", "the weakest enemy (read as random)"]);
    expect(w.store.idea(idea.ideaId)!.state).toBe("pick-reading");
  });
});
