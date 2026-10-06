// Slice 5 (mission #574): the day's pure rules.
import { describe, expect, it } from "vitest";
import { MVP_RULES, type Champion, type LineUnit, type MvpContent, type PlayerRef, type Slay } from "./contract.js";
import { dayLabel, nextRollover, playoffEntrants, playoffSlays, playRoundRobin } from "./day.js";
import { lineUnitOf } from "./forms.js";
import { mvpPool } from "./units.js";

const content: MvpContent = { version: "test", ...mvpPool() };
const p = (id: string, bot = false): PlayerRef => ({ id, name: id, bot });
const big = (n: number): LineUnit[] =>
  [...content.units].sort((a, b) => b.base.hp + b.base.pwr - (a.base.hp + a.base.pwr)).slice(0, n).map((u, i) => lineUnitOf(u, `u${i}`, 3));
const tiny = (): LineUnit[] => [{ ...lineUnitOf(content.units[0]!, "t1"), stats: { pwr: 0, hp: 1 } }];
const slay = (player: PlayerRef, line: LineUnit[], at: string, over: Partial<Slay> = {}): Slay => ({ seq: 1, player, runId: `r-${player.id}-${at}`, battleId: `b-${player.id}-${at}`, line, contentVersion: content.version, at, ...over });
const champion: Champion = { seq: 1, day: "2026-10-05", player: p("bot", true), line: big(3), since: "", contentVersion: content.version };

describe("the day's clock (04:00 Moscow)", () => {
  it("rolls over at the next 04:00 Moscow, 01:00 UTC", () => {
    expect(nextRollover(new Date("2026-10-05T10:00:00Z"), MVP_RULES).toISOString()).toBe("2026-10-06T01:00:00.000Z");
    expect(nextRollover(new Date("2026-10-05T22:30:00Z"), MVP_RULES).toISOString()).toBe("2026-10-06T01:00:00.000Z");
    // exactly at the rollover: the next one, a day later
    expect(nextRollover(new Date("2026-10-06T01:00:00Z"), MVP_RULES).toISOString()).toBe("2026-10-07T01:00:00.000Z");
    expect(nextRollover(new Date("2026-12-31T23:00:00Z"), MVP_RULES).toISOString()).toBe("2027-01-01T01:00:00.000Z");
  });

  it("follows the zone's clock changes", () => {
    const berlin = { dayEndsAt: "04:00", dayTimeZone: "Europe/Berlin" };
    expect(nextRollover(new Date("2026-07-01T12:00:00Z"), berlin).toISOString()).toBe("2026-07-02T02:00:00.000Z");
    expect(nextRollover(new Date("2026-12-01T12:00:00Z"), berlin).toISOString()).toBe("2026-12-02T03:00:00.000Z");
  });

  it("labels a day by the date its window starts on", () => {
    expect(dayLabel(new Date("2026-10-05T10:00:00Z"), MVP_RULES)).toBe("2026-10-05");
    expect(dayLabel(new Date("2026-10-05T22:30:00Z"), MVP_RULES)).toBe("2026-10-05"); // 01:30 Moscow, Oct 6
    expect(dayLabel(new Date("2026-10-06T01:00:00Z"), MVP_RULES)).toBe("2026-10-06");
  });
});

describe("the playoff", () => {
  it("enters each slayer's strongest slaying team, by simulation against the champion", () => {
    const ann = p("ann");
    const entrants = playoffEntrants([slay(ann, big(5), "1"), slay(ann, tiny(), "2")], champion, content, MVP_RULES);
    expect(entrants).toHaveLength(1);
    expect(entrants[0]!.slay.at).toBe("1");
    // the same, whatever order the slays came in
    expect(playoffEntrants([slay(ann, tiny(), "1"), slay(ann, big(5), "2")], champion, content, MVP_RULES)[0]!.slay.at).toBe("2");
  });

  it("enters bots' slays like humans' (#587); skips slays on other content and the champion's own", () => {
    const slays = [slay(p("b", true), big(5), "1"), slay(p("old"), big(5), "2", { contentVersion: "old" }), slay(p("ann"), big(2), "3"), slay(champion.player, big(5), "4")];
    expect(playoffSlays(slays, champion, content).map((s) => s.player.id)).toEqual(["b", "ann"]);
    const entrants = playoffEntrants(slays, champion, content, MVP_RULES);
    expect(entrants.map((e) => e.player.id)).toEqual(["b", "ann"]);
    expect(entrants[0]!.player.bot).toBe(true);
  });

  it("bot slayers and one human slayer play a round-robin, which a bot can win (#587)", () => {
    const slays = [slay(p("bot-a", true), big(5), "1"), slay(p("maks"), tiny(), "2"), slay(p("bot-b", true), big(2), "3")];
    const { result, battles } = playRoundRobin(playoffEntrants(slays, champion, content, MVP_RULES), { seq: 1, day: "d", at: "t", content, rules: MVP_RULES, battleId: (i) => `g${i}` });
    expect(result.entrants.map((e) => e.id)).toEqual(["bot-a", "maks", "bot-b"]);
    expect(battles).toHaveLength(6);
    expect(result.winner).toMatchObject({ id: "bot-a", bot: true });
  });

  it("plays a double round-robin; the strongest team tops the table", () => {
    const entrants = playoffEntrants([slay(p("weak"), tiny(), "1"), slay(p("strong"), big(5), "2"), slay(p("mid"), big(2), "3")], champion, content, MVP_RULES);
    const { result, battles } = playRoundRobin(entrants, { seq: 1, day: "2026-10-05", at: "t", content, rules: MVP_RULES, battleId: (i) => `g${i}` });
    expect(battles).toHaveLength(6);
    expect(battles.every((b) => b.kind === "playoff" && b.runId === null && b.round === 0)).toBe(true);
    expect(result.games.map((g) => `${g.a.id}-${g.b.id}`)).toEqual(["weak-strong", "strong-weak", "weak-mid", "mid-weak", "strong-mid", "mid-strong"]);
    expect(result.battleIds).toEqual(["g0", "g1", "g2", "g3", "g4", "g5"]);
    expect(result.winner?.id).toBe("strong");
    expect(result.standings[0]).toMatchObject({ player: { id: "strong" }, wins: 4, losses: 0 });
    expect(result.standings.at(-1)!.player.id).toBe("weak");
    const n = result.standings.reduce((s, r) => s + r.wins + r.draws + r.losses, 0);
    expect(n).toBe(12);
  });

  it("one entrant wins without a game; none, no winner", () => {
    const one = playRoundRobin(playoffEntrants([slay(p("ann"), big(2), "1")], champion, content, MVP_RULES), { seq: 1, day: "d", at: "t", content, rules: MVP_RULES, battleId: String });
    expect(one.result).toMatchObject({ winner: { id: "ann" }, games: [], standings: [{ player: { id: "ann" }, wins: 0 }] });
    expect(playRoundRobin([], { seq: 1, day: "d", at: "t", content, rules: MVP_RULES, battleId: String }).result).toMatchObject({ winner: null, entrants: [] });
  });
});
