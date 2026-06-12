/**
 * Arena of Ideas — creation worker (the bounce loop), PRD #013 slice 2.
 *
 * A thin worker shell that drives a *harness* (an AI agent) at a creation task
 * directory until it produces an in-band candidate, or fails loudly after a
 * bounded number of attempts. The shell knows NOTHING about game rules — it is
 * the contract's transport, not its author:
 *
 *   - it points the harness at the task README (the contract lives there),
 *   - it collects whatever the harness wrote to `out/candidate.json`,
 *   - it runs the gauntlet (`check-candidate`) — that is what carries the rules,
 *   - on a bounce it feeds the gauntlet's own JSON numbers back to the harness
 *     verbatim as the next attempt's prompt, and re-runs,
 *   - it logs every attempt as machine-readable JSON (provenance for slice 4).
 *
 * Two things make this testable without ever calling a real AI: the harness is
 * an injected `Harness` function (a stub in tests; the Claude Code CLI in prod,
 * see ./claude-code.ts), and the gauntlet is an injected `Gauntlet` function
 * (the in-process check in tests; a subprocess of `check-candidate` in prod).
 * The loop's state machine — read output, judge, bounce, bound, log — is pure
 * over those two and is what the unit tests exercise.
 *
 * If the worker finds itself encoding what a "good" unit is, that is a bug: the
 * win-rate band, the per-matchup floor, the schema, the reference meta all live
 * behind the gauntlet. The worker only relays text and numbers.
 */

import { readFileSync } from "node:fs";
import { join } from "node:path";

// ---------------------------------------------------------------------------
// The two injected ports — the only things that touch the outside world.
// ---------------------------------------------------------------------------

/** What the harness is asked to do on one attempt. `kind` is "initial" for the
 * first turn (just the README pointer) or "bounce" for a re-attempt carrying
 * the previous gauntlet result. `feedback` is the verbatim text the worker
 * hands the harness — a README pointer, or the bounce numbers. */
export interface Attempt {
  kind: "initial" | "bounce";
  /** 1-based attempt number. */
  index: number;
  /** The prompt text the worker gives the harness — relayed, never game rules. */
  feedback: string;
}

/** The harness port: run the AI agent for one attempt. It must read the task
 * README, do its work, and write `out/candidate.json` under the task dir. It
 * returns a transcript/handle for the log; the worker does not parse it for
 * meaning — the candidate file and the gauntlet are the source of truth. */
export type Harness = (attempt: Attempt) => Promise<HarnessOutcome>;

export interface HarnessOutcome {
  /** True if the harness process completed without erroring (exit 0). */
  ok: boolean;
  /** A short machine handle for the log — e.g. a session id, or an error tag. */
  handle: string;
  /** Optional raw detail (stderr, error message) for the log — never parsed. */
  detail?: string;
}

/** The gauntlet port: run the checks against the emitted candidate file and
 * return the parsed machine result. In prod this is a `check-candidate`
 * subprocess; in tests it is the in-process `checkCandidate`. The shape is the
 * `check-candidate` machine line. */
export type Gauntlet = (candidatePath: string) => Promise<GauntletResult>;

/** The parsed `check-candidate` machine line (a subset the worker reads). */
export interface GauntletResult {
  status: "passed" | "validator-failed" | "gate-bounced";
  validator: "ok" | string;
  gate: GateNumbers | null;
}

/** Just the numbers a bounce feeds back — mirrors GateReport, kept structural
 * so the worker never imports the gate module (no rules leak in). */
export interface GateNumbers {
  pass: boolean;
  verdict: string;
  overallWinRate: number;
  band: { min: number; max: number };
  floor: number;
  foldedTo: string[];
  matchups: { opponent: string; winRate: number; wins: number; losses: number; draws: number; seeds: number }[];
}

// ---------------------------------------------------------------------------
// Config + log shapes
// ---------------------------------------------------------------------------

export interface WorkerConfig {
  /** Absolute path to the creation task directory (holds README, gate.json). */
  taskDir: string;
  /** Where the harness must emit, relative to taskDir. Default "out/candidate.json". */
  outRel: string;
  /** Maximum attempts before loud failure. Default 5. */
  maxAttempts: number;
}

/** One attempt's record in the run log — fully machine-readable (slice-4
 * provenance). Every field is what *happened*, not a judgement. */
export interface AttemptLog {
  index: number;
  kind: "initial" | "bounce";
  /** The prompt text the worker handed the harness this attempt. */
  feedback: string;
  /** The harness outcome (ok + handle). */
  harness: HarnessOutcome;
  /** "ok" | "missing" | "<read/parse error>" — could the worker read the emit? */
  candidate: "ok" | "missing" | string;
  /** The gauntlet result, when the candidate was readable and checked. */
  gauntlet: GauntletResult | null;
  /** This attempt's terminal-for-the-loop outcome. */
  outcome: "passed" | "bounced" | "harness-error" | "no-candidate";
}

export interface RunResult {
  /** True iff some attempt produced an in-band candidate. */
  converged: boolean;
  /** The attempt index that converged, or null on loud failure. */
  convergedAt: number | null;
  /** Every attempt, in order — the bounce log. */
  attempts: AttemptLog[];
}

// ---------------------------------------------------------------------------
// Prompt building — relay only. No game vocabulary appears here.
// ---------------------------------------------------------------------------

/** The initial prompt: point the harness at the README and the emit target.
 * It names files and the DONE contract location, never a rule. */
export function initialPrompt(taskDir: string, outRel: string): string {
  return [
    `You are a creation worker for a task in this repository.`,
    `Read the task contract first: ${join(taskDir, "README.md")}`,
    `Follow it exactly. The README names the schema, the validator, the examples,`,
    `the self-test command, and the DONE condition — that is the whole contract.`,
    ``,
    `Produce your output at: ${join(taskDir, outRel)}`,
    `(create the directory if needed). Then run the self-test command from the`,
    `README and ensure it exits 0. Do not modify anything outside the out/`,
    `directory. When the self-test exits 0, you are done.`,
  ].join("\n");
}

/** The bounce prompt: hand back the gauntlet's own numbers, verbatim, and ask
 * for a revised candidate. The *only* interpretation the worker adds is the
 * generic "here is the check output, adjust and re-emit" framing — the README
 * already told the harness how to read these numbers (it owns the rules). */
export function bouncePrompt(taskDir: string, outRel: string, result: GauntletResult): string {
  const lines: string[] = [
    `Your previous candidate did not pass the task's self-test. The check output`,
    `(the contract's own verdict — read the README for how to act on it) was:`,
    ``,
    JSON.stringify(result),
    ``,
  ];
  if (result.status === "validator-failed") {
    lines.push(`The validator rejected the candidate: ${result.validator}`);
    lines.push(`Fix the content so it is valid DSL, then re-emit.`);
  } else if (result.gate) {
    lines.push(`The sim gate bounced it (verdict "${result.gate.verdict}").`);
    lines.push(`Adjust per the README's guidance for that verdict and re-emit.`);
  }
  lines.push(``);
  lines.push(`Re-emit the revised candidate at ${join(taskDir, outRel)} and re-run`);
  lines.push(`the self-test until it exits 0.`);
  return lines.join("\n");
}

// ---------------------------------------------------------------------------
// Reading the emitted candidate — a transport concern, not a content one.
// ---------------------------------------------------------------------------

/** Read the emit target as text. Returns "ok" + text, or a transport status the
 * log records ("missing" when absent). Content validity is the gauntlet's job;
 * here we only confirm the harness wrote *something* parseable as JSON, so a
 * truncated/empty write is caught as a transport failure, not silently passed
 * to the gauntlet (which would mislabel it a validator failure). */
export function readEmit(path: string): { ok: true } | { ok: false; status: string } {
  let raw: string;
  try {
    raw = readFileSync(path, "utf8");
  } catch {
    return { ok: false, status: "missing" };
  }
  try {
    JSON.parse(raw);
  } catch (err) {
    return { ok: false, status: `unparseable JSON: ${(err as Error).message}` };
  }
  return { ok: true };
}

// ---------------------------------------------------------------------------
// The bounce loop — the pure state machine over the two ports.
// ---------------------------------------------------------------------------

/**
 * Drive the harness at the task until it converges or the attempt bound is hit.
 * Pure over (harness, gauntlet, readEmit): same ports → same RunResult, so the
 * loop is unit-tested with stubs and never needs a real AI or subprocess.
 */
export async function runLoop(
  config: WorkerConfig,
  harness: Harness,
  gauntlet: Gauntlet,
  read: (path: string) => { ok: true } | { ok: false; status: string } = readEmit,
): Promise<RunResult> {
  const outPath = join(config.taskDir, config.outRel);
  const attempts: AttemptLog[] = [];
  let lastGauntlet: GauntletResult | null = null;

  for (let i = 1; i <= config.maxAttempts; i++) {
    // A "bounce" only when a previous attempt produced gauntlet numbers to feed
    // back. After a harness error or a missing/unreadable emit there are no
    // numbers, so we re-issue the README ask (an "initial" prompt) — the kind
    // tracks *what we can say*, not merely the attempt index.
    const kind: Attempt["kind"] = lastGauntlet === null ? "initial" : "bounce";
    const feedback =
      kind === "initial"
        ? initialPrompt(config.taskDir, config.outRel)
        : bouncePrompt(config.taskDir, config.outRel, lastGauntlet!);

    const harnessOutcome = await harness({ kind, index: i, feedback });

    // Harness errored — log it and try again (a flaky run is not a content
    // failure; bounded retries still apply).
    if (!harnessOutcome.ok) {
      attempts.push({
        index: i, kind, feedback, harness: harnessOutcome,
        candidate: "harness-error", gauntlet: null, outcome: "harness-error",
      });
      continue;
    }

    // Did the harness emit a readable candidate?
    const emit = read(outPath);
    if (!emit.ok) {
      attempts.push({
        index: i, kind, feedback, harness: harnessOutcome,
        candidate: emit.status, gauntlet: null, outcome: "no-candidate",
      });
      // Nothing to bounce on (no numbers); a re-prompt repeats the README ask.
      continue;
    }

    // Run the gauntlet — the rules-bearing check.
    const result = await gauntlet(outPath);
    lastGauntlet = result;

    if (result.status === "passed") {
      attempts.push({
        index: i, kind, feedback, harness: harnessOutcome,
        candidate: "ok", gauntlet: result, outcome: "passed",
      });
      return { converged: true, convergedAt: i, attempts };
    }

    attempts.push({
      index: i, kind, feedback, harness: harnessOutcome,
      candidate: "ok", gauntlet: result, outcome: "bounced",
    });
  }

  // Bound hit without a pass — loud failure (the caller exits non-zero).
  return { converged: false, convergedAt: null, attempts };
}
