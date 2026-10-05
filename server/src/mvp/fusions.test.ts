import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { afterEach, describe, expect, it } from "vitest";
import type { DecisionResponse, FusionDiscovery, PlayerRef, UnitContent } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { mvpContent } from "./content.js";
import { cleanModelName, drainFusionNames, fusionNaming, httpModelNamer, portmanteau, storedOrPortmanteau, type ModelNamer } from "./fusions.js";
import { decide, preview, startRun } from "./runs.js";
import { mvpRuntime } from "./runtime.js";
import { MemoryMvpStore } from "./store.js";

const form = { when: [], who: [], does: ["a"] };
const unit = (id: string, name: string): UnitContent => ({ id, name, emoji: "x", tier: 1, base: { pwr: 1, hp: 1 }, forms: { sleeping: form, awoken: form } });
const brawler = unit("brawler", "Brawler");
const medic = unit("medic", "Medic");
const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };
const bot: PlayerRef = { id: "bot", name: "bot-Ash", bot: true };

describe("MVP fusion names: the fallback and the store", () => {
  it("falls back to a deterministic portmanteau", () => {
    expect(portmanteau("Brawler", "Medic")).toBe("Brawdic");
    expect(portmanteau("Brawler", "Medic")).toBe(portmanteau("Brawler", "Medic"));
    expect(portmanteau("Medic", "Brawler")).not.toBe(portmanteau("Brawler", "Medic"));
  });

  it("credits a human, never a bot, and keeps a stored name and credit", () => {
    const store = new MemoryMvpStore();
    const name = storedOrPortmanteau(store);
    expect(name(brawler, medic, maks)).toEqual({ name: "Brawdic", discoveredBy: maks });
    expect(name(brawler, medic, bot)).toEqual({ name: "Brawdic", discoveredBy: null });

    const byBot: FusionDiscovery = { first: "brawler", second: "medic", name: "Ironcare", discoveredBy: null, discoveredAt: "t", nameSource: "model" };
    store.putFusion(byBot);
    expect(name(brawler, medic, maks)).toEqual({ name: "Ironcare", discoveredBy: maks });
    store.putFusion({ ...byBot, discoveredBy: maks });
    expect(name(brawler, medic, { id: "p2", name: "Eva", bot: false })).toEqual({ name: "Ironcare", discoveredBy: maks });
    expect(store.fusion("medic", "brawler")).toBeUndefined();
    expect(store.fusions()).toHaveLength(1);
  });

  it("without a model, decide's namer and the preview's agree on the portmanteau", () => {
    const store = new MemoryMvpStore();
    const naming = fusionNaming({ store, content: { version: "t", units: [brawler, medic], abilities: {}, statuses: {} }, now: () => new Date(0) }, { model: null });
    expect(naming.nameFusion(brawler, medic, maks)).toEqual(naming.peek(brawler, medic, maks));
    expect(naming.peek(brawler, medic, maks)).toEqual({ name: "Brawdic", discoveredBy: maks });
  });
});

describe("MVP fusion names: the model's answer through the blocklist", () => {
  it("keeps short invented names, title-cased", () => {
    expect(cleanModelName("Ironcare")).toBe("Ironcare");
    expect(cleanModelName('Name: "iron mender".\nIt combines…')).toBe("Iron Mender");
    expect(cleanModelName("<think>hmm</think>\nBloodmend")).toBe("Bloodmend");
    expect(cleanModelName("Ash-Warden")).toBe("Ash-Warden");
    expect(cleanModelName("🥊🎯 Stormancer!")).toBe("Stormancer");
  });

  it("refuses franchise names, fake official titles, spam and non-names", () => {
    for (const raw of ["Pikachu", "Darth Medic", "Gandalf", "Super Mario", "Elsa", "Sonic Brawl", "Nefarian", "Captain Swindle", "Admin Approved", "FREE V-BUCKS", "Frozen", "Hollow Knight", "Kel'Thuzad"])
      expect(cleanModelName(raw), raw).toBeNull();
    for (const raw of ["", "a", "This is a very long name for a unit", "Iron care mend heal", "Name42", "www scam example", "Ye Ye"]) expect(cleanModelName(raw), raw).toBeNull();
    expect(cleanModelName("Medic", brawler, medic)).toBeNull();
    // Ordinary words that are also franchise names pass inside a longer name.
    expect(cleanModelName("Cloud Warden")).toBe("Cloud Warden");
  });
});

describe("MVP fusion names: through the runtime", () => {
  const content = mvpContent();
  const [a, b, c] = content.units;

  /** A runtime whose namer asks `model`, with a human run holding a, b and c Awoken. */
  function world(model: ModelNamer | null, player: PlayerRef = maks) {
    const rt = mvpRuntime({ content, seed: () => 7 });
    // Swap in a namer with this model (the runtime's own reads ARENA_NAMER_URL).
    const naming = fusionNaming(rt, { model });
    rt.hooks[0] = naming.hooks;
    rt.nameFusion = naming.nameFusion;
    rt.peekFusionName = naming.peek;
    const run = startRun(rt, player);
    rt.store.putRun({ ...run, line: [lineUnitOf(a!, "u1", 3, rt.rules), lineUnitOf(b!, "u2", 3, rt.rules), lineUnitOf(c!, "u3", 3, rt.rules)], nextUid: 4 });
    const stored = () => rt.store.run(run.runId)!;
    return { rt, runId: run.runId, stored };
  }

  it("prefetches model names for every fusable ordered pair, and the first fuse stores the model's name with the credit", async () => {
    const asked: string[] = [];
    const { rt, stored } = world(async (x, y) => (asked.push(`${x.id}+${y.id}`), `${x.name.slice(0, 3)}${y.name.slice(-3)}wyn`));
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 });
    await drainFusionNames(rt.store);
    expect(asked).toHaveLength(6);

    const pv = preview(rt, stored(), { kind: "fuse", first: 1, second: 0 });
    const want = cleanModelName(`${b!.name.slice(0, 3)}${a!.name.slice(-3)}wyn`)!;
    expect(pv.run.line[0]).toMatchObject({ kind: "fused", name: want });
    const done: DecisionResponse = decide(rt, stored(), { kind: "fuse", first: 1, second: 0 });
    expect(done.run.line[0]).toMatchObject({ name: want, fusion: { name: want, discoveredBy: maks } });
    expect(rt.store.fusions()).toEqual([{ first: b!.id, second: a!.id, name: want, discoveredBy: maks, discoveredAt: expect.any(String), nameSource: "model" }]);

    // The same pair in the same order gets the same name again, for anyone; the credit stays.
    const other = world(null, { id: "p2", name: "Eva", bot: false });
    const eva = fusionNaming({ ...other.rt, store: rt.store }, { model: null });
    expect(eva.peek(b!, a!, { id: "p2", name: "Eva", bot: false })).toEqual({ name: want, discoveredBy: maks });
  });

  it("with the model down, fuses at once with the portmanteau and never asks again for a stored pair", async () => {
    const { rt, stored } = world(async () => {
      throw new Error("connection refused");
    });
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 });
    await drainFusionNames(rt.store);
    const done = decide(rt, stored(), { kind: "fuse", first: 0, second: 1 });
    const name = portmanteau(a!.name, b!.name);
    expect(done.run.line[0]).toMatchObject({ name, fusion: { name, discoveredBy: maks } });
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name, nameSource: "fallback", discoveredBy: maks });
  });

  it("renames a fallback pair once when the model answers later, on the store and the fused unit", async () => {
    let answer: string | null = "Grimward";
    const { rt, stored } = world(async () => answer);
    // Fused before the model answered: the fallback name.
    decide(rt, stored(), { kind: "fuse", first: 0, second: 1 });
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: portmanteau(a!.name, b!.name), nameSource: "fallback" });
    await drainFusionNames(rt.store);
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: "Grimward", nameSource: "model", discoveredBy: maks });
    expect(stored().line[0]).toMatchObject({ name: "Grimward", fusion: { name: "Grimward" } });
    // Never again: a later answer changes nothing.
    answer = "Otherwise";
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 });
    await drainFusionNames(rt.store);
    expect(rt.store.fusion(a!.id, b!.id)?.name).toBe("Grimward");
  });

  it("a blocked answer keeps the portmanteau", async () => {
    let asked = 0;
    const { rt, stored } = world(async () => (asked++, "Pikachu"));
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 });
    await drainFusionNames(rt.store);
    expect(asked).toBe(6 * 3); // three tries for each of the 6 ordered pairs
    decide(rt, stored(), { kind: "fuse", first: 0, second: 1 });
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: portmanteau(a!.name, b!.name), nameSource: "fallback" });
  });

  it("a bot's fusion is stored uncredited; the first human to fuse the pair claims it, the name stays", async () => {
    const asked: string[] = [];
    const botWorld = world(async (x, y) => (asked.push(`${x.id}+${y.id}`), "Duskmend"), bot);
    decide(botWorld.rt, botWorld.stored(), { kind: "reorder", from: 0, to: 0 });
    expect(botWorld.rt.store.fusions()).toEqual([]);
    decide(botWorld.rt, botWorld.stored(), { kind: "fuse", first: 0, second: 1 });
    await drainFusionNames(botWorld.rt.store);
    // Bots don't prefetch; only the fused pair is asked, to rename its fallback.
    expect(asked).toEqual([`${a!.id}+${b!.id}`]);
    expect(botWorld.rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: "Duskmend", discoveredBy: null });

    const { rt } = botWorld;
    const human = startRun(rt, maks);
    rt.store.putRun({ ...human, line: [lineUnitOf(a!, "h1", 3, rt.rules), lineUnitOf(b!, "h2", 3, rt.rules)], nextUid: 3 });
    const done = decide(rt, rt.store.run(human.runId)!, { kind: "fuse", first: 0, second: 1 });
    expect(done.run.line[0]).toMatchObject({ name: "Duskmend", fusion: { discoveredBy: maks } });
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: "Duskmend", discoveredBy: maks });
  });
});

describe("MVP fusion names: the HTTP model client", () => {
  let server: Server | undefined;
  afterEach(() => server?.close());

  it("asks an OpenAI-compatible chat endpoint and returns its reply", async () => {
    let got: { messages: { content: string }[] } | undefined;
    server = createServer((req, res) => {
      let body = "";
      req.on("data", (c) => (body += c));
      req.on("end", () => {
        got = JSON.parse(body);
        res.setHeader("content-type", "application/json");
        res.end(JSON.stringify({ choices: [{ message: { content: "Ironcare" } }] }));
      });
    });
    await new Promise<void>((r) => server!.listen(0, "127.0.0.1", r));
    const { port } = server.address() as AddressInfo;
    const ask = httpModelNamer(`http://127.0.0.1:${port}/v1/chat/completions`);
    expect(await ask(brawler, medic)).toBe("Ironcare");
    expect(got?.messages.at(-1)?.content).toContain("Brawler (first) merges with x Medic (second)");
  });
});
