// Season history — the READ surface over the season archive (PRD #077 slice 3).
// The archive store (season-archive.ts) keeps every finished season; this module
// turns its records into what a player reads: a list of completed seasons (season
// number + content version + a one-line champion summary) and a single season's
// final tower, rendered floor by floor. No writes, no transition — the store owns
// the storage boundary, this owns the presentation, so the CLI and the web view
// can never disagree on what a season "reads as".
//
// The champion of a season is DERIVED from its archived final tower the one way
// every backing derives it (deriveChampion over the tower's bosses) — the summit
// of the frozen ladder. A season with an empty tower (no boss seated) has no
// champion; its summary says so rather than inventing one.

import { deriveChampion } from "./ladder.js";
import type { TeamSnapshot } from "./ladder.js";
import type { SeasonRecord } from "./season-archive.js";

/** One season's read-line: its number, the content version it ran on, and a
 * one-line champion summary (the final tower's top-floor boss). Everything a
 * history list shows per row — the shaped read, not the raw record. */
export interface SeasonSummary {
  season: number;
  version: number;
  /** The season champion: the boss of the final tower's highest floor, or null
   * when the archived tower seated no boss. */
  champion: TeamSnapshot | null;
}

/** The champion of an archived season — the summit of its frozen final tower,
 * derived exactly the way the live ladder derives its champion. null when the
 * archived tower has no seated boss. */
export function seasonChampion(record: SeasonRecord): TeamSnapshot | null {
  return deriveChampion(record.finalTower.bosses);
}

/** Shape one archived record into its history read-line. */
export function summarizeSeason(record: SeasonRecord): SeasonSummary {
  return { season: record.season, version: record.version, champion: seasonChampion(record) };
}

/** Shape a whole archive (store.list() output) into history read-lines, in the
 * completion order the store hands them back. */
export function summarizeSeasons(records: readonly SeasonRecord[]): SeasonSummary[] {
  return records.map(summarizeSeason);
}

/** A one-line label for a season champion — the run that holds the top floor and
 * the team it fields. A vacant summit reads "(no champion)" so an empty archived
 * tower never looks like a missing render. */
export function championLabel(champion: TeamSnapshot | null): string {
  if (champion === null) return "(no champion)";
  const team = champion.team.map((u) => u.name).join(", ");
  return `${champion.runId} — ${team}`;
}

/** One history list line, human-readable: the season number, the version it ran
 * on, and its champion. The CLI prints these; the web view renders the same
 * fields. */
export function formatSeasonLine(summary: SeasonSummary): string {
  return `Season ${summary.season} (content v${summary.version}): ${championLabel(summary.champion)}`;
}

/** The whole history list — one line per completed season, in completion order.
 * The empty-archive case reads as its own line rather than blank output, so a
 * fresh archive is legibly empty, not silently so. */
export function formatHistoryList(records: readonly SeasonRecord[]): string {
  if (records.length === 0) return "No seasons archived yet.";
  return summarizeSeasons(records).map(formatSeasonLine).join("\n");
}

/** One archived season's final tower, rendered floor by floor from the top down
 * (champion first), each floor naming its boss's run and team. The header names
 * the season and its version; the champion floor is marked. Reads the archived
 * LadderData directly — the frozen leaderboard, exactly as it stood at season
 * end. */
export function formatFinalTower(record: SeasonRecord): string {
  const champion = seasonChampion(record);
  const floors = Object.keys(record.finalTower.bosses)
    .map(Number)
    .sort((a, b) => b - a); // top floor (champion) first
  const lines = [
    `Season ${record.season} (content v${record.version}) — final tower`,
    `Champion: ${championLabel(champion)}`,
  ];
  if (floors.length === 0) {
    lines.push("  (the tower seated no boss)");
    return lines.join("\n");
  }
  for (const floor of floors) {
    const boss = record.finalTower.bosses[String(floor)]!;
    const team = boss.team.map((u) => u.name).join(", ");
    const crown = floor === floors[0] ? " ★ champion" : "";
    lines.push(`  Floor ${floor}: ${boss.runId} — ${team}${crown}`);
  }
  return lines.join("\n");
}
