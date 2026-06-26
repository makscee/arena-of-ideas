// Season history READ test (PRD #077 slice 3) — the acceptance line:
//   "archived seasons listed + a season's final tower read back".
// Seed the archive with a couple of completed-season records, then assert the
// read path (1) lists them in completion order with version + season number, and
// (2) reads back a specific season's final tower deep-equal to what was archived.
// The store (season-archive.ts) owns storage and is tested there; this exercises
// the READ surface the CLI and web view share — list summaries + the final-tower
// read.

import { describe, expect, test } from "vitest";
import { stressRegistry } from "./content/stress.js";
import { InMemoryLadderStore, deriveChampion, openLadder } from "./ladder.js";
import type { LadderData } from "./ladder.js";
import { FIRST_CONTENT_VERSION, InMemorySeasonArchiveStore } from "./season-archive.js";
import type { SeasonRecord } from "./season-archive.js";
import {
  championLabel,
  formatFinalTower,
  formatHistoryList,
  seasonChampion,
  summarizeSeason,
  summarizeSeasons,
} from "./season-history.js";

// ---------------------------------------------------------------------------
// Helpers — realistic final-tower snapshots out of a seeded ladder (the #075
// LadderData shape), the same way season-archive.test.ts builds them: the
// embedded tower must be exactly what the live ladder stores.
// ---------------------------------------------------------------------------

function towerSnapshot(): LadderData {
  const ladder = openLadder(new InMemoryLadderStore(), stressRegistry);
  const bosses: LadderData["bosses"] = {};
  const pools: LadderData["pools"] = {};
  for (let floor = 1; ladder.bossAt(floor) !== null; floor++) {
    bosses[String(floor)] = ladder.bossAt(floor)!;
  }
  for (let round = 1; ladder.poolAt(round).length > 0; round++) {
    pools[String(round)] = [...ladder.poolAt(round)];
  }
  return { bosses, pools };
}

function record(season: number, version: number, tower: LadderData = towerSnapshot()): SeasonRecord {
  return { season, version, finalTower: tower };
}

// ---------------------------------------------------------------------------
// THE ACCEPTANCE: seed the archive, list the seasons, read back a final tower.
// ---------------------------------------------------------------------------

describe("season history read", () => {
  test("archived seasons are listed in order, each with its season number and version", () => {
    const store = new InMemorySeasonArchiveStore();
    store.archive(record(1, FIRST_CONTENT_VERSION));
    store.archive(record(2, FIRST_CONTENT_VERSION + 1));

    // The read path lists every archived season, in completion order, carrying
    // the season number and the content version it ran on.
    const summaries = summarizeSeasons(store.list());
    expect(summaries.map((s) => s.season)).toEqual([1, 2]);
    expect(summaries.map((s) => s.version)).toEqual([FIRST_CONTENT_VERSION, FIRST_CONTENT_VERSION + 1]);
  });

  test("a specific season's final tower reads back deep-equal to what was archived", () => {
    const store = new InMemorySeasonArchiveStore();
    const tower1 = towerSnapshot();
    const tower2 = towerSnapshot();
    store.archive(record(1, FIRST_CONTENT_VERSION, tower1));
    store.archive(record(2, 5, tower2));

    // Read back season 2's record and assert its final tower is byte-for-byte
    // the LadderData that was archived — the frozen leaderboard, intact.
    const read = store.seasonAt(2)!;
    expect(read.season).toBe(2);
    expect(read.version).toBe(5);
    expect(read.finalTower).toEqual(tower2);
    // …and season 1's, distinct read, equals its own archived tower.
    expect(store.seasonAt(1)!.finalTower).toEqual(tower1);
  });

  test("the empty archive lists as no seasons", () => {
    const store = new InMemorySeasonArchiveStore();
    expect(summarizeSeasons(store.list())).toEqual([]);
    expect(formatHistoryList(store.list())).toBe("No seasons archived yet.");
  });
});

// ---------------------------------------------------------------------------
// Champion summary — the one-line read each season list row carries.
// ---------------------------------------------------------------------------

describe("season champion summary", () => {
  test("a season's champion is the summit of its archived final tower", () => {
    const tower = towerSnapshot();
    const rec = record(1, FIRST_CONTENT_VERSION, tower);
    // Derived the same way the live ladder derives its champion — the top floor's
    // boss of the FROZEN tower.
    expect(seasonChampion(rec)).toEqual(deriveChampion(tower.bosses));
    expect(seasonChampion(rec)).not.toBeNull();
    // The summary carries that champion alongside the number + version.
    const summary = summarizeSeason(rec);
    expect(summary).toEqual({ season: 1, version: FIRST_CONTENT_VERSION, champion: deriveChampion(tower.bosses) });
  });

  test("a season whose archived tower seated no boss has no champion", () => {
    const rec: SeasonRecord = { season: 1, version: FIRST_CONTENT_VERSION, finalTower: { bosses: {}, pools: {} } };
    expect(seasonChampion(rec)).toBeNull();
    expect(championLabel(seasonChampion(rec))).toBe("(no champion)");
  });
});

// ---------------------------------------------------------------------------
// Rendering — the lines the CLI prints (the web view renders the same fields).
// ---------------------------------------------------------------------------

describe("history rendering", () => {
  test("the history list names each season's number, version, and champion", () => {
    const store = new InMemorySeasonArchiveStore();
    store.archive(record(1, FIRST_CONTENT_VERSION));
    store.archive(record(2, 4));
    const text = formatHistoryList(store.list());
    const lines = text.split("\n");
    expect(lines.length).toBe(2);
    expect(lines[0]).toContain("Season 1");
    expect(lines[0]).toContain("content v1");
    expect(lines[1]).toContain("Season 2");
    expect(lines[1]).toContain("content v4");
  });

  test("the final-tower render lists every floor, champion floor first and marked", () => {
    const tower = towerSnapshot();
    const rec = record(3, 2, tower);
    const text = formatFinalTower(rec);
    const lines = text.split("\n");
    expect(lines[0]).toContain("Season 3");
    expect(lines[0]).toContain("content v2");
    expect(text).toContain("★ champion");
    // One floor line per seated boss in the archived tower.
    const floorLines = lines.filter((l) => l.trim().startsWith("Floor "));
    expect(floorLines.length).toBe(Object.keys(tower.bosses).length);
    // Top floor (champion) is printed first among the floor lines.
    const topFloor = Math.max(...Object.keys(tower.bosses).map(Number));
    expect(floorLines[0]).toContain(`Floor ${topFloor}`);
    expect(floorLines[0]).toContain("★ champion");
  });
});
