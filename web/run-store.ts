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
  /** The replay position the viewer last held (#015 slice 4): a hard reload
   * mid-battle resumes the parked event, not event 0. Absent on a battle
   * stored before its first unmount/pagehide — resume starts fresh then. */
  position?: number;
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

/** Delete the stored ladder — every ghost and the champion. The explicit
 * destructive way out of a corrupt ladder (which is refused loudly and would
 * otherwise dead-end the run screen until localStorage is hand-cleared);
 * callers own the confirm step. The active run goes with it (clearRun): a run
 * mid-climb on a deleted ladder would fight pools that no longer exist. */
export function resetLadder(storage: KVStorage): void {
  storage.removeItem(LADDER_KEY);
  clearRun(storage);
}

/** The next run's id — "web-N" off a persisted counter, so runs sharing this
 * browser's ladder stay distinct (own-ghost exclusion is by runId) without a
 * wall-clock read.
 *
 * If the stored counter is corrupt (not an integer), falls back to the highest
 * web-N already on the ladder plus one, or 1 if the ladder is empty — so a
 * hand-corrupted RUN_SEQ_KEY never yields "web-NaN". */
export function nextRunId(storage: KVStorage): string {
  const stored = Number(storage.getItem(RUN_SEQ_KEY) ?? "0");
  let base: number;
  if (Number.isInteger(stored)) {
    base = stored;
  } else {
    // Corrupt counter: scan existing ladder run ids for the highest web-N.
    const ladderRaw = storage.getItem(LADDER_KEY);
    let maxSeen = 0;
    if (ladderRaw !== null) {
      for (const m of ladderRaw.matchAll(/"web-(\d+)"/g)) {
        const n = Number(m[1]);
        if (n > maxSeen) maxSeen = n;
      }
    }
    base = maxSeen;
  }
  const n = base + 1;
  storage.setItem(RUN_SEQ_KEY, String(n));
  return `web-${n}`;
}
