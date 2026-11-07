// Slice C (arena#37): a new unit gets its sigil spec from its LLM; the server
// validates it, asks once more with the problems, falls back to the generic
// sigil. Fake LLMs only: no test calls a model.
import { describe, expect, it } from "vitest";
import { MVP_RULES } from "../../../src/mvp/contract.js";
import { Candle, FIXTURE_ABILITIES, FIXTURE_STATUSES, Hourglass, Mirror, Nautilus } from "../../../src/words/fixtures.js";
import { plain, rule } from "../../../src/words/examples.js";
import type { UnitDef } from "../../../src/types.js";
import { FALLBACK_SPEC } from "../../../mobile/ui/sigil/fallback.js";
import { validate } from "../../../mobile/ui/sigil/spec.js";
import { cyrillic } from "./translate.js";
import { designOf } from "./night-codex.js";
import { runBatch, pickPairs, type Field, type Parent } from "./night-batch.js";
import { claudeDesigner, DESIGN_SCHEMA, designerSystemPrompt, fakeDesigner, PORTRAIT_SYSTEM, portraitPrompt } from "./night-designer.js";
import type { ArchetypeDraft, ReadingDraft } from "./idea-checks.js";
import { ARCHETYPES_SCHEMA, archetypesPrompt, fakeIdeaReader, portraitPrompt as ideaPortraitPrompt, readerSystemPrompt, type IdeaReader, type ReaderAnswer } from "./idea-reader.js";
import { pickArchetype, pickReading, readIdeas } from "./idea-reading.js";
import { grantIdea, writeIdea } from "./ideas.js";
import { PORTRAIT_SCHEMA, portraitProblems, resolvePortrait, sigilBrief } from "./portrait.js";
import { seedUnits } from "./pool.js";
import { MemoryMvpStore } from "./store.js";

const GOOD = { body: [{ shape: "circle", r: 0.6, fill: "ember" }, { shape: "petal", len: "0.5+0.1*beat", fill: "gold" }], mark: [{ shape: "ring", r: "0.7+0.05*aw", stroke: "paper" }] };
const OTHER = { body: [{ shape: "rect", w: 0.8, h: 0.8, rad: 0.1, fill: { grad: ["frost", "night"] } }] };
const BAD = { body: [{ shape: "blob", r: 0.5 }] }; // unknown shape
const HOSTILE = { body: [{ shape: "circle", r: "constructor.constructor('return process')().exit()", fill: "ember" }] };

describe("resolvePortrait", () => {
  it("keeps a good spec and never asks again", async () => {
    let asked = 0;
    const got = await resolvePortrait(GOOD, async () => (asked++, OTHER));
    expect(got).toEqual(GOOD);
    expect(asked).toBe(0);
  });

  it("a bad spec, then a good one: asks once with the problem list and keeps the second", async () => {
    const seen: string[][] = [];
    const got = await resolvePortrait(BAD, async (problems) => (seen.push(problems), OTHER));
    expect(got).toEqual(OTHER);
    expect(seen).toHaveLength(1);
    expect(seen[0]!.join(" ")).toMatch(/unknown shape blob/);
  });

  it("bad twice: the generic sigil, and only two tries", async () => {
    let asked = 0;
    const logs: string[] = [];
    const got = await resolvePortrait(BAD, async () => (asked++, HOSTILE), (l) => logs.push(l));
    expect(got).toEqual(FALLBACK_SPEC);
    expect(got).not.toBe(FALLBACK_SPEC); // a copy
    expect(asked).toBe(1);
    expect(logs.join(" ")).toMatch(/refused twice/);
    expect(validate(got).ok).toBe(true);
  });

  it("a second try that throws, or no way to ask, falls back too", async () => {
    expect(await resolvePortrait(BAD, async () => { throw new Error("timeout"); })).toEqual(FALLBACK_SPEC);
    expect(await resolvePortrait(undefined, undefined)).toEqual(FALLBACK_SPEC);
    expect(await resolvePortrait("a string", undefined)).toEqual(FALLBACK_SPEC);
  });

  it("a hostile expression is refused by the grammar, never run", async () => {
    const problems = portraitProblems(HOSTILE);
    expect(problems.length).toBeGreaterThan(0);
    expect(problems.join(" ")).toMatch(/constructor|unexpected|unknown/);
    expect(await resolvePortrait(HOSTILE, undefined)).toEqual(FALLBACK_SPEC);
    // other hostile shapes: a prototype key, a colour outside the palette, an over-long expression, too many copies
    for (const spec of [
      JSON.parse('{"body":[{"shape":"circle","__proto__":{"x":1}}]}'),
      { body: [{ shape: "circle", fill: "#ff0000" }] },
      { body: [{ shape: "circle", r: "1+".repeat(200) + "1" }] },
      { body: [{ shape: "circle", count: 999 }] },
    ]) expect(await resolvePortrait(spec, undefined)).toEqual(FALLBACK_SPEC);
  });
});

describe("the brief and the schema", () => {
  it("the brief names the closed vocabulary and the caps, with two examples that validate", () => {
    const brief = sigilBrief();
    for (const word of ["circle", "petal", "owner.dark", "beat", "charge", "aw", "clamp", "16 copies", "48 shapes"]) expect(brief).toContain(word);
    const examples = brief.split("\n").filter((l) => l.startsWith('{"body"'));
    expect(examples).toHaveLength(2);
    for (const e of examples) expect(validate(JSON.parse(e)).ok).toBe(true);
  });

  it("the schema asks for a body and offers only the known shapes and colours", () => {
    expect(PORTRAIT_SCHEMA.required).toEqual(["body"]);
    expect(PORTRAIT_SCHEMA.properties.body.items.properties.shape.enum).toContain("star");
    expect(PORTRAIT_SCHEMA.properties.body.items.properties.stroke.enum).toContain("owner");
    expect(ARCHETYPES_SCHEMA.properties.archetypes.items.required).toContain("portrait");
    expect(DESIGN_SCHEMA.properties.candidates.items.required).toContain("portrait");
  });

  it("the archetype stage and the designer put the format in their prompt; the plain reader prompt stays short", () => {
    expect(readerSystemPrompt([], true)).toContain("THE SIGIL FORMAT");
    expect(readerSystemPrompt([])).not.toContain("THE SIGIL FORMAT");
    expect(designerSystemPrompt()).toContain("THE SIGIL FORMAT");
    expect(PORTRAIT_SYSTEM).toContain("sigil");
    expect(ideaPortraitPrompt("a wolf", { name: "Wolf", emoji: "🐺", line: "A wolf." }, ["body: no shape"])).toContain("- body: no shape");
  });
});

// ---------- the idea reader ----------

function world() {
  const store = new MemoryMvpStore();
  const t = Date.parse("2026-10-08T08:00:00.000Z");
  const deps = { store, rules: MVP_RULES, now: () => new Date(t), today: () => ({ seq: 1, day: "Oct 8, 2026", startedAt: "2026-10-08T01:00:00.000Z", endsAt: "2026-10-09T01:00:00.000Z" }) };
  seedUnits(store, deps.now());
  grantIdea(deps, "p1");
  return { store, deps, idea: writeIdea(deps, "p1", "a hedgehog that punishes whoever hits it") };
}

const arch = (name: string, portrait: unknown): ArchetypeDraft => ({ name, emoji: "🦔", line: `A ${name.toLowerCase()} with spines for every foe.`, ru: { name: cyrillic(name), line: `Ру: ${cyrillic(name)}.` }, portrait });

/** A reader whose archetypes carry `portraits`, and whose portrait() answers from `again`, counting its calls. */
function reader(portraits: unknown[], again: unknown[] = []) {
  const asked: { name: string; problems?: string[] }[] = [];
  const fake = fakeIdeaReader();
  const r: IdeaReader = {
    ...fake,
    kind: "scripted",
    async archetypes(_t, opts) {
      const options = ["Spinehog", "Burrower", "Quillback"].slice(0, opts.want).map((n, i) => arch(n, portraits[i]));
      return { options, cantExpress: [] } satisfies ReaderAnswer<ArchetypeDraft>;
    },
    async portrait(_t, a, problems) {
      asked.push({ name: a.name, ...(problems ? { problems } : {}) });
      const next = again.shift();
      if (next instanceof Error) throw next;
      return next;
    },
  };
  return { r, asked };
}

describe("the idea reader's archetypes carry a sigil spec (slice C)", () => {
  it("good specs ride on the archetypes and the picked unit, untouched, with no second ask", async () => {
    const { store, deps, idea } = world();
    const { r, asked } = reader([GOOD, OTHER, GOOD]);
    await readIdeas(deps, r);
    const got = store.idea(idea.ideaId)!;
    expect(got.state).toBe("pick-archetype");
    expect(got.data.archetypes!.map((a) => a.portrait)).toEqual([GOOD, OTHER, GOOD]);
    expect(asked).toEqual([]);
    pickArchetype(deps, "p1", idea.ideaId, 1);
    await readIdeas(deps, fakeIdeaReader());
    pickReading(deps, "p1", idea.ideaId, 0);
    const unit = store.unit(store.idea(idea.ideaId)!.data.unitId!)!;
    expect(unit.portrait).toEqual(OTHER);
    expect(unit.row.name).toBe("Burrower");
  });

  it("a bad spec, then a good one: one more ask for that archetype, with the problem list", async () => {
    const { store, deps, idea } = world();
    const { r, asked } = reader([GOOD, BAD, GOOD], [OTHER]);
    await readIdeas(deps, r);
    const archetypes = store.idea(idea.ideaId)!.data.archetypes!;
    expect(archetypes.map((a) => a.portrait)).toEqual([GOOD, OTHER, GOOD]);
    expect(asked).toHaveLength(1);
    expect(asked[0]!.name).toBe("Burrower");
    expect(asked[0]!.problems!.join(" ")).toMatch(/unknown shape blob/);
  });

  it("bad twice: the generic sigil, and the archetype still shows", async () => {
    const { store, deps, idea } = world();
    const { r, asked } = reader([BAD, GOOD, GOOD], [BAD]);
    await readIdeas(deps, r);
    const archetypes = store.idea(idea.ideaId)!.data.archetypes!;
    expect(archetypes).toHaveLength(3);
    expect(archetypes[0]!.portrait).toEqual(FALLBACK_SPEC);
    expect(asked).toHaveLength(1);
    pickArchetype(deps, "p1", idea.ideaId, 0);
    await readIdeas(deps, fakeIdeaReader());
    pickReading(deps, "p1", idea.ideaId, 0);
    expect(store.unit(store.idea(idea.ideaId)!.data.unitId!)!.portrait).toEqual(FALLBACK_SPEC);
  });

  it("a hostile expression is never kept: it is asked about once, then the generic sigil", async () => {
    const { store, deps, idea } = world();
    const { r, asked } = reader([HOSTILE, GOOD, GOOD], [HOSTILE]);
    await readIdeas(deps, r);
    expect(store.idea(idea.ideaId)!.data.archetypes![0]!.portrait).toEqual(FALLBACK_SPEC);
    expect(JSON.stringify(store.idea(idea.ideaId)!.data)).not.toContain("constructor");
    expect(asked).toHaveLength(1);
  });

  it("a reader that cannot write a portrait again (or none at all) still gives the generic sigil, and a failing second try never fails the idea", async () => {
    const { store, deps, idea } = world();
    const { r } = reader([undefined, GOOD, GOOD], [new Error("claude -p failed")]);
    await readIdeas(deps, r);
    expect(store.idea(idea.ideaId)!.state).toBe("pick-archetype");
    expect(store.idea(idea.ideaId)!.data.archetypes![0]!.portrait).toEqual(FALLBACK_SPEC);
    const w = world();
    const { portrait: _p, ...noPortrait } = reader([BAD, GOOD, GOOD]).r;
    await readIdeas(w.deps, noPortrait);
    expect(w.store.idea(w.idea.ideaId)!.data.archetypes![0]!.portrait).toEqual(FALLBACK_SPEC);
  });

  it("the fake reader's archetypes carry valid, different specs", async () => {
    const { store, deps, idea } = world();
    await readIdeas(deps, fakeIdeaReader());
    const specs = store.idea(idea.ideaId)!.data.archetypes!.map((a) => a.portrait);
    for (const s of specs) expect(validate(s).ok).toBe(true);
    expect(new Set(specs.map((s) => JSON.stringify(s))).size).toBeGreaterThan(1);
  });

  it("a retried archetype batch also gets its portraits", async () => {
    const { store, deps, idea } = world();
    const fake = fakeIdeaReader();
    let calls = 0;
    const r: IdeaReader = {
      ...fake,
      async archetypes(t, opts) {
        calls++;
        const a = await fake.archetypes(t, opts);
        // the first call: two options fail the name check (lower-case); the retry asks for the rest
        return calls === 1 ? { ...a, options: a.options.map((o, i) => (i ? { ...o, name: o.name.toLowerCase() } : o)) } : a;
      },
    };
    await readIdeas(deps, r);
    const archetypes = store.idea(idea.ideaId)!.data.archetypes!;
    expect(calls).toBe(2);
    expect(archetypes.length).toBeGreaterThan(1);
    for (const a of archetypes) expect(validate(a.portrait).ok).toBe(true);
  });

  it("the archetypes prompt is unchanged in its text (the format goes in the system prompt)", () => {
    expect(archetypesPrompt("a wolf", { want: 3, units: [] })).not.toContain("sigil");
  });
});

// ---------- the night designer ----------

const parent = (def: UnitDef): Parent => ({ id: def.name, def, abilities: FIXTURE_ABILITIES, statuses: FIXTURE_STATUSES });
// the batch test's field: the fake designer's candidates land in band on it
const Archer = rule("Archer", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Shoot");
const Brute = plain("Brute", 14, 3);
const field: Field = { ghosts: [[Brute, Hourglass, Archer], [Nautilus, Archer, Brute], [Archer, Candle, Brute]], breakers: [[Brute, Brute, Archer]], abilities: FIXTURE_ABILITIES, statuses: FIXTURE_STATUSES };
const DAY = 24 * 3600 * 1000;
const now = 1_760_000_000_000;
const units = ["Hourglass", "Mirror"].map((id, i) => ({ id, addedAt: now - i * DAY }));
const list = () => pickPairs({ units, now, size: 1, small: true, requests: ["Hourglass+Mirror"] });
const parents = { Hourglass: parent(Hourglass), Mirror: parent(Mirror) };

describe("the night designer's fusions carry a sigil spec (slice C)", () => {
  it("the fake designer's ready candidates have a valid spec, and the codex design keeps it", async () => {
    const [res] = await runBatch(list(), { designer: fakeDesigner(), field, parents });
    const ready = res!.candidates.filter((c) => c.state === "ready");
    expect(ready.length).toBeGreaterThan(0);
    for (const c of ready) {
      expect(validate(c.portrait).ok).toBe(true);
      expect(designOf(c).portrait).toEqual(c.portrait);
    }
    for (const c of res!.candidates.filter((c) => c.state !== "ready")) expect(c.portrait).toBeUndefined();
  });

  it("good, bad then good, bad twice, hostile: each candidate ends with a valid spec; the retry gets the problems", async () => {
    const base = fakeDesigner();
    const drafts = [GOOD, BAD, HOSTILE];
    const asked: { lean: string; problems: string[] }[] = [];
    const designer = {
      ...base,
      async design(a: Parent, b: Parent) {
        return (await base.design(a, b)).map((d, i) => ({ ...d, portrait: drafts[i] }));
      },
      async portrait(_a: Parent, _b: Parent, d: { lean: string }, _n: unknown, problems: string[]) {
        asked.push({ lean: d.lean, problems });
        return d.lean === "B" ? OTHER : HOSTILE; // B gets it right the second time; twist does not
      },
    };
    const [res] = await runBatch(list(), { designer, field, parents });
    const by = Object.fromEntries(res!.candidates.map((c) => [c.lean, c]));
    const ready = (lean: string) => by[lean]!.state === "ready";
    expect(res!.candidates.filter((c) => c.state === "ready").length).toBeGreaterThan(0);
    if (ready("A")) expect(by.A!.portrait).toEqual(GOOD);
    if (ready("B")) expect(by.B!.portrait).toEqual(OTHER);
    if (ready("twist")) expect(by.twist!.portrait).toEqual(FALLBACK_SPEC);
    expect(asked.map((x) => x.lean).sort()).toEqual(["B", "twist"].filter(ready).sort());
    for (const a of asked) expect(a.problems.length).toBeGreaterThan(0);
    for (const c of res!.candidates) expect(JSON.stringify(c.portrait ?? null)).not.toContain("constructor");
  });

  it("a designer that cannot ask again gives the generic sigil after a bad first try", async () => {
    const base = fakeDesigner();
    const { portrait: _p, ...noAgain } = base;
    const designer = { ...noAgain, async design(a: Parent, b: Parent) { return (await base.design(a, b)).map((d) => ({ ...d, portrait: BAD })); } };
    const [res] = await runBatch(list(), { designer, field, parents });
    const ready = res!.candidates.filter((c) => c.state === "ready");
    expect(ready.length).toBeGreaterThan(0);
    for (const c of ready) expect(c.portrait).toEqual(FALLBACK_SPEC);
  });

  it("the claude designer retries through portraitPrompt with the problems", () => {
    const a = parent(Hourglass), b = parent(Mirror);
    const p = portraitPrompt(a, b, { lean: "A", intent: "hits the front", rule: {}, abilities: {} } as never, { name: { en: "Echo", ru: "Эхо" }, flavour: { en: "", ru: "" } }, ["body[0]: unknown shape blob"]);
    expect(p).toContain("Echo");
    expect(p).toContain("- body[0]: unknown shape blob");
    expect(typeof claudeDesigner({ model: "opus", namerModel: "sonnet", bin: "claude", timeoutMs: 1 }).portrait).toBe("function");
  });
});
