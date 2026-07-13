import { asc, eq } from "drizzle-orm";
import { type ApprovedRegistry, type SeasonRecord } from "../../src/index.js";
import { parseArenaContentSnapshot, type ArenaContent } from "./content.js";
import type { DB } from "./db.js";
import { contentVersions, seasonArchives, seasonState } from "./schema.js";

export interface ServerSeasonPointer { season: number; contentVersion: number }
export interface ActiveContent extends ArenaContent {
  approvedRegistry: ApprovedRegistry;
}

function parseJson<T>(raw: string, label: string): T {
  try { return JSON.parse(raw) as T; }
  catch (err) { throw new Error(`${label} is corrupt JSON: ${(err as Error).message}`); }
}

export function readSeasonPointer(db: DB): ServerSeasonPointer {
  const rows = db.select().from(seasonState).where(eq(seasonState.id, 1)).all();
  if (rows.length !== 1) throw new Error("corrupt season_state: expected exactly one authoritative pointer");
  return { season: rows[0]!.season, contentVersion: rows[0]!.contentVersion };
}

export function readContentVersion(db: DB, version: number): ActiveContent {
  const row = db.select().from(contentVersions).where(eq(contentVersions.version, version)).all()[0];
  if (row === undefined) throw new Error(`content version ${version} is missing`);
  return parseArenaContentSnapshot(row, `content v${version}`);
}

export function readActiveContent(db: DB): ActiveContent {
  return readContentVersion(db, readSeasonPointer(db).contentVersion);
}

export interface PublicArchive extends SeasonRecord {
  selectionReceipt: unknown;
  outcomeReceipt: unknown;
}

export function listServerArchives(db: DB): PublicArchive[] {
  return db.select().from(seasonArchives).orderBy(asc(seasonArchives.season)).all().map((row) => ({
    season: row.season,
    version: row.contentVersion,
    finalTower: parseJson(row.finalTower, `season ${row.season} final tower`),
    selectionReceipt: parseJson(row.selectionReceipt, `season ${row.season} selection receipt`),
    outcomeReceipt: parseJson(row.outcomeReceipt, `season ${row.season} outcome receipt`),
  }));
}
