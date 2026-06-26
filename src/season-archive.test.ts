import { mkdtempSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, test } from "vitest";
import { stressRegistry } from "./content/stress.js";
import { InMemoryLadderStore, openLadder } from "./ladder.js";
import type { LadderData } from "./ladder.js";
import {
  FIRST_CONTENT_VERSION,
  InMemorySeasonArchiveStore,
  PersistedSeasonArchiveStore,
  emptySeasonArchiveData,
  parseSeasonArchiveData,
  serializeSeasonArchive,
} from "./season-archive.js";
import type { SeasonArchiveStore, SeasonRecord } from "./season-archive.js";
import { FileSeasonArchiveStore } from "./season-archive-file.js";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

/** A realistic final-tower snapshot: a seeded ladder (the #075 LadderData shape)
 * read out of an InMemoryLadderStore. Reusing the live ladder's serialized shape
 * is the point — the embedded snapshot must be exactly what the ladder stores. */
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

/** A localStorage-like medium: the last persisted JSON string, behind a store. */
function memoryMedium(): { read(): string | null; store(): SeasonArchiveStore } {
  let raw: string | null = null;
  return {
    read: () => raw,
    store: () =>
      new PersistedSeasonArchiveStore(
        raw === null ? emptySeasonArchiveData() : parseSeasonArchiveData(raw, "memory"),
        (d) => {
          raw = serializeSeasonArchive(d);
        },
      ),
  };
}

// Both backings of the SeasonArchiveStore interface, driven through the same
// cases — the shared semantics must hold identically (the ladder.test.ts way).
const backings: { name: string; fresh: () => SeasonArchiveStore }[] = [
  { name: "InMemory", fresh: () => new InMemorySeasonArchiveStore() },
  {
    name: "Persisted(memory)",
    fresh: () => memoryMedium().store(),
  },
  {
    name: "File",
    fresh: () =>
      new FileSeasonArchiveStore(join(mkdtempSync(join(tmpdir(), "season-")), "archive.json")),
  },
];

// ---------------------------------------------------------------------------
// 1. Append: a season record appears in list / seasonAt, in order
// ---------------------------------------------------------------------------

describe.each(backings)("append [$name]", ({ fresh }) => {
  test("an archived season appears in list() and at seasonAt(n)", () => {
    const store = fresh();
    expect(store.list()).toEqual([]); // a fresh archive is empty
    expect(store.seasonAt(1)).toBeNull();

    const s1 = record(1, FIRST_CONTENT_VERSION);
    store.archive(s1);
    expect(store.list()).toEqual([s1]);
    expect(store.seasonAt(1)).toEqual(s1);

    const s2 = record(2, FIRST_CONTENT_VERSION + 1);
    store.archive(s2);
    expect(store.list()).toEqual([s1, s2]); // completion order preserved
    expect(store.seasonAt(2)).toEqual(s2);
    expect(store.seasonAt(3)).toBeNull(); // not yet archived
  });

  test("archive is append-only by season number — a gap or a repeat throws and lands nowhere", () => {
    const store = fresh();
    store.archive(record(1, FIRST_CONTENT_VERSION));
    // A gap (season 3 when 2 is next) is out-of-order archiving.
    expect(() => store.archive(record(3, FIRST_CONTENT_VERSION))).toThrow(/desyncs/);
    // A repeat (re-archiving season 1) would mutate finished history.
    expect(() => store.archive(record(1, FIRST_CONTENT_VERSION))).toThrow(/desyncs/);
    expect(store.list().length).toBe(1); // neither rejected write landed
  });
});

// ---------------------------------------------------------------------------
// 2. Immutability: there is no mutating method, and returned records are
//    detached — mutating one does NOT corrupt the store (must-fail-first)
// ---------------------------------------------------------------------------

describe.each(backings)("immutability [$name]", ({ fresh }) => {
  test("the interface exposes no update or delete — only archive + the two reads", () => {
    const store = fresh();
    // A typed store has exactly these three methods; nothing mutates history.
    const methods = ["archive", "list", "seasonAt"];
    for (const m of methods) expect(typeof (store as unknown as Record<string, unknown>)[m]).toBe("function");
    // The shape the brief forbids: no update/delete/remove/set on the store.
    for (const forbidden of ["update", "delete", "remove", "set", "replace"]) {
      expect((store as unknown as Record<string, unknown>)[forbidden]).toBeUndefined();
    }
  });

  test("mutating a record handed back by list() does NOT corrupt the store (detached copies)", () => {
    const store = fresh();
    store.archive(record(1, FIRST_CONTENT_VERSION));

    // MUST-FAIL-FIRST: if list() returned the stored record by reference (no
    // clone-on-read), this mutation would reach the store and the re-read below
    // would observe the corruption. The guard is the detached deep copy.
    const handed = store.list()[0]!;
    handed.version = 999;
    handed.season = 42;
    handed.finalTower.bosses["1"]!.team[0]!.name = "Corrupted";

    const reread = store.list()[0]!;
    expect(reread.version).toBe(FIRST_CONTENT_VERSION); // untouched
    expect(reread.season).toBe(1);
    expect(reread.finalTower.bosses["1"]!.team[0]!.name).not.toBe("Corrupted");
    // seasonAt hands back a detached copy too.
    expect(store.seasonAt(1)!.version).toBe(FIRST_CONTENT_VERSION);
  });

  test("mutating the caller's record AFTER archive() does NOT reach the store (clone-on-write)", () => {
    const store = fresh();
    const rec = record(1, FIRST_CONTENT_VERSION);
    store.archive(rec);
    // MUST-FAIL-FIRST: without clone-on-write the store would hold rec by
    // reference and this post-archive mutation would corrupt stored history
    // (and persist the corruption through on the next write).
    rec.version = 999;
    rec.finalTower.bosses["1"]!.team[0]!.name = "Corrupted";
    expect(store.seasonAt(1)!.version).toBe(FIRST_CONTENT_VERSION);
    expect(store.seasonAt(1)!.finalTower.bosses["1"]!.team[0]!.name).not.toBe("Corrupted");
  });
});

// ---------------------------------------------------------------------------
// 3. Version stamp: stored and read back, on the record
// ---------------------------------------------------------------------------

describe.each(backings)("version stamp [$name]", ({ fresh }) => {
  test("the content-version stamp is stored on the record and read back", () => {
    const store = fresh();
    store.archive(record(1, FIRST_CONTENT_VERSION));
    store.archive(record(2, 7)); // a later, bumped version (slice 2 does the bumping)
    expect(store.seasonAt(1)!.version).toBe(FIRST_CONTENT_VERSION);
    expect(store.seasonAt(2)!.version).toBe(7);
    expect(store.list().map((r) => r.version)).toEqual([FIRST_CONTENT_VERSION, 7]);
  });

  test("the first season runs on FIRST_CONTENT_VERSION", () => {
    expect(FIRST_CONTENT_VERSION).toBe(1);
  });
});

// ---------------------------------------------------------------------------
// 4. Serialized round-trip: serialize → parse → deep-equal, embedded tower too
// ---------------------------------------------------------------------------

describe("serialized round-trip", () => {
  test("serialize → parse is deep-equal, including the embedded final-tower snapshot", () => {
    const data = emptySeasonArchiveData();
    data.seasons.push(record(1, FIRST_CONTENT_VERSION));
    data.seasons.push(record(2, 5));
    const round = parseSeasonArchiveData(serializeSeasonArchive(data), "memory");
    expect(round).toEqual(data);
    // The embedded tower is the #075 LadderData shape, preserved exactly.
    expect(round.seasons[0]!.finalTower).toEqual(towerSnapshot());
    expect(round.seasons[0]!.finalTower.bosses).toEqual(towerSnapshot().bosses);
  });

  test("serialize is byte-stable through a round-trip", () => {
    const data = emptySeasonArchiveData();
    data.seasons.push(record(1, FIRST_CONTENT_VERSION));
    const bytes = serializeSeasonArchive(data);
    expect(serializeSeasonArchive(parseSeasonArchiveData(bytes, "memory"))).toBe(bytes);
  });

  test("file and localStorage backings round-trip the same bytes, including the tower", () => {
    const dir = mkdtempSync(join(tmpdir(), "season-rt-"));
    const tower = towerSnapshot();
    const seat = (store: SeasonArchiveStore) => {
      store.archive(record(1, FIRST_CONTENT_VERSION, tower));
      store.archive(record(2, 2, tower));
    };

    const path = join(dir, "archive.json");
    const fileStore = new FileSeasonArchiveStore(path);
    seat(fileStore);
    const fileBytes = readFileSync(path, "utf8");

    const medium = memoryMedium();
    const memStore = medium.store();
    seat(memStore);
    const memBytes = medium.read()!;

    // Same serialized archive on both media (the file pretty-prints, so does
    // serializeSeasonArchive) — and a reopen reads the same seasons back.
    expect(memBytes).toBe(fileBytes);
    expect(JSON.parse(memBytes)).toEqual(JSON.parse(fileBytes));
    expect(new FileSeasonArchiveStore(path).list()).toEqual(fileStore.list());
    expect(new FileSeasonArchiveStore(path).seasonAt(2)!.finalTower).toEqual(tower);
  });
});

// ---------------------------------------------------------------------------
// 5. Corrupt JSON throws loudly — never a silent fresh archive (history erased)
// ---------------------------------------------------------------------------

describe("corrupt storage is refused loudly", () => {
  test("parseSeasonArchiveData throws on non-JSON and on the wrong shape", () => {
    expect(() => parseSeasonArchiveData("not json", "memory")).toThrow(/not valid JSON/);
    expect(() => parseSeasonArchiveData('{"foo":1}', "memory")).toThrow(/no seasons array/);
    expect(() => parseSeasonArchiveData("null", "memory")).toThrow(/no seasons array/);
  });

  test("a corrupt archive file throws on open — not a silently reset history", () => {
    const dir = mkdtempSync(join(tmpdir(), "season-corrupt-"));
    // A missing file opens an empty archive…
    expect(new FileSeasonArchiveStore(join(dir, "missing.json")).list()).toEqual([]);
    // …but a present-but-unreadable one throws loudly (erasing it would lose
    // every finished season, the one thing this store exists to keep).
    const corrupt = join(dir, "corrupt.json");
    writeFileSync(corrupt, "not json", "utf8");
    expect(() => new FileSeasonArchiveStore(corrupt)).toThrow(/not valid JSON/);
  });
});

// ---------------------------------------------------------------------------
// 6. Persisted parity: a reopened store equals the one that wrote it
// ---------------------------------------------------------------------------

describe("persisted parity", () => {
  test("every append writes through — a store reopened from the medium is equal", () => {
    const medium = memoryMedium();
    const store = medium.store();
    store.archive(record(1, FIRST_CONTENT_VERSION));
    store.archive(record(2, 3));
    const reopened = medium.store(); // parses the last persisted JSON
    expect(reopened.list()).toEqual(store.list());
    expect(reopened.seasonAt(2)).toEqual(store.seasonAt(2));
  });
});
