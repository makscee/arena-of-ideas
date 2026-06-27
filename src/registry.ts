/**
 * Approved-units registry — the playable pool the game draws from on top of the
 * shipped DEFAULT_RUN_POOL (PRD #013 slice 4).
 *
 * The shipped pool (tunables.ts DEFAULT_RUN_POOL) is the bootstrap meta — a pin
 * baked into the build. The *creation loop* adds to the game by APPROVING
 * candidates: a passing candidate (gauntlet-judged, provenance in candidates/)
 * is approved into this registry, which the web run screen merges onto the
 * shipped pool so a new run can draft it. The file is the seam between "an AI
 * made a unit" and "a player can meet it" — data only, validated like all
 * content, the kernel never touched.
 *
 * Creator credit rides on each approved unit as a `_creator` field: a non-DSL
 * key the validator tolerates (it allowlists the fields it knows) and the
 * battle kernel ignores (it reads name/base/level/abilities/statuses). The
 * codex reads it back to show who made the unit. Keeping credit IN the unit
 * data means it travels with the unit through pool, run, and store with no
 * parallel bookkeeping to drift.
 *
 * Pure + framework-free, like the kernel and the validator: merging and parsing
 * are functions of their inputs, so the web shell and the approve CLI share one
 * code path and one definition of "the playable pool".
 */

import { assertValidContent } from "./validate.js";
import type { AbilityRegistry, StatusRegistry, UnitDef } from "./types.js";

/** A unit as stored in the approved registry: a plain UnitDef plus an optional
 * creator credit. `_creator` is data the validator tolerates and the kernel
 * ignores — it exists for display (codex credit line), never for rules. */
export type ApprovedUnit = UnitDef & { _creator?: string };

/** The approved-units file shape: `{ units: ApprovedUnit[] }` — the same team-
 * file envelope examples/ and candidates use, so one parser fits all. */
export interface ApprovedRegistry {
  units: ApprovedUnit[];
  /** Abilities the approved units reference that the shipped registry does not
   * (PRD #081 — an approved unit travels WITH its Ability). Optional: the
   * registry ships empty, and units using only shipped abilities carry none. */
  abilities?: AbilityRegistry;
}

/** Parse + validate the approved-units file contents. Structure is checked
 * loudly (a corrupt registry must never silently drop units), then the units
 * pass the SAME content validator every battle input passes — an approved unit
 * is exactly as gated as a shipped one. An empty registry (`units: []`) is
 * valid: the game ships with no approvals and grows them. */
export function parseApprovedRegistry(data: unknown, registry: StatusRegistry, abilities: AbilityRegistry, label = "approved-units"): ApprovedRegistry {
  if (typeof data !== "object" || data === null || Array.isArray(data)) {
    throw new Error(`${label}: expected a JSON object with a "units" array`);
  }
  const obj = data as Record<string, unknown>;
  if (!Array.isArray(obj["units"])) {
    throw new Error(`${label}: missing or non-array "units" field`);
  }
  const units = obj["units"] as unknown[];
  // The approved units may carry their own abilities (#081); merge onto the
  // shipped registry for the content gate.
  const fileAbilities =
    typeof obj["abilities"] === "object" && obj["abilities"] !== null && !Array.isArray(obj["abilities"])
      ? (obj["abilities"] as AbilityRegistry)
      : {};
  const merged: AbilityRegistry = { ...abilities, ...fileAbilities };
  // Empty is fine; a non-empty registry passes the content gate. The validator
  // reads only the DSL fields, so the `_creator` credit rides along untouched.
  if (units.length > 0) assertValidContent(units, registry, merged, `${label}.units`);
  // Credit, when present, must be a non-empty string — a typo'd credit field is
  // a silent loss of authorship, so it fails loudly like any other content typo.
  units.forEach((u, i) => {
    const cred = (u as Record<string, unknown>)["_creator"];
    if (cred !== undefined && (typeof cred !== "string" || cred.length === 0)) {
      throw new Error(`${label}.units[${i}]._creator must be a non-empty string when present`);
    }
  });
  return { units: units as ApprovedUnit[], ...(Object.keys(fileAbilities).length > 0 ? { abilities: fileAbilities } : {}) };
}

/** Merge the approved units onto a base pool, by name. The base (the shipped
 * DEFAULT_RUN_POOL) comes first so its draws stay stable; an approved unit with
 * a NEW name is appended. A name collision is rejected loudly — an approval must
 * never silently shadow a shipped unit (the shop stacks copies by name, so a
 * shadowed name would merge two different units' copies into one stack). The
 * result is a fresh array; neither input is mutated. */
export function mergePool(base: readonly UnitDef[], approved: readonly ApprovedUnit[]): UnitDef[] {
  const names = new Set(base.map((u) => u.name));
  const out: UnitDef[] = [...base];
  for (const u of approved) {
    if (names.has(u.name)) {
      throw new Error(`approved unit "${u.name}" collides with an existing pool unit — rename before approving`);
    }
    names.add(u.name);
    out.push(u);
  }
  return out;
}

/** The credit map from an approved registry: unit name → creator, for the
 * units that carry a credit. The codex reads this to show authorship. */
export function creditsOf(approved: readonly ApprovedUnit[]): Record<string, string> {
  const out: Record<string, string> = {};
  for (const u of approved) if (u._creator !== undefined) out[u.name] = u._creator;
  return out;
}
