// Own-team Crown setup for the phone e2e (mission #574, #587): through the
// HTTP API of a local dev server (MVP_DEV=1), one player plays as slice 6's
// bots do until a run slays the champion, the dev "end day now" crowns them,
// then they play on: a run that beats their own champion team must count as a
// slay (#591), and another run waits at the Crown against their own team.
// Prints that player and run as JSON on the last line.
//   node --import tsx/esm e2e/mvp-own-crown.ts --url http://127.0.0.1:8791/arena
import { MVP_RULES, type DayView, type DecisionResponse, type HomeView, type MvpContent, type PlayerRef, type RunView } from "../src/mvp/contract.js";
import { botDecision } from "../server/src/mvp/bots.js";

const args = process.argv.slice(2);
const i = args.indexOf("--url");
if (i < 0) throw new Error("--url <local server> is required");
const base = args[i + 1]!.replace(/\/$/, "") + "/api/v1";

let player = "";
async function call<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(base + path, {
    method,
    headers: { "content-type": "application/json", ...(player ? { "X-Arena-Player": player } : {}) },
    ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
    signal: AbortSignal.timeout(10_000),
  });
  const json = await res.json();
  if (!res.ok) throw new Error(`${method} ${path} → ${res.status} ${JSON.stringify(json)}`);
  return json as T;
}

/** Plays `run` with botDecision until it is over, or (with `stopAtCrown`) at the Crown. */
async function play(run: RunView, content: MvpContent, stopAtCrown: boolean): Promise<RunView> {
  let round = run.round;
  let rerolled = 0;
  for (let guard = 0; run.phase !== "over" && guard < 1000; guard++) {
    if (stopAtCrown && run.phase === "crown") return run;
    if (run.round !== round) [round, rerolled] = [run.round, 0];
    const d = botDecision(run, content, MVP_RULES, rerolled);
    if (d.kind === "reroll") rerolled++;
    run = (await call<DecisionResponse>("POST", `/runs/${run.runId}/decisions`, d)).run;
  }
  return run;
}

const content = await call<MvpContent>("GET", "/content");
const me = await call<PlayerRef>("POST", "/players", { name: "Reigning" });
player = me.id;

// Bots slay too, so a bot may win the playoff: slay and end the day again
// until this player is crowned.
let crowned = false;
let last = "";
for (let days = 0; days < 12 && !crowned; days++) {
  let slew = false;
  for (let n = 0; n < 400 && !slew; n++) slew = (await play(await call<RunView>("POST", "/runs"), content, false)).endedBy === "crown-won";
  if (!slew) throw new Error("no slay in 400 runs");
  await call("POST", "/dev/end-day");
  const day = await call<DayView>("GET", "/day");
  crowned = day.champion?.player.id === me.id;
  last = day.champion?.player.name ?? "nobody";
}
if (!crowned) throw new Error(`after 12 day ends the champion is @${last}, not @${me.name}`);

// Beating your own champion team is a slay (#591): a Slay row, so /day counts
// this player and their records gain one.
const slaysOf = async () => (await call<HomeView>("GET", "/home")).rating?.slays ?? 0;
let ownSlay = false;
for (let n = 0; n < 400 && !ownSlay; n++) {
  const before = await slaysOf();
  const run = await play(await call<RunView>("POST", "/runs"), content, false);
  const crown = run.fights.find((f) => f.kind === "crown");
  if (!crown) continue;
  if (crown.opponent.player.id !== me.id) throw new Error(`the Crown was @${crown.opponent.player.name}, not the champion's own team`);
  if (run.endedBy !== "crown-won") continue;
  const after = await slaysOf();
  if (after !== before + 1) throw new Error(`beating the own champion team: slays ${before} → ${after}, not a slay`);
  if ((await call<DayView>("GET", "/day")).slayers < 1) throw new Error("beating the own champion team: /day counts no slayer");
  ownSlay = true;
}
if (!ownSlay) throw new Error("no win against the own champion team in 400 runs");

for (let n = 0; n < 100; n++) {
  const run = await play(await call<RunView>("POST", "/runs"), content, true);
  if (run.phase !== "crown") continue;
  if (run.nextOpponent?.player.id !== me.id) throw new Error(`the Crown is @${run.nextOpponent?.player.name}, not the champion's own team`);
  console.log(JSON.stringify({ player: me, runId: run.runId }));
  process.exit(0);
}
throw new Error("no run reached the Crown in 100 tries");
