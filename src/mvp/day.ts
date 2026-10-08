// The day's rules (mission #574, slice 5), pure: when a day ends, its label,
// each slayer's strongest slaying team and the day-end round-robin playoff.
// The server (server/src/mvp/day.ts) supplies the clock, the ids and the store.

import type { Side } from "../types.js";
import type { BattleRecord, Champion, LineUnit, MvpContent, MvpRules, PlayerRef, PlayoffResult, Slay } from "./contract.js";
import { fightLines } from "./fight.js";

// ---------- time ----------

/** The wall clock in `timeZone` at `at`. */
function wallClock(at: Date, timeZone: string): { y: number; m: number; d: number; hh: number; mm: number } {
  const parts = new Intl.DateTimeFormat("en-US", { timeZone, hourCycle: "h23", year: "numeric", month: "2-digit", day: "2-digit", hour: "2-digit", minute: "2-digit" }).formatToParts(at);
  const n = (t: string) => Number(parts.find((p) => p.type === t)!.value);
  return { y: n("year"), m: n("month"), d: n("day"), hh: n("hour"), mm: n("minute") };
}

/** `timeZone`'s offset from UTC at `at`, in ms. */
function offsetMs(at: Date, timeZone: string): number {
  const w = wallClock(at, timeZone);
  return Date.UTC(w.y, w.m - 1, w.d, w.hh, w.mm) - Math.floor(at.getTime() / 60_000) * 60_000;
}

/** The instant the wall clock in `timeZone` shows y-m-d hh:mm. */
function instant(y: number, m: number, d: number, hh: number, mm: number, timeZone: string): Date {
  const guess = Date.UTC(y, m - 1, d, hh, mm);
  const first = guess - offsetMs(new Date(guess), timeZone);
  return new Date(guess - offsetMs(new Date(first), timeZone));
}

function hhmm(rules: Pick<MvpRules, "dayEndsAt">): [number, number] {
  const [hh, mm] = rules.dayEndsAt.split(":").map(Number);
  return [hh ?? 4, mm ?? 0];
}

/** The first rollover (rules.dayEndsAt in rules.dayTimeZone) strictly after `at`. */
export function nextRollover(at: Date, rules: Pick<MvpRules, "dayEndsAt" | "dayTimeZone">): Date {
  const [hh, mm] = hhmm(rules);
  const w = wallClock(at, rules.dayTimeZone);
  for (let add = 0; add < 3; add++) {
    const t = instant(w.y, w.m, w.d + add, hh, mm, rules.dayTimeZone);
    if (t.getTime() > at.getTime()) return t;
  }
  throw new Error("no rollover within 3 days");
}

/** A day's label: the date (YYYY-MM-DD in rules.dayTimeZone) its rollover-to-
 * rollover window starts on. A day ending at 04:00 on Oct 6 is "2026-10-05". */
export function dayLabel(at: Date, rules: Pick<MvpRules, "dayEndsAt" | "dayTimeZone">): string {
  const [hh, mm] = hhmm(rules);
  const w = wallClock(at, rules.dayTimeZone);
  const before = w.hh * 60 + w.mm < hh * 60 + mm;
  const d = new Date(Date.UTC(w.y, w.m - 1, w.d - (before ? 1 : 0)));
  return d.toISOString().slice(0, 10);
}

// ---------- the playoff ----------

/** A small deterministic seed for game `i` of day `seq`'s playoff (or its sims). */
export function playoffSeed(seq: number, i: number, salt = 0): number {
  let h = (seq * 0x9e3779b1 + i * 0x85ebca6b + salt * 0xc2b2ae35) >>> 0;
  h = Math.imul(h ^ (h >>> 16), 0x45d9f3b) >>> 0;
  return (h ^ (h >>> 16)) >>> 0;
}

/** Seeds per side each strongest-team candidate plays against the champion. */
export const STRONGEST_SIM_SEEDS = 8;
/** At most this many of a slayer's latest slays are simulated. */
export const STRONGEST_MAX_CANDIDATES = 6;

export interface PlayoffEntrant {
  player: PlayerRef;
  line: LineUnit[];
  /** The slay this team comes from. */
  slay: Slay;
}

/** Each slayer's strongest slaying team of the day, by simulation: every
 * candidate (their latest STRONGEST_MAX_CANDIDATES slays on the live content)
 * fights the day's champion STRONGEST_SIM_SEEDS times as side A and as many as
 * side B; the most wins (then fewest losses, then the latest slay) enters.
 * Bots' slays enter like humans' (#587: a player alone still sees a real
 * playoff); slays on other content are skipped. The reigning champion's own
 * slays enter like anyone's, so they can be crowned again with another team.
 * Entrants come in the order of each slayer's
 * first slay. */
export function playoffEntrants(slays: Slay[], champion: Champion | undefined, content: MvpContent, rules: MvpRules): PlayoffEntrant[] {
  const live = playoffSlays(slays, content);
  const byPlayer = new Map<string, Slay[]>();
  for (const s of live) byPlayer.set(s.player.id, [...(byPlayer.get(s.player.id) ?? []), s]);
  const out: PlayoffEntrant[] = [];
  for (const own of byPlayer.values()) {
    const candidates = own.slice(-STRONGEST_MAX_CANDIDATES);
    let best = candidates.at(-1)!;
    if (candidates.length > 1 && champion && champion.contentVersion === content.version) {
      let bestScore = -Infinity;
      candidates.forEach((s, ci) => {
        const score = simScore(s.line, champion, content, rules, ci);
        // >= keeps the later slay on a tie
        if (score >= bestScore) [best, bestScore] = [s, score];
      });
    }
    out.push({ player: best.player, line: structuredClone(best.line), slay: best });
  }
  return out;
}

/** The slays that can enter the playoff: on the live content, by a human or
 * a bot alike, the reigning champion's own included. Home's slayer count uses
 * the same rule. */
export function playoffSlays(slays: Slay[], content: Pick<MvpContent, "version">): Slay[] {
  return slays.filter((s) => s.contentVersion === content.version);
}

/** Wins minus a thousandth per loss, over both sides, against the champion. */
function simScore(line: LineUnit[], champion: Champion, content: MvpContent, rules: MvpRules, salt: number): number {
  const me: PlayerRef = { id: "sim", name: "sim", bot: true };
  let wins = 0;
  let losses = 0;
  for (let i = 0; i < STRONGEST_SIM_SEEDS; i++) {
    for (const asA of [true, false]) {
      const mine = { player: me, line };
      const theirs = { player: champion.player, line: champion.line };
      const b = fightLines(asA ? mine : theirs, asA ? theirs : mine, { battleId: "sim", seed: playoffSeed(champion.seq, i, salt), kind: "playoff", round: 0, runId: null, at: "", content, rules });
      const won: Side = asA ? "A" : "B";
      if (b.winner === won) wins++;
      else if (b.winner !== "draw") losses++;
    }
  }
  return wins - losses / 1000;
}

/** A double round-robin: every pair plays twice, each entrant once as side A.
 * Standings: wins, then draws, then fewest losses, then the earlier first
 * slay. The top entrant wins; with one entrant it wins without a game; with
 * none there is no winner. */
export function playRoundRobin(
  entrants: PlayoffEntrant[],
  o: { seq: number; day: string; at: string; content: MvpContent; rules: MvpRules; battleId: (i: number) => string },
): { result: PlayoffResult; battles: BattleRecord[] } {
  const battles: BattleRecord[] = [];
  const games: PlayoffResult["games"] = [];
  const table = new Map(entrants.map((e, i) => [e.player.id, { player: e.player, wins: 0, draws: 0, losses: 0, order: i }]));
  for (let i = 0; i < entrants.length; i++) {
    for (let j = i + 1; j < entrants.length; j++) {
      for (const [a, b] of [[entrants[i]!, entrants[j]!], [entrants[j]!, entrants[i]!]] as const) {
        const n = battles.length;
        const rec = fightLines({ player: a.player, line: a.line }, { player: b.player, line: b.line }, { battleId: o.battleId(n), seed: playoffSeed(o.seq, n), kind: "playoff", round: 0, runId: null, at: o.at, content: o.content, rules: o.rules });
        battles.push(rec);
        games.push({ a: a.player, b: b.player, battleId: rec.battleId, winner: rec.winner });
        const ta = table.get(a.player.id)!;
        const tb = table.get(b.player.id)!;
        if (rec.winner === "draw") [ta.draws, tb.draws] = [ta.draws + 1, tb.draws + 1];
        else if (rec.winner === "A") [ta.wins, tb.losses] = [ta.wins + 1, tb.losses + 1];
        else [tb.wins, ta.losses] = [tb.wins + 1, ta.losses + 1];
      }
    }
  }
  const standings = [...table.values()].sort((x, y) => y.wins - x.wins || y.draws - x.draws || x.losses - y.losses || x.order - y.order);
  return {
    battles,
    result: {
      seq: o.seq,
      day: o.day,
      entrants: entrants.map((e) => e.player),
      winner: standings[0]?.player ?? null,
      battleIds: battles.map((b) => b.battleId),
      standings: standings.map(({ player, wins, draws, losses }) => ({ player, wins, draws, losses })),
      games,
    },
  };
}
