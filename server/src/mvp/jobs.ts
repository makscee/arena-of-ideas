// The MVP's background jobs (mission #574). main.ts starts them on its
// runtime, and so does the API bot's in-process server, which stands in for
// main.ts; app tests start none. Each slice fills in its own job in its own
// module; this list already names them all.
import { botWorld } from "./bots.js";
import { dailyPostJob } from "./daily-post.js";
import { dayRollover } from "./day.js";
import { fusionRuNamingJob } from "./fusion-names-ru.js";
import { fusionNamingJob } from "./fusions.js";
import { ideaReadingJob } from "./idea-reading.js";
import type { MvpJob, MvpRuntime } from "./runtime.js";
import { telegramJob } from "./telegram.js";
import { overnightJob } from "./votes.js";

export const MVP_JOBS: MvpJob[] = [
  dayRollover, // slice 5: the 04:00 rollover
  botWorld, // slice 6: the champion seed and the bot top-up
  fusionNamingJob, // slice 10: the naming queue
  fusionRuNamingJob, // M4-4: Russian names for stored discoveries (ARENA_NAMER_URL, ARENA_NAMER_RU=1)
  ideaReadingJob, // M2-5: reads written ideas (ARENA_IDEA_READER)
  overnightJob, // M2-8: the overnight check after the day end
  telegramJob, // M4-6: the Telegram bot's long poll (only with rt.telegram)
  dailyPostJob, // M4-8: the daily post to Telegram (ARENA_TELEGRAM_POST=1)
];

/** Starts every job on `rt`; returns one function that stops them all. */
export function startMvpJobs(rt: MvpRuntime, jobs: MvpJob[] = MVP_JOBS): () => void {
  const stops = jobs.map((job) => job(rt));
  return () => {
    for (const stop of stops) stop();
  };
}
