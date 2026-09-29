// Approved-units resolution for the web shell (PRD #013 slice 4). The playable
// pool a new run drafts from is the shipped DEFAULT_RUN_POOL plus whatever the
// creation loop has APPROVED — `npm run approve` writes those into the committed
// registry/approved-units.json, which the build imports here.
//
// A localStorage override (aoi.approved.v1) is merged ON TOP of the committed
// file when present: it lets the e2e walk approve a fixture candidate without a
// rebuild, and gives a future in-browser approve flow a home. The committed
// file is the source of truth; the override only adds. Both pass the same
// content gate via parseApprovedRegistry — an invalid override is ignored, never
// crashing the app (a bad stored value must not brick a run).

import { parseApprovedRegistry, stressAbilities, stressRegistry, type AbilityRegistry, type ApprovedRegistry, type ApprovedUnit } from "../src/index.js";
import approvedJson from "../registry/approved-units.json";

const OVERRIDE_KEY = "aoi.approved.v1";

/** The committed approved units, validated. A malformed committed file is a
 * build-time bug, so this throws loudly (unlike the override). */
export function committedApproved(): ApprovedUnit[] {
  return committedRegistry().units;
}

function committedRegistry(): ApprovedRegistry {
  return parseApprovedRegistry(approvedJson, stressRegistry, stressAbilities, "registry/approved-units.json");
}

/** Approved units added via the localStorage override, validated. An invalid or
 * absent override yields none — it must never brick the app. */
function overrideApproved(): ApprovedUnit[] {
  return overrideRegistry()?.units ?? [];
}

function overrideRegistry(): ApprovedRegistry | null {
  let raw: string | null;
  try {
    raw = window.localStorage.getItem(OVERRIDE_KEY);
  } catch {
    return null;
  }
  if (raw === null) return null;
  try {
    return parseApprovedRegistry(JSON.parse(raw), stressRegistry, stressAbilities, OVERRIDE_KEY);
  } catch {
    return null; // a corrupt override is ignored, not fatal
  }
}

/** The Abilities logged-out play resolves: the shipped ones plus every Ability
 * an approved Unit brought with it (committed first; the override only adds).
 * Without these, an approved Unit with a new Ability fails the content gate. */
export function approvedAbilities(): AbilityRegistry {
  const out: AbilityRegistry = { ...stressAbilities, ...(committedRegistry().abilities ?? {}) };
  for (const [id, ability] of Object.entries(overrideRegistry()?.abilities ?? {})) {
    if (out[id] === undefined) out[id] = ability;
  }
  return out;
}

/** The full set of approved units the web shell plays with: committed first,
 * then override-added (by new name; an override name that collides with a
 * committed one keeps the committed unit — the override only adds). */
export function approvedUnits(): ApprovedUnit[] {
  const committed = committedApproved();
  const names = new Set(committed.map((u) => u.name));
  const out = [...committed];
  for (const u of overrideApproved()) {
    if (!names.has(u.name)) {
      names.add(u.name);
      out.push(u);
    }
  }
  return out;
}
