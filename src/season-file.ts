// File backing for the SeasonPointerStore — JSON on disk, for the CLI.
// Kept out of season.ts (and index.ts, which the browser imports) so the kernel
// modules stay free of node built-ins; CLI-side callers import this module
// directly, the ladder-file.ts / season-archive-file.ts way.
//
// The store semantics and the serialized shape (SeasonPointerData) live in
// season.ts as PersistedSeasonPointerStore — shared with the web client's
// localStorage backing. This module owns only the file medium: one file holds
// the live pointer and every set rewrites it synchronously. The pointer moves
// once per season (one transition), so plain write-through is more than enough.

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import {
  emptySeasonPointerData,
  parseSeasonPointerData,
  PersistedSeasonPointerStore,
  serializeSeasonPointer,
} from "./season.js";
import type { SeasonPointerData } from "./season.js";

export class FileSeasonPointerStore extends PersistedSeasonPointerStore {
  /** Open the pointer at `path`, creating an empty one (season 1, first version,
   * and its directory) if the file does not exist. A present-but-unreadable file
   * throws loudly — silently resetting to season 1 would re-archive a finished
   * season and rewind the content version. */
  constructor(path: string) {
    super(load(path), (data) => writeFileSync(path, serializeSeasonPointer(data), "utf8"));
  }
}

function load(path: string): SeasonPointerData {
  let raw: string;
  try {
    raw = readFileSync(path, "utf8");
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code !== "ENOENT") {
      throw new Error(`Cannot read season pointer file "${path}": ${(err as Error).message}`);
    }
    mkdirSync(dirname(path), { recursive: true });
    return emptySeasonPointerData();
  }
  return parseSeasonPointerData(raw, `file "${path}"`);
}
