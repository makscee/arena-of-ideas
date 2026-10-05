// Fusion names (mission #574). Slice 10 owns this file: the local model on m1
// behind a blocklist, the stored discoveries (MvpStore.fusion, putFusion) and
// GET /fusions. mvpRuntime (./runtime.ts) wires fusionNaming() once, so slice
// 10 fills it in here and never edits decide() or the routes. The rules are on
// FusionDiscovery in the contract: the first fuse of a pair fixes its name, a
// model name is only ever prepared for pairs nobody has fused yet, and bots
// are never credited.
import type { FuseContext, PlayerRef, UnitContent } from "../../../src/mvp/contract.js";
import type { RunDeps, RunHooks } from "./runs.js";
import type { MvpJob } from "./runtime.js";
import type { MvpStore } from "./store.js";

/** Names an ordered pair for a fuse by `by`, synchronously. */
export type NameFusion = (first: UnitContent, second: UnitContent, by: PlayerRef) => FuseContext;

/** The deterministic fallback name: the front of first's name and the back of
 * second's ("Brawler" + "Medic" → "Brawdic"). The only portmanteau. */
export function portmanteau(first: string, second: string): string {
  const a = first.replace(/\s+/g, "");
  const b = second.replace(/\s+/g, "").toLowerCase();
  const head = a.slice(0, Math.ceil(a.length / 2));
  const tail = b.slice(Math.floor(b.length / 2));
  return head.slice(-1).toLowerCase() === tail[0] ? head + tail.slice(1) : head + tail;
}

/** The stored name, else the portmanteau, with the credit by FusionDiscovery's
 * rule (the stored discoverer, else `by` unless it is a bot). It only reads:
 * it never records a discovery or calls a model. */
export function storedOrPortmanteau(store: MvpStore): NameFusion {
  return (first, second, by) => {
    const known = store.fusion(first.id, second.id);
    return {
      name: known?.name ?? portmanteau(first.name, second.name),
      discoveredBy: known?.discoveredBy ?? (by.bot ? null : by),
    };
  };
}

/** Everything the runtime takes from slice 10, built once per runtime. */
export interface FusionNaming {
  /** decide()'s namer. Read-only like peek: the discovery is recorded by
   * hooks.onFuse, after the fuse went through. */
  nameFusion: NameFusion;
  /** RunDeps.peekFusionName: the stored name, else a prefetched model name,
   * else the portmanteau. Never records, never waits on the model. */
  peek: NameFusion;
  /** Slice 10 fills these in: onFuse records the FusionDiscovery (name and
   * credit rules on the contract); onDecision prefetches model names for every
   * ordered pair of Awoken, unfused units on the line, in the background, never
   * for a pair already discovered. */
  hooks: RunHooks;
}

/** Slice 10 replaces the body. Keep the prefetched-name cache in this module
 * (for example a WeakMap keyed by the store, which fusionNamingJob can reach
 * through rt.store), never in MvpStore. */
export function fusionNaming(rt: Pick<RunDeps, "store" | "content" | "now">): FusionNaming {
  const peek = storedOrPortmanteau(rt.store);
  return { nameFusion: peek, peek, hooks: {} };
}

/** Slice 10's background job, if the naming needs one (the model queue);
 * started with the other jobs (./jobs.ts). A no-op until then. */
export const fusionNamingJob: MvpJob = () => () => {};
