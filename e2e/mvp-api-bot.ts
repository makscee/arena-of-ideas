// MVP API bot (mission #574 verification ladder): plays N full runs through
// the HTTP API and fails on any error. Without --url it starts the MVP server
// in-process on a free port, with the background jobs, as main.ts does.
//   npm run mvp:bot -- [--runs 50] [--url http://127.0.0.1:8791/arena]
import { serve } from "@hono/node-server";
import type { AddressInfo } from "node:net";
import type { DecisionResponse, PlayerRef, RunView } from "../src/mvp/contract.js";
import { createMvpApp } from "../server/src/mvp/app.js";
import { mvpContent } from "../server/src/mvp/content.js";
import { startMvpJobs } from "../server/src/mvp/jobs.js";
import { mvpRuntime } from "../server/src/mvp/runtime.js";

const args = process.argv.slice(2);
const opt = (name: string) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 ? args[i + 1] : undefined;
};
const runs = Number(opt("runs") ?? 50);

let base = opt("url");
let close = () => {};
if (!base) {
  const rt = mvpRuntime({ content: mvpContent() });
  const server = serve({ fetch: createMvpApp(rt).fetch, port: 0, hostname: "127.0.0.1" });
  const stopJobs = startMvpJobs(rt);
  await new Promise((r) => server.once("listening", r));
  base = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
  close = () => {
    stopJobs();
    server.close();
  };
}
base = base.replace(/\/$/, "") + "/api/v1";

let player = "";
async function call<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(base + path, {
    method,
    headers: { "content-type": "application/json", ...(player ? { "X-Arena-Player": player } : {}) },
    ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
    signal: AbortSignal.timeout(10_000),
  });
  const json = await res.json();
  if (!res.ok) throw new Error(`${method} ${path} ${JSON.stringify(body ?? "")} → ${res.status} ${JSON.stringify(json)}`);
  return json as T;
}

const tally = { runs: 0, fights: 0, wins: 0, losses: 0, draws: 0, ends: {} as Record<string, number>, errors: 0 };
const t0 = Date.now();
for (let i = 0; i < runs; i++) {
  try {
    player = (await call<PlayerRef>("POST", "/players", { name: `bot-${i % 5}` })).id;
    let run = await call<RunView>("POST", "/runs");
    let guard = 0;
    while (run.phase !== "over" && guard++ < 500) {
      // Buy what the gold allows (prefer copies of what's on the line), reroll once if broke on offers, fight.
      for (;;) {
        const owned = new Set(run.line.map((u) => u.unitId));
        const affordable = run.offers.filter((o) => o.cost <= run.gold && (run.line.length < 5 || owned.has(o.unitId)));
        if (affordable.length === 0) break;
        const pick = affordable.find((o) => owned.has(o.unitId)) ?? affordable[0]!;
        run = (await call<DecisionResponse>("POST", `/runs/${run.runId}/decisions`, { kind: "buy", slot: pick.slot })).run;
      }
      if (run.line.length > 1 && run.round % 3 === 0) {
        run = (await call<DecisionResponse>("POST", `/runs/${run.runId}/decisions`, { kind: "reorder", from: run.line.length - 1, to: 0 })).run;
      }
      const f = await call<DecisionResponse>("POST", `/runs/${run.runId}/decisions`, { kind: "fight" });
      await call("GET", `/battles/${f.fight!.battleId}`);
      tally.fights++;
      tally[f.fight!.outcome === "win" ? "wins" : f.fight!.outcome === "loss" ? "losses" : "draws"]++;
      run = f.run;
    }
    if (run.phase !== "over") throw new Error(`run ${run.runId} did not end`);
    tally.ends[run.endedBy!] = (tally.ends[run.endedBy!] ?? 0) + 1;
    tally.runs++;
  } catch (e) {
    tally.errors++;
    console.error(`run ${i}: ${(e as Error).message}`);
  }
}
close();
console.log(JSON.stringify({ ...tally, ms: Date.now() - t0 }));
process.exit(tally.errors === 0 && tally.runs === runs ? 0 : 1);
