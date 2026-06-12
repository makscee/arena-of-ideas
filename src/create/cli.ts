/**
 * Creation worker CLI (PRD #013 slices 2–3) — `npm run create`.
 *
 * Drives a harness at a creation task until it produces an in-band candidate, or
 * fails loudly after the attempt bound. Writes the full bounce log
 * (machine-readable JSONL + a final JSON summary) to the task's out/ directory —
 * that log is the provenance slice 4 consumes, and the evidence a run actually
 * converged (or failed) unattended.
 *
 * Two interchangeable harness adapters sit behind the SAME worker bounce loop
 * (harness-agnosticism is the PRD's bet): `--adapter=claude-code` (default) shells
 * out to the Claude Code CLI; `--adapter=chat` drives any OpenAI-compatible
 * `/v1/chat/completions` endpoint over HTTP. Neither adapter knows game rules —
 * the task README + the gauntlet carry them.
 *
 * Usage (from the repo root):
 *   npm run create -- <taskDir> [--adapter claude-code|chat]
 *     [--max-attempts N] [--model M] [--timeout-ms MS]
 *     claude-code: [--bin PATH]
 *     chat:        [--base-url URL] [--key KEY]
 *                  (key also via OPENAI_API_KEY / DEEPSEEK_API_KEY;
 *                   base-url also via OPENAI_BASE_URL)
 *
 * --bin overrides the Claude Code binary (default the `claude` CLI); a faithful
 * stand-in lets the whole loop run end-to-end when the live CLI is
 * unauthenticated in CI/sandboxes.
 *
 * Examples:
 *   npm run create -- tasks/frostbite-striker
 *   npm run create -- tasks/frostbite-striker --adapter=chat \
 *       --base-url=https://api.deepseek.com --model=deepseek-chat
 *
 * Exit codes: 0 = converged (an attempt passed the gauntlet); 1 = loud failure
 * (bound hit, or a usage error). The worker holds no game rules — it relays the
 * README and the gauntlet's numbers (see ./worker.ts).
 */

import { mkdirSync, writeFileSync } from "node:fs";
import { isAbsolute, join, resolve } from "node:path";
import { runLoop } from "./worker.js";
import type { Harness, WorkerConfig, RunResult } from "./worker.js";
import { claudeCodeHarness } from "./claude-code.js";
import { chatCompletionsHarness } from "./chat-completions.js";
import { subprocessGauntlet } from "./gauntlet.js";

const DEFAULT_MAX_ATTEMPTS = 5;
const DEFAULT_TIMEOUT_MS = 10 * 60 * 1000; // 10 min per headless attempt
const DEFAULT_ADAPTER = "claude-code";
const OUT_REL = join("out", "candidate.json");

const ADAPTERS = ["claude-code", "chat"] as const;
type Adapter = (typeof ADAPTERS)[number];

const USAGE =
  "Usage: create <taskDir> [--adapter claude-code|chat] [--max-attempts N] " +
  "[--model M] [--timeout-ms MS] [--bin PATH] [--base-url URL] [--key KEY]";

interface Args {
  taskDir: string;
  adapter: Adapter;
  maxAttempts: number;
  model: string | undefined;
  timeoutMs: number;
  /** claude-code: override the harness binary (default "claude"). */
  bin: string | undefined;
  /** chat: the OpenAI-compatible endpoint base URL. */
  baseUrl: string | undefined;
  /** chat: the API key (Bearer). */
  key: string | undefined;
}

/** Parse argv into the worker args. A flag is accepted in either `--flag value`
 * or `--flag=value` form. Exported for the unit test. */
export function parseArgs(argv: string[]): Args {
  const positionals: string[] = [];
  let adapter: Adapter = DEFAULT_ADAPTER;
  let maxAttempts = DEFAULT_MAX_ATTEMPTS;
  let timeoutMs = DEFAULT_TIMEOUT_MS;
  let model: string | undefined;
  let bin: string | undefined;
  let baseUrl: string | undefined;
  let key: string | undefined;

  for (let i = 0; i < argv.length; i++) {
    const tok = argv[i]!;
    // Support both "--flag value" and "--flag=value".
    let flag = tok;
    let inlineVal: string | undefined;
    const eq = tok.indexOf("=");
    if (tok.startsWith("--") && eq !== -1) {
      flag = tok.slice(0, eq);
      inlineVal = tok.slice(eq + 1);
    }
    const next = (): string | undefined => (inlineVal !== undefined ? inlineVal : argv[++i]);

    if (flag === "--adapter") adapter = mustAdapter(next());
    else if (flag === "--max-attempts") maxAttempts = mustInt(next(), "--max-attempts");
    else if (flag === "--timeout-ms") timeoutMs = mustInt(next(), "--timeout-ms");
    else if (flag === "--model") model = mustStr(next(), "--model");
    else if (flag === "--bin") bin = mustStr(next(), "--bin");
    else if (flag === "--base-url") baseUrl = mustStr(next(), "--base-url");
    else if (flag === "--key") key = mustStr(next(), "--key");
    else if (tok.startsWith("--")) throw new Error(`unknown flag: ${flag}`);
    else positionals.push(tok);
  }
  if (positionals.length !== 1) throw new Error(USAGE);
  return { taskDir: positionals[0]!, adapter, maxAttempts, model, timeoutMs, bin, baseUrl, key };
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
function mustAdapter(v: string | undefined): Adapter {
  if (v === undefined || !ADAPTERS.includes(v as Adapter)) {
    throw new Error(`--adapter expects one of ${ADAPTERS.join("|")}`);
  }
  return v as Adapter;
}

/** Build the harness the chosen adapter names. Reads endpoint/key from the
 * environment when the flags are absent, so a real chat run is one command +
 * exported key. Throws a usage error if a required chat field is missing.
 * Exported for the unit test (it must stay rules-blind — it only wires
 * transport config). */
export function buildHarness(args: Args, repoRoot: string): Harness {
  if (args.adapter === "chat") {
    const baseUrl = args.baseUrl ?? process.env.OPENAI_BASE_URL;
    const apiKey = args.key ?? process.env.OPENAI_API_KEY ?? process.env.DEEPSEEK_API_KEY;
    if (!baseUrl) {
      throw new Error("--adapter=chat needs --base-url (or OPENAI_BASE_URL)");
    }
    if (!args.model) {
      throw new Error("--adapter=chat needs --model (the endpoint's model id)");
    }
    return chatCompletionsHarness({
      baseUrl,
      model: args.model,
      apiKey,
      repoRoot,
      maxTurns: 12,
      maxTokens: 100_000,
      requestTimeoutMs: args.timeoutMs,
    });
  }
  return claudeCodeHarness({
    repoRoot,
    timeoutMs: args.timeoutMs,
    model: args.model,
    bin: args.bin,
  });
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

  let harness: Harness;
  try {
    harness = buildHarness(args, repoRoot);
  } catch (err) {
    process.stderr.write((err as Error).message + "\n");
    process.exit(1);
  }
  const gauntlet = subprocessGauntlet({
    repoRoot,
    taskDir,
    timeoutMs: args.timeoutMs,
  });

  process.stderr.write(
    `[create] task=${taskDir} adapter=${args.adapter} maxAttempts=${args.maxAttempts} model=${args.model ?? "(default)"}\n`,
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
