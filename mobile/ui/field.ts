// The battle's fixed field (arena#48, MOVES 2 "Remember the field"): each side has numbered columns, and a unit
// keeps its column for the whole fight. A death leaves a ghost in its column ("× T2", the battle turn it fell);
// nothing slides or compacts. A summon takes a vacancy: the fallen summoner's own column, else the first ghost's,
// else a new column at the back. Pure (events in, columns out): the screen only draws it.
import type { BattleEvent, Side } from "../../src/types";

export interface Cell {
  id: string;
  alive: boolean;
  /** The battle turn this unit fell, for a ghost; null while it stands. */
  fell: number | null;
  /** The unit whose ghost this summon took the place of (it fell at this column before). */
  displaced?: string;
}
export type Field = Record<Side, Cell[]>;

/** The field after events `0..upto` have applied. `skip` holds events that don't apply yet (as boardAt's). */
export function fieldAt(log: BattleEvent[], upto: number, skip?: ReadonlySet<number>): Field {
  const field: Field = { A: [], B: [] };
  const last = Math.min(upto, log.length - 1);
  const cellOf = (id: string): [Side, Cell] | null => {
    for (const side of ["A", "B"] as const) {
      const cell = field[side].find((c) => c.id === id);
      if (cell) return [side, cell];
    }
    return null;
  };
  for (let i = 0; i <= last; i++) {
    const e = log[i];
    if (!e || skip?.has(i)) continue;
    if (e.type === "BattleStart") {
      for (const side of ["A", "B"] as const) field[side] = e.teams[side].map((r) => ({ id: r.id, alive: true, fell: null }));
    } else if (e.type === "Death") {
      const found = cellOf(e.unit);
      if (found) Object.assign(found[1], { alive: false, fell: e.turn });
    } else if (e.type === "Summon") {
      const cols = field[e.side];
      if (e.resurrected) {
        // A revive stands again in its own column; if a summon took it meanwhile, it takes a vacancy like one.
        const own = cols.find((c) => c.id === e.unit);
        if (own) Object.assign(own, { alive: true, fell: null });
        else cols.splice(vacancy(cols, null), 1, { id: e.unit, alive: true, fell: null });
        continue;
      }
      const summoner = e.source !== "kernel" ? e.source.unit : null;
      const at = vacancy(cols, summoner);
      const gone = cols[at];
      cols[at] = { id: e.unit, alive: true, fell: null, ...(gone ? { displaced: gone.id } : {}) };
    }
  }
  return field;
}

/** The column a summon (or a displaced revive) takes: the summoner's own if it has fallen, else the first ghost's, else a new one at the back. */
function vacancy(cols: Cell[], summoner: string | null): number {
  const own = summoner ? cols.findIndex((c) => c.id === summoner && !c.alive) : -1;
  if (own >= 0) return own;
  const first = cols.findIndex((c) => !c.alive);
  return first >= 0 ? first : cols.length;
}
