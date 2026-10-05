// Slice 5 (mission #574): the day on the server: slayers hidden until the day
// ends, the day-end playoff crowning the next champion, a day with no slayers
// keeping the champion, the 04:00 Moscow rollover and the dev "end day now".
import { describe, expect, it, vi } from "vitest";
import type { BattleRecord, Champion, DayView, LineUnit, MvpContent, PlayerRef, Rating } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { ratingChange, type MvpRunState } from "../../../src/mvp/run.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { dayRollover, END_DAY_RETRY_MS, endDay } from "./day.js";
import { decide, startRun } from "./runs.js";
import { mvpRuntime, type MvpDeps, type MvpRuntime } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore } from "./store.js";

/** A store whose next `fails` writes of day `seq` throw, as a full disk would. */
class FlakyDayStore extends MemoryMvpStore {
  tries = 0;
  constructor(private seq: number, private fails: number) {
    super();
  }
  override putDay(d: Parameters<MemoryMvpStore["putDay"]>[0]): void {
    if (d.seq === this.seq && (this.tries++, this.fails-- > 0)) throw new Error("disk full");
    super.putDay(d);
  }
}

const botP: PlayerRef = { id: "b1", name: "bot-Ash", bot: true };

function world(extra: Partial<MvpDeps> = {}) {
  let n = 7;
  let clock = new Date("2026-10-05T10:00:00.000Z"); // 13:00 Moscow
  const rt = mvpRuntime({ content: mvpContent(), seed: () => (n = (n * 1103515245 + 12345) >>> 0), now: () => clock, dev: true, ...extra });
  const app = createMvpApp(rt);
  const call = async <T>(method: string, path: string, player?: PlayerRef) => {
    const res = await app.request(`/api/v1${path}`, { method, headers: player ? { "X-Arena-Player": player.id } : {} });
    return { status: res.status, json: (await res.json()) as T };
  };
  const human = (name: string): PlayerRef => {
    const p = { id: `id-${name}`, name, bot: false };
    rt.store.addPlayer(p);
    return p;
  };
  return { rt, call, human, setClock: (iso: string) => (clock = new Date(iso)) };
}

function bigLine(content: MvpContent, n = 5): LineUnit[] {
  return [...content.units].sort((a, b) => b.base.hp + b.base.pwr - (a.base.hp + a.base.pwr)).slice(0, n).map((u, i) => lineUnitOf(u, `u${i + 1}`, 3));
}

/** A champion that can't win: one 1 HP unit. */
function weakChampion(rt: MvpRuntime): Champion {
  const c: Champion = { seq: rt.today().seq, day: rt.today().day, player: botP, line: [{ ...lineUnitOf(rt.content.units[0]!, "c1"), stats: { pwr: 0, hp: 1 } }], since: "2026-10-05T00:00:00.000Z", contentVersion: rt.content.version };
  rt.store.putChampion(c);
  return c;
}

/** Plays `player` from round 12 through the Crown with `line`; returns the Crown battle id. */
function slay(rt: MvpRuntime, player: PlayerRef, line: LineUnit[]): string {
  const run = startRun(rt, player);
  const at12: MvpRunState = { ...run, round: rt.rules.rounds, line, nextUid: line.length + 1 };
  rt.store.putRun(at12);
  decide(rt, at12, { kind: "fight" });
  const won = decide(rt, rt.store.run(run.runId)!, { kind: "fight" });
  expect(won.run.endedBy).toBe("crown-won");
  return won.fight!.battleId;
}

describe("MVP day", () => {
  it("keeps a slayer's Crown hidden until the day ends, then crowns the playoff winner", async () => {
    const { rt, call, human } = world();
    weakChampion(rt);
    const ann = human("ann");
    const eve = human("eve");
    const crown = slay(rt, ann, bigLine(rt.content));

    expect((await call<DayView>("GET", "/day")).json).toMatchObject({ seq: 1, slayers: 1, champion: { player: botP } });
    expect((await call("GET", `/battles/${crown}`, eve)).status).toBe(404);
    expect((await call("GET", `/battles/${crown}`)).status).toBe(404);
    expect((await call<BattleRecord>("GET", `/battles/${crown}`, ann)).json.kind).toBe("crown");

    const { json: day } = await call<DayView>("POST", "/dev/end-day");
    expect(day).toMatchObject({ seq: 2, slayers: 0, champion: { seq: 2, player: ann }, lastPlayoff: { seq: 1, entrants: [ann], winner: ann, games: [] } });
    expect(day.champion!.line.map((u) => u.unitId)).toEqual(bigLine(rt.content).map((u) => u.unitId));
    expect(rt.store.rating(ann.id)).toMatchObject({ slays: 1, playoffWins: 1, daysAsChampion: 1 });
    expect((await call("GET", `/battles/${crown}`, eve)).status).toBe(200);
    expect((await call<DayView>("GET", "/day")).json.seq).toBe(2);
  });

  it("plays two slayers' strongest teams against each other; the winner takes the throne", async () => {
    const { rt, call, human } = world();
    weakChampion(rt);
    const ann = human("ann");
    const bob = human("bob");
    slay(rt, ann, bigLine(rt.content, 1));
    slay(rt, bob, bigLine(rt.content, 5));
    slay(rt, ann, bigLine(rt.content, 2)); // ann's stronger team
    expect((await call<DayView>("GET", "/day")).json.slayers).toBe(2);

    const { json: day } = await call<DayView>("POST", "/dev/end-day");
    const p = day.lastPlayoff!;
    expect(p.entrants.map((e) => e.id)).toEqual([ann.id, bob.id]);
    expect(p.games).toHaveLength(2);
    expect(p.winner?.id).toBe(bob.id);
    expect(day.champion).toMatchObject({ seq: 2, player: bob });
    const g = await call<BattleRecord>("GET", `/battles/${p.games[0]!.battleId}`);
    expect(g.json).toMatchObject({ kind: "playoff", runId: null, round: 0, player: ann, opponent: bob });
    expect(g.json.teamA).toHaveLength(2); // ann's strongest, not her first
    expect(rt.store.rating(bob.id)).toMatchObject({ playoffWins: 1, daysAsChampion: 1 });
    expect(rt.store.rating(ann.id)).toMatchObject({ playoffWins: 0, daysAsChampion: 0 });
  });

  it("a day with no slayers keeps the champion", async () => {
    const { rt, call } = world();
    const champ = weakChampion(rt);
    const { json: day } = await call<DayView>("POST", "/dev/end-day");
    expect(day).toMatchObject({ seq: 2, champion: { ...champ, seq: 2 }, lastPlayoff: { seq: 1, entrants: [], winner: null } });
    expect(rt.store.champions().map((c) => c.seq)).toEqual([1, 2]);
    expect(rt.store.rating(botP.id)).toBeUndefined();
  });

  it("a human champion kept for another day counts it", () => {
    const { rt, human } = world();
    weakChampion(rt);
    const ann = human("ann");
    slay(rt, ann, bigLine(rt.content));
    endDay(rt);
    endDay(rt);
    expect(rt.store.currentChampion()).toMatchObject({ seq: 3, player: ann });
    expect(rt.store.rating(ann.id)).toMatchObject({ daysAsChampion: 2, playoffWins: 1 });
  });

  it("rolls over at 04:00 Moscow by itself, and the dev end-day starts a day ending at the same 04:00", () => {
    const { rt, setClock } = world();
    expect(rt.today()).toMatchObject({ seq: 1, day: "2026-10-05", endsAt: "2026-10-06T01:00:00.000Z" });
    setClock("2026-10-05T22:00:00.000Z");
    endDay(rt);
    expect(rt.today()).toMatchObject({ seq: 2, day: "2026-10-05", startedAt: "2026-10-05T22:00:00.000Z", endsAt: "2026-10-06T01:00:00.000Z" });
    setClock("2026-10-06T00:59:59.999Z");
    expect(rt.today().seq).toBe(2);
    setClock("2026-10-06T01:00:00.000Z");
    expect(rt.today()).toMatchObject({ seq: 3, day: "2026-10-06", endsAt: "2026-10-07T01:00:00.000Z" });
    expect(rt.store.playoff(2)).toMatchObject({ seq: 2, winner: null });
    // a server down for days rolls over once, to the next 04:00
    setClock("2026-10-09T12:00:00.000Z");
    expect(rt.today()).toMatchObject({ seq: 4, day: "2026-10-09", endsAt: "2026-10-10T01:00:00.000Z" });
  });

  it("the rollover job ends the day on its tick", () => {
    const { rt, setClock } = world();
    const stop = dayRollover(rt);
    expect(rt.store.currentDay()?.seq).toBe(1);
    setClock("2026-10-06T01:00:00.000Z");
    stop();
    dayRollover(rt)();
    expect(rt.store.currentDay()?.seq).toBe(2);
  });

  it("gives a day stored before slice 5 (endsAt = startedAt) its real end instead of rolling it over", () => {
    const { rt } = world();
    rt.store.putDay({ seq: 1, day: "2026-10-05", startedAt: "2026-10-05T09:00:00.000Z", endsAt: "2026-10-05T09:00:00.000Z" });
    expect(rt.today()).toMatchObject({ seq: 1, endsAt: "2026-10-06T01:00:00.000Z" });
  });

  it("the reigning champion beating their own team is no slay: no Slay row, no slay bonus, no playoff entry", async () => {
    const { rt, call, human } = world();
    const ann = human("ann");
    const own = { ...weakChampion(rt), player: ann };
    rt.store.putChampion(own);
    const run = startRun(rt, ann);
    const line = bigLine(rt.content);
    // three lost rounds before, so the share (1 of 4) leaves room for a bonus
    const lost = [1, 2, 3].map((round) => ({ battleId: `l${round}`, kind: "round" as const, round, opponent: { ghostId: `g${round}`, player: botP, round }, outcome: "loss" as const, heartsLost: 1, heartsAfter: 5 - round }));
    const at12: MvpRunState = { ...run, round: rt.rules.rounds, line, nextUid: line.length + 1, fights: lost, hearts: 2, losses: 3 };
    rt.store.putRun(at12);
    decide(rt, at12, { kind: "fight" });
    const won = decide(rt, rt.store.run(run.runId)!, { kind: "fight" });
    expect(won.fight).toMatchObject({ kind: "crown", outcome: "win", opponent: { player: ann } });
    expect(won.run.endedBy).toBe("crown-won");
    expect(rt.store.slays(own.seq)).toEqual([]);
    expect((await call<DayView>("GET", "/day")).json.slayers).toBe(0);
    // rated on the rounds alone: the same run against someone else's team gets the bonus
    expect(won.run.rating).toEqual(ratingChange(1000, { fights: won.run.fights, endedBy: "crown-won", player: ann }));
    expect(won.run.rating!.actual).toBe(ratingChange(1000, { fights: won.run.fights, endedBy: "crown-lost" }).actual);
    expect(ratingChange(1000, { fights: won.run.fights, endedBy: "crown-won" }).actual).toBeGreaterThan(won.run.rating!.actual);
    expect(rt.store.rating(ann.id)).toMatchObject({ runs: 1, slays: 0 });
    // a stray Slay row of the champion's own still doesn't enter the playoff
    rt.store.addSlay({ seq: own.seq, player: ann, runId: run.runId, battleId: won.fight!.battleId, line, contentVersion: rt.content.version, at: "2026-10-05T10:00:00.000Z" });
    const { json: day } = await call<DayView>("POST", "/dev/end-day");
    expect(day.lastPlayoff).toMatchObject({ entrants: [], winner: null });
    expect(day.champion).toMatchObject({ seq: 2, player: ann, line: own.line });
    expect(rt.store.rating(ann.id)).toMatchObject({ slays: 0, playoffWins: 0, daysAsChampion: 1 });
  });

  it("a day end that throws doesn't fail requests: the day is served, retried a minute later, and the records move once", async () => {
    const store = new FlakyDayStore(2, 2);
    const { rt, call, human, setClock } = world({ store });
    weakChampion(rt);
    const ann = human("ann");
    slay(rt, ann, bigLine(rt.content));
    const log = vi.spyOn(console, "error").mockImplementation(() => {});
    try {
      setClock("2026-10-06T01:00:00.000Z"); // the 04:00 rollover
      const day = await call<DayView>("GET", "/day");
      expect(day.status).toBe(200);
      expect(day.json).toMatchObject({ seq: 1, slayers: 1 });
      expect(store.tries).toBe(1);
      expect(log).toHaveBeenCalledTimes(1);
      // within the minute: no new attempt, every request still served
      setClock("2026-10-06T01:00:59.000Z");
      expect((await call("GET", "/home")).status).toBe(200);
      expect(rt.today().seq).toBe(1);
      expect(store.tries).toBe(1);
      // a minute later it tries again, fails again, and waits again
      setClock("2026-10-06T01:01:00.000Z");
      expect(rt.today().seq).toBe(1);
      expect(store.tries).toBe(2);
      setClock("2026-10-06T01:01:30.000Z");
      expect(rt.today().seq).toBe(1);
      expect(store.tries).toBe(2);
      // the next try works: one playoff, one champion, the records once
      setClock(new Date(Date.parse("2026-10-06T01:01:00.000Z") + END_DAY_RETRY_MS).toISOString());
      expect(rt.today()).toMatchObject({ seq: 2, day: "2026-10-06" });
      expect(store.tries).toBe(3);
      expect(store.currentChampion()).toMatchObject({ seq: 2, player: ann });
      expect(store.playoff(1)?.winner?.id).toBe(ann.id);
      expect(store.rating(ann.id)).toMatchObject({ slays: 1, playoffWins: 1, daysAsChampion: 1 });
      expect(log).toHaveBeenCalledTimes(2);
    } finally {
      log.mockRestore();
    }
  });

  it("an end-day that failed after crowning, run again, plays the same playoff against the old champion", () => {
    const store = new FlakyDayStore(2, 1);
    const { rt, human } = world({ store });
    const old = weakChampion(rt);
    const ann = human("ann");
    slay(rt, ann, bigLine(rt.content));
    expect(() => endDay(rt)).toThrow("disk full");
    // the failed attempt stored ann as day 2's champion, but day 1 is still on
    expect(store.currentChampion()).toMatchObject({ seq: 2, player: ann });
    expect(store.rating(ann.id)).toMatchObject({ playoffWins: 0, daysAsChampion: 0 });
    endDay(rt);
    expect(store.currentDay()?.seq).toBe(2);
    expect(store.champions().map((c) => [c.seq, c.player.id])).toEqual([[old.seq, botP.id], [2, ann.id]]);
    expect(store.rating(ann.id)).toMatchObject({ slays: 1, playoffWins: 1, daysAsChampion: 1 });
  });

  it("works on the SQLite store", () => {
    const store = new SqliteMvpStore(":memory:");
    const { rt, human } = world({ store });
    weakChampion(rt);
    const ann = human("ann");
    slay(rt, ann, bigLine(rt.content));
    endDay(rt);
    expect(store.currentChampion()).toMatchObject({ seq: 2, player: ann });
    expect(store.playoff(1)?.winner?.id).toBe(ann.id);
    expect(store.rating(ann.id) as Rating).toMatchObject({ playoffWins: 1, daysAsChampion: 1 });
    store.close();
  });
});
