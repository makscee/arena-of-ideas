// Fusion names (mission #574). Slice 10 owns naming: the local model on m1
// behind a blocklist, filled in the background, and the discoveries in
// MvpStore. A fuse never waits on the model: it takes the stored name, else
// the deterministic portmanteau below.
import type { FuseContext, PlayerRef, UnitContent } from "../../../src/mvp/contract.js";
import type { MvpStore } from "./store.js";

/** Names an ordered pair for a fuse by `by`, synchronously. */
export type NameFusion = (first: UnitContent, second: UnitContent, by: PlayerRef) => FuseContext;

/** The deterministic fallback name: the front of first's name and the back of
 * second's ("Brawler" + "Medic" → "Brawdic"). */
export function portmanteau(first: string, second: string): string {
  const a = first.replace(/\s+/g, "");
  const b = second.replace(/\s+/g, "").toLowerCase();
  const head = a.slice(0, Math.ceil(a.length / 2));
  const tail = b.slice(Math.floor(b.length / 2));
  return head.slice(-1).toLowerCase() === tail[0] ? head + tail.slice(1) : head + tail;
}

/** The default namer, and the only one the preview uses: the stored name, else
 * the portmanteau, with the credit by FusionDiscovery's rule. It only reads:
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
