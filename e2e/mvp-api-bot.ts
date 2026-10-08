// MVP API bot (mission #574 verification ladder): plays N full runs through
// the HTTP API and fails on any error. Without --url it starts the MVP server
// in-process on a free port, with the background jobs, as main.ts does: a dev
// server with the fake idea reader and a tuner that passes at once.
//
// Ideas mode (M2-11, makscee/void-board#796), on by default against a dev
// server (GET /home says `dev`): after each run, a bot that holds an idea
// writes one from IDEAS, picks an archetype and a reading when they're ready,
// and votes on the cards it's shown; it starts the overnight check now and
// then. The summary gains `ideas: {written, picked, voted, failed}`. Never on
// the live game: bots don't submit ideas there (Maks, mission #735).
//   npm run mvp:bot -- [--runs 50] [--url http://127.0.0.1:8791/arena] [--no-ideas]
import { serve } from "@hono/node-server";
import type { AddressInfo } from "node:net";
import { MVP_RULES, type DecisionResponse, type HomeView, type LibraryView, type MvpContent, type MyIdeasView, type PlayerRef, type RunView, type VoteCard } from "../src/mvp/contract.js";
import { botDecision } from "../server/src/mvp/bots.js";
import { createMvpApp } from "../server/src/mvp/app.js";
import { poolContent, seedUnits } from "../server/src/mvp/pool.js";
import { MemoryMvpStore } from "../server/src/mvp/store.js";
import { fakeIdeaReader } from "../server/src/mvp/idea-reader.js";
import { ideaReadingJob, ideaReadingJobWith } from "../server/src/mvp/idea-reading.js";
import { MVP_JOBS, startMvpJobs } from "../server/src/mvp/jobs.js";
import { mvpRuntime } from "../server/src/mvp/runtime.js";
import { instantTuner } from "../server/src/mvp/votes.js";

const args = process.argv.slice(2);
const opt = (name: string) => {
  const i = args.indexOf(`--${name}`);
  return i >= 0 ? args[i + 1] : undefined;
};
const runs = Number(opt("runs") ?? 50);

let base = opt("url");
let close = () => {};
if (!base) {
  // A dev server: the fake reader every 100 ms, never a model; the tuner
  // passes at once, so the overnight check doesn't take minutes.
  // Units as data, as the server starts (main.ts): the seed pool live and the
  // units cut in round 4 in the Library, so bots can propose versions.
  const store = new MemoryMvpStore();
  seedUnits(store, new Date());
  const rt = mvpRuntime({ content: poolContent(store), store, dev: true, tuner: { night: instantTuner, dev: instantTuner } });
  const server = serve({ fetch: createMvpApp(rt).fetch, port: 0, hostname: "127.0.0.1" });
  const jobs = MVP_JOBS.map((job) => (job === ideaReadingJob ? ideaReadingJobWith(fakeIdeaReader(), 100) : job));
  const stopJobs = startMvpJobs(rt, jobs);
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
class ApiError extends Error {
  constructor(
    message: string,
    readonly status: number,
  ) {
    super(message);
  }
}
async function call<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(base + path, {
    method,
    headers: { "content-type": "application/json", ...(player ? { "X-Arena-Player": player } : {}) },
    ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
    signal: AbortSignal.timeout(10_000),
  });
  const json = await res.json();
  if (!res.ok) throw new ApiError(`${method} ${path} ${JSON.stringify(body ?? "")} → ${res.status} ${JSON.stringify(json)}`, res.status);
  return json as T;
}

const content = await call<MvpContent>("GET", "/content");
const tally = { decisions: {} as Record<string, number>, runs: 0, fights: 0, wins: 0, losses: 0, draws: 0, crowns: 0, ends: {} as Record<string, number>, ratingDelta: 0, errors: 0 };

// ---------- ideas mode ----------

/** What the bots write: a hedgehog, a medic, and an odd one. */
const IDEAS = [
  "A hedgehog that punishes whoever attacks it with its spines.",
  "A medic that always heals the weakest ally first.",
  "An odd little thief that steals gold from the shop.",
];
/** What they write for a new version of a Library unit (mission 3): every
 * third held idea goes to one, when the Library has a unit. */
const VERSIONS = [
  "Make it hit whoever attacks it instead of the front enemy.",
  "It should help the allies at the back more.",
];
const live = /makscee\.ru|twin-pogona/.test(base);
const wantIdeas = args.includes("--no-ideas") ? false : args.includes("--ideas") ? true : null;
const devServer = (await call<HomeView>("GET", "/home")).dev;
if (wantIdeas && (live || !devServer)) throw new Error("ideas mode is for a local dev server (MVP_DEV=1), never the live game");
const ideasOn = wantIdeas ?? (devServer && !live);
const ideas = { written: 0, proposed: 0, picked: 0, voted: 0, failed: 0 };
/** Picks the game refused because another idea just took the name or the
 * shape (409): the idea goes back a step and the bot picks again later. */
let retaken = 0;
const failedIds = new Set<string>();
let written = 0;

/** One bot's turn on the idea path: picks what's ready, writes what it
 * holds, votes on up to `votes` cards. Returns whether an idea moved. */
async function ideaTurn(votes: number): Promise<boolean> {
  let moved = false;
  let view = await call<MyIdeasView>("GET", "/ideas");
  for (const idea of view.sent) {
    if (idea.state === "failed" && !failedIds.has(idea.ideaId)) {
      failedIds.add(idea.ideaId);
      ideas.failed++;
    }
    const step = idea.state === "pick-archetype" ? "archetype" : idea.state === "pick-reading" ? "reading" : null;
    if (!step) continue;
    const n = (step === "archetype" ? idea.data.archetypes : idea.data.readings)?.length ?? 0;
    if (n === 0) throw new Error(`idea ${idea.ideaId} waits for its ${step} pick with no options`);
    try {
      view = await call<MyIdeasView>("POST", `/ideas/${idea.ideaId}/${step}`, { index: (written + ideas.picked) % n });
      ideas.picked++;
    } catch (e) {
      if (!(e instanceof ApiError && e.status === 409 && /just (named|took)/.test(e.message))) throw e;
      retaken++;
      view = await call<MyIdeasView>("GET", "/ideas");
    }
    moved = true;
  }
  while (view.ideas.held > 0) {
    // Every third idea proposes a new version of a Library unit (M3-4); a
    // unit that left the Library meanwhile (409) gets a new idea instead.
    const library = written % 3 === 2 ? (await call<LibraryView>("GET", "/library")).units : [];
    const target = library[(written / 3) % Math.max(1, library.length) | 0];
    if (target) {
      try {
        view = await call<MyIdeasView>("POST", "/ideas", { kind: "evolve", target: target.unit.id, text: VERSIONS[written++ % VERSIONS.length] });
        ideas.proposed++;
        moved = true;
        continue;
      } catch (e) {
        if (!(e instanceof ApiError && e.status === 409)) throw e;
      }
    }
    view = await call<MyIdeasView>("POST", "/ideas", { text: IDEAS[written++ % IDEAS.length] });
    ideas.written++;
    moved = true;
  }
  let card = (await call<{ card: VoteCard | null }>("GET", "/votes/next")).card;
  for (let v = 0; card && v < votes; v++) {
    // Mostly the candidate, now and then the live unit or a skip.
    const r = (ideas.voted * 7 + 3) % 10;
    const pick = r < 6 ? card.candidateId : r < 9 ? card.otherId : null;
    card = (await call<{ card: VoteCard | null }>("POST", "/votes", { candidateId: card.candidateId, otherId: card.otherId, pick })).card;
    ideas.voted++;
  }
  return moved;
}
const t0 = Date.now();
const tag = t0.toString(36).slice(-5);
const known = new Map<string, string>();
for (let i = 0; i < runs; i++) {
  try {
    // Five players take turns; each name is registered once, tagged per
    // invocation so a second pass at one server plays as new players.
    const name = `bot-${i % 5}-${tag}`;
    player = known.get(name) ?? (await call<PlayerRef>("POST", "/players", { name })).id;
    known.set(name, player);
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
    if (ideasOn) {
      await ideaTurn(3);
      // Now and then, the dev button: ideas picked to the end go to the vote.
      if (i % 10 === 9) await call("POST", "/dev/overnight-check");
    }
  } catch (e) {
    tally.errors++;
    console.error(`run ${i}: ${(e as Error).message}`);
  }
}
// The ideas still on their way: each bot picks (and votes a little) until no
// idea moves for 3 s (the reader and the check run in the background), within 60 s.
if (ideasOn) {
  let quiet = Date.now();
  for (const end = Date.now() + 60_000; Date.now() < end && Date.now() - quiet < 3_000; ) {
    try {
      let moved = false;
      for (const [, id] of known) {
        player = id;
        if (await ideaTurn(2)) moved = true;
      }
      await call("POST", "/dev/overnight-check");
      if (moved) quiet = Date.now();
    } catch (e) {
      tally.errors++;
      console.error(`ideas: ${(e as Error).message}`);
      break;
    }
    await new Promise((r) => setTimeout(r, 200));
  }
  // 50 runs earn each of the 5 bots 3 ideas: the path must go end to end.
  if (runs >= 50 && (!ideas.written || !ideas.proposed || !ideas.picked || !ideas.voted)) {
    tally.errors++;
    console.error(`the idea path stalled: ${JSON.stringify(ideas)}`);
  }
}
close();
// Slice 6 seeds a champion at start: a run that survives round 12 meets it.
if (tally.ends["no-champion"]) {
  tally.errors++;
  console.error(`${tally.ends["no-champion"]} runs found no champion at the Crown`);
}
console.log(JSON.stringify({ ...tally, ...(ideasOn ? { ideas, retaken } : {}), ms: Date.now() - t0 }));
process.exit(tally.errors === 0 && tally.runs === runs ? 0 : 1);
