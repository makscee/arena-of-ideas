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
import { dayView, endDay } from "./day.js";
import { fakeIdeaReader } from "./idea-reader.js";
import { pickArchetype, readIdea } from "./idea-reading.js";
import { grantIdea, writeIdea } from "./ideas.js";
import { decide, startRun } from "./runs.js";
import type { MvpRuntime } from "./runtime.js";
import { fixtureBattle } from "./scene-fixtures.js";
import { contentOf } from "./content.js";
import { poolBattle, poolBattleId } from "../../../src/mvp/pool-battles.js";
import { ROWS_V2 } from "../../../src/mvp/units-v2.js";
import { codexPair, pairState, sortPair, type FusionDesign, type PairState } from "../../../src/mvp/codex.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { codexForged, forgeOf } from "./codex.js";
import { claimFixture, forgeFixture, type ClaimView, type ForgeView } from "../../../src/mvp/forge-view.js";
import { instantTuner, nextCard, overnightCheck, seedCandidate } from "./votes.js";

/** What the client opens once the fixture is built (mobile/main.ts openScene). */
export type SceneEntry =
  | { screen: "name" }
  | { screen: "holding" }
  | { screen: "home"; ended?: number; click?: string; scroll?: string }
  /** `forge`: open that pair's forge (#1049); `click`: then press that button. */
  | { screen: "shop"; runId: string; forge?: [number, number]; click?: string }
  | { screen: "battle"; battleId: string; runId?: string; keys?: string[] }
  | { screen: "run-over"; runId: string }
  | { screen: "codex"; tab: "units" | "fusions" | "library" | "keywords" | "champions" | "howto" }
  | { screen: "ideas"; click?: string }
  | { screen: "pick"; ideaId: string }
  | { screen: "forge"; tab: "units" | "feed" | "words"; unit?: string; view: ForgeView }
  | { screen: "claim"; claim: ClaimView; view: ForgeView; picked?: number };

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
  /** A view only (a battle, a run's end) of a state that takes many runs to
   * reach: built once, at the server's start (warmScenes), and kept. */
  keep?: boolean;
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
      // With the Codex on, an unforged pair can't be fused: the bots skip it.
      const d = botDecision(run, rt.contentFor(run.contentVersion), rt.rules, rerolled, rt.codex ? (x, y) => codexForged(rt, x, y) : undefined);
      if (d.kind === "reroll") rerolled++;
      decide(rt, run, d);
      run = rt.store.run(run.runId)!;
    }
    // A rare state takes many runs: let requests through between them.
    await new Promise((r) => setImmediate(r));
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
/** A round fight where a unit that had already acted in a chain was stopped from acting again (#1030). */
const spentFight = (rt: MvpRuntime) => (run: MvpRunState) =>
  run.fights.find((f) => f.kind === "round" && !!rt.store.battle(f.battleId)?.log.some((e) => e.type === "ChainBlocked" && e.spent));
const chainOnce = async (rt: MvpRuntime): Promise<SceneBuilt> => {
  const run = await playTo(rt, (r) => r.phase === "over" && !!spentFight(rt)(r), "a fight where a unit is stopped from acting twice in a chain", 80);
  return { player: run.player, entry: { screen: "battle", battleId: spentFight(rt)(run)!.battleId, runId: run.runId } };
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
/** Codex v2 (#1051): the screens over the design page's eight units, from
 * forgeFixture until the Codex data (#1049) serves the world's own. */
const codexV2 = (tab: "units" | "feed" | "words", unit?: string) => (rt: MvpRuntime): SceneBuilt => {
  const player = fixturePlayer(rt);
  return { player, entry: { screen: "forge", tab, ...(unit ? { unit } : {}), view: forgeFixture(player.name) } };
};
const claim = (picked?: number) => (rt: MvpRuntime): SceneBuilt => {
  const player = fixturePlayer(rt);
  return { player, entry: { screen: "claim", claim: claimFixture(), view: forgeFixture(player.name), ...(picked === undefined ? {} : { picked }) } };
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

/** Three designs for a forge scene's pair: the rules of other pool units'
 * Awoken forms, named and flavoured, standing in for a night batch (#1050). */
export function sceneDesigns(rt: MvpRuntime, a: UnitId, b: UnitId): FusionDesign[] {
  const others = rt.content.units.filter((u) => u.id !== a && u.id !== b && u.forms?.awoken);
  // without the unit's own `say`: it names that unit (#1120: "Hollow Echo" read as Prism)
  const pickOf = (i: number) => {
    const { say: _, ...form } = structuredClone(others[(i * 7) % others.length]!.forms.awoken);
    return form;
  };
  return [
    { name: "Ember Warden", emoji: "🔥", lean: "a", recipe: pickOf(1), flavour: "Leans toward the first: it keeps the fire.", texts: { ru: { name: "Страж углей", flavour: "Ближе к первому: хранит огонь." } } },
    { name: "Tide Caller", emoji: "🌊", lean: "b", recipe: pickOf(2), flavour: "Leans toward the second: it calls the water.", texts: { ru: { name: "Зовущий прилив", flavour: "Ближе ко второму: зовёт воду." } } },
    { name: "Hollow Echo", emoji: "🌀", lean: "twist", recipe: pickOf(3), flavour: "A twist: neither parent, a third creature.", texts: { ru: { name: "Пустое эхо", flavour: "Поворот: ни один из родителей, третье существо." } } },
  ];
}

/** A shop with two Awoken to fuse and the forge open on their pair, in
 * Codex `state` (#1049); `click` then presses a button (the claim). Needs a
 * scenes server with the Codex on (ARENA_CODEX=1). */
const forge = (state: PairState, click?: string) => async (rt: MvpRuntime): Promise<SceneBuilt> => {
  if (!rt.codex) throw new SceneMissing("the forge scenes need the Codex: start the scenes server with ARENA_CODEX=1");
  const run = await playTo(rt, (r) => r.phase === "shop" && !r.gift && forgeOf(rt, r).some((p) => p.state === "unforged"), "two Awoken to fuse", 80);
  const slot = forgeOf(rt, run).find((p) => p.state === "unforged")!;
  if (state !== "unforged") rt.store.putCodexPair({ ...codexPair(slot.a, slot.b), candidates: sceneDesigns(rt, slot.a, slot.b), preparedAt: rt.now().toISOString() });
  if (state === "discovered") rt.store.claimCodexPair(slot.a, slot.b, { pick: 2, by: fixturePlayer(rt), at: rt.now().toISOString() });
  return { player: run.player, entry: { screen: "shop", runId: run.runId, forge: [slot.first, slot.second], ...(click ? { click } : {}) } };
};

/** The shop's fusion marks (#1049 reopened, cases 1–4), on a board set by
 * hand so each shows on its own: `none`, two Awoken whose pair has no design
 * (no mark, no Fuse); `now`, two Awoken of a discovered pair (⚗ on both);
 * `shop`, an offer whose third copy awakens into a ready pair with your
 * Awoken unit (its pill), one that would still need awakening into a
 * discovered pair (a quiet ⚗) and one into an unforged pair (nothing). */
const fusionMarks = (kind: "none" | "now" | "shop") => async (rt: MvpRuntime): Promise<SceneBuilt> => {
  if (!rt.codex) throw new SceneMissing("the fusion-mark scenes need the Codex: start the scenes server with ARENA_CODEX=1");
  const run = await playTo(rt, shopAt(3), "round 3");
  // Units whose pairs nothing stored yet, so the scene sets each pair's state
  // itself; each scene from its own part of the pool (builds are kept, so
  // fusion-now must never design fusion-none's pair).
  const all = rt.content.units.filter((u) => u.forms?.awoken);
  const part = Math.floor(all.length / 3);
  const from = { none: 0, now: 1, shop: 2 }[kind] * part;
  const units = all.slice(from, from + part);
  const free = (x: UnitId, y: UnitId) => pairState(rt.store.codexPair(x, y)) === "unforged";
  const pick: typeof units = [];
  for (const u of units) if (pick.length < 5 && pick.every((v) => free(u.id, v.id))) pick.push(u);
  if (pick.length < 5) throw new SceneMissing("five units with no stored pairs");
  const [a, b, c, d, e] = pick as [typeof units[0], typeof units[0], typeof units[0], typeof units[0], typeof units[0]];
  const at = rt.now().toISOString();
  const store = (x: UnitId, y: UnitId, state: PairState) => {
    if (state === "unforged") return;
    rt.store.putCodexPair({ ...codexPair(x, y), candidates: sceneDesigns(rt, x, y), preparedAt: at });
    if (state === "discovered") rt.store.claimCodexPair(...sortPair(x, y), { pick: 2, by: fixturePlayer(rt), at });
  };
  const n = rt.rules.copiesToAwaken;
  const offer = (slot: number, unitId: UnitId) => ({ slot, unitId, tier: 1 as const, cost: rt.rules.unitCost });
  let s: MvpRunState;
  if (kind === "shop") {
    store(a.id, c.id, "ready");
    store(a.id, d.id, "discovered");
    s = { ...run, gold: 10, bench: [], line: [lineUnitOf(a, `${run.runId}-a`, n, rt.rules), lineUnitOf(c, `${run.runId}-c`, n - 1, rt.rules), lineUnitOf(b, `${run.runId}-b`, 1, rt.rules)], offers: [offer(0, c.id), offer(1, d.id), offer(2, e.id)] };
  } else {
    if (kind === "now") store(a.id, b.id, "discovered");
    s = { ...run, gold: 10, bench: [], line: [lineUnitOf(a, `${run.runId}-a`, n, rt.rules), lineUnitOf(b, `${run.runId}-b`, n, rt.rules), lineUnitOf(c, `${run.runId}-c`, 1, rt.rules)], offers: [offer(0, d.id), offer(1, e.id), offer(2, c.id)] };
  }
  rt.store.putRun(s);
  return shop(s);
};

let dayEnded: number | null = null;

/** A rules v2 fixture battle (#1045): the golden battle of src/words/fixtures.ts, stored and opened as a replay. */
const v2 = (name: string) => (rt: MvpRuntime): SceneBuilt => {
  const player = fixturePlayer(rt);
  const rec = fixtureBattle(name, player, rt.content.version, rt.now().toISOString());
  rt.store.putBattle(rec);
  return { player, entry: { screen: "battle", battleId: rec.battleId } };
};

/** `v2-unit-<slug>[-awoken]`: a battle of the v2 pool where the unit acts (#1046). */
const v2unit = (name: string, awoken: boolean) => (rt: MvpRuntime): SceneBuilt => {
  const player = fixturePlayer(rt);
  const { record } = poolBattle(contentOf(ROWS_V2), name, awoken, player, rt.now().toISOString(), rt.content.version);
  rt.store.putBattle(record);
  return { player, entry: { screen: "battle", battleId: record.battleId } };
};
/** The scene group of a pool unit by its batch (20 each, in ROWS_V2 order). */
const V2_BATCH_GROUP = ["Rules v2 (unit batch 1, #1046)", "Rules v2 (unit batch 2, #1047)", "Rules v2 (unit batch 3, #1048)"];
const v2unitScenes = (): Scene[] =>
  ROWS_V2.slice(8).flatMap((row, i) => [
    { id: poolBattleId(row.name, false), keep: true, group: V2_BATCH_GROUP[Math.floor((i + 8) / 20)]!, title: `${row.name} (sleeping) in a pool battle`, look: `${row.shape}, tier ${row.tier}: ${row.archetype}`, desktop: true, build: v2unit(row.name, false) },
    { id: poolBattleId(row.name, true), keep: true, group: V2_BATCH_GROUP[Math.floor((i + 8) / 20)]!, title: `${row.name} (awoken) in a pool battle`, look: `the awoken upgrade of one clause, see the report page`, desktop: true, build: v2unit(row.name, true) },
  ]);

export const SCENES: Scene[] = [
  { id: "signin", group: "Start", title: "Create a player", look: "The first screen: name field, sign-in options", desktop: true, build: () => ({ player: null, entry: { screen: "name" } }) },
  { id: "holding", group: "Start", title: "Holding page (Arena dark)", look: "What the public URL shows while Arena is rebuilt: title, what's coming, language switch", desktop: true, build: () => ({ player: null, entry: { screen: "holding" } }) },
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
    id: "fuse", group: "Awaken & fuse", title: "Two Awoken from play", look: "Two Awoken units reached in play; with the Codex on their pair has no design yet, so no Fuse (fusion-now shows one that fuses)", desktop: true,
    build: async (rt) => shop(await playTo(rt, (r) => r.phase === "shop" && !r.gift && botDecision(r, rt.contentFor(r.contentVersion), rt.rules, 9).kind === "fuse", "two Awoken to fuse", 80)),
  },

  { id: "forge-unforged", group: "Awaken & fuse", title: "The forge: not forged yet (Wish)", look: "Rules v2 (#1049): a pair with no designs yet: 'not forged yet', today's wishes, Wish", desktop: true, build: forge("unforged") },
  { id: "forge-ready", group: "Awaken & fuse", title: "The forge: ready to claim", look: "Rules v2 (#1049, #1120): three sealed designs, a row each: card, rule, flavour; pick one (1–3), Claim", desktop: true, build: forge("ready") },
  { id: "forge-claim", group: "Awaken & fuse", title: "The forge: a claim", look: "Rules v2 (#1049): Claim pressed on the first design: 'You claimed …', the fused unit", desktop: true, build: forge("ready", "forge-claim") },
  { id: "forge-discovered", group: "Awaken & fuse", title: "The forge: discovered", look: "Rules v2 (#1049): the canonical design, 'Discovered by @… · date', Fuse", desktop: true, build: forge("discovered") },
  { id: "fusion-none", group: "Awaken & fuse", title: "Two Awoken, no design: no fuse", look: "Rules v2 (#1049): their pair isn't forged: no ⚗, no glow, no Fuse button, no fuse hint", desktop: true, build: fusionMarks("none") },
  { id: "fusion-now", group: "Awaken & fuse", title: "A fusion you can make now", look: "Rules v2 (#1049): two Awoken of a discovered pair: ⚗ and a glow on both; Fuse opens the forge", desktop: true, build: fusionMarks("now") },
  { id: "fusion-shop", group: "Awaken & fuse", title: "A fusion one buy away", look: "Rules v2 (#1049): the first offer's 3rd copy awakens into a pair ready to claim (its pill); the second needs awakening (a quiet ⚗); the third pairs with nothing designed (no mark)", desktop: true, build: fusionMarks("shop") },
  { id: "battle-win", keep: true, group: "Battle", title: "A battle you win", look: "Pacing, the current beam, step numbers and spent cards, the end card", desktop: true, build: battleOf("round", "win") },
  { id: "battle-loss", keep: true, group: "Battle", title: "A battle you lose", look: "The loss end card, hearts lost", desktop: true, build: battleOf("round", "loss") },
  { id: "battle-draw", keep: true, group: "Battle", title: "A draw", look: "The draw end card", desktop: true, build: battleOf("round", "draw") },
  { id: "battle-chain-once", keep: true, group: "Battle", title: "A unit acts once per chain", look: "A chain where a unit would have acted twice before #1030: step numbers, spent cards fade, big numbers", desktop: true, build: chainOnce },
  { id: "battle-ledger", keep: true, group: "Battle", title: "Paused mid-battle", look: "Stepped a few beats and paused: the turn bar, the board", desktop: true, build: battleOf("round", "win", ["Space", "ArrowRight", "ArrowRight", "ArrowRight", "ArrowRight", "ArrowRight", "ArrowRight"]) },

  { id: "crown", group: "Crown & day", title: "The Crown fight is next", look: "After round 12: the champion's team, the Crown button", desktop: true, build: async (rt) => shop(await playTo(rt, (r) => r.phase === "crown", "the Crown", 80)) },
  { id: "crown-fight", keep: true, group: "Crown & day", title: "The Crown fight, lost", look: "Your team against the champion", desktop: true, build: battleOf("crown", "loss") },
  { id: "slay", keep: true, group: "Crown & day", title: "A slay (the Crown won)", look: "Beating the champion: the battle and its end", desktop: true, build: battleOf("crown", "win") },
  {
    id: "day-end", keep: true, group: "Crown & day", title: "Day end with a playoff", look: "'The day ended' notice, the playoff panel, the new champion", desktop: true,
    build: async (rt) => {
      await world(rt);
      if (dayEnded === null) {
        // Slayers first (the bot world makes a few each day), so the day's
        // end has a playoff; then a champion again for the other scenes.
        for (let i = 0; i < 3 && dayView(rt).slayers < 2; i++) await playTo(rt, (r) => r.phase === "over" && r.endedBy === "crown-won", "a slay", 80).catch(() => null);
        dayEnded = endDay(rt).seq - 1;
        await world(rt);
      }
      return { player: fixturePlayer(rt), entry: { screen: "home", ended: dayEnded } };
    },
  },

  { id: "run-won", keep: true, group: "Run end", title: "Run over: the Crown won", look: "The run's end: record, rating change, New run", desktop: true, build: ended(["crown-won"]) },
  { id: "run-crown-lost", keep: true, group: "Run end", title: "Run over: the Crown lost", look: "Reached the Crown, lost it", desktop: false, build: ended(["crown-lost"]) },
  { id: "run-out", keep: true, group: "Run end", title: "Run over: out of hearts", look: "The run's end with no hearts left", desktop: true, build: ended(["out-of-hearts"]) },

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
  // #1122: the archetype icon pictures the idea's subject, the same on all three
  { id: "idea-archetype-ru", group: "Ideas", title: "Pick an archetype (a Russian idea)", look: "#1122: 'Старый волк…' gives 🐺 on all three archetypes", desktop: true, build: async (rt) => {
    const { player, ideaId } = await ideaAtArchetypes(rt, "Старый волк, который бьёт сильнее, когда остаётся один");
    return { player, entry: { screen: "pick", ideaId } };
  } },
  { id: "idea-archetype-plain", group: "Ideas", title: "Pick an archetype (nothing to picture)", look: "#1122: an idea with no subject to picture gives one neutral ✨ on all three, never random animals", desktop: true, build: async (rt) => {
    const { player, ideaId } = await ideaAtArchetypes(rt, "Whenever an ally is healed, it gets a little stronger");
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
      // Past the overnight check, so they are up for votes.
      await overnightCheck(rt, instantTuner);
      const voter = fixturePlayer(rt);
      if (!nextCard(rt, voter)) throw new SceneMissing("no vote card after seeding two candidates");
      return { player: voter, entry: { screen: "home", scroll: "vote-box" } };
    },
  },

  { id: "codex-units", group: "Codex", title: "Codex: units", look: "The unit grid, tiers, filters", desktop: true, build: codex("units") },
  { id: "codex-fusions", group: "Codex", title: "Codex: fusions", look: "Discovered fusions and their names", desktop: true, build: codex("fusions") },
  { id: "codex-library", group: "Codex", title: "Codex: library", look: "Units out of the pool, their versions", desktop: true, build: codex("library") },
  { id: "codex-keywords", group: "Codex", title: "Codex: keywords", look: "Terms and the units that use them", desktop: true, build: codex("keywords") },
  { id: "codex-champions", group: "Codex", title: "Codex: champions / stats", look: "Records and past champions", desktop: true, build: codex("champions") },
  { id: "codex-howto", group: "Codex", title: "Codex: how to play", look: "The rules page", desktop: true, build: codex("howto") },
  { id: "codex-unit", group: "Codex v2 (#1051)", title: "A unit's Codex page", look: "Hourglass: its partners Discovered (credit), Ready to claim (sealed), Not forged yet (Wish, a pair waiting for a word)", desktop: true, build: codexV2("units", "hourglass") },
  { id: "codex-unit-mine", group: "Codex v2 (#1051)", title: "A page with your own claim", look: "Raven: Grave Tree discovered by you; the unit picker", desktop: true, build: codexV2("units", "raven") },
  { id: "codex-today", group: "Codex v2 (#1051)", title: "Today: the daily feed", look: "What tonight's batch brought, who claimed what, words found, by day", desktop: true, build: codexV2("feed") },
  { id: "codex-dictionary", group: "Codex v2 (#1051)", title: "The Dictionary", look: "'N words forged, not yet found'; each found word with credit and lineage", desktop: true, build: codexV2("words") },
  { id: "claim", group: "Codex v2 (#1051)", title: "The claim moment", look: "Three sealed cards turn over (lean A, lean B, the twist with a new word); tap one, then Claim", desktop: true, build: claim() },
  { id: "claim-done", group: "Codex v2 (#1051)", title: "Claimed: your name on it", look: "The picked design with 'Forged by @you', the other two sealed away", desktop: true, build: claim(2) },
  { id: "v2-hourglass", keep: true, group: "Rules v2 (fixtures, #1045)", title: "Hourglass: a counter, then Archer acts again", look: "3.3 counter (Grains) and 3.5 actAgain: the grain count on the card, 'Hourglass: Archer acts again'", desktop: true, build: v2("hourglass") },
  { id: "v2-mirror", keep: true, group: "Rules v2 (fixtures, #1045)", title: "Mirror copies the front enemy's rule", look: "3.5 copyRule at battle start: Mirror then fights with Hedgehog's rule", desktop: true, build: v2("mirror") },
  { id: "v2-candle", keep: true, group: "Rules v2 (fixtures, #1045)", title: "Candle survives at 1 HP, pays 1 PWR", look: "3.4 preventDeath: the intercepted Death, the stat drop, death at 0 PWR", desktop: true, build: v2("candle") },
  { id: "v2-gambler", keep: true, group: "Rules v2 (fixtures, #1045)", title: "Gambler flips on each strike", look: "3.2 flip and 3.4 replaceAmount: heads doubles the hit; the coin in the trace", desktop: true, build: v2("gambler") },
  { id: "v2-nautilus", keep: true, group: "Rules v2 (fixtures, #1045)", title: "Nautilus shields by hits taken", look: "3.3 count hitsTaken: the shield grows 1, 2, 3 with each hit", desktop: true, build: v2("nautilus") },
  { id: "v2-magnet", keep: true, group: "Rules v2 (fixtures, #1045)", title: "Magnet takes the hit aimed at Archer", look: "3.4 redirect: 'the hit on Archer lands on Magnet', once per event", desktop: true, build: v2("magnet") },
  { id: "v2-raven", keep: true, group: "Rules v2 (fixtures, #1045)", title: "Raven remembers a fallen ally's rule", look: "3.5 remember / performRemembered: Archer dies, Raven shoots in its place next turn", desktop: true, build: v2("raven") },
  { id: "v2-seed", keep: true, group: "Rules v2 (fixtures, #1045)", title: "Seed becomes a Tree after two turns", look: "3.6 delay and transform: the delay set, 'Seed becomes Tree', the Tree's rule", desktop: true, build: v2("seed") },
  { id: "v2-gambler-tails", keep: true, group: "Rules v2 (unit batch 1, #1046)", title: "Gambler: tails deals none", look: "the branch word's else: 'Gambler flips: tails', the strike hits for 0", desktop: true, build: v2("gamblerTails") },
  { id: "v2-magnet-random", keep: true, group: "Rules v2 (unit batch 1, #1046)", title: "Magnet takes a randomly aimed poison", look: "aim random: Spitter's poison on Archer lands on Magnet; front strikes pass", desktop: true, build: v2("magnetRandom") },
  { id: "v2-hourglass-awoken", keep: true, group: "Rules v2 (unit batch 1, #1046)", title: "Awoken Hourglass: every ally behind acts again", look: "Sands at 3: Archer and Bowman both act again", desktop: true, build: v2("hourglassAwoken") },
  { id: "v2-mirror-awoken", keep: true, group: "Rules v2 (unit batch 1, #1046)", title: "Awoken Mirror takes the rule and the power", look: "copyRule withPower: Mirror gains Strength equal to Hedgehog's power", desktop: true, build: v2("mirrorAwoken") },
  { id: "v2-candle-awoken", keep: true, group: "Rules v2 (unit batch 1, #1046)", title: "Awoken Candle heals its neighbours", look: "each survival heals Squire by Candle's power", desktop: true, build: v2("candleAwoken") },
  { id: "v2-gambler-awoken", keep: true, group: "Rules v2 (unit batch 1, #1046)", title: "Awoken Gambler strikes again on heads", look: "'Gambler strikes again' after a heads flip, never twice", desktop: true, build: v2("gamblerAwoken") },
  { id: "v2-nautilus-awoken", keep: true, group: "Rules v2 (unit batch 1, #1046)", title: "Awoken Nautilus counts allies' hits", look: "Shield grows by hits taken plus hits allies took", desktop: true, build: v2("nautilusAwoken") },
  { id: "v2-magnet-awoken", keep: true, group: "Rules v2 (unit batch 1, #1046)", title: "Awoken Magnet gains 1 power per redirect", look: "each redirected effect: 1 Strength to Magnet", desktop: true, build: v2("magnetAwoken") },
  { id: "v2-raven-awoken", keep: true, group: "Rules v2 (unit batch 1, #1046)", title: "Awoken Raven performs every remembered rule", look: "Archer and Bowman fall; Raven performs both rules each turn", desktop: true, build: v2("ravenAwoken") },
  { id: "v2-seed-awoken", keep: true, group: "Rules v2 (unit batch 1, #1046)", title: "Awoken Seed: the Tree shields its neighbours", look: "the Tree also gives Squire and Archer 1 Shield each turn", desktop: true, build: v2("seedAwoken") },
  { id: "v2-deja-vu", keep: true, group: "Rules v2 (fixtures, #1045)", title: "Déjà Vu (Hourglass + Mirror)", look: "§5 fusion: what the front enemy does, Déjà Vu does one turn later", desktop: true, build: v2("dejaVu") },
  { id: "v2-grave-tree", keep: true, group: "Rules v2 (fixtures, #1045)", title: "Grave Tree (Raven + Seed)", look: "§5 fusion, 3.6 summonWithRule: a Sapling with the fallen Archer's rule, at its slot", desktop: true, build: v2("graveTree") },
  ...v2unitScenes(),
];

export const sceneById = (id: string): Scene | undefined => SCENES.find((s) => s.id === id);

const kept = new Map<string, Promise<SceneBuilt>>();
/** Builds a scene: a kept one once (a failed build is tried again next time). */
export function buildScene(rt: MvpRuntime, scene: Scene): Promise<SceneBuilt> {
  if (!scene.keep) return Promise.resolve(scene.build(rt));
  let built = kept.get(scene.id);
  if (!built) {
    built = Promise.resolve(scene.build(rt));
    kept.set(scene.id, built);
    built.catch(() => kept.delete(scene.id));
  }
  return built;
}

/** Builds the kept scenes one by one in the background, so their links open at once. */
export async function warmScenes(rt: MvpRuntime): Promise<void> {
  for (const s of SCENES.filter((x) => x.keep)) {
    const started = Date.now();
    await buildScene(rt, s).then(
      () => console.log(`[scenes] ${s.id} ready in ${Date.now() - started} ms`),
      (err) => console.warn(`[scenes] ${s.id} failed: ${(err as Error).message}`),
    );
  }
}

/** The table without builders, for GET /dev/scenes, the index and the check. */
export const sceneList = () => SCENES.map(({ id, group, title, look, desktop }) => ({ id, group, title, look, desktop }));

/** Refuses a scenes server that could touch a real world (main.ts calls it at start). */
export function checkScenesEnv(env: NodeJS.ProcessEnv): boolean {
  if (env.ARENA_SCENES !== "1") return false;
  if (env.MVP_DEV !== "1" || env.MVP_DB !== ":memory:") throw new Error("ARENA_SCENES=1 needs MVP_DEV=1 and MVP_DB=:memory: (scenes never run against a real world)");
  if (env.MVP_INVITES === "1") throw new Error("ARENA_SCENES=1 runs on an open server, not with MVP_INVITES=1");
  return true;
}
