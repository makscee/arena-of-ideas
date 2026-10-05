// Slice 6 (mission #574): the day-1 champion, bots keeping every round's
// ghost pool full, the pool staying bounded, and the job that does both.
import { describe, expect, it, vi } from "vitest";
import type { Champion } from "../../../src/mvp/contract.js";
import { BOT_TARGET, botDecision, botWorld, playBotRun, seedChampion, thinRounds, topUpGhosts } from "./bots.js";
import { mvpContent } from "./content.js";
import { mvpRuntime, type MvpDeps } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";

function world(deps: Partial<MvpDeps> = {}) {
  let s = 7;
  return mvpRuntime({ content: mvpContent(), seed: () => (s = (s * 1103515245 + 12345) >>> 0), ...deps });
}
const pool = (rt: ReturnType<typeof world>, round: number) => rt.store.ghosts(round, { excludePlayerId: "", contentVersion: rt.content.version });

describe("MVP bots and world (slice 6)", () => {
  it("seeds a strong bot team as today's champion on a fresh store, once", () => {
    const rt = world();
    const champ = seedChampion(rt)!;
    expect(champ).toMatchObject({ seq: rt.today().seq, day: rt.today().day, contentVersion: rt.content.version, player: { bot: true } });
    expect(champ.line.length).toBe(rt.rules.lineSize);
    expect(rt.store.currentChampion()).toEqual(champ);
    // The search left nothing behind: no ghosts, runs or battles on the real store.
    expect(pool(rt, 1)).toEqual([]);
    expect(rt.store.battles()).toEqual([]);
    expect(seedChampion(rt)).toBeUndefined();
  });

  it("replaces a champion built with other content, for today's seq", () => {
    const rt = world();
    const stale: Champion = { seq: rt.today().seq, day: rt.today().day, player: { id: "b", name: "old", bot: true }, line: [], since: "t", contentVersion: "old" };
    rt.store.putChampion(stale);
    const champ = seedChampion(rt)!;
    expect(champ.contentVersion).toBe(rt.content.version);
    expect(rt.store.champions()).toHaveLength(1);
  });

  it("fills every round's ghost pool with bot runs, and stops there", () => {
    for (const store of [undefined, new SqliteMvpStore(":memory:")]) {
      const rt = world(store ? { store } : {});
      seedChampion(rt);
      expect(thinRounds(rt)).toHaveLength(rt.rules.rounds);
      // A smaller target keeps the test quick; the job uses BOT_TARGET.
      const target = 8;
      const first = topUpGhosts(rt, { target });
      expect(first.thin).toEqual([]);
      expect(first.runs).toBeGreaterThan(target);
      for (let r = 1; r <= rt.rules.rounds; r++) {
        const ghosts = pool(rt, r);
        expect(ghosts.length).toBeGreaterThanOrEqual(target);
        expect(ghosts.every((g) => g.player.bot)).toBe(true);
      }
      // Bounded: a full pool plays no more runs.
      expect(topUpGhosts(rt, { target }).runs).toBe(0);
      // Bots stop before the Crown: no Crown battles, no slays, no ratings.
      expect(rt.store.battles({ kind: "crown" })).toEqual([]);
      expect(rt.store.slays(rt.today().seq)).toEqual([]);
      // Bots fuse, and never claim the credit.
      expect(rt.store.fusions().length).toBeGreaterThan(0);
      expect(rt.store.fusions().every((f) => f.discoveredBy === null)).toBe(true);
    }
  }, 30_000);

  it("refills for new content: ghosts of other content don't count", () => {
    const rt = world();
    topUpGhosts(rt, { target: 2 });
    const retuned = world({ store: rt.store, content: { ...rt.content, version: "retuned" } });
    expect(thinRounds(retuned)).toHaveLength(rt.rules.rounds);
  });

  it("the bot only makes decisions the rules accept, and fights when nothing else is worth doing", () => {
    const rt = world();
    // playBotRun goes through decide(), which throws on a refused decision.
    for (let i = 0; i < 20; i++) expect(["shop", "crown", "over"]).toContain(playBotRun(rt).phase);
    const d = botDecision({ phase: "shop", line: [], offers: [], gold: 0 } as never, rt.content, rt.rules, 0);
    expect(d).toEqual({ kind: "fight" });
    expect(botDecision({ phase: "crown" } as never, rt.content, rt.rules, 0)).toEqual({ kind: "fight" });
  });

  it("botWorld seeds the champion at start and tops up in the background until stopped", async () => {
    const rt = world();
    const log = vi.spyOn(console, "log").mockImplementation(() => {});
    const stop = botWorld(rt);
    try {
      expect(rt.store.currentChampion()?.contentVersion).toBe(rt.content.version);
      await vi.waitFor(() => expect(thinRounds(rt)).toEqual([]), { timeout: 20_000, interval: 50 });
      expect(pool(rt, rt.rules.rounds).length).toBeGreaterThanOrEqual(BOT_TARGET);
    } finally {
      stop();
      log.mockRestore();
    }
  }, 30_000);
});
