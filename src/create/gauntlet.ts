/**
 * Subprocess gauntlet — the prod `Gauntlet` port for the creation worker
 * (PRD #013 slice 2). Runs the task's own self-test (`check-candidate`) as a
 * child process and parses its machine line, so the worker exercises the *same*
 * command the README tells the harness to run — no second code path for "the
 * rules". The gauntlet is the rules; the worker only reads its JSON verdict.
 *
 * Why a subprocess and not an in-process call: the contract is the command. If
 * the worker imported checkCandidate directly it could drift from what the agent
 * runs; spawning `check-candidate` keeps one source of truth and one set of
 * numbers. The last stdout line is the machine JSON (check-candidate emits the
 * human transcript first, the JSON line last).
 */

import { spawnSync } from "node:child_process";
import { relative } from "node:path";
import type { Gauntlet, GauntletResult } from "./worker.js";

export interface GauntletOptions {
  /** Repo working tree — cwd for the check-candidate run. */
  repoRoot: string;
  /** Absolute task directory; the check takes it as its first arg. */
  taskDir: string;
  /** Hard timeout (ms) for one check run. */
  timeoutMs: number;
}

/** Pull the machine line (last non-empty JSON line) out of check-candidate's
 * stdout. Exported for the unit test. Throws if no JSON line is present. */
export function parseMachineLine(stdout: string): GauntletResult {
  const lines = stdout.split("\n").map((l) => l.trim()).filter(Boolean);
  for (let i = lines.length - 1; i >= 0; i--) {
    const line = lines[i]!;
    if (line.startsWith("{")) {
      return JSON.parse(line) as GauntletResult;
    }
  }
  throw new Error("check-candidate produced no machine JSON line");
}

/** Construct the subprocess `Gauntlet`. It runs:
 *   npm run check-candidate -- <taskDir-rel> <candidatePath-rel>
 * from the repo root, matching the README's command exactly. */
export function subprocessGauntlet(opts: GauntletOptions): Gauntlet {
  return async (candidatePath: string): Promise<GauntletResult> => {
    const taskRel = relative(opts.repoRoot, opts.taskDir);
    const candRel = relative(opts.taskDir, candidatePath);
    const run = spawnSync(
      "npm",
      ["run", "--silent", "check-candidate", "--", taskRel, candRel],
      { cwd: opts.repoRoot, encoding: "utf8", timeout: opts.timeoutMs },
    );
    if (run.error) {
      throw new Error(`gauntlet spawn failed: ${run.error.message}`);
    }
    const stdout = run.stdout ?? "";
    try {
      return parseMachineLine(stdout);
    } catch (err) {
      throw new Error(
        `gauntlet output unparseable (exit ${run.status}): ${(err as Error).message}\n` +
          `stdout tail: ${stdout.slice(-500)}\nstderr tail: ${(run.stderr ?? "").slice(-500)}`,
      );
    }
  };
}
