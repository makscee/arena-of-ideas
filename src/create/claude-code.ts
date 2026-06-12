/**
 * Claude Code headless adapter — the real `Harness` for the creation worker
 * (PRD #013 slice 2). Spawns the Claude Code CLI in print/headless mode, scoped
 * to the task's working tree, and lets it read the README + write the candidate.
 *
 * It runs on Maks's Claude subscription (the vision's explicit intent): the same
 * `claude` binary the operator uses interactively, here `-p` (non-interactive).
 * The adapter is game-rules-blind: it relays the worker's prompt text and hands
 * back a process outcome. The contract lives in the task README the agent reads.
 *
 * Flags (verified against `claude --help`, v2.1.175):
 *   -p / --print                 headless: run the prompt and exit, no TTY UI.
 *   --output-format json         one JSON envelope on stdout (result, is_error,
 *                                session_id, num_turns, cost) — parseable.
 *   --permission-mode <mode>     "bypassPermissions" for an unattended sandbox
 *                                run (the operator's `claude` alias already
 *                                skips permission prompts; this makes the
 *                                non-interactive run explicit and self-contained).
 *   --add-dir <repoRoot>         scope file tools to the repo working tree.
 *   --model <model>              optional override (else the subscription default).
 * There is no --max-turns in this CLI; the worker bounds *attempts* itself.
 *
 * The prompt is passed as the final positional arg (the CLI reads the prompt
 * from argv in -p mode, not stdin). A hard timeout guards the unattended run.
 */

import { spawn } from "node:child_process";
import type { Attempt, Harness, HarnessOutcome } from "./worker.js";

export interface ClaudeCodeOptions {
  /** Repo working tree to scope the agent to (cwd + --add-dir). */
  repoRoot: string;
  /** Hard per-attempt timeout (ms). A headless run may take minutes. */
  timeoutMs: number;
  /** Optional model override (alias like "opus"/"sonnet", or a full id). */
  model?: string | undefined;
  /** The CLI binary; default "claude". Overridable for a fake in integration. */
  bin?: string | undefined;
  /** Sink for the adapter's own progress lines (default: process.stderr). */
  onProgress?: ((line: string) => void) | undefined;
}

/** The CLI's result envelope, the fields we read for the log. With
 * `--output-format json` the CLI emits a JSON *array* of stream objects; the
 * final `type:"result"` element carries these. */
interface ClaudeEnvelope {
  type?: string;
  is_error?: boolean;
  result?: string;
  session_id?: string;
  num_turns?: number;
  total_cost_usd?: number;
  subtype?: string;
}

/** Extract the result envelope from `claude -p --output-format json` stdout.
 * The output is a JSON array of stream objects (system/assistant/user/result);
 * we take the last `type:"result"` element, else the last element, else — if a
 * bare object was emitted — that object. Returns null if stdout isn't JSON.
 * Exported for the unit test. */
export function parseEnvelope(stdout: string): ClaudeEnvelope | null {
  let parsed: unknown;
  try {
    parsed = JSON.parse(stdout);
  } catch {
    return null;
  }
  if (Array.isArray(parsed)) {
    const objs = parsed.filter((x): x is ClaudeEnvelope => typeof x === "object" && x !== null);
    const result = [...objs].reverse().find((x) => x.type === "result");
    return result ?? objs[objs.length - 1] ?? null;
  }
  if (typeof parsed === "object" && parsed !== null) return parsed as ClaudeEnvelope;
  return null;
}

/** Build the argv for one headless attempt. Exported for the unit test that
 * proves the adapter passes the right flags without spawning anything.
 *
 * The prompt is fed on STDIN, not argv: `--add-dir` is variadic
 * (`<directories...>`) and greedily swallows any trailing positional, so a
 * prompt appended after it is consumed as a directory and the CLI then errors
 * "Input must be provided … when using --print". Stdin sidesteps the
 * flag-parsing entirely and is robust for multi-line prompts. */
export function buildArgs(opts: ClaudeCodeOptions): string[] {
  const args = [
    "-p",
    "--output-format", "json",
    "--permission-mode", "bypassPermissions",
  ];
  if (opts.model) args.push("--model", opts.model);
  // --add-dir is variadic and must be last on argv — nothing may trail it.
  args.push("--add-dir", opts.repoRoot);
  return args;
}

/** Construct the Claude Code `Harness` the worker drives. */
export function claudeCodeHarness(opts: ClaudeCodeOptions): Harness {
  const bin = opts.bin ?? "claude";
  const progress = opts.onProgress ?? ((l: string) => process.stderr.write(l + "\n"));

  return async (attempt: Attempt): Promise<HarnessOutcome> => {
    const args = buildArgs(opts);
    progress(`[claude-code] attempt ${attempt.index} (${attempt.kind}) — spawning ${bin} -p …`);

    return await new Promise<HarnessOutcome>((resolve) => {
      const child = spawn(bin, args, {
        cwd: opts.repoRoot,
        stdio: ["pipe", "pipe", "pipe"],
      });
      // Prompt on stdin (see buildArgs: --add-dir would otherwise eat it).
      child.stdin.write(attempt.feedback);
      child.stdin.end();

      let stdout = "";
      let stderr = "";
      let timedOut = false;

      const timer = setTimeout(() => {
        timedOut = true;
        child.kill("SIGKILL");
      }, opts.timeoutMs);

      child.stdout.on("data", (d) => (stdout += d));
      child.stderr.on("data", (d) => (stderr += d));

      child.on("error", (err) => {
        clearTimeout(timer);
        resolve({ ok: false, handle: "spawn-error", detail: err.message });
      });

      child.on("close", (code) => {
        clearTimeout(timer);
        if (timedOut) {
          resolve({ ok: false, handle: "timeout", detail: `killed after ${opts.timeoutMs}ms` });
          return;
        }
        // Parse the json envelope for a session handle + error flag. A parse
        // failure is itself a harness error (the run did not complete cleanly).
        const env = parseEnvelope(stdout);
        const handle = env?.session_id ? `session:${env.session_id}` : `exit:${code}`;
        progress(
          `[claude-code] attempt ${attempt.index} done — ${handle}` +
            (env?.num_turns !== undefined ? ` (${env.num_turns} turns` : "") +
            (env?.total_cost_usd !== undefined ? `, $${env.total_cost_usd.toFixed(4)})` : env?.num_turns !== undefined ? ")" : ""),
        );
        if (code !== 0 || env?.is_error) {
          resolve({
            ok: false,
            handle: env?.is_error ? "api-error" : `exit-${code}`,
            detail: (env?.result ?? stderr ?? "").slice(0, 2000),
          });
          return;
        }
        const detail = env?.result?.slice(0, 2000);
        resolve(detail === undefined ? { ok: true, handle } : { ok: true, handle, detail });
      });
    });
  };
}
