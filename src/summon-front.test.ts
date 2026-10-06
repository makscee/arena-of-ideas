// R3 (#642): a summon enters at the FRONT of its line, so it acts at once.
// Revives still return at the back. The Summon event says `front: true`, and
// boardAt honours it, so logs written before R3 still replay at the back.
import { describe, expect, test } from "vitest";
import { battle, TEAM_SIZE } from "./battle.js";
import { boardAt } from "./board.js";
import type { BattleEvent, BattleInput, UnitDef } from "./types.js";
import { Necromancer, stressAbilities, stressRegistry, Summoner } from "./content/stress.js";

const dummy = (name: string, hp = 10, pwr = 2): UnitDef => ({ name, base: { hp, pwr }, ability: "Strike" });
/** Calls an Imp (1/2) at battle start. */
const caller = (name: string): UnitDef => ({
  name,
  base: { hp: 10, pwr: 1 },
  triggers: [{ kind: "trigger", on: { on: "BattleStart" } }],
  selectors: [{ kind: "holder" }],
  abilities: ["Conjure"],
});
const run = (teamA: UnitDef[], teamB: UnitDef[], seed = 1) =>
  battle({ teamA, teamB, seed, statuses: stressRegistry, abilities: stressAbilities } as BattleInput);
const summons = (log: BattleEvent[]) => log.filter((e): e is Extract<BattleEvent, { type: "Summon" }> => e.type === "Summon");

describe("summons enter at the front", () => {
  test("a battle-start summon is the front in turn 1's PairFaced", () => {
    const log = run([caller("Planter"), dummy("Wall")], [dummy("Foe", 30, 1)]);
    const [s] = summons(log);
    expect(s).toMatchObject({ side: "A", name: "Imp", front: true });
    const pair = log.find((e) => e.type === "PairFaced" && e.turn === 1)!;
    expect(pair.type === "PairFaced" && pair.a).toBe(s!.unit);
    expect(boardAt(log, s!.id).lines.A.map((u) => u.name)).toEqual(["Imp", "Planter", "Wall"]);
  });

  test("a Summoner killed by a strike leaves its Imp as the next turn's front", () => {
    const log = run([Summoner, dummy("Back", 20, 1)], [dummy("Killer", 30, 6)]);
    const death = log.find((e) => e.type === "Death" && e.unit.includes("Summoner"))!;
    const s = summons(log).find((e) => e.id > death.id)!;
    expect(s).toMatchObject({ name: "Imp", front: true });
    const next = log.find((e) => e.type === "PairFaced" && e.id > s.id)!;
    expect(next.type === "PairFaced" && next.a).toBe(s.unit);
  });

  test("two summons in one cascade: the newest is the front", () => {
    const log = run([caller("First"), caller("Second"), dummy("Wall")], [dummy("Foe", 30, 1)]);
    const ss = summons(log);
    expect(ss.length).toBe(2);
    const line = boardAt(log, ss[1]!.id).lines.A.map((u) => u.id);
    expect(line.slice(0, 2)).toEqual([ss[1]!.unit, ss[0]!.unit]);
    expect(line.length).toBe(5);
  });

  test("a full line still skips the summon", () => {
    const team = [caller("Planter"), ...Array.from({ length: TEAM_SIZE - 1 }, (_, i) => dummy(`D${i}`))];
    expect(summons(run(team, [dummy("Foe", 30, 1)]))).toEqual([]);
  });

  test("a revive still returns at the back, without `front`", () => {
    const log = run([dummy("Front", 3, 1), Necromancer, dummy("Back", 20, 1)], [dummy("Killer", 40, 4)]);
    const rev = summons(log).find((e) => e.resurrected)!;
    expect(rev.front).toBeUndefined();
    const line = boardAt(log, rev.id).lines.A;
    expect(line[line.length - 1]!.id).toBe(rev.unit);
  });

  test("an old log without `front` still replays its summon at the back", () => {
    const log = run([caller("Planter"), dummy("Wall")], [dummy("Foe", 30, 1)]);
    const old = log.map((e) => {
      if (e.type !== "Summon") return e;
      const { front: _front, ...rest } = e;
      return rest as BattleEvent;
    });
    const s = summons(old)[0]!;
    expect(boardAt(old, s.id).lines.A.map((u) => u.name)).toEqual(["Planter", "Wall", "Imp"]);
  });
});
