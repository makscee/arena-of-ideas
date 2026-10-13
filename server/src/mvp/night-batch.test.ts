// The night batch (makscee/void-board#1050): tonight's list, the fake
// designer end to end (compile, sim check, quick tune), the report page.
import { describe, expect, it } from "vitest";
import { Candle, FIXTURE_ABILITIES, FIXTURE_STATUSES, Gambler, Hourglass, Mirror, Nautilus } from "../../../src/words/fixtures.js";
import { plain, rule } from "../../../src/words/examples.js";
import type { UnitDef } from "../../../src/types.js";
import { BATCH_MAX, BATCH_MIN, compile, type DesignDraft, type Field, type Parent, pairKey, pickPairs, REDESIGN_AFTER_MS, runBatch } from "./night-batch.js";
import { DESIGN_SCHEMA, DESIGNER_RULES, designerSystemPrompt, designPrompt, designerFromEnv, fakeDesigner, namePrompt } from "./night-designer.js";
import { nightReport } from "./night-report.js";

const DAY = 24 * 3600 * 1000;
const now = 1_760_000_000_000;
const units = ["Hourglass", "Mirror", "Candle", "Gambler", "Nautilus", "Magnet", "Raven", "Seed"].map((id, i) => ({ id, addedAt: now - i * DAY }));

describe("tonight's list", () => {
  it("is 50-100 pairs, requested first, then wished, then bots' reach, then a spread over the newest", () => {
    const list = pickPairs({ units, now, size: 10, requests: ["Seed+Raven"], wishes: { "Candle+Gambler": 3, "Hourglass+Mirror": 5 }, botReach: { "Nautilus+Magnet": 40, "Gambler+Hourglass": 9 } });
    expect(list.length).toBe(28); // every pair of 8 units
    const many = Array.from({ length: 20 }, (_, i) => ({ id: `u${i}`, addedAt: now - i * DAY }));
    expect(pickPairs({ units: many, now, size: 10 }).length).toBe(BATCH_MIN);
    expect(list.slice(0, 5).map((p) => `${p.why}:${p.key}`)).toEqual(["requested:Raven+Seed", "wished:Hourglass+Mirror", "wished:Candle+Gambler", "bots:Magnet+Nautilus", "bots:Gambler+Hourglass"]);
    expect(list.filter((p) => p.why === "newest").length).toBeGreaterThan(0);
    expect(new Set(list.map((p) => p.key)).size).toBe(list.length);
    expect(pickPairs({ units: many, now, size: 500 }).length).toBe(BATCH_MAX);
  });
  it("drops order and skips pairs designed this week unless players passed, and claimed pairs forever", () => {
    expect(pairKey("Seed", "Raven")).toBe(pairKey("Raven", "Seed"));
    const history = { "Hourglass+Mirror": { designedAt: now - 2 * DAY }, "Candle+Gambler": { designedAt: now - 2 * DAY, passed: true }, "Magnet+Nautilus": { designedAt: now - 30 * DAY, claimed: true }, "Gambler+Hourglass": { designedAt: now - REDESIGN_AFTER_MS } };
    const keys = pickPairs({ units, now, size: 4, small: true, requests: ["Mirror+Hourglass", "Candle+Gambler", "Nautilus+Magnet", "Gambler+Hourglass"], history }).map((p) => p.key);
    expect(keys.slice(0, 2)).toEqual(["Candle+Gambler", "Gambler+Hourglass"]);
    expect(keys).not.toContain("Hourglass+Mirror");
    expect(keys).not.toContain("Magnet+Nautilus");
  });
  it("ignores units not in the pool and a unit paired with itself", () => {
    const list = pickPairs({ units: units.slice(0, 2), now, size: 3, small: true, requests: ["Seed+Raven", "Mirror+Mirror"] });
    expect(list.map((p) => p.key)).toEqual(["Hourglass+Mirror"]);
  });
});

const parent = (def: UnitDef): Parent => ({ id: def.name, def, abilities: FIXTURE_ABILITIES, statuses: FIXTURE_STATUSES });
const Archer = rule("Archer", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Shoot");
const Brute = plain("Brute", 14, 3);
const field: Field = { ghosts: [[Brute, Hourglass, Archer], [Nautilus, Archer, Brute], [Archer, Candle, Brute]], breakers: [[Brute, Brute, Archer]], abilities: FIXTURE_ABILITIES, statuses: FIXTURE_STATUSES };

describe("one pair through the batch", () => {
  it("gives three candidates with the parents' summed stats, a rendered EN/RU sentence and a check", async () => {
    const list = pickPairs({ units, now, size: 1, small: true, requests: ["Hourglass+Mirror"] });
    const [r] = await runBatch(list, { designer: fakeDesigner(), field, parents: { Hourglass: parent(Hourglass), Mirror: parent(Mirror) } });
    expect(r!.error).toBeUndefined();
    expect(r!.candidates.map((c) => c.lean)).toEqual(["A", "B", "twist"]);
    for (const c of r!.candidates) {
      expect(c.unit.base).toEqual({ hp: 14, pwr: 2 });
      expect(c.name.en).toBeTruthy();
      expect(c.name.ru).toBeTruthy();
      expect(c.check).not.toBeNull();
      expect(c.check!.games).toBeGreaterThan(0);
      expect(["ready", "rejected"]).toContain(c.state);
      if (c.state === "ready") {
        expect(c.text.en).toMatch(/enemy/i);
        expect(c.text.ru.length).toBeGreaterThan(3);
        expect(c.check!.terminates).toBe(true);
        expect(c.check!.inBand).toBe(true);
      } else expect(c.why).toMatch(/doesn't terminate|out of band/);
    }
  });
  it("names a pair's candidates one by one, each name unique within the pair", async () => {
    const designer = fakeDesigner();
    const seen: string[][] = [];
    const same = { ...designer, name: async (a: Parent, b: Parent, d: DesignDraft, taken: string[]) => (seen.push(taken), { name: { en: "Echo", ru: "Эхо" }, flavour: { en: "", ru: "" } }) };
    const list = pickPairs({ units, now, size: 1, small: true, requests: ["Hourglass+Mirror"] });
    const [r] = await runBatch(list, { designer: same, field, parents: { Hourglass: parent(Hourglass), Mirror: parent(Mirror) } });
    expect(seen).toEqual([[], ["Echo"], ["Echo", "Echo B"]]);
    expect(r!.candidates.map((c) => c.name.en)).toEqual(["Echo", "Echo B", "Echo twist"]);
  });
  it("stores a design that needs a missing word as a word request, not a bent rule", async () => {
    const designer = fakeDesigner();
    const wordy = { ...designer, design: async (a: Parent, b: Parent) => (await designer.design(a, b)).map((d, i) => (i === 2 ? { ...d, wordRequest: { word: "steal", meaning: "take a stat from the target" } } : d)) };
    const list = pickPairs({ units, now, size: 1, small: true, requests: ["Candle+Gambler"] });
    const [r] = await runBatch(list, { designer: wordy, field, parents: { Candle: parent(Candle), Gambler: parent(Gambler) } });
    const twist = r!.candidates[2]!;
    expect(twist.state).toBe("word");
    expect(twist.wordRequest?.word).toBe("steal");
    expect(twist.why).toMatch(/waiting for a new word: steal/);
  });
  it("rejects a rule the engine can't validate", () => {
    const d: DesignDraft = { lean: "A", intent: "nonsense", rule: { triggers: [{ kind: "trigger", on: { on: "TurnStart" } }], selectors: [{ kind: "frontEnemy" }], abilities: ["NoSuch"] }, abilities: {} };
    const c = compile(d, parent(Hourglass), parent(Mirror), "X", new Set());
    expect("issues" in c && c.issues.length > 0).toBe(true);
  });
  it("names a missing parent as the pair's error", async () => {
    const list = pickPairs({ units, now, size: 1, small: true, requests: ["Seed+Raven"] });
    const [r] = await runBatch(list, { designer: fakeDesigner(), field, parents: {} });
    expect(r!.error).toBe("parent missing");
  });
});

describe("the designer", () => {
  it("prompts with the four rules, the language and the written fusions, and asks for three leans", () => {
    const sys = designerSystemPrompt();
    for (const r of DESIGNER_RULES) expect(sys).toContain(r);
    expect(sys).toContain("Déjà Vu");
    expect(sys).toContain("Grave Tree");
    expect(sys).toContain("export interface UnitDef");
    expect(designPrompt(parent(Hourglass), parent(Mirror))).toMatch(/<parentA id="Hourglass">.*<parentB id="Mirror">/s);
    const d: DesignDraft = { lean: "twist", intent: "x", rule: {}, abilities: {} };
    expect(namePrompt(parent(Hourglass), parent(Mirror), d, ["Echo Glass"])).toContain('already named "Echo Glass"');
    expect((DESIGN_SCHEMA.properties.candidates.items.properties.lean as { enum: string[] }).enum).toEqual(["A", "B", "twist"]);
  });
  it("is the fake unless ARENA_NIGHT_DESIGNER=claude", () => {
    expect(designerFromEnv({}).kind).toBe("fake");
    expect(designerFromEnv({ ARENA_NIGHT_DESIGNER: "claude" }).kind).toBe("claude");
    expect(() => designerFromEnv({ ARENA_NIGHT_DESIGNER: "gpt" })).toThrow();
  });
});

describe("the report page", () => {
  it("shows each candidate in EN and RU with its state", async () => {
    const list = pickPairs({ units, now, size: 1, small: true, requests: ["Hourglass+Gambler"] });
    const results = await runBatch(list, { designer: fakeDesigner(), field, parents: { Hourglass: parent(Hourglass), Gambler: parent(Gambler) } });
    const html = nightReport({ day: "2026-10-13", designer: "fake", ranAt: "t", seconds: 1, parents: { Hourglass: { emoji: "⏳" }, Gambler: { emoji: "🎲" } }, results });
    expect(html).toContain("🎲⏳ Gambler + Hourglass");
    expect(html).toContain("Пробное слияние.");
    expect(html).toMatch(/ready to claim|rejected/);
  });
});
