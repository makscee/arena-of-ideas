// The season archive — an immutable, append-only record of finished seasons.
// A season runs on one content version and one live tower; when a season ends,
// its FINAL leaderboard (the full tower at that moment) is archived here, and
// never mutated. Season history is kept from day one: a completed-season record
// is written once and only read thereafter — no update, no delete.
//
// This module owns the storage boundary for that history: the
// SeasonArchiveStore interface and its in-memory backing (tests, browser). The
// file backing lives in season-archive-file.ts so this module — and index.ts,
// which the browser imports — stays free of node built-ins. It mirrors the
// LadderStore pattern (ladder.ts) deliberately: one serialized shape every
// backing round-trips byte-equivalent, jsonClone on every read AND write so a
// returned record is detached and a stored one is isolated, and a loud throw on
// corrupt JSON rather than a silent fresh archive.
//
// The embedded final-tower snapshot REUSES #075's serialized ladder shape
// (LadderData: bosses keyed by floor, pools keyed by round) verbatim — the
// archive and the live tower stay one shape, so a snapshot round-trips exactly
// what the ladder serialized.
//
// What this slice does NOT do: it does not run the season transition (that op,
// which BUMPS the version and seeds the next live tower, is slice 2) nor any UI
// (slice 3). Here the store only DEFINES the record + version stamp and STAMPS
// each archived season with the version it ran on.

import { emptyLadderData, jsonClone } from "./ladder.js";
import type { LadderData } from "./ladder.js";

/** A content-version stamp: which content version a season ran on. A small
 * typed integer (1, 2, 3, …) — monotonic, comparable, JSON-trivial. The op that
 * BUMPS it on a season transition is slice 2; here it is only defined + stamped.
 * The very first season runs on version FIRST_CONTENT_VERSION. */
export type ContentVersion = number;

/** The content version the first-ever season runs on. */
export const FIRST_CONTENT_VERSION: ContentVersion = 1;

/** A completed-season record — written once at season end, immutable thereafter.
 * Holds the season number, the content version it ran on, and the FINAL tower
 * snapshot (the full #075 LadderData: every floor's boss + every round's pool),
 * so the season's last leaderboard is preserved exactly as it stood. */
export interface SeasonRecord {
  /** Which season this is — 1, 2, 3, … in completion order. */
  season: number;
  /** The content version the season ran on (the stamp). */
  version: ContentVersion;
  /** The final tower at season end — the live ladder's serialized shape,
   * frozen. Reuses #075's LadderData so it round-trips exactly. */
  finalTower: LadderData;
}

/** The storage boundary the season clock depends on — nothing else. Backings:
 * InMemorySeasonArchiveStore (below), FileSeasonArchiveStore
 * (season-archive-file.ts). The store is APPEND-ONLY: archive() writes a
 * finished season, list()/seasonAt() read; there is NO update and NO delete, by
 * design — season history is never mutated. Returned records are detached deep
 * copies (clone on read), so a caller mutating one cannot corrupt the store. */
export interface SeasonArchiveStore {
  /** Append a completed-season record. record.season must be list().length + 1
   * — a backing throws on a gap or a repeat, because a wrong season number means
   * the caller is archiving out of order or re-archiving a finished season (the
   * history would no longer be the dense, append-once sequence it must stay). */
  archive(record: SeasonRecord): void;
  /** Every archived season in completion order — a detached deep copy, so
   * mutating it cannot reach the store. [] for an untouched archive. */
  list(): SeasonRecord[];
  /** The season numbered `n` (1-based), or null if it has not been archived.
   * A detached deep copy, like list(). */
  seasonAt(n: number): SeasonRecord | null;
}

/** The in-memory backing — tests now, the browser client later. Records are
 * jsonClone'd on write so a stored record is exactly what the file backing would
 * round-trip (isolated from later caller mutation, byte-equivalent across
 * backings), and jsonClone'd again on read so a returned record is detached. */
export class InMemorySeasonArchiveStore implements SeasonArchiveStore {
  private records: SeasonRecord[] = [];

  archive(record: SeasonRecord): void {
    assertSeasonInOrder(record, this.records.length);
    this.records.push(jsonClone(record));
  }

  list(): SeasonRecord[] {
    return jsonClone(this.records);
  }

  seasonAt(n: number): SeasonRecord | null {
    const record = this.records[n - 1];
    return record === undefined ? null : jsonClone(record);
  }
}

/** The serialized season archive — the one shape every persistent backing
 * stores: the seasons in completion order. (A bare object wrapping the array, so
 * the top level is an object the parser can validate by shape, like LadderData.)
 * FileSeasonArchiveStore writes it to disk; the web client writes it to
 * localStorage; both round-trip byte-equivalent archives. */
export interface SeasonArchiveData {
  seasons: SeasonRecord[];
}

/** A fresh, empty archive — what a backing starts from when nothing is stored. */
export function emptySeasonArchiveData(): SeasonArchiveData {
  return { seasons: [] };
}

/** Parse stored archive JSON, loudly: a present-but-unreadable archive must
 * never silently become a fresh one — that would erase every finished season's
 * history, the one thing this store exists to keep. `label` names the backing in
 * the error (a file path, a storage key). */
export function parseSeasonArchiveData(raw: string, label: string): SeasonArchiveData {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (err) {
    throw new Error(`Season archive ${label} is not valid JSON: ${(err as Error).message}`);
  }
  const data = parsed as Partial<SeasonArchiveData>;
  if (typeof data !== "object" || data === null || !Array.isArray(data.seasons)) {
    throw new Error(`Season archive ${label} has no seasons array — not a season archive`);
  }
  return { seasons: data.seasons };
}

/** Serialize an archive to its stored JSON. Pretty-printed with a trailing
 * newline, the way the file backing writes it, so a serialize → parse round-trip
 * is byte-stable and a stored archive is human-legible. */
export function serializeSeasonArchive(data: SeasonArchiveData): string {
  return JSON.stringify(data, null, 2) + "\n";
}

/** A SeasonArchiveStore over a SeasonArchiveData record with a write-through
 * persist hook — the shared engine of every persistent backing (file,
 * localStorage): same clone-on-write/clone-on-read isolation as
 * InMemorySeasonArchiveStore, plus persist(data) after every append. The hook
 * owns serialization and the medium; this class owns the SeasonArchiveStore
 * semantics, so backings can never disagree on them. */
export class PersistedSeasonArchiveStore implements SeasonArchiveStore {
  private readonly data: SeasonArchiveData;
  private readonly persist: (data: SeasonArchiveData) => void;

  constructor(data: SeasonArchiveData, persist: (data: SeasonArchiveData) => void) {
    this.data = data;
    this.persist = persist;
  }

  archive(record: SeasonRecord): void {
    assertSeasonInOrder(record, this.data.seasons.length);
    // Clone on write, like InMemorySeasonArchiveStore: holding the caller's
    // object by reference would let a later mutation corrupt the stored record
    // and the next persist would write the corruption through.
    this.data.seasons.push(jsonClone(record));
    this.persist(this.data);
  }

  list(): SeasonRecord[] {
    return jsonClone(this.data.seasons);
  }

  seasonAt(n: number): SeasonRecord | null {
    const record = this.data.seasons[n - 1];
    return record === undefined ? null : jsonClone(record);
  }
}

/** A fresh, empty final-tower snapshot — an empty LadderData, for a season that
 * somehow ends with an untouched tower. Re-exported here so archive callers need
 * not reach into the ladder module for the empty shape. */
export function emptyFinalTower(): LadderData {
  return emptyLadderData();
}

/** The season-number precondition, enforced (SeasonArchiveStore.archive) —
 * shared by both backings. The archive is a dense, append-once sequence: season
 * N is archived exactly when N-1 already are, so its number must be the next
 * ordinal. A gap or a repeat means the caller archived out of order or twice. */
export function assertSeasonInOrder(record: SeasonRecord, archivedCount: number): void {
  const expected = archivedCount + 1;
  if (record.season !== expected) {
    throw new Error(
      `season ${record.season} desyncs from the archive (next season is ${expected})`,
    );
  }
}
