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
  PersistedSeasonArchiveStore,
  deserializeRun,
  emptyLadderData,
  emptySeasonArchiveData,
  parseLadderData,
  parseSeasonArchiveData,
  serializeRun,
  serializeSeasonArchive,
  type LadderStore,
  type RunState,
  type SeasonArchiveStore,
  type UnitDef,
} from "../src/index.js";

const LADDER_KEY = "aoi.ladder.v1";
const ARCHIVE_KEY = "aoi.season-archive.v1";
const RUN_KEY = "aoi.run.v1";
const BATTLE_KEY = "aoi.run-battle.v1";
const RUN_SEQ_KEY = "aoi.run-seq.v1";
const SUBMIT_KEY = "aoi.submit.v1";
const SESSION_KEY = "aoi.session.v1";
const DEV_KEY = "aoi.dev.v1";
const LOCAL_ONLY_KEY = "aoi.run-local-only.v1";

/** The storage surface this module needs — window.localStorage, or a test stub. */
export type KVStorage = Pick<Storage, "getItem" | "setItem" | "removeItem">;

/** A key-prefixed view over a storage (#016 slice 3): logged-in play keeps its
 * run under different keys than local play, so logging in never clobbers the
 * local run and a remote run never revives into a logged-out session. The
 * session token itself stays unprefixed — it is what picks the namespace. */
export function prefixedStorage(storage: KVStorage, prefix: string): KVStorage {
  return {
    getItem: (key) => storage.getItem(prefix + key),
    setItem: (key, value) => storage.setItem(prefix + key, value),
    removeItem: (key) => storage.removeItem(prefix + key),
  };
}

// ---------------------------------------------------------------------------
// Session token (#016 slice 3) — the bearer the arena server minted, persisted
// so a reload stays logged in. Stored raw: localStorage is this device's
// session boundary, same as a cookie jar.
// ---------------------------------------------------------------------------

export function loadSession(storage: KVStorage): string | null {
  const token = storage.getItem(SESSION_KEY);
  return token === null || token === "" ? null : token;
}

export function saveSession(storage: KVStorage, token: string): void {
  storage.setItem(SESSION_KEY, token);
}

export function clearSession(storage: KVStorage): void {
  storage.removeItem(SESSION_KEY);
}

// ---------------------------------------------------------------------------
// Dev mode (#066 slice 1) — a local convenience switch, off by default, that
// gates the dev surfaces on this device. Stored raw "1"/absent, same medium as
// the session token: localStorage is this device's boundary. This is NOT a
// security boundary (the arena is client-authoritative; a cheated run is
// structurally unsubmittable) — only which surfaces a developer sees here.
// ---------------------------------------------------------------------------

/** Whether dev mode is on. Off unless the flag is explicitly set — a fresh
 * profile, a cleared key, or any non-"1" value all read as off. */
export function loadDevMode(storage: KVStorage): boolean {
  return storage.getItem(DEV_KEY) === "1";
}

export function setDevMode(storage: KVStorage, on: boolean): void {
  if (on) storage.setItem(DEV_KEY, "1");
  else storage.removeItem(DEV_KEY);
}

/** Open the localStorage-backed ladder. Corrupt stored JSON throws loudly
 * (the FileLadderStore rule: silently starting fresh would orphan every
 * ghost); the caller surfaces the error, never swallows it. */
export function openLocalLadder(storage: KVStorage): LadderStore {
  const raw = storage.getItem(LADDER_KEY);
  const data = raw === null ? emptyLadderData() : parseLadderData(raw, `localStorage "${LADDER_KEY}"`);
  return new PersistedLadderStore(data, (d) => storage.setItem(LADDER_KEY, JSON.stringify(d)));
}

/** Open the localStorage-backed season archive (PRD #077 slice 3) — the same
 * engine and SeasonArchiveData shape FileSeasonArchiveStore writes to disk. The
 * history view reads it; the season transition (slice 2) is what writes it, so
 * until a season ends this is empty on a player's device. Corrupt stored JSON
 * throws loudly (the archive's whole point is never to lose finished history —
 * a silent fresh archive would erase it); the caller surfaces the error. */
export function openLocalArchive(storage: KVStorage): SeasonArchiveStore {
  const raw = storage.getItem(ARCHIVE_KEY);
  const data = raw === null ? emptySeasonArchiveData() : parseSeasonArchiveData(raw, `localStorage "${ARCHIVE_KEY}"`);
  return new PersistedSeasonArchiveStore(data, (d) => storage.setItem(ARCHIVE_KEY, serializeSeasonArchive(d)));
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
  /** #066 slice 4: a dev cheat was applied to this run, so the client skips
   * submission for it (a cheated run can't re-derive — it would only generate a
   * doomed 422). Hygiene, not security: the server is immune regardless. */
  localOnly?: boolean;
}

export function saveRun(storage: KVStorage, state: RunState, battle?: StoredBattle, localOnly?: boolean): void {
  storage.setItem(RUN_KEY, serializeRun(state));
  if (battle !== undefined) storage.setItem(BATTLE_KEY, JSON.stringify(battle));
  else storage.removeItem(BATTLE_KEY);
  if (localOnly) storage.setItem(LOCAL_ONLY_KEY, "1");
  else storage.removeItem(LOCAL_ONLY_KEY);
}

/** The active run, or null when none is stored. Corrupt data throws loudly. */
export function loadRun(storage: KVStorage): StoredRun | null {
  const raw = storage.getItem(RUN_KEY);
  if (raw === null) return null;
  const state = deserializeRun(raw);
  // Only carried when set, so a non-cheated run round-trips to exactly { state }
  // / { state, battle } (the storage shape callers compare against).
  const localOnly = storage.getItem(LOCAL_ONLY_KEY) === "1" ? { localOnly: true } : {};
  const battleRaw = storage.getItem(BATTLE_KEY);
  if (battleRaw === null) return { state, ...localOnly };
  const battle = JSON.parse(battleRaw) as StoredBattle;
  if (!Array.isArray(battle?.teamA) || !Array.isArray(battle?.teamB) || typeof battle?.seed !== "number") {
    throw new Error(`stored battle ("${BATTLE_KEY}") is not a StoredBattle — refusing to revive it`);
  }
  return { state, battle, ...localOnly };
}

export function clearRun(storage: KVStorage): void {
  storage.removeItem(RUN_KEY);
  storage.removeItem(BATTLE_KEY);
  storage.removeItem(SUBMIT_KEY);
  storage.removeItem(LOCAL_ONLY_KEY);
}

// ---------------------------------------------------------------------------
// Submit outcome (#016 slice 3) — the server's verdict on a finished remote
// run, persisted per runId so a reload on the end screen neither re-submits an
// accepted run nor forgets a rejection. Transient failures (offline) are NOT
// stored — those retry.
// ---------------------------------------------------------------------------

/** The durable verdicts: the server accepted (crown confirmed or lapsed in
 * the crown race) or rejected with its reason. */
export interface StoredSubmit {
  runId: string;
  accepted: boolean;
  /** Present when accepted: whether the crown really landed. */
  crowned?: boolean;
  /** Present when rejected: the server's reason. */
  reason?: string;
}

export function saveSubmitResult(storage: KVStorage, result: StoredSubmit): void {
  storage.setItem(SUBMIT_KEY, JSON.stringify(result));
}

/** The stored verdict for `runId`, or null (none stored, another run's, or
 * corrupt — a bad stored verdict must never block a retry). */
export function loadSubmitResult(storage: KVStorage, runId: string): StoredSubmit | null {
  const raw = storage.getItem(SUBMIT_KEY);
  if (raw === null) return null;
  try {
    const parsed = JSON.parse(raw) as Partial<StoredSubmit>;
    if (parsed?.runId !== runId || typeof parsed.accepted !== "boolean") return null;
    return parsed as StoredSubmit;
  } catch {
    return null;
  }
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
