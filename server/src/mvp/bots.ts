// Bots and the world (mission #574). Slice 6 owns this file:
// - At start, seed the champion for rt.today().seq when currentChampion() is
//   missing or stale (contract.ts, "day, champion, rating"): a strong bot team,
//   picked by simulation with fightLines (src/mvp/fight.ts).
// - Keep every round's ghost pool full with a periodic top-up: bot players
//   (PlayerRef bot: true, store.addPlayer) play whole runs in-process through
//   startRun and decide (./runs.ts) on `rt`, so they share the store, the day
//   and the hooks with HTTP runs.
// The jobs runner (./jobs.ts) starts it in main.ts; tests call it themselves.
import type { MvpJob } from "./runtime.js";

/** Starts the bots; returns a stop function. A no-op until slice 6. */
export const botWorld: MvpJob = () => () => {};
