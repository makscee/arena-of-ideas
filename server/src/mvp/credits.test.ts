// M2-9 (mission #735): credits, NEW, the creator number and the library,
// read from M2-1's units as data.
import { describe, expect, it } from "vitest";
import type { CreditsView, FusionDiscovery, LibraryView, PlayerRef } from "../../../src/mvp/contract.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { liveDays, NEW_DAYS } from "./credits.js";
import { CUT_IN_ROUND_4, poolContent, seedUnits } from "./pool.js";
import { mvpRuntime } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";

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
      expect(credits.units).toEqual([]);
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
    const unitId = json.units[0]!.unitId;
    expect(rt.content.units.find((u) => u.id === unitId)!.tier).toBe(1);
    expect(json.units).toEqual([{ unitId, by: maks, isNew: true }]);
    expect(json.you).toEqual({ days: 1, units: 1 });
    expect(poolContent(rt.store).version).toBe(version); // credits are not content

    toDay(1 + NEW_DAYS - 1);
    let c = (await call<CreditsView>("GET", "/credits", undefined, other.id)).json;
    expect(c.units).toEqual([{ unitId, by: maks, isNew: true }]);
    expect(c.you).toEqual({ days: 0, units: 0 });
    toDay(1 + NEW_DAYS);
    c = (await call<CreditsView>("GET", "/credits", undefined, maks.id)).json;
    expect(c.units).toEqual([{ unitId, by: maks, isNew: false }]);
    expect(c.you).toEqual({ days: NEW_DAYS + 1, units: 1 });
  });

  it("a unit that left shows in the library with its days live, its author and its fusions", async () => {
    const { call, store, toDay } = world();
    await call("GET", "/day");
    const { json: maks } = await call<PlayerRef>("POST", "/players", { name: "Maks" });
    const { json: c } = await call<CreditsView>("POST", "/dev/credit-unit", {}, maks.id);
    const unitId = c.units[0]!.unitId;
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
