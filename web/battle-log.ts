// Side lookup off the causal log — unit id → its team side, read from the
// BattleStart rosters and Summon events. The inline battle-log renderer this
// file once carried was retired with the acting-card battle (#082 slice D): the
// running transcript is now the bottom trace strip (acting.ts). The side map it
// relied on stays here — the viewer and the side-tint helpers still read it.

import type { BattleEvent, Side } from "../src/index.js";

export type SideOf = (unitId: string) => Side | undefined;

/** Unit id → side, read off BattleStart rosters and Summon events. */
export function sideMap(log: BattleEvent[]): SideOf {
  const sides = new Map<string, Side>();
  for (const e of log) {
    if (e.type === "BattleStart") {
      for (const side of ["A", "B"] as const) for (const r of e.teams[side]) sides.set(r.id, side);
    } else if (e.type === "Summon") {
      sides.set(e.unit, e.side);
    }
  }
  return (id) => sides.get(id);
}
