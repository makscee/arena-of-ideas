// Ideas-table persistence — the browser's ideas table, in localStorage.
// The backing is the kernel's PersistedIdeaStore over a storage key (the same
// engine and IdeasData shape any persistent backing writes); this file owns the
// key and the medium only, semantics stay in the kernel.
//
// Storage is injected (main.ts passes window.localStorage) so tests drive the
// same code over an in-memory stub — the same pattern run-store.ts uses for the
// ladder and active run.

import {
  PersistedIdeaStore,
  emptyIdeasData,
  parseIdeasData,
  type IdeaStore,
  type IdeasData,
} from "../src/index.js";
import type { KVStorage } from "./run-store.js";

// v2: the serialized vote shape changed (a flat playerId[] set became a
// directional playerId→"up"|"down" map), so a v1 blob is not a v2 blob.
// parseIdeasData throws on a shape mismatch; bumping the key drops any stale v1
// table instead — the local ideas table is dev-only, so dropping it on the shape
// change is acceptable (the server backing is the real one).
const IDEAS_KEY = "aoi.ideas.v2";

export type { KVStorage } from "./run-store.js";

/** Open the localStorage-backed ideas table. Corrupt stored JSON throws loudly
 * (the FileLadderStore rule: silently starting fresh would drop every idea and
 * vote); the caller surfaces the error, never swallows it. */
export function openLocalIdeas(storage: KVStorage): IdeaStore {
  const raw = storage.getItem(IDEAS_KEY);
  const data = raw === null ? emptyIdeasData() : parseIdeasData(raw, `localStorage "${IDEAS_KEY}"`);
  return new PersistedIdeaStore(data, (d) => storage.setItem(IDEAS_KEY, JSON.stringify(d)));
}

/** Serialize an ideas table to the stored JSON string. The inverse of
 * parseIdeasData (re-exported from the kernel): serialize → parse round-trips
 * the table exactly. */
export function serializeIdeas(data: IdeasData): string {
  return JSON.stringify(data);
}

export { parseIdeasData, emptyIdeasData } from "../src/index.js";
