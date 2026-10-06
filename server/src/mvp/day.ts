// The day (mission #574). Slice 5 owns this file: the day counter, the
// rollover at rules.dayEndsAt in rules.dayTimeZone (04:00 Moscow), the
// playoff, and the dev "end day now". Everyone else reads the day through
// RunDeps.today() (= today() below) and the routes through dayView(). The
// contract says who writes which row ("day, champion, rating"). The pure
// rules (times, the strongest-team pick, the round-robin) are in
// src/mvp/day.ts.
import type { Champion, DayState, DayView, PlayerRef, Rating } from "../../../src/mvp/contract.js";
import { dayLabel, nextRollover, playoffEntrants, playoffSlays, playRoundRobin } from "../../../src/mvp/day.js";
import type { RunDeps } from "./runs.js";
import type { MvpJob } from "./runtime.js";
import type { MvpStore } from "./store.js";

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

/** How long today() keeps serving a day whose end failed before it tries
 * endDay again. */
export const END_DAY_RETRY_MS = 60_000;
/** When ending each store's current day last failed (rt.now() ms). */
const endDayFailedAt = new WeakMap<MvpStore, number>();

/** The current day; the first call creates day 1, and a call past its endsAt
 * ends it first (endDay), so the day is never stale even between job ticks.
 * When endDay throws, today() logs it and keeps serving the current day, and
 * tries again at most once per END_DAY_RETRY_MS: a broken day end never
 * fails every request. */
export function today(rt: DayDeps): DayState {
  const cur = currentDay(rt);
  const now = rt.now().getTime();
  if (now < Date.parse(cur.endsAt)) return cur;
  const failed = endDayFailedAt.get(rt.store);
  if (failed !== undefined && now - failed < END_DAY_RETRY_MS) return cur;
  try {
    endDay(rt);
  } catch (err) {
    endDayFailedAt.set(rt.store, now);
    console.error(`[day] ending day ${cur.seq} failed; serving it and retrying in ${END_DAY_RETRY_MS / 1000}s`, err);
    return cur;
  }
  endDayFailedAt.delete(rt.store);
  return currentDay(rt);
}

/** Day `seq`'s champion: the latest at or before it. A day end that failed
 * after crowning day seq + 1 leaves that row stored early; it isn't anyone's
 * champion until day seq + 1 starts. */
export function championOf(store: MvpStore, seq: number): Champion | undefined {
  const latest = store.currentChampion();
  if (!latest || latest.seq <= seq) return latest;
  return store.champions().filter((c) => c.seq <= seq).at(-1);
}

/** Today's champion (championOf today().seq): the one Home shows and the
 * Crown fights. */
export function todaysChampion(rt: Pick<RunDeps, "store" | "today">): Champion | undefined {
  return championOf(rt.store, rt.today().seq);
}

/** What Home and GET /day show. `slayers` counts the players whose slays can
 * enter tonight's playoff (playoffSlays), so it never promises more. */
export function dayView(rt: Pick<RunDeps, "store" | "today" | "content">): DayView {
  const d = rt.today();
  const champion = championOf(rt.store, d.seq);
  return {
    seq: d.seq,
    day: d.day,
    endsAt: d.endsAt,
    champion: champion ?? null,
    slayers: new Set(playoffSlays(rt.store.slays(d.seq), champion, rt.content).map((s) => s.player.id)).size,
    lastPlayoff: rt.store.playoff(d.seq - 1) ?? null,
  };
}

/** Ends the day now (POST /dev/end-day, and the rollover): each slayer's
 * (human or bot) strongest slaying team enters a round-robin playoff, its
 * winner is the next day's champion (a day with no slayers keeps the
 * champion), and day seq + 1 starts, ending at the next rules.dayEndsAt.
 * Writes the playoff, its battles, the new Champion row and the records
 * (playoffWins, daysAsChampion; a bot's too, never its rating). */
export function endDay(rt: DayDeps): DayView {
  const { store, content, rules } = rt;
  const d = currentDay(rt);
  const now = rt.now();
  const at = now.toISOString();
  // Day d's champion: the latest at or before its seq, not one that an earlier
  // attempt at this day end crowned before it failed.
  const champ = championOf(store, d.seq);
  const entrants = playoffEntrants(store.slays(d.seq), champ, content, rules);
  const { result, battles } = playRoundRobin(entrants, { seq: d.seq, day: d.day, at, content, rules, battleId: (i) => `playoff-${d.seq}-${i + 1}` });
  for (const b of battles) store.putBattle(b);
  store.putPlayoff(result);

  const next: DayState = { seq: d.seq + 1, day: dayLabel(now, rules), startedAt: at, endsAt: nextRollover(now, rules).toISOString() };
  const winner = result.winner ? entrants.find((e) => e.player.id === result.winner!.id)! : undefined;
  let crowned: Champion | undefined;
  if (winner) {
    crowned = { seq: next.seq, day: next.day, player: winner.player, line: structuredClone(winner.line), since: at, contentVersion: winner.slay.contentVersion, rating: winner.slay.rating ?? rules.ratingStart };
  } else if (champ) {
    crowned = { ...structuredClone(champ), seq: next.seq, day: next.day };
  }
  if (crowned) store.putChampion(crowned);
  // Every write above is keyed by seq, so a retry after a failure rewrites
  // the same rows. The records only move once day seq + 1 is stored: an end
  // that failed before this point and runs again never counts them twice.
  store.putDay(next);
  // Records count for bots too (bump never touches a rating).
  if (winner) bump(rt, winner.player, "playoffWins");
  if (crowned) bump(rt, crowned.player, "daysAsChampion");
  return dayView({ store, content, today: () => next });
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
