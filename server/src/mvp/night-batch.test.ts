// The night batch (makscee/void-board#1050): tonight's list, the fake
// designer end to end (compile, sim check, quick tune), the report page.
import { describe, expect, it } from "vitest";
import { Candle, FIXTURE_ABILITIES, FIXTURE_STATUSES, Gambler, Hourglass, Mirror, Nautilus } from "../../../src/words/fixtures.js";
import { plain, rule } from "../../../src/words/examples.js";
import type { UnitDef } from "../../../src/types.js";
import { BATCH_MAX, BATCH_MIN, compile, type DesignDraft, draftFace, foreignUnits, type Field, type Parent, pairKey, pickPairs, REDESIGN_AFTER_MS, ruleText, runBatch, scaled } from "./night-batch.js";
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

describe("a candidate's face (#1291)", () => {
  const rule = { triggers: [{ kind: "trigger" as const, on: { on: "TurnEnd" as const } }] };
  it("keeps the designer's face when it fits, else takes its trigger's", () => {
    const face = { when: { icon: "🔔", label: "When healed" }, headline: { icon: "🔁", amount: "=" } };
    expect(draftFace({ face, rule })).toEqual(face);
    expect(draftFace({ face: { ...face, when: { icon: "🔔", label: "Whenever it gets healed" } }, rule }).when).toEqual({ icon: "🌙", label: "Turn end" });
    expect(draftFace({ rule }).headline.icon).toBeTruthy();
  });
});

describe("one pair through the batch", () => {
  it("gives three candidates with fusedStats (Hourglass 8/1 plus half of Mirror 6/1), a rendered EN/RU sentence and a check", async () => {
    const list = pickPairs({ units, now, size: 1, small: true, requests: ["Hourglass+Mirror"] });
    const [r] = await runBatch(list, { designer: fakeDesigner(), field, parents: { Hourglass: parent(Hourglass), Mirror: parent(Mirror) } });
    expect(r!.error).toBeUndefined();
    expect(r!.candidates.map((c) => c.lean)).toEqual(["A", "B", "twist"]);
    for (const c of r!.candidates) {
      expect(c.unit.base).toEqual({ hp: 11, pwr: 1 });
      expect(c.name.en).toBeTruthy();
      expect(c.name.ru).toBeTruthy();
      // the fake designer writes no face: the trigger's fills in (#1291)
      expect(c.face?.when.label).toBeTruthy();
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
  it("rejects a design whose texts name a unit outside its pair, and keeps one naming its own (#1120)", async () => {
    const designer = fakeDesigner();
    const texts = [
      { name: { en: "Spite Mirror", ru: "Зеркало мести" }, flavour: { en: "", ru: "" } },
      { name: { en: "Glass Hour", ru: "Стеклянный час" }, flavour: { en: "Raven taught it patience.", ru: "" } },
      { name: { en: "Mirror Sand", ru: "Зеркальный песок" }, flavour: { en: "An Hourglass that reflects; a candle burns nearby.", ru: "" } },
    ];
    let n = 0;
    const naming = { ...designer, name: async () => texts[n++]! };
    const list = pickPairs({ units, now, size: 1, small: true, requests: ["Hourglass+Nautilus"] });
    const parents = { Hourglass: parent(Hourglass), Nautilus: parent(Nautilus), Mirror: parent(Mirror), Candle: parent(Candle), Raven: parent({ ...plain("Raven", 8, 1) }) };
    const [r] = await runBatch(list, { designer: naming, field, parents });
    expect(r!.candidates.map((c) => c.state === "rejected" && c.why?.startsWith("names a unit") ? c.why : "ok")).toEqual(["names a unit outside its pair: Mirror", "names a unit outside its pair: Raven", "names a unit outside its pair: Mirror"]);
    expect(foreignUnits(["An Hourglass that reflects; a candle burns nearby."], ["Hourglass", "Nautilus"], parents)).toEqual([]);
    expect(foreignUnits(["Hourglasses everywhere"], ["Mirror", "Nautilus"], parents)).toEqual([]);
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
  it("the quick tune scales power, never a counter's spend: at 3+ Glints it spends 3 (#1047, Echo Sands)", () => {
    const glints = { name: "Glints", counter: true as const, triggers: [{ kind: "trigger" as const, on: { on: "TurnEnd" as const } }], condition: { kind: "stacksAtLeast" as const, status: "Glints", n: 3, of: "holder" as const }, abilities: [{ selectors: [{ kind: "allyBehind" as const }], effects: [{ kind: "applyStatus" as const, status: "Strength", stacks: { kind: "const" as const, value: 2 } }] }, { selectors: [{ kind: "holder" as const }], effects: [{ kind: "consumeStacks" as const, stacks: { kind: "const" as const, value: 3 } }] }] };
    const c = scaled({ unit: { name: "Echo", base: { hp: 9, pwr: 2 } }, abilities: {}, statuses: { Glints: glints } }, new Set(["Glints"]), 1.6) as { statuses: Record<string, typeof glints> };
    const out = c.statuses.Glints!;
    expect(JSON.stringify(out.abilities[0]!.effects)).toContain('"value":3'); // 2 × 1.6, rounded
    expect(out.abilities[1]!.effects[0]).toEqual({ kind: "consumeStacks", stacks: { kind: "const", value: 3 } });
  });

  it("a fusion's sentence opens each clause on a verb, never a bare number (#1047)", () => {
    const glints = { name: "Glints", names: { en: "Glints", ru: { name: "Блики", acc: "Блики", count: ["Блик", "Блика", "Бликов"] as [string, string, string] } }, counter: true as const, triggers: [{ kind: "trigger" as const, on: { on: "TurnEnd" as const } }], condition: { kind: "stacksAtLeast" as const, status: "Glints", n: 3, of: "holder" as const }, abilities: [{ selectors: [{ kind: "allyBehind" as const }], effects: [{ kind: "applyStatus" as const, status: "Strength", stacks: { kind: "const" as const, value: 1 } }] }, { selectors: [{ kind: "holder" as const }], effects: [{ kind: "consumeStacks" as const, stacks: { kind: "const" as const, value: 3 } }] }] };
    const c = { unit: { name: "Echo", base: { hp: 9, pwr: 2 }, triggers: [{ kind: "trigger" as const, on: { on: "TurnEnd" as const } }], selectors: [{ kind: "holder" as const }], abilities: ["Glint"] }, abilities: { Glint: { name: "Glint", family: "Arcane" as const, effects: [{ kind: "applyStatus" as const, status: "Glints", stacks: { kind: "const" as const, value: 2 } }] } }, statuses: { ...FIXTURE_STATUSES, Glints: glints } };
    expect(ruleText(c as never, "en")).toBe("Turn end: gain 2 Glints. Turn end, if it has 3+ Glints: give the ally behind 1 Strength, and spend 3 Glints.");
    expect(ruleText(c as never, "ru")).toBe("Конец хода: получить 2 Блика. Конец хода, если у него 3 Блика или больше: дать союзнику позади 1 Силу, и потратить 3 Блика.");
  });

  it("a fusion's sentence says the chance and the turns, never a coin or 'every third turn' (Maks, 10-14)", () => {
    const c = {
      unit: { name: "Wick", base: { hp: 9, pwr: 2 }, triggers: [{ kind: "interceptor" as const, on: { on: "Death" as const, unit: "otherAlly" as const } }], condition: { kind: "turnEvery" as const, n: 3 }, selectors: [{ kind: "holder" as const }], abilities: ["Toss"] },
      abilities: { Toss: { name: "Toss", family: "Shield" as const, effects: [{ kind: "branch" as const, if: { kind: "flip" as const, p: 0.5 }, then: [{ kind: "preventDeath" as const, at: 1, cost: { stat: "pwr" as const, amount: 0 } }], else: [{ kind: "preventDeath" as const, at: 1, cost: { stat: "pwr" as const, amount: 2 } }] }] } },
      statuses: FIXTURE_STATUSES,
    };
    const en = ruleText(c as never, "en");
    const ru = ruleText(c as never, "ru");
    expect(en).toContain("on turns 3, 6, 9…");
    expect(en).toContain("with a 50% chance");
    expect(ru).toContain("на 3-м, 6-м, 9-м… ходу");
    expect(ru).toContain("с вероятностью 50%");
    expect(en).not.toMatch(/\b(heads|tails|flips?|coins?)\b|every third/i);
    expect(ru).not.toMatch(/орёл|орле|решк|монет|каждый 3-й/i);
  });

  it("rejects a rule the engine can't validate", () => {
    const d: DesignDraft = { lean: "A", intent: "nonsense", rule: { triggers: [{ kind: "trigger", on: { on: "TurnStart" } }], selectors: [{ kind: "frontEnemy" }], abilities: ["NoSuch"] }, abilities: {} };
    const c = compile(d, parent(Hourglass), parent(Mirror), "X", new Set());
    expect("issues" in c && c.issues.length > 0).toBe(true);
  });
  it("keeps the registry's Shield when a design redeclares it, so no Shield2 reaches the card", () => {
    const d: DesignDraft = { lean: "A", intent: "shield", rule: { triggers: [{ kind: "trigger", on: { on: "TurnStart" } }], selectors: [{ kind: "holder" }], abilities: ["Harden"] }, abilities: { Harden: { name: "Harden", family: "Shield", effects: [{ kind: "applyStatus", status: "Shield", stacks: { kind: "const", value: 2 } }] } }, statuses: { Shield: { name: "Shield", stacks: true } as never } };
    const c = compile(d, parent(Nautilus), parent(Mirror), "X", new Set(Object.keys(FIXTURE_STATUSES)));
    expect("ok" in c).toBe(true);
    if ("ok" in c) {
      expect(c.ok.statuses.Shield).toBe(FIXTURE_STATUSES.Shield);
      expect(Object.keys(c.ok.statuses)).not.toContain("Shield2");
      expect([...c.own]).toEqual(["Harden"]);
    }
  });
  it("refuses a coined status without EN and RU names, takes it with them", () => {
    const wax = (names?: unknown): DesignDraft => ({
      lean: "A",
      intent: "wax",
      rule: { triggers: [{ kind: "trigger", on: { on: "TurnStart" } }], selectors: [{ kind: "holder" }], abilities: ["Drip"] },
      abilities: { Drip: { name: "Drip", family: "Shield", effects: [{ kind: "applyStatus", status: "Wax", stacks: { kind: "const", value: 1 } }] } },
      statuses: { Wax: { name: "Wax", counter: true, abilities: [], ...(names ? { names } : {}) } as never },
    });
    const bare = compile(wax(), parent(Candle), parent(Mirror), "X", new Set(Object.keys(FIXTURE_STATUSES)));
    expect("issues" in bare && bare.issues).toEqual(["statuses.Wax: a coined status needs names.en"]);
    const enOnly = compile(wax({ en: "Wax" }), parent(Candle), parent(Mirror), "X", new Set(Object.keys(FIXTURE_STATUSES)));
    expect("issues" in enOnly && enOnly.issues[0]).toMatch(/names\.ru/);
    const named = compile(wax({ en: "Wax", ru: { name: "Воск", acc: "Воск", count: ["Воск", "Воска", "Воска"] } }), parent(Candle), parent(Mirror), "X", new Set(Object.keys(FIXTURE_STATUSES)));
    expect("ok" in named).toBe(true);
    if ("ok" in named) expect(ruleText(named.ok, "ru")).toBe("Начало хода: получить 1 Воск.");
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
