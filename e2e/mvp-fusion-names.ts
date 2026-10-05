// Fusion names, end to end (mission #574, slice 10): the MVP server in-process
// with its jobs and the namer at ARENA_NAMER_URL, driven through the HTTP
// routes. A player's line is handed two Awoken units (no shop luck needed),
// then: fuse → named, "discovered by" them; the same pair in another run → the
// same name and credit; /fusions lists it. With the model down, a fuse still
// names the result at once with the portmanteau.
//   ARENA_NAMER_URL=http://127.0.0.1:8792/v1/chat/completions npm run -s mvp:fusion-names
import type { DecisionResponse, FusionDiscovery, PlayerRef, RunView } from "../src/mvp/contract.js";
import { lineUnitOf } from "../src/mvp/forms.js";
import { createMvpApp } from "../server/src/mvp/app.js";
import { mvpContent } from "../server/src/mvp/content.js";
import { startMvpJobs } from "../server/src/mvp/jobs.js";
import { mvpRuntime } from "../server/src/mvp/runtime.js";

const rt = mvpRuntime({ content: mvpContent() });
const stopJobs = startMvpJobs(rt);
const app = createMvpApp(rt);
const call = async <T>(method: string, path: string, body?: unknown, player?: string): Promise<T> => {
  const res = await app.request(`/api/v1${path}`, {
    method,
    headers: { "content-type": "application/json", ...(player ? { "X-Arena-Player": player } : {}) },
    ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
  });
  if (!res.ok) throw new Error(`${method} ${path}: ${res.status} ${await res.text()}`);
  return (await res.json()) as T;
};
const [a, b] = rt.content.units;

async function fuseAs(name: string): Promise<void> {
  const p = await call<PlayerRef>("POST", "/players", { name });
  const r = await call<RunView>("POST", "/runs", undefined, p.id);
  const run = rt.store.run(r.runId)!;
  rt.store.putRun({ ...run, line: [lineUnitOf(a!, "u1", 3, rt.rules), lineUnitOf(b!, "u2", 3, rt.rules)], nextUid: 3 });
  // Any decision queues model names for the line's fusable pairs; give the model a moment.
  await call("POST", `/runs/${r.runId}/decisions`, { kind: "reorder", from: 0, to: 0 }, p.id);
  await new Promise((ok) => setTimeout(ok, 3000));
  const t = Date.now();
  const { run: after } = await call<DecisionResponse>("POST", `/runs/${r.runId}/decisions`, { kind: "fuse", first: 0, second: 1 }, p.id);
  const f = after.line[0]!.fusion!;
  console.log(`${name} fuses ${a!.name} + ${b!.name} → "${f.name}", discovered by @${f.discoveredBy?.name} (${Date.now() - t} ms)`);
}

const url = process.env.ARENA_NAMER_URL;
console.log(url ? `namer: ${url}` : "namer: none (ARENA_NAMER_URL unset): portmanteau only");
await fuseAs("maks");
await fuseAs("eva");
const list = await call<FusionDiscovery[]>("GET", "/fusions");
console.log("GET /fusions:", JSON.stringify(list.map((f) => ({ pair: `${f.first}+${f.second}`, name: f.name, by: f.discoveredBy?.name ?? null, source: f.nameSource }))));
stopJobs();
