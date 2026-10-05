// The day (mission #574). Slice 5 owns this file: the day counter, the
// rollover at rules.dayEndsAt in rules.dayTimeZone (04:00 Moscow), the
// playoff, and the dev "end day now". Everyone else reads the day through
// RunDeps.today() (= today() below) and the routes through dayView(). The
// contract says who writes which row ("day, champion, rating"). The pure
// rules (times, the strongest-team pick, the round-robin) are in
// src/mvp/day.ts.
import type { Champion, DayState, DayView, PlayerRef, Rating } from "../../../src/mvp/contract.js";
import { dayLabel, nextRollover, playoffEntrants, playRoundRobin } from "../../../src/mvp/day.js";
import type { RunDeps } from "./runs.js";
import type { MvpJob } from "./runtime.js";

type DayDeps = Pick<RunDeps, "store" | "now" | "rules" | "content">;

/** The stored day without rolling it over: the first call creates day 1, and
 * a day stored with a placeholder endsAt (before slice 5) gets the real one. */
function currentDay(rt: DayDeps): DayState {
  const cur = rt.store.currentDay();
  if (cur && cur.endsAt > cur.startedAt) return cur;
  if (cur) {
    const fixed = { ...cur, endsAt: nextRollover(new Date(cur.startedAt), rt.rules).toISOString() };
    rt.store.putDay(fixed);
    return fixed;
  }
  const now = rt.now();
  const day: DayState = { seq: 1, day: dayLabel(now, rt.rules), startedAt: now.toISOString(), endsAt: nextRollover(now, rt.rules).toISOString() };
  rt.store.putDay(day);
  return day;
}

/** The current day; the first call creates day 1, and a call past its endsAt
 * ends it first (endDay), so the day is never stale even between job ticks. */
export function today(rt: DayDeps): DayState {
  const cur = currentDay(rt);
  if (rt.now().getTime() < Date.parse(cur.endsAt)) return cur;
  endDay(rt);
  return currentDay(rt);
}

/** What Home and GET /day show. */
export function dayView(rt: Pick<RunDeps, "store" | "today">): DayView {
  const d = rt.today();
  return {
    seq: d.seq,
    day: d.day,
    endsAt: d.endsAt,
    champion: rt.store.currentChampion() ?? null,
    slayers: new Set(rt.store.slays(d.seq).map((s) => s.player.id)).size,
    lastPlayoff: rt.store.playoff(d.seq - 1) ?? null,
  };
}

/** Ends the day now (POST /dev/end-day, and the rollover): each slayer's
 * strongest slaying team enters a round-robin playoff, its winner is the next
 * day's champion (a day with no slayers keeps the champion), and day seq + 1
 * starts, ending at the next rules.dayEndsAt. Writes the playoff, its battles,
 * the new Champion row and the records (playoffWins, daysAsChampion). */
export function endDay(rt: DayDeps): DayView {
  const { store, content, rules } = rt;
  const d = currentDay(rt);
  const now = rt.now();
  const at = now.toISOString();
  const champ = store.currentChampion();
  const entrants = playoffEntrants(store.slays(d.seq), champ, content, rules);
  const { result, battles } = playRoundRobin(entrants, { seq: d.seq, day: d.day, at, content, rules, battleId: (i) => `playoff-${d.seq}-${i + 1}` });
  for (const b of battles) store.putBattle(b);
  store.putPlayoff(result);

  const next: DayState = { seq: d.seq + 1, day: dayLabel(now, rules), startedAt: at, endsAt: nextRollover(now, rules).toISOString() };
  const winner = result.winner ? entrants.find((e) => e.player.id === result.winner!.id)! : undefined;
  let crowned: Champion | undefined;
  if (winner) {
    crowned = { seq: next.seq, day: next.day, player: winner.player, line: structuredClone(winner.line), since: at, contentVersion: winner.slay.contentVersion };
    bump(rt, winner.player, "playoffWins");
  } else if (champ) {
    crowned = { ...structuredClone(champ), seq: next.seq, day: next.day };
  }
  if (crowned) {
    store.putChampion(crowned);
    if (!crowned.player.bot) bump(rt, crowned.player, "daysAsChampion");
  }
  store.putDay(next);
  return dayView({ store, today: () => next });
}

function bump(rt: DayDeps, player: PlayerRef, field: "playoffWins" | "daysAsChampion"): void {
  const r: Rating = rt.store.rating(player.id) ?? { player, rating: rt.rules.ratingStart, runs: 0, slays: 0, daysAsChampion: 0, playoffWins: 0 };
  rt.store.putRating({ ...r, [field]: r[field] + 1 });
}

/** True when `battleId` is a Crown fight that slew today's champion and
 * `viewerId` isn't its slayer: slayers' teams stay hidden until the day ends. */
export function hiddenSlay(rt: RunDeps, battleId: string, viewerId: string | undefined): boolean {
  const s = rt.store.slays(rt.today().seq).find((x) => x.battleId === battleId);
  return !!s && s.player.id !== viewerId;
}

/** How often the rollover job looks at the clock. */
export const ROLLOVER_TICK_MS = 30_000;

/** Slice 5's rollover timer: today() ends the day once its endsAt has passed. */
export const dayRollover: MvpJob = (rt) => {
  const tick = () => {
    try {
      rt.today();
    } catch (err) {
      console.error("day rollover failed", err);
    }
  };
  tick();
  const timer = setInterval(tick, ROLLOVER_TICK_MS);
  timer.unref?.();
  return () => clearInterval(timer);
};
