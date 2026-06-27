// Team catalogue — the one list of pickable teams (shipped + saved), shared
// by the battle pickers and the gauntlet. Owns the picker-value namespacing,
// the invalid-team marking, and the resolve gate in front of battle() — the
// same role the CLI's team-file loader plays. Zero rules live here: validity
// is the kernel validator's verdict.

import {
  Necromancer,
  Silencer,
  Summoner,
  Venomancer,
  assertValidContent,
  stressAbilities,
  stressRegistry,
  validateTeam,
  type UnitDef,
} from "../src/index.js";
import teamAlphaJson from "../examples/team-alpha.json";
import teamBetaJson from "../examples/team-beta.json";
import { loadTeams } from "./teams.js";

function loadShipped(name: string, units: unknown): UnitDef[] {
  assertValidContent(units, stressRegistry, stressAbilities, name);
  return units;
}

/** The shipped teams: the example files plus a squad of stress-set units. */
export const SHIPPED_TEAMS: Record<string, UnitDef[]> = {
  "Team Alpha (aggro venom)": loadShipped("Team Alpha", teamAlphaJson.units),
  "Team Beta (control/sustain)": loadShipped("Team Beta", teamBetaJson.units),
  "Stress Squad (kernel units)": loadShipped("Stress Squad", [Venomancer, Summoner, Silencer, Necromancer]),
};

// Picker values are namespaced so a saved team may share a shipped team's name.
export const SHIPPED_PREFIX = "shipped:";
export const EDITED_PREFIX = "edited:";

export interface TeamOption {
  value: string; // namespaced picker value
  label: string; // display name, with the invalid marker when it applies
  name: string; // bare display name (for result tables)
  shipped: boolean;
  invalid: boolean;
}

/** All pickable teams: shipped first, then saved teams by name. An invalid
 * saved team stays listed — marked, editable, but blocked from battle by
 * resolveUnits below. */
export function teamOptions(): TeamOption[] {
  const out: TeamOption[] = Object.keys(SHIPPED_TEAMS).map((name) => ({
    value: SHIPPED_PREFIX + name,
    label: name,
    name,
    shipped: true,
    invalid: false,
  }));
  const saved = loadTeams();
  for (const name of Object.keys(saved).sort()) {
    const invalid = validateTeam(saved[name], stressRegistry, stressAbilities, name).length > 0;
    out.push({
      value: EDITED_PREFIX + name,
      label: invalid ? `${name} — ⚠ invalid` : name,
      name,
      shipped: false,
      invalid,
    });
  }
  return out;
}

/** Resolve a picker value to battle-ready units — the gate before battle().
 * Saved teams are re-validated here, so an invalid team never reaches the
 * kernel no matter which view asks. */
export function resolveUnits(value: string): { units: UnitDef[] } | { error: string } {
  if (value.startsWith(SHIPPED_PREFIX)) {
    const units = SHIPPED_TEAMS[value.slice(SHIPPED_PREFIX.length)];
    return units ? { units } : { error: `Unknown shipped team "${value}".` };
  }
  const name = value.slice(EDITED_PREFIX.length);
  const units = loadTeams()[name];
  if (!units) return { error: `Team "${name}" is no longer saved.` };
  const issues = validateTeam(units, stressRegistry, stressAbilities, name);
  if (issues.length > 0) {
    return {
      error: `Team "${name}" is invalid (${issues.length} issue${issues.length === 1 ? "" : "s"}) — fix it in the editor. First: ${issues[0]!.path}: ${issues[0]!.message}`,
    };
  }
  return { units };
}
