// A unit's win and pick rate on its card and its sheet (mission #574). Slice
// 11 owns this file: unitStatsLine shows the rates it is given, else the ones
// kept from the last api.stats() (loadUnitRates; Home and the stats page
// refresh it). card() and unitSheet() already call it, so no caller changes.
import type { StatsView, UnitId } from "../../src/mvp/contract";
import { api } from "../api";
import { h } from "./dom";

export interface UnitRates {
  winRate: number;
  pickRate: number;
}

let known = new Map<UnitId, UnitRates>();

/** Keeps `units`' rates for unitStatsLine (the stats page passes the view it already has). */
export function keepUnitRates(units: StatsView["units"]): void {
  known = new Map(units.map((u) => [u.unitId, { winRate: u.winRate, pickRate: u.pickRate }]));
}

/** Fetches and keeps the rates; on an error the old ones stay (cards just show fewer). */
export async function loadUnitRates(): Promise<StatsView | null> {
  try {
    const s = await api.stats();
    keepUnitRates(s.units);
    return s;
  } catch {
    return null;
  }
}

export const pct = (x: number): string => `${Math.round(x * 100)}%`;

/** "W 54% P 12%" on a card, "Win 54% · Pick 12%" on a sheet; null while no run has counted the unit. */
export function unitStatsLine(unitId?: UnitId, rates?: UnitRates): Node | null {
  const r = rates ?? (unitId !== undefined ? known.get(unitId) : undefined);
  if (!r) return null;
  return h(
    "div",
    { class: "rates", "data-testid": "unit-rates", title: `Win rate ${pct(r.winRate)}, pick rate ${pct(r.pickRate)}` },
    h("span", { class: "w" }, h("span", { class: "long" }, "Win "), h("span", { class: "short" }, "W"), pct(r.winRate)),
    h("span", { class: "long" }, " · "),
    h("span", { class: "short" }, " "),
    h("span", { class: "p" }, h("span", { class: "long" }, "Pick "), h("span", { class: "short" }, "P"), pct(r.pickRate)),
  );
}
