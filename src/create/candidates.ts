/**
 * Candidates pool — the file-backed store of passing candidates with provenance
 * (PRD #013 slice 4). One file per candidate under candidates/, validated on
 * read like all content; the pool is append-only and legible (house style).
 *
 * This module owns the candidate record's persistence + validation only. The
 * record shape and how it's built live in ./provenance.ts; approving a record
 * into the playable registry lives in ./approve.ts. Reading a record re-runs the
 * content validator on its units — a candidate file hand-edited into invalid DSL
 * fails loudly here, never reaches a run.
 */

import { assertValidContent } from "../validate.js";
import type { StatusRegistry, UnitDef } from "../types.js";
import type { CandidateRecord, GateStats } from "./provenance.js";

/** Parse + validate a candidate record from its file contents. Structure is
 * checked loudly, then the units pass the content validator (the same gate a
 * battle input passes). Returns the typed record. Throws on any malformed field
 * or invalid content — a corrupt pool entry must never become a silent bad run. */
export function parseCandidateRecord(data: unknown, registry: StatusRegistry, label: string): CandidateRecord {
  if (typeof data !== "object" || data === null || Array.isArray(data)) {
    throw new Error(`${label}: expected a candidate record object`);
  }
  const o = data as Record<string, unknown>;
  if (typeof o["id"] !== "string" || o["id"].length === 0) {
    throw new Error(`${label}: "id" must be a non-empty string`);
  }
  if (!Array.isArray(o["units"])) {
    throw new Error(`${label}: "units" must be an array`);
  }
  // Content gate: a hand-edited candidate with a typo'd part fails here.
  assertValidContent(o["units"], registry, `${label}.units`);
  const prov = parseProvenance(o["provenance"], label);
  const gate = parseGate(o["gate"], label);
  return { id: o["id"], units: o["units"] as UnitDef[], provenance: prov, gate };
}

function parseProvenance(data: unknown, label: string): CandidateRecord["provenance"] {
  if (typeof data !== "object" || data === null || Array.isArray(data)) {
    throw new Error(`${label}.provenance: expected an object`);
  }
  const p = data as Record<string, unknown>;
  const str = (k: string): string => {
    if (typeof p[k] !== "string" || (p[k] as string).length === 0) {
      throw new Error(`${label}.provenance.${k} must be a non-empty string`);
    }
    return p[k] as string;
  };
  const attempts = p["attempts"];
  if (typeof attempts !== "number" || !Number.isInteger(attempts) || attempts < 1) {
    throw new Error(`${label}.provenance.attempts must be a positive integer`);
  }
  return {
    ideaText: str("ideaText"),
    creator: str("creator"),
    harness: str("harness"),
    model: str("model"),
    timestamp: str("timestamp"),
    attempts,
  };
}

function parseGate(data: unknown, label: string): GateStats {
  if (typeof data !== "object" || data === null || Array.isArray(data)) {
    throw new Error(`${label}.gate: expected an object`);
  }
  const g = data as Record<string, unknown>;
  if (typeof g["verdict"] !== "string") throw new Error(`${label}.gate.verdict must be a string`);
  if (typeof g["overallWinRate"] !== "number") throw new Error(`${label}.gate.overallWinRate must be a number`);
  const band = g["band"] as Record<string, unknown> | undefined;
  if (typeof band !== "object" || band === null || typeof band["min"] !== "number" || typeof band["max"] !== "number") {
    throw new Error(`${label}.gate.band must be { min, max } numbers`);
  }
  if (typeof g["floor"] !== "number") throw new Error(`${label}.gate.floor must be a number`);
  if (!Array.isArray(g["matchups"])) throw new Error(`${label}.gate.matchups must be an array`);
  const matchups = (g["matchups"] as unknown[]).map((m, i) => {
    const mo = m as Record<string, unknown>;
    for (const k of ["opponent"]) if (typeof mo[k] !== "string") throw new Error(`${label}.gate.matchups[${i}].${k} must be a string`);
    for (const k of ["winRate", "wins", "losses", "draws", "seeds"]) {
      if (typeof mo[k] !== "number") throw new Error(`${label}.gate.matchups[${i}].${k} must be a number`);
    }
    return {
      opponent: mo["opponent"] as string,
      winRate: mo["winRate"] as number,
      wins: mo["wins"] as number,
      losses: mo["losses"] as number,
      draws: mo["draws"] as number,
      seeds: mo["seeds"] as number,
    };
  });
  return {
    verdict: g["verdict"],
    overallWinRate: g["overallWinRate"],
    band: { min: band["min"] as number, max: band["max"] as number },
    floor: g["floor"],
    matchups,
  };
}
