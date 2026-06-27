import { describe, expect, test } from "vitest";
import { InMemoryIdeaStore } from "./ideas.js";
import type { Idea } from "./ideas.js";
import { stressAbilities, stressRegistry } from "./content/stress.js";
import { InMemoryLadderStore, seedBootstrapTower } from "./ladder.js";
import type { LadderData, LadderStore, TeamSnapshot } from "./ladder.js";
import { FIRST_CONTENT_VERSION, InMemorySeasonArchiveStore } from "./season-archive.js";
import type { ContentVersion, SeasonArchiveStore } from "./season-archive.js";
import {
  FIRST_SEASON,
  InMemorySeasonPointerStore,
  PersistedSeasonPointerStore,
  emptySeasonPointer,
  emptySeasonPointerData,
  parseSeasonPointerData,
  serializeSeasonPointer,
  snapshotLadder,
  transitionSeason,
} from "./season.js";
import type {
  SeasonPointer,
  SeasonPointerStore,
  SeasonTransitionOps,
} from "./season.js";

// ---------------------------------------------------------------------------
// Helpers — a live world (ladder + archive + pointer) plus a reset that wipes
// and re-bootstraps the ladder, exactly what the CLI/web would inject.
// ---------------------------------------------------------------------------

/** A localStorage-like medium for the pointer: the last persisted JSON string. */
function memoryPointerMedium(): { read(): string | null; store(): SeasonPointerStore } {
  let raw: string | null = null;
  return {
    read: () => raw,
    store: () =>
      new PersistedSeasonPointerStore(
        raw === null ? emptySeasonPointerData() : parseSeasonPointerData(raw, "memory"),
        (d) => {
          raw = serializeSeasonPointer(d);
        },
      ),
  };
}

/** A live season world: a freshly-bootstrapped ladder behind a holder the reset
 * can swap, an archive, and a pointer. `reset` wipes the live tower and opens a
 * fresh bootstrap on a NEW store (the CLI/web reuse seedBootstrapTower this way), and
 * `ladder()` always reads the current live store. */
function freshWorld(): {
  ladder: () => LadderStore;
  ops: () => SeasonTransitionOps;
  archive: SeasonArchiveStore;
  pointer: SeasonPointerStore;
} {
  let live: LadderStore = seedBootstrapTower(new InMemoryLadderStore(), stressRegistry, stressAbilities);
  const archive = new InMemorySeasonArchiveStore();
  const pointer = new InMemorySeasonPointerStore();
  const reset = () => {
    live = seedBootstrapTower(new InMemoryLadderStore(), stressRegistry, stressAbilities);
  };
  return {
    ladder: () => live,
    archive,
    pointer,
    ops: () => ({ live, archive, pointer, reset }),
  };
}

/** A read-out of the live tower in the #075 LadderData shape, the way the
 * season-archive test reads it — the independent reference the transition's own
 * snapshot must deep-equal. */
function readTower(store: LadderStore): LadderData {
  return snapshotLadder(store);
}

/** Field a ghost into a round's pool — a played-on tower carries run ghosts the
 * reset must wipe. */
function fieldGhost(store: LadderStore, round: number, runId: string): void {
  const seq = store.poolAt(round).length;
  const snap: TeamSnapshot = { runId, round, seq, team: [{ name: "Brawler", base: { hp: 9, pwr: 2 }, ability: "Strike" }] };
  store.addSnapshot(snap);
}

// ---------------------------------------------------------------------------
// 1. The live season pointer — backing parity, fresh state, isolation
// ---------------------------------------------------------------------------

const pointerBackings: { name: string; fresh: () => SeasonPointerStore }[] = [
  { name: "InMemory", fresh: () => new InMemorySeasonPointerStore() },
  { name: "Persisted(memory)", fresh: () => memoryPointerMedium().store() },
];

describe.each(pointerBackings)("season pointer [$name]", ({ fresh }) => {
  test("a fresh pointer reads season 1 on the first content version", () => {
    const store = fresh();
    expect(store.get()).toEqual({ season: FIRST_SEASON, version: FIRST_CONTENT_VERSION });
    expect(emptySeasonPointer()).toEqual({ season: 1, version: 1 });
  });

  test("set overwrites the live pointer and reads back", () => {
    const store = fresh();
    store.set({ season: 4, version: 3 });
    expect(store.get()).toEqual({ season: 4, version: 3 });
  });

  test("get hands back a detached copy — mutating it cannot corrupt the store", () => {
    const store = fresh();
    const handed = store.get();
    handed.season = 99;
    handed.version = 99;
    expect(store.get()).toEqual({ season: FIRST_SEASON, version: FIRST_CONTENT_VERSION });
  });

  test("set stores a clone — mutating the caller's pointer afterward does not reach the store", () => {
    const store = fresh();
    const p: SeasonPointer = { season: 2, version: 2 };
    store.set(p);
    p.season = 99;
    p.version = 99;
    expect(store.get()).toEqual({ season: 2, version: 2 });
  });
});

describe("season pointer serialized round-trip / corrupt-loud", () => {
  test("serialize → parse is deep-equal and byte-stable", () => {
    const data = { season: 3, version: 5 };
    const bytes = serializeSeasonPointer(data);
    expect(parseSeasonPointerData(bytes, "memory")).toEqual(data);
    expect(serializeSeasonPointer(parseSeasonPointerData(bytes, "memory"))).toBe(bytes);
  });

  test("a persisted pointer reopened from the medium is equal (write-through)", () => {
    const medium = memoryPointerMedium();
    const store = medium.store();
    store.set({ season: 7, version: 4 });
    expect(medium.store().get()).toEqual({ season: 7, version: 4 });
  });

  test("corrupt JSON / wrong shape throws loudly — never a silent reset to season 1", () => {
    expect(() => parseSeasonPointerData("not json", "memory")).toThrow(/not valid JSON/);
    expect(() => parseSeasonPointerData('{"season":1}', "memory")).toThrow(/not a season pointer/);
    expect(() => parseSeasonPointerData('{"version":1}', "memory")).toThrow(/not a season pointer/);
    expect(() => parseSeasonPointerData("null", "memory")).toThrow(/not a season pointer/);
  });
});

// ---------------------------------------------------------------------------
// 2. snapshotLadder reads the FULL live tower, including a grown (sparse) one
// ---------------------------------------------------------------------------

describe("snapshotLadder", () => {
  test("reads every boss + pool of a seeded tower in the LadderData shape", () => {
    const ladder = seedBootstrapTower(new InMemoryLadderStore(), stressRegistry, stressAbilities);
    const snap = snapshotLadder(ladder);
    // Every seeded floor's boss + pool is present.
    for (let floor = 1; ladder.bossAt(floor) !== null; floor++) {
      expect(snap.bosses[String(floor)]).toEqual(ladder.bossAt(floor));
      expect(snap.pools[String(floor)]).toEqual([...ladder.poolAt(floor)]);
    }
  });

  test("captures a floor seated ABOVE a vacant one — does not stop at the first gap", () => {
    const ladder = seedBootstrapTower(new InMemoryLadderStore(), stressRegistry, stressAbilities);
    // A grown summit: an ascend seats a boss two floors above the top with the
    // intervening floor left vacant (overshoot). A stop-at-first-gap scan would
    // miss it; snapshotLadder must capture it.
    let top = 1;
    while (ladder.bossAt(top + 1) !== null) top++;
    const high = top + 3;
    const boss: TeamSnapshot = { runId: "web-9", round: high, seq: 0, team: [{ name: "Warlord", base: { hp: 30, pwr: 9 }, ability: "Strike" }] };
    ladder.setBoss(high, boss);
    const snap = snapshotLadder(ladder);
    expect(snap.bosses[String(high)]).toEqual(boss);
    expect(snap.bosses[String(top + 1)]).toBeUndefined(); // the gap stays a gap
  });

  test("the snapshot is detached — mutating it cannot reach the live store", () => {
    const ladder = seedBootstrapTower(new InMemoryLadderStore(), stressRegistry, stressAbilities);
    const snap = snapshotLadder(ladder);
    snap.bosses["1"]!.team[0]!.name = "Corrupted";
    expect(snapshotLadder(ladder).bosses["1"]!.team[0]!.name).not.toBe("Corrupted");
  });
});

// ---------------------------------------------------------------------------
// 3. The season transition — the acceptance cases
// ---------------------------------------------------------------------------

describe("transitionSeason", () => {
  test("archives a snapshot that deep-equals the pre-reset live tower", () => {
    const w = freshWorld();
    // A played-on tower: extra ghosts the archived snapshot must include.
    fieldGhost(w.ladder(), 1, "web-1");
    fieldGhost(w.ladder(), 2, "web-1");
    const preReset = readTower(w.ladder()); // the live tower right before reset

    const { archived } = transitionSeason(w.ops());

    // The archived finalTower IS the pre-reset live tower, deep-equal.
    expect(archived.finalTower).toEqual(preReset);
    expect(w.archive.seasonAt(1)!.finalTower).toEqual(preReset);
  });

  test("post-transition live state is empty/fresh — prior-season ghosts wiped, a fresh bootstrap", () => {
    const w = freshWorld();
    fieldGhost(w.ladder(), 1, "web-1"); // a prior-season ghost
    const beforeFloor1 = w.ladder().poolAt(1).length;

    transitionSeason(w.ops());

    // The live tower is a FRESH bootstrap (a real seeded tower), not the old one:
    // it equals a brand-new seedBootstrapTower and carries no web-1 ghost.
    const freshSeed = snapshotLadder(seedBootstrapTower(new InMemoryLadderStore(), stressRegistry, stressAbilities));
    expect(snapshotLadder(w.ladder())).toEqual(freshSeed);
    // The prior-season run ghost is gone (the bootstrap floor-1 pool is shorter
    // than the played-on one was, and no snapshot carries runId web-1).
    expect(w.ladder().poolAt(1).length).toBeLessThan(beforeFloor1);
    expect(JSON.stringify(snapshotLadder(w.ladder()))).not.toContain("web-1");
  });

  test("version incremented N→N+1; archived record carries OLD N; new season carries N+1", () => {
    const w = freshWorld();
    const before = w.pointer.get();
    expect(before).toEqual({ season: 1, version: FIRST_CONTENT_VERSION });

    const { archived, pointer } = transitionSeason(w.ops());

    expect(archived.version).toBe(before.version); // OLD version N on the record
    expect(pointer.version).toBe(before.version + 1); // live version is N+1
    expect(w.pointer.get().version).toBe(before.version + 1);
  });

  test("season number advances: archive gets season N; the live pointer moves to N+1", () => {
    const w = freshWorld();
    const { archived, pointer } = transitionSeason(w.ops());
    expect(archived.season).toBe(FIRST_SEASON); // season N archived
    expect(pointer.season).toBe(FIRST_SEASON + 1); // live cursor on N+1
    expect(w.archive.list().map((r) => r.season)).toEqual([1]); // dense order
  });

  test("rolls several seasons — dense append-only history, monotonic version", () => {
    const w = freshWorld();
    transitionSeason(w.ops());
    transitionSeason(w.ops());
    transitionSeason(w.ops());
    expect(w.archive.list().map((r) => r.season)).toEqual([1, 2, 3]);
    expect(w.archive.list().map((r) => r.version)).toEqual([1, 2, 3]); // OLD version each
    expect(w.pointer.get()).toEqual({ season: 4, version: 4 });
  });

  test("a custom bumpVersion that ships to a specific later version is honored", () => {
    const w = freshWorld();
    const { archived, pointer } = transitionSeason({ ...w.ops(), bumpVersion: () => 7 });
    expect(archived.version).toBe(1); // old
    expect(pointer.version).toBe(7); // shipped target
  });

  test("a non-increasing bumpVersion throws — the content clock is monotonic", () => {
    const w = freshWorld();
    expect(() => transitionSeason({ ...w.ops(), bumpVersion: (v) => v })).toThrow(/strictly increase/);
    expect(() => transitionSeason({ ...w.ops(), bumpVersion: () => 0 })).toThrow(/strictly increase/);
  });
});

// ---------------------------------------------------------------------------
// 4. The version-boundary invariant (the verify clause) — must-fail-first
//
//   "a content change only ever lands on an EMPTY tower; no live ghost
//    references a changed unit mid-season."
//
// Encoded as an OBSERVABLE ordering invariant: at the moment the content version
// changes (pointer.set raising the version), the live tower must be empty/just-
// reset — it must NOT still hold the prior season's ghosts. A mutant that bumps
// the version BEFORE the reset (or without archiving) reorders the steps and
// reddens here.
// ---------------------------------------------------------------------------

describe("version-boundary invariant", () => {
  /** A pointer store that asserts, the instant the content version changes, that
   * the live tower is the prior season's ghosts no longer — i.e. the reset has
   * already run. This is the invariant made observable: the version may only
   * move once the tower is empty/just-reset. */
  function boundaryGuardPointer(liveAtCheck: () => LadderStore, hadGhost: () => boolean): SeasonPointerStore {
    const inner = new InMemorySeasonPointerStore();
    return {
      get: () => inner.get(),
      set: (p) => {
        const prev = inner.get();
        if (p.version !== prev.version) {
          // The version is changing NOW. The live tower must already be reset:
          // the prior-season ghost must be gone. If it is still present, the bump
          // raced ahead of the reset — the invariant is violated.
          if (hadGhost()) {
            throw new Error(
              "version-boundary violated: content version changed while a prior-season ghost is still live",
            );
          }
        }
        inner.set(p);
      },
    };
  }

  test("the version changes only AFTER the reset — a prior-season ghost is gone by the bump", () => {
    let live: LadderStore = seedBootstrapTower(new InMemoryLadderStore(), stressRegistry, stressAbilities);
    fieldGhost(live, 1, "web-ghost"); // a prior-season ghost
    const archive = new InMemorySeasonArchiveStore();
    const reset = () => {
      live = seedBootstrapTower(new InMemoryLadderStore(), stressRegistry, stressAbilities);
    };
    const hadGhost = () => JSON.stringify(snapshotLadder(live)).includes("web-ghost");
    const pointer = boundaryGuardPointer(() => live, hadGhost);

    // Sanity: the ghost is live before the transition.
    expect(hadGhost()).toBe(true);

    // The real ordering (archive → reset → bump) satisfies the guard: by the time
    // pointer.set raises the version, reset() has wiped the ghost.
    expect(() =>
      transitionSeason({ live, archive, pointer, reset }),
    ).not.toThrow();
    expect(hadGhost()).toBe(false); // reset really happened
    expect(pointer.get().version).toBe(FIRST_CONTENT_VERSION + 1);
  });

  test("MUST-FAIL-FIRST: a mutant that bumps the version BEFORE the reset reddens the guard", () => {
    let live: LadderStore = seedBootstrapTower(new InMemoryLadderStore(), stressRegistry, stressAbilities);
    fieldGhost(live, 1, "web-ghost");
    const archive = new InMemorySeasonArchiveStore();
    const reset = () => {
      live = seedBootstrapTower(new InMemoryLadderStore(), stressRegistry, stressAbilities);
    };
    const hadGhost = () => JSON.stringify(snapshotLadder(live)).includes("web-ghost");
    const pointer = boundaryGuardPointer(() => live, hadGhost);

    // The mutant transition: archive, then BUMP (set the version) BEFORE reset.
    // This is exactly the reordering the invariant forbids. It must throw at the
    // guard — proving the guard has teeth and the real ordering's pass is meaningful.
    const mutantBumpBeforeReset = () => {
      const current = pointer.get();
      archive.archive({ season: current.season, version: current.version, finalTower: snapshotLadder(live) });
      pointer.set({ season: current.season + 1, version: current.version + 1 }); // bump BEFORE reset — forbidden
      reset();
    };
    expect(mutantBumpBeforeReset).toThrow(/version-boundary violated/);
  });

  test("MUST-FAIL-FIRST: a mutant that bumps WITHOUT archiving breaks append-only history", () => {
    // The archive is the record of WHAT version a finished season ran on. A
    // transition that bumps the version but skips the archive loses that record:
    // the next archive would desync (season 1 never written, season 2 expected
    // next would be wrong) — the append-only store catches it.
    const w = freshWorld();
    // Skip-archive mutant: advance the pointer without archiving season 1…
    w.pointer.set({ season: 2, version: 2 });
    // …then a later real transition tries to archive season 2 first — but the
    // archive is empty, so season 2 desyncs (it expects season 1 next).
    expect(() => transitionSeason(w.ops())).toThrow(/desyncs/);
  });

  test("a failed archive (desynced season) leaves the live tower and pointer untouched — no half-roll", () => {
    const w = freshWorld();
    transitionSeason(w.ops()); // season 1 archived, pointer now {2,2}
    const liveBefore = snapshotLadder(w.ladder());
    const pointerBefore = w.pointer.get();
    // Corrupt the pointer to a season that desyncs the archive (season 5 when 2
    // is next): the archive throws, and nothing after it runs.
    w.pointer.set({ season: 5, version: 9 });
    expect(() => transitionSeason(w.ops())).toThrow(/desyncs/);
    // The reset/bump never ran: the live tower is unchanged and the pointer is
    // exactly what the failed roll read (the corrupt {5,9}, not advanced).
    expect(snapshotLadder(w.ladder())).toEqual(liveBefore);
    expect(w.pointer.get()).toEqual({ season: 5, version: 9 });
    expect(pointerBefore).toEqual({ season: 2, version: 2 }); // (sanity on the pre-corruption cursor)
  });
});

// ---------------------------------------------------------------------------
// 5. Selection wired into the roll (#083 slice 3) — the roll returns the build
// slate, the ideas table is NOT reset (carry-over survives with votes intact),
// and selection is read-only over the version (it never bumps it).
// ---------------------------------------------------------------------------

describe("transitionSeason with selection wired", () => {
  /** A seeded ideas table: one HOT idea past the floor (6 up-votes, ratio 1.0,
   * → eligible+selected) and one MILD idea below the floor (1 up-vote → carried). */
  function seededIdeas(): { store: InMemoryIdeaStore; hot: Idea; mild: Idea } {
    const store = new InMemoryIdeaStore();
    const hot = store.submit("hot idea", "ada");
    const mild = store.submit("mild idea", "ada");
    for (let i = 0; i < 6; i++) store.castVote(hot.id, `voter-${i}`, "up"); // 6 ≥ floor 5
    store.castVote(mild.id, "z", "up"); // 1 < floor 5
    return { store, hot, mild };
  }

  const votesById = (ideas: readonly Idea[]) => Object.fromEntries(ideas.map((i) => [i.id, i.votes]));

  test("the roll returns the build slate (selected = top eligible)", () => {
    const w = freshWorld();
    const { store, hot, mild } = seededIdeas();
    const result = transitionSeason({ ...w.ops(), selection: { ideas: store.list() } });
    expect(result.selection).toBeDefined();
    expect(result.selection!.selected.map((r) => r.idea.id)).toEqual([hot.id]);
    expect(result.selection!.carried.map((i) => i.id)).toContain(mild.id);
  });

  test("carry-over survives the roll: the live ideas table is untouched, votes byte-equal pre/post", () => {
    const w = freshWorld();
    const { store } = seededIdeas();
    const before = store.list();
    transitionSeason({ ...w.ops(), selection: { ideas: store.list() } });
    // The transition resets the TOWER but never the ideas table — every idea and
    // its vote map is exactly as it was (nothing destroyed, the priority queue
    // carries forward).
    const after = store.list();
    expect(after.map((i) => i.id)).toEqual(before.map((i) => i.id));
    expect(votesById(after)).toEqual(votesById(before));
  });

  test("the tower still resets while the ideas table carries over — both invariants hold together", () => {
    const w = freshWorld();
    fieldGhost(w.ladder(), 1, "web-roll"); // a prior-season tower ghost
    const { store } = seededIdeas();
    transitionSeason({ ...w.ops(), selection: { ideas: store.list() } });
    // Tower: fresh bootstrap, the prior-season ghost wiped.
    expect(JSON.stringify(snapshotLadder(w.ladder()))).not.toContain("web-roll");
    // Table: still holds both ideas (carry-over), unreset by the roll.
    expect(store.list().length).toBe(2);
  });

  test("selection is READ-ONLY over the version: it never bumps the version itself — bump is still the op's last step", () => {
    const w = freshWorld();
    const { store } = seededIdeas();
    const before = w.pointer.get();
    const { archived, pointer } = transitionSeason({ ...w.ops(), selection: { ideas: store.list() } });
    expect(archived.version).toBe(before.version); // OLD version on the record
    expect(pointer.version).toBe(before.version + 1); // exactly one monotonic bump
    expect(w.pointer.get().version).toBe(before.version + 1);
  });

  test("a roll with NO selection input returns no slate (unchanged #077 behaviour)", () => {
    const w = freshWorld();
    const result = transitionSeason(w.ops());
    expect(result.selection).toBeUndefined();
  });

  test("the version-boundary invariant still holds with selection wired in", () => {
    let live: LadderStore = seedBootstrapTower(new InMemoryLadderStore(), stressRegistry, stressAbilities);
    fieldGhost(live, 1, "web-ghost");
    const archive = new InMemorySeasonArchiveStore();
    const reset = () => {
      live = seedBootstrapTower(new InMemoryLadderStore(), stressRegistry, stressAbilities);
    };
    const hadGhost = () => JSON.stringify(snapshotLadder(live)).includes("web-ghost");
    const inner = new InMemorySeasonPointerStore();
    const pointer: SeasonPointerStore = {
      get: () => inner.get(),
      set: (p) => {
        const prev = inner.get();
        if (p.version !== prev.version && hadGhost()) {
          throw new Error("version-boundary violated: content version changed while a prior-season ghost is still live");
        }
        inner.set(p);
      },
    };
    const { store } = seededIdeas();
    expect(() =>
      transitionSeason({ live, archive, pointer, reset, selection: { ideas: store.list() } }),
    ).not.toThrow();
    expect(pointer.get().version).toBe(FIRST_CONTENT_VERSION + 1);
  });
});
