// A unit's win and pick rate on its card and its sheet (mission #574). Slice
// 11 owns this file and fills unitStatsLine in: the rates it is given, else
// the ones it keeps from api.stats() for unitId. card() and unitSheet()
// already call it, so no caller changes.
import type { UnitId } from "../../src/mvp/contract";

export interface UnitRates {
  winRate: number;
  pickRate: number;
}

/** The rates line, or null when there is nothing to show (always, until slice 11). */
export function unitStatsLine(_unitId?: UnitId, _rates?: UnitRates): Node | null {
  return null;
}
