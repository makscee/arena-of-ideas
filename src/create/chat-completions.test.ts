// Chat-completions adapter tests (PRD #013 slice 3) — the second Harness behind
// the same worker bounce loop. Two layers:
//   1. Unit: the adapter's tool loop driven by an INJECTED fetch that scripts the
//      OpenAI wire responses — convergence, malformed body, transport failure,
//      and the turn/token budgets. No network, deterministic.
//   2. Integration: the real protocol-faithful stub HTTP server (evidence/
//      stub-openai-server.mjs) spawned on an ephemeral port, driving the FULL
//      runLoop end-to-end (read README -> write candidate -> stop), proving the
//      adapter speaks the wire format a real endpoint produces.
//
// No game rules appear here or in the adapter — the conversation relays the
// worker's prompt text; the stub's "model" earns convergence from bounce numbers.

import { describe, expect, test, afterEach } from "vitest";
import {
  chatCompletionsHarness,
  resolveEndpoint,
  executeToolCall,
} from "./chat-completions.js";
import type { Attempt } from "./worker.js";
import { mkdtempSync, readFileSync, existsSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";

const INITIAL: Attempt = {
  kind: "initial",
  index: 1,
  feedback: "Read the task contract first: /repo/tasks/x/README.md\nProduce your output at: /repo/tasks/x/out/candidate.json",
};

const silent = { onProgress: () => {} };

// A fetch double that returns scripted OpenAI completion bodies in order, and
// records the request bodies it saw (so we can assert the wire shape).
function scriptedFetch(bodies: unknown[]): {
  fetchImpl: typeof fetch;
  requests: Array<{ url: string; body: any }>;
} {
  const requests: Array<{ url: string; body: any }> = [];
  let i = 0;
  const fetchImpl = (async (url: string, init: RequestInit) => {
    requests.push({ url: String(url), body: JSON.parse(String(init.body)) });
    const payload = bodies[i++];
    return new Response(JSON.stringify(payload), { status: 200, headers: { "content-type": "application/json" } });
  }) as unknown as typeof fetch;
  return { fetchImpl, requests };
}

function asstToolCall(name: string, args: object) {
  return {
    choices: [
      {
        message: {
          role: "assistant",
          content: null,
          tool_calls: [{ id: "c1", type: "function", function: { name, arguments: JSON.stringify(args) } }],
        },
        finish_reason: "tool_calls",
      },
    ],
    usage: { completion_tokens: 10 },
  };
}
function asstStop(content: string, completion_tokens = 5) {
  return {
    choices: [{ message: { role: "assistant", content }, finish_reason: "stop" }],
    usage: { completion_tokens },
  };
}

// ---------------------------------------------------------------------------
// resolveEndpoint — base-url normalisation
// ---------------------------------------------------------------------------

describe("resolveEndpoint", () => {
  test("adds /v1/chat/completions to a bare base", () => {
    expect(resolveEndpoint("https://api.deepseek.com")).toBe("https://api.deepseek.com/v1/chat/completions");
    expect(resolveEndpoint("https://api.deepseek.com/")).toBe("https://api.deepseek.com/v1/chat/completions");
  });
  test("respects a base that already ends in /vN", () => {
    expect(resolveEndpoint("https://api.openai.com/v1")).toBe("https://api.openai.com/v1/chat/completions");
    expect(resolveEndpoint("http://127.0.0.1:8123/v2/")).toBe("http://127.0.0.1:8123/v2/chat/completions");
  });
});

// ---------------------------------------------------------------------------
// executeToolCall — generic, jailed file IO; no game vocabulary
// ---------------------------------------------------------------------------

describe("executeToolCall", () => {
  test("read_file returns file contents, write_file writes and confirms", () => {
    const dir = mkdtempSync(join(tmpdir(), "aoi-chat-"));
    const w = executeToolCall(dir, {
      id: "1", type: "function",
      function: { name: "write_file", arguments: JSON.stringify({ path: "out/candidate.json", content: "{}" }) },
    });
    expect(w).toMatch(/^OK: wrote/);
    expect(existsSync(join(dir, "out/candidate.json"))).toBe(true);
    const r = executeToolCall(dir, {
      id: "2", type: "function",
      function: { name: "read_file", arguments: JSON.stringify({ path: "out/candidate.json" }) },
    });
    expect(r).toBe("{}");
  });

  test("a path escaping the repo is refused (jailed), not honoured", () => {
    const dir = mkdtempSync(join(tmpdir(), "aoi-chat-"));
    const r = executeToolCall(dir, {
      id: "1", type: "function",
      function: { name: "read_file", arguments: JSON.stringify({ path: "../../etc/passwd" }) },
    });
    expect(r).toMatch(/ERROR: path escapes repo/);
  });

  test("malformed tool arguments are returned as an error string, not thrown", () => {
    const dir = mkdtempSync(join(tmpdir(), "aoi-chat-"));
    const r = executeToolCall(dir, {
      id: "1", type: "function", function: { name: "read_file", arguments: "{not json" },
    });
    expect(r).toMatch(/ERROR: tool arguments were not valid JSON/);
  });
});

// ---------------------------------------------------------------------------
// The tool loop — scripted fetch
// ---------------------------------------------------------------------------

describe("chatCompletionsHarness tool loop", () => {
  test("converges: reads a file, writes the candidate, stops -> ok outcome", async () => {
    const dir = mkdtempSync(join(tmpdir(), "aoi-chat-"));
    const { fetchImpl, requests } = scriptedFetch([
      asstToolCall("read_file", { path: "README.md" }), // turn 1: read
      asstToolCall("write_file", { path: "out/candidate.json", content: '{"units":[]}' }), // turn 2: write
      asstStop("done — wrote the candidate."), // turn 3: stop
    ]);
    // The repo must hold a README.md for the read to succeed.
    executeToolCall(dir, { id: "0", type: "function", function: { name: "write_file", arguments: JSON.stringify({ path: "README.md", content: "hi" }) } });

    const harness = chatCompletionsHarness({
      baseUrl: "https://example.test", model: "m", repoRoot: dir,
      maxTurns: 12, maxTokens: 100_000, requestTimeoutMs: 5000, fetchImpl, ...silent,
    });
    const outcome = await harness(INITIAL);
    expect(outcome.ok).toBe(true);
    expect(outcome.handle).toContain("turns=3");
    // The candidate was actually written by the tool.
    expect(readFileSync(join(dir, "out/candidate.json"), "utf8")).toBe('{"units":[]}');
    // The wire request carried system + user + tools and stream:false.
    const first = requests[0]!.body;
    expect(first.messages[0].role).toBe("system");
    expect(first.messages[1].role).toBe("user");
    expect(first.stream).toBe(false);
    expect(first.tools.map((t: any) => t.function.name).sort()).toEqual(["read_file", "write_file"]);
    // The system prompt carries no game vocabulary.
    for (const rule of ["win-rate", "Poison", "Curse", "UnitDef", "band"]) {
      expect(JSON.stringify(first.messages[0])).not.toContain(rule);
    }
  });

  test("bounce path: the user turn carries the gate numbers verbatim into the request", async () => {
    const dir = mkdtempSync(join(tmpdir(), "aoi-chat-"));
    const { fetchImpl, requests } = scriptedFetch([asstStop("ok")]);
    const harness = chatCompletionsHarness({
      baseUrl: "https://example.test", model: "m", repoRoot: dir,
      maxTurns: 12, maxTokens: 100_000, requestTimeoutMs: 5000, fetchImpl, ...silent,
    });
    const bounce: Attempt = {
      kind: "bounce", index: 2,
      feedback: 'The check output was:\n{"status":"gate-bounced","gate":{"verdict":"overtuned","overallWinRate":0.9}}',
    };
    await harness(bounce);
    // The adapter relays the worker's bounce feedback as the user turn, untouched.
    expect(requests[0]!.body.messages[1].content).toContain("overtuned");
    expect(requests[0]!.body.messages[1].content).toContain("0.9");
  });

  test("malformed model output: a completion with no choices is a loud transport failure", async () => {
    const dir = mkdtempSync(join(tmpdir(), "aoi-chat-"));
    const { fetchImpl } = scriptedFetch([{ usage: {} }]); // no choices[]
    const harness = chatCompletionsHarness({
      baseUrl: "https://example.test", model: "m", repoRoot: dir,
      maxTurns: 12, maxTokens: 100_000, requestTimeoutMs: 5000, fetchImpl, ...silent,
    });
    const outcome = await harness(INITIAL);
    expect(outcome.ok).toBe(false);
    expect(outcome.handle).toBe("malformed-body");
  });

  test("transport failure: a non-2xx HTTP status is loud, tagged http-<code>", async () => {
    const dir = mkdtempSync(join(tmpdir(), "aoi-chat-"));
    const fetchImpl = (async () =>
      new Response("nope", { status: 401 })) as unknown as typeof fetch;
    const harness = chatCompletionsHarness({
      baseUrl: "https://example.test", model: "m", repoRoot: dir,
      maxTurns: 12, maxTokens: 100_000, requestTimeoutMs: 5000, fetchImpl, ...silent,
    });
    const outcome = await harness(INITIAL);
    expect(outcome.ok).toBe(false);
    expect(outcome.handle).toBe("http-401");
  });

  test("transport failure: a network error is loud, tagged network-error", async () => {
    const dir = mkdtempSync(join(tmpdir(), "aoi-chat-"));
    const fetchImpl = (async () => { throw new Error("ECONNREFUSED"); }) as unknown as typeof fetch;
    const harness = chatCompletionsHarness({
      baseUrl: "https://example.test", model: "m", repoRoot: dir,
      maxTurns: 12, maxTokens: 100_000, requestTimeoutMs: 5000, fetchImpl, ...silent,
    });
    const outcome = await harness(INITIAL);
    expect(outcome.ok).toBe(false);
    expect(outcome.handle).toBe("network-error");
  });

  test("turn budget: a model that loops on tools forever stops loudly at maxTurns", async () => {
    const dir = mkdtempSync(join(tmpdir(), "aoi-chat-"));
    executeToolCall(dir, { id: "0", type: "function", function: { name: "write_file", arguments: JSON.stringify({ path: "f", content: "x" }) } });
    // Always asks to read — never stops.
    const fetchImpl = (async () =>
      new Response(JSON.stringify(asstToolCall("read_file", { path: "f" })), { status: 200 })) as unknown as typeof fetch;
    const harness = chatCompletionsHarness({
      baseUrl: "https://example.test", model: "m", repoRoot: dir,
      maxTurns: 3, maxTokens: 100_000, requestTimeoutMs: 5000, fetchImpl, ...silent,
    });
    const outcome = await harness(INITIAL);
    expect(outcome.ok).toBe(false);
    expect(outcome.handle).toBe("turn-budget");
  });

  test("token budget: completion tokens summed past maxTokens stops loudly", async () => {
    const dir = mkdtempSync(join(tmpdir(), "aoi-chat-"));
    executeToolCall(dir, { id: "0", type: "function", function: { name: "write_file", arguments: JSON.stringify({ path: "f", content: "x" }) } });
    const fetchImpl = (async () => {
      const r = asstToolCall("read_file", { path: "f" });
      r.usage = { completion_tokens: 1000 } as any;
      return new Response(JSON.stringify(r), { status: 200 });
    }) as unknown as typeof fetch;
    const harness = chatCompletionsHarness({
      baseUrl: "https://example.test", model: "m", repoRoot: dir,
      maxTurns: 50, maxTokens: 500, requestTimeoutMs: 5000, fetchImpl, ...silent,
    });
    const outcome = await harness(INITIAL);
    expect(outcome.ok).toBe(false);
    expect(outcome.handle).toBe("token-budget");
  });
});

// ---------------------------------------------------------------------------
// Integration — the real stub HTTP server driving the full runLoop end-to-end
// ---------------------------------------------------------------------------

import { spawn } from "node:child_process";
import { fileURLToPath } from "node:url";
import { runLoop } from "./worker.js";
import type { Gauntlet, GauntletResult, WorkerConfig } from "./worker.js";

const STUB = fileURLToPath(new URL("../../tasks/frostbite-striker/evidence/stub-openai-server.mjs", import.meta.url));
const REPO_ROOT = fileURLToPath(new URL("../..", import.meta.url));

let stubProc: ReturnType<typeof spawn> | undefined;
afterEach(() => {
  if (stubProc) { stubProc.kill("SIGTERM"); stubProc = undefined; }
});

function startStub(): Promise<number> {
  return new Promise((resolve, reject) => {
    stubProc = spawn("node", [STUB], { stdio: ["ignore", "pipe", "pipe"] });
    let out = "";
    const t = setTimeout(() => reject(new Error("stub did not report a port")), 5000);
    stubProc.stdout!.on("data", (d) => {
      out += d;
      const m = out.match(/PORT (\d+)/);
      if (m) { clearTimeout(t); resolve(Number(m[1])); }
    });
    stubProc.on("error", reject);
  });
}

describe("integration: stub OpenAI server drives the full runLoop", () => {
  test("the adapter completes a candidate end-to-end against a real HTTP endpoint", async () => {
    const port = await startStub();
    const dir = mkdtempSync(join(tmpdir(), "aoi-chat-e2e-"));
    // The model needs a README.md to read; the worker prompt points at it.
    const taskDir = join(dir, "tasks", "frostbite-striker");
    executeToolCall(dir, {
      id: "0", type: "function",
      function: { name: "write_file", arguments: JSON.stringify({ path: "tasks/frostbite-striker/README.md", content: "Self-test: write out/candidate.json" }) },
    });

    const harness = chatCompletionsHarness({
      baseUrl: `http://127.0.0.1:${port}`, model: "stub-openai", repoRoot: dir,
      maxTurns: 12, maxTokens: 100_000, requestTimeoutMs: 5000, ...silent,
    });

    // A fake gauntlet: the candidate file is what the worker collects; we bounce
    // once (overtuned), then pass — proving the adapter re-attempts on a bounce.
    let calls = 0;
    const gauntlet: Gauntlet = async (): Promise<GauntletResult> => {
      calls++;
      const verdict = calls === 1 ? "overtuned" : "in-band";
      return {
        status: verdict === "in-band" ? "passed" : "gate-bounced",
        validator: "ok",
        gate: { pass: verdict === "in-band", verdict, overallWinRate: calls === 1 ? 0.9 : 0.5, band: { min: 0.35, max: 0.65 }, floor: 0.25, foldedTo: [], matchups: [] },
      };
    };

    const config: WorkerConfig = { taskDir, outRel: join("out", "candidate.json"), maxAttempts: 5 };
    const result = await runLoop(config, harness, gauntlet);

    expect(result.converged).toBe(true);
    expect(result.convergedAt).toBe(2); // one bounce, then a pass (stub adjusted)
    // The 2nd attempt was a bounce carrying the overtuned numbers back to the model.
    expect(result.attempts[1]!.kind).toBe("bounce");
    expect(result.attempts[1]!.feedback).toContain("overtuned");
    // The real candidate the stub wrote (via the adapter's write_file tool) is
    // valid JSON the worker collected — a true end-to-end HTTP round trip.
    const written = JSON.parse(readFileSync(join(taskDir, "out", "candidate.json"), "utf8"));
    expect(Array.isArray(written.units)).toBe(true);
  });
});
