/**
 * Creation worker CLI (PRD #013 slice 2) — `npm run create`.
 *
 * Drives the Claude Code headless harness at a creation task until it produces
 * an in-band candidate, or fails loudly after the attempt bound. Writes the full
 * bounce log (machine-readable JSONL + a final JSON summary) to the task's out/
 * directory — that log is the provenance slice 4 consumes, and the evidence a
 * run actually converged (or failed) unattended.
 *
 * Usage (from the repo root):
 *   npm run create -- <taskDir> [--max-attempts N] [--model M] [--timeout-ms MS]
 *
 * Example:
 *   npm run create -- tasks/frostbite-striker
 *
 * Exit codes: 0 = converged (an attempt passed the gauntlet); 1 = loud failure
 * (bound hit, or a usage error). The worker holds no game rules — it relays the
 * README and the gauntlet's numbers (see ./worker.ts).
 */

import { mkdirSync, writeFileSync } from "node:fs";
import { isAbsolute, join, resolve } from "node:path";
import { runLoop } from "./worker.js";
import type { WorkerConfig, RunResult } from "./worker.js";
import { claudeCodeHarness } from "./claude-code.js";
import { subprocessGauntlet } from "./gauntlet.js";

const DEFAULT_MAX_ATTEMPTS = 5;
const DEFAULT_TIMEOUT_MS = 10 * 60 * 1000; // 10 min per headless attempt
const OUT_REL = join("out", "candidate.json");

const USAGE =
  "Usage: create <taskDir> [--max-attempts N] [--model M] [--timeout-ms MS]";

interface Args {
  taskDir: string;
  maxAttempts: number;
  model: string | undefined;
  timeoutMs: number;
}

/** Parse argv into the worker args. Exported for the unit test. */
export function parseArgs(argv: string[]): Args {
  const positionals: string[] = [];
  let maxAttempts = DEFAULT_MAX_ATTEMPTS;
  let timeoutMs = DEFAULT_TIMEOUT_MS;
  let model: string | undefined;

  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]!;
    if (a === "--max-attempts") maxAttempts = mustInt(argv[++i], "--max-attempts");
    else if (a === "--timeout-ms") timeoutMs = mustInt(argv[++i], "--timeout-ms");
    else if (a === "--model") model = mustStr(argv[++i], "--model");
    else if (a.startsWith("--")) throw new Error(`unknown flag: ${a}`);
    else positionals.push(a);
  }
  if (positionals.length !== 1) throw new Error(USAGE);
  return { taskDir: positionals[0]!, maxAttempts, model, timeoutMs };
}

function mustInt(v: string | undefined, flag: string): number {
  const n = Number(v);
  if (v === undefined || !Number.isInteger(n) || n < 1) {
    throw new Error(`${flag} expects a positive integer`);
  }
  return n;
}
function mustStr(v: string | undefined, flag: string): string {
  if (v === undefined) throw new Error(`${flag} expects a value`);
  return v;
}

function main(): void {
  let args: Args;
  try {
    args = parseArgs(process.argv.slice(2));
  } catch (err) {
    process.stderr.write((err as Error).message + "\n");
    process.exit(1);
  }

  const repoRoot = process.cwd();
  const taskDir = isAbsolute(args.taskDir) ? args.taskDir : resolve(repoRoot, args.taskDir);
  const outDir = join(taskDir, "out");
  mkdirSync(outDir, { recursive: true });

  const config: WorkerConfig = {
    taskDir,
    outRel: OUT_REL,
    maxAttempts: args.maxAttempts,
  };

  const harness = claudeCodeHarness({
    repoRoot,
    timeoutMs: args.timeoutMs,
    model: args.model,
  });
  const gauntlet = subprocessGauntlet({
    repoRoot,
    taskDir,
    timeoutMs: args.timeoutMs,
  });

  process.stderr.write(
    `[create] task=${taskDir} maxAttempts=${args.maxAttempts} model=${args.model ?? "(default)"}\n`,
  );

  runLoop(config, harness, gauntlet)
    .then((result) => {
      writeLog(outDir, result);
      const where = join(outDir, "run-log.jsonl");
      if (result.converged) {
        process.stderr.write(
          `[create] CONVERGED at attempt ${result.convergedAt}/${args.maxAttempts}. Log: ${where}\n`,
        );
        process.exit(0);
      } else {
        process.stderr.write(
          `[create] LOUD FAILURE — no in-band candidate in ${args.maxAttempts} attempts. Log: ${where}\n`,
        );
        process.exit(1);
      }
    })
    .catch((err) => {
      process.stderr.write(`[create] worker crashed: ${(err as Error).stack ?? err}\n`);
      process.exit(1);
    });
}

/** Write the bounce log: one JSONL line per attempt + a final summary object. */
function writeLog(outDir: string, result: RunResult): void {
  const jsonl =
    result.attempts.map((a) => JSON.stringify(a)).join("\n") +
    "\n" +
    JSON.stringify({ summary: true, converged: result.converged, convergedAt: result.convergedAt, attempts: result.attempts.length }) +
    "\n";
  writeFileSync(join(outDir, "run-log.jsonl"), jsonl, "utf8");
}

// Run only as the entrypoint (same guard idiom as src/cli.ts / check-candidate).
const isMain =
  typeof process !== "undefined" &&
  typeof import.meta?.url === "string" &&
  process.argv[1] !== undefined &&
  import.meta.url.endsWith(process.argv[1].replace(/^.*[\\/]/, ""));

if (isMain) main();
