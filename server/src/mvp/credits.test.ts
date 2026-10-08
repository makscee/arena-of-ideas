// M2-9 (mission #735): credits, NEW, the creator number and the library,
// read from M2-1's units as data.
import { describe, expect, it } from "vitest";
import type { CreditsView, FusionDiscovery, LibraryView, PlayerRef } from "../../../src/mvp/contract.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { creditsView, libraryView, liveDays, NEW_DAYS } from "./credits.js";
import { CUT_IN_ROUND_4, poolContent, seedUnits, swapUnit } from "./pool.js";
import { mvpRuntime } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore, type MvpStore, type StoredUnit } from "./store.js";

function world(store: MvpStore = new MemoryMvpStore(), dev = true) {
  seedUnits(store, new Date("2026-10-08T08:00:00.000Z"));
  const rt = mvpRuntime({ content: poolContent(store), store, dev });
  const app = createMvpApp(rt);
  const call = async <T>(method: string, path: string, body?: unknown, player?: string) => {
    const res = await app.request(`/api/v1${path}`, {
      method,
      headers: { "content-type": "application/json", ...(player ? { "X-Arena-Player": player } : {}) },
      ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
    });
    return { status: res.status, json: (await res.json()) as T };
  };
  /** Moves the world to day `seq` (the day's other fields are labels here). */
  const toDay = (seq: number) => {
    const d = store.currentDay()!;
    store.putDay({ ...d, seq });
  };
  return { rt, store, call, toDay };
}

describe("credits and the library (M2-9)", () => {
  it("liveDays counts each day a unit was in the pool", () => {
    expect(liveDays([{ unitId: "x", enteredSeq: 5, leftSeq: null, reason: "seed" }], 5)).toBe(1);
    expect(liveDays([{ unitId: "x", enteredSeq: 5, leftSeq: null, reason: "seed" }], 9)).toBe(5);
    expect(liveDays([{ unitId: "x", enteredSeq: 2, leftSeq: 4, reason: "seed" }, { unitId: "x", enteredSeq: 7, leftSeq: null, reason: "seed" }], 8)).toBe(4);
  });

  it("the seed has no credits and nothing NEW; the 8 cut in round 4 are the library", async () => {
    for (const store of [new MemoryMvpStore(), new SqliteMvpStore(":memory:")] as MvpStore[]) {
      const { call } = world(store);
      await call("GET", "/day"); // day 1
      const { json: p } = await call<PlayerRef>("POST", "/players", { name: "Maks" });
      const { json: credits } = await call<CreditsView>("GET", "/credits", undefined, p.id);
      // Every live unit: no author, nothing NEW, its first day live (M3-8).
      expect(credits.units).toHaveLength(store.currentPool()!.unitIds.length);
      for (const c of credits.units) expect(c).toEqual({ unitId: c.unitId, by: null, evolvedBy: null, version: 1, versions: [], isNew: false, liveDays: 1 });
      expect(credits.you).toEqual({ days: 0, units: 0 });
      expect((await call<CreditsView>("GET", "/credits")).json.you).toBeNull();
      const { json: lib } = await call<LibraryView>("GET", "/library");
      expect(lib.units.map((l) => l.unit.name).sort()).toEqual(CUT_IN_ROUND_4.map((r) => r.name).sort());
      for (const l of lib.units) {
        expect(l.by).toBeNull();
        expect(l.liveDays).toBeNull(); // cut before units were data: no stint
        // Its sheet reads: every ability its forms name comes with it.
        for (const f of [l.unit.forms.sleeping, l.unit.forms.awoken]) for (const d of f.does) expect(lib.abilities[d], `${l.unit.name}: ${d}`).toBeDefined();
      }
    }
  });

  it("a unit credited to a player shows its author and NEW for 3 days, and counts in their creator number", async () => {
    const { call, toDay, rt } = world();
    await call("GET", "/day");
    const { json: maks } = await call<PlayerRef>("POST", "/players", { name: "Maks" });
    const { json: other } = await call<PlayerRef>("POST", "/players", { name: "Other" });
    const version = rt.content.version;
    const { status, json } = await call<CreditsView>("POST", "/dev/credit-unit", {}, maks.id);
    expect(status).toBe(200);
    const unitId = json.units.find((u) => u.by)!.unitId;
    expect(rt.content.units.find((u) => u.id === unitId)!.tier).toBe(1);
    const credited = (c: CreditsView) => c.units.filter((u) => u.by || u.isNew).map(({ unitId, by, isNew }) => ({ unitId, by, isNew }));
    expect(credited(json)).toEqual([{ unitId, by: maks, isNew: true }]);
    expect(json.you).toEqual({ days: 1, units: 1 });
    expect(poolContent(rt.store).version).toBe(version); // credits are not content

    toDay(1 + NEW_DAYS - 1);
    let c = (await call<CreditsView>("GET", "/credits", undefined, other.id)).json;
    expect(credited(c)).toEqual([{ unitId, by: maks, isNew: true }]);
    expect(c.you).toEqual({ days: 0, units: 0 });
    toDay(1 + NEW_DAYS);
    c = (await call<CreditsView>("GET", "/credits", undefined, maks.id)).json;
    expect(credited(c)).toEqual([{ unitId, by: maks, isNew: false }]);
    expect(c.you).toEqual({ days: NEW_DAYS + 1, units: 1 });
  });

  it("a unit that left shows in the library with its days live, its author and its fusions", async () => {
    const { call, store, toDay } = world();
    await call("GET", "/day");
    const { json: maks } = await call<PlayerRef>("POST", "/players", { name: "Maks" });
    const { json: c } = await call<CreditsView>("POST", "/dev/credit-unit", {}, maks.id);
    const unitId = c.units.find((u) => u.by)!.unitId;
    // It leaves at day 6 (M2-10's rotation will do this): live days 1–5.
    toDay(6);
    const open = store.stints(unitId).find((s) => s.leftSeq === null)!;
    store.putStint({ ...open, leftSeq: 6, reason: "rotated" });
    store.putUnit({ ...store.unit(unitId)!, status: "library" });
    const other = store.currentPool()!.unitIds.find((id) => id !== unitId)!;
    const f: FusionDiscovery = { first: unitId, second: other, name: "Testfuse", discoveredBy: maks, discoveredAt: "2026-10-08T09:00:00.000Z", nameSource: "fallback" };
    store.putFusion(f);
    toDay(9);
    const { json: lib } = await call<LibraryView>("GET", "/library");
    expect(lib.units).toHaveLength(CUT_IN_ROUND_4.length + 1);
    const it = lib.units[0]!; // the one that left last first
    expect(it.unit.id).toBe(unitId);
    expect(it.by).toEqual(maks);
    expect(it.liveDays).toBe(5);
    expect(it.fusions).toEqual([f]);
    // Still Maks's: the creator number keeps the days it was live.
    expect((await call<CreditsView>("GET", "/credits", undefined, maks.id)).json.you).toEqual({ days: 5, units: 1 });
  });

  it("the dev tool is 404 off a dev server, and for a unit that isn't live", async () => {
    const off = world(new MemoryMvpStore(), false);
    const { json: p } = await off.call<PlayerRef>("POST", "/players", { name: "Maks" });
    expect((await off.call("POST", "/dev/credit-unit", {}, p.id)).status).toBe(404);
    const on = world();
    const { json: q } = await on.call<PlayerRef>("POST", "/players", { name: "Maks" });
    expect((await on.call("POST", "/dev/credit-unit", { unitId: "rat" }, q.id)).status).toBe(404);
    expect((await on.call("POST", "/dev/credit-unit", {})).status).toBe(401);
  });

  it("a runtime without a pool (the tests' code content) answers with nothing to credit", async () => {
    const rt = mvpRuntime({ content: mvpContent() });
    const app = createMvpApp(rt);
    const credits = (await (await app.request("/api/v1/credits")).json()) as CreditsView;
    expect(credits.units).toEqual([]);
    const lib = (await (await app.request("/api/v1/library")).json()) as LibraryView;
    expect(lib.units).toEqual([]);
  });
});

// M3-8 (mission #800): an archetype's versions (M3-3's lineage) share the
// root's "idea by" and their days live; each adds "evolved by".
describe("credits across versions (M3-8)", () => {
  const AT = new Date("2026-10-08T08:00:00.000Z");
  const A: PlayerRef = { id: "pa", name: "a", bot: false };
  const B: PlayerRef = { id: "pb", name: "b", bot: false };
  const C: PlayerRef = { id: "pc", name: "c", bot: false };

  /** A seeded world with 3 players, on day `seq`. */
  function chainWorld(store: MvpStore) {
    seedUnits(store, AT);
    for (const p of [A, B, C]) store.addPlayer(p);
    const toDay = (seq: number) => store.putDay({ ...store.currentDay()!, seq });
    const deps = () => ({ store, content: poolContent(store) });
    /** A version of `root` (or a new root) by `author`, stored but not live. */
    const version = (unitId: string, author: PlayerRef | null, root: string | null, parent: string | null, status: StoredUnit["status"] = "library"): void =>
      store.putUnit({ unitId, status, row: root ? store.unit(root)!.row : { ...store.unit("rat")!.row, name: "Hog" }, authorId: author?.id ?? null, origin: root ? "evolution" : "idea", parentId: parent, ...(root ? { rootId: root } : {}), createdAt: AT.toISOString() });
    return { toDay, deps, version };
  }

  for (const make of [() => new MemoryMvpStore(), () => new SqliteMvpStore(":memory:")] as (() => MvpStore)[])
    it(`a 3-version chain by a, b and c: credits, days, history and both creator numbers (${make().constructor.name})`, () => {
      const store = make();
      const { toDay, deps, version } = chainWorld(store);
      store.putDay({ ...store.currentDay()!, seq: 2 });
      // v1 by a, live days 2–5; v2 by b, days 6–9; v3 by c, live from day 10.
      version("hog", A, null, null);
      swapUnit(store, "fighter", "hog", AT);
      toDay(6);
      version("hog-2", B, "hog", "hog");
      swapUnit(store, "hog", "hog-2", AT);
      toDay(10);
      version("hog-3", C, "hog", "hog-2");
      swapUnit(store, "hog-2", "hog-3", AT);
      // Proposals that never entered: one still a candidate, one that lost (M3-7 closes it to the library).
      version("hog-4", B, "hog", "hog-3", "candidate");
      version("hog-5", B, "hog", "hog-3", "library");
      toDay(12);

      const history = [
        { unitId: "hog", version: 1, by: A, liveDays: 4, live: false },
        { unitId: "hog-2", version: 2, by: B, liveDays: 4, live: false },
        { unitId: "hog-3", version: 3, by: C, liveDays: 3, live: true },
      ];
      const credits = creditsView(deps(), A.id);
      // A version keeps its name, so the pool serves it under its root's slug.
      expect(credits.units.find((u) => u.unitId === "hog")).toEqual({ unitId: "hog", by: A, evolvedBy: C, version: 3, versions: history, isNew: true, liveDays: 11 });
      expect(credits.units.filter((u) => u.unitId.startsWith("hog"))).toHaveLength(1);
      // The first author counts the whole archetype; each evolver their own versions.
      expect(credits.you).toEqual({ units: 1, days: 11 });
      expect(creditsView(deps(), B.id).you).toEqual({ units: 3, days: 4 });
      expect(creditsView(deps(), C.id).you).toEqual({ units: 1, days: 3 });

      // The Library: v1 and v2 (v2 left last, so first), with the archetype's days; never the proposals that lost.
      const lib = libraryView(deps());
      expect(lib.units.map((l) => l.unit.id).filter((id) => id.startsWith("hog"))).toEqual(["hog-2", "hog"]);
      const [v2, v1] = lib.units;
      expect({ by: v2!.by, evolvedBy: v2!.evolvedBy, version: v2!.version, liveDays: v2!.liveDays, versions: v2!.versions }).toEqual({ by: A, evolvedBy: B, version: 2, liveDays: 11, versions: history });
      expect({ by: v1!.by, evolvedBy: v1!.evolvedBy, version: v1!.version, liveDays: v1!.liveDays }).toEqual({ by: A, evolvedBy: null, version: 1, liveDays: 11 });
      // A unit with one version has no history.
      expect(lib.units.find((l) => l.unit.id === "fighter")).toMatchObject({ version: 1, versions: [], evolvedBy: null, liveDays: 1 });
    });

  it("a seed unit evolved: \"evolved by\" alone, its seed days counted, the evolver's number only theirs", () => {
    const store = new MemoryMvpStore();
    const { toDay, deps, version } = chainWorld(store);
    toDay(3);
    // Fighter (seed, live days 1–2) gives way to b's version on day 3.
    version("fighter-2", B, "fighter", "fighter", "candidate");
    swapUnit(store, "fighter", "fighter-2", AT);
    toDay(5);
    const c = creditsView(deps(), B.id);
    expect(c.units.find((u) => u.unitId === "fighter")).toEqual({
      unitId: "fighter",
      by: null,
      evolvedBy: B,
      version: 2,
      versions: [
        { unitId: "fighter", version: 1, by: null, liveDays: 2, live: false },
        { unitId: "fighter-2", version: 2, by: B, liveDays: 3, live: true },
      ],
      isNew: true,
      liveDays: 5,
    });
    expect(c.you).toEqual({ units: 1, days: 3 });
    expect(libraryView(deps()).units.find((l) => l.unit.id === "fighter")).toMatchObject({ by: null, evolvedBy: null, version: 1, liveDays: 5 });
  });

  it("NEW applies to an entering version, and to a seed unit back unchanged", () => {
    const store = new MemoryMvpStore();
    const { toDay, deps } = chainWorld(store);
    toDay(3);
    swapUnit(store, "fighter", "rat", AT);
    toDay(4);
    swapUnit(store, "rat", "fighter", AT);
    const back = store.stints("fighter").find((s) => s.leftSeq === null)!;
    store.putStint({ ...back, reason: "return" });
    const c = creditsView(deps());
    expect(c.units.find((u) => u.unitId === "fighter")).toMatchObject({ isNew: true, liveDays: 3, version: 1 });
    toDay(4 + NEW_DAYS);
    expect(creditsView(deps()).units.find((u) => u.unitId === "fighter")!.isNew).toBe(false);
  });
});
