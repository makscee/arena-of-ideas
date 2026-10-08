import { copyFileSync, mkdirSync, mkdtempSync, readdirSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { fileURLToPath } from "node:url";
import { describe, expect, it } from "vitest";
import type { DecisionResponse, LineUnit, PlayerRef, RunView, StatsView } from "../../../src/mvp/contract.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { mvpRuntime } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { decide, startRun } from "./runs.js";
import { isWalkover, lineUnitIds } from "./stats.js";

function world() {
  let n = 11;
  const rt = mvpRuntime({ content: mvpContent(), seed: () => (n = (n * 1103515245 + 12345) >>> 0) });
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

/** One run: buy offer 0, then fight until it ends. */
async function playOne(call: ReturnType<typeof world>["call"], name: string): Promise<RunView> {
  const { json: p } = await call<PlayerRef>("POST", "/players", { name });
  let { json: cur } = await call<RunView>("POST", "/runs", undefined, p.id);
  cur = (await call<DecisionResponse>("POST", `/runs/${cur.runId}/decisions`, { kind: "buy", slot: 0 }, p.id)).json.run;
  while (cur.phase !== "over") cur = (await call<DecisionResponse>("POST", `/runs/${cur.runId}/decisions`, { kind: "fight" }, p.id)).json.run;
  return cur;
}

describe("basic stats (slice 11)", () => {
  it("counts win and pick rates as runs go", async () => {
    const { rt, call } = world();
    expect((await call<StatsView>("GET", "/stats")).json).toEqual({ units: [], champions: rt.store.champions(), fusions: [] });

    const a = await playOne(call, "one");
    const unit = a.line[0]!.unitId;
    const wins = a.fights.filter((f) => f.outcome === "win").length;
    const s1 = (await call<StatsView>("GET", "/stats")).json;
    expect(s1.units).toEqual([{ unitId: unit, winRate: wins / a.fights.length, pickRate: 1, runs: 1 }]);

    // A second run that ends with another unit halves the first one's pick rate.
    const b = await playOne(call, "two");
    const s2 = (await call<StatsView>("GET", "/stats")).json;
    const other = b.line[0]!.unitId;
    expect(s2.units.find((u) => u.unitId === unit)?.pickRate).toBe(other === unit ? 1 : 0.5);
    expect(rt.store.unitTallies(rt.content.version).runs).toBe(2);
  });

  it("also tallies per day (M2-1): fights, wins, finished runs and picks", async () => {
    const { rt, call } = world();
    const a = await playOne(call, "one");
    const unit = a.line[0]!.unitId;
    const day = rt.store.dayTallies(rt.today().seq);
    expect(day.runs).toBe(1);
    const v = rt.store.unitTallies(rt.content.version).units.find((u) => u.unitId === unit)!;
    expect(day.units.find((u) => u.unitId === unit)).toEqual({ ...v, picks: 1 });
    expect(rt.store.dayTallies(rt.today().seq + 1)).toEqual({ runs: 0, units: [] });
  });

  it("rates units over the last 14 days' tallies, whatever pool they were played on, live units only (M2-2)", async () => {
    const { rt } = world();
    rt.store.putDay({ ...rt.today(), seq: 20 });
    const live = rt.content.units[0]!.id;
    const tally = (fights: number, wins: number, runs: number) => ({ unitId: live, fights, wins, runs, picks: 0 });
    rt.store.addDayTallies(6, { runs: 9, units: [tally(9, 9, 9)] }); // 15 days ago: out of the window
    rt.store.addDayTallies(7, { runs: 2, units: [tally(2, 0, 1)] });
    rt.store.addDayTallies(20, { runs: 2, units: [tally(2, 1, 0), { unitId: "retired", fights: 9, wins: 9, runs: 2, picks: 0 }] });
    rt.store.addUnitTallies(rt.content.version, { runs: 4, units: [{ unitId: live, fights: 40, wins: 40, runs: 4 }] });
    const res = await createMvpApp(rt).request("/api/v1/stats");
    expect(((await res.json()) as StatsView).units).toEqual([{ unitId: live, winRate: 0.25, pickRate: 0.25, runs: 1 }]);
  });

  it("a world with only per-version tallies keeps its rates: the migration puts them on the current day once, without counting a day tally twice (M2-2)", async () => {
    const dir = mkdtempSync(join(tmpdir(), "arena-789-"));
    try {
      const sqlDir = fileURLToPath(new URL("./sql/", import.meta.url));
      const oldSql = join(dir, "sql");
      mkdirSync(oldSql);
      // The live world before M2-2: every migration up to the backfill.
      for (const f of readdirSync(sqlDir)) if (f.endsWith(".sql") && f < "m2-02b") copyFileSync(join(sqlDir, f), join(oldSql, f));
      const path = join(dir, "world.db");
      const content = mvpContent();
      const [x, y] = [content.units[0]!.id, content.units[1]!.id];
      const old = new SqliteMvpStore(path, oldSql);
      const oldRt = mvpRuntime({ content, store: old, seed: () => 1 });
      old.putDay({ ...oldRt.today(), seq: 5 });
      // Before M2-1: per-version tallies only, on an older content version and this one.
      old.addUnitTallies("mvp-older", { runs: 4, units: [{ unitId: x, fights: 10, wins: 6, runs: 3 }, { unitId: y, fights: 3, wins: 3, runs: 1 }] });
      old.addUnitTallies(content.version, { runs: 1, units: [{ unitId: x, fights: 4, wins: 2, runs: 1 }] });
      // Since M2-1 the same hooks also wrote today's day tallies: a part of the above.
      old.addDayTallies(5, { runs: 1, units: [{ unitId: x, fights: 2, wins: 1, runs: 1, picks: 3 }] });
      old.close();

      for (let open = 0; open < 2; open++) {
        const store = new SqliteMvpStore(path);
        expect(store.dayTallies(5)).toEqual({ runs: 5, units: [{ unitId: x, fights: 14, wins: 8, runs: 4, picks: 3 }, { unitId: y, fights: 3, wins: 3, runs: 1, picks: 0 }] });
        const rt = mvpRuntime({ content, store, seed: () => 1 });
        const res = await createMvpApp(rt).request("/api/v1/stats");
        expect(((await res.json()) as StatsView).units).toEqual([
          { unitId: x, winRate: 8 / 14, pickRate: 4 / 5, runs: 4 },
          { unitId: y, winRate: 1, pickRate: 1 / 5, runs: 1 },
        ]);
        store.close();
      }
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  });

  it("leaves walkovers (an empty enemy line) out of the win rates", () => {
    const { rt } = world();
    const p: PlayerRef = { id: "p-walk", name: "walk", bot: false };
    rt.store.addPlayer(p);
    let run = startRun(rt, p);
    run = rt.store.run(decide(rt, run, { kind: "buy", slot: 0 }).run.runId)!;
    const unit = run.line[0]!.unitId;
    const empty = { ...run, opponent: { ...run.opponent!, line: [] } };
    rt.store.putRun(empty);
    const walk = decide(rt, empty, { kind: "fight" });
    expect(walk.fight?.outcome).toBe("win");
    expect(rt.store.unitTallies(rt.content.version).units).toEqual([]);
    // a real fight still counts
    const real = decide(rt, rt.store.run(run.runId)!, { kind: "fight" });
    expect(rt.store.unitTallies(rt.content.version).units).toEqual([{ unitId: unit, fights: 1, wins: real.fight!.outcome === "win" ? 1 : 0, runs: 0 }]);
    expect(isWalkover({ teamA: [], teamB: [] })).toBe(true);
  });

  it("counts a fused unit for both parts, each unit once per line", () => {
    const line = [
      { kind: "fused", unitId: "a", fusion: { first: "a", second: "b", name: "Ab", discoveredBy: null } },
      { kind: "unit", unitId: "a" },
      { kind: "unit", unitId: "c" },
    ] as LineUnit[];
    expect(lineUnitIds(line)).toEqual(["a", "b", "c"]);
  });
});
