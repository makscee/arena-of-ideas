// The Codex screens' view (#1051): GET /codex from the world's own pairs,
// wishes, claims and words.
import { describe, expect, it } from "vitest";
import { codexPair, type FusionDesign } from "../../../src/mvp/codex.js";
import type { PlayerRef, UnitContent } from "../../../src/mvp/contract.js";
import type { ForgeView } from "../../../src/mvp/forge-view.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import type { MvpRunState } from "../../../src/mvp/run.js";
import { createMvpApp } from "./app.js";
import { codexOverview } from "./codex-view.js";
import { mvpContent } from "./content.js";
import { decide, startRun } from "./runs.js";
import { mvpRuntime, type MvpDeps } from "./runtime.js";

const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };

function world(extra: Partial<MvpDeps> = {}) {
  let n = 7;
  return mvpRuntime({ content: mvpContent(), codex: true, seed: () => (n = (n * 1103515245 + 12345) >>> 0), ...extra });
}

function designs(units: UnitContent[]): FusionDesign[] {
  const [, , c, d, e] = units;
  return [
    { name: "Leaner", lean: "a", recipe: structuredClone(c!.forms.awoken), flavour: "Leans one way." },
    { name: "Other", lean: "b", recipe: structuredClone(d!.forms.awoken) },
    { name: "Twist", lean: "twist", recipe: structuredClone(e!.forms.awoken), flavour: "Turns.", texts: { ru: { name: "Твист", flavour: "Поворот." } } },
  ];
}

const get = async (app: ReturnType<typeof createMvpApp>, path: string, who = maks) => (await app.request(path, { headers: { "X-Arena-Player": who.id } })).json();

describe("GET /codex", () => {
  it("lists a ready pair, then the claim as discovered by its claimer, with the design's RU name", async () => {
    const rt = world();
    rt.store.addPlayer(maks);
    const [a, b] = rt.content.units;
    rt.store.putCodexPair({ ...codexPair(a!.id, b!.id), candidates: designs(rt.content.units), preparedAt: rt.now().toISOString() });
    const app = createMvpApp(rt);
    let view = (await get(app, "/api/v1/codex")) as ForgeView;
    expect(view.viewer).toBe("Maks");
    expect(view.units.map((u) => u.id)).toEqual(rt.content.units.map((u) => u.id));
    expect(view.units[0]!.rule.en.length).toBeGreaterThan(0);
    expect(view.pairs).toEqual([{ a: [a!.id, b!.id].sort()[0], b: [a!.id, b!.id].sort()[1], state: "ready" }]);
    expect(view.feed).toMatchObject([{ kind: "arrived", n: 1 }]);

    // The claim happens at the forge in a run: fuse the pair with a pick.
    const run = startRun(rt, maks);
    const s: MvpRunState = { ...run, line: [lineUnitOf(a!, "m-a", 3, rt.rules), lineUnitOf(b!, "m-b", 3, rt.rules)], bench: [] };
    rt.store.putRun(s);
    decide(rt, s, { kind: "fuse", first: 0, second: 1, pick: 2 });
    view = (await get(app, "/api/v1/codex")) as ForgeView;
    expect(view.pairs[0]).toMatchObject({ state: "discovered", claimedBy: "Maks", design: { name: { en: "Twist", ru: "Твист" }, flavour: { en: "Turns.", ru: "Поворот." }, lean: "twist" } });
    expect(view.pairs[0]!.design!.rule.en.length).toBeGreaterThan(0);
    expect(view.feed.map((f) => f.kind).sort()).toEqual(["arrived", "claimed"]);
    expect(view.feed.find((f) => f.kind === "claimed")).toMatchObject({ by: "Maks", name: { en: "Twist", ru: "Твист" } });
  });

  it("a wish shows as wished, with today's count, for the one who wished", async () => {
    const rt = world();
    rt.store.addPlayer(maks);
    rt.store.addPlayer({ id: "p2", name: "Kim", bot: false });
    const [a, , c] = rt.content.units;
    const app = createMvpApp(rt);
    const res = await app.request("/api/v1/codex/wish", { method: "POST", headers: { "content-type": "application/json", "X-Arena-Player": maks.id }, body: JSON.stringify({ a: a!.id, b: c!.id }) });
    expect(res.status).toBe(200);
    const mine = (await get(app, "/api/v1/codex")) as ForgeView;
    expect(mine.pairs).toEqual([{ ...[a!.id, c!.id].sort().reduce((o, id, i) => ({ ...o, [i ? "b" : "a"]: id }), {}), state: "unforged", wishes: 1, wished: true }]);
    const kims = (await get(app, "/api/v1/codex", { id: "p2", name: "Kim", bot: false })) as ForgeView;
    expect(kims.pairs[0]).toMatchObject({ wishes: 1, wished: false });
  });

  it("the Dictionary: a forged word nobody found is sealed and counted", () => {
    const rt = world();
    const view = codexOverview(rt, maks, [{ word: "unhurt", kind: "unhurt", card: 1052 }]);
    expect(view.forgedNotFound).toBe(1);
    expect(view.forged).toEqual([{ en: "unhurt", ru: "unhurt" }]);
    expect(view.words).toEqual([]);
  });

  it("is 404 while the Codex is off", async () => {
    const app = createMvpApp(world({ codex: false }));
    expect((await app.request("/api/v1/codex")).status).toBe(404);
  });
});
