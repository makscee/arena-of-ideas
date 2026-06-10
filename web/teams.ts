// Saved-team store — named teams in localStorage, round-tripping the CLI's
// team-file JSON ({ "units": UnitDef[] }) exactly. This file owns persistence
// and the file format only; content validity is the kernel validator's job
// (the editor runs it live, the battle tab re-checks before battle()).

import type { UnitDef } from "../src/index.js";

const STORAGE_KEY = "aoi.teams.v1";

/** Saved teams by name. Units are stored as-is — invalid drafts persist too;
 * only battle/export are gated on validity. */
export type SavedTeams = Record<string, UnitDef[]>;

function storage(): Storage | null {
  try {
    return window.localStorage;
  } catch {
    return null; // e.g. storage disabled — the editor still works, nothing persists
  }
}

export function loadTeams(): SavedTeams {
  const raw = storage()?.getItem(STORAGE_KEY);
  if (!raw) return {};
  try {
    const parsed: unknown = JSON.parse(raw);
    if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) return {};
    const out: SavedTeams = {};
    for (const [name, units] of Object.entries(parsed)) {
      if (Array.isArray(units)) out[name] = units as UnitDef[];
    }
    return out;
  } catch {
    return {}; // a corrupt store never blocks the app
  }
}

export function saveTeam(name: string, units: UnitDef[]): void {
  const teams = loadTeams();
  teams[name] = units;
  storage()?.setItem(STORAGE_KEY, JSON.stringify(teams));
}

export function deleteTeam(name: string): void {
  const teams = loadTeams();
  delete teams[name];
  storage()?.setItem(STORAGE_KEY, JSON.stringify(teams));
}

// ---------------------------------------------------------------------------
// Team-file JSON — the CLI's format, byte-compatible both ways.
// ---------------------------------------------------------------------------

/** Serialize units as a CLI team file: `{ "units": [...] }`, pretty-printed. */
export function toTeamFileJson(units: UnitDef[]): string {
  return JSON.stringify({ units }, null, 2) + "\n";
}

/** Parse a team file's text. Mirrors the CLI loader's structural checks
 * (object with a "units" array; extra keys like "_comment" ignored) — content
 * validation happens downstream, in the editor's live validator. */
export function parseTeamFileJson(text: string): UnitDef[] {
  let parsed: unknown;
  try {
    parsed = JSON.parse(text);
  } catch (err) {
    throw new Error(`not valid JSON: ${(err as Error).message}`);
  }
  if (typeof parsed !== "object" || parsed === null || Array.isArray(parsed)) {
    throw new Error('expected a JSON object with a "units" array');
  }
  const units = (parsed as Record<string, unknown>)["units"];
  if (!Array.isArray(units)) {
    throw new Error('missing or non-array "units" field');
  }
  return units as UnitDef[];
}
