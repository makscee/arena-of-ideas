import { describe, expect, it } from "vitest";
import type { BattleRecord, DecisionResponse, HomeView, PlayerRef, RunView } from "../../../src/mvp/contract.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";

function client() {
  let n = 7;
  const app = createMvpApp({ content: mvpContent(), seed: () => (n = (n * 1103515245 + 12345) >>> 0) });
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
    expect(run).toMatchObject({ round: 1, hearts: 5, gold: 10, phase: "shop" });
    expect(run.offers).toHaveLength(5);

    let cur = run;
    let fights = 0;
    while (cur.phase === "shop") {
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
      cur = f.json.run;
      fights++;
    }
    expect(fights).toBeGreaterThan(0);
    expect(fights).toBeLessThanOrEqual(12);
    expect(cur.endedBy).toMatch(/out-of-hearts|no-champion/);
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
