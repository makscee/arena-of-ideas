/**
 * mint-candidate (PRD #013 slice 4) — `npm run mint-candidate`.
 *
 * Lands a passing creation run into the candidates/ pool with provenance. Points
 * at a creation task directory whose worker run converged (out/run-log.jsonl
 * shows a passed attempt), reads the gate numbers from that log, gathers the
 * idea text + authorship + harness/model, and writes one candidate record file
 * to candidates/<id>.json. A subsequent `npm run approve <id>` makes it playable.
 *
 * Usage (from the repo root):
 *   npm run mint-candidate -- <taskDir> --creator <name> \
 *     [--harness <h>] [--model <m>] [--id <id>] [--idea <text>] [--out <dir>]
 *
 * Provenance sources, honest by construction:
 *   - gate stats + attempts → out/run-log.jsonl (the worker's machine record).
 *   - timestamp            → the run-log file's mtime (the run's own time on
 *                            disk — from the log, not invented).
 *   - idea text            → --idea, else the task's idea.txt.
 *   - creator/harness/model→ flags (--creator required; harness defaults
 *                            "claude-code", model "(default)").
 * If the run never converged, mint refuses (exit 1) — only a passing candidate
 * earns a provenance record. mint holds no game rules: the gate already judged.
 *
 * Exit codes: 0 = candidate written; 1 = usage / no-converge / bad input.
 */

import { mkdirSync, readFileSync, statSync, writeFileSync } from "node:fs";
import { basename, isAbsolute, join, resolve } from "node:path";
import { stressRegistry } from "../content/stress.js";
import { validateTeamFile } from "../cli.js";
import { buildRecord, readConvergedAttempt, serializeRecord } from "./provenance.js";
import type { RunManifest } from "./provenance.js";

const USAGE =
  "Usage: mint-candidate <taskDir> --creator <name> [--harness h] [--model m] [--id id] [--idea text] [--out dir]";

interface Args {
  taskDir: string;
  creator: string;
  harness: string;
  model: string;
  id: string | undefined;
  idea: string | undefined;
  outDir: string | undefined;
}

/** Parse argv. Exported for the unit test. */
export function parseArgs(argv: string[]): Args {
  const positionals: string[] = [];
  let creator: string | undefined;
  let harness = "claude-code";
  let model = "(default)";
  let id: string | undefined;
  let idea: string | undefined;
  let outDir: string | undefined;
  for (let i = 0; i < argv.length; i++) {
    const a = argv[i]!;
    if (a === "--creator") creator = need(argv[++i], "--creator");
    else if (a === "--harness") harness = need(argv[++i], "--harness");
    else if (a === "--model") model = need(argv[++i], "--model");
    else if (a === "--id") id = need(argv[++i], "--id");
    else if (a === "--idea") idea = need(argv[++i], "--idea");
    else if (a === "--out") outDir = need(argv[++i], "--out");
    else if (a.startsWith("--")) throw new Error(`unknown flag: ${a}`);
    else positionals.push(a);
  }
  if (positionals.length !== 1) throw new Error(USAGE);
  if (creator === undefined) throw new Error("--creator <name> is required");
  return { taskDir: positionals[0]!, creator, harness, model, id, idea, outDir };
}

function need(v: string | undefined, flag: string): string {
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
  const id = args.id ?? basename(taskDir);
  const logPath = join(taskDir, "out", "run-log.jsonl");
  const candPath = join(taskDir, "out", "candidate.json");

  // 1) Gate stats + the timestamp come from the run log (file + its mtime).
  let logRaw: string;
  let runTime: string;
  try {
    logRaw = readFileSync(logPath, "utf8");
    runTime = statSync(logPath).mtime.toISOString();
  } catch {
    process.stderr.write(`mint-candidate: no run log at ${logPath} — run the worker first\n`);
    process.exit(1);
  }
  const converged = readConvergedAttempt(logRaw);
  if (converged === null) {
    process.stderr.write(`mint-candidate: run at ${taskDir} never converged — no candidate to mint\n`);
    process.exit(1);
  }

  // 2) The candidate units — validated through the same team-file gate.
  let units;
  try {
    units = validateTeamFile(JSON.parse(readFileSync(candPath, "utf8")), candPath).units;
  } catch (err) {
    process.stderr.write(`mint-candidate: candidate at ${candPath} is invalid: ${(err as Error).message}\n`);
    process.exit(1);
  }

  // 3) Idea text — flag wins, else the task's idea.txt.
  let ideaText = args.idea;
  if (ideaText === undefined) {
    try {
      ideaText = readFileSync(join(taskDir, "idea.txt"), "utf8").trim();
    } catch {
      process.stderr.write(`mint-candidate: no idea.txt in ${taskDir}; pass --idea "<text>"\n`);
      process.exit(1);
    }
  }

  const manifest: RunManifest = {
    ideaText: ideaText!,
    creator: args.creator,
    harness: args.harness,
    model: args.model,
    startedAt: runTime,
  };

  const record = buildRecord(id, units, manifest, converged.result, converged.attempts);

  const candidatesDir = args.outDir
    ? (isAbsolute(args.outDir) ? args.outDir : resolve(repoRoot, args.outDir))
    : join(repoRoot, "candidates");
  mkdirSync(candidatesDir, { recursive: true });
  const dest = join(candidatesDir, `${id}.json`);
  writeFileSync(dest, serializeRecord(record), "utf8");

  process.stderr.write(
    `[mint] wrote ${dest} (creator=${args.creator}, attempts=${converged.attempts}, ` +
      `winRate=${record.gate.overallWinRate.toFixed(3)}, verdict=${record.gate.verdict})\n`,
  );
  process.exit(0);
}

const isMain =
  typeof process !== "undefined" &&
  typeof import.meta?.url === "string" &&
  process.argv[1] !== undefined &&
  import.meta.url.endsWith(process.argv[1].replace(/^.*[\\/]/, ""));

if (isMain) main();
