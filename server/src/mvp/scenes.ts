// Test scenes (#1026): every screen and state Maks judges opens from one link,
// `#scene=<id>`, with no playing to reach it. A scene is a row here: the
// server builds its fixture on a fresh player (a run played by the bot's
// rules until the state shows up, a battle record, ideas, a day end), and the
// client opens the screen the row's entry names. The index page
// (scripts/scenes-index.ts) and the check (e2e/scenes.mjs) read the same table.
//
// Only a server with MVP_DEV=1, ARENA_SCENES=1 and MVP_DB=:memory: serves them
// (main.ts refuses anything else), so a fixture never touches a real world.
import { randomUUID } from "node:crypto";
import { benchSizeOf, type LineUnit, type PlayerRef, type RunView, type UnitId } from "../../../src/mvp/contract.js";
import type { MvpRunState } from "../../../src/mvp/run.js";
import { botDecision, seedChampion } from "./bots.js";
import { endDay } from "./day.js";
import { fakeIdeaReader } from "./idea-reader.js";
import { pickArchetype, readIdea } from "./idea-reading.js";
import { grantIdea, writeIdea } from "./ideas.js";
import { decide, startRun } from "./runs.js";
import type { MvpRuntime } from "./runtime.js";
import { seedCandidate } from "./votes.js";

/** What the client opens once the fixture is built (mobile/main.ts openScene). */
export type SceneEntry =
  | { screen: "name" }
  | { screen: "home"; ended?: number; click?: string; scroll?: string }
  | { screen: "shop"; runId: string }
  | { screen: "battle"; battleId: string; runId?: string; keys?: string[] }
  | { screen: "run-over"; runId: string }
  | { screen: "codex"; tab: "units" | "fusions" | "library" | "keywords" | "champions" | "howto" }
  | { screen: "ideas"; click?: string }
  | { screen: "pick"; ideaId: string };

export interface SceneBuilt {
  /** The fixture's player, or null for the sign-in screen. */
  player: PlayerRef | null;
  entry: SceneEntry;
}

export interface Scene {
  id: string;
  group: string;
  title: string;
  /** What to look at, one line for the index page. */
  look: string;
  /** Desktop gets its own link where its layout differs from the phone's. */
  desktop: boolean;
  build: (rt: MvpRuntime) => SceneBuilt | Promise<SceneBuilt>;
}

/** No such fixture after the tries (a rare state the bot's runs didn't reach). */
export class SceneMissing extends Error {}

let n = 0;
function fixturePlayer(rt: MvpRuntime): PlayerRef {
  const p: PlayerRef = { id: randomUUID(), name: `Tester ${++n}`, bot: false };
  rt.store.addPlayer(p);
  return p;
}

/** The day's champion, which the Crown fights (the bot world seeds it too, a little later). */
async function world(rt: MvpRuntime): Promise<void> {
  await seedChampion(rt);
}

const board = (run: RunView): LineUnit[] => [...run.line, ...(run.bench ?? [])];

/** Plays fresh runs by the bot's rules, stopping at the first state `stop`
 * accepts (checked before each decision, and on the ended run). */
async function playTo(rt: MvpRuntime, stop: (run: MvpRunState) => boolean, what: string, tries = 40): Promise<MvpRunState> {
  await world(rt);
  for (let i = 0; i < tries; i++) {
    let run = startRun(rt, fixturePlayer(rt));
    let round = run.round;
    let rerolled = 0;
    for (let guard = 0; guard < 1000; guard++) {
      if (stop(run)) return run;
      if (run.phase === "over") break;
      if (run.round !== round) [round, rerolled] = [run.round, 0];
      const d = botDecision(run, rt.contentFor(run.contentVersion), rt.rules, rerolled);
      if (d.kind === "reroll") rerolled++;
      decide(rt, run, d);
      run = rt.store.run(run.runId)!;
    }
  }
  throw new SceneMissing(`no run reached ${what} in ${tries} tries`);
}

const shopAt = (round: number) => (run: MvpRunState) => run.phase === "shop" && run.round === round && !run.gift;
const fightOf = (kind: "round" | "crown", outcome: "win" | "loss" | "draw") => (run: MvpRunState) => run.fights.find((f) => f.kind === kind && f.outcome === outcome);
const shop = (run: MvpRunState): SceneBuilt => ({ player: run.player, entry: { screen: "shop", runId: run.runId } });
const battleOf = (kind: "round" | "crown", outcome: "win" | "loss" | "draw", keys?: string[]) => async (rt: MvpRuntime): Promise<SceneBuilt> => {
  const run = await playTo(rt, (r) => r.phase === "over" && !!fightOf(kind, outcome)(r), `a ${kind} ${outcome}`, 80);
  return { player: run.player, entry: { screen: "battle", battleId: fightOf(kind, outcome)(run)!.battleId, runId: run.runId, ...(keys ? { keys } : {}) } };
};
const ended = (reasons: string[]) => async (rt: MvpRuntime): Promise<SceneBuilt> => {
  const run = await playTo(rt, (r) => r.phase === "over" && reasons.includes(r.endedBy ?? ""), `a run ended by ${reasons.join("/")}`, 80);
  return { player: run.player, entry: { screen: "run-over", runId: run.runId } };
};
/** An affordable offer that is a copy of a board unit with `copies` copies. */
const copyOffer = (copies: number) => (run: MvpRunState) =>
  run.phase === "shop" && !run.gift && run.offers.some((o) => o.cost <= run.gold && board(run).some((u) => u.kind === "unit" && u.unitId === o.unitId && u.copies === copies));
const home = (extra: Omit<Extract<SceneEntry, { screen: "home" }>, "screen"> = {}) => async (rt: MvpRuntime): Promise<SceneBuilt> => {
  await world(rt);
  return { player: fixturePlayer(rt), entry: { screen: "home", ...extra } };
};
const codex = (tab: Extract<SceneEntry, { screen: "codex" }>["tab"]) => async (rt: MvpRuntime): Promise<SceneBuilt> => {
  await world(rt);
  return { player: fixturePlayer(rt), entry: { screen: "codex", tab } };
};
/** A player holding an idea written as `text`, read by the fake reader up to its archetype pick. */
async function ideaAtArchetypes(rt: MvpRuntime, text = "A lantern keeper who heals the ally behind it whenever an enemy dies"): Promise<{ player: PlayerRef; ideaId: string }> {
  await world(rt);
  const player = fixturePlayer(rt);
  grantIdea(rt, player.id);
  const idea = writeIdea(rt, player.id, text);
  await readIdea(rt, fakeIdeaReader(), idea);
  return { player, ideaId: idea.ideaId };
}

let dayEnded: number | null = null;

export const SCENES: Scene[] = [
  { id: "signin", group: "Start", title: "Create a player", look: "The first screen: name field, sign-in options", desktop: true, build: () => ({ player: null, entry: { screen: "name" } }) },
  { id: "home", group: "Start", title: "Home / Throne", look: "Champion hero, Play, ideas meter, vote card, nav", desktop: true, build: home() },
  { id: "settings", group: "Start", title: "Settings sheet", look: "Sound, music, language, palette rows, How to play", desktop: true, build: home({ click: "settings" }) },

  { id: "shop-r1", group: "Shop", title: "Shop, round 1 (empty board)", look: "Empty line and bench, the first offers, gold", desktop: true, build: async (rt) => shop(await playTo(rt, shopAt(1), "round 1")) },
  { id: "shop-r3", group: "Shop", title: "Shop, round 3", look: "A few units on the line, offers, hearts, run track", desktop: true, build: async (rt) => shop(await playTo(rt, shopAt(3), "round 3")) },
  { id: "shop-r8", group: "Shop", title: "Shop, round 8", look: "A late line, tier offers, the run track near the Crown", desktop: true, build: async (rt) => shop(await playTo(rt, shopAt(8), "round 8")) },
  { id: "shop-bench-full", group: "Shop", title: "Shop, full bench", look: "Line of 5 plus a full bench: does it all fit?", desktop: true, build: async (rt) => shop(await playTo(rt, (r) => r.phase === "shop" && !r.gift && r.line.length >= rt.rules.lineSize && (r.bench ?? []).length >= benchSizeOf(rt.rules), "a full bench", 80)) },
  { id: "shop-merge-hint", group: "Shop", title: "Shop, a merge hint", look: "An offer that is a copy of a unit you own: its hint", desktop: true, build: async (rt) => shop(await playTo(rt, copyOffer(1), "a copy on offer")) },

  { id: "awaken", group: "Awaken & fuse", title: "Awaken: the 3rd copy on offer", look: "Buying it awakens the unit: the hint on the offer", desktop: true, build: async (rt) => shop(await playTo(rt, copyOffer(2), "a 3rd copy on offer", 80)) },
  { id: "gift", group: "Awaken & fuse", title: "The awakening gift", look: "The gift's choices after an awakening", desktop: true, build: async (rt) => shop(await playTo(rt, (r) => r.phase === "shop" && !!r.gift, "a gift", 80)) },
  {
    id: "fuse", group: "Awaken & fuse", title: "Fuse two Awoken", look: "Two Awoken units on the board, ready to fuse", desktop: true,
    build: async (rt) => shop(await playTo(rt, (r) => r.phase === "shop" && !r.gift && botDecision(r, rt.contentFor(r.contentVersion), rt.rules, 9).kind === "fuse", "two Awoken to fuse", 80)),
  },

  { id: "battle-win", group: "Battle", title: "A battle you win", look: "Pacing, beams, the ledger, the end card", desktop: true, build: battleOf("round", "win") },
  { id: "battle-loss", group: "Battle", title: "A battle you lose", look: "The loss end card, hearts lost", desktop: true, build: battleOf("round", "loss") },
  { id: "battle-draw", group: "Battle", title: "A draw", look: "The draw end card", desktop: true, build: battleOf("round", "draw") },
  { id: "battle-ledger", group: "Battle", title: "The ledger, paused mid-battle", look: "Stepped a few beats and paused: the turn ledger readable?", desktop: true, build: battleOf("round", "win", ["Space", "ArrowRight", "ArrowRight", "ArrowRight", "ArrowRight", "ArrowRight", "ArrowRight"]) },

  { id: "crown", group: "Crown & day", title: "The Crown fight is next", look: "After round 12: the champion's team, the Crown button", desktop: true, build: async (rt) => shop(await playTo(rt, (r) => r.phase === "crown", "the Crown", 80)) },
  { id: "crown-fight", group: "Crown & day", title: "The Crown fight, lost", look: "Your team against the champion", desktop: true, build: battleOf("crown", "loss") },
  { id: "slay", group: "Crown & day", title: "A slay (the Crown won)", look: "Beating the champion: the battle and its end", desktop: true, build: battleOf("crown", "win") },
  {
    id: "day-end", group: "Crown & day", title: "Day end with a playoff", look: "'The day ended' notice, the playoff panel, the new champion", desktop: true,
    build: async (rt) => {
      await world(rt);
      if (dayEnded === null) {
        // Slayers first, so the day's end has a playoff; then a champion again for the other scenes.
        const slayers = new Set<string>();
        for (let i = 0; i < 4 && slayers.size < 2; i++) {
          const run = await playTo(rt, (r) => r.phase === "over" && r.endedBy === "crown-won", "a slay", 80).catch(() => null);
          if (run) slayers.add(run.player.id);
        }
        dayEnded = endDay(rt).seq - 1;
        await world(rt);
      }
      return { player: fixturePlayer(rt), entry: { screen: "home", ended: dayEnded } };
    },
  },

  { id: "run-won", group: "Run end", title: "Run over: the Crown won", look: "The run's end: record, rating change, New run", desktop: true, build: ended(["crown-won"]) },
  { id: "run-crown-lost", group: "Run end", title: "Run over: the Crown lost", look: "Reached the Crown, lost it", desktop: false, build: ended(["crown-lost"]) },
  { id: "run-out", group: "Run end", title: "Run over: out of hearts", look: "The run's end with no hearts left", desktop: true, build: ended(["out-of-hearts"]) },

  {
    id: "ideas", group: "Ideas", title: "My ideas", look: "Ideas held, refill line, sent ideas at different stages", desktop: true,
    build: async (rt) => {
      const { player } = await ideaAtArchetypes(rt);
      grantIdea(rt, player.id);
      return { player, entry: { screen: "ideas" } };
    },
  },
  {
    id: "idea-write", group: "Ideas", title: "Write an idea", look: "The idea text field and its hints", desktop: true,
    build: async (rt) => {
      await world(rt);
      const player = fixturePlayer(rt);
      grantIdea(rt, player.id);
      return { player, entry: { screen: "ideas", click: "idea-new" } };
    },
  },
  { id: "idea-archetype", group: "Ideas", title: "Pick an archetype", look: "The idea's three archetype options", desktop: true, build: async (rt) => {
    const { player, ideaId } = await ideaAtArchetypes(rt);
    return { player, entry: { screen: "pick", ideaId } };
  } },
  {
    id: "idea-implementation", group: "Ideas", title: "Pick an implementation", look: "The archetype's readings as cards", desktop: true,
    build: async (rt) => {
      const { player, ideaId } = await ideaAtArchetypes(rt);
      pickArchetype(rt, player.id, ideaId, 0);
      await readIdea(rt, fakeIdeaReader(), rt.store.idea(ideaId)!);
      return { player, entry: { screen: "pick", ideaId } };
    },
  },
  {
    id: "vote", group: "Ideas", title: "Voting", look: "The either/or vote card under the menu", desktop: true,
    build: async (rt) => {
      await world(rt);
      const author = fixturePlayer(rt);
      const units = rt.content.units.filter((u) => u.tier === 1).map((u) => u.id as UnitId);
      for (const unit of units.slice(0, 2)) seedCandidate(rt, author.id, unit);
      return { player: fixturePlayer(rt), entry: { screen: "home", scroll: "vote-box" } };
    },
  },

  { id: "codex-units", group: "Codex", title: "Codex: units", look: "The unit grid, tiers, filters", desktop: true, build: codex("units") },
  { id: "codex-fusions", group: "Codex", title: "Codex: fusions", look: "Discovered fusions and their names", desktop: true, build: codex("fusions") },
  { id: "codex-library", group: "Codex", title: "Codex: library", look: "Units out of the pool, their versions", desktop: true, build: codex("library") },
  { id: "codex-keywords", group: "Codex", title: "Codex: keywords", look: "Terms and the units that use them", desktop: true, build: codex("keywords") },
  { id: "codex-champions", group: "Codex", title: "Codex: champions / stats", look: "Records and past champions", desktop: true, build: codex("champions") },
  { id: "codex-howto", group: "Codex", title: "Codex: how to play", look: "The rules page", desktop: true, build: codex("howto") },
];

export const sceneById = (id: string): Scene | undefined => SCENES.find((s) => s.id === id);

/** The table without builders, for GET /dev/scenes, the index and the check. */
export const sceneList = () => SCENES.map(({ id, group, title, look, desktop }) => ({ id, group, title, look, desktop }));

/** Refuses a scenes server that could touch a real world (main.ts calls it at start). */
export function checkScenesEnv(env: NodeJS.ProcessEnv): boolean {
  if (env.ARENA_SCENES !== "1") return false;
  if (env.MVP_DEV !== "1" || env.MVP_DB !== ":memory:") throw new Error("ARENA_SCENES=1 needs MVP_DEV=1 and MVP_DB=:memory: (scenes never run against a real world)");
  if (env.MVP_INVITES === "1") throw new Error("ARENA_SCENES=1 runs on an open server, not with MVP_INVITES=1");
  return true;
}
