/**
 * Candidate provenance — the record that lands in candidates/ when a worker run
 * (or a manual check-candidate pass) produces a passing candidate (PRD #013
 * slice 4).
 *
 * Provenance answers "where did this unit come from": the idea text a human
 * typed, who gets credit, which harness+model produced it, the gauntlet numbers
 * it passed on (pooled win-rate + every matchup), when the run happened, and how
 * many attempts the bounce loop took. It is bookkeeping over the creation loop's
 * own artifacts — it invents nothing and judges nothing (no game-balance
 * knowledge lives here; the gate already judged, this only records its verdict).
 *
 * The seam with the worker (src/create/worker.ts, owned by the slice-2/3 build):
 * the run log the worker writes (out/run-log.jsonl) carries the per-attempt
 * gauntlet results — that is the gate-stats source. What the log does NOT record
 * — idea text, creator, harness/model, a timestamp — comes from a small
 * `manifest.json` the run writes alongside the log (see writeManifest). This
 * module is PURE: `buildRecord` takes already-parsed inputs and returns the
 * record, so it unit-tests with no filesystem and the round-trip is byte-stable.
 */

import type { GauntletResult } from "./worker.js";
import type { UnitDef } from "../types.js";

// ---------------------------------------------------------------------------
// The run manifest — the provenance the run log alone does not carry.
// ---------------------------------------------------------------------------

/** The non-gauntlet provenance a creation run records about itself. Written by
 * the run (alongside the bounce log) so the timestamp and authorship come from
 * the run, never invented at approve time. */
export interface RunManifest {
  /** The plain-text idea the run was pointed at (the task's idea.txt). */
  ideaText: string;
  /** Who gets authorship credit — the human who supplied the idea. */
  creator: string;
  /** The harness that drove the run, e.g. "claude-code" or "raw-chat". */
  harness: string;
  /** The model the harness used, e.g. "opus" or a full id; "(default)" allowed. */
  model: string;
  /** ISO-8601 timestamp the run started — the provenance timestamp. */
  startedAt: string;
}

// ---------------------------------------------------------------------------
// Gate stats — a structural copy of the gauntlet numbers (no rules imported).
// ---------------------------------------------------------------------------

/** The pooled + per-matchup gate numbers a passing candidate cleared, lifted
 * verbatim from the converging attempt's gauntlet result. Structural (mirrors
 * GateReport) so provenance never imports the gate module — the gate judged
 * already; this is the receipt. */
export interface GateStats {
  verdict: string;
  overallWinRate: number;
  band: { min: number; max: number };
  floor: number;
  matchups: { opponent: string; winRate: number; wins: number; losses: number; draws: number; seeds: number }[];
}

// ---------------------------------------------------------------------------
// The candidate record — one file per candidate in candidates/.
// ---------------------------------------------------------------------------

/** A persisted candidate: the candidate data plus full provenance. One file per
 * candidate in candidates/ — legible, append-only, validated on read. */
export interface CandidateRecord {
  /** Stable id — the basename of the candidate file (e.g. "frostbite-striker"). */
  id: string;
  /** The candidate's units, exactly as the gauntlet passed them. */
  units: UnitDef[];
  provenance: {
    ideaText: string;
    creator: string;
    harness: string;
    model: string;
    /** ISO-8601 — from the run, not invented. */
    timestamp: string;
    /** How many attempts the bounce loop took to converge. */
    attempts: number;
  };
  gate: GateStats;
}

/** The converging attempt's gauntlet result + its 1-based index, pulled from a
 * worker run log (out/run-log.jsonl). A run that never converged has no passing
 * attempt — readConvergedAttempt returns null and the caller refuses to mint. */
export interface ConvergedAttempt {
  result: GauntletResult;
  attempts: number;
}

/** Parse a worker run-log (JSONL: one AttemptLog per line, then a summary line)
 * and return the converging attempt's gauntlet result + the converged-at index,
 * or null if no attempt passed. Pure over the file text — the gate-stats source.
 * The log's own structure is the contract (src/create/worker.ts AttemptLog). */
export function readConvergedAttempt(jsonl: string): ConvergedAttempt | null {
  const lines = jsonl.split("\n").map((l) => l.trim()).filter(Boolean);
  for (const line of lines) {
    let row: unknown;
    try {
      row = JSON.parse(line);
    } catch {
      continue; // a stray line never derails the read
    }
    const r = row as Record<string, unknown>;
    if (r["summary"] === true) continue; // the trailing summary, not an attempt
    if (r["outcome"] === "passed" && r["gauntlet"] !== null && typeof r["gauntlet"] === "object") {
      const result = r["gauntlet"] as GauntletResult;
      if (result.status === "passed") {
        return { result, attempts: typeof r["index"] === "number" ? (r["index"] as number) : lines.length };
      }
    }
  }
  return null;
}

/** Lift the structural gate stats from a converging gauntlet result. Throws if
 * the result has no gate numbers (a candidate with no passing gate is not a
 * candidate — the caller must only ever pass the converging attempt). */
export function gateStatsOf(result: GauntletResult): GateStats {
  if (result.status !== "passed" || result.gate === null) {
    throw new Error(`gateStatsOf: expected a passed gauntlet result, got "${result.status}"`);
  }
  const g = result.gate;
  return {
    verdict: g.verdict,
    overallWinRate: g.overallWinRate,
    band: g.band,
    floor: g.floor,
    matchups: g.matchups.map((m) => ({
      opponent: m.opponent,
      winRate: m.winRate,
      wins: m.wins,
      losses: m.losses,
      draws: m.draws,
      seeds: m.seeds,
    })),
  };
}

/** Build a candidate record from already-parsed inputs. Pure: same inputs →
 * byte-identical record (field order fixed), so the provenance round-trip
 * (write → read → validate) is stable. `units` is the validated candidate; the
 * manifest and the converging gauntlet result supply the rest. */
export function buildRecord(
  id: string,
  units: UnitDef[],
  manifest: RunManifest,
  converged: GauntletResult,
  attempts: number,
): CandidateRecord {
  return {
    id,
    units,
    provenance: {
      ideaText: manifest.ideaText,
      creator: manifest.creator,
      harness: manifest.harness,
      model: manifest.model,
      timestamp: manifest.startedAt,
      attempts,
    },
    gate: gateStatsOf(converged),
  };
}

/** Serialize a record to its on-disk form — pretty-printed JSON with a trailing
 * newline, the house style for committed content (examples/, fixtures/). Field
 * order is fixed by buildRecord, so write→read→write is byte-stable. */
export function serializeRecord(record: CandidateRecord): string {
  return JSON.stringify(record, null, 2) + "\n";
}
