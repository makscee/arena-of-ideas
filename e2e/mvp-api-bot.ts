// MVP API bot (mission #574 verification ladder): plays N full runs through
// the HTTP API and fails on any error. Without --url it starts the MVP server
// in-process on a free port, with the background jobs, as main.ts does.
//   npm run mvp:bot -- [--runs 50] [--url http://127.0.0.1:8791/arena]
import { serve } from "@hono/node-server";
import type { AddressInfo } from "node:net";
import { MVP_RULES, type DecisionResponse, type MvpContent, type PlayerRef, type RunView } from "../src/mvp/contract.js";
import { botDecision } from "../server/src/mvp/bots.js";
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
  // Slice 6 seeds the champion once the model named its fusions (or failed).
  for (let waited = 0; !rt.store.currentChampion() && waited < 120_000; waited += 50) await new Promise((r) => setTimeout(r, 50));
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

const content = await call<MvpContent>("GET", "/content");
const tally = { decisions: {} as Record<string, number>, runs: 0, fights: 0, wins: 0, losses: 0, draws: 0, crowns: 0, ends: {} as Record<string, number>, ratingDelta: 0, errors: 0 };
const t0 = Date.now();
for (let i = 0; i < runs; i++) {
  try {
    player = (await call<PlayerRef>("POST", "/players", { name: `bot-${i % 5}` })).id;
    let run = await call<RunView>("POST", "/runs");
    // Plays as slice 6's bots do (botDecision), so the API sees every kind of
    // decision; fights until the run is over, the Crown included.
    let guard = 0;
    let round = run.round;
    let rerolled = 0;
    while (run.phase !== "over" && guard++ < 1000) {
      if (run.round !== round) {
        round = run.round;
        rerolled = 0;
      }
      const d = botDecision(run, content, MVP_RULES, rerolled);
      if (d.kind === "reroll") rerolled++;
      tally.decisions[d.kind] = (tally.decisions[d.kind] ?? 0) + 1;
      const r = await call<DecisionResponse>("POST", `/runs/${run.runId}/decisions`, d);
      run = r.run;
      if (!r.fight) continue;
      await call("GET", `/battles/${r.fight.battleId}`);
      tally.fights++;
      if (r.fight.kind === "crown") tally.crowns++;
      tally[r.fight.outcome === "win" ? "wins" : r.fight.outcome === "loss" ? "losses" : "draws"]++;
    }
    if (run.phase !== "over") throw new Error(`run ${run.runId} did not end`);
    // A human's run moves the rating exactly once, at its end.
    if (!run.rating) throw new Error(`run ${run.runId} ended (${run.endedBy}) without a rating change`);
    tally.ratingDelta += run.rating.after - run.rating.before;
    tally.ends[run.endedBy!] = (tally.ends[run.endedBy!] ?? 0) + 1;
    tally.runs++;
  } catch (e) {
    tally.errors++;
    console.error(`run ${i}: ${(e as Error).message}`);
  }
}
close();
// Slice 6 seeds a champion at start: a run that survives round 12 meets it.
if (tally.ends["no-champion"]) {
  tally.errors++;
  console.error(`${tally.ends["no-champion"]} runs found no champion at the Crown`);
}
console.log(JSON.stringify({ ...tally, ms: Date.now() - t0 }));
process.exit(tally.errors === 0 && tally.runs === runs ? 0 : 1);
