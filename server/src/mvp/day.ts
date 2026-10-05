// The day (mission #574). Slice 5 owns this file: the day counter, the
// rollover at rules.dayEndsAt in rules.dayTimeZone (04:00 Moscow), the
// playoff, and the dev "end day now". Everyone else reads the day through
// RunDeps.today() (= today() below) and the routes through dayView(). The
// contract says who writes which row ("day, champion, rating").
import type { DayState, DayView } from "../../../src/mvp/contract.js";
import { MvpNotYet } from "./errors.js";
import type { RunDeps } from "./runs.js";
import type { MvpJob, MvpRuntime } from "./runtime.js";

/** The current day; the first call creates day 1. Slice 5 adds the rollover
 * check (now() past endsAt rolls the day over first) and the real endsAt;
 * until then day 1 lasts forever and endsAt is a placeholder. */
export function today(rt: Pick<RunDeps, "store" | "now">): DayState {
  const cur = rt.store.currentDay();
  if (cur) return cur;
  const at = rt.now().toISOString();
  const day: DayState = { seq: 1, day: at.slice(0, 10), startedAt: at, endsAt: at };
  rt.store.putDay(day);
  return day;
}

/** What Home and GET /day show. */
export function dayView(rt: RunDeps): DayView {
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

/** Ends the day now (POST /dev/end-day, and the rollover job): the playoff,
 * the next champion row, seq + 1. Slice 5 fills it in. */
export function endDay(_rt: MvpRuntime): DayView {
  throw new MvpNotYet("the day arrives in slice 5");
}

/** Slice 5's rollover timer: calls endDay at rules.dayEndsAt. A no-op until then. */
export const dayRollover: MvpJob = () => () => {};
