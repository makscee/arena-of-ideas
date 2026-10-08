// A unit's win and pick rate (mission #574, slice 11): since round 2 only a
// hint: the last line of its sheet, and the Codex's rate sorts (R2-11).
// unitStatsLine shows the rates it is given, else the ones kept from the last
// api.stats() (loadUnitRates; Home, the Stats page and the Codex refresh it).
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

/** "wins 54% · picked 12%": the one dim line at the bottom of a unit's
 * sheet (round 2: rates are a hint); null while no run has counted the unit.
 * The Codex's Units tab shows them on its cards only while sorted by a rate. */
export function unitStatsLine(unitId?: UnitId, rates?: UnitRates): Node | null {
  const r = rates ?? (unitId !== undefined ? known.get(unitId) : undefined);
  if (!r) return null;
  return h(
    "div",
    {
      class: "rates",
      "data-testid": "unit-rates",
      title: "Win: how often a team with it won its fight. Picked: how often it was on a finished run's line. From every run since the units last changed.",
    },
    `wins ${pct(r.winRate)} · picked ${pct(r.pickRate)}`,
  );
}
