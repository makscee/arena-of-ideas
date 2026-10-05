import { describe, expect, it } from "vitest";
import type { Champion, Ghost, PlayerRef, Rating, Slay } from "../../../src/mvp/contract.js";
import { MemoryMvpStore } from "./store.js";

const bot: PlayerRef = { id: "bot", name: "bot-Ash", bot: true };
const maks: PlayerRef = { id: "p1", name: "Maks", bot: false };
const champion = (seq: number, player: PlayerRef): Champion => ({ seq, day: "2026-10-05", player, line: [], since: "t", contentVersion: "v" });

describe("MVP memory store: the day", () => {
  it("keys champions by seq, so several days can share a date", () => {
    const store = new MemoryMvpStore();
    expect(store.currentChampion()).toBeUndefined();
    store.putChampion(champion(2, maks));
    store.putChampion(champion(1, bot));
    expect(store.champion(1)?.player).toEqual(bot);
    expect(store.currentChampion()?.player).toEqual(maks);
    expect(store.champions().map((c) => c.seq)).toEqual([1, 2]);
  });

  it("keeps slays per day and ratings per player", () => {
    const store = new MemoryMvpStore();
    const slay: Slay = { seq: 1, player: maks, runId: "r", battleId: "b", line: [], contentVersion: "v", at: "t" };
    store.addSlay(slay);
    store.addSlay({ ...slay, runId: "r2" });
    expect(store.slays(1)).toHaveLength(2);
    expect(store.slays(2)).toEqual([]);
    const r: Rating = { player: maks, rating: 1016, runs: 1, slays: 1, daysAsChampion: 0, playoffWins: 0 };
    store.putRating(r);
    expect(store.rating("p1")).toEqual(r);
    expect(store.rating("p2")).toBeUndefined();
  });
});

describe("MVP memory store: matchmaking", () => {
  it("serves ghosts at the round, never the player's own, only of the current content", () => {
    const store = new MemoryMvpStore();
    const ghost = (ghostId: string, player: PlayerRef, round: number, contentVersion: string): Ghost => ({ ghostId, runId: `run-${ghostId}`, player, round, line: [], contentVersion, createdAt: "t" });
    store.addGhost(ghost("mine", maks, 1, "v2"));
    store.addGhost(ghost("old", bot, 1, "v1"));
    store.addGhost(ghost("theirs", bot, 1, "v2"));
    store.addGhost(ghost("later", bot, 2, "v2"));
    expect(store.ghosts(1, { excludePlayerId: "p1", contentVersion: "v2" }).map((g) => g.ghostId)).toEqual(["theirs"]);
    expect(store.ghosts(1, { excludePlayerId: "bot", contentVersion: "v2" }).map((g) => g.ghostId)).toEqual(["mine"]);
    expect(store.ghosts(3, { excludePlayerId: "p1", contentVersion: "v2" })).toEqual([]);
  });
});
