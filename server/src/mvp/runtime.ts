// The MVP world (mission #574). One runtime holds the store, the content, the
// rules, the clock, the day, the fusion namer and the hooks; every way in uses
// the same one: the HTTP routes (./app.ts), slice 6's in-process bots and the
// background jobs (./jobs.ts). mvpRuntime() is the only place defaults are
// filled, and it wires each slice's module once, so a slice fills in its own
// file instead of editing this one:
//   ./day.ts      slice 5   today(), dayView(), endDay(), the rollover job
//   ./bots.ts     slice 6   the champion seed and the bot top-up job
//   ./fusions.ts  slice 10  the namer, its hooks and its job
//   ./stats.ts    slice 11  the stats hooks and GET /stats
//   ./votes.ts    M2-8      the overnight check's tuner and job, votes
// Slice 4 passes its SQLite store from main.ts; tests keep the memory store.
import { MVP_RULES, type MvpContent, type MvpRules } from "../../../src/mvp/contract.js";
import { today } from "./day.js";
import { fusionNaming, type NameFusion } from "./fusions.js";
import { poolBook } from "./pool.js";
import type { RunDeps, RunHooks } from "./runs.js";
import { statsHooks } from "./stats.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";
import { childTuner, type Tuner } from "./votes.js";

export interface MvpDeps {
  /** The content while the store has no pool (tests with a bare store); with
   * a pool (main.ts seeds one), rt.content is always the store's live pool. */
  content?: MvpContent;
  /** Default: an in-memory store. */
  store?: MvpStore;
  /** Default: MVP_RULES. */
  rules?: MvpRules;
  /** Seed source; tests pin it. */
  seed?: () => number;
  now?: () => Date;
  /** Overrides decide()'s namer (tests). Default: slice 10's fusionNaming. */
  nameFusion?: NameFusion;
  /** Extra observers (tests). The slices' own hooks are wired here already. */
  hooks?: RunHooks[];
  /** Serves /dev/* (main.ts: MVP_DEV=1); without it they answer 404. */
  dev?: boolean;
  /** Invite-only (slice 13, main.ts: MVP_INVITES=1): players come from
   * invite links and session tokens, and only admin invites see /dev/*. */
  invites?: boolean;
  /** Open to all (main.ts: MVP_OPEN=1), on an invite-only server: anyone who
   * comes without a link gets the join name screen, as if they had opened the
   * open join link (R4-20). Sessions, links and the join limit stay. */
  open?: boolean;
  /** M2-8's overnight check: `night` for the job (the full meta check),
   * `dev` for the dev button (the quick one). Default: M2-7's tuner as a
   * child process (./votes.ts childTuner). */
  tuner?: { night: Tuner; dev: Tuner };
  /** M2-10's rotation at the day end (main.ts: MVP_ROTATION=1); off, the day
   * end never changes the pool. */
  rotation?: boolean;
}

export interface MvpRuntime extends RunDeps {
  dev: boolean;
  invites: boolean;
  open: boolean;
  tuner: { night: Tuner; dev: Tuner };
  rotation: boolean;
}

/** A background job: starts on the runtime, returns its stop function. */
export type MvpJob = (rt: MvpRuntime) => () => void;

export function mvpRuntime(deps: MvpDeps): MvpRuntime {
  const store = deps.store ?? new MemoryMvpStore();
  const now = deps.now ?? (() => new Date());
  const pools = poolBook(store, deps.content);
  // A getter: a pool change (a new snapshot) reaches every reader at once.
  const base = { store, get content() { return pools.current(); }, contentFor: pools.of, now };
  const naming = fusionNaming(base);
  const rt: MvpRuntime = {
    store,
    get content() { return pools.current(); },
    contentFor: pools.of,
    now,
    rules: deps.rules ?? MVP_RULES,
    seed: deps.seed ?? (() => Math.floor(Math.random() * 2 ** 32)),
    today: () => today(rt),
    hooks: [naming.hooks, statsHooks(base), ...(deps.hooks ?? [])],
    nameFusion: deps.nameFusion ?? naming.nameFusion,
    peekFusionName: naming.peek,
    dev: deps.dev ?? false,
    invites: deps.invites ?? false,
    open: (deps.invites ?? false) && (deps.open ?? false),
    tuner: deps.tuner ?? { night: childTuner("full"), dev: childTuner("quick") },
    rotation: deps.rotation ?? false,
  };
  return rt;
}

export function isMvpRuntime(x: MvpDeps | MvpRuntime): x is MvpRuntime {
  return typeof (x as Partial<MvpRuntime>).today === "function";
}
