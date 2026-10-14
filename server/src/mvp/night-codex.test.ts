// The night batch into the world (#1050, reopened): candidates into the Codex
// tables, word requests into the queue, wishes into tonight's list, and a
// stored design that fights with its coined entries.
import { describe, expect, it } from "vitest";
import { forgeView, fuseDesigned, pairState } from "../../../src/mvp/codex.js";
import { MVP_RULES } from "../../../src/mvp/contract.js";
import { fightLines } from "../../../src/mvp/fight.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import type { UnitDef } from "../../../src/types.js";
import { Candle, FIXTURE_ABILITIES, FIXTURE_STATUSES, Gambler, Hourglass, Mirror, Nautilus } from "../../../src/words/fixtures.js";
import { plain, rule } from "../../../src/words/examples.js";
import { mvpContent } from "./content.js";
import { type Field, type Parent, pickPairs, runBatch } from "./night-batch.js";
import { designOf, listInputFrom, storeNight } from "./night-codex.js";
import { fakeDesigner } from "./night-designer.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";

const DAY = 24 * 3600 * 1000;
const now = 1_760_000_000_000;
const units = ["hourglass", "mirror", "candle", "gambler", "nautilus"].map((id, i) => ({ id, addedAt: now - i * DAY }));
const parent = (def: UnitDef): Parent => ({ id: def.name, def, abilities: FIXTURE_ABILITIES, statuses: FIXTURE_STATUSES });
const parents: Record<string, Parent> = { hourglass: parent(Hourglass), mirror: parent(Mirror), candle: parent(Candle), gambler: parent(Gambler), nautilus: parent(Nautilus) };
const Archer = rule("Archer", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Shoot");
const Brute = plain("Brute", 14, 3);
const field: Field = { ghosts: [[Brute, Hourglass, Archer], [Nautilus, Archer, Brute], [Archer, Candle, Brute]], breakers: [[Brute, Brute, Archer]], abilities: FIXTURE_ABILITIES, statuses: FIXTURE_STATUSES };
const at = new Date("2026-10-14T03:30:00.000Z");

describe.each([
  ["memory", () => new MemoryMvpStore() as MvpStore],
  ["sqlite", () => new SqliteMvpStore(":memory:") as MvpStore],
])("the night batch into the Codex (%s)", (_name, make) => {
  it("makes a pair ready to claim with its candidates, EN/RU names and flavour, and the coined entries", async () => {
    const store = make();
    const list = pickPairs({ units, now, size: 1, small: true, requests: ["mirror+hourglass"] });
    const results = await runBatch(list, { designer: fakeDesigner(), field, parents });
    const ready = results[0]!.candidates.filter((c) => c.state === "ready");
    expect(ready.length).toBeGreaterThan(0);
    const stored = storeNight(store, results, at, () => "⏳🪞");
    expect(stored).toEqual({ ready: ["hourglass+mirror"], claimed: [], words: 0 });
    const pair = store.codexPair("hourglass", "mirror")!;
    expect(pairState(pair)).toBe("ready");
    expect(pair.preparedAt).toBe(at.toISOString());
    expect(pair.candidates).toHaveLength(ready.length);
    for (const [i, d] of pair.candidates.entries()) {
      expect(d.name).toBe(ready[i]!.name.en);
      expect(d.texts?.ru?.name).toBe(ready[i]!.name.ru);
      expect(d.emoji).toBe("⏳🪞");
      expect(d.recipe.does).toEqual(ready[i]!.unit.abilities);
      expect(d.recipe.when).toEqual(ready[i]!.unit.triggers);
      expect(d.recipe.text).toBeUndefined(); // the card renders the rule, EN or RU (#1047)
    }
    const view = forgeView("mirror", "hourglass", pair, 0, false);
    expect(view.state).toBe("ready");
    expect(view.sealed).toBe(ready.length);
  });
  it("queues a word request for a candidate waiting for a word and stores no pair without a ready one", async () => {
    const store = make();
    const designer = fakeDesigner();
    const wordy = { ...designer, design: async (a: Parent, b: Parent) => (await designer.design(a, b)).map((d) => ({ ...d, wordRequest: { word: "steal", meaning: "take a stat from the target" } })) };
    const list = pickPairs({ units, now, size: 1, small: true, requests: ["candle+gambler"] });
    const results = await runBatch(list, { designer: wordy, field, parents });
    const stored = storeNight(store, results, at);
    expect(stored).toEqual({ ready: [], claimed: [], words: 1 });
    expect(store.codexPair("candle", "gambler")).toBeUndefined();
    expect(store.wordRequests().map((r) => [r.source, r.ref, r.word])).toEqual([["night", "candle+gambler", "steal"]]);
    expect(storeNight(store, results, at).words).toBe(0);
  });
  it("leaves a claimed pair alone, and the history skips it from tonight's list", async () => {
    const store = make();
    const list = pickPairs({ units, now, size: 1, small: true, requests: ["hourglass+mirror"] });
    const results = await runBatch(list, { designer: fakeDesigner(), field, parents });
    storeNight(store, results, new Date(at.getTime() - 10 * DAY));
    expect(store.claimCodexPair("hourglass", "mirror", { pick: 0, by: { id: "p1", name: "Maks", bot: false }, at: at.toISOString() })).toBe(true);
    const before = store.codexPair("hourglass", "mirror");
    expect(storeNight(store, results, at)).toEqual({ ready: [], claimed: ["hourglass+mirror"], words: 0 });
    expect(store.codexPair("hourglass", "mirror")).toEqual(before);
    const input = listInputFrom(store, units, at.getTime());
    expect(input.history!["hourglass+mirror"]).toEqual({ designedAt: at.getTime() - 10 * DAY, claimed: true });
    expect(pickPairs({ units, now: at.getTime(), size: 10, small: true, ...input }).map((p) => p.key)).not.toContain("hourglass+mirror");
  });
  it("puts a pair wished in the forge ahead of unwished ones, and a pair designed this week waits", () => {
    const store = make();
    store.putDay({ seq: 4, day: "2026-10-13", startedAt: "2026-10-13T00:00:00.000Z", endsAt: "2026-10-14T00:00:00.000Z" });
    store.addWish({ a: "gambler", b: "nautilus", playerId: "p1", seq: 4, at: at.toISOString() });
    store.addWish({ a: "gambler", b: "nautilus", playerId: "p2", seq: 3, at: at.toISOString() });
    store.addWish({ a: "candle", b: "mirror", playerId: "p1", seq: 4, at: at.toISOString() });
    store.putCodexPair({ a: "hourglass", b: "mirror", candidates: [{ name: "Fresh", lean: "a", recipe: { when: [], who: [], does: ["Strike"] } }], preparedAt: new Date(at.getTime() - DAY).toISOString() });
    const input = listInputFrom(store, units, at.getTime());
    expect(input.wishes).toEqual({ "gambler+nautilus": 2, "candle+mirror": 1 });
    const list = pickPairs({ units, now: at.getTime(), size: 4, small: true, ...input });
    expect(list.slice(0, 2).map((p) => [p.key, p.why])).toEqual([["gambler+nautilus", "wished"], ["candle+mirror", "wished"]]);
    expect(list.map((p) => p.key)).not.toContain("hourglass+mirror");
  });
});

describe("a stored night design", () => {
  it("fights with the entries it coined, which the pool lacks", async () => {
    const list = pickPairs({ units, now, size: 1, small: true, requests: ["hourglass+mirror"] });
    const [r] = await runBatch(list, { designer: fakeDesigner(), field, parents });
    const c = r!.candidates.find((x) => x.state === "ready")!;
    const design = designOf({ ...c, unit: { ...c.unit, abilities: ["Night Echo"] }, abilities: { ...c.abilities, "Night Echo": { name: "Night Echo", family: "Strike", effects: [{ kind: "damage", amount: { kind: "const", value: 2 } }] } } });
    expect(design.abilities?.["Night Echo"]).toBeDefined();
    const content = mvpContent();
    const [a, b] = content.units;
    const fused = fuseDesigned(lineUnitOf(a!, "u1", 3), lineUnitOf(b!, "u2", 3), design, null);
    expect(fused.fusion?.abilities?.["Night Echo"]).toBeDefined();
    const me = { id: "p1", name: "Maks", bot: false };
    const foe = { id: "p2", name: "Kim", bot: false };
    const rules = MVP_RULES;
    const rec = fightLines({ player: me, line: [fused] }, { player: foe, line: [lineUnitOf(a!, "v1", 1)] }, { battleId: "t", seed: 1, kind: "round", round: 1, runId: null, at: "", content, rules });
    expect(["A", "B", "draw"]).toContain(rec.winner);
  });
});
