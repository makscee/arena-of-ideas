// Run persistence — the browser's ladder and active run, in localStorage.
// The ladder backing is the kernel's PersistedLadderStore over a storage key
// (the same engine and LadderData shape FileLadderStore writes to disk); the
// active run round-trips through the kernel's serializeRun/deserializeRun.
// This file owns keys and the medium only; semantics stay in the kernel.
//
// Storage is injected (main.ts passes window.localStorage) so tests drive the
// same code over an in-memory stub — the teams.ts store predates this pattern.

import {
  PersistedLadderStore,
  deserializeRun,
  emptyLadderData,
  parseLadderData,
  serializeRun,
  type LadderStore,
  type RunState,
  type UnitDef,
} from "../src/index.js";

const LADDER_KEY = "aoi.ladder.v1";
const RUN_KEY = "aoi.run.v1";
const BATTLE_KEY = "aoi.run-battle.v1";
const RUN_SEQ_KEY = "aoi.run-seq.v1";

/** The storage surface this module needs — window.localStorage, or a test stub. */
export type KVStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">;

/** Open the localStorage-backed ladder. Corrupt stored JSON throws loudly
 * (the FileLadderStore rule: silently starting fresh would orphan every
 * ghost); the caller surfaces the error, never swallows it. */
export function openLocalLadder(storage: KVStorage): LadderStore {
  const raw = storage.getItem(LADDER_KEY);
  const data = raw === null ? emptyLadderData() : parseLadderData(raw, `localStorage "${LADDER_KEY}"`);
  return new PersistedLadderStore(data, (d) => storage.setItem(LADDER_KEY, JSON.stringify(d)));
}

/** A fought battle awaiting its replay/result screen — stored BY VALUE
 * (teams + seed), so a reload mid-battle recomputes the identical log
 * without consulting the ladder. The run's own registry replays it. */
export interface StoredBattle {
  /** The line as fielded, battle-ready (toBattleTeam output). */
  teamA: UnitDef[];
  /** The drawn ghost / challenged champion, as it was when drawn. */
  teamB: UnitDef[];
  seed: number;
  /** Display line: who the opponent was ("ghost auto-3", "champion web-2"). */
  opponentLabel: string;
}

/** The persisted active run: its state, plus the pending battle when the
 * player abandoned mid-replay. Exactly one run is active at a time. */
export interface StoredRun {
  state: RunState;
  battle?: StoredBattle;
}

export function saveRun(storage: KVStorage, state: RunState, battle?: StoredBattle): void {
  storage.setItem(RUN_KEY, serializeRun(state));
  if (battle !== undefined) storage.setItem(BATTLE_KEY, JSON.stringify(battle));
  else storage.removeItem(BATTLE_KEY);
}

/** The active run, or null when none is stored. Corrupt data throws loudly. */
export function loadRun(storage: KVStorage): StoredRun | null {
  const raw = storage.getItem(RUN_KEY);
  if (raw === null) return null;
  const state = deserializeRun(raw);
  const battleRaw = storage.getItem(BATTLE_KEY);
  if (battleRaw === null) return { state };
  const battle = JSON.parse(battleRaw) as StoredBattle;
  if (!Array.isArray(battle?.teamA) || !Array.isArray(battle?.teamB) || typeof battle?.seed !== "number") {
    throw new Error(`stored battle ("${BATTLE_KEY}") is not a StoredBattle — refusing to revive it`);
  }
  return { state, battle };
}

export function clearRun(storage: KVStorage): void {
  storage.removeItem(RUN_KEY);
  storage.removeItem(BATTLE_KEY);
}

/** The next run's id — "web-N" off a persisted counter, so runs sharing this
 * browser's ladder stay distinct (own-ghost exclusion is by runId) without a
 * wall-clock read. */
export function nextRunId(storage: KVStorage): string {
  const n = Number(storage.getItem(RUN_SEQ_KEY) ?? "0") + 1;
  storage.setItem(RUN_SEQ_KEY, String(n));
  return `web-${n}`;
}
