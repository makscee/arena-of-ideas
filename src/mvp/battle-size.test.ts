// R4-10 (#693): the battle line holds 8 (MvpRules.battleSize), the team stays
// 5. Sexton + Necromancer fused ("Mortwrought") on a full team of 5 used to
// find no room for the Imp and the revive (no-room.test.ts); with room for 8
// it calls the Imp and revives, and the line never passes 8.
import { describe, expect, it } from "vitest";
import { battle, TEAM_SIZE } from "../battle.js";
import { boardAt } from "../board.js";
import type { BattleEvent } from "../types.js";
import { MVP_RULES, type LineUnit, type MvpContent, type MvpRules, type PlayerRef, type UnitContent } from "./contract.js";
import { fightLines, toBattleDef } from "./fight.js";
import { fuseUnits, lineUnitOf } from "./forms.js";
import { mvpPool } from "./units.js";

const pool = mvpPool();
const content: MvpContent = { version: "battle-size", ...pool };
const p: PlayerRef = { id: "p", name: "p", bot: false };
const unit = (id: string): UnitContent => {
  const u = pool.units.find((x) => x.id === id);
  if (!u) throw new Error(`no unit ${id} in the pool`);
  return u;
};

function team(): LineUnit[] {
  const fused = fuseUnits(lineUnitOf(unit("sexton"), "a0", 3), lineUnitOf(unit("necromancer"), "a9", 3), { name: "Mortwrought", discoveredBy: null }, content, MVP_RULES);
  const fillers = ["squire", "squire", "squire", "squire"].map((id, i) => lineUnitOf(unit(id), `f${i}`, 1));
  return [...fillers, fused];
}

function fight(rules: MvpRules, seed = 1): BattleEvent[] {
  const a = team();
  const b: LineUnit[] = [lineUnitOf(unit("squire"), "b0", 3)].map((u) => ({ ...u, stats: { pwr: 6, hp: 80 } }));
  return fightLines({ player: p, line: a }, { player: p, line: b }, { battleId: "x", seed, kind: "round", round: 9, runId: null, at: "2026-10-07T00:00:00.000Z", content, rules }).log;
}

const of = <T extends BattleEvent["type"]>(log: BattleEvent[], type: T) => log.filter((e): e is Extract<BattleEvent, { type: T }> => e.type === type);

/** Side A's line size right after each Summon (where it peaks). */
const lineSizes = (log: BattleEvent[]) => log.flatMap((e, i) => (e.type === "Summon" && e.side === "A" ? [boardAt(log, i).lines.A.length] : []));

describe("Battle line of 8 (R4-10)", () => {
  it("new runs fight with a line of 8 and a team of 5", () => {
    expect(MVP_RULES.battleSize).toBe(8);
    expect(MVP_RULES.lineSize).toBe(TEAM_SIZE);
  });

  it("Mortwrought on a full team of 5 calls the Imp and revives", () => {
    const log = fight(MVP_RULES);
    expect(team()).toHaveLength(5);
    const summons = of(log, "Summon");
    expect(summons.some((e) => e.name === "Imp")).toBe(true);
    expect(summons.some((e) => e.name === "Ghoul")).toBe(true);
    expect(summons.some((e) => e.resurrected)).toBe(true);
  });

  it("the line grows past 5 but never past 8", () => {
    const log = fight(MVP_RULES);
    const sizes = lineSizes(log);
    expect(Math.max(...sizes)).toBeGreaterThan(TEAM_SIZE);
    expect(Math.max(...sizes)).toBeLessThanOrEqual(8);
  });

  it("a stored run without battleSize keeps the line of 5", () => {
    const { battleSize: _bs, ...old } = MVP_RULES;
    const log = fight(old);
    expect(of(log, "NoRoom").length).toBeGreaterThan(0);
    expect(Math.max(...lineSizes(log))).toBeLessThanOrEqual(TEAM_SIZE);
  });

  it("the team still enters with at most 5, and lineCap can't go below it", () => {
    const six = [...team(), ...team()].slice(0, 6).map(toBattleDef);
    const foe = [toBattleDef(lineUnitOf(unit("squire"), "b0", 1))];
    expect(() => battle({ teamA: six, teamB: foe, seed: 1, lineCap: 8, abilities: content.abilities, statuses: content.statuses })).toThrow(/1\.\.5 units/);
    expect(() => battle({ teamA: foe, teamB: foe, seed: 1, lineCap: 4 })).toThrow(/lineCap/);
  });
});
