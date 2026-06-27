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
 *                                                [--trust-task-gate]
 *
 * <taskDir> is a self-contained creation task (see tasks/<id>/README.md).
 *
 * THE BAR IS OUT-OF-BAND BY DEFAULT (PRD #067 slice 2). The authoritative gate
 * — worker-time convergence and any run acting as the authority — judges a
 * candidate against the TRUSTED tunable defaults (GATE_* in tunables.ts), NOT
 * a gate.json shipped inside the task dir. The task dir is untrusted input: in
 * the server era a submitter ships the whole task, gate.json included, so a
 * malicious `{"bandMin":0,"bandMax":1,"floor":0}` would wave any garbage
 * through. The bar a candidate is judged against must come from the trusted
 * driver, not the untrusted task it's judging.
 *
 * Local dev tuning of a task's gate.json stays reachable — but only behind the
 * explicit `--trust-task-gate` flag (a "this task dir is trusted/dev" opt-in).
 * With the flag, a task gate.json overrides the defaults (the old behaviour);
 * without it, gate.json is ignored. The band is never prose, always a number.
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
import { stressRegistry, stressAbilities } from "./content/stress.js";
import { GATE_BAND_MIN, GATE_BAND_MAX, GATE_MATCHUP_FLOOR, GATE_SEEDS } from "./tunables.js";
import { validateTeamFile, type TeamFile } from "./cli.js";
import { ValidationError } from "./validate.js";
import type { AbilityRegistry, UnitDef } from "./types.js";

// ---------------------------------------------------------------------------
// Gate config — TRUSTED tunable defaults by default; a task gate.json is
// honoured only when the task dir is explicitly trusted (--trust-task-gate).
// ---------------------------------------------------------------------------

/** The default gate config, derived from the tunables — never hardcoded prose.
 * This is the TRUSTED, out-of-band bar: it comes from the driving process's
 * own source (tunables.ts), never from the task dir under judgement. */
export function defaultGateConfig(): GateConfig {
  return { bandMin: GATE_BAND_MIN, bandMax: GATE_BAND_MAX, floor: GATE_MATCHUP_FLOOR, seeds: GATE_SEEDS };
}

/**
 * Resolve the gate config for a task dir.
 *
 * By default (`trustTaskGate` false — the authoritative, untrusted-submission
 * path), the task dir's gate.json is IGNORED and the trusted tunable defaults
 * win. The task dir is untrusted input; the bar it is judged against must not
 * be sourced from it (PRD #067 slice 2). A gate.json present in the task dir
 * has no effect on the verdict.
 *
 * When `trustTaskGate` is true (the explicit "this task dir is trusted/dev"
 * opt-in, e.g. local tuning), a gate.json present in the task dir overrides any
 * subset of { bandMin, bandMax, floor, seeds } over the defaults — the old
 * behaviour, now reachable only on purpose.
 */
export function loadGateConfig(taskDir: string, trustTaskGate = false): GateConfig {
  const base = defaultGateConfig();
  if (!trustTaskGate) return base; // untrusted task dir → trusted out-of-band bar
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
export function loadCandidate(path: string): TeamFile {
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
  // A candidate travels with its Ability (#081); validateTeamFile merges the
  // file's abilities onto the shipped registry and returns both.
  return validateTeamFile(parsed, path);
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
export function checkCandidate(candidate: UnitDef[], config: GateConfig, abilities: AbilityRegistry = stressAbilities): CheckResult {
  // (Loading already ran the validator; this re-check exists so a caller that
  // builds a candidate in-memory still goes through the same loud-failure gate
  // and so the result records "ok" explicitly.) `abilities` carries any ability
  // the candidate ships with (#081), merged onto the shipped registry by load.
  const report = runGate(candidate, REFERENCE_META, config, stressRegistry, abilities);
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

const USAGE = "Usage: check-candidate <taskDir> <candidate.json> [--trust-task-gate]";

function main(): void {
  const argv = process.argv.slice(2);
  // --trust-task-gate (explicit trusted/dev opt-in) is the only flag; strip it
  // out so the positionals are taskDir + candidate as before. Default is the
  // safe, out-of-band bar — a task dir's gate.json is ignored unless trusted.
  const trustTaskGate = argv.includes("--trust-task-gate");
  const args = argv.filter((a) => a !== "--trust-task-gate");
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
  let candidate: TeamFile;
  try {
    config = loadGateConfig(taskDir, trustTaskGate);
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

  const result = checkCandidate(candidate.units, config, candidate.abilities);
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
