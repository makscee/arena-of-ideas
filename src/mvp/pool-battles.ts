// A battle of the rules v2 pool around one unit (unit batch 1, #1046): the
// unit (sleeping, or awoken with 3 copies) and a pool-mate against three
// other pool units, fought the MVP way. The report page takes its trace lines
// from it and the lab opens it as `#scene=v2-unit-<slug>[-awoken]`, so every
// unit of a batch has a battle where it acts even before it has a fixture.

import type { BattleRecord, MvpContent, PlayerRef, UnitContent } from "./contract.js";
import { MVP_RULES } from "./contract.js";
import { fightLines, toBattleDef } from "./fight.js";
import { lineUnitOf } from "./forms.js";
import { slug } from "./units.js";
import { traceLines } from "../words/trace.js";

export interface PoolBattle {
  record: BattleRecord;
  /** Every trace line of the battle. */
  lines: string[];
  /** The lines that name the unit, in order. */
  own: string[];
}

export const poolBattleId = (name: string, awoken: boolean): string => `v2-unit-${slug(name)}${awoken ? "-awoken" : ""}`;

const pick = (units: UnitContent[], i: number, step: number) => units[(i + step) % units.length]!;

/** Pool-mate offsets from the unit: its teammate, then the three enemies. The
 * first lineup is every unit's scene; the others are tried when the unit
 * never acts in it (a Lone Wolf whose mate outlives it, a Chrysalis that dies
 * before turn 3). */
const LINEUPS: [number, number[]][] = [
  [3, [7, 11, 13]],
  [1, [5, 9, 17]],
  [2, [4, 6, 8]],
  [5, [10, 15, 19]],
  [4, [12, 14, 16]],
  [6, [1, 2, 3]],
];

/** Fights `name` of `content` in a fixed line, trying seeds (then other
 * lineups) until a trace line names the unit (the first seed otherwise), so
 * the battle shows it acting. */
export function poolBattle(content: MvpContent, name: string, awoken: boolean, player: PlayerRef, at: string, contentVersion = content.version): PoolBattle {
  const i = content.units.findIndex((u) => u.name === name);
  if (i < 0) throw new Error(`no unit ${name} in the pool`);
  const u = content.units[i]!;
  let first: PoolBattle | undefined;
  let named: PoolBattle | undefined;
  for (const [mate, enemies] of LINEUPS) {
    const a = [lineUnitOf(u, "a1", awoken ? MVP_RULES.copiesToAwaken : 1), lineUnitOf(pick(content.units, i, mate), "a2")];
    const b = enemies.map((step, j) => lineUnitOf(pick(content.units, i, step), `b${j + 1}`));
    const input = { teamA: a.map(toBattleDef), teamB: b.map(toBattleDef), statuses: content.statuses, abilities: content.abilities };
    for (let seed = 1; seed <= 8; seed++) {
      const record = fightLines({ player, line: a }, { player, line: b }, { battleId: poolBattleId(name, awoken), seed, kind: "round", round: 1, runId: null, at, content, rules: MVP_RULES });
      const lines = traceLines({ ...input, seed }, record.log);
      // the unit's own lines first (it acts); else lines that name it (it is hit, copied, traded)
      const acts = lines.filter((l) => l.startsWith(name));
      const own = acts.length ? acts : lines.filter((l) => l.includes(name));
      const out = { record: { ...record, contentVersion }, lines, own };
      if (acts.length) return out;
      if (own.length) named ??= out;
      first ??= out;
    }
  }
  return named ?? first!;
}
