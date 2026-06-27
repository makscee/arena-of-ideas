/**
 * Approve — move a candidate from the candidates/ pool into the playable
 * approved-units registry (PRD #013 slice 4). The vote gate is out of scope; a
 * human runs `approve <id>` and the candidate's units become draftable in a new
 * run. Approval stamps creator credit and carries the data across the seam — but
 * it does NOT trust the candidate's recorded gate stats. Before promoting, it
 * RE-SIMS the candidate from scratch against the canonical reference meta (PRD
 * #067 slice 3): a candidate whose file claims in-band but whose actual unit data
 * re-sims out-of-band is rejected loudly with the re-sim numbers, and a recorded
 * win-rate that disagrees with the re-sim (the record lied) is rejected too. The
 * recorded number is a receipt, not the verdict — the verdict is re-earned here.
 *
 * Pure core (approveInto): (candidate record, current registry) → new registry.
 * Structure is checked first (there must be new, non-colliding content), then the
 * verdict is re-earned by re-sim before the promotion. The re-sim reuses
 * src/gate.ts unchanged (battle()
 * stays pure/untouched); the reference meta + band/floor come from the same
 * canonical source check-candidate uses, NOT from the candidate's file. The CLI
 * (./approve-cli.ts) wraps it with the filesystem; the pure function is what the
 * unit tests exercise — including the rule that a rejected or merely pending
 * candidate (one never written to candidates/) can never reach the pool, because
 * approveInto only ever sees a parsed, validated CandidateRecord.
 */

import { mergePool, parseApprovedRegistry } from "../registry.js";
import type { ApprovedRegistry, ApprovedUnit } from "../registry.js";
import { runGate } from "../gate.js";
import type { GateReport } from "../gate.js";
import { REFERENCE_META } from "../content/reference-meta.js";
import { defaultGateConfig } from "../check-candidate.js";
import type { AbilityRegistry, StatusRegistry } from "../types.js";
import type { CandidateRecord } from "./provenance.js";

/** Tolerance on |recorded overallWinRate − re-sim overallWinRate|.
 *
 * The re-sim runs the EXACT same gate (src/gate.ts) against the SAME canonical
 * reference meta and the SAME default gate config the honest mint used, and the
 * sim is deterministic (seeded). So an honest record reproduces its win-rate
 * BYTE-IDENTICALLY — the difference is 0. The tolerance therefore only needs to
 * absorb floating-point representation noise, not legitimate sim variance, so it
 * is kept very tight: anything a real run could not produce is a lie. A forged
 * "in-band" stat over truly-overtuned data re-sims far outside this window (a
 * 999-power unit wins ~100%), so it is caught both by the verdict check AND by
 * this mismatch check. 1e-9 is ~9 significant figures — orders of magnitude
 * below any forge worth making, comfortably above f64 round-trip error. */
export const RESIM_WIN_RATE_TOLERANCE = 1e-9;

/** Thrown when the re-sim refuses a candidate (out-of-band, or its recorded
 * stats disagree with the truth). Carries the re-sim report so the CLI can print
 * the real numbers — the loud rejection the brief requires. */
export class ResimRejectedError extends Error {
  constructor(
    message: string,
    readonly report: GateReport,
  ) {
    super(message);
    this.name = "ResimRejectedError";
  }
}

/**
 * Re-sim the candidate from scratch against the canonical reference meta and
 * default gate config — the trust boundary `approve` enforces. Returns the fresh
 * gate report when the candidate genuinely passes AND its recorded win-rate
 * matches the re-sim within tolerance. Throws ResimRejectedError (with the report)
 * otherwise. Pure: same record → same report (the gate is seeded/deterministic).
 *
 * Two independent refusals, both loud:
 *   1. The re-sim verdict is not in-band → the unit data itself is out-of-band,
 *      regardless of what the record claims (closes the 999-power forge).
 *   2. The record's overallWinRate disagrees with the re-sim beyond tolerance →
 *      the record lied about a verdict the data does not actually earn.
 */
export function reSimCandidate(record: CandidateRecord, registry: StatusRegistry, abilities: AbilityRegistry): GateReport {
  // The candidate carries any ability it ships with (#081); merge it onto the
  // shipped registry so the re-sim resolves the candidate's refs.
  const report = runGate(record.units, REFERENCE_META, defaultGateConfig(), registry, { ...abilities, ...record.abilities });
  if (!report.pass) {
    throw new ResimRejectedError(
      `candidate "${record.id}" re-sims OUT OF BAND (${report.verdict}): ` +
        `overall win-rate ${report.overallWinRate.toFixed(4)} vs band ` +
        `${report.band.min}–${report.band.max}` +
        (report.foldedTo.length > 0 ? `, folded to ${report.foldedTo.join(", ")}` : "") +
        ` — the recorded stats (claimed ${record.gate.overallWinRate.toFixed(4)}, ` +
        `${record.gate.verdict}) are not trusted; not promoting.`,
      report,
    );
  }
  const drift = Math.abs(record.gate.overallWinRate - report.overallWinRate);
  if (drift > RESIM_WIN_RATE_TOLERANCE) {
    throw new ResimRejectedError(
      `candidate "${record.id}" recorded win-rate ${record.gate.overallWinRate.toFixed(6)} ` +
        `disagrees with the re-sim ${report.overallWinRate.toFixed(6)} ` +
        `(drift ${drift.toExponential(2)} > tolerance ${RESIM_WIN_RATE_TOLERANCE.toExponential(0)}) ` +
        `— the record lied; not promoting.`,
      report,
    );
  }
  return report;
}

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
  abilities: AbilityRegistry,
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

  // RE-SIM — the trust boundary. Structure is sound (there IS new, non-colliding
  // content); now re-earn the gate verdict from scratch against the canonical
  // reference meta rather than trusting the candidate's recorded stats. An
  // out-of-band team — or one whose record lied about its win-rate — is rejected
  // loudly here (ResimRejectedError, carrying the re-sim numbers) before anything
  // is promoted. The recorded number is a receipt; this is the verdict.
  reSimCandidate(record, registry, abilities);
  // The approved registry carries the candidate's abilities too (#081 — an
  // approved unit travels with its Ability), merged with any already approved.
  const nextAbilities = { ...(current.abilities ?? {}), ...record.abilities };
  const next: ApprovedRegistry = { units: [...priorApproved, ...stamped], abilities: nextAbilities };
  // Re-parse the result through the same gate the web shell reads it with, so a
  // bad approval can never be written: the file is valid by construction.
  return parseApprovedRegistry(next, registry, abilities, "approved-units(after approve)");
}
