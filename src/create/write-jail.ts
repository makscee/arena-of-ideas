/**
 * Write-jail — the one place that decides where a creation worker may WRITE.
 * (PRD #067 slice 1: confine worker writes to the task's `out/` directory.)
 *
 * Threat model shift: the creation loop ([[013]]) was built cooperative. A
 * hostile model driving either adapter must not be able to write anywhere it
 * pleases. The honest worker only ever writes the candidate (and its run log)
 * under the task's `out/` directory; everything else it touches it merely READS
 * (the README, idea.txt, the DSL schema/validate/describe entry points, the
 * examples — all repo-wide). So `out/` is the *only* writable surface, and it is
 * the whole repo no longer.
 *
 * Both adapters route their write decision through this module so the jail lives
 * in ONE code path, tested once:
 *   - chat-completions.ts calls `resolveWriteInJail` directly (it does the IO in
 *     JS), and `resolveReadInRepo` for reads (still repo-wide).
 *   - claude-code.ts can't intercept the CLI's file IO, so it confines writes by
 *     handing the CLI a permission allow-glob built by `writeAllowGlob` (reads
 *     stay repo-wide via --add-dir). Same source of truth for "what is `out/`".
 *
 * Game-rules-blind: this module knows nothing about units, win-rates, or gates —
 * only about paths and a writable root.
 */

import { isAbsolute, relative, resolve } from "node:path";

/** A path-escape rejection. Carries the offending path for a loud log. */
export class JailEscapeError extends Error {
  constructor(public readonly attemptedPath: string, reason: string) {
    super(reason);
    this.name = "JailEscapeError";
  }
}

/**
 * Resolve a worker-supplied path against a `root`, rejecting any escape. The
 * defenses (shared by read and write): a `..` segment, an absolute path that
 * lands outside, or a normalize-back escape (`a/../../b`) all resolve to a
 * `relative()` that either starts with `..` or is itself absolute — that is the
 * single, robust test. Returns the absolute in-jail path, or throws
 * `JailEscapeError`.
 *
 * Note this is exactly the defense the repo-scoped jail used before #067; the
 * only change slice 1 makes is *which root* writes are measured against.
 */
export function resolveInRoot(root: string, p: string): string {
  const abs = isAbsolute(p) ? p : resolve(root, p);
  const rel = relative(root, abs);
  if (rel === "" || rel.startsWith("..") || isAbsolute(rel)) {
    throw new JailEscapeError(p, `path escapes jail (${root}): ${p}`);
  }
  return abs;
}

/**
 * Read jail: a worker may read anywhere under the repo working tree (it must
 * reach the schema, validator, describe entry points, examples the README points
 * at). Unchanged from before #067 — slice 1 does not constrain reads. (Locking
 * down gate-config *reads* is slice 2.)
 */
export function resolveReadInRepo(repoRoot: string, p: string): string {
  return resolveInRoot(repoRoot, p);
}

/**
 * Write jail: a worker may write ONLY under `writeRoot` (the task's `out/`
 * directory). A write to the repo root, the task dir root, a `..` escape, an
 * absolute path outside, or a normalize-back escape all throw. A write to
 * `writeRoot` itself (rel === "") is rejected too — the worker writes *files
 * under* out/, never out/ as a file.
 */
export function resolveWriteInJail(writeRoot: string, p: string): string {
  return resolveInRoot(writeRoot, p);
}

/**
 * The task's writable root from its task directory. The honest emit target is
 * `<taskDir>/out/candidate.json`; the writable surface is `<taskDir>/out`.
 */
export function writeRootForTask(taskDir: string): string {
  return resolve(taskDir, "out");
}

/**
 * The Claude Code permission allow-glob that confines writes to the task's
 * `out/` for the claude-code adapter. The CLI matches Write/Edit rules with
 * gitignore-style globs; `//<abs>/**` is an absolute-rooted recursive match, so
 * a model writing anywhere outside `out/` is denied (under --permission-mode
 * dontAsk the denial is a silent tool error, never an interactive hang).
 */
export function writeAllowGlob(taskDir: string): string {
  return `${writeRootForTask(taskDir)}/**`;
}
