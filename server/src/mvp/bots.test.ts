// Slice 6 (mission #574): the day-1 champion, bots keeping every round's
// ghost pool full, the pool staying bounded, and the job that does both.
import { describe, expect, it, vi } from "vitest";
import type { Champion } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { BOT_DAILY_CROWNS, BOT_TARGET, botDecision, botPlayer, botWorld, crownsOwed, playBotRun, seedChampion, takenBotNames, thinRounds, topUpGhosts } from "./bots.js";
import { endDay } from "./day.js";
import { mvpContent } from "./content.js";
import { mvpRuntime, type MvpDeps } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";

function world(deps: Partial<MvpDeps> = {}) {
  let s = 7;
  return mvpRuntime({ content: mvpContent(), seed: () => (s = (s * 1103515245 + 12345) >>> 0), ...deps });
}
const pool = (rt: ReturnType<typeof world>, round: number) => rt.store.ghosts(round, { excludePlayerId: "", contentVersion: rt.content.version });

describe("MVP bots and world (slice 6)", () => {
  it("seeds a strong bot team as today's champion on a fresh store, once", async () => {
    const rt = world();
    const champ = (await seedChampion(rt))!;
    expect(champ).toMatchObject({ seq: rt.today().seq, day: rt.today().day, contentVersion: rt.content.version, player: { bot: true } });
    expect(champ.line.length).toBe(rt.rules.lineSize);
    expect(rt.store.currentChampion()).toEqual(champ);
    // The search left nothing behind: no ghosts, runs or battles on the real store.
    expect(pool(rt, 1)).toEqual([]);
    expect(rt.store.battles()).toEqual([]);
    expect(await seedChampion(rt)).toBeUndefined();
  });

  it("stores the champion's fusions as bot discoveries, so a later fuse of the pair shows the same name", async () => {
    for (let seed = 1; seed <= 6; seed++) {
      let s = seed;
      const rt = world({ seed: () => (s = (s * 1103515245 + 12345) >>> 0) });
      const champ = (await seedChampion(rt))!;
      const fused = champ.line.filter((u) => u.fusion);
      for (const u of fused) {
        expect(rt.store.fusion(u.fusion!.first, u.fusion!.second)).toMatchObject({ name: u.name, discoveredBy: null });
        expect(u.fusion).toMatchObject({ name: u.name, discoveredBy: null });
      }
      if (fused.length > 0) return;
    }
    throw new Error("no seeded champion held a fusion");
  }, 30_000);

  it("replaces a champion built with other content, for today's seq", async () => {
    const rt = world();
    const stale: Champion = { seq: rt.today().seq, day: rt.today().day, player: { id: "b", name: "old", bot: true }, line: [], since: "t", contentVersion: "old" };
    rt.store.putChampion(stale);
    const champ = (await seedChampion(rt))!;
    expect(champ.contentVersion).toBe(rt.content.version);
    expect(rt.store.champions()).toHaveLength(1);
  });

  it("fills every round's ghost pool with bot runs, and stops there", async () => {
    for (const store of [undefined, new SqliteMvpStore(":memory:")]) {
      const rt = world(store ? { store } : {});
      await seedChampion(rt);
      expect(thinRounds(rt)).toHaveLength(rt.rules.rounds);
      // A smaller target keeps the test quick; the job uses BOT_TARGET.
      const target = 8;
      const first = topUpGhosts(rt, { target, dailyCrowns: 0 });
      expect(first.thin).toEqual([]);
      expect(first.runs).toBeGreaterThan(target);
      for (let r = 1; r <= rt.rules.rounds; r++) {
        const ghosts = pool(rt, r);
        expect(ghosts.length).toBeGreaterThanOrEqual(target);
        expect(ghosts.every((g) => g.player.bot)).toBe(true);
      }
      // Bounded: a full pool plays no more runs.
      expect(topUpGhosts(rt, { target, dailyCrowns: 0 }).runs).toBe(0);
      // Bots play whole runs: they fight the Crown, and a win is a slay (#587).
      const crowns = rt.store.battles({ kind: "crown" });
      expect(crowns.length).toBeGreaterThan(0);
      expect(rt.store.slays(rt.today().seq).map((x) => x.battleId)).toEqual(crowns.filter((b) => b.winner === "A").map((b) => b.battleId));
      // Bots fuse, and never claim the credit.
      expect(rt.store.fusions().length).toBeGreaterThan(0);
      expect(rt.store.fusions().every((f) => f.discoveredBy === null)).toBe(true);
    }
  }, 30_000);

  it("bots fight the day's Crown quota every day, not only on a fresh world's first (#587)", async () => {
    const rt = world();
    // No live champion: nothing owed, so no endless "no-champion" runs.
    expect(crownsOwed(rt)).toBe(0);
    await seedChampion(rt);
    expect(crownsOwed(rt)).toBe(BOT_DAILY_CROWNS);
    const quota = 6;
    const botCrowns = () => rt.store.battles({ kind: "crown", since: rt.today().startedAt }).filter((b) => b.player.bot).length;
    for (let day = 1; day <= 3; day++) {
      // The pool is full after day 1: only the quota makes bots play.
      const t = topUpGhosts(rt, { target: 4, dailyCrowns: quota });
      expect(t).toMatchObject({ thin: [], crownsOwed: 0 });
      expect(botCrowns()).toBeGreaterThanOrEqual(quota);
      // Met: the next tick plays nothing.
      expect(topUpGhosts(rt, { target: 4, dailyCrowns: quota }).runs).toBe(0);
      endDay(rt);
    }
  }, 30_000);

  it("refills for new content: ghosts of other content don't count", () => {
    const rt = world();
    topUpGhosts(rt, { target: 2 });
    const retuned = world({ store: rt.store, content: { ...rt.content, version: "retuned" } });
    expect(thinRounds(retuned)).toHaveLength(rt.rules.rounds);
  });

  it("the bot only makes decisions the rules accept, and fights when nothing else is worth doing", async () => {
    const rt = world();
    // playBotRun goes through decide(), which throws on a refused decision.
    await seedChampion(rt);
    // Whole runs, the Crown included; no rating for a bot (a slayer's row
    // only counts its slay).
    for (let i = 0; i < 20; i++) {
      const run = playBotRun(rt);
      expect(run.phase).toBe("over");
      expect(run.rating).toBeNull();
      const row = rt.store.rating(run.player.id);
      if (run.endedBy === "crown-won") expect(row).toMatchObject({ rating: rt.rules.ratingStart, runs: 0, slays: 1 });
      else expect(row).toBeUndefined();
    }
    const d = botDecision({ phase: "shop", line: [], offers: [], gold: 0 } as never, rt.content, rt.rules, 0);
    expect(d).toEqual({ kind: "fight" });
    expect(botDecision({ phase: "crown" } as never, rt.content, rt.rules, 0)).toEqual({ kind: "fight" });
  });

  it("a bot run never takes a name today's slayers or any champion has, so playoff entrants never share one (#587)", async () => {
    // The roster name, else the next free one, else a numbered one.
    expect(botPlayer(4).name).toBe("bot-Esk");
    expect(botPlayer(4, new Set(["bot-Esk"])).name).toBe("bot-Fyn");
    const roster = new Set(Array.from({ length: 16 }, (_, i) => botPlayer(i).name));
    expect(roster.size).toBe(16);
    expect(botPlayer(4, roster).name).toBe("bot-Esk-2");
    // In a world: many bot runs against today's champion, every slayer's name is
    // unique and no champion's.
    const rt = world();
    await seedChampion(rt);
    for (let i = 0; i < 80; i++) playBotRun(rt);
    const slayers = new Map<string, string>();
    for (const s of rt.store.slays(rt.today().seq)) {
      expect(slayers.get(s.player.name) ?? s.player.id).toBe(s.player.id);
      slayers.set(s.player.name, s.player.id);
    }
    expect(slayers.size).toBeGreaterThan(0);
    const champ = rt.store.currentChampion()!;
    expect(slayers.has(champ.player.name)).toBe(false);
    expect(takenBotNames(rt)).toEqual(new Set([...slayers.keys(), champ.player.name]));
  }, 30_000);

  it("the bot fuses only a pair whose name is ready", () => {
    const rt = world();
    const [a, b] = rt.content.units;
    const line = [lineUnitOf(a!, "u1", 3, rt.rules), lineUnitOf(b!, "u2", 3, rt.rules)];
    const run = { phase: "shop", line, offers: [], gold: 0 } as never;
    expect(botDecision(run, rt.content, rt.rules, 0)).toMatchObject({ kind: "fuse" });
    expect(botDecision(run, rt.content, rt.rules, 0, () => false)).not.toMatchObject({ kind: "fuse" });
    const asked: string[] = [];
    const d = botDecision(run, rt.content, rt.rules, 0, (x, y) => (asked.push(`${x.uid}+${y.uid}`), true));
    expect(asked).toHaveLength(1);
    expect(d).toEqual({ kind: "fuse", first: line.findIndex((u) => asked[0]!.startsWith(u.uid)), second: line.findIndex((u) => asked[0]!.endsWith(u.uid)) });
  });

  it("botWorld seeds the champion at start and tops up in the background until stopped", async () => {
    const rt = world();
    const log = vi.spyOn(console, "log").mockImplementation(() => {});
    const stop = botWorld(rt);
    try {
      await vi.waitFor(() => expect(rt.store.currentChampion()?.contentVersion).toBe(rt.content.version), { timeout: 10_000, interval: 20 });
      await vi.waitFor(() => expect(thinRounds(rt)).toEqual([]), { timeout: 20_000, interval: 50 });
      expect(pool(rt, rt.rules.rounds).length).toBeGreaterThanOrEqual(BOT_TARGET);
    } finally {
      stop();
      log.mockRestore();
    }
  }, 30_000);
});
