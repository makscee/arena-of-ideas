// M2-2 (mission #735): a pool change ends nothing. A run pins the pool it
// started on; ghosts, the Crown and the champion work across pools; bots never
// take a human's crown because the pool changed; GET /content serves the units
// that left; sync-seed brings code rows in without ending a run.
import { describe, expect, it } from "vitest";
import type { Champion, LineUnit, MvpContent, PlayerRef } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import type { MvpRunState } from "../../../src/mvp/run.js";
import { ROWS } from "../../../src/mvp/units.js";
import { createMvpApp } from "./app.js";
import { seedChampion } from "./bots.js";
import { contentOf } from "./content.js";
import { leftUnits, poolContent, seedUnits, swapUnit, syncSeed } from "./pool.js";
import { decide, startRun } from "./runs.js";
import { mvpRuntime, type MvpRuntime } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";

const maks: PlayerRef = { id: "p-maks", name: "Maks", bot: false };
const ann: PlayerRef = { id: "p-ann", name: "Ann", bot: false };
const AT = new Date("2026-10-08T08:00:00.000Z");

/** A seeded world (pool A = the code pool) on `store`. */
function world(store: MvpStore = new MemoryMvpStore()): MvpRuntime {
  seedUnits(store, AT);
  let n = 7;
  return mvpRuntime({ store, seed: () => (n = (n * 1103515245 + 12345) >>> 0), now: () => AT });
}

/** The last shop round with a strong line from the run's own pool. */
function atLastRound(rt: MvpRuntime, run: MvpRunState, content: MvpContent): MvpRunState {
  const units = [...content.units].sort((a, b) => b.base.hp + b.base.pwr - (a.base.hp + a.base.pwr)).slice(0, 5);
  const at12: MvpRunState = { ...run, round: rt.rules.rounds, line: units.map((u, i) => lineUnitOf(u, `u${i + 1}`, 3)), nextUid: 6 };
  rt.store.putRun(at12);
  return at12;
}

const weak = (u: LineUnit): LineUnit => ({ ...u, stats: { pwr: 0, hp: 1 } });

describe("a pool change ends nothing (M2-2)", { timeout: 60_000 }, () => {
  for (const kind of ["memory", "sqlite"] as const) {
    it(`a run started on pool A finishes on it after a switch to B (${kind})`, () => {
      const rt = world(kind === "memory" ? new MemoryMvpStore() : new SqliteMvpStore(":memory:"));
      const a = rt.content;
      const leaving = a.units.find((u) => u.tier === 1)!.id;
      const entering = rt.store.units({ status: "library" })[0]!.unitId; // Rat
      // A human champion made on A, holding the unit that will leave.
      const champ: Champion = { seq: rt.today().seq, day: rt.today().day, player: ann, line: [weak(lineUnitOf(a.units.find((u) => u.id === leaving)!, "c1"))], since: AT.toISOString(), contentVersion: a.version, rating: 1000 };
      rt.store.putChampion(champ);
      let run = startRun(rt, maks);
      expect(run.contentVersion).toBe(a.version);

      const b = swapUnit(rt.store, leaving, entering, AT);
      expect(rt.content.version).toBe(b);
      expect(b).not.toBe(a.version);
      expect(rt.content.units.map((u) => u.id)).toContain(entering);
      expect(rt.content.units.map((u) => u.id)).not.toContain(leaving);

      // The run goes on: the same run comes back, and it shops on A.
      expect(startRun(rt, maks).runId).toBe(run.runId);
      const aIds = new Set(a.units.map((u) => u.id));
      const seen = new Set<string>();
      for (let i = 0; i < 40; i++) {
        run = { ...rt.store.run(run.runId)!, gold: 99 };
        rt.store.putRun(run);
        const r = decide(rt, run, { kind: "reroll" });
        expect(r.run.phase).toBe("shop");
        for (const o of r.run.offers) seen.add(o.unitId);
      }
      expect([...seen].every((id) => aIds.has(id))).toBe(true);
      expect(seen.has(entering)).toBe(false);
      expect(seen.has(leaving)).toBe(true); // the unit that left B is still in A's shop
      decide(rt, { ...rt.store.run(run.runId)!, gold: 99 }, { kind: "buy", slot: 0 });
      expect(rt.store.run(run.runId)!.line).toHaveLength(1);

      // Ann's new run starts on B and leaves a ghost made on B, holding the newcomer.
      const annRun = startRun(rt, { ...ann, id: "p-ann2" });
      expect(annRun.contentVersion).toBe(b);
      const newcomer = rt.content.units.find((u) => u.id === entering)!;
      const round = rt.store.run(run.runId)!.round;
      rt.store.addGhost({ ghostId: "g-b", runId: annRun.runId, player: { ...ann, id: "p-ann2" }, round, line: [lineUnitOf(newcomer, "g1")], contentVersion: b, createdAt: AT.toISOString(), rating: 1000 });
      run = { ...rt.store.run(run.runId)!, opponent: null }; // pick at the fight
      rt.store.putRun(run);
      const fought = decide(rt, run, { kind: "fight" });
      expect(fought.fight!.opponent.ghostId).toBe("g-b");
      expect(fought.run.phase).not.toBe("over");

      // It reaches the Crown against the champion made on A, and wins.
      run = atLastRound(rt, rt.store.run(run.runId)!, a);
      const r12 = decide(rt, run, { kind: "fight" });
      expect(r12.run.phase).toBe("crown");
      const crown = decide(rt, rt.store.run(run.runId)!, { kind: "fight" });
      expect(crown.fight).toMatchObject({ kind: "crown", outcome: "win", opponent: { player: { id: ann.id } } });
      expect(crown.run.endedBy).toBe("crown-won");
      expect(rt.store.run(run.runId)!.contentVersion).toBe(a.version);
    });
  }

  it("a human champion survives a pool switch: the bot seeder leaves it", async () => {
    const rt = world();
    const a = rt.content;
    const champ: Champion = { seq: rt.today().seq, day: rt.today().day, player: maks, line: [lineUnitOf(a.units[0]!, "c1")], since: AT.toISOString(), contentVersion: a.version, rating: 1000 };
    rt.store.putChampion(champ);
    swapUnit(rt.store, a.units[0]!.id, rt.store.units({ status: "library" })[0]!.unitId, AT);
    expect(await seedChampion(rt)).toBeUndefined();
    expect(rt.store.currentChampion()).toEqual(champ);
  });

  it("GET /content serves the live pool plus the units that left; /health's version follows the pool", async () => {
    const rt = world();
    const app = createMvpApp(rt);
    const get = async <T>(path: string) => (await (await app.request(`/api/v1${path}`)).json()) as T;
    const a = rt.content;
    const before = await get<MvpContent>("/content");
    expect(before.units.map((u) => u.id)).toEqual(a.units.map((u) => u.id));
    expect(before.left!.map((u) => u.name)).toEqual(["Rat", "Spike", "Wither", "Rot", "Pediatrician", "Redirector", "Director", "Doctor"]);
    const leaving = a.units[3]!.id;
    const b = swapUnit(rt.store, leaving, "rat", AT);
    expect((await get<{ contentVersion: string }>("/health")).contentVersion).toBe(b);
    const after = await get<MvpContent>("/content");
    expect(after.version).toBe(b);
    expect(after.units.map((u) => u.id)).toContain("rat");
    expect(after.left!.map((u) => u.id)).toContain(leaving);
    expect(after.left!.map((u) => u.id)).not.toContain("rat");
    // Everything a left unit names resolves on the client.
    for (const u of after.left!) for (const f of [u.forms.sleeping, u.forms.awoken]) for (const d of f.does) expect(after.abilities[d], d).toBeDefined();
    expect(leftUnits(rt.store, rt.content).map((u) => u.id)).toContain(leaving);
  });

  it("the namer still names a fusion whose part has left", () => {
    const rt = world();
    const a = rt.content;
    const [first, second] = [a.units[0]!, a.units[1]!];
    swapUnit(rt.store, first.id, "rat", AT);
    const name = rt.peekFusionName(first, second, maks).name;
    expect(name.length).toBeGreaterThan(0);
  });

  it("a run whose pool isn't stored plays on with the live one, with a warning, never a crash", () => {
    const rt = world();
    const run = startRun(rt, maks);
    rt.store.putRun({ ...run, contentVersion: "mvp-gone" });
    const r = decide(rt, rt.store.run(run.runId)!, { kind: "buy", slot: 0 });
    expect(r.run.phase).toBe("shop");
  });

  it("a ghost whose line names an unknown ability is skipped, not fought", () => {
    const rt = world();
    const run = startRun(rt, maks);
    const u = lineUnitOf(rt.content.units[0]!, "g1");
    const broken = { ...u, recipe: { ...u.recipe, does: ["Teleport 9"] } };
    rt.store.addGhost({ ghostId: "g-broken", runId: "r-x", player: ann, round: 1, line: [broken], contentVersion: "x", createdAt: AT.toISOString(), rating: 1000 });
    rt.store.putRun({ ...rt.store.run(run.runId)!, opponent: null });
    decide(rt, rt.store.run(run.runId)!, { kind: "buy", slot: 0 });
    const fought = decide(rt, { ...rt.store.run(run.runId)!, opponent: null }, { kind: "fight" });
    expect(fought.fight!.opponent.ghostId).not.toBe("g-broken");
  });
});

describe("sync-seed (M2-2)", () => {
  it("writes changed code rows, adds a snapshot, keeps pinned runs on the old rows, and is safe to run twice", () => {
    const store = new SqliteMvpStore(":memory:");
    const code = contentOf(ROWS);
    seedUnits(store, AT);
    expect(syncSeed(store, AT)).toEqual({ changed: [], added: [], version: code.version, wrote: false });
    // A DB on an older balance: the first unit's stored row has 5 more HP,
    // and the live pool (no rows of its own, like the seed) builds from it.
    const id = code.units[0]!.id;
    const stored = store.unit(id)!;
    store.putUnit({ ...stored, row: { ...stored.row, hp: stored.row.hp + 5 } });
    const oldRows = store.currentPool()!.unitIds.map((u) => store.unit(u)!.row);
    const old = contentOf(oldRows).version;
    store.putPool({ ...store.currentPool()!, version: old });
    let n = 3;
    const rt = mvpRuntime({ store, seed: () => (n = (n * 1103515245 + 12345) >>> 0), now: () => AT });
    expect(rt.content.version).toBe(old);
    const run = startRun(rt, maks);
    expect(run.contentVersion).toBe(old);

    const dry = syncSeed(store, AT, { dryRun: true });
    expect(dry).toEqual({ changed: [id], added: [], version: code.version, wrote: false });
    expect(store.unit(id)!.row.hp).toBe(stored.row.hp + 5);
    expect(store.currentPool()!.version).toBe(old);

    expect(syncSeed(store, AT)).toEqual({ changed: [id], added: [], version: code.version, wrote: true });
    expect(store.unit(id)!.row).toEqual(ROWS[0]);
    expect(rt.content.version).toBe(code.version);
    expect(poolContent(store).version).toBe(code.version);
    expect(syncSeed(store, AT)).toEqual({ changed: [], added: [], version: code.version, wrote: false });

    // A fresh server (a restart) still builds the old pool from its own rows.
    const fresh = mvpRuntime({ store, seed: () => 1, now: () => AT });
    expect(fresh.contentFor(old).units.find((u) => u.id === id)!.base.hp).toBe(stored.row.hp + 5);
    expect(fresh.content.units.find((u) => u.id === id)!.base.hp).toBe(stored.row.hp);
    // The run started before goes on, on its pool.
    expect(startRun(fresh, maks).runId).toBe(run.runId);
    expect(decide(fresh, fresh.store.run(run.runId)!, { kind: "reroll" }).run.phase).toBe("shop");
  });
});
