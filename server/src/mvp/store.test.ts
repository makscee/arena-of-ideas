import { describe, expect, it } from "vitest";
import type { Champion, PlayerRef, Rating, Slay } from "../../../src/mvp/contract.js";
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
    const slay: Slay = { seq: 1, player: maks, runId: "r", battleId: "b", line: [], at: "t" };
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
