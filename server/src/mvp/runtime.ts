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
// Slice 4 passes its SQLite store from main.ts; tests keep the memory store.
import { MVP_RULES, type MvpContent, type MvpRules } from "../../../src/mvp/contract.js";
import { today } from "./day.js";
import { fusionNaming, type NameFusion } from "./fusions.js";
import type { RunDeps, RunHooks } from "./runs.js";
import { statsHooks } from "./stats.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";

export interface MvpDeps {
  content: MvpContent;
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
}

export interface MvpRuntime extends RunDeps {
  dev: boolean;
  invites: boolean;
}

/** A background job: starts on the runtime, returns its stop function. */
export type MvpJob = (rt: MvpRuntime) => () => void;

export function mvpRuntime(deps: MvpDeps): MvpRuntime {
  const store = deps.store ?? new MemoryMvpStore();
  const now = deps.now ?? (() => new Date());
  const base = { store, content: deps.content, now };
  const naming = fusionNaming(base);
  const rt: MvpRuntime = {
    ...base,
    rules: deps.rules ?? MVP_RULES,
    seed: deps.seed ?? (() => Math.floor(Math.random() * 2 ** 32)),
    today: () => today(rt),
    hooks: [naming.hooks, statsHooks(base), ...(deps.hooks ?? [])],
    nameFusion: deps.nameFusion ?? naming.nameFusion,
    peekFusionName: naming.peek,
    dev: deps.dev ?? false,
    invites: deps.invites ?? false,
  };
  return rt;
}

export function isMvpRuntime(x: MvpDeps | MvpRuntime): x is MvpRuntime {
  return typeof (x as Partial<MvpRuntime>).today === "function";
}
