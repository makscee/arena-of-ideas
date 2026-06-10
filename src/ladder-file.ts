// File backing for the LadderStore interface — JSON on disk, for the CLI.
// Kept out of ladder.ts (and index.ts, which the browser imports) so the
// kernel modules stay free of node built-ins; CLI-side callers import this
// module directly, the cli.ts way.
//
// The store semantics and the serialized shape (LadderData) live in ladder.ts
// as PersistedLadderStore — shared with the web client's localStorage backing.
// This module owns only the file medium: one file holds the whole ladder and
// every mutation rewrites it synchronously. A ladder file is small (teams,
// not battle logs) and the CLI is the only writer, so plain write-through
// beats anything cleverer here.

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { emptyLadderData, parseLadderData, PersistedLadderStore } from "./ladder.js";
import type { LadderData } from "./ladder.js";

export class FileLadderStore extends PersistedLadderStore {
  /** Open the ladder at `path`, creating an empty one (and its directory) if
   * the file does not exist. A present-but-unreadable file throws loudly —
   * silently starting a fresh ladder would orphan every ghost in the old one. */
  constructor(path: string) {
    super(load(path), (data) => writeFileSync(path, JSON.stringify(data, null, 2) + "\n", "utf8"));
  }
}

function load(path: string): LadderData {
  let raw: string;
  try {
    raw = readFileSync(path, "utf8");
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code !== "ENOENT") {
      throw new Error(`Cannot read ladder file "${path}": ${(err as Error).message}`);
    }
    mkdirSync(dirname(path), { recursive: true });
    return emptyLadderData();
  }
  return parseLadderData(raw, `file "${path}"`);
}
