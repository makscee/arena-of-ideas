/**
 * Arena of Ideas — creation candidate checker (creation loop slice 1).
 *
 * Points at a creation TASK DIRECTORY and a candidate file, and runs the two
 * checks that *are* the creation contract (vision: "the contract is the checks,
 * not the prompt"):
 *
 *   1. VALIDATOR — the candidate is content-valid DSL (src/validate.ts via the
 *      shared team-file gate). A typo'd part fails loudly here, before the gate
 *      ever sims it.
 *   2. SIM GATE — the candidate, swept across N seeds against the reference
 *      meta, lands inside the configured win-rate band (src/gate.ts).
 *
 * Usage:
 *   node --import=tsx/esm src/check-candidate.ts <taskDir> <candidate.json>
 *
 * <taskDir> is a self-contained creation task (see tasks/<id>/README.md). If it
 * contains a gate.json, that config wins; otherwise the tunable defaults
 * (GATE_* in tunables.ts) apply — the band is never prose, always a number.
 *
 * Output is BOTH machine- and human-readable:
 *   - stdout ends with one JSON line: the GateReport plus the validator verdict
 *     (the numbers a later slice's AI loop reads back on a bounce).
 *   - the human transcript precedes it on stdout.
 *   - exit code: 0 = passed both checks; 1 = validator failed; 2 = gate bounced.
 *
 * Deterministic: same candidate + same gate config → byte-identical output
 * (the sweeps are seeded). All logic below the entrypoint is exported for unit
 * testing without spawning a subprocess.
 */

import { readFileSync } from "node:fs";
import { isAbsolute, join } from "node:path";
import { runGate, formatGateReport } from "./gate.js";
import type { GateConfig, GateReport } from "./gate.js";
import { REFERENCE_META } from "./content/reference-meta.js";
import { stressRegistry } from "./content/stress.js";
import { GATE_BAND_MIN, GATE_BAND_MAX, GATE_MATCHUP_FLOOR, GATE_SEEDS } from "./tunables.js";
import { validateTeamFile } from "./cli.js";
import { ValidationError } from "./validate.js";
import type { UnitDef } from "./types.js";

// ---------------------------------------------------------------------------
// Gate config — tunable defaults, overridable per task by a gate.json
// ---------------------------------------------------------------------------

/** The default gate config, derived from the tunables — never hardcoded prose. */
export function defaultGateConfig(): GateConfig {
  return { bandMin: GATE_BAND_MIN, bandMax: GATE_BAND_MAX, floor: GATE_MATCHUP_FLOOR, seeds: GATE_SEEDS };
}

/** Read a task dir's gate.json if present, else the tunable defaults. A
 * gate.json may override any subset of { bandMin, bandMax, floor, seeds }. */
export function loadGateConfig(taskDir: string): GateConfig {
  const base = defaultGateConfig();
  let raw: string;
  try {
    raw = readFileSync(join(taskDir, "gate.json"), "utf8");
  } catch {
    return base; // no per-task override → tunable defaults
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (err) {
    throw new Error(`gate.json in "${taskDir}" is not valid JSON: ${(err as Error).message}`);
  }
  return mergeGateConfig(base, parsed, taskDir);
}

/** Merge a parsed gate.json over the defaults, validating each provided knob. */
export function mergeGateConfig(base: GateConfig, override: unknown, label = "gate.json"): GateConfig {
  if (typeof override !== "object" || override === null || Array.isArray(override)) {
    throw new Error(`${label}: expected a JSON object of gate-config overrides`);
  }
  const o = override as Record<string, unknown>;
  const num = (key: keyof GateConfig, lo: number, hi: number): number => {
    if (o[key] === undefined) return base[key];
    const v = o[key];
    if (typeof v !== "number" || !Number.isFinite(v) || v < lo || v > hi) {
      throw new Error(`${label}: "${key}" must be a number in [${lo}, ${hi}], got ${JSON.stringify(v)}`);
    }
    return v;
  };
  const cfg: GateConfig = {
    bandMin: num("bandMin", 0, 1),
    bandMax: num("bandMax", 0, 1),
    floor: num("floor", 0, 1),
    seeds: num("seeds", 1, 100000),
  };
  if (!Number.isInteger(cfg.seeds)) throw new Error(`${label}: "seeds" must be an integer`);
  if (cfg.bandMin > cfg.bandMax) {
    throw new Error(`${label}: bandMin (${cfg.bandMin}) must not exceed bandMax (${cfg.bandMax})`);
  }
  return cfg;
}

// ---------------------------------------------------------------------------
// Candidate loading — reuses the team-file validator (shared path with cli.ts)
// ---------------------------------------------------------------------------

/** Load + validate a candidate file. The candidate format is a team file:
 * { units: UnitDef[] } (same as examples/), so a candidate is just a team the
 * gate runs as side A. Throws ValidationError on invalid content — the
 * validator path the brief requires a typo to fail loudly through. */
export function loadCandidate(path: string): UnitDef[] {
  let raw: string;
  try {
    raw = readFileSync(path, "utf8");
  } catch (err) {
    throw new Error(`Cannot read candidate "${path}": ${(err as NodeJS.ErrnoException).message}`);
  }
  let parsed: unknown;
  try {
    parsed = JSON.parse(raw);
  } catch (err) {
    throw new Error(`Candidate "${path}" is not valid JSON: ${(err as Error).message}`);
  }
  return validateTeamFile(parsed, path).units;
}

// ---------------------------------------------------------------------------
// The check — validator then gate
// ---------------------------------------------------------------------------

export type CheckStatus = "passed" | "validator-failed" | "gate-bounced";

export interface CheckResult {
  status: CheckStatus;
  /** "ok" when the validator passed; otherwise the validation issues' message. */
  validator: "ok" | string;
  /** The gate report — present only when the validator passed. */
  gate: GateReport | null;
  /** Process exit code: 0 passed, 1 validator failed, 2 gate bounced. */
  exitCode: 0 | 1 | 2;
}

/** Run both checks against an already-loaded candidate. Pure given its inputs. */
export function checkCandidate(candidate: UnitDef[], config: GateConfig): CheckResult {
  // (Loading already ran the validator; this re-check exists so a caller that
  // builds a candidate in-memory still goes through the same loud-failure gate
  // and so the result records "ok" explicitly.)
  const report = runGate(candidate, REFERENCE_META, config, stressRegistry);
  if (report.pass) {
    return { status: "passed", validator: "ok", gate: report, exitCode: 0 };
  }
  return { status: "gate-bounced", validator: "ok", gate: report, exitCode: 2 };
}

/** The machine-readable line: a single JSON object a downstream loop parses. */
export function machineReport(result: CheckResult): string {
  return JSON.stringify({
    status: result.status,
    validator: result.validator,
    gate: result.gate,
  });
}

/** The human transcript — validator verdict, then the gate report. */
export function humanReport(result: CheckResult): string {
  const lines: string[] = [];
  lines.push(`Validator: ${result.validator === "ok" ? "PASS" : "FAIL"}`);
  if (result.validator !== "ok") {
    lines.push(result.validator);
    return lines.join("\n");
  }
  if (result.gate !== null) lines.push(formatGateReport(result.gate));
  return lines.join("\n");
}

// ---------------------------------------------------------------------------
// Entrypoint
// ---------------------------------------------------------------------------

const USAGE = "Usage: check-candidate <taskDir> <candidate.json>";

function main(): void {
  const args = process.argv.slice(2);
  if (args.length < 2) {
    process.stderr.write(USAGE + "\n");
    process.exit(1);
  }
  const [taskDir, candidateArg] = args as [string, string];
  // The DONE condition has the harness emit candidate.json *into the task dir*,
  // and the gauntlet command passes a bare "candidate.json". Resolve a relative
  // candidate path against the task dir so the command in the README works from
  // the repo root; an absolute path is used as-is.
  const candidatePath = isAbsolute(candidateArg) ? candidateArg : join(taskDir, candidateArg);

  let config: GateConfig;
  let candidate: UnitDef[];
  try {
    config = loadGateConfig(taskDir);
    candidate = loadCandidate(candidatePath);
  } catch (err) {
    // Validator (or config) failure: loud, with the issue text, exit 1.
    const isValidation = err instanceof ValidationError;
    const result: CheckResult = {
      status: "validator-failed",
      validator: (err as Error).message,
      gate: null,
      exitCode: 1,
    };
    process.stdout.write(humanReport(result) + "\n");
    process.stdout.write(machineReport(result) + "\n");
    process.exit(isValidation ? 1 : 1);
  }

  const result = checkCandidate(candidate, config);
  process.stdout.write(humanReport(result) + "\n");
  process.stdout.write(machineReport(result) + "\n");
  process.exit(result.exitCode);
}

// Run only as the entrypoint (not when imported by tests) — same guard as cli.ts.
const isMain =
  typeof process !== "undefined" &&
  typeof import.meta?.url === "string" &&
  process.argv[1] !== undefined &&
  import.meta.url.endsWith(process.argv[1].replace(/^.*[\\/]/, ""));

if (isMain) main();
