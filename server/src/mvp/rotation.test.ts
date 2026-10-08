// M2-10 (mission #735): rotation at the day end. Qualified candidates enter in
// the places of the least played units that stayed long enough; the pool
// never shrinks; a retried day end rotates once; the switch off changes
// nothing; a run started before the rotation finishes on its pool.
import { describe, expect, it } from "vitest";
import { MVP_RULES, type Champion, type PlayerRef } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { creditsView } from "./credits.js";
import { endDay } from "./day.js";
import { seedUnits } from "./pool.js";
import { rotate, rotationPlan } from "./rotation.js";
import { decide, startRun } from "./runs.js";
import { mvpRuntime, type MvpRuntime } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";
import { judgeIdea, seedCandidate } from "./votes.js";

const maks: PlayerRef = { id: "p-maks", name: "Maks", bot: false };
const ann: PlayerRef = { id: "p-ann", name: "Ann", bot: false };
const AT = new Date("2026-10-08T08:00:00.000Z");

function world(opts: { store?: MvpStore; rotation?: boolean } = {}): MvpRuntime {
  const store = opts.store ?? new MemoryMvpStore();
  seedUnits(store, AT);
  for (const p of [maks, ann]) store.addPlayer(p);
  let n = 7;
  return mvpRuntime({ store, seed: () => (n = (n * 1103515245 + 12345) >>> 0), now: () => AT, rotation: opts.rotation ?? true });
}

/** Ends days until day `seq` is the current one. */
function toDay(rt: MvpRuntime, seq: number): void {
  while (rt.today().seq < seq) endDay(rt);
}

/** Maks's candidate copied from `from`, through the check and `votes` votes (all for it). */
function qualifiedCandidate(rt: MvpRuntime, from: string, votes = 5): string {
  const idea = seedCandidate(rt, maks.id, from);
  const unit = rt.store.unit(idea.data.unitId!)!;
  judgeIdea(rt, idea, unit, { pass: true, reason: "fits", row: unit.row });
  for (let i = 0; i < votes; i++) rt.store.addVote({ playerId: `v${i}`, candidateId: unit.unitId, otherId: "fighter", pick: unit.unitId, createdAt: AT.toISOString() });
  return unit.unitId;
}

/** Every live unit picked `n` times on day `seq`, but `few` once each. */
function tallies(rt: MvpRuntime, seq: number, few: string[], n = 9): void {
  rt.store.addDayTallies(seq, { runs: 10, units: rt.content.units.map((u) => ({ unitId: u.id, fights: 10, wins: 5, runs: 2, picks: few.includes(u.id) ? 1 : n })) });
}

describe("rotation at the day end (M2-10)", { timeout: 60_000 }, () => {
  for (const kind of ["memory", "sqlite"] as const) {
    it(`enters qualified candidates in the least played units' places (${kind})`, () => {
      const rt = world({ store: kind === "memory" ? new MemoryMvpStore() : new SqliteMvpStore(":memory:") });
      toDay(rt, 14);
      const before = rt.content;
      const ids = before.units.map((u) => u.id);
      const few = [ids[5]!, ids[20]!];
      tallies(rt, 14, few);
      const a = qualifiedCandidate(rt, ids[0]!);
      const b = qualifiedCandidate(rt, ids[1]!);
      const champ: Champion = { seq: 14, day: rt.today().day, player: ann, line: [lineUnitOf(before.units.find((u) => u.id === few[0])!, "c1")], since: AT.toISOString(), contentVersion: before.version, rating: 1000 };
      rt.store.putChampion(champ);

      endDay(rt);
      const after = rt.content;
      expect(rt.today().seq).toBe(15);
      expect(after.version).not.toBe(before.version);
      expect(after.units).toHaveLength(before.units.length);
      const live = after.units.map((u) => u.id);
      expect(live).toEqual(expect.arrayContaining([a, b]));
      for (const id of few) expect(live).not.toContain(id);
      // Each entrant takes its leaver's place.
      expect(new Set([live[5], live[20]])).toEqual(new Set([a, b]));
      expect(rt.store.currentPool()!.daySeq).toBe(15);
      for (const id of few) {
        expect(rt.store.unit(id)!.status).toBe("library");
        expect(rt.store.stints(id)).toEqual([{ unitId: id, enteredSeq: 1, leftSeq: 15, reason: "rotated" }]);
      }
      for (const id of [a, b]) {
        expect(rt.store.unit(id)).toMatchObject({ status: "live", origin: "idea", authorId: maks.id });
        expect(rt.store.stints(id)).toEqual([{ unitId: id, enteredSeq: 15, leftSeq: null, reason: "idea" }]);
      }
      expect(rt.store.ideas({ playerId: maks.id }).map((i) => i.state)).toEqual(["live", "live"]);
      // NEW with "idea by Maks"; the champion stays.
      const credits = creditsView(rt).units.filter((u) => [a, b].includes(u.unitId));
      expect(credits.map((c) => [c.isNew, c.by?.name])).toEqual([[true, "Maks"], [true, "Maks"]]);
      expect(rt.store.currentChampion()).toMatchObject({ seq: 15, player: { id: ann.id } });
    });
  }

  it("rotates once when a day end is retried", () => {
    const store = new SqliteMvpStore(":memory:");
    const rt = world({ store });
    toDay(rt, 14);
    const ids = rt.content.units.map((u) => u.id);
    tallies(rt, 14, [ids[3]!]);
    const a = qualifiedCandidate(rt, ids[0]!);
    // The day end fails once after the rotation, before the next day is stored.
    const putDay = store.putDay.bind(store);
    let fail = true;
    store.putDay = (d) => {
      if (fail && d.seq === 15) {
        fail = false;
        throw new Error("disk full");
      }
      putDay(d);
    };
    expect(() => endDay(rt)).toThrow("disk full");
    expect(rt.store.currentPool()!.daySeq).toBe(15);
    const pools = store.db.prepare("SELECT COUNT(*) AS n FROM mvp_pools").get() as { n: number };
    // A second candidate qualifies meanwhile: the retry still changes nothing more.
    qualifiedCandidate(rt, ids[1]!);
    endDay(rt);
    expect(rt.today().seq).toBe(15);
    expect((store.db.prepare("SELECT COUNT(*) AS n FROM mvp_pools").get() as { n: number }).n).toBe(pools.n);
    expect(rt.content.units.map((u) => u.id).filter((id) => !ids.includes(id))).toEqual([a]);
    expect(rt.store.stints(ids[3]!)).toEqual([{ unitId: ids[3]!, enteredSeq: 1, leftSeq: 15, reason: "rotated" }]);
    expect(rotate(rt, 14).none).toBe("the pool for day 15 is already stored");
  });

  it("never shrinks the pool: with fewer leavers than entrants, the rest wait", () => {
    const rt = world();
    toDay(rt, 14);
    const ids = rt.content.units.map((u) => u.id);
    // All but one unit re-entered on day 5: too new to leave on day 14.
    for (const id of ids.slice(1)) {
      rt.store.putStint({ unitId: id, enteredSeq: 1, leftSeq: 3, reason: "swapped" });
      rt.store.putStint({ unitId: id, enteredSeq: 5, leftSeq: null, reason: "swapped" });
    }
    const cands = [qualifiedCandidate(rt, ids[1]!, 6), qualifiedCandidate(rt, ids[2]!, 5), qualifiedCandidate(rt, ids[3]!, 5)];
    const plan = rotationPlan(rt);
    expect(plan.swaps.map((s) => s.leaver.unitId)).toEqual([ids[0]]);
    expect(plan.waiting).toHaveLength(2);
    expect(plan.tooNew).toBe(ids.length - 1);
    endDay(rt);
    expect(rt.content.units).toHaveLength(ids.length);
    expect(rt.content.units.map((u) => u.id).filter((id) => cands.includes(id))).toHaveLength(1);
    expect(rt.store.ideas({ state: "voting" })).toHaveLength(2);
  });

  it("keeps every unit at least rules.rotationMinStay days", () => {
    const rt = world();
    toDay(rt, 13);
    const ids = rt.content.units.map((u) => u.id);
    const a = qualifiedCandidate(rt, ids[0]!);
    expect(rotationPlan(rt).none).toBe("no live unit has stayed 14 days yet (the first may leave at the end of day 14)");
    endDay(rt); // day 13 ends: 13 days live, nobody leaves
    expect(rt.content.units.map((u) => u.id)).toEqual(ids);
    endDay(rt); // day 14 ends: 14 days live
    expect(rt.content.units.map((u) => u.id)).toContain(a);
    // A shorter stay is a rule.
    const short = world();
    short.rules = { ...MVP_RULES, rotationMinStay: 2 };
    endDay(short);
    const b = qualifiedCandidate(short, short.content.units[0]!.id);
    endDay(short); // day 2 ends: 2 days live
    expect(short.content.units.map((u) => u.id)).toContain(b);
  });

  it("changes nothing with the switch off", () => {
    const rt = world({ rotation: false });
    toDay(rt, 20);
    const before = rt.store.currentPool();
    qualifiedCandidate(rt, rt.content.units[0]!.id);
    expect(rotationPlan(rt).swaps).toHaveLength(1); // it would
    endDay(rt);
    expect(rt.store.currentPool()).toEqual(before);
    expect(rt.store.ideas({ state: "voting" })).toHaveLength(1);
  });

  it("lets a run started before the rotation finish on its pool", () => {
    const rt = world();
    toDay(rt, 14);
    const a = rt.content;
    const ids = a.units.map((u) => u.id);
    tallies(rt, 14, [ids[0]!]);
    const entering = qualifiedCandidate(rt, ids[1]!);
    let run = startRun(rt, maks);
    expect(run.contentVersion).toBe(a.version);
    endDay(rt);
    expect(rt.content.version).not.toBe(a.version);
    expect(rt.content.units.map((u) => u.id)).not.toContain(ids[0]);

    expect(startRun(rt, maks).runId).toBe(run.runId);
    const seen = new Set<string>();
    for (let i = 0; i < 60; i++) {
      run = { ...rt.store.run(run.runId)!, gold: 99 };
      rt.store.putRun(run);
      const r = decide(rt, run, { kind: "reroll" });
      expect(r.run.phase).toBe("shop");
      for (const o of r.run.offers) seen.add(o.unitId);
    }
    expect(seen.has(entering)).toBe(false);
    expect([...seen].every((id) => ids.includes(id))).toBe(true);
    expect(rt.store.run(run.runId)!.contentVersion).toBe(a.version);
    // A new run starts on the new pool.
    expect(startRun(rt, ann).contentVersion).toBe(rt.content.version);
  });

  it("ranks leavers by play first, then coolness and meta health", () => {
    const rt = world();
    toDay(rt, 14);
    const ids = rt.content.units.map((u) => u.id);
    rt.store.addDayTallies(14, {
      runs: 10,
      units: ids.map((id, i) => ({ unitId: id, fights: 10, wins: i === 1 ? 10 : 5, runs: 2, picks: i < 3 ? 1 : 9 })),
    });
    // ids[2] loses every card it is on.
    for (let i = 0; i < 4; i++) rt.store.addVote({ playerId: `v${i}`, candidateId: "x", otherId: ids[2]!, pick: "x", createdAt: "" });
    const order = rotationPlan(rt).leavers.slice(0, 3).map((l) => l.unitId);
    // Same play: ids[2] (coolness 0) and ids[1] (health 0, it always wins) before ids[0].
    expect(order).toEqual([ids[1], ids[2], ids[0]]);
  });
});
