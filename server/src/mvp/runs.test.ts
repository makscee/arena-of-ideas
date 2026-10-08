// Slice 4 (mission #574): the whole run on the server: the opponent picked at
// round start, the Crown against today's champion, the Slay, the run-end
// rating, one active run per player and runs outliving their content.
import { describe, expect, it } from "vitest";
import type { Champion, DecisionResponse, FightResult, Ghost, MvpContent, Outcome, PlayerRef, RunView } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { DRAW_SCORE, ratingChange, ratingK, type MvpRunState } from "../../../src/mvp/run.js";
import { rngStep } from "../../../src/rng.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { decide, GHOST_PICK_POOL, preview, startRun } from "./runs.js";
import { MemoryMvpStore } from "./store.js";
import { mvpRuntime, type MvpDeps } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";

const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };
const botP: PlayerRef = { id: "b1", name: "bot-Ash", bot: true };

function world(extra: Partial<MvpDeps> = {}) {
  let n = 7;
  return mvpRuntime({ content: mvpContent(), seed: () => (n = (n * 1103515245 + 12345) >>> 0), ...extra });
}

/** The strongest line the content allows: its five biggest units, awoken. */
function bigLine(content: MvpContent) {
  const units = [...content.units].sort((a, b) => b.base.hp + b.base.pwr - (a.base.hp + a.base.pwr)).slice(0, 5);
  return units.map((u, i) => lineUnitOf(u, `u${i + 1}`, 3));
}

/** A run at its last shop round with a strong line and every heart left. */
function lastRound(rt: ReturnType<typeof world>, player: PlayerRef): MvpRunState {
  const run = startRun(rt, player);
  const at12: MvpRunState = { ...run, round: rt.rules.rounds, line: bigLine(rt.content), nextUid: 6 };
  rt.store.putRun(at12);
  return at12;
}

/** A champion that can't win: one 1 HP unit. */
function weakChampion(rt: ReturnType<typeof world>, over: Partial<Champion> = {}): Champion {
  const u = rt.content.units[0]!;
  const c: Champion = { seq: rt.today().seq, day: rt.today().day, player: botP, line: [{ ...lineUnitOf(u, "c1"), stats: { pwr: 0, hp: 1 } }], since: "2026-10-05T00:00:00.000Z", contentVersion: rt.content.version, ...over };
  rt.store.putChampion(c);
  return c;
}

describe("MVP run server", () => {
  it("picks each round's opponent at round start, and the fight uses it", () => {
    const rt = world();
    let run = startRun(rt, maks);
    expect(run.nextOpponent).toMatchObject({ round: 1 });
    expect(run.opponent?.ghostId).toBeTruthy();
    decide(rt, run, { kind: "buy", slot: 0 });
    run = rt.store.run(run.runId)!;
    const shown = run.opponent!;
    const r = decide(rt, run, { kind: "fight" });
    expect(r.fight!.opponent.ghostId).toBe(shown.ghostId);
    expect(r.run.nextOpponent).toMatchObject({ round: 2 });
    expect(rt.store.run(run.runId)!.opponent!.round).toBe(2);
    expect(r.run).not.toHaveProperty("opponent");
  });

  it("after round 12 the Crown fights today's champion; a human's win is a Slay and moves the rating once", () => {
    const rt = world();
    const champ = weakChampion(rt);
    const run = lastRound(rt, maks);
    const r12 = decide(rt, run, { kind: "fight" });
    expect(r12.run).toMatchObject({ phase: "crown", round: 13, gold: 0, offers: [], nextOpponent: { player: botP, round: 13 } });
    let crown = rt.store.run(run.runId)!;
    expect(crown.crownSeq).toBe(champ.seq);
    for (const d of [{ kind: "buy", slot: 0 }, { kind: "reroll" }, { kind: "sell", index: 0 }, { kind: "reorder", from: 0, to: 1 }] as const) {
      expect(() => decide(rt, crown, d)).toThrow(/only the Crown fight/);
    }
    const won = decide(rt, crown, { kind: "fight" });
    expect(won.fight).toMatchObject({ kind: "crown", round: 13, outcome: "win", opponent: { ghostId: `champion-${champ.seq}`, player: botP } });
    expect(won.run).toMatchObject({ phase: "over", endedBy: "crown-won", nextOpponent: null });
    expect(rt.store.battle(won.fight!.battleId)?.kind).toBe("crown");
    crown = rt.store.run(run.runId)!;
    expect(rt.store.slays(champ.seq)).toEqual([{ seq: champ.seq, player: maks, runId: run.runId, battleId: won.fight!.battleId, line: crown.line, contentVersion: rt.content.version, at: expect.any(String), rating: 1000 }]);
    const change = ratingChange(1000, 0, won.run);
    expect(won.run.rating).toEqual(change);
    expect(rt.store.rating(maks.id)).toEqual({ player: maks, rating: change.after, runs: 1, slays: 1, daysAsChampion: 0, playoffWins: 0 });
    // No Crown ghost is saved: ghosts are shop rounds only.
    expect(rt.store.ghosts(13, { excludePlayerId: "x" })).toEqual([]);
  });

  it("a bot's Crown win writes a Slay like a human's and counts it in its records; its rating never moves (#587)", () => {
    const rt = world();
    const champ = weakChampion(rt);
    const bot2 = { ...botP, id: "b2" };
    for (let i = 0; i < 2; i++) {
      const run = lastRound(rt, bot2);
      decide(rt, run, { kind: "fight" });
      const won = decide(rt, rt.store.run(run.runId)!, { kind: "fight" });
      expect(won.run).toMatchObject({ endedBy: "crown-won", rating: null });
      expect(rt.store.slays(champ.seq).at(-1)).toMatchObject({ seq: champ.seq, player: bot2, runId: run.runId, battleId: won.fight!.battleId });
    }
    expect(rt.store.slays(champ.seq)).toHaveLength(2);
    expect(rt.store.rating("b2")).toEqual({ player: bot2, rating: rt.rules.ratingStart, runs: 0, slays: 2, daysAsChampion: 0, playoffWins: 0 });
    // A bot's lost Crown writes nothing.
    const lost = lastRound(rt, { ...botP, id: "b3" });
    decide(rt, lost, { kind: "fight" });
    rt.store.putRun({ ...rt.store.run(lost.runId)!, line: [] });
    expect(decide(rt, rt.store.run(lost.runId)!, { kind: "fight" }).run.endedBy).toBe("crown-lost");
    expect(rt.store.slays(champ.seq)).toHaveLength(2);
    expect(rt.store.rating("b3")).toBeUndefined();
  });

  it("with no champion the run ends after round 12 (no-champion), still rated; a champion made on another pool is fought (M2-2)", () => {
    const rt = world();
    const run = lastRound(rt, maks);
    const r = decide(rt, run, { kind: "fight" });
    expect(r.run).toMatchObject({ phase: "over", endedBy: "no-champion" });
    expect(r.run.rating).toEqual(ratingChange(1000, 0, r.run));
    expect(rt.store.rating(maks.id)?.runs).toBe(1);
    const other = world();
    weakChampion(other, { contentVersion: "old" });
    const crown = decide(other, lastRound(other, maks), { kind: "fight" });
    expect(crown.run.phase).toBe("crown");
    expect(decide(other, other.store.run(crown.run.runId)!, { kind: "fight" }).run.endedBy).toBe("crown-won");
  });

  it("out of hearts ends the run before the Crown, rated; ratings carry over runs", () => {
    const rt = world();
    weakChampion(rt);
    for (let i = 0; i < 2; i++) {
      const run = { ...startRun(rt, maks), hearts: 1 };
      rt.store.putRun(run);
      let r: DecisionResponse = { run };
      let cur = run;
      decide(rt, cur, { kind: "buy", slot: 0 });
      while (r.run.phase !== "over") {
        cur = rt.store.run(run.runId)!;
        r = decide(rt, cur, { kind: "fight" });
      }
      if (r.run.endedBy === "out-of-hearts") expect(r.run.round).toBeLessThanOrEqual(12);
      expect(r.run.rating).not.toBeNull();
      expect(rt.store.rating(maks.id)?.rating).toBe(r.run.rating!.after);
      expect(rt.store.rating(maks.id)?.runs).toBe(i + 1);
    }
  });

  it("keeps one active run per player, even for parallel starts", async () => {
    const app = createMvpApp(world());
    const call = async <T>(method: string, path: string, player?: string, body?: unknown) => {
      const res = await app.request(`/api/v1${path}`, { method, headers: { "content-type": "application/json", ...(player ? { "X-Arena-Player": player } : {}) }, ...(body ? { body: JSON.stringify(body) } : {}) });
      return (await res.json()) as T;
    };
    const p = await call<PlayerRef>("POST", "/players", undefined, { name: "Maks" });
    const runs = await Promise.all([1, 2, 3].map(() => call<RunView>("POST", "/runs", p.id)));
    expect(new Set(runs.map((r) => r.runId)).size).toBe(1);
  });

  it("a content change ends nothing (M2-2): the run takes its next decision, and its player's start hands it back", () => {
    const store = new SqliteMvpStore(":memory:");
    const old = world({ store });
    const a = startRun(old, maks);
    const b = startRun(old, { ...maks, id: "p2" });
    const content = { ...old.content, version: "retuned" };
    const ends: string[] = [];
    const rt = world({ store, content, hooks: [{ onRunEnd: (r) => ends.push(`${r.runId} ${r.endedBy}`) }] });
    const went = decide(rt, store.run(a.runId)!, { kind: "buy", slot: 0 });
    expect(went.run.phase).toBe("shop");
    expect(went.run.line).toHaveLength(1);
    expect(startRun(rt, { ...maks, id: "p2" }).runId).toBe(b.runId);
    expect(store.run(b.runId)!.contentVersion).toBe(old.content.version);
    expect(startRun(rt, { ...maks, id: "p3" }).contentVersion).toBe("retuned");
    expect(ends).toEqual([]);
  });

  it("plays a whole run on the SQLite store and reopens it from the file", () => {
    const dir = `${process.env.TMPDIR ?? "/tmp"}/mvp-store-${process.pid}-${Date.now()}.db`;
    const store = new SqliteMvpStore(dir);
    const rt = world({ store });
    weakChampion(rt);
    const run = lastRound(rt, maks);
    decide(rt, run, { kind: "fight" });
    const end = decide(rt, store.run(run.runId)!, { kind: "fight" });
    store.close();
    const again = new SqliteMvpStore(dir);
    expect(again.run(run.runId)).toMatchObject({ phase: "over", endedBy: end.run.endedBy, rating: end.run.rating });
    expect(again.rating(maks.id)?.runs).toBe(1);
    expect(again.db.prepare("SELECT name FROM mvp_migrations").all()).toEqual([{ name: "04-runs.sql" }, { name: "11-stats.sql" }, { name: "13-invites.sql" }, { name: "13b-join.sql" }, { name: "m2-01-units.sql" }, { name: "m2-02-pins.sql" }, { name: "m2-02b-day-backfill.sql" }, { name: "m2-03-ideas.sql" }, { name: "m2-04-ideas.sql" }, { name: "m2-05-words.sql" }, { name: "m2-08-votes.sql" }]);
    again.close();
  });
});

describe("MVP bench on the server (R3-13)", () => {
  it("ghosts and the Slay hold the line only; the bench stays in the run", () => {
    const rt = world();
    const champ = weakChampion(rt);
    const run = lastRound(rt, maks);
    const benched: MvpRunState = { ...run, bench: [lineUnitOf(rt.content.units[0]!, "b1", 1, rt.rules)] };
    rt.store.putRun(benched);
    decide(rt, benched, { kind: "fight" });
    const ghost = rt.store.ghosts(rt.rules.rounds, { excludePlayerId: "x" }).find((g) => g.runId === run.runId)!;
    expect(ghost.line.map((u) => u.uid)).toEqual(run.line.map((u) => u.uid));
    const crown = rt.store.run(run.runId)!;
    expect(crown.bench.map((u) => u.uid)).toEqual(["b1"]);
    decide(rt, crown, { kind: "fight" });
    expect(rt.store.slays(champ.seq)[0]!.line.map((u) => u.uid)).toEqual(run.line.map((u) => u.uid));
  });

  it("fuses a line unit with a bench unit through decide and preview; the fused unit stands on the line", () => {
    const rt = world();
    const run = startRun(rt, maks);
    const [a, b, c] = rt.content.units;
    const s: MvpRunState = { ...run, line: [lineUnitOf(a!, "l1", 3, rt.rules)], bench: [lineUnitOf(b!, "b1", 1, rt.rules), lineUnitOf(c!, "b2", 3, rt.rules)] };
    rt.store.putRun(s);
    const d = { kind: "fuse", first: 6, second: 0 } as const;
    expect(preview(rt, s, d).run.line[0]).toMatchObject({ kind: "fused", uid: "b2" });
    const r = decide(rt, s, d).run;
    expect(r.line).toHaveLength(1);
    expect(r.line[0]).toMatchObject({ kind: "fused", uid: "b2", fusion: { first: c!.id, second: a!.id } });
    expect(r.bench.map((u) => u.uid)).toEqual(["b1"]);
    expect(rt.store.fusion(c!.id, a!.id)).toMatchObject({ discoveredBy: maks });
  });

  it("a run stored before the bench reopens from SQLite with an empty bench and no bench slots", () => {
    const store = new SqliteMvpStore(":memory:");
    const rt = world({ store });
    const run = startRun(rt, maks);
    const { bench: _b, ...old } = { ...run, rules: (({ benchSize: _s, ...r }) => r)(run.rules) };
    store.putRun(old as MvpRunState);
    const back = store.run(run.runId)!;
    expect(back.bench).toEqual([]);
    expect(back.rules.benchSize).toBeUndefined();
    expect(() => decide(rt, back, { kind: "reorder", from: 0, to: 5 })).toThrow(/bad move/);
  });
});

describe("MVP giving up and stamped ratings (round 2)", () => {
  /** Plays `player`'s run from round 1 with `losses` lost and `wins` won fights:
   * a strong line wins, an empty one loses. */
  function playTo(rt: ReturnType<typeof world>, player: PlayerRef, outcomes: Outcome[]): MvpRunState {
    let run = startRun(rt, player);
    for (const o of outcomes) {
      run = { ...run, line: o === "win" ? bigLine(rt.content) : [], nextUid: 6 };
      rt.store.putRun(run);
      expect(decide(rt, run, { kind: "fight" }).fight?.outcome).toBe(o);
      run = rt.store.run(run.runId)!;
    }
    return run;
  }
  const api = (rt: ReturnType<typeof world>) => {
    const app = createMvpApp(rt);
    return async (path: string, player?: PlayerRef) => {
      const res = await app.request(`/api/v1${path}`, { method: "POST", headers: player ? { "X-Arena-Player": player.id } : {} });
      return { status: res.status, json: (await res.json()) as RunView & { error?: string } };
    };
  };

  it("POST /runs/:id/abandon at round 4 with 4 hearts: the change is the fights played plus 4 losses", async () => {
    const rt = world();
    rt.store.addPlayer(maks);
    const run = playTo(rt, maks, ["win", "loss", "win"]);
    expect(run).toMatchObject({ round: 4, hearts: 4 });
    const opp = run.opponent!.rating;
    const post = api(rt);
    const res = await post(`/runs/${run.runId}/abandon`, maks);
    expect(res.status).toBe(200);
    expect(res.json).toMatchObject({ phase: "over", endedBy: "abandoned", hearts: 0, losses: 5, forfeit: { fights: 4, opponentRating: opp } });
    const losses = [0, 1, 2, 3].map((i): FightResult => ({ ...run.fights[1]!, battleId: `f${i}`, round: 4, opponent: { ...run.fights[1]!.opponent, rating: opp } }));
    expect(res.json.rating).toEqual(ratingChange(1000, 0, { fights: [...run.fights, ...losses] }));
    expect(res.json.rating!.after).toBeLessThan(1000);
    expect(rt.store.rating(maks.id)).toMatchObject({ rating: res.json.rating!.after, runs: 1 });
    // ended once: a second give-up is refused, and a new run starts
    expect((await post(`/runs/${run.runId}/abandon`, maks)).status).toBe(409);
    expect(startRun(rt, maks).runId).not.toBe(run.runId);
  });

  it("only the run's player can give it up; an unknown run is 404", async () => {
    const rt = world();
    const ann = { ...maks, id: "p2", name: "Ann" };
    rt.store.addPlayer(maks);
    rt.store.addPlayer(ann);
    const run = startRun(rt, maks);
    const post = api(rt);
    expect((await post(`/runs/${run.runId}/abandon`, ann)).status).toBe(401);
    expect((await post(`/runs/${run.runId}/abandon`)).status).toBe(401);
    expect((await post(`/runs/nope/abandon`, maks)).status).toBe(404);
    expect(rt.store.run(run.runId)?.phase).toBe("shop");
  });

  it("giving up at the Crown scores like a lost Crown, against the champion's rating", async () => {
    const rt = world();
    rt.store.addPlayer(maks);
    weakChampion(rt, { rating: 1250 });
    const run = lastRound(rt, maks);
    decide(rt, run, { kind: "fight" });
    const crown = rt.store.run(run.runId)!;
    expect(crown.phase).toBe("crown");
    const res = await api(rt)(`/runs/${run.runId}/abandon`, maks);
    expect(res.json).toMatchObject({ endedBy: "abandoned", forfeit: { fights: 1, opponentRating: 1250 } });
    expect(res.json.rating).toEqual(ratingChange(1000, 0, { fights: [...crown.fights, { ...crown.fights[0]!, kind: "crown", outcome: "loss", opponent: { ...crown.fights[0]!.opponent, rating: 1250 } }] }));
  });

  it("giving up a run on a pool that is no longer live is an ordinary abandon, rated (M2-2)", async () => {
    const rt = world();
    rt.store.addPlayer(maks);
    const run = startRun(rt, maks);
    rt.store.putRun({ ...run, contentVersion: "old-content" });
    const res = await api(rt)(`/runs/${run.runId}/abandon`, maks);
    expect(res.status).toBe(200);
    expect(res.json).toMatchObject({ phase: "over", endedBy: "abandoned" });
    expect(res.json.rating).not.toBeNull();
    expect(rt.store.rating(maks.id)?.runs).toBe(1);
  });

  it("ghosts and slays carry their owner's rating from the run's start; bots carry botRating", () => {
    const rt = world();
    rt.store.putRating({ player: maks, rating: 1137, runs: 20, slays: 0, daysAsChampion: 0, playoffWins: 0 });
    weakChampion(rt);
    const run = lastRound(rt, maks);
    decide(rt, run, { kind: "fight" });
    const ghost = rt.store.ghosts(rt.rules.rounds, { excludePlayerId: "x" }).find((g) => g.runId === run.runId);
    expect(ghost?.rating).toBe(1137);
    const won = decide(rt, rt.store.run(run.runId)!, { kind: "fight" });
    expect(won.run.endedBy).toBe("crown-won");
    expect(rt.store.slays(rt.today().seq).at(-1)?.rating).toBe(1137);
    expect(won.run.rating?.k).toBe(10);
    const bot = startRun(rt, botP);
    expect(bot.ratingAtStart).toBe(rt.rules.botRating);
    expect(bot.opponent?.rating).toBe(1000);
  });
});

describe("MVP run-end rating: per-fight Elo (docs/round2/rating.md)", () => {
  const fight = (outcome: Outcome, i: number, rating = 1000): FightResult => ({ battleId: `b${i}`, kind: "round", round: i + 1, opponent: { ghostId: `g${i}`, player: botP, round: i + 1, rating }, outcome, heartsLost: outcome === "loss" ? 1 : 0, heartsAfter: 5 });
  const fights = (...outcomes: Outcome[]): FightResult[] => outcomes.map((o, i) => fight(o, i));
  const record = (wins: number, losses: number): FightResult[] => fights(...Array<Outcome>(wins).fill("win"), ...Array<Outcome>(losses).fill("loss"));
  const crown = (outcome: Outcome, rating = 1250): FightResult => ({ ...fight(outcome, 12, rating), kind: "crown", round: 13 });
  const VETERAN = 15;

  it("pins the tunables: K 32 for the first 5 runs, 16 for the next 10, then 10; draws 0.5; bots 1000", () => {
    expect(DRAW_SCORE).toBe(0.5);
    expect([0, 4, 5, 14, 15, 200].map((runs) => ratingK(runs))).toEqual([32, 32, 16, 16, 10, 10]);
    expect(world().rules).toMatchObject({ ratingStart: 1000, botRating: 1000 });
  });

  it("matches rating.md's run-end table (a 1000 player against 1000 ghosts, champion 1250)", () => {
    const rows: [string, FightResult[], number, number][] = [
      ["0-5", record(0, 5), -25, -80],
      ["3-5", record(3, 5), -10, -32],
      ["6-5", record(6, 5), 5, 16],
      ["7-5", record(7, 5), 10, 32],
      ["9-3, Crown won", [...record(9, 3), crown("win")], 38, 122],
      ["12-0, Crown lost", [...record(12, 0), crown("loss")], 58, 186],
      ["12-0, Crown won", [...record(12, 0), crown("win")], 68, 218],
    ];
    for (const [label, fs, veteran, fresh] of rows) {
      expect([label, ratingChange(1000, VETERAN, { fights: fs }).after - 1000, ratingChange(1000, 0, { fights: fs }).after - 1000]).toEqual([label, veteran, fresh]);
    }
    // expected and actual are the run's expected and actual wins
    expect(ratingChange(1000, VETERAN, { fights: [...record(9, 3), crown("win")] })).toEqual({ before: 1000, after: 1038, expected: 6 + 1 / (1 + 10 ** (250 / 400)), actual: 10, k: 10 });
    // a draw scores half; an equal opponent is +5 / −5 for a veteran
    expect(ratingChange(1000, VETERAN, { fights: fights("draw", "draw") }).after).toBe(1000);
    expect(ratingChange(1000, VETERAN, { fights: fights("win") }).after).toBe(1005);
    // stronger opponents pay more for a win, weaker ones cost more for a loss
    expect(ratingChange(1000, VETERAN, { fights: [fight("win", 0, 1200)] }).after).toBeGreaterThan(1005);
    expect(ratingChange(1000, VETERAN, { fights: [fight("loss", 0, 800)] }).after).toBeLessThan(995);
    // a fight saved before round 2 has no opponent rating: a start rating
    const { rating: _r, ...old } = fight("win", 0).opponent;
    expect(ratingChange(1000, VETERAN, { fights: [{ ...fight("win", 0), opponent: old as FightResult["opponent"] }] }).after).toBe(1005);
  });

  it("a correctly rated player's mean change is about 0, however the run is cut short", () => {
    // Players and ghosts rated 700..1300, each fight won with exactly its
    // expected chance (5% of fights drawn, same mean); a run stops at 5 lost
    // hearts or after round 12, then maybe fights a 1250 Crown.
    let rng = 592;
    const rand = () => {
      const { value, state } = rngStep(rng);
      rng = state;
      return value;
    };
    const runs = 20000;
    let total = 0;
    for (let i = 0; i < runs; i++) {
      const me = 700 + rand() * 600;
      const fs: FightResult[] = [];
      let hearts = 5;
      const play = (opp: number): Outcome => {
        const e = 1 / (1 + 10 ** ((opp - me) / 400));
        const roll = rand();
        // a draw takes 5% of the mass, half from wins and half from losses: E[S] stays e
        if (Math.abs(roll - e) < 0.025) return "draw";
        const o: Outcome = roll < e ? "win" : "loss";
        if (o === "loss") hearts -= 1;
        return o;
      };
      for (let round = 0; round < 12 && hearts > 0; round++) {
        const opp = 700 + rand() * 600;
        fs.push(fight(play(opp), round, opp));
      }
      if (hearts > 0) fs.push(crown(play(1250)));
      total += ratingChange(me, VETERAN, { fights: fs }).after - me;
    }
    expect(Math.abs(total / runs)).toBeLessThan(0.4);
  });

  it("giving up rates every heart left as a loss, never better than playing on", () => {
    const played = fights("win", "loss", "win"); // 4 hearts left at round 4
    const abandoned = ratingChange(1000, VETERAN, { fights: played, forfeit: { fights: 4, opponentRating: 1100 } });
    expect(abandoned).toEqual(ratingChange(1000, VETERAN, { fights: [...played, ...[0, 1, 2, 3].map((i) => fight("loss", 3 + i, 1100))] }));
    // any way of playing on, from four straight losses to winning out, does at least as well
    for (const tail of [fights("loss", "loss", "loss", "loss"), fights("win", "loss", "loss", "loss", "loss"), record(9, 0)]) {
      const moved = tail.map((f, i) => ({ ...f, round: 4 + i, opponent: { ...f.opponent, rating: 1100 } }));
      expect(ratingChange(1000, VETERAN, { fights: [...played, ...moved] }).after).toBeGreaterThanOrEqual(abandoned.after);
    }
  });

  it("the Crown against your own champion is worth about ±K/2 (you face your own past rating)", () => {
    expect(ratingChange(1000, VETERAN, { fights: [crown("win", 1000)] }).after - 1000).toBe(5);
    expect(ratingChange(1000, VETERAN, { fights: [crown("loss", 1000)] }).after - 1000).toBe(-5);
    expect(ratingChange(1000, 0, { fights: [crown("win", 1000)] }).after - 1000).toBe(16);
  });
});

describe("MVP run server fixes (#579 check of a3c9b113)", () => {
  it("an empty line still fights and loses a heart, so a broke player with no units is never stuck", () => {
    const rt = world();
    let run = startRun(rt, maks);
    // Reroll until broke: 0 gold, no units.
    for (let i = 0; i < rt.rules.goldPerRound; i++) decide(rt, rt.store.run(run.runId)!, { kind: "reroll" });
    run = rt.store.run(run.runId)!;
    expect(run).toMatchObject({ gold: 0, line: [] });
    expect(() => decide(rt, run, { kind: "buy", slot: 0 })).toThrow();
    const r = decide(rt, run, { kind: "fight" });
    expect(r.fight).toMatchObject({ kind: "round", round: 1, outcome: "loss", heartsLost: 1 });
    expect(r.run).toMatchObject({ round: 2, gold: rt.rules.goldPerRound, hearts: rt.rules.hearts - 1 });
    // The walkover is a stored battle the viewer can open, and nobody's ghost.
    const battle = rt.store.battle(r.fight!.battleId)!;
    expect(battle.log.map((e) => e.type)).toEqual(["BattleStart", "BattleEnd"]);
    expect(battle.winner).toBe("B");
    expect(rt.store.ghosts(1, { excludePlayerId: "x" })).toEqual([]);
    // Fighting empty to the end runs out of hearts and the next start is a new run.
    let cur = r;
    while (cur.run.phase !== "over") cur = decide(rt, rt.store.run(run.runId)!, { kind: "fight" });
    expect(cur.run).toMatchObject({ endedBy: "out-of-hearts", hearts: 0 });
    expect(startRun(rt, maks).runId).not.toBe(run.runId);
  });

  it("the Crown fights the champion of the moment: a rollover after round 12 switches it and the Slay's seq", () => {
    const rt = world();
    const old = weakChampion(rt);
    const run = lastRound(rt, maks);
    decide(rt, run, { kind: "fight" });
    expect(rt.store.run(run.runId)!.crownSeq).toBe(old.seq);
    rt.store.putDay({ ...rt.today(), seq: old.seq + 1 }); // the rollover: day 2 starts, crowning bot-Next
    const next = weakChampion(rt, { seq: old.seq + 1, player: { ...botP, id: "b9", name: "bot-Next" } });
    const won = decide(rt, rt.store.run(run.runId)!, { kind: "fight" });
    expect(won.fight).toMatchObject({ kind: "crown", outcome: "win", opponent: { ghostId: `champion-${next.seq}`, player: { id: "b9" } } });
    expect(rt.store.slays(old.seq)).toEqual([]);
    expect(rt.store.slays(next.seq)).toHaveLength(1);
  });

  it("a newer champion made on another pool replaces the Crown's opponent like any (M2-2)", () => {
    const rt = world();
    const old = weakChampion(rt);
    const run = lastRound(rt, maks);
    decide(rt, run, { kind: "fight" });
    rt.store.putDay({ ...rt.today(), seq: old.seq + 1 });
    weakChampion(rt, { seq: old.seq + 1, contentVersion: "old" });
    const won = decide(rt, rt.store.run(run.runId)!, { kind: "fight" });
    expect(won.fight!.opponent.ghostId).toBe(`champion-${old.seq + 1}`);
    expect(rt.store.slays(old.seq + 1)).toHaveLength(1);
  });

  it("a preview of a run on content that is no longer live goes through on the run's own pool (M2-2)", async () => {
    const store = new SqliteMvpStore(":memory:");
    const old = world({ store });
    const a = startRun(old, maks);
    const bought = decide(old, a, { kind: "buy", slot: 0 }).run;
    expect(bought.line).toHaveLength(1);
    // The retuned content drops the unit the run holds; the run has no stored
    // pool here, so it plays on with the live one, and selling needs no unit.
    const content = { ...old.content, version: "retuned", units: old.content.units.filter((u) => u.id !== bought.line[0]!.unitId) };
    const rt = world({ store, content });
    expect(preview(rt, store.run(a.runId)!, { kind: "sell", index: 0 }).run.line).toHaveLength(0);
    store.addPlayer(maks);
    const app = createMvpApp(rt);
    const res = await app.request(`/api/v1/runs/${a.runId}/preview`, { method: "POST", headers: { "content-type": "application/json", "X-Arena-Player": maks.id }, body: JSON.stringify({ kind: "sell", index: 0 }) });
    expect(res.status).toBe(200);
  });

  it("a slayer's round-12 team is nobody's opponent until the day ends", () => {
    const rt = world();
    weakChampion(rt);
    const run = lastRound(rt, maks);
    decide(rt, run, { kind: "fight" });
    const won = decide(rt, rt.store.run(run.runId)!, { kind: "fight" });
    expect(won.run.endedBy).toBe("crown-won");
    // The slayer's round-12 line is the only saved team at round 12.
    expect(rt.store.ghosts(rt.rules.rounds, { excludePlayerId: "x" }).map((g) => g.runId)).toEqual([run.runId]);
    const ann: PlayerRef = { id: "p2", name: "Ann", bot: false };
    const slayerGhost = `${run.runId}-r${rt.rules.rounds}`;
    // Ann reaches round 12 with no opponent picked yet: the fight picks it now.
    const opponentAt12 = () => {
      const r = { ...lastRound(rt, ann), opponent: null };
      const fought = decide(rt, r, { kind: "fight" });
      rt.store.putRun({ ...rt.store.run(r.runId)!, phase: "over" }); // Ann's next start is a new run
      return fought.fight!.opponent.ghostId;
    };
    for (let i = 0; i < 5; i++) expect(opponentAt12()).not.toBe(slayerGhost);
    rt.store.putDay({ ...rt.today(), seq: rt.today().seq + 1 }); // the day ends
    expect(opponentAt12()).toBe(slayerGhost);
  });

  it("a run waiting in the Crown hides its round-12 team until the Crown is fought", () => {
    const rt = world();
    weakChampion(rt);
    const run = lastRound(rt, maks);
    decide(rt, run, { kind: "fight" });
    expect(rt.store.run(run.runId)!.phase).toBe("crown");
    const slayerGhost = `${run.runId}-r${rt.rules.rounds}`;
    const ann: PlayerRef = { id: "p2", name: "Ann", bot: false };
    const opponentAt12 = () => {
      const r = { ...lastRound(rt, ann), opponent: null };
      const fought = decide(rt, r, { kind: "fight" });
      rt.store.putRun({ ...rt.store.run(r.runId)!, phase: "over" });
      return fought.fight!.opponent.ghostId;
    };
    for (let i = 0; i < 5; i++) expect(opponentAt12()).not.toBe(slayerGhost);
    // The Crown went the champion's way: no slay, so the team is an opponent again.
    rt.store.putRun({ ...rt.store.run(run.runId)!, phase: "over", endedBy: "crown-lost" });
    expect(opponentAt12()).toBe(slayerGhost);
  });

  it("a bot's Crown win hides its round-12 team like a human's, until the day ends (#587)", () => {
    const rt = world();
    weakChampion(rt);
    const run = lastRound(rt, { ...botP, id: "b2" });
    decide(rt, run, { kind: "fight" });
    expect(decide(rt, rt.store.run(run.runId)!, { kind: "fight" }).run.endedBy).toBe("crown-won");
    const slayerGhost = `${run.runId}-r${rt.rules.rounds}`;
    const opponentAt12 = () => {
      const r = { ...lastRound(rt, maks), opponent: null };
      const fought = decide(rt, r, { kind: "fight" });
      rt.store.putRun({ ...rt.store.run(r.runId)!, phase: "over" });
      return fought.fight!.opponent.ghostId;
    };
    for (let i = 0; i < 5; i++) expect(opponentAt12()).not.toBe(slayerGhost);
    rt.store.putDay({ ...rt.today(), seq: rt.today().seq + 1 }); // the day ends
    expect(opponentAt12()).toBe(slayerGhost);
  });

  it("a ghost pick reads only the round's newest GHOST_PICK_POOL saved teams", () => {
    for (const store of [new MemoryMvpStore(), new SqliteMvpStore(":memory:")]) {
      const g = (i: number, player = botP, round = 1): Ghost => ({ ghostId: `g${i}`, runId: `r${i}`, player, round, line: [], contentVersion: "v", createdAt: "t", rating: 1000 });
      for (let i = 0; i < 10; i++) store.addGhost(g(i));
      store.addGhost(g(10, maks));
      store.addGhost(g(11, botP, 2));
      const ids = (limit?: number) => store.ghosts(1, { excludePlayerId: maks.id, ...(limit === undefined ? {} : { limit }) }).map((x) => x.ghostId);
      expect(ids()).toEqual(["g0", "g1", "g2", "g3", "g4", "g5", "g6", "g7", "g8", "g9"]);
      expect(ids(3)).toEqual(["g7", "g8", "g9"]);
      expect(ids(30)).toHaveLength(10);
    }
    expect(GHOST_PICK_POOL).toBe(200);
  });
});
