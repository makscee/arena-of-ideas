// R4-2 (#685): a Summon or a Revive that finds its line full used to do
// nothing and log nothing. It now logs a NoRoom event, caused by the event the
// ability answered and sourced to that ability, so the trace can follow it.
import { describe, expect, test } from "vitest";
import { battle, TEAM_SIZE } from "./battle.js";
import { renderReplay } from "./replay.js";
import type { BattleEvent, BattleInput, UnitDef } from "./types.js";
import { Necromancer, stressAbilities, stressRegistry } from "./content/stress.js";

const dummy = (name: string, hp = 10, pwr = 1): UnitDef => ({ name, base: { hp, pwr }, ability: "Strike" });
/** Calls an Imp at battle start. */
const planter = (name: string): UnitDef => ({
  name,
  base: { hp: 10, pwr: 1 },
  triggers: [{ kind: "trigger", on: { on: "BattleStart" } }],
  selectors: [{ kind: "holder" }],
  abilities: ["Conjure"],
});
/** Calls an Imp when another ally dies. */
const caller: UnitDef = {
  name: "Caller",
  base: { hp: 10, pwr: 1 },
  triggers: [{ kind: "trigger", on: { on: "Death", unit: "otherAlly" } }],
  selectors: [{ kind: "holder" }],
  abilities: ["Conjure"],
};
const run = (teamA: UnitDef[], teamB: UnitDef[], seed = 1) =>
  battle({ teamA, teamB, seed, statuses: stressRegistry, abilities: stressAbilities } as BattleInput);
const noRooms = (log: BattleEvent[]) => log.filter((e): e is Extract<BattleEvent, { type: "NoRoom" }> => e.type === "NoRoom");

describe("No room (R4-2)", () => {
  test("a summon on a full line logs NoRoom, caused by its trigger, and summons nothing", () => {
    const team = [planter("Planter"), ...Array.from({ length: TEAM_SIZE - 1 }, (_, i) => dummy(`Wall${i}`))];
    const log = run(team, [dummy("Foe", 30)]);
    const [nr] = noRooms(log);
    expect(nr).toMatchObject({ type: "NoRoom", unit: "A1:Planter", side: "A", name: "Imp", turn: 0 });
    expect(nr!.revive).toBeUndefined();
    expect(log[nr!.causedBy!]?.type).toBe("BattleStart");
    expect(nr!.source).toMatchObject({ unit: "A1:Planter" });
    expect(log.some((e) => e.type === "Summon" && e.turn === 0)).toBe(false);
  });

  test("a summon with room left logs no NoRoom", () => {
    const log = run([planter("Planter"), dummy("Wall")], [dummy("Foe", 30)]);
    expect(noRooms(log)).toEqual([]);
    expect(log.some((e) => e.type === "Summon")).toBe(true);
  });

  test("a revive that finds the line refilled logs NoRoom with the dead unit", () => {
    // Martyr falls; Caller (in front of Necromancer) fills its slot with an
    // Imp, so Necromancer's revive of Martyr finds no room.
    const log = run([dummy("Martyr", 1), caller, Necromancer, dummy("W1"), dummy("W2")], [dummy("Killer", 40, 3)]);
    const death = log.find((e) => e.type === "Death" && e.unit === "A1:Martyr")!;
    const nr = noRooms(log).find((e) => e.revive === "A1:Martyr")!;
    expect(nr).toMatchObject({ unit: "A3:Necromancer", side: "A", name: "Martyr", causedBy: death.id });
    expect(log.some((e) => e.type === "Summon" && e.resurrected && e.causedBy === death.id)).toBe(false);
  });

  test("NoRoom mutates nothing and no unit reacts to it", () => {
    const team = [planter("Planter"), ...Array.from({ length: TEAM_SIZE - 1 }, (_, i) => dummy(`Wall${i}`))];
    const log = run(team, [dummy("Foe", 30)]);
    const nr = noRooms(log)[0]!;
    expect(log.filter((e) => e.causedBy === nr.id)).toEqual([]);
  });

  test("the replay text says it", () => {
    const team = [planter("Planter"), ...Array.from({ length: TEAM_SIZE - 1 }, (_, i) => dummy(`Wall${i}`))];
    expect(renderReplay(run(team, [dummy("Foe", 30)]))).toMatch(/no room: Planter tries to summon Imp, but side A's line is full/);
  });
});
