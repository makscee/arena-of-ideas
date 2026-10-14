// Rules v2, the Codex (#1049): pair states, claims, wishes and the forge.
import { describe, expect, it } from "vitest";
import { canonicalDesign, codexPair, forgeView, fuseDesigned, pairKey, pairState, type CodexPair, type FusionDesign } from "../../../src/mvp/codex.js";
import type { MvpContent, PlayerRef, UnitContent } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import type { MvpRunState } from "../../../src/mvp/run.js";
import { createMvpApp } from "./app.js";
import { playBotRun } from "./bots.js";
import { botForge, codexForged, forgeOffers } from "./codex.js";
import { mvpContent } from "./content.js";
import { decide, preview, startRun } from "./runs.js";
import { mvpRuntime, type MvpDeps } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";

const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };
const kim: PlayerRef = { id: "p2", name: "Kim", bot: false };
const botP: PlayerRef = { id: "b1", name: "bot-Ash", bot: true };

function world(extra: Partial<MvpDeps> = {}) {
  let n = 7;
  return mvpRuntime({ content: mvpContent(), codex: true, seed: () => (n = (n * 1103515245 + 12345) >>> 0), ...extra });
}

/** Three candidates for the pair: rules borrowed from other units' awoken forms. */
function designs(content: MvpContent): FusionDesign[] {
  const [, , c, d, e] = content.units as UnitContent[];
  return [
    { name: "Leaner", lean: "a", recipe: structuredClone(c!.forms.awoken), flavour: "Leans one way." },
    { name: "Other", lean: "b", recipe: structuredClone(d!.forms.awoken) },
    { name: "Twist", lean: "twist", recipe: structuredClone(e!.forms.awoken), texts: { ru: { name: "Твист" } } },
  ];
}

function ready(content: MvpContent, x: string, y: string): CodexPair {
  return { ...codexPair(x, y), candidates: designs(content), preparedAt: "2026-10-13T03:00:00.000Z" };
}

/** A shop run for `who` holding two Awoken units, a and b, on its line. */
function holding(rt: ReturnType<typeof world>, who: PlayerRef, order: "ab" | "ba" = "ab"): MvpRunState {
  rt.store.addPlayer(who);
  const run = startRun(rt, who);
  const [a, b] = rt.content.units;
  const ua = lineUnitOf(a!, `${who.id}-a`, 3, rt.rules);
  const ub = lineUnitOf(b!, `${who.id}-b`, 3, rt.rules);
  const s: MvpRunState = { ...run, line: order === "ab" ? [ua, ub] : [ub, ua], bench: [] };
  rt.store.putRun(s);
  return s;
}

describe("the Codex kernel", () => {
  it("keys a pair without order and walks unforged → ready → discovered", () => {
    expect(pairKey("b", "a")).toBe(pairKey("a", "b"));
    const p = codexPair("zed", "ash");
    expect([p.a, p.b]).toEqual(["ash", "zed"]);
    expect(pairState(undefined)).toBe("unforged");
    expect(pairState(p)).toBe("unforged");
    const content = mvpContent();
    const r = ready(content, "ash", "zed");
    expect(pairState(r)).toBe("ready");
    const d = { ...r, claim: { pick: 2, by: maks, at: "2026-10-13T20:00:00.000Z" } };
    expect(pairState(d)).toBe("discovered");
    expect(canonicalDesign(d)!.name).toBe("Twist");
  });

  it("keeps a ready pair's designs sealed in its view; a discovered one shows its design and credit", () => {
    const content = mvpContent();
    const r = ready(content, "ash", "zed");
    const v = forgeView("zed", "ash", r, 2, true);
    expect(v).toEqual({ a: "ash", b: "zed", state: "ready", sealed: 3, wishes: 2, wished: true });
    expect(JSON.stringify(v)).not.toContain("Twist");
    const d = forgeView("ash", "zed", { ...r, claim: { pick: 1, by: kim, at: "t" } }, 0, false);
    expect(d.design!.name).toBe("Other");
    expect(d.claimedBy).toEqual(kim);
  });

  it("fuses into the design: summed stats and copies, the design's rule, final", () => {
    const content = mvpContent();
    const [a, b] = content.units;
    const ua = lineUnitOf(a!, "u1", 3);
    const ub = lineUnitOf(b!, "u2", 3);
    const design = designs(content)[2]!;
    const f = fuseDesigned(ub, ua, design, maks);
    expect(f.uid).toBe("u2");
    expect(f.kind).toBe("fused");
    expect(f.name).toBe("Twist");
    expect(f.stats).toEqual({ pwr: ua.stats.pwr + ub.stats.pwr, hp: ua.stats.hp + ub.stats.hp });
    expect(f.copies).toBe(6);
    expect(f.recipe).toEqual(design.recipe);
    expect([f.fusion!.first, f.fusion!.second]).toEqual([a!.id, b!.id].sort());
    expect(f.fusion!.discoveredBy).toEqual(maks);
    expect(() => fuseDesigned(lineUnitOf(a!, "s", 1), ub, design, null)).toThrow(/Awoken/);
  });
});

describe.each([
  ["memory", () => new MemoryMvpStore() as MvpStore],
  ["sqlite", () => new SqliteMvpStore(":memory:") as MvpStore],
])("the Codex store (%s)", (_name, make) => {
  it("claims a pair once, only with candidates, and keeps it", () => {
    const store = make();
    const content = mvpContent();
    store.putCodexPair(codexPair("a", "b"));
    expect(store.claimCodexPair("a", "b", { pick: 0, by: maks, at: "t1" })).toBe(false);
    store.putCodexPair(ready(content, "a", "b"));
    expect(store.claimCodexPair("a", "b", { pick: 1, by: maks, at: "t1" })).toBe(true);
    expect(store.claimCodexPair("a", "b", { pick: 2, by: kim, at: "t2" })).toBe(false);
    expect(store.codexPair("a", "b")!.claim).toEqual({ pick: 1, by: maks, at: "t1" });
    expect(store.codexPairs().map((p) => `${p.a}+${p.b}`)).toEqual(["a+b"]);
    expect(store.claimCodexPair("x", "y", { pick: 0, by: maks, at: "t" })).toBe(false);
  });

  it("counts one wish per player, pair and day, most wished first", () => {
    const store = make();
    const w = (a: string, b: string, playerId: string, seq = 1) => store.addWish({ a, b, playerId, seq, at: "t" });
    expect(w("a", "b", "p1")).toBe(true);
    expect(w("a", "b", "p1")).toBe(false);
    expect(w("a", "b", "p2")).toBe(true);
    expect(w("c", "d", "p1")).toBe(true);
    expect(w("a", "b", "p1", 2)).toBe(true);
    expect(store.wishes(1)).toEqual([{ a: "a", b: "b", count: 2 }, { a: "c", b: "d", count: 1 }]);
    expect(store.wishedBy("a", "b", "p1", 1)).toBe(true);
    expect(store.wishedBy("c", "d", "p2", 1)).toBe(false);
  });
});

describe("the forge in a run", () => {
  it("refuses an unforged pair; a wish puts it on tonight's list", () => {
    const rt = world();
    const run = holding(rt, maks);
    const [a, b] = rt.content.units;
    expect(() => decide(rt, run, { kind: "fuse", first: 0, second: 1 })).toThrow(/not forged yet: wish for it/);
    const app = createMvpApp(rt);
    return (async () => {
      const forge = await (await app.request(`/api/v1/runs/${run.runId}/forge`, { headers: { "X-Arena-Player": maks.id } })).json();
      expect(forge.pairs).toEqual([{ a: [a!.id, b!.id].sort()[0], b: [a!.id, b!.id].sort()[1], state: "unforged", wishes: 0, wished: false, first: 0, second: 1 }]);
      const res = await app.request("/api/v1/codex/wish", { method: "POST", headers: { "content-type": "application/json", "X-Arena-Player": maks.id }, body: JSON.stringify({ a: b!.id, b: a!.id }) });
      expect(res.status).toBe(200);
      expect(await res.json()).toMatchObject({ state: "unforged", wishes: 1, wished: true });
      expect(rt.store.wishes(rt.today().seq)).toHaveLength(1);
    })();
  });

  it("claims a ready pair with the picked design, credit only, final; the next fuse gets the canonical one", () => {
    const rt = world();
    const [a, b] = rt.content.units;
    rt.store.putCodexPair(ready(rt.content, a!.id, b!.id));
    const run = holding(rt, maks);
    expect(() => decide(rt, run, { kind: "fuse", first: 0, second: 1 })).toThrow(/pick one of the sealed designs/);
    expect(() => decide(rt, run, { kind: "fuse", first: 0, second: 1, pick: 3 })).toThrow(/no sealed design 3/);
    // A preview shows the result without claiming.
    const seen = preview(rt, run, { kind: "fuse", first: 0, second: 1, pick: 2 });
    expect(seen.run.line[0]!.name).toBe("Twist");
    expect(pairState(rt.store.codexPair(...([a!.id, b!.id].sort() as [string, string])))).toBe("ready");
    const done = decide(rt, run, { kind: "fuse", first: 0, second: 1, pick: 2 });
    const fused = done.run.line[0]!;
    expect(fused.name).toBe("Twist");
    expect(fused.fusion!.discoveredBy).toEqual(maks);
    const pair = rt.store.codexPair(...([a!.id, b!.id].sort() as [string, string]))!;
    expect(pair.claim).toMatchObject({ pick: 2, by: maks });
    // Kim, the other order: the canonical design, Maks's credit, no new claim.
    const other = holding(rt, kim, "ba");
    const again = decide(rt, other, { kind: "fuse", first: 0, second: 1, pick: 0 });
    expect(again.run.line[0]!.name).toBe("Twist");
    expect(again.run.line[0]!.fusion!.discoveredBy).toEqual(maks);
    expect(rt.store.codexPair(pair.a, pair.b)!.claim).toEqual(pair.claim);
    // its design uses no forged word: nothing is found
    expect(rt.store.foundWords()).toEqual([]);
  });

  it("finds a forged word with the first claim of a design that uses it (#1052)", () => {
    const rt = world();
    const [a, b] = rt.content.units;
    const p = ready(rt.content, a!.id, b!.id);
    // the word as the design's JSON names it; unhurt is in registry/forged-words.json
    p.candidates[1] = { ...p.candidates[1]!, uses: { kind: "unhurt" } } as FusionDesign;
    rt.store.putCodexPair(p);
    decide(rt, holding(rt, kim), { kind: "fuse", first: 0, second: 1, pick: 1 });
    expect(rt.store.foundWords()).toEqual([expect.objectContaining({ word: "unhurt", kind: "unhurt", pair: [a!.id, b!.id].sort().join("+"), playerId: kim.id })]);
    decide(rt, holding(rt, maks), { kind: "fuse", first: 0, second: 1 });
    expect(rt.store.foundWords().map((f) => f.playerId)).toEqual([kim.id]);
  });

  it("shows a ready pair's three designs on the forge of a run that holds it, and keeps them sealed elsewhere", async () => {
    const rt = world();
    const [a, b] = rt.content.units;
    rt.store.putCodexPair(ready(rt.content, a!.id, b!.id));
    const run = holding(rt, maks);
    const app = createMvpApp(rt);
    const forge = await (await app.request(`/api/v1/runs/${run.runId}/forge`, { headers: { "X-Arena-Player": maks.id } })).json();
    expect(forge.pairs[0]).toMatchObject({ state: "ready", sealed: 3, first: 0, second: 1 });
    expect(forge.pairs[0].candidates.map((d: FusionDesign) => d.name)).toEqual(["Leaner", "Other", "Twist"]);
    const sealed = await (await app.request(`/api/v1/codex/${b!.id}/${a!.id}`)).json();
    expect(sealed).toMatchObject({ state: "ready", sealed: 3 });
    expect(sealed.candidates).toBeUndefined();
  });

  // #1049 reopened, cases 1, 3 and 4: the shop marks only offers that bring a designed pair within reach.
  it("marks a shop offer that completes a designed pair, and nothing for an unforged one", async () => {
    const rt = world();
    rt.store.addPlayer(maks);
    const run = startRun(rt, maks);
    const [a, b, c, d] = rt.content.units;
    // On the board: a Awoken, b sleeping one copy short, c sleeping with one copy.
    const s: MvpRunState = {
      ...run,
      line: [lineUnitOf(a!, "m-a", 3, rt.rules), lineUnitOf(b!, "m-b", 2, rt.rules), lineUnitOf(c!, "m-c", 1, rt.rules)],
      bench: [],
      offers: [0, 1, 2, 3].map((slot) => ({ slot, unitId: [b, c, d, a][slot]!.id, tier: 1 as const, cost: 3 })),
    };
    rt.store.putRun(s);
    // Nothing forged: no marks (case 1).
    expect(forgeOffers(rt, s)).toEqual([]);
    rt.store.putCodexPair(ready(rt.content, a!.id, b!.id));
    rt.store.putCodexPair(ready(rt.content, a!.id, c!.id));
    rt.store.claimCodexPair(a!.id, c!.id, { pick: 2, by: kim, at: "t" });
    const offers = forgeOffers(rt, s);
    // b's third copy awakens it: one buy away, ready to claim (case 3).
    expect(offers).toContainEqual(expect.objectContaining({ slot: 0, partner: 0, state: "ready", awakens: true }));
    // c would be its second copy: close, discovered as Twist (case 4).
    expect(offers).toContainEqual(expect.objectContaining({ slot: 1, partner: 0, state: "discovered", awakens: false, design: expect.objectContaining({ name: "Twist" }) }));
    // d's pair with a is unforged, and a copy of a is already Awoken: neither shows.
    expect(offers.map((o) => o.slot).sort()).toEqual([0, 1]);
    const app = createMvpApp(rt);
    const forge = await (await app.request(`/api/v1/runs/${s.runId}/forge`, { headers: { "X-Arena-Player": maks.id } })).json();
    expect(forge.offers).toHaveLength(2);
    expect(forge.pairs).toEqual([]);
  });

  it("lets a bot fuse a ready pair with its chosen design, claiming nothing", () => {
    const rt = world();
    const [a, b] = rt.content.units;
    rt.store.putCodexPair(ready(rt.content, a!.id, b!.id));
    const run = holding(rt, botP);
    const done = decide(rt, run, { kind: "fuse", first: 0, second: 1, pick: 1 });
    expect(done.run.line[0]!.name).toBe("Other");
    expect(done.run.line[0]!.fusion!.discoveredBy).toBeNull();
    expect(pairState(rt.store.codexPair(...([a!.id, b!.id].sort() as [string, string])))).toBe("ready");
  });

  it("has bots pick a ready pair's design by fighting each against ghosts: the most wins, the same pick every time", () => {
    const rt = world();
    const [a, b] = rt.content.units;
    rt.store.putCodexPair(ready(rt.content, a!.id, b!.id));
    // Ghosts to fight: other players' lines this round.
    for (const [i, who] of [kim, maks].entries()) {
      rt.store.addPlayer(who);
      rt.store.addGhost({ ghostId: `g-${who.id}`, runId: `r-${who.id}`, player: who, round: 1, line: rt.content.units.slice(2 + i, 5 + i).map((u, k) => lineUnitOf(u, `${who.id}-g${k}`, 1, rt.rules)), contentVersion: rt.content.version, createdAt: "2026-10-13T00:00:00.000Z", rating: 1000 });
    }
    const run = holding(rt, botP);
    const d = botForge(rt, run, { kind: "fuse", first: 0, second: 1 });
    expect(d.pick).toBeGreaterThanOrEqual(0);
    expect(d.pick).toBeLessThan(3);
    expect(botForge(rt, run, { kind: "fuse", first: 0, second: 1 }).pick).toBe(d.pick);
    // An unforged pair is never fused by a bot; a discovered one keeps its decision as is.
    expect(codexForged(rt, run.line[0]!, run.line[1]!)).toBe(true);
    const [c, e] = rt.content.units.slice(5, 7);
    expect(codexForged(rt, lineUnitOf(c!, "x", 3, rt.rules), lineUnitOf(e!, "y", 3, rt.rules))).toBe(false);
  });

  it("plays a whole bot run with the Codex on: no fuse of an unforged pair is refused", () => {
    const rt = world();
    rt.store.addPlayer(botP);
    const run = playBotRun(rt, botP);
    expect(run.phase).toBe("over");
  });

  it("answers 404 for the Codex routes while it is off", async () => {
    const app = createMvpApp(world({ codex: false }));
    expect((await app.request("/api/v1/codex/a/b")).status).toBe(404);
  });
});
