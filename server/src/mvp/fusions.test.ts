import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { afterEach, describe, expect, it } from "vitest";
import type { DecisionResponse, FusionDiscovery, PlayerRef, UnitContent } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { mvpContent } from "./content.js";
import { awaitFusionName, cleanModelName, drainFusionNames, fusionNameReady, fusionNaming, httpModelNamer, MODEL_FAILURES, portmanteau, storedOrPortmanteau, type ModelNamer } from "./fusions.js";
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
    for (const raw of ["Pikachu", "Darth Medic", "Gandalf", "Super Mario", "Elsa", "Sonic", "Nefarian", "Swindle", "Admin Approved", "FREE V-BUCKS", "Frozen", "Hollow Knight", "Kel'Thuzad"])
      expect(cleanModelName(raw), raw).toBeNull();
    for (const raw of ["", "a", "This is a very long name for a unit", "Iron care mend heal", "Name42", "www scam example", "Ye Ye"]) expect(cleanModelName(raw), raw).toBeNull();
    expect(cleanModelName("Medic", brawler, medic)).toBeNull();
    // Ordinary words that are also franchise names pass inside a longer name.
    expect(cleanModelName("Cloud Warden")).toBe("Cloud Warden");
  });

  it("matches the blocklist after folding: glued, inflected and accented forms", () => {
    for (const raw of ["Darthvader", "Gandalfs", "Marios", "Hogwart", "Pikachú", "Pikach\u0075\u0301", "Thor's Edge", "SkyWalker", "Gul'dan", "Zélda", "Iron Hulks"])
      expect(cleanModelName(raw), raw).toBeNull();
    // Short names are matched per word, so ordinary words that contain them pass.
    for (const raw of ["Thornback", "Invader", "Marionette", "Smuggler", "Scamper", "Supersonic", "Subterran Ox"]) expect(cleanModelName(raw), raw).not.toBeNull();
  });

  it("splits CamelCase before matching words, and stems don't over-block ordinary names", () => {
    for (const raw of ["LordVader", "SuperMario", "BabyYoda", "IronMan", "Xmen", "Witchers"]) expect(cleanModelName(raw), raw).toBeNull();
    for (const raw of ["Hexmender", "Badminton", "Iron Mantle", "Twitcher", "Exterminator", "Carambola", "Fluffy"]) expect(cleanModelName(raw), raw).not.toBeNull();
  });

  it("catches lowercase joins, and never matches a stem across a word's edge", () => {
    for (const raw of ["Supermario", "Lordvader", "Babyyoda", "Thorhammer", "Hulkbuster", "Yodaling", "Fusion", "Fused Medic", "Spider Man Medic", "Lordbatman", "Darthawk"])
      expect(cleanModelName(raw), raw).toBeNull();
    for (const raw of ["Aqua Mantis", "Spider Mantis", "Bat Mantle", "Batmancer", "Dart Hawk", "Sonic Shrieker", "Grim Joker", "Shadow Swindle", "Confusion", "Hulking Brute", "Evader"])
      expect(cleanModelName(raw), raw).not.toBeNull();
  });

  it("refuses names that only join the two parts", () => {
    const squire = unit("squire", "Squire");
    const gnat = unit("gnat", "Gnat");
    for (const raw of ["SquireGnat", "Gnat Squire", "Squire-Gnat", "Squires", "Gnat"]) expect(cleanModelName(raw, squire, gnat), raw).toBeNull();
    expect(cleanModelName("Squat", squire, gnat)).toBe("Squat");
    expect(cleanModelName("Gnatling Warden", squire, gnat)).toBe("Gnatling Warden");
    const golem = unit("golem", "Iron Golem");
    expect(cleanModelName("GolemGnat", golem, gnat)).toBeNull();
  });
});

describe("MVP fusion names: through the runtime", () => {
  const content = mvpContent();
  const [a, b, c] = content.units;

  /** A runtime whose namer asks `model`, with a human run holding a, b and c Awoken. */
  function world(model: ModelNamer | null, player: PlayerRef = maks, backoffMs?: (failures: number) => number) {
    const rt = mvpRuntime({ content, seed: () => 7 });
    // Swap in a namer with this model (the runtime's own reads ARENA_NAMER_URL).
    const naming = fusionNaming(rt, { model, ...(backoffMs ? { backoffMs } : {}) });
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

  it("with the model down, fuses at once with the portmanteau, and a human's fuse fixes it for good", async () => {
    let down = true;
    const asked: string[] = [];
    const { rt, stored } = world(async (x, y) => {
      asked.push(`${x.id}+${y.id}`);
      if (down) throw new Error("connection refused");
      return "Grimward";
    }, maks, () => 0);
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 });
    await drainFusionNames(rt.store);
    expect(asked).toHaveLength(6);
    const done = decide(rt, stored(), { kind: "fuse", first: 0, second: 1 });
    const name = portmanteau(a!.name, b!.name);
    expect(done.run.line[0]).toMatchObject({ name, fusion: { name, discoveredBy: maks } });
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name, nameSource: "fallback", discoveredBy: maks });
    // The model comes back: the failed pairs are asked again, except the one a human fused.
    down = false;
    asked.length = 0;
    await drainFusionNames(rt.store);
    expect(asked).not.toContain(`${a!.id}+${b!.id}`);
    expect(asked.length).toBeGreaterThan(0);
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name, nameSource: "fallback" });
    expect(stored().line[0]).toMatchObject({ name, fusion: { name } });
  });

  it("asks again with a backoff when the model fails, until it answers", async () => {
    let calls = 0;
    const waits: number[] = [];
    const { rt, stored } = world(async () => {
      calls++;
      if (calls <= 2) throw new Error("timeout");
      return "Grimward";
    }, maks, (failures) => (waits.push(failures), 0));
    rt.store.putRun({ ...stored(), line: stored().line.slice(0, 2) });
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 }); // queues a+b and b+a
    await drainFusionNames(rt.store); // both pairs fail once
    await drainFusionNames(rt.store); // and are asked again: both answer
    expect(waits).toEqual([1, 1]);
    const pv = preview(rt, stored(), { kind: "fuse", first: 0, second: 1 });
    expect(pv.run.line[0]).toMatchObject({ name: "Grimward" });
  });

  it("the first fuse fixes a pair's name for good, a bot's too: a later model answer changes nothing", async () => {
    let release!: (name: string) => void;
    const answered = new Promise<string>((r) => (release = r));
    const { rt, stored } = world(() => answered, bot);
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 }); // queues the bot's pairs
    const draining = drainFusionNames(rt.store);
    // Fused before the model answered: the fallback, for good.
    decide(rt, stored(), { kind: "fuse", first: 0, second: 1 });
    const fallback = portmanteau(a!.name, b!.name);
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: fallback, nameSource: "fallback", discoveredBy: null });
    release("Grimward");
    await draining;
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: fallback, nameSource: "fallback" });
    expect(stored().line[0]).toMatchObject({ name: fallback, fusion: { name: fallback } });
    // A human fusing it later gets the same name and claims the credit.
    const human = startRun(rt, maks);
    rt.store.putRun({ ...human, line: [lineUnitOf(a!, "h1", 3, rt.rules), lineUnitOf(b!, "h2", 3, rt.rules)], nextUid: 3 });
    const done = decide(rt, rt.store.run(human.runId)!, { kind: "fuse", first: 0, second: 1 });
    expect(done.run.line[0]).toMatchObject({ name: fallback, fusion: { discoveredBy: maks } });
    expect(rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: fallback, discoveredBy: maks });
  });

  it("asks for humans' pairs before bots', also pairs queued while it drains", async () => {
    const asked: string[] = [];
    const botSide = world(async (x, y) => (asked.push(`${x.id}+${y.id}`), `Name${"xyz"[asked.length % 3]}wyn`), bot);
    const { rt } = botSide;
    decide(rt, botSide.stored(), { kind: "reorder", from: 0, to: 0 }); // 6 bots' pairs of a, b, c
    const human = startRun(rt, maks);
    rt.store.putRun({ ...human, line: [lineUnitOf(c!, "h1", 3, rt.rules), lineUnitOf(a!, "h2", 3, rt.rules)], nextUid: 3 });
    decide(rt, rt.store.run(human.runId)!, { kind: "reorder", from: 0, to: 0 }); // c+a and a+c move up
    await drainFusionNames(rt.store);
    expect(asked).toHaveLength(6);
    expect(asked.slice(0, 2).sort()).toEqual([`${a!.id}+${c!.id}`, `${c!.id}+${a!.id}`].sort());
  });

  it("a bot fuses a pair only once its name is ready, or once the model gave up on it", async () => {
    // The model fails on one pair only, so it never counts as down.
    const d = content.units[3]!;
    const { rt, stored } = world(async (x, y) => {
      if (x.id === a!.id && y.id === d.id) throw new Error("503");
      return "Grimward";
    }, bot, () => 0);
    expect(fusionNameReady(rt.store, a!.id, b!.id)).toBe(false);
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 });
    await drainFusionNames(rt.store);
    expect(fusionNameReady(rt.store, a!.id, b!.id)).toBe(true);
    rt.store.putRun({ ...stored(), line: [...stored().line, lineUnitOf(d, "u4", 3, rt.rules)], nextUid: 5 });
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 });
    for (let i = 1; i < MODEL_FAILURES; i++) {
      await drainFusionNames(rt.store);
      expect(fusionNameReady(rt.store, a!.id, d.id)).toBe(false);
    }
    await drainFusionNames(rt.store);
    expect(fusionNameReady(rt.store, a!.id, d.id)).toBe(true);
    // With no model at all, every pair is ready.
    expect(fusionNameReady(new MemoryMvpStore(), a!.id, b!.id)).toBe(true);
  });

  it("after MODEL_FAILURES failed asks in a row the model is down: no bot waits, until it answers again", async () => {
    let down = true;
    const { rt, stored } = world(async () => {
      if (down) throw new Error("connection refused");
      return "Grimward";
    }, bot, () => 0);
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 }); // 6 pairs, each fails once
    await drainFusionNames(rt.store);
    expect(fusionNameReady(rt.store, a!.id, b!.id)).toBe(true);
    // Back up: one answer and the rest wait for theirs again.
    down = false;
    const d = content.units[3]!;
    rt.store.putRun({ ...stored(), line: [lineUnitOf(a!, "u1", 3, rt.rules), lineUnitOf(d, "u4", 3, rt.rules)], nextUid: 5 });
    const waited = awaitFusionName(rt.store, a!.id, d.id);
    expect(fusionNameReady(rt.store, a!.id, d.id)).toBe(true); // still down until an answer
    await drainFusionNames(rt.store);
    await waited;
    expect(rt.peekFusionName(a!, d, bot).name).toBe("Grimward");
  });

  it("a bot waiting for a pair has it asked next and wakes when it is named, or when the model gives up", async () => {
    const asked: string[] = [];
    let failOn = "";
    const { rt, stored } = world(async (x, y) => {
      asked.push(`${x.id}+${y.id}`);
      if (`${x.id}+${y.id}` === failOn) throw new Error("timeout");
      return "Grimward";
    }, bot, () => 0);
    decide(rt, stored(), { kind: "reorder", from: 0, to: 0 }); // 6 pairs queued ahead
    let woke = false;
    const waited = awaitFusionName(rt.store, c!.id, b!.id).then(() => (woke = true));
    await drainFusionNames(rt.store);
    expect(asked[0]).toBe(`${c!.id}+${b!.id}`);
    await waited;
    expect(woke).toBe(true);
    expect(fusionNameReady(rt.store, c!.id, b!.id)).toBe(true);
    // A pair the model keeps failing on: the bot wakes after MODEL_FAILURES asks.
    const d = content.units[3]!;
    failOn = `${d.id}+${a!.id}`;
    let gaveUp = false;
    const late = awaitFusionName(rt.store, d.id, a!.id).then(() => (gaveUp = true));
    for (let i = 1; i < MODEL_FAILURES; i++) {
      await drainFusionNames(rt.store);
      expect(gaveUp).toBe(false);
    }
    await drainFusionNames(rt.store);
    await late;
    expect(gaveUp).toBe(true);
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
    // Bots' lines are asked ahead too.
    await drainFusionNames(botWorld.rt.store);
    expect(asked).toHaveLength(6);
    decide(botWorld.rt, botWorld.stored(), { kind: "fuse", first: 0, second: 1 });
    expect(botWorld.rt.store.fusion(a!.id, b!.id)).toMatchObject({ name: "Duskmend", discoveredBy: null, nameSource: "model" });

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

  it("throws on a 5xx (mlx still loading), so the pair is asked again", async () => {
    server = createServer((_req, res) => {
      res.statusCode = 503;
      res.end("loading");
    });
    await new Promise<void>((r) => server!.listen(0, "127.0.0.1", r));
    const { port } = server.address() as AddressInfo;
    await expect(httpModelNamer(`http://127.0.0.1:${port}/v1/chat/completions`)(brawler, medic)).rejects.toThrow("503");
  });
});
