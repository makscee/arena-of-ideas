import { describe, expect, it } from "vitest";
import { MVP_RULES, offersAt, type BattleRecord, type DayView, type DecisionResponse, type HomeView, type LineUnit, type PlayerRef, type RunView } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { portmanteau } from "./fusions.js";
import { mvpRuntime, type MvpDeps } from "./runtime.js";

function world(extra: Partial<MvpDeps> = {}) {
  let n = 7;
  const rt = mvpRuntime({ content: mvpContent(), seed: () => (n = (n * 1103515245 + 12345) >>> 0), ...extra });
  const app = createMvpApp(rt);
  const call = async <T>(method: string, path: string, body?: unknown, player?: string) => {
    const res = await app.request(`/api/v1${path}`, {
      method,
      headers: { "content-type": "application/json", ...(player ? { "X-Arena-Player": player } : {}) },
      ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
    });
    return { status: res.status, json: (await res.json()) as T };
  };
  return { rt, call };
}
const client = (extra: Partial<MvpDeps> = {}) => world(extra).call;

describe("MVP API thin path", () => {
  it("start run → buy → fight a ghost → result, until the run ends", async () => {
    const call = client();
    const { json: p } = await call<PlayerRef>("POST", "/players", { name: "Maks" });
    expect(p.name).toBe("Maks");
    const { json: run } = await call<RunView>("POST", "/runs", undefined, p.id);
    expect(run).toMatchObject({ round: 1, hearts: 5, gold: 10, phase: "shop", day: 1 });
    expect(Date.parse(run.startedAt)).not.toBeNaN();
    expect(run.endedAt).toBeUndefined();
    expect(run.offers).toHaveLength(3);

    let cur = run;
    let fights = 0;
    while (cur.phase !== "over") {
      while (cur.gold >= 3 && cur.offers.length > 0) {
        const r = await call<DecisionResponse>("POST", `/runs/${cur.runId}/decisions`, { kind: "buy", slot: 0 }, p.id);
        if (r.status !== 200) break; // line full
        cur = r.json.run;
      }
      const f = await call<DecisionResponse>("POST", `/runs/${cur.runId}/decisions`, { kind: "fight" }, p.id);
      expect(f.status).toBe(200);
      expect(f.json.fight?.outcome).toMatch(/win|loss|draw/);
      const b = await call<BattleRecord>("GET", `/battles/${f.json.fight!.battleId}`);
      expect(b.json.log.at(-1)?.type).toBe("BattleEnd");
      expect(b.json.player.id).toBe(p.id);
      cur = f.json.run;
      fights++;
      if (cur.phase === "shop") expect(cur.offers).toHaveLength(offersAt(MVP_RULES, cur.round));
    }
    expect(fights).toBeGreaterThan(0);
    expect(fights).toBeLessThanOrEqual(12);
    expect(cur.endedBy).toMatch(/out-of-hearts|no-champion/);
    expect(Date.parse(cur.endedAt!)).not.toBeNaN();
    const home = await call<HomeView>("GET", "/home", undefined, p.id);
    expect(home.json.activeRunId).toBeNull();
  });

  it("matches a saved ghost from another run at the same round, never your own", async () => {
    const call = client();
    const { json: a } = await call<PlayerRef>("POST", "/players", { name: "a" });
    const { json: b } = await call<PlayerRef>("POST", "/players", { name: "b" });
    const { json: ra } = await call<RunView>("POST", "/runs", undefined, a.id);
    await call("POST", `/runs/${ra.runId}/decisions`, { kind: "buy", slot: 0 }, a.id);
    const fa = await call<DecisionResponse>("POST", `/runs/${ra.runId}/decisions`, { kind: "fight" }, a.id);
    expect(fa.json.fight?.opponent.player.bot).toBe(true);
    const { json: rb } = await call<RunView>("POST", "/runs", undefined, b.id);
    await call("POST", `/runs/${rb.runId}/decisions`, { kind: "buy", slot: 0 }, b.id);
    const fb = await call<DecisionResponse>("POST", `/runs/${rb.runId}/decisions`, { kind: "fight" }, b.id);
    expect(fb.json.fight?.opponent.player.name).toBe("a");
  });

  it("never matches a player against their own earlier run", async () => {
    const call = client();
    const { json: p } = await call<PlayerRef>("POST", "/players", { name: "solo" });
    for (let i = 0; i < 2; i++) {
      const { json: r } = await call<RunView>("POST", "/runs", undefined, p.id);
      await call("POST", `/runs/${r.runId}/decisions`, { kind: "buy", slot: 0 }, p.id);
      let cur = r;
      while (cur.phase !== "over") {
        const f = await call<DecisionResponse>("POST", `/runs/${cur.runId}/decisions`, { kind: "fight" }, p.id);
        expect(f.json.fight?.opponent.player.id).not.toBe(p.id);
        cur = f.json.run;
      }
    }
  });

  it("applies parallel decisions on one run one after another", async () => {
    const call = client();
    const { json: p } = await call<PlayerRef>("POST", "/players", { name: "racer" });
    const { json: r } = await call<RunView>("POST", "/runs", undefined, p.id);
    const decide = (d: unknown) => call<DecisionResponse>("POST", `/runs/${r.runId}/decisions`, d, p.id);

    const buys = await Promise.all([decide({ kind: "buy", slot: 0 }), decide({ kind: "buy", slot: 0 })]);
    expect(buys.map((b) => b.status)).toEqual([200, 200]);
    const { json: bought } = await call<RunView>("GET", `/runs/${r.runId}`, undefined, p.id);
    expect(bought.gold).toBe(10 - 2 * 3);
    expect(bought.line.reduce((n, u) => n + u.copies, 0)).toBe(2);

    const fights = await Promise.all(Array.from({ length: 5 }, () => decide({ kind: "fight" })));
    expect(fights.map((f) => f.status)).toEqual([200, 200, 200, 200, 200]);
    expect(new Set(fights.map((f) => f.json.fight!.battleId)).size).toBe(5);
    const { json: after } = await call<RunView>("GET", `/runs/${r.runId}`, undefined, p.id);
    expect(after.fights.map((f) => f.round)).toEqual([1, 2, 3, 4, 5]);
    expect(after.round).toBe(after.phase === "over" ? 5 : 6);
    expect(after.hearts).toBe(5 - after.losses);
    expect(after.gold).toBe(10);
    expect(after.line).toEqual(bought.line);
  });

  it("tells hooks about every fight and the run's end", async () => {
    const seen: string[] = [];
    const call = client({ hooks: [{ onFight: (run, fight, battle) => seen.push(`fight ${fight.round} ${battle.battleId === fight.battleId} ${run.fights.length}`), onRunEnd: (run) => seen.push(`end ${run.endedBy}`) }] });
    const { json: p } = await call<PlayerRef>("POST", "/players", { name: "hooked" });
    let { json: cur } = await call<RunView>("POST", "/runs", undefined, p.id);
    await call("POST", `/runs/${cur.runId}/decisions`, { kind: "buy", slot: 0 }, p.id);
    while (cur.phase !== "over") cur = (await call<DecisionResponse>("POST", `/runs/${cur.runId}/decisions`, { kind: "fight" }, p.id)).json.run;
    expect(seen).toEqual([...cur.fights.map((f, i) => `fight ${f.round} true ${i + 1}`), `end ${cur.endedBy}`]);
  });

  it("previews a shop decision without writing anything", async () => {
    const seen: string[] = [];
    const call = client({ hooks: [{ onFight: () => seen.push("fight"), onRunEnd: () => seen.push("end") }] });
    const { json: p } = await call<PlayerRef>("POST", "/players", { name: "peek" });
    const { json: r } = await call<RunView>("POST", "/runs", undefined, p.id);
    const pv = await call<DecisionResponse>("POST", `/runs/${r.runId}/preview`, { kind: "buy", slot: 0 }, p.id);
    expect(pv.status).toBe(200);
    expect(pv.json.run).toMatchObject({ gold: 7, line: [{ unitId: r.offers[0]!.unitId }] });
    expect((await call<RunView>("GET", `/runs/${r.runId}`, undefined, p.id)).json).toEqual(r);
    const real = await call<DecisionResponse>("POST", `/runs/${r.runId}/decisions`, { kind: "buy", slot: 0 }, p.id);
    expect(real.json.run).toEqual(pv.json.run);
    expect((await call("POST", `/runs/${r.runId}/preview`, { kind: "fight" }, p.id)).status).toBe(400);
    expect((await call("POST", `/runs/${r.runId}/preview`, { kind: "fuse", first: 0, second: 1 }, p.id)).status).toBe(409);
    expect((await call("POST", `/runs/${r.runId}/preview`, { kind: "buy", slot: 9 }, p.id)).status).toBe(409);
    expect((await call("POST", `/runs/${r.runId}/preview`, { kind: "buy", slot: 0 })).status).toBe(401);
    expect(seen).toEqual([]);
    const fusions = await call<unknown[]>("GET", "/fusions");
    expect(fusions).toEqual({ status: 200, json: [] });
  });

  it("fuses two Awoken units: hooks see the decision and the fuse, a preview hides the new pair's name and calls none", async () => {
    const seen: string[] = [];
    const { rt, call } = world({
      hooks: [
        {
          onDecision: (before, d, after) => seen.push(`decision ${d.kind} ${before.line.length}→${after.line.length}`),
          onFuse: (run, fused, ctx) => seen.push(`fuse ${fused.uid} ${fused.name} ${ctx.discoveredBy?.name} ${run.line.length}`),
        },
      ],
    });
    const { json: p } = await call<PlayerRef>("POST", "/players", { name: "fuser" });
    const { json: r } = await call<RunView>("POST", "/runs", undefined, p.id);
    const [a, b] = rt.content.units;
    const run = rt.store.run(r.runId)!;
    rt.store.putRun({ ...run, line: [lineUnitOf(a!, "u1", 3, rt.rules), lineUnitOf(b!, "u2", 3, rt.rules)], nextUid: 3 });

    const fuse = { kind: "fuse", first: 1, second: 0 };
    const pv = await call<DecisionResponse>("POST", `/runs/${r.runId}/preview`, fuse, p.id);
    expect(pv.status).toBe(200);
    expect(seen).toEqual([]);
    const name = portmanteau(b!.name, a!.name);
    // Nobody has fused this pair: the preview's response doesn't carry its name.
    expect(pv.json.run.line).toMatchObject([{ uid: "u2", kind: "fused", name: "", fusion: { first: b!.id, second: a!.id, name: "", discoveredBy: p } }]);
    expect(JSON.stringify(pv.json)).not.toContain(name);
    const done = await call<DecisionResponse>("POST", `/runs/${r.runId}/decisions`, fuse, p.id);
    expect(done.status).toBe(200);
    // The same as the preview, except the name the fuse gave it.
    const named = (u: LineUnit) => ({ ...u, name, fusion: { ...u.fusion!, name } });
    expect(done.json.run).toEqual({ ...pv.json.run, line: pv.json.run.line.map(named) });
    expect(done.json.run.line).toMatchObject([{ uid: "u2", kind: "fused", name, fusion: { first: b!.id, second: a!.id, name, discoveredBy: p } }]);
    expect(seen).toEqual(["decision fuse 2→1", `fuse u2 ${name} fuser 1`]);
    // Slice 10's hook records the discovery; GET /fusions lists it.
    expect((await call("GET", "/fusions")).json).toEqual([{ first: b!.id, second: a!.id, name, discoveredBy: p, discoveredAt: expect.any(String), nameSource: "fallback" }]);
    // Once fused, the pair previews with its name and credit.
    rt.store.putRun({ ...rt.store.run(r.runId)!, line: [lineUnitOf(b!, "u4", 3, rt.rules), lineUnitOf(a!, "u5", 3, rt.rules)], nextUid: 6 });
    const again = await call<DecisionResponse>("POST", `/runs/${r.runId}/preview`, { kind: "fuse", first: 0, second: 1 }, p.id);
    expect(again.json.run.line).toMatchObject([{ uid: "u4", name, fusion: { name, discoveredBy: p } }]);
  });

  it("keeps one day: runs start on it, /day and /home show it", async () => {
    const { rt, call } = world();
    const { json: p } = await call<PlayerRef>("POST", "/players", { name: "day" });
    const day = await call<DayView>("GET", "/day");
    expect(day.status).toBe(200);
    expect(day.json).toMatchObject({ seq: 1, champion: null, slayers: 0, lastPlayoff: null });
    expect((await call<HomeView>("GET", "/home", undefined, p.id)).json.day).toEqual(day.json);
    expect((await call<RunView>("POST", "/runs", undefined, p.id)).json.day).toBe(1);
    expect(rt.store.currentDay()?.seq).toBe(1);
    expect(rt.today()).toEqual(rt.store.currentDay());
  });

  it("serves dev tools only in dev", async () => {
    expect((await client()("POST", "/dev/end-day")).status).toBe(404);
    const dev = client({ dev: true });
    const ended = await dev<DayView>("POST", "/dev/end-day");
    expect(ended.status).toBe(200);
    expect(ended.json).toMatchObject({ seq: 2, slayers: 0, lastPlayoff: { seq: 1, entrants: [], winner: null } });
  });

  it("shows a run only to its player", async () => {
    const call = client();
    const { json: p } = await call<PlayerRef>("POST", "/players", { name: "owner" });
    const { json: q } = await call<PlayerRef>("POST", "/players", { name: "snoop" });
    const { json: r } = await call<RunView>("POST", "/runs", undefined, p.id);
    expect((await call("GET", `/runs/${r.runId}`, undefined, p.id)).status).toBe(200);
    expect((await call("GET", `/runs/${r.runId}`, undefined, q.id)).status).toBe(401);
    expect((await call("GET", `/runs/${r.runId}`)).status).toBe(401);
  });

  it("rejects bad input with 4xx", async () => {
    const call = client();
    expect((await call("POST", "/players", { name: "" })).status).toBe(400);
    expect((await call("POST", "/runs")).status).toBe(401);
    const { json: p } = await call<PlayerRef>("POST", "/players", { name: "x" });
    const { json: r } = await call<RunView>("POST", "/runs", undefined, p.id);
    expect((await call("POST", `/runs/${r.runId}/decisions`, { kind: "sell", index: 0 }, p.id)).status).toBe(409);
    expect((await call("POST", `/runs/${r.runId}/decisions`, { kind: "buy", slot: 9 }, p.id)).status).toBe(409);
    expect((await call("POST", `/runs/${r.runId}/decisions`, { kind: "fuse", first: 0, second: 1 }, p.id)).status).toBe(409);
  });
});
