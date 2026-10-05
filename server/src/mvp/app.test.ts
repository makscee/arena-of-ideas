import { describe, expect, it } from "vitest";
import type { BattleRecord, DecisionResponse, HomeView, PlayerRef, RunView } from "../../../src/mvp/contract.js";
import { createMvpApp, type MvpDeps } from "./app.js";
import { mvpContent } from "./content.js";

function client(extra: Partial<MvpDeps> = {}) {
  let n = 7;
  const app = createMvpApp({ content: mvpContent(), seed: () => (n = (n * 1103515245 + 12345) >>> 0), ...extra });
  const call = async <T>(method: string, path: string, body?: unknown, player?: string) => {
    const res = await app.request(`/api/v1${path}`, {
      method,
      headers: { "content-type": "application/json", ...(player ? { "X-Arena-Player": player } : {}) },
      ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
    });
    return { status: res.status, json: (await res.json()) as T };
  };
  return call;
}

describe("MVP API thin path", () => {
  it("start run → buy → fight a ghost → result, until the run ends", async () => {
    const call = client();
    const { json: p } = await call<PlayerRef>("POST", "/players", { name: "Maks" });
    expect(p.name).toBe("Maks");
    const { json: run } = await call<RunView>("POST", "/runs", undefined, p.id);
    expect(run).toMatchObject({ round: 1, hearts: 5, gold: 10, phase: "shop", day: 1 });
    expect(Date.parse(run.startedAt)).not.toBeNaN();
    expect(run.endedAt).toBeUndefined();
    expect(run.offers).toHaveLength(5);

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

  it("tells hooks about every fight and the run's end", async () => {
    const seen: string[] = [];
    const call = client({ hooks: [{ onFight: (run, fight, battle) => seen.push(`fight ${fight.round} ${battle.battleId === fight.battleId} ${run.fights.length}`), onRunEnd: (run) => seen.push(`end ${run.endedBy}`) }] });
    const { json: p } = await call<PlayerRef>("POST", "/players", { name: "hooked" });
    let { json: cur } = await call<RunView>("POST", "/runs", undefined, p.id);
    await call("POST", `/runs/${cur.runId}/decisions`, { kind: "buy", slot: 0 }, p.id);
    while (cur.phase !== "over") cur = (await call<DecisionResponse>("POST", `/runs/${cur.runId}/decisions`, { kind: "fight" }, p.id)).json.run;
    expect(seen).toEqual([...cur.fights.map((f, i) => `fight ${f.round} true ${i + 1}`), `end ${cur.endedBy}`]);
  });

  it("rejects bad input with 4xx", async () => {
    const call = client();
    expect((await call("POST", "/players", { name: "" })).status).toBe(400);
    expect((await call("POST", "/runs")).status).toBe(401);
    const { json: p } = await call<PlayerRef>("POST", "/players", { name: "x" });
    const { json: r } = await call<RunView>("POST", "/runs", undefined, p.id);
    expect((await call("POST", `/runs/${r.runId}/decisions`, { kind: "fight" }, p.id)).status).toBe(409);
    expect((await call("POST", `/runs/${r.runId}/decisions`, { kind: "buy", slot: 9 }, p.id)).status).toBe(409);
    expect((await call("POST", `/runs/${r.runId}/decisions`, { kind: "fuse", first: 0, second: 1 }, p.id)).status).toBe(501);
  });
});
