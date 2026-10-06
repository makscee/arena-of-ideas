// The MvpStore contract as tests (mission #574): every store class runs it,
// the memory store now (store.test.ts) and slice 4's SQLite store. A slice
// that adds a method to MvpStore adds its case here. Compare with toEqual:
// a store may hand back copies.
import { describe, expect, it } from "vitest";
import type { BattleRecord, Champion, DayState, FusionDiscovery, Ghost, MvpContent, PlayerRef, PlayoffResult, Rating, Slay } from "../../../src/mvp/contract.js";
import { initMvpRun } from "../../../src/mvp/run.js";
import type { MvpStore } from "./store.js";

const bot: PlayerRef = { id: "bot", name: "bot-Ash", bot: true };
const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };
const eva: PlayerRef = { id: "p2", name: "Eva", bot: false };
const form = { when: [], who: [], does: ["Strike"] };
const content: MvpContent = {
  version: "v2",
  units: [{ id: "brawler", name: "Brawler", emoji: "🥊", tier: 1, base: { pwr: 2, hp: 5 }, forms: { sleeping: form, awoken: form } }],
  abilities: {},
  statuses: {},
};
const run = (runId: string, player: PlayerRef) => initMvpRun({ runId, player, seed: 1, content, day: 1, startedAt: "2026-10-05T10:00:00.000Z" });
const champion = (seq: number, player: PlayerRef): Champion => ({ seq, day: "2026-10-05", player, line: [], since: "t", contentVersion: "v2" });
const battle = (battleId: string, kind: BattleRecord["kind"], at: string): BattleRecord => ({
  battleId,
  runId: kind === "playoff" ? null : "r1",
  player: maks,
  seed: 1,
  contentVersion: "v2",
  kind,
  round: kind === "playoff" ? 0 : 1,
  at,
  teamA: [],
  teamB: [],
  opponent: bot,
  winner: "draw",
  log: [],
});

export function describeMvpStore(name: string, make: () => MvpStore): void {
  describe(`MvpStore contract: ${name}`, () => {
    it("keeps players and runs; the active run is the one not over", () => {
      const store = make();
      store.addPlayer(maks);
      expect(store.player("p1")).toEqual(maks);
      expect(store.player("nobody")).toBeUndefined();
      // R2-17: a name is found whatever its case.
      expect(store.playerByName("MAKS")).toEqual(maks);
      expect(store.playerByName("maks2")).toBeUndefined();
      const r = run("r1", maks);
      store.putRun(r);
      expect(store.run("r1")).toEqual(r);
      expect(store.activeRun("p1")?.runId).toBe("r1");
      store.putRun({ ...r, phase: "over", endedBy: "out-of-hearts" });
      expect(store.run("r1")?.phase).toBe("over");
      expect(store.activeRun("p1")).toBeUndefined();
    });

    it("serves ghosts at the round, never the player's own, only of the current content", () => {
      const store = make();
      const ghost = (ghostId: string, player: PlayerRef, round: number, contentVersion: string): Ghost => ({ ghostId, runId: `run-${ghostId}`, player, round, line: [], contentVersion, createdAt: "t", rating: 1000 });
      store.addGhost(ghost("mine", maks, 1, "v2"));
      store.addGhost(ghost("old", bot, 1, "v1"));
      store.addGhost(ghost("theirs", bot, 1, "v2"));
      store.addGhost(ghost("later", bot, 2, "v2"));
      expect(store.ghosts(1, { excludePlayerId: "p1", contentVersion: "v2" }).map((g) => g.ghostId)).toEqual(["theirs"]);
      expect(store.ghosts(1, { excludePlayerId: "bot", contentVersion: "v2" }).map((g) => g.ghostId)).toEqual(["mine"]);
      expect(store.ghosts(3, { excludePlayerId: "p1", contentVersion: "v2" })).toEqual([]);
    });

    it("keeps battles and lists them oldest first, by kind and time", () => {
      const store = make();
      store.putBattle(battle("b2", "round", "2026-10-05T10:02:00.000Z"));
      store.putBattle(battle("b1", "round", "2026-10-05T10:01:00.000Z"));
      store.putBattle(battle("p1", "playoff", "2026-10-05T10:03:00.000Z"));
      expect(store.battle("b1")).toEqual(battle("b1", "round", "2026-10-05T10:01:00.000Z"));
      expect(store.battle("nope")).toBeUndefined();
      expect(store.battles().map((b) => b.battleId)).toEqual(["b1", "b2", "p1"]);
      expect(store.battles({ kind: "round" }).map((b) => b.battleId)).toEqual(["b1", "b2"]);
      expect(store.battles({ since: "2026-10-05T10:02:00.000Z" }).map((b) => b.battleId)).toEqual(["b2", "p1"]);
      expect(store.battles({ kind: "crown" })).toEqual([]);
    });

    it("keeps one discovery per ordered pair", () => {
      const store = make();
      const f: FusionDiscovery = { first: "brawler", second: "medic", name: "Brawdic", discoveredBy: null, discoveredAt: "t", nameSource: "fallback" };
      store.putFusion(f);
      expect(store.fusion("brawler", "medic")).toEqual(f);
      expect(store.fusion("medic", "brawler")).toBeUndefined();
      store.putFusion({ ...f, discoveredBy: maks });
      expect(store.fusions()).toEqual([{ ...f, discoveredBy: maks }]);
    });

    it("keeps the day by seq; the current day is the highest", () => {
      const store = make();
      expect(store.currentDay()).toBeUndefined();
      const d1: DayState = { seq: 1, day: "2026-10-05", startedAt: "2026-10-05T10:00:00.000Z", endsAt: "2026-10-06T01:00:00.000Z" };
      store.putDay(d1);
      store.putDay({ ...d1, seq: 2, startedAt: "2026-10-05T11:00:00.000Z" });
      expect(store.currentDay()?.seq).toBe(2);
      store.putDay({ ...d1, endsAt: "x" });
      expect(store.currentDay()?.seq).toBe(2);
    });

    it("keys champions by seq, so several days can share a date", () => {
      const store = make();
      expect(store.currentChampion()).toBeUndefined();
      store.putChampion(champion(2, maks));
      store.putChampion(champion(1, bot));
      expect(store.champion(1)?.player).toEqual(bot);
      expect(store.currentChampion()?.player).toEqual(maks);
      expect(store.champions().map((c) => c.seq)).toEqual([1, 2]);
    });

    it("keeps slays and playoffs per day, ratings per player", () => {
      const store = make();
      const slay: Slay = { seq: 1, player: maks, runId: "r", battleId: "b", line: [], contentVersion: "v2", at: "t" };
      store.addSlay(slay);
      store.addSlay({ ...slay, runId: "r2" });
      expect(store.slays(1)).toHaveLength(2);
      expect(store.slays(2)).toEqual([]);
      const p: PlayoffResult = {
        seq: 1,
        day: "2026-10-05",
        entrants: [maks, eva],
        winner: maks,
        battleIds: ["p1"],
        standings: [{ player: maks, wins: 1, draws: 0, losses: 0 }, { player: eva, wins: 0, draws: 0, losses: 1 }],
        games: [{ a: maks, b: eva, battleId: "p1", winner: "A" }],
      };
      store.putPlayoff(p);
      expect(store.playoff(1)).toEqual(p);
      expect(store.playoff(2)).toBeUndefined();
      const r: Rating = { player: maks, rating: 1016, runs: 1, slays: 1, daysAsChampion: 0, playoffWins: 0 };
      store.putRating(r);
      expect(store.rating("p1")).toEqual(r);
      expect(store.rating("p2")).toBeUndefined();
    });
    it("adds unit tallies per content version", () => {
      const store = make();
      expect(store.unitTallies("v2")).toEqual({ runs: 0, units: [] });
      store.addUnitTallies("v2", { runs: 0, units: [{ unitId: "brawler", fights: 1, wins: 1, runs: 0 }, { unitId: "archer", fights: 1, wins: 0, runs: 0 }] });
      store.addUnitTallies("v2", { runs: 1, units: [{ unitId: "brawler", fights: 1, wins: 0, runs: 1 }] });
      store.addUnitTallies("v3", { runs: 1, units: [{ unitId: "brawler", fights: 2, wins: 2, runs: 1 }] });
      const v2 = store.unitTallies("v2");
      expect(v2.runs).toBe(1);
      expect([...v2.units].sort((a, b) => a.unitId.localeCompare(b.unitId))).toEqual([
        { unitId: "archer", fights: 1, wins: 0, runs: 0 },
        { unitId: "brawler", fights: 2, wins: 1, runs: 1 },
      ]);
      expect(store.unitTallies("v3")).toEqual({ runs: 1, units: [{ unitId: "brawler", fights: 2, wins: 2, runs: 1 }] });
    });
  });
}
