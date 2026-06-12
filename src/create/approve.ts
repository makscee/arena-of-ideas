/**
 * Approve — move a candidate from the candidates/ pool into the playable
 * approved-units registry (PRD #013 slice 4). The vote gate is out of scope; a
 * human runs `approve <id>` and the candidate's units become draftable in a new
 * run. Approval is bookkeeping, not judgement: the gauntlet already decided the
 * unit is in-band (the candidate record proves it); this only carries the data
 * across the seam, stamping the creator credit so authorship survives the move.
 *
 * Pure core (approveInto): (candidate record, current registry) → new registry.
 * The CLI (./approve-cli.ts) wraps it with the filesystem; the pure function is
 * what the unit tests exercise — including the rule that a rejected or merely
 * pending candidate (one never written to candidates/) can never reach the pool,
 * because approveInto only ever sees a parsed, validated CandidateRecord.
 */

import { mergePool, parseApprovedRegistry } from "../registry.js";
import type { ApprovedRegistry, ApprovedUnit } from "../registry.js";
import type { StatusRegistry } from "../types.js";
import type { CandidateRecord } from "./provenance.js";

/** Approve a candidate into the registry: append its units (creator credit
 * stamped from the record's provenance), then re-validate the whole merged pool
 * against the registry. A name collision with an already-approved or shipped
 * unit is rejected loudly (mergePool's rule) BEFORE anything is written — an
 * approval never silently shadows an existing unit. Returns the new registry; the
 * input is not mutated.
 *
 * `shippedNames` are the names already in the build's DEFAULT_RUN_POOL — passed
 * in (not imported) so this stays a pure function over its inputs and the
 * collision check covers shipped units too, not only prior approvals. */
export function approveInto(
  current: ApprovedRegistry,
  record: CandidateRecord,
  shippedNames: readonly string[],
  registry: StatusRegistry,
): ApprovedRegistry {
  // A candidate is a team file (1..5 units): the genuinely new creation plus,
  // often, shipped support bodies (e.g. Squire) that flesh out the gauntlet team.
  // Approve only the NEW units — a unit whose name is already in the shipped pool
  // is already draftable, so it is skipped, not collided on. The new units are
  // what authorship attaches to.
  const shipped = new Set(shippedNames);
  const newUnits = record.units.filter((u) => !shipped.has(u.name));
  if (newUnits.length === 0) {
    throw new Error(
      `candidate "${record.id}" introduces no new unit — every unit name is already in the shipped pool`,
    );
  }
  const stamped: ApprovedUnit[] = newUnits.map((u) => ({ ...u, _creator: record.provenance.creator }));
  const priorApproved = current.units;
  // mergePool's guard: a new unit colliding with an already-approved name (or a
  // shipped one, via the stubs) is rejected loudly BEFORE anything is written —
  // an approval never silently shadows an existing unit.
  const shippedStubs = shippedNames.map((name) => ({ name, base: { hp: 0, pwr: 0 } }));
  mergePool([...shippedStubs, ...priorApproved], stamped);
  const next: ApprovedRegistry = { units: [...priorApproved, ...stamped] };
  // Re-parse the result through the same gate the web shell reads it with, so a
  // bad approval can never be written: the file is valid by construction.
  return parseApprovedRegistry(next, registry, "approved-units(after approve)");
}
