// File backing for the SeasonArchiveStore interface — JSON on disk, for the CLI.
// Kept out of season-archive.ts (and index.ts, which the browser imports) so the
// kernel modules stay free of node built-ins; CLI-side callers import this
// module directly, the ladder-file.ts way.
//
// The store semantics and the serialized shape (SeasonArchiveData) live in
// season-archive.ts as PersistedSeasonArchiveStore — shared with the web
// client's localStorage backing. This module owns only the file medium: one
// file holds the whole archive and every append rewrites it synchronously. An
// archive grows slowly (one record per finished season) and the CLI is the only
// writer, so plain write-through beats anything cleverer here.

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import {
  emptySeasonArchiveData,
  parseSeasonArchiveData,
  PersistedSeasonArchiveStore,
  serializeSeasonArchive,
} from "./season-archive.js";
import type { SeasonArchiveData } from "./season-archive.js";

export class FileSeasonArchiveStore extends PersistedSeasonArchiveStore {
  /** Open the archive at `path`, creating an empty one (and its directory) if
   * the file does not exist. A present-but-unreadable file throws loudly —
   * silently starting a fresh archive would erase every finished season in it. */
  constructor(path: string) {
    super(load(path), (data) => writeFileSync(path, serializeSeasonArchive(data), "utf8"));
  }
}

function load(path: string): SeasonArchiveData {
  let raw: string;
  try {
    raw = readFileSync(path, "utf8");
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code !== "ENOENT") {
      throw new Error(`Cannot read season archive file "${path}": ${(err as Error).message}`);
    }
    mkdirSync(dirname(path), { recursive: true });
    return emptySeasonArchiveData();
  }
  return parseSeasonArchiveData(raw, `file "${path}"`);
}
