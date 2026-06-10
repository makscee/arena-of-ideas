// File backing for the LadderStore interface — JSON on disk, for the CLI.
// Kept out of ladder.ts (and index.ts, which the browser imports) so the
// kernel modules stay free of node built-ins; CLI-side callers import this
// module directly, the cli.ts way.
//
// One file holds the whole ladder; every mutation rewrites it synchronously.
// A ladder file is small (teams, not battle logs) and the CLI is the only
// writer, so plain write-through beats anything cleverer here.

import { mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { dirname } from "node:path";
import { assertSeqInOrder, jsonClone } from "./ladder.js";
import type { LadderStore, TeamSnapshot } from "./ladder.js";

/** The on-disk shape — pools keyed by round number (JSON keys are strings). */
interface LadderFile {
  champion: TeamSnapshot | null;
  pools: Record<string, TeamSnapshot[]>;
}

export class FileLadderStore implements LadderStore {
  private readonly path: string;
  private data: LadderFile;

  /** Open the ladder at `path`, creating an empty one (and its directory) if
   * the file does not exist. A present-but-unreadable file throws loudly —
   * silently starting a fresh ladder would orphan every ghost in the old one. */
  constructor(path: string) {
    this.path = path;
    this.data = load(path);
  }

  poolAt(round: number): readonly TeamSnapshot[] {
    return this.data.pools[String(round)] ?? [];
  }

  addSnapshot(snap: TeamSnapshot): void {
    const pool = (this.data.pools[String(snap.round)] ??= []);
    assertSeqInOrder(snap, pool.length);
    // Clone on write, like InMemoryLadderStore: holding the caller's object by
    // reference would let a later mutation corrupt the stored ghost and the
    // next persist would write the corruption to disk.
    pool.push(jsonClone(snap));
    this.persist();
  }

  champion(): TeamSnapshot | null {
    return this.data.champion;
  }

  setChampion(snap: TeamSnapshot): void {
    this.data.champion = jsonClone(snap);
    this.persist();
  }

  private persist(): void {
    writeFileSync(this.path, JSON.stringify(this.data, null, 2) + "\n", "utf8");
  }
}

function load(path: string): LadderFile {
  let raw: string;
  try {
    raw = readFileSync(path, "utf8");
  } catch (err) {
    if ((err as NodeJS.ErrnoException).code !== "ENOENT") {
      throw new Error(`Cannot read ladder file "${path}": ${(err as Error).message}`);
    }
    mkdirSync(dirname(path), { recursive: true });
    return { champion: null, pools: {} };
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (err) {
    throw new Error(`Ladder file "${path}" is not valid JSON: ${(err as Error).message}`);
  }
  const data = parsed as Partial<LadderFile>;
  if (typeof data !== "object" || data === null || typeof data.pools !== "object" || data.pools === null) {
    throw new Error(`Ladder file "${path}" has no pools object — not a ladder file`);
  }
  return { champion: data.champion ?? null, pools: data.pools };
}
