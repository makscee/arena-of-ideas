// The MVP's background jobs (mission #574). main.ts starts them on its
// runtime, and so does the API bot's in-process server, which stands in for
// main.ts; app tests start none. Each slice fills in its own job in its own
// module; this list already names them all.
import { botWorld } from "./bots.js";
import { dayRollover } from "./day.js";
import { fusionNamingJob } from "./fusions.js";
import type { MvpJob, MvpRuntime } from "./runtime.js";

export const MVP_JOBS: MvpJob[] = [
  dayRollover, // slice 5: the 04:00 rollover
  botWorld, // slice 6: the champion seed and the bot top-up
  fusionNamingJob, // slice 10: the naming queue
];

/** Starts every job on `rt`; returns one function that stops them all. */
export function startMvpJobs(rt: MvpRuntime, jobs: MvpJob[] = MVP_JOBS): () => void {
  const stops = jobs.map((job) => job(rt));
  return () => {
    for (const stop of stops) stop();
  };
}
