// M3-7 (mission #800): the rotation's slots split between ideas and
// evolutions (rules.rotationEvolveShare), a slot one queue can't fill going to
// the other; a version entering as `evolution`, a Library unit returning
// unchanged as `return`; the archetype's other proposals closed; and live
// units served under their stored ids, so a version's runs and stats are its own.
import { describe, expect, it } from "vitest";
import { MVP_RULES, type Idea, type PlayerRef, type UnitId } from "../../../src/mvp/contract.js";
import type { Row } from "../../../src/mvp/units.js";
import { mvpContent } from "./content.js";
import { endDay } from "./day.js";
import { poolContent, seedUnits } from "./pool.js";
import { describePlan, rotationPlan } from "./rotation.js";
import { decide, startRun } from "./runs.js";
import { mvpRuntime, type MvpRuntime } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";

const maks: PlayerRef = { id: "p-maks", name: "Maks", bot: false };
const ann: PlayerRef = { id: "p-ann", name: "Ann", bot: false };
const lev: PlayerRef = { id: "p-lev", name: "Lev", bot: false };
const AT = new Date("2026-10-08T08:00:00.000Z");

function world(opts: { store?: MvpStore; rules?: Partial<typeof MVP_RULES> } = {}): MvpRuntime {
  const store = opts.store ?? new MemoryMvpStore();
  seedUnits(store, AT);
  for (const p of [maks, ann, lev]) store.addPlayer(p);
  let n = 7;
  const rt = mvpRuntime({ store, seed: () => (n = (n * 1103515245 + 12345) >>> 0), now: () => AT, rotation: true });
  if (opts.rules) rt.rules = { ...MVP_RULES, ...opts.rules };
  return rt;
}

/** Every live unit picked 9 times on day `seq`, but `few` once each: they leave first. */
function tallies(rt: MvpRuntime, seq: number, few: UnitId[]): void {
  rt.store.addDayTallies(seq, { runs: 10, units: rt.content.units.map((u) => ({ unitId: u.id, fights: 10, wins: 5, runs: 2, picks: few.includes(u.id) ? 1 : 9 })) });
}

/** `won` of 5 votes for the candidate. */
function votes(rt: MvpRuntime, candidateId: UnitId, won = 5): void {
  const others = rt.content.units.map((u) => u.id);
  for (let i = 0; i < 5; i++) rt.store.addVote({ playerId: `voter-${candidateId}-${i}`, candidateId, otherId: others[i]!, pick: i < won ? candidateId : others[i]!, createdAt: "" });
}

/** A new idea's candidate in `voting`, a copy of live unit `from` renamed, with 5 of 5 votes. */
function idea(rt: MvpRuntime, author: PlayerRef, from: UnitId, name: string, won = 5): UnitId {
  const unitId = name.toLowerCase();
  rt.store.putUnit({ unitId, status: "candidate", row: { ...rt.store.unit(from)!.row, name }, authorId: author.id, origin: "idea", parentId: null, createdAt: "" });
  rt.store.putIdea({ ideaId: `idea-${unitId}`, playerId: author.id, text: "a new one", state: "voting", createdAt: "", data: { unitId } });
  votes(rt, unitId, won);
  return unitId;
}

/** `author`'s version `k` of Library unit `target` in `voting` (its name kept,
 * its id `<target>-<k>`, as freshUnitId makes it), with `won` of 5 votes. */
function version(rt: MvpRuntime, author: PlayerRef, target: UnitId, k: number, won = 5, change: Partial<Row> = {}): UnitId {
  const t = rt.store.unit(target)!;
  const unitId = `${target}-${k}`;
  rt.store.putUnit({ unitId, status: "candidate", row: { ...t.row, hp: t.row.hp + k, ...change }, authorId: author.id, origin: "evolution", parentId: target, rootId: t.rootId ?? t.unitId, createdAt: "" });
  const i: Idea = { ideaId: `idea-${unitId}`, playerId: author.id, text: "make it hit every enemy", state: "voting", createdAt: "", data: { kind: "evolve", target, unitId } };
  rt.store.putIdea(i);
  votes(rt, unitId, won);
  return unitId;
}

const liveIds = (rt: MvpRuntime) => rt.content.units.map((u) => u.id);
const library = (rt: MvpRuntime) => rt.store.units({ status: "library" }).map((u) => u.unitId);

describe("the rotation's slots (M3-7)", { timeout: 60_000 }, () => {
  for (const kind of ["memory", "sqlite"] as const) {
    it(`enters 2 ideas and 1 evolution, a third idea waiting (${kind})`, () => {
      const rt = world({ store: kind === "memory" ? new MemoryMvpStore() : new SqliteMvpStore(":memory:") });
      const ids = liveIds(rt);
      tallies(rt, 1, [ids[4]!, ids[9]!, ids[14]!, ids[19]!]);
      const [rat] = library(rt);
      const a = idea(rt, maks, ids[0]!, "Alpha");
      const b = idea(rt, ann, ids[1]!, "Beta");
      const c = idea(rt, lev, ids[2]!, "Gamma", 4);
      const v = version(rt, ann, rat!, 2, 4);
      const plan = rotationPlan(rt);
      expect(plan.slots).toEqual({ idea: 2, evolution: 1 });
      expect(plan.swaps.map((s) => [s.entrant.unitId, s.slot, s.passed]).sort()).toEqual([[a, "idea", false], [b, "idea", false], [v, "evolution", false]].sort());
      expect(plan.waiting.map((s) => s.unitId)).toEqual([c]);
      const text = describePlan(plan, rt.rules).join("\n");
      expect(text).toContain("at most 3 a day: 2 for ideas, 1 for evolutions and returns");
      expect(text).toContain(`(${v}, version by ${ann.id}; evolution slot)`);
      expect(text).toContain(`(${a}, idea by ${maks.id}; idea slot)`);
      expect(text).toContain(`waits ${rt.store.unit(c)!.row.emoji} Gamma (${c}): qualified, score`);
      expect(text).toContain("today's slots are taken (idea queue)");

      endDay(rt);
      const live = liveIds(rt);
      expect(live).toEqual(expect.arrayContaining([a, b, v]));
      expect(live).not.toContain(c);
      expect(live).toHaveLength(ids.length);
      // Rat v2 is in the shop under its own id, with the root's name.
      expect(rt.content.units.find((u) => u.id === v)!.name).toBe(rt.store.unit(rat!)!.row.name);
      expect(rt.store.unit(v)).toMatchObject({ status: "live", origin: "evolution", authorId: ann.id });
      expect(rt.store.stints(v)).toEqual([{ unitId: v, enteredSeq: 2, leftSeq: null, reason: "evolution" }]);
      expect(rt.store.idea(`idea-${v}`)!.state).toBe("live");
      // The old version stays in the Library; the idea that didn't fit waits.
      expect(rt.store.unit(rat!)!.status).toBe("library");
      expect(rt.store.idea(`idea-${c}`)!.state).toBe("voting");
    });
  }

  it("gives the evolution slot to an idea when no evolution qualified: 3 ideas", () => {
    const rt = world();
    const ids = liveIds(rt);
    tallies(rt, 1, [ids[4]!, ids[9]!, ids[14]!]);
    const [rat] = library(rt);
    const cands = [idea(rt, maks, ids[0]!, "Alpha"), idea(rt, ann, ids[1]!, "Beta"), idea(rt, lev, ids[2]!, "Gamma", 4)];
    version(rt, ann, rat!, 2, 2); // on the cards, not qualified
    const plan = rotationPlan(rt);
    expect(plan.swaps.map((s) => s.entrant.unitId).sort()).toEqual([...cands].sort());
    const passed = plan.swaps.filter((s) => s.passed);
    expect(passed.map((s) => [s.entrant.unitId, s.slot])).toEqual([[cands[2], "evolution"]]);
    expect(describePlan(plan, rt.rules).join("\n")).toContain(`(${cands[2]}, idea by ${lev.id}; evolution slot, passed on: no evolution or return qualified for it)`);
    endDay(rt);
    expect(liveIds(rt)).toEqual(expect.arrayContaining(cands));
    expect(rt.store.stints(cands[2]!)[0]!.reason).toBe("idea");
  });

  it("fills the idea slots with evolutions when no idea qualified", () => {
    const rt = world();
    const ids = liveIds(rt);
    tallies(rt, 1, [ids[4]!, ids[9]!, ids[14]!]);
    const [rat, spike, wither] = library(rt);
    const vs = [version(rt, maks, rat!, 2), version(rt, ann, spike!, 2), version(rt, lev, wither!, 2, 4)];
    const plan = rotationPlan(rt);
    expect(plan.swaps.map((s) => s.entrant.unitId).sort()).toEqual([...vs].sort());
    expect(plan.swaps.filter((s) => s.passed).map((s) => s.slot)).toEqual(["idea", "idea"]);
    expect(describePlan(plan, rt.rules).join("\n")).toContain("idea slot, passed on: no idea qualified for it");
    endDay(rt);
    for (const v of vs) expect(rt.store.stints(v)).toEqual([{ unitId: v, enteredSeq: 2, leftSeq: null, reason: "evolution" }]);
  });

  it("splits by rules.rotationEvolveShare, never entering more than leave", () => {
    const rt = world({ rules: { rotationEvolveShare: 2 } });
    const ids = liveIds(rt);
    tallies(rt, 1, [ids[4]!]); // every seed unit may leave: 3 do
    const [rat, spike] = library(rt);
    const a = idea(rt, maks, ids[0]!, "Alpha");
    idea(rt, ann, ids[1]!, "Beta", 4);
    const vs = [version(rt, maks, rat!, 2), version(rt, ann, spike!, 2)];
    const plan = rotationPlan(rt);
    expect(plan.slots).toEqual({ idea: 1, evolution: 2 });
    expect(plan.swaps.map((s) => s.entrant.unitId).sort()).toEqual([a, ...vs].sort());
  });

  it("returns a Library unit unchanged: a new stint, its proposals closed and not refunded", () => {
    const rt = world();
    const ids = liveIds(rt);
    tallies(rt, 1, [ids[4]!]);
    const [rat] = library(rt);
    const a = version(rt, maks, rat!, 2, 3);
    const b = version(rt, ann, rat!, 3, 3);
    votes(rt, rat!, 5);
    const held = rt.store.ideaCounts(maks.id);
    const plan = rotationPlan(rt);
    expect(plan.swaps.map((s) => [s.entrant.unitId, s.entrant.kind, s.slot])).toEqual([[rat, "unchanged", "evolution"]]);
    expect(describePlan(plan, rt.rules).join("\n")).toContain(`(${rat}, returns unchanged; evolution slot)`);
    endDay(rt);
    expect(liveIds(rt)).toContain(rat);
    expect(rt.store.unit(rat!)).toMatchObject({ status: "live", origin: "seed" });
    expect(rt.store.stints(rat!)).toEqual([{ unitId: rat, enteredSeq: 2, leftSeq: null, reason: "return" }]);
    for (const v of [a, b]) {
      expect(rt.store.unit(v)!.status).toBe("library");
      expect(rt.store.idea(`idea-${v}`)!.state).toBe("library");
    }
    expect(rt.store.ideaCounts(maks.id)).toEqual(held);
  });

  it("closes the archetype's other proposals when a version enters; the next contest's unchanged is a version that was live", () => {
    const rt = world();
    const ids = liveIds(rt);
    tallies(rt, 1, [ids[4]!]);
    const [rat] = library(rt);
    const a = version(rt, maks, rat!, 2);
    const b = version(rt, ann, rat!, 3, 4);
    endDay(rt);
    expect(liveIds(rt)).toContain(a);
    expect(rt.store.unit(b)!.status).toBe("library");
    expect(rt.store.idea(`idea-${b}`)!.state).toBe("library");
    expect(rotationPlan(rt).contests).toEqual([]);
    // a leaves; a new proposal for Rat: "unchanged" is a (the newest version
    // that was live), never b, which lost.
    rt.store.putUnit({ ...rt.store.unit(a)!, status: "library" });
    const c = version(rt, lev, a, 4, 1);
    expect(rotationPlan(rt).contests[0]!.candidates.map((s) => [s.unitId, s.kind]).sort()).toEqual([[a, "unchanged"], [c, "version"]].sort());
  });

  it("never enters a version while its archetype has a live one", () => {
    const rt = world();
    const ids = liveIds(rt);
    tallies(rt, 1, [ids[4]!, ids[9]!]);
    const [rat] = library(rt);
    const a = version(rt, maks, rat!, 2);
    endDay(rt);
    expect(liveIds(rt)).toContain(a);
    // A proposal still in flight when a entered reaches the cards later.
    version(rt, ann, rat!, 3);
    expect(rotationPlan(rt).swaps).toEqual([]);
  });

  it("holds an entering version and a returned unit for the minimum stay", () => {
    const rt = world({ rules: { rotationMinStay: 3 } });
    const ids = liveIds(rt);
    tallies(rt, 1, [ids[4]!, ids[9]!]);
    const [rat, spike] = library(rt);
    const v = version(rt, maks, rat!, 2);
    version(rt, ann, spike!, 2, 3);
    votes(rt, spike!, 5);
    rt.rules = { ...rt.rules, rotationEntrants: 2, rotationEvolveShare: 2 };
    endDay(rt); // day 1 ends: v and Spike enter on day 2
    expect(liveIds(rt)).toEqual(expect.arrayContaining([v, spike]));
    expect(rt.store.stints(spike!)[0]!.reason).toBe("return");
    // Played least from now on, they still stay 3 days (days 2, 3, 4).
    for (const seq of [2, 3, 4]) {
      tallies(rt, seq, [v, spike!]);
      const plan = rotationPlan(rt);
      const leavers = plan.leavers.map((l) => l.unitId);
      if (seq < 4) expect(leavers).not.toEqual(expect.arrayContaining([v]));
      if (seq < 4) expect(leavers).not.toContain(spike);
      else expect(leavers.slice(0, 2).sort()).toEqual([v, spike].sort());
      endDay(rt);
    }
  });

  it("rotates an evolution once when a day end is retried", () => {
    const store = new SqliteMvpStore(":memory:");
    const rt = world({ store });
    const ids = liveIds(rt);
    tallies(rt, 1, [ids[3]!]);
    const [rat] = library(rt);
    const v = version(rt, maks, rat!, 2);
    const lost = version(rt, ann, rat!, 3, 3);
    const putDay = store.putDay.bind(store);
    let fail = true;
    store.putDay = (d) => {
      if (fail && d.seq === 2) {
        fail = false;
        throw new Error("disk full");
      }
      putDay(d);
    };
    expect(() => endDay(rt)).toThrow("disk full");
    const after = { pool: store.currentPool(), units: store.units(), stints: store.stints(), ideas: store.ideas() };
    endDay(rt);
    expect(rt.today().seq).toBe(2);
    expect({ pool: store.currentPool(), units: store.units(), stints: store.stints(), ideas: store.ideas() }).toEqual(after);
    expect(rt.store.stints(v)).toEqual([{ unitId: v, enteredSeq: 2, leftSeq: null, reason: "evolution" }]);
    expect(rt.store.unit(lost)!.status).toBe("library");
  });

  it("lets a run on the old pool finish without the version, while a new run gets it", () => {
    const rt = world();
    const before = rt.content;
    const ids = before.units.map((u) => u.id);
    tallies(rt, 1, [ids[0]!]);
    const [rat] = library(rt);
    const v = version(rt, maks, rat!, 2);
    let run = startRun(rt, maks);
    endDay(rt);
    expect(liveIds(rt)).toContain(v);
    const seen = new Set<string>();
    for (let i = 0; i < 40; i++) {
      run = { ...rt.store.run(run.runId)!, gold: 99 };
      rt.store.putRun(run);
      for (const o of decide(rt, run, { kind: "reroll" }).run.offers) seen.add(o.unitId);
    }
    expect(seen.has(v)).toBe(false);
    expect(rt.store.run(run.runId)!.contentVersion).toBe(before.version);
    expect(startRun(rt, ann).contentVersion).toBe(rt.content.version);
  });
});

describe("live units under their stored ids (M3-7, found by M3-8)", { timeout: 60_000 }, () => {
  it("serves today's pool as before: ids are the slugs, the version and content unchanged", () => {
    const rt = world();
    const code = mvpContent();
    expect(poolContent(rt.store)).toEqual(code);
    expect(rt.content.version).toBe(code.version);
  });

  it("records a version's picks, fights and line under its own id, not its root's", () => {
    const rt = world();
    const ids = liveIds(rt);
    tallies(rt, 1, [ids[0]!]);
    const [rat] = library(rt);
    const v = version(rt, maks, rat!, 2);
    endDay(rt);
    const unit = rt.content.units.find((u) => u.id === v)!;
    expect(unit.name).toBe(rt.store.unit(rat!)!.row.name);
    expect(rt.content.units.some((u) => u.id === rat)).toBe(false);
    expect(poolContent(rt.store).units.map((u) => u.id)).toContain(v);

    let run = startRun(rt, ann);
    for (let i = 0; i < 200 && !run.offers.some((o) => o.unitId === v); i++) {
      run = { ...rt.store.run(run.runId)!, gold: 99 };
      rt.store.putRun(run);
      decide(rt, run, { kind: "reroll" });
      run = rt.store.run(run.runId)!;
    }
    const offer = run.offers.find((o) => o.unitId === v);
    expect(offer).toBeDefined();
    decide(rt, run, { kind: "buy", slot: offer!.slot });
    run = rt.store.run(run.runId)!;
    expect(JSON.stringify(run.line)).toContain(`"${v}"`);
    const tally = rt.store.dayTallies(2).units;
    expect(tally.find((u) => u.unitId === v)?.picks).toBe(1);
    expect(tally.some((u) => u.unitId === rat)).toBe(false);
  });
});
