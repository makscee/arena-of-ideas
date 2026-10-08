// M4-5 (mission #810): ideas in Russian. The reader takes an idea or a
// version proposal in Russian or English; every archetype comes back with its
// name and line in both (English the unit's Row, Russian kept beside it in
// StoredUnit.texts.ru), both checked; the parts it quotes stay in the idea's
// own words; vote cards and the Library carry the Russian.
import { describe, expect, it } from "vitest";
import { MVP_RULES, type LibraryView, type MyIdeasView, type PlayerRef, type VoteCard } from "../../../src/mvp/contract.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { archetypeDraftProblems, type ArchetypeDraft } from "./idea-checks.js";
import { ARCHETYPES_SCHEMA, fakeIdeaReader, readerSystemPrompt, type IdeaReader, type ReaderAnswer } from "./idea-reader.js";
import { pickReading, readIdeas } from "./idea-reading.js";
import { grantIdea, writeIdea } from "./ideas.js";
import { seedUnits, swapUnit } from "./pool.js";
import { mvpRuntime } from "./runtime.js";
import { MemoryMvpStore, nameKey } from "./store.js";
import { overnightCheck, type Tuner } from "./votes.js";

const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };
const eva: PlayerRef = { id: "p2", name: "Eva", bot: false };
const HOG_RU = "ёж, который травит того, кто его бьёт";
const CYR = /[А-Яа-яЁё]/;

const tuner: Tuner = async (row) => ({ pass: true, reason: "fits", row: { ...row, pwr: row.pwr + 1 } });

/** A seeded world (Fighter in the Library, with a Russian name) and the API. */
function world() {
  const store = new MemoryMvpStore();
  const now = new Date("2026-10-08T08:00:00.000Z");
  seedUnits(store, now);
  swapUnit(store, "fighter", "rat", now);
  store.putUnit({ ...store.unit("fighter")!, texts: { ru: { name: "Боец", line: "Простой боец, что бьёт первого врага." } } });
  const rt = mvpRuntime({ content: mvpContent(), store, now: () => now, rules: MVP_RULES, dev: true, tuner: { night: tuner, dev: tuner } });
  store.addPlayer(maks);
  store.addPlayer(eva);
  const app = createMvpApp(rt);
  const call = async <T>(method: string, path: string, body?: unknown, who = maks): Promise<{ status: number; json: T }> => {
    const headers = { "content-type": "application/json", "X-Arena-Player": who.id, "Accept-Language": "ru" };
    const res = await app.request(`/api/v1${path}`, { method, headers, ...(body !== undefined ? { body: JSON.stringify(body) } : {}) });
    return { status: res.status, json: (await res.json()) as T };
  };
  return { rt, store, call };
}

describe("a Russian idea with the fake reader (M4-5)", () => {
  it("goes end to end to simulating, every archetype named in both languages, the unit keeping the Russian", async () => {
    const w = world();
    await w.call("POST", "/dev/grant-idea");
    const id = (await w.call<MyIdeasView>("POST", "/ideas", { text: HOG_RU })).json.sent[0]!.ideaId;
    expect(await readIdeas(w.rt, fakeIdeaReader())).toBe(1);
    const read = (await w.call<MyIdeasView>("GET", "/ideas")).json.sent[0]!;
    expect(read.state).toBe("pick-archetype");
    const archetypes = read.data.archetypes!;
    expect(archetypes).toHaveLength(3);
    for (const a of archetypes) {
      expect(a.name).toMatch(/^[A-Z][A-Za-z ]+$/);
      expect(a.texts?.ru?.name).toMatch(/^[А-ЯЁ][а-яё]+( [А-Яа-яЁё]+)*$/);
      expect(a.texts?.ru?.line).toMatch(CYR);
    }
    expect(archetypes.map((a) => [a.name, a.texts!.ru!.name])).toEqual([
      ["Travit", "Травит"],
      ["Travitling", "Малый Травит"],
      ["Old Travit", "Старый Травит"],
    ]);
    expect((await w.call("POST", `/ideas/${id}/archetype`, { index: 2 })).status).toBe(200);
    await readIdeas(w.rt, fakeIdeaReader());
    const readings = w.store.idea(id)!;
    expect(readings.state).toBe("pick-reading");
    // «кто его бьёт» is the attacker, «травит» names Poison: that reading comes first.
    expect(readings.data.readings![0]).toMatchObject({ who: "attacker", does: "Poison 1" });
    const picked = await w.call<MyIdeasView>("POST", `/ideas/${id}/reading`, { index: 0 });
    expect(picked.json.sent[0]).toMatchObject({ state: "simulating", data: { archetype: { name: "Old Travit", texts: { ru: { name: "Старый Травит" } } } } });
    const unit = w.store.unit(picked.json.sent[0]!.data.unitId!)!;
    expect(unit.row.name).toBe("Old Travit");
    expect(unit.row.archetype).not.toMatch(CYR);
    expect(unit.texts).toEqual({ ru: archetypes[2]!.texts!.ru });
    // The vote card carries it to other players.
    expect(await overnightCheck(w.rt, w.rt.tuner.dev)).toBe(1);
    const card = (await w.call<{ card: VoteCard | null }>("GET", "/votes/next", undefined, eva)).json.card!;
    const cand = card.units.find((u) => u.id === card.candidateId)!;
    expect(cand.texts?.ru?.name).toBe("Старый Травит");
  });

  it("names an English idea in Russian too, its English names unchanged", async () => {
    const w = world();
    grantIdea(w.rt, maks.id);
    const idea = writeIdea(w.rt, maks.id, "a hedgehog that punishes whoever hits it");
    await readIdeas(w.rt, fakeIdeaReader());
    const a = w.store.idea(idea.ideaId)!.data.archetypes!;
    expect(a.map((x) => x.name)).toEqual(["Hedgehog", "Hedgehogling", "Old Hedgehog"]);
    expect(a.map((x) => x.texts?.ru?.name)).toEqual(["Хедгехог", "Малый Хедгехог", "Старый Хедгехог"]);
  });

  it("keeps the player's own Russian words in the can't-express and near-miss logs", async () => {
    const w = world();
    grantIdea(w.rt, maks.id);
    const text = "вор, который крадёт золото у врагов. Бьёт слабейшего врага";
    const idea = writeIdea(w.rt, maks.id, text);
    const reader = fakeIdeaReader();
    await readIdeas(w.rt, reader);
    const after = w.store.idea(idea.ideaId)!;
    expect(after.data.cantExpress).toEqual(["который крадёт золото у врагов"]);
    w.store.putIdea({ ...after, state: "reading", data: { ...after.data, archetype: after.data.archetypes![0]! } });
    await readIdeas(w.rt, reader);
    expect(w.store.wordRequests().map((r) => [r.word, r.part])).toEqual([
      ["steal gold", "который крадёт золото у врагов"],
      ["самый слабый враг (read as random)", "Бьёт слабейшего врага"],
    ]);
  });

  it("reads a Russian version proposal, its archetype and candidate keeping the target's Russian name", async () => {
    const w = world();
    grantIdea(w.rt, maks.id);
    const idea = writeIdea(w.rt, maks.id, "пусть бьёт всех врагов, а не только первого", { target: "fighter" });
    await readIdeas(w.rt, fakeIdeaReader());
    const read = w.store.idea(idea.ideaId)!;
    expect(read.state).toBe("pick-reading");
    expect(read.data.archetype).toMatchObject({ name: "Fighter", texts: { ru: { name: "Боец" } } });
    pickReading(w.rt, maks.id, idea.ideaId, 0);
    expect(w.store.unit("fighter-2")).toMatchObject({ row: { name: "Fighter" }, texts: { ru: { name: "Боец" } } });
    // The Library shows the target's Russian name to the proposal screens.
    const lib = (await w.call<LibraryView>("GET", "/library")).json;
    expect(lib.units.find((l) => l.unit.id === "fighter")?.unit.texts?.ru?.name).toBe("Боец");
  });
});

/** A reader that answers archetypes from scripts. */
function scripted(answers: ReaderAnswer<ArchetypeDraft>[]) {
  const problems: (string[] | undefined)[] = [];
  const reader: IdeaReader = {
    kind: "scripted",
    async archetypes(_t, opts) {
      problems.push(opts.problems);
      return answers.shift()!;
    },
    async readings() {
      throw new Error("not here");
    },
    faithful: async () => ({ true: true, reason: "" }),
  };
  return { reader, problems };
}

const draft = (name: string, ru?: { name: string; line: string }): ArchetypeDraft => ({ name, emoji: "🦔", line: `A ${name.toLowerCase()} with spines for every foe.`, ...(ru ? { ru } : {}) });

describe("the Russian name and line are checked like the English (M4-5)", () => {
  it("refuses a draft without Russian, with Latin, a crude or a taken Russian name", () => {
    const taken = new Set([nameKey("Боец")]);
    const ok = { name: "Ёжик", line: "Ёжик с иглами для каждого врага." };
    expect(archetypeDraftProblems(draft("Spinehog", ok), [], taken)).toEqual([]);
    expect(archetypeDraftProblems(draft("Spinehog"), [], taken)).toEqual(['"Spinehog": in Russian, the Russian name and line are missing']);
    expect(archetypeDraftProblems(draft("Spinehog", { ...ok, name: "Spinehog" }), [], taken)[0]).toMatch(/Russian words/);
    expect(archetypeDraftProblems(draft("Spinehog", { ...ok, name: "Ёбарь" }), [], taken)[0]).toMatch(/crude/);
    expect(archetypeDraftProblems(draft("Spinehog", { ...ok, name: "БОЕЦ" }), [], taken)[0]).toMatch(/already has this Russian name/);
    expect(archetypeDraftProblems(draft("Spinehog", { ...ok, line: "Spines for every foe." }), [], taken)[0]).toMatch(/in Russian/);
    // Without takenRu (the live units' own check) Russian isn't asked for.
    expect(archetypeDraftProblems(draft("Spinehog"), [])).toEqual([]);
  });

  it("retries the reader once when a Russian name is taken by a unit or another option", async () => {
    const w = world();
    grantIdea(w.rt, maks.id);
    const idea = writeIdea(w.rt, maks.id, HOG_RU);
    const { reader, problems } = scripted([
      { options: [draft("Spinehog", { name: "Боец", line: "Ёж с иглами." }), draft("Quillpig", { name: "Иглач", line: "Иглач с иглами." }), draft("Thornback", { name: "Иглач", line: "Колючий." })], cantExpress: [] },
      { options: [draft("Thornback", { name: "Колючка", line: "Колючка с шипами." }), draft("Burrhog", { name: "Репейник", line: "Репейник с шипами." })], cantExpress: [] },
    ]);
    await readIdeas(w.rt, reader);
    expect(problems[1]).toEqual([
      '"Spinehog": in Russian, another unit already has this Russian name',
      '"Thornback": in Russian, another unit already has this Russian name',
    ]);
    expect(w.store.idea(idea.ideaId)!.data.archetypes!.map((a) => a.texts!.ru!.name)).toEqual(["Иглач", "Колючка", "Репейник"]);
  });
});

describe("the claude -p reader in Russian (M4-5)", () => {
  it("asks for both languages and keeps quotes in the idea's own", () => {
    const sys = readerSystemPrompt([]);
    expect(sys).toContain("English or in Russian");
    expect(sys).toContain("in the idea's own language");
    expect(ARCHETYPES_SCHEMA.properties.archetypes.items.required).toContain("ru");
  });
});
