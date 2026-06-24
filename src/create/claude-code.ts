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
 * Containment (PRD #067 slice 1): reads are repo-wide (the README points at the
 * schema/validator/describe entry points and examples across the repo), but
 * WRITES are confined to the task's `out/`. The hostile-model hole the old flags
 * left open was `--permission-mode bypassPermissions`, which skips ALL permission
 * checks — under it a model could write anywhere `--add-dir` reached (the whole
 * repo). We replace it with an explicit permission policy:
 *   - `--settings {"permissions":{"allow":["Read","Write(<out>/**)","Edit(<out>/**)"]}}`
 *     allows reads everywhere and Write/Edit only under the task's out/.
 *   - `--permission-mode dontAsk` denies anything not matched by an allow rule
 *     *without prompting* — in a headless -p run a denied write becomes a silent
 *     tool error the model sees, never an interactive hang. (bypassPermissions
 *     would have ignored the allow list entirely.)
 *
 * Flags (verified against `claude --help`, v2.x):
 *   -p / --print                 headless: run the prompt and exit, no TTY UI.
 *   --output-format json         one JSON envelope on stdout (result, is_error,
 *                                session_id, num_turns, cost) — parseable.
 *   --permission-mode dontAsk    auto-deny anything outside the allow list, no
 *                                prompt — safe for an unattended sandbox run.
 *   --settings <json>            the read-wide / write-to-out permission policy.
 *   --add-dir <repoRoot>         grant repo-wide read access to the file tools.
 *   --model <model>              optional override (else the subscription default).
 * There is no --max-turns in this CLI; the worker bounds *attempts* itself.
 *
 * The prompt is fed on STDIN (see buildArgs). A hard timeout guards the run.
 */

import { spawn } from "node:child_process";
import type { Attempt, Harness, HarnessOutcome } from "./worker.js";
import { writeAllowGlob } from "./write-jail.js";

export interface ClaudeCodeOptions {
  /** Repo working tree the agent may READ across (cwd + --add-dir). The schema,
   * validator, describe entry points and examples the README points at live all
   * over the repo, so reads stay repo-wide. */
  repoRoot: string;
  /** The task directory whose `out/` is the single writable surface (PRD #067
   * slice 1). Writes outside `<taskDir>/out/` are denied — see buildArgs, which
   * confines Write/Edit to that glob and runs --permission-mode dontAsk so a
   * denied write is a silent tool error, never an interactive hang. */
  taskDir: string;
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

/** The permission policy that contains a hostile model (PRD #067 slice 1):
 * reads everywhere, Write/Edit only under the task's `out/`. Serialised into the
 * `--settings` JSON. Deny rules take precedence over allow in this CLI and a
 * broad deny would block the narrow out/ allow, so we express the jail purely as
 * an allow list + dontAsk mode (anything unmatched is denied). Exported so the
 * unit test can assert the writable surface is the task out/ and nothing wider.
 */
export function buildSettings(opts: ClaudeCodeOptions): {
  permissions: { allow: string[] };
} {
  const out = writeAllowGlob(opts.taskDir); // <taskDir>/out/**
  return {
    permissions: {
      // "Read" (no path) = repo-wide reads; Write/Edit jailed to the task out/.
      allow: ["Read", `Write(${out})`, `Edit(${out})`],
    },
  };
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
    // dontAsk: deny anything outside the allow list without prompting (the
    // headless run has no user to answer). Replaces bypassPermissions, which
    // would have skipped the allow list and left the repo writable.
    "--permission-mode", "dontAsk",
    "--settings", JSON.stringify(buildSettings(opts)),
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
