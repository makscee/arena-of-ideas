/**
 * Raw chat-completions adapter — a second `Harness` for the creation worker
 * (PRD #013 slice 3). Where ./claude-code.ts shells out to an agent CLI, this
 * adapter drives ANY OpenAI-compatible `/v1/chat/completions` endpoint (DeepSeek,
 * OpenAI, a local llama.cpp server, …) directly over `fetch`, with a minimal
 * tool loop so a bare completion endpoint can do what an agentic CLI does on its
 * own: read the files the README points at, and write the candidate.
 *
 * Harness-agnosticism is the whole point: the worker drives this adapter through
 * the SAME `Harness` port it drives Claude Code through. One worker attempt =
 * one call into this adapter; inside, the adapter runs a multi-turn conversation
 * (the model reads, reasons, writes, self-tests) until the model says it is done
 * or a turn/token budget is hit. The contract lives in the task README the model
 * reads through the tools — never in this code.
 *
 * Game-rules-blind by construction. The only tools are `read_file` and
 * `write_file`, scoped to the repo working tree. This adapter knows how to move
 * bytes in and out of a sandbox and how to talk the chat-completions wire format
 * — it has no notion of a unit, a status, a win-rate, or a band. Grep it for
 * `Poison`/`band`/`UnitDef`: nothing. The README the model reads carries all of
 * that; the worker's gauntlet enforces it.
 *
 * Multi-turn shape (the brief's "system stays the README pointer relay, bounce
 * numbers as user turns"):
 *   - system message  = the adapter's generic operating instructions (how to use
 *                        the two tools; that the user turn is the task pointer).
 *   - first user turn = the worker's prompt verbatim (initial README pointer, or
 *                        on a re-attempt the bounce numbers — the worker decides
 *                        which; this adapter relays it).
 *   - the loop        = assistant turns request tool calls, the adapter executes
 *                        them and appends tool results, until the assistant
 *                        stops calling tools (it is done) or the budget is spent.
 *
 * Transport failures are loud, never silent: a non-2xx HTTP status, a network
 * error, a malformed completion body, or a budget exhaustion all return an
 * `ok:false` HarnessOutcome with a tagged handle the worker logs. The worker
 * then re-issues the README ask (no gauntlet numbers exist for a failed attempt)
 * and bounded retries still apply — the same degradation path as the CLI adapter.
 */

import { readFileSync, writeFileSync, mkdirSync } from "node:fs";
import { dirname } from "node:path";
import type { Attempt, Harness, HarnessOutcome } from "./worker.js";
import { resolveReadInRepo, resolveWriteInJail } from "./write-jail.js";

// ---------------------------------------------------------------------------
// Config
// ---------------------------------------------------------------------------

export interface ChatCompletionsOptions {
  /** The endpoint base URL, e.g. "https://api.deepseek.com" or
   * "https://api.openai.com/v1". The adapter posts to `${baseUrl}/chat/completions`
   * (a trailing `/v1` in the base is respected; one is added if absent — see
   * resolveEndpoint). */
  baseUrl: string;
  /** The model id the endpoint expects, e.g. "deepseek-chat" or "gpt-4o-mini". */
  model: string;
  /** API key (Bearer). Empty/undefined sends no Authorization header — fine for
   * a keyless local server, a loud 401 for a real provider. */
  apiKey?: string | undefined;
  /** Repo working tree READS are scoped to — the worker must reach the schema,
   * validator, describe entry points and examples the README points at, all
   * over the repo. */
  repoRoot: string;
  /** The single writable root (PRD #067 slice 1): the task's `out/` directory.
   * WRITES are jailed here — a write to the repo, the task dir root, `../`, an
   * absolute path, or a normalize-back escape is rejected loudly. The honest
   * emit target (`out/candidate.json`) lives under it, so the cooperative path
   * is unaffected. */
  writeRoot: string;
  /** Max assistant turns inside one attempt before the adapter gives up loudly.
   * Bounds an endpoint that loops without ever finishing. Default 12. */
  maxTurns: number;
  /** Max total completion tokens (summed across turns) before loud stop.
   * Default 100_000. A budget, not a per-call cap. */
  maxTokens: number;
  /** Per-HTTP-request timeout (ms). Default 120_000. */
  requestTimeoutMs: number;
  /** Injected fetch (the global by default; overridden in tests for the stub). */
  fetchImpl?: typeof fetch;
  /** Sink for the adapter's progress lines (default: process.stderr). */
  onProgress?: ((line: string) => void) | undefined;
}

// ---------------------------------------------------------------------------
// Wire types — the OpenAI chat-completions shape (the subset we depend on).
// ---------------------------------------------------------------------------

interface ChatMessage {
  role: "system" | "user" | "assistant" | "tool";
  content: string | null;
  tool_calls?: ToolCall[];
  tool_call_id?: string;
  name?: string;
}

interface ToolCall {
  id: string;
  type: "function";
  function: { name: string; arguments: string };
}

interface ChatChoice {
  message: { role: "assistant"; content: string | null; tool_calls?: ToolCall[] };
  finish_reason: string | null;
}

interface ChatUsage {
  prompt_tokens?: number;
  completion_tokens?: number;
  total_tokens?: number;
}

interface ChatCompletion {
  choices?: ChatChoice[];
  usage?: ChatUsage;
}

// ---------------------------------------------------------------------------
// The two tools — generic file IO, jailed to the repo. No game vocabulary.
// ---------------------------------------------------------------------------

const TOOLS = [
  {
    type: "function" as const,
    function: {
      name: "read_file",
      description:
        "Read a UTF-8 text file from the repository. Use this to read the task " +
        "README and every file it points at (schema, examples, entry points).",
      parameters: {
        type: "object",
        properties: {
          path: { type: "string", description: "Repo-relative or absolute path to read." },
        },
        required: ["path"],
      },
    },
  },
  {
    type: "function" as const,
    function: {
      name: "write_file",
      description:
        "Write a UTF-8 text file under the task's output directory (creating " +
        "parent directories). Writes are confined to that directory — the path " +
        "the task README names for your output. A write anywhere else is refused.",
      parameters: {
        type: "object",
        properties: {
          path: { type: "string", description: "Path to write (under the task output directory)." },
          content: { type: "string", description: "Full file contents to write." },
        },
        required: ["path", "content"],
      },
    },
  },
];

const SYSTEM_PROMPT = [
  "You are a creation worker operating inside a code repository through two tools:",
  "`read_file(path)` and `write_file(path, content)`. You have no other access —",
  "no shell, no network — so gather everything you need by reading files.",
  "",
  "The user message is your task pointer. Follow it exactly: read the file it",
  "names first, then every file that one points at, until you understand the",
  "contract. Produce the output file the task names with `write_file`. The task's",
  "own self-test (described in what you read) is the sole judge of done — satisfy",
  "it. When you have written the output and believe it satisfies the task, reply",
  "with a short plain-text confirmation and STOP calling tools.",
  "",
  "Do not assume facts not present in the files you read — the files are the",
  "whole contract.",
].join("\n");

// ---------------------------------------------------------------------------
// Endpoint URL resolution
// ---------------------------------------------------------------------------

/** Resolve the chat-completions URL from a base. Accepts bases with or without
 * a trailing `/v1`, with or without a trailing slash. Exported for the test. */
export function resolveEndpoint(baseUrl: string): string {
  const trimmed = baseUrl.replace(/\/+$/, "");
  if (/\/v\d+$/.test(trimmed)) return `${trimmed}/chat/completions`;
  return `${trimmed}/v1/chat/completions`;
}

// ---------------------------------------------------------------------------
// File-tool execution — jailed to repoRoot, returns a string result for the
// model. Errors are returned as text (the model can react), not thrown: a bad
// path the model invented should not crash the whole attempt.
// ---------------------------------------------------------------------------

/** The two roots a tool call is scoped against: reads jail to `repoRoot`,
 * writes jail to the narrower `writeRoot` (the task's `out/`). */
export interface ToolJail {
  repoRoot: string;
  writeRoot: string;
}

function runReadFile(jail: ToolJail, args: { path?: unknown }): string {
  if (typeof args.path !== "string") return "ERROR: read_file requires a string `path`.";
  let abs: string;
  try {
    abs = resolveReadInRepo(jail.repoRoot, args.path);
  } catch (err) {
    return `ERROR: ${(err as Error).message}`;
  }
  try {
    return readFileSync(abs, "utf8");
  } catch (err) {
    return `ERROR: could not read ${args.path}: ${(err as Error).message}`;
  }
}

function runWriteFile(jail: ToolJail, args: { path?: unknown; content?: unknown }): string {
  if (typeof args.path !== "string") return "ERROR: write_file requires a string `path`.";
  if (typeof args.content !== "string") return "ERROR: write_file requires a string `content`.";
  let abs: string;
  try {
    // PRD #067 slice 1: writes confine to the task's out/, not the repo.
    abs = resolveWriteInJail(jail.writeRoot, args.path);
  } catch (err) {
    return `ERROR: ${(err as Error).message}`;
  }
  try {
    mkdirSync(dirname(abs), { recursive: true });
    writeFileSync(abs, args.content, "utf8");
    return `OK: wrote ${args.content.length} bytes to ${args.path}`;
  } catch (err) {
    return `ERROR: could not write ${args.path}: ${(err as Error).message}`;
  }
}

/** Dispatch one tool call to its executor. Exported for the test. */
export function executeToolCall(jail: ToolJail, call: ToolCall): string {
  let args: Record<string, unknown>;
  try {
    args = JSON.parse(call.function.arguments || "{}");
  } catch {
    return `ERROR: tool arguments were not valid JSON: ${call.function.arguments}`;
  }
  switch (call.function.name) {
    case "read_file":
      return runReadFile(jail, args);
    case "write_file":
      return runWriteFile(jail, args);
    default:
      return `ERROR: unknown tool ${call.function.name}`;
  }
}

// ---------------------------------------------------------------------------
// One HTTP turn — POST the conversation, return the parsed completion or throw a
// loud, tagged transport error.
// ---------------------------------------------------------------------------

/** A transport failure carrying the handle tag the worker logs. */
export class TransportError extends Error {
  constructor(public handle: string, message: string) {
    super(message);
    this.name = "TransportError";
  }
}

async function postCompletion(
  opts: ChatCompletionsOptions,
  messages: ChatMessage[],
): Promise<ChatCompletion> {
  const fetchImpl = opts.fetchImpl ?? fetch;
  const url = resolveEndpoint(opts.baseUrl);
  const headers: Record<string, string> = { "content-type": "application/json" };
  if (opts.apiKey) headers["authorization"] = `Bearer ${opts.apiKey}`;

  const controller = new AbortController();
  const timer = setTimeout(() => controller.abort(), opts.requestTimeoutMs);
  let res: Response;
  try {
    res = await fetchImpl(url, {
      method: "POST",
      headers,
      body: JSON.stringify({ model: opts.model, messages, tools: TOOLS, stream: false }),
      signal: controller.signal,
    });
  } catch (err) {
    const e = err as Error;
    throw new TransportError(
      e.name === "AbortError" ? "timeout" : "network-error",
      `request to ${url} failed: ${e.message}`,
    );
  } finally {
    clearTimeout(timer);
  }

  if (!res.ok) {
    const body = await res.text().catch(() => "");
    throw new TransportError(`http-${res.status}`, `endpoint returned ${res.status}: ${body.slice(0, 1000)}`);
  }

  let body: unknown;
  try {
    body = await res.json();
  } catch (err) {
    throw new TransportError("malformed-body", `endpoint returned non-JSON: ${(err as Error).message}`);
  }
  const completion = body as ChatCompletion;
  if (!completion.choices || completion.choices.length === 0) {
    throw new TransportError("malformed-body", `completion had no choices: ${JSON.stringify(body).slice(0, 500)}`);
  }
  return completion;
}

// ---------------------------------------------------------------------------
// The adapter — the Harness the worker drives.
// ---------------------------------------------------------------------------

export function chatCompletionsHarness(opts: ChatCompletionsOptions): Harness {
  const progress = opts.onProgress ?? ((l: string) => process.stderr.write(l + "\n"));

  return async (attempt: Attempt): Promise<HarnessOutcome> => {
    progress(
      `[chat] attempt ${attempt.index} (${attempt.kind}) — ${opts.model} @ ${resolveEndpoint(opts.baseUrl)}`,
    );

    const messages: ChatMessage[] = [
      { role: "system", content: SYSTEM_PROMPT },
      { role: "user", content: attempt.feedback },
    ];

    let tokensUsed = 0;
    try {
      for (let turn = 1; turn <= opts.maxTurns; turn++) {
        const completion = await postCompletion(opts, messages);
        tokensUsed += completion.usage?.completion_tokens ?? 0;
        const choice = completion.choices![0]!;
        const msg = choice.message;

        // Record the assistant turn (with any tool calls) in the conversation.
        messages.push({
          role: "assistant",
          content: msg.content ?? null,
          ...(msg.tool_calls ? { tool_calls: msg.tool_calls } : {}),
        });

        const toolCalls = msg.tool_calls ?? [];
        if (toolCalls.length === 0) {
          // The model stopped calling tools — it considers the task done. The
          // worker (not this adapter) re-checks via the gauntlet; we just report
          // a clean completion with the model's closing text as the handle.
          progress(
            `[chat] attempt ${attempt.index} done — ${turn} turn(s), ${tokensUsed} completion tokens`,
          );
          const detail = (msg.content ?? "").slice(0, 2000);
          return detail
            ? { ok: true, handle: `chat:${opts.model}:turns=${turn}`, detail }
            : { ok: true, handle: `chat:${opts.model}:turns=${turn}` };
        }

        // Execute each requested tool and append its result as a tool message.
        const jail: ToolJail = { repoRoot: opts.repoRoot, writeRoot: opts.writeRoot };
        for (const call of toolCalls) {
          const result = executeToolCall(jail, call);
          progress(
            `[chat]   turn ${turn}: ${call.function.name}(${truncateArgs(call.function.arguments)}) -> ${summarize(result)}`,
          );
          messages.push({
            role: "tool",
            tool_call_id: call.id,
            name: call.function.name,
            content: result,
          });
        }

        if (tokensUsed > opts.maxTokens) {
          return {
            ok: false,
            handle: "token-budget",
            detail: `exceeded ${opts.maxTokens} completion tokens (used ${tokensUsed}) before finishing`,
          };
        }
      }
      // Ran out of turns without the model stopping — loud.
      return {
        ok: false,
        handle: "turn-budget",
        detail: `model did not finish within ${opts.maxTurns} turns`,
      };
    } catch (err) {
      if (err instanceof TransportError) {
        progress(`[chat] attempt ${attempt.index} transport failure — ${err.handle}: ${err.message}`);
        return { ok: false, handle: err.handle, detail: err.message.slice(0, 2000) };
      }
      progress(`[chat] attempt ${attempt.index} crashed — ${(err as Error).message}`);
      return { ok: false, handle: "adapter-error", detail: (err as Error).message.slice(0, 2000) };
    }
  };
}

function truncateArgs(s: string): string {
  return s.length > 80 ? s.slice(0, 77) + "..." : s;
}
function summarize(result: string): string {
  const first = result.split("\n", 1)[0] ?? "";
  return first.length > 80 ? first.slice(0, 77) + "..." : first;
}
