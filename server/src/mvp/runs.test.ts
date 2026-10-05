// Slice 4 (mission #574): the whole run on the server: the opponent picked at
// round start, the Crown against today's champion, the Slay, the run-end
// rating, one active run per player and runs outliving their content.
import { describe, expect, it } from "vitest";
import type { Champion, DecisionResponse, MvpContent, PlayerRef, RunView } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { ratingChange, type MvpRunState } from "../../../src/mvp/run.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { decide, startRun } from "./runs.js";
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
    expect(rt.store.slays(champ.seq)).toEqual([{ seq: champ.seq, player: maks, runId: run.runId, battleId: won.fight!.battleId, line: crown.line, contentVersion: rt.content.version, at: expect.any(String) }]);
    const wins = won.run.wins;
    const change = ratingChange(1000, { wins, endedBy: "crown-won" });
    expect(won.run.rating).toEqual(change);
    expect(rt.store.rating(maks.id)).toEqual({ player: maks, rating: change.after, runs: 1, slays: 1, daysAsChampion: 0, playoffWins: 0 });
    // No Crown ghost is saved: ghosts are shop rounds only.
    expect(rt.store.ghosts(13, { excludePlayerId: "x", contentVersion: rt.content.version })).toEqual([]);
  });

  it("a bot's Crown win writes no Slay and no rating", () => {
    const rt = world();
    const champ = weakChampion(rt);
    const run = lastRound(rt, { ...botP, id: "b2" });
    decide(rt, run, { kind: "fight" });
    const won = decide(rt, rt.store.run(run.runId)!, { kind: "fight" });
    expect(won.run).toMatchObject({ endedBy: "crown-won", rating: null });
    expect(rt.store.slays(champ.seq)).toEqual([]);
    expect(rt.store.rating("b2")).toBeUndefined();
  });

  it("with no champion, or a stale one, the run ends after round 12 (no-champion), still rated", () => {
    for (const stale of [false, true]) {
      const rt = world();
      if (stale) weakChampion(rt, { contentVersion: "old" });
      const run = lastRound(rt, maks);
      const r = decide(rt, run, { kind: "fight" });
      expect(r.run).toMatchObject({ phase: "over", endedBy: "no-champion" });
      expect(r.run.rating).toEqual(ratingChange(1000, { wins: r.run.wins, endedBy: "no-champion" }));
      expect(rt.store.rating(maks.id)?.runs).toBe(1);
    }
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

  it("ends a run whose content is no longer live: on its next decision, or when its player starts a run", () => {
    const store = new SqliteMvpStore(":memory:");
    const old = world({ store });
    const a = startRun(old, maks);
    const b = startRun(old, { ...maks, id: "p2" });
    const content = { ...old.content, version: "retuned" };
    const rt = world({ store, content });
    const ended = decide(rt, store.run(a.runId)!, { kind: "buy", slot: 0 });
    expect(ended.run).toMatchObject({ phase: "over", endedBy: "content-changed", rating: null, line: [] });
    expect(store.rating(maks.id)).toBeUndefined();
    const fresh = startRun(rt, { ...maks, id: "p2" });
    expect(fresh.runId).not.toBe(b.runId);
    expect(fresh.contentVersion).toBe("retuned");
    expect(store.run(b.runId)).toMatchObject({ phase: "over", endedBy: "content-changed" });
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
    expect(again.db.prepare("SELECT name FROM mvp_migrations").all()).toEqual([{ name: "04-runs.sql" }]);
    again.close();
  });
});

describe("MVP run-end rating", () => {
  it("is Elo-style: wins plus the slay bonus against the expected result", () => {
    expect(ratingChange(1000, { wins: 12, endedBy: "crown-won" })).toEqual({ before: 1000, after: 1016, expected: 0.5, actual: 1 });
    expect(ratingChange(1000, { wins: 0, endedBy: "out-of-hearts" })).toMatchObject({ after: 984, actual: 0 });
    // A higher rating expects more: the same run gains less.
    expect(ratingChange(1200, { wins: 9, endedBy: "crown-lost" }).after - 1200).toBeLessThan(ratingChange(1000, { wins: 9, endedBy: "crown-lost" }).after - 1000);
  });
});
