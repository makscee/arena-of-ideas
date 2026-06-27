// The season clock — the live season pointer and the season-transition op.
// The season is the single clock: one season runs on one content version and
// one live tower. The transition is the MANUAL season-roll (v1: Maks triggers
// it after shipping the top-N voted ideas as a new content version) — the one
// moment a content version changes, and it does so only against an EMPTY,
// just-reset tower. In order it ARCHIVES the final tower, RESETS the live tower,
// BUMPS the content version, and OPENS a fresh season on the new version.
//
// This module owns two things the season clock needs and the stores beside it
// (ladder.ts, season-archive.ts) do not provide:
//
//   1. The live season POINTER — the current season number + current content
//      version. The archive only stamps the version onto a finished record
//      (season-archive.ts, slice 1); the pointer is the live cursor the
//      transition reads and advances. It mirrors the store pattern verbatim: an
//      in-memory backing, a persisted engine over a serialized SeasonPointerData
//      shape that round-trips byte-equivalent, clone-on-read/write isolation,
//      and a loud throw on corrupt JSON.
//
//   2. The season-TRANSITION operation itself — the archive→reset→bump→open
//      ordering, with the version-boundary invariant guaranteed by construction.
//
// No node built-ins here (index.ts imports this for the browser): the file
// backing lives in season-file.ts, the ladder/season-archive way.

import { jsonClone } from "./ladder.js";
import { FIRST_CONTENT_VERSION } from "./season-archive.js";
import type { LadderData, LadderStore } from "./ladder.js";
import type { ContentVersion, SeasonArchiveStore, SeasonRecord } from "./season-archive.js";

/** The live season cursor: which season is running and on which content
 * version. The archive stamps the version onto a FINISHED record; this is the
 * LIVE pointer the transition reads and advances. The first-ever season is
 * season 1 on FIRST_CONTENT_VERSION (FIRST_SEASON / FIRST_CONTENT_VERSION). */
export interface SeasonPointer {
  /** The current season — 1, 2, 3, … in start order. */
  season: number;
  /** The content version the current season runs on. */
  version: ContentVersion;
}

/** The first-ever season number. The first season runs on FIRST_CONTENT_VERSION
 * (season-archive.ts); together they are the fresh pointer (emptySeasonPointer). */
export const FIRST_SEASON = 1;

/** A fresh live pointer — season 1 on the first content version, what a backing
 * starts from before any transition has run. */
export function emptySeasonPointer(): SeasonPointer {
  return { season: FIRST_SEASON, version: FIRST_CONTENT_VERSION };
}

// ---------------------------------------------------------------------------
// The live season pointer store — same backing pattern as LadderStore /
// SeasonArchiveStore: an interface, an in-memory backing, a persisted engine,
// one serialized shape that round-trips.
// ---------------------------------------------------------------------------

/** The storage boundary for the live season pointer the transition reads and
 * advances. Backings: InMemorySeasonPointerStore (below), FileSeasonPointerStore
 * (season-file.ts). The returned pointer is a detached deep copy (clone on
 * read), so a caller mutating it cannot corrupt the store. */
export interface SeasonPointerStore {
  /** The live pointer — a detached deep copy. A fresh store reads the empty
   * pointer (season 1, first version), never null: a season is always running. */
  get(): SeasonPointer;
  /** Overwrite the live pointer (the transition's bump-and-advance). Stored by
   * a clone, so a later caller mutation cannot reach the store. */
  set(pointer: SeasonPointer): void;
}

/** The in-memory backing — tests now, the browser client later. The pointer is
 * jsonClone'd on write and on read, so a stored pointer is isolated from later
 * caller mutation and a returned one is detached (the LadderStore pattern). */
export class InMemorySeasonPointerStore implements SeasonPointerStore {
  private pointer: SeasonPointer = emptySeasonPointer();

  get(): SeasonPointer {
    return jsonClone(this.pointer);
  }

  set(pointer: SeasonPointer): void {
    this.pointer = jsonClone(pointer);
  }
}

/** The serialized live pointer — the one shape every persistent backing stores
 * (a bare object so the parser can validate it by shape, like LadderData /
 * SeasonArchiveData). FileSeasonPointerStore writes it to disk; the web client
 * writes it to localStorage; both round-trip byte-equivalent pointers. */
export interface SeasonPointerData {
  season: number;
  version: ContentVersion;
}

/** A fresh, empty pointer state — what a backing starts from when nothing is
 * stored (season 1 on the first content version). */
export function emptySeasonPointerData(): SeasonPointerData {
  return emptySeasonPointer();
}

/** Parse stored pointer JSON, loudly: a present-but-unreadable pointer must
 * never silently reset to season 1 — that would re-archive a finished season
 * and rewind the content version. `label` names the backing in the error. */
export function parseSeasonPointerData(raw: string, label: string): SeasonPointerData {
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (err) {
    throw new Error(`Season pointer ${label} is not valid JSON: ${(err as Error).message}`);
  }
  const data = parsed as Partial<SeasonPointerData>;
  if (
    typeof data !== "object" ||
    data === null ||
    typeof data.season !== "number" ||
    typeof data.version !== "number"
  ) {
    throw new Error(`Season pointer ${label} has no season/version — not a season pointer`);
  }
  return { season: data.season, version: data.version };
}

/** Serialize a pointer to its stored JSON — pretty-printed with a trailing
 * newline, the way the file backings write, so a serialize → parse round-trip is
 * byte-stable and a stored pointer is human-legible. */
export function serializeSeasonPointer(data: SeasonPointerData): string {
  return JSON.stringify(data, null, 2) + "\n";
}

/** A SeasonPointerStore over a SeasonPointerData record with a write-through
 * persist hook — the shared engine of every persistent backing (file,
 * localStorage), the PersistedLadderStore / PersistedSeasonArchiveStore way:
 * clone-on-read/write isolation, plus persist(data) after every set. */
export class PersistedSeasonPointerStore implements SeasonPointerStore {
  private data: SeasonPointerData;
  private readonly persist: (data: SeasonPointerData) => void;

  constructor(data: SeasonPointerData, persist: (data: SeasonPointerData) => void) {
    this.data = data;
    this.persist = persist;
  }

  get(): SeasonPointer {
    return jsonClone(this.data);
  }

  set(pointer: SeasonPointer): void {
    this.data = jsonClone(pointer);
    this.persist(this.data);
  }
}

// ---------------------------------------------------------------------------
// Reading a full final-tower snapshot out of a live LadderStore
// ---------------------------------------------------------------------------

/** How far above the season-start tower a snapshot scans for grown floors. A
 * played-on tower grows ABOVE TOWER_HEIGHT crown by crown (ladder.ts: an ascend
 * seats one floor higher), so a snapshot cannot stop at the first vacant floor —
 * it must reach the grown summit. This bound is generous: a run advances at most
 * one floor per fight and a run survives STARTING_LIVES losses, so the summit
 * cannot outrun a wide scan. The scan reads every occupied floor/round within
 * it; nothing above a real game's summit is ever occupied. */
const SNAPSHOT_SCAN_LIMIT = 1024;

/** Read the FULL live tower out of a LadderStore as a #075 LadderData — every
 * seated boss (keyed by floor) and every non-empty pool (keyed by round),
 * exactly the shape season-archive.ts embeds as a SeasonRecord.finalTower. The
 * returned data is detached (deep copies of what the store hands back), so the
 * archive owns its own snapshot independent of the live tower it came from.
 *
 * It scans floors/rounds 1..SNAPSHOT_SCAN_LIMIT rather than stopping at the
 * first gap: a played-on tower can have a vacant floor below an occupied one
 * (a run that overshot left a high pool with no boss; an ascend seats above a
 * still-seated lower boss), so a stop-at-first-gap scan would truncate the
 * snapshot. Within a real game's summit the bound never bites. */
export function snapshotLadder(store: LadderStore): LadderData {
  const bosses: LadderData["bosses"] = {};
  const pools: LadderData["pools"] = {};
  for (let floor = 1; floor <= SNAPSHOT_SCAN_LIMIT; floor++) {
    const boss = store.bossAt(floor);
    if (boss !== null) bosses[String(floor)] = jsonClone(boss);
    const pool = store.poolAt(floor);
    if (pool.length > 0) pools[String(floor)] = jsonClone([...pool]);
  }
  return { bosses, pools };
}

// ---------------------------------------------------------------------------
// The season transition — archive → reset → bump → open
// ---------------------------------------------------------------------------

/** What the transition needs to roll the season. The caller owns store
 * construction and the medium (CLI files, web localStorage); the transition
 * owns only the ORDER and the version-boundary invariant.
 *
 *  - `live` is the season's final tower, snapshotted before the reset.
 *  - `archive` receives the finished season's record (the OLD version + that
 *    final snapshot).
 *  - `pointer` is the live cursor: read for the old season/version, advanced to
 *    the next season on the bumped version.
 *  - `reset` wipes the live tower and re-opens a FRESH bootstrap on it (the
 *    caller reuses #075's seedBootstrapTower so a fresh season starts seeded). It runs
 *    AFTER the snapshot is archived, so the archived tower is the pre-reset one
 *    and the new live tower carries no prior-season ghost.
 *  - `bumpVersion` advances the content version; default is +1 (the next dense
 *    version). A caller that ships to a specific version can override it, but it
 *    must return a STRICTLY GREATER version — the content clock only moves
 *    forward (a non-increment throws, below). */
export interface SeasonTransitionOps {
  live: LadderStore;
  archive: SeasonArchiveStore;
  pointer: SeasonPointerStore;
  reset: () => void;
  bumpVersion?: (current: ContentVersion) => ContentVersion;
}

/** What a completed transition reports back: the record that was archived and
 * the new live pointer (post-bump, post-advance). Both are detached copies. */
export interface SeasonTransitionResult {
  archived: SeasonRecord;
  pointer: SeasonPointer;
}

/** Roll the season: ARCHIVE the final tower, RESET the live tower to a fresh
 * bootstrap, BUMP the content version, OPEN the next season on it. This is the
 * ONLY path that changes the content version, and it changes it strictly in
 * this order, so the version-boundary invariant holds BY CONSTRUCTION:
 *
 *   1. ARCHIVE — snapshot the live tower NOW (before any reset) and append it as
 *      the finished season's record, stamped with the OLD version. The append is
 *      append-only (assertSeasonInOrder): the record's season must be the next
 *      ordinal, or the archive throws and NOTHING after this runs.
 *   2. RESET — wipe the live tower and re-bootstrap it fresh (caller's `reset`).
 *      After this the live tower carries no prior-season ghost: every ghost that
 *      could reference a soon-to-change unit is gone.
 *   3. BUMP — advance the content version. This is the FIRST line that touches
 *      the version, and it runs only AFTER the tower is empty/just-reset, so a
 *      content change NEVER lands on a tower holding live ghosts. The bump must
 *      strictly increase (a non-increment throws): the content clock is monotonic.
 *   4. OPEN — advance the live pointer to the next season on the bumped version.
 *
 * The version-boundary invariant — "a content change only ever lands on an empty
 * tower; no live ghost references a changed unit mid-season" — is exactly this
 * ordering: the bump (step 3) is unreachable until the archive (step 1) and the
 * reset (step 2) have run, so there is NO path that bumps the version with the
 * prior season's ghosts still live. A mutant that bumps before archiving or
 * before resetting reorders these steps and a transition test reddens.
 *
 * The transition is all-or-nothing on its first failable step: if the archive
 * append throws (a desynced season number), the reset/bump/open never run and
 * the live tower + pointer are untouched — a failed roll leaves the prior season
 * intact rather than half-rolled. */
export function transitionSeason(ops: SeasonTransitionOps): SeasonTransitionResult {
  const { live, archive, pointer, reset, bumpVersion } = ops;
  const current = pointer.get();

  // PRECONDITION — compute and validate the next version up front, BEFORE any
  // store mutation. A non-increasing bump (the content clock is monotonic) is a
  // bad request, so it must throw with NOTHING written: archiving then failing
  // the bump would be a half-roll (a finished season recorded, the tower reset,
  // but the pointer never advanced). Validating here keeps the op all-or-nothing.
  // This computes the version but does NOT change it — the version only CHANGES
  // at step 3 (pointer.set), after archive + reset, so the version-boundary
  // invariant (a content change lands only on an empty tower) still holds.
  const nextVersion = (bumpVersion ?? ((v) => v + 1))(current.version);
  if (!(nextVersion > current.version)) {
    throw new Error(
      `season transition: content version must strictly increase (was ${current.version}, got ${nextVersion})`,
    );
  }

  // 1. ARCHIVE — the final tower, stamped with the OLD version, as the finished
  //    season's record. assertSeasonInOrder (inside archive) throws on a desync
  //    BEFORE anything below runs, so a desynced roll is a no-op too.
  const finalTower = snapshotLadder(live);
  const archived: SeasonRecord = { season: current.season, version: current.version, finalTower };
  archive.archive(archived);

  // 2. RESET — wipe + re-bootstrap the live tower. The archived snapshot above is
  //    the PRE-reset tower; the live tower below is a fresh season's seed.
  reset();

  // 3. BUMP + OPEN — advance the live pointer to the next season on the bumped
  //    version. This is the only line that CHANGES the content version, and it is
  //    unreachable until archive + reset have run, so a content change never lands
  //    on a tower holding live ghosts: that is the version-boundary invariant.
  const next: SeasonPointer = { season: current.season + 1, version: nextVersion };
  pointer.set(next);
  return { archived, pointer: next };
}
