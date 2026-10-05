// Basic stats (mission #574). Slice 11 owns this file: unit win and pick
// rates, champion history, the discovered fusions (read from slice 10's
// store) and players' records, served as GET /stats. mvpRuntime wires
// statsHooks() once, so its hooks see bot runs and HTTP runs alike; past
// battles are in MvpStore.battles().
import type { StatsView } from "../../../src/mvp/contract.js";
import { MvpNotYet } from "./errors.js";
import type { RunDeps, RunHooks } from "./runs.js";

/** Slice 11's observers of the run engine. None until then. */
export function statsHooks(_rt: Pick<RunDeps, "store" | "content" | "now">): RunHooks {
  return {};
}

/** GET /stats. Slice 11 fills it in. */
export function statsView(_rt: RunDeps): StatsView {
  throw new MvpNotYet("stats arrive in slice 11");
}
