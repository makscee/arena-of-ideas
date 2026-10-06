// Slice 6 (mission #574): the day-1 champion, bots keeping every round's
// ghost pool full, the pool staying bounded, and the job that does both.
import { describe, expect, it, vi } from "vitest";
import { offersAt, type Champion } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { BOT_DAILY_CROWNS, BOT_DAILY_SLAYERS, BOT_MAX_DAILY_CROWNS, BOT_TARGET, botDecision, botPlayer, giftDecision, botWorld, crownsOwed, playBotRun, seedChampion, takenBotNames, thinRounds, topUpGhosts } from "./bots.js";
import { endDay } from "./day.js";
import { mvpContent } from "./content.js";
import { mvpRuntime, type MvpDeps } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";

function world(deps: Partial<MvpDeps> = {}) {
  let s = 7;
  return mvpRuntime({ content: mvpContent(), seed: () => (s = (s * 1103515245 + 12345) >>> 0), ...deps });
}
const pool = (rt: ReturnType<typeof world>, round: number) => rt.store.ghosts(round, { excludePlayerId: "", contentVersion: rt.content.version });

  // Champion seeding simulates whole bot runs; fused stalemates (a team Blessing re-armed every turn) make it slow under load.
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
  }, 30_000);

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
  }, 30_000);

  it("fills every round's ghost pool with bot runs, and stops there", async () => {
    for (const store of [undefined, new SqliteMvpStore(":memory:")]) {
      const rt = world(store ? { store } : {});
      await seedChampion(rt);
      expect(thinRounds(rt)).toHaveLength(rt.rules.rounds);
      // A smaller target keeps the test quick; the job uses BOT_TARGET.
      const target = 8;
      const first = topUpGhosts(rt, { target, dailyCrowns: 0 });
      expect(first.thin).toEqual([]);
      expect(first.runs).toBeGreaterThanOrEqual(target); // more when some runs end early
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
      // Past the quota, bots play on until 2 different bots slew (or the cap),
      // so a player alone sees a real playoff at the day's end.
      const slayers = new Set(rt.store.slays(rt.today().seq).filter((s) => s.player.bot).map((s) => s.player.id)).size;
      expect(slayers >= BOT_DAILY_SLAYERS || botCrowns() >= BOT_MAX_DAILY_CROWNS).toBe(true);
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
  }, 30_000);

  it("a bot run never takes a name today's slayers or any champion has, so playoff entrants never share one (#587)", async () => {
    // The roster name, else the next free one, else a numbered one.
    expect(botPlayer(4).name).toBe("bot-Esk");
    expect(botPlayer(4, new Set(["bot-Esk"])).name).toBe("bot-Fyn");
    const roster = new Set(Array.from({ length: 16 }, (_, i) => botPlayer(i).name));
    expect(roster.size).toBe(16);
    expect(botPlayer(4, roster).name).toBe("bot-Esk-2");
    // In a world: many bot runs against today's champion, every slayer's name is
    // unique and no champion's. Seed 3: a world whose champion bots do slay
    // (seed 7's, since the awakening gift, R3-15, wasn't slain in 400 runs).
    let n = 3;
    const rt = world({ seed: () => (n = (n * 1103515245 + 12345) >>> 0) });
    await seedChampion(rt);
    // Slays are rare (the Crown is hard), so play until a few land, capped.
    for (let i = 0; i < 400 && (i < 80 || rt.store.slays(rt.today().seq).length < 2); i++) playBotRun(rt);
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

  it("the bot locks a copy it can't afford, once (not in the last shop round), and never rerolls a shop full of locks", () => {
    const rt = world();
    const [a, b, c] = rt.content.units;
    const line = [lineUnitOf(a!, "u1", 1, rt.rules)];
    const offers = [{ slot: 0, unitId: b!.id, tier: b!.tier, cost: 3 }, { slot: 1, unitId: a!.id, tier: a!.tier, cost: 3 }];
    const run = (o: typeof offers, gold: number, round = 1) => ({ phase: "shop", round, line, offers: o, gold }) as never;
    expect(botDecision(run(offers, 2), rt.content, rt.rules, 0)).toEqual({ kind: "lock", slot: 1 });
    expect(botDecision(run(offers, 2, rt.rules.rounds), rt.content, rt.rules, 0)).not.toMatchObject({ kind: "lock" });
    const lockedCopy = [offers[0]!, { ...offers[1]!, locked: true }];
    expect(botDecision(run(lockedCopy, 2), rt.content, rt.rules, 0)).not.toMatchObject({ kind: "lock" });
    // Gold for a reroll but not the (pricier) offers: rerolls while the locks leave room.
    const pricey = lockedCopy.map((o) => ({ ...o, cost: 10 }));
    expect(botDecision(run(pricey, 9), rt.content, rt.rules, 0)).toEqual({ kind: "reroll" });
    // Every offer locked but a slot empty (one was bought): the reroll refills it.
    const allLocked = pricey.map((o) => ({ ...o, locked: true }));
    expect(botDecision(run(allLocked, 9), rt.content, rt.rules, 0)).toEqual({ kind: "reroll" });
    // Locks filling the whole shop: a reroll would redraw nothing.
    const full = [...allLocked, { slot: 2, unitId: c!.id, tier: c!.tier, cost: 10, locked: true }];
    expect(full).toHaveLength(offersAt(rt.rules, 1));
    expect(botDecision(run(full, 9), rt.content, rt.rules, 0)).toEqual({ kind: "fight" });
  });

  it("the bot uses the bench (R3-13): keeps a better tier there, fields the best 5, fuses from it", () => {
    const rt = world();
    const t1 = rt.content.units.filter((u) => u.tier === 1);
    const t3 = rt.content.units.find((u) => u.tier === 3)!;
    const unit = (u: (typeof t1)[number], uid: string, copies = 1) => lineUnitOf(u, uid, copies, rt.rules);
    const line = t1.slice(0, 5).map((u, i) => unit(u, `l${i}`));
    const offers = [{ slot: 0, unitId: t3.id, tier: 3, cost: 3 }];
    const run = (bench: ReturnType<typeof unit>[], l = line, o = offers, gold = 3) => ({ phase: "shop", round: 9, line: l, bench, offers: o, gold }) as never;
    // Line full, a higher tier on offer, the bench has room: buy it (onto the bench).
    expect(botDecision(run([]), rt.content, rt.rules, 0)).toEqual({ kind: "buy", slot: 0 });
    // Bench full of singles: sell the weakest single instead.
    const bench = t1.slice(5, 8).map((u, i) => unit(u, `b${i}`));
    expect(botDecision(run(bench), rt.content, rt.rules, 0)).toMatchObject({ kind: "sell" });
    // A copy of a bench unit is a copy: buy it.
    expect(botDecision(run(bench, line, [{ slot: 0, unitId: bench[1]!.unitId, tier: 1, cost: 3 }]), rt.content, rt.rules, 0)).toEqual({ kind: "buy", slot: 0 });
    // Nothing to buy: a stronger bench unit comes in for the weakest line unit.
    const strong = unit(t1[5]!, "b0", 2);
    const d = botDecision(run([strong], line, [], 0), rt.content, rt.rules, 0);
    expect(d).toMatchObject({ kind: "reorder", from: rt.rules.lineSize });
    // A line with room takes the bench unit into its empty slot.
    expect(botDecision(run([strong], line.slice(0, 4), [], 0), rt.content, rt.rules, 0)).toEqual({ kind: "reorder", from: 5, to: 4 });
    // Two Awoken units on the bench fuse, by board slot.
    expect(botDecision(run([unit(t1[5]!, "b0", 3), unit(t1[6]!, "b1", 3)], line, [], 0), rt.content, rt.rules, 0)).toMatchObject({ kind: "fuse", first: expect.any(Number), second: expect.any(Number) });
  });

  it("bot runs use the bench", async () => {
    const rt = world();
    await seedChampion(rt);
    let benched = 0;
    for (let i = 0; i < 20; i++) {
      const run = playBotRun(rt);
      if (run.bench.length > 0) benched++;
    }
    expect(benched).toBeGreaterThan(0);
  }, 30_000);

  it("the bot takes a waiting gift first (R3-15): a copy it has, else the best fresh unit with room, else it skips", () => {
    const rt = world();
    const t1 = rt.content.units.filter((u) => u.tier === 1);
    const unit = (u: (typeof t1)[number], uid: string, copies = 1) => lineUnitOf(u, uid, copies, rt.rules);
    const run = (gift: string[], line: ReturnType<typeof unit>[], bench: ReturnType<typeof unit>[] = []) => ({ phase: "shop", round: 1, line, bench, gift, offers: [], gold: 10 }) as never;
    const line = t1.slice(0, 5).map((u, i) => unit(u, `l${i}`, 3));
    const bench = t1.slice(5, 8).map((u, i) => unit(u, `b${i}`));
    const fresh = t1.slice(8, 11).map((u) => u.id);
    // Even with two Awoken units to fuse and gold to spend, the gift comes first.
    expect(botDecision(run(fresh, line.slice(0, 2)), rt.content, rt.rules, 0)).toMatchObject({ kind: "gift" });
    // A copy of a bench unit merges, so it is taken with line and bench full.
    expect(botDecision(run([fresh[0]!, t1[6]!.id, fresh[1]!], line, bench), rt.content, rt.rules, 0)).toEqual({ kind: "gift", pick: 1 });
    // No copy, no room: skip.
    expect(botDecision(run(fresh, line, bench), rt.content, rt.rules, 0)).toEqual({ kind: "gift", pick: null });
    // No copy, room: the best by the bot's score.
    const score = (id: string) => {
      const u = unit(rt.content.units.find((x) => x.id === id)!, "s");
      return u.stats.pwr * 2 + u.stats.hp;
    };
    const best = fresh.map(score).indexOf(Math.max(...fresh.map(score)));
    expect(giftDecision(fresh, line, [], rt.content, rt.rules)).toEqual({ kind: "gift", pick: best });
  });

  it("bot runs pick gifts, and every gift decision is one the rules accept", async () => {
    const rt = world();
    await seedChampion(rt);
    let gifts = 0;
    let picked = 0;
    rt.hooks.push({ onDecision: (before, d) => void (d.kind === "gift" && (gifts++, d.pick !== null && picked++, expect(before.gift).toBeDefined())) });
    for (let i = 0; i < 20; i++) expect(playBotRun(rt).gift).toBeUndefined();
    expect(gifts).toBeGreaterThan(0);
    expect(picked).toBeGreaterThan(0);
  }, 30_000);

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
