// Creation worker tests (PRD #013 slice 2) — the bounce-loop state machine and
// its transport, with the AI harness and the gauntlet faked. The real CLI is
// never spawned here: that is the e2e proof, run once by hand. These tests pin
// convergence, the retry bound + loud failure, the machine log shape, malformed
// / missing output handling, and that the worker carries no game rules.

import { describe, expect, test } from "vitest";
import {
  runLoop,
  initialPrompt,
  bouncePrompt,
  readEmit,
} from "./worker.js";
import type {
  Attempt,
  Gauntlet,
  GauntletResult,
  Harness,
  HarnessOutcome,
  WorkerConfig,
} from "./worker.js";
import { buildArgs, parseEnvelope } from "./claude-code.js";
import { parseMachineLine } from "./gauntlet.js";
import { parseArgs } from "./cli.js";

const CONFIG: WorkerConfig = {
  taskDir: "/repo/tasks/frostbite-striker",
  outRel: "out/candidate.json",
  maxAttempts: 5,
};

// A gauntlet result factory — pooled numbers only, no game knowledge in the test
// beyond the shape check-candidate emits.
function gateResult(verdict: string, overall: number, folded: string[] = []): GauntletResult {
  return {
    status: verdict === "in-band" ? "passed" : "gate-bounced",
    validator: "ok",
    gate: {
      pass: verdict === "in-band",
      verdict,
      overallWinRate: overall,
      band: { min: 0.35, max: 0.65 },
      floor: 0.25,
      foldedTo: folded,
      matchups: [],
    },
  };
}

const okHarness: Harness = async (a: Attempt): Promise<HarnessOutcome> => ({
  ok: true,
  handle: `session:fake-${a.index}`,
});

// A read stub that always finds a parseable candidate.
const readOk = () => ({ ok: true }) as const;

// ---------------------------------------------------------------------------
// 1. Convergence — the loop stops at the first pass and records where
// ---------------------------------------------------------------------------

describe("runLoop convergence", () => {
  test("a first-attempt pass converges at attempt 1", async () => {
    const gauntlet: Gauntlet = async () => gateResult("in-band", 0.5);
    const result = await runLoop(CONFIG, okHarness, gauntlet, readOk);
    expect(result.converged).toBe(true);
    expect(result.convergedAt).toBe(1);
    expect(result.attempts).toHaveLength(1);
    expect(result.attempts[0]!.outcome).toBe("passed");
    expect(result.attempts[0]!.kind).toBe("initial");
  });

  test("a bounce then a pass converges at attempt 2; the 2nd attempt is a bounce", async () => {
    let calls = 0;
    const gauntlet: Gauntlet = async () => {
      calls++;
      return calls === 1 ? gateResult("overtuned", 0.9) : gateResult("in-band", 0.5);
    };
    const result = await runLoop(CONFIG, okHarness, gauntlet, readOk);
    expect(result.converged).toBe(true);
    expect(result.convergedAt).toBe(2);
    expect(result.attempts).toHaveLength(2);
    expect(result.attempts[0]!.outcome).toBe("bounced");
    expect(result.attempts[1]!.kind).toBe("bounce");
    expect(result.attempts[1]!.outcome).toBe("passed");
    // The bounce prompt carried the previous attempt's numbers, verbatim.
    expect(result.attempts[1]!.feedback).toContain("overtuned");
    expect(result.attempts[1]!.feedback).toContain("0.9");
  });

  test("a counter-folded bounce names the folded opponents in the next prompt", async () => {
    let calls = 0;
    const gauntlet: Gauntlet = async () => {
      calls++;
      return calls === 1
        ? gateResult("counter-folded", 0.54, ["StatStack"])
        : gateResult("in-band", 0.5);
    };
    const result = await runLoop(CONFIG, okHarness, gauntlet, readOk);
    expect(result.converged).toBe(true);
    expect(result.attempts[1]!.feedback).toContain("StatStack");
    expect(result.attempts[1]!.feedback).toContain("counter-folded");
  });
});

// ---------------------------------------------------------------------------
// 2. The retry bound — loud failure when no attempt passes
// ---------------------------------------------------------------------------

describe("runLoop bound", () => {
  test("never-passing gauntlet fails loudly after exactly maxAttempts", async () => {
    const gauntlet: Gauntlet = async () => gateResult("overtuned", 0.9);
    const result = await runLoop(CONFIG, okHarness, gauntlet, readOk);
    expect(result.converged).toBe(false);
    expect(result.convergedAt).toBeNull();
    expect(result.attempts).toHaveLength(5);
    expect(result.attempts.every((a) => a.outcome === "bounced")).toBe(true);
  });

  test("the bound is honoured at maxAttempts=1 (one shot, no bounce)", async () => {
    const gauntlet: Gauntlet = async () => gateResult("underpowered", 0.1);
    const result = await runLoop({ ...CONFIG, maxAttempts: 1 }, okHarness, gauntlet, readOk);
    expect(result.converged).toBe(false);
    expect(result.attempts).toHaveLength(1);
  });
});

// ---------------------------------------------------------------------------
// 3. Transport failures — harness error and missing/malformed output
// ---------------------------------------------------------------------------

describe("runLoop transport", () => {
  test("a harness error is logged and retried, not counted as a bounce", async () => {
    let calls = 0;
    const flaky: Harness = async (a) => {
      calls++;
      return calls === 1
        ? { ok: false, handle: "timeout", detail: "killed" }
        : { ok: true, handle: `session:${a.index}` };
    };
    const gauntlet: Gauntlet = async () => gateResult("in-band", 0.5);
    const result = await runLoop(CONFIG, flaky, gauntlet, readOk);
    expect(result.converged).toBe(true);
    expect(result.convergedAt).toBe(2);
    expect(result.attempts[0]!.outcome).toBe("harness-error");
    expect(result.attempts[0]!.gauntlet).toBeNull();
  });

  test("a missing candidate is recorded as no-candidate and the gauntlet is not run", async () => {
    let reads = 0;
    const read = () => {
      reads++;
      return reads === 1 ? ({ ok: false, status: "missing" } as const) : ({ ok: true } as const);
    };
    let gauntletRuns = 0;
    const gauntlet: Gauntlet = async () => {
      gauntletRuns++;
      return gateResult("in-band", 0.5);
    };
    const result = await runLoop(CONFIG, okHarness, gauntlet, read);
    expect(result.attempts[0]!.outcome).toBe("no-candidate");
    expect(result.attempts[0]!.candidate).toBe("missing");
    expect(gauntletRuns).toBe(1); // only the 2nd (readable) attempt reached the gauntlet
    expect(result.converged).toBe(true);
  });

  test("readEmit flags unparseable output as a transport failure", () => {
    // A truncated/empty write must not reach the gauntlet as a validator failure.
    const tmp = makeFile("{ truncated");
    const r = readEmit(tmp);
    expect(r.ok).toBe(false);
    if (!r.ok) expect(r.status).toContain("unparseable");
  });

  test("readEmit reports missing when the file is absent", () => {
    const r = readEmit("/no/such/path/candidate.json");
    expect(r.ok).toBe(false);
    if (!r.ok) expect(r.status).toBe("missing");
  });
});

// ---------------------------------------------------------------------------
// 4. The log shape — machine-readable, per attempt (slice-4 provenance)
// ---------------------------------------------------------------------------

describe("attempt log shape", () => {
  test("every attempt records index, kind, harness handle, candidate, gauntlet, outcome", async () => {
    let calls = 0;
    const gauntlet: Gauntlet = async () => {
      calls++;
      return calls === 1 ? gateResult("underpowered", 0.2) : gateResult("in-band", 0.5);
    };
    const result = await runLoop(CONFIG, okHarness, gauntlet, readOk);
    for (const a of result.attempts) {
      expect(typeof a.index).toBe("number");
      expect(["initial", "bounce"]).toContain(a.kind);
      expect(a.harness.handle).toMatch(/session:/);
      expect(["passed", "bounced", "harness-error", "no-candidate"]).toContain(a.outcome);
    }
    // The bounced attempt carries the full gauntlet numbers for provenance.
    expect(result.attempts[0]!.gauntlet!.gate!.overallWinRate).toBe(0.2);
    // The whole log round-trips through JSON (it is the provenance artifact).
    expect(() => JSON.parse(JSON.stringify(result))).not.toThrow();
  });
});

// ---------------------------------------------------------------------------
// 5. No game rules in the worker — the prompts relay, they do not author
// ---------------------------------------------------------------------------

describe("prompts are rules-blind transport", () => {
  test("the initial prompt points at the README and the out path, naming no rules", () => {
    const p = initialPrompt(CONFIG.taskDir, CONFIG.outRel);
    expect(p).toContain("README.md");
    expect(p).toContain("out/candidate.json");
    // It must not encode game vocabulary — the contract lives in the README.
    for (const rule of ["win-rate", "Poison", "Curse", "Freeze", "band", "reference meta", "UnitDef"]) {
      expect(p).not.toContain(rule);
    }
  });

  test("the bounce prompt relays the gauntlet JSON verbatim without interpreting balance", () => {
    const res = gateResult("counter-folded", 0.54, ["StatStack"]);
    const p = bouncePrompt(CONFIG.taskDir, CONFIG.outRel, res);
    // The verbatim JSON is present (the numbers the loop feeds back).
    expect(p).toContain(JSON.stringify(res));
    // It defers to the README for *how* to act — it does not say "lower damage"
    // or any magnitude rule itself.
    expect(p).toContain("README");
    expect(p).not.toMatch(/raise (the )?(hp|pwr|damage|magnitude)/i);
  });
});

// ---------------------------------------------------------------------------
// 6. Adapter wiring — argv flags, gauntlet line parse, CLI arg parse
// ---------------------------------------------------------------------------

describe("claude-code buildArgs", () => {
  test("builds a headless, json, dir-scoped invocation (prompt goes on stdin, not argv)", () => {
    const args = buildArgs({ repoRoot: "/repo", timeoutMs: 1000 });
    expect(args).toContain("-p");
    expect(args.slice(args.indexOf("--output-format"), args.indexOf("--output-format") + 2)).toEqual([
      "--output-format",
      "json",
    ]);
    expect(args).toContain("--permission-mode");
    expect(args[args.indexOf("--permission-mode") + 1]).toBe("bypassPermissions");
    // --add-dir must be LAST among the args: it is variadic, so nothing may
    // follow it on argv (the prompt is fed via stdin for exactly this reason).
    expect(args[args.length - 2]).toBe("--add-dir");
    expect(args[args.length - 1]).toBe("/repo");
  });

  test("a model override is passed through (before the trailing --add-dir)", () => {
    const args = buildArgs({ repoRoot: "/repo", timeoutMs: 1000, model: "opus" });
    expect(args[args.indexOf("--model") + 1]).toBe("opus");
    expect(args[args.length - 2]).toBe("--add-dir");
  });
});

describe("claude-code parseEnvelope", () => {
  test("pulls the result element from the json *array* envelope (-p --output-format json)", () => {
    // The real CLI (v2.1.175) emits an array of stream objects; the last
    // type:"result" element carries result/session_id/is_error.
    const stdout = JSON.stringify([
      { type: "system", subtype: "init", session_id: "abc" },
      { type: "assistant", message: {} },
      { type: "result", subtype: "success", is_error: false, result: "done", session_id: "abc", num_turns: 4 },
    ]);
    const env = parseEnvelope(stdout)!;
    expect(env.type).toBe("result");
    expect(env.session_id).toBe("abc");
    expect(env.is_error).toBe(false);
  });

  test("surfaces an auth/api error envelope (is_error true)", () => {
    const stdout = JSON.stringify([
      { type: "system", subtype: "init", session_id: "x" },
      { type: "result", is_error: true, result: "Failed to authenticate. API Error: 401" },
    ]);
    const env = parseEnvelope(stdout)!;
    expect(env.is_error).toBe(true);
    expect(env.result).toContain("401");
  });

  test("accepts a bare object and rejects non-JSON", () => {
    expect(parseEnvelope(JSON.stringify({ type: "result", result: "ok" }))!.result).toBe("ok");
    expect(parseEnvelope("not json")).toBeNull();
  });
});

describe("gauntlet parseMachineLine", () => {
  test("pulls the last JSON line out of a human-then-machine transcript", () => {
    const stdout = [
      "Validator: PASS",
      "Sim gate: PASS (in-band)",
      "  overall win-rate: 50.0%",
      JSON.stringify({ status: "passed", validator: "ok", gate: { pass: true } }),
    ].join("\n");
    const parsed = parseMachineLine(stdout);
    expect(parsed.status).toBe("passed");
  });

  test("throws loudly when there is no machine line", () => {
    expect(() => parseMachineLine("Validator: PASS\nno json here")).toThrow(/no machine JSON/);
  });
});

describe("cli parseArgs", () => {
  test("defaults: maxAttempts 5, no model, no bin override", () => {
    const a = parseArgs(["tasks/frostbite-striker"]);
    expect(a.taskDir).toBe("tasks/frostbite-striker");
    expect(a.maxAttempts).toBe(5);
    expect(a.model).toBeUndefined();
    expect(a.bin).toBeUndefined();
  });

  test("flags parse", () => {
    const a = parseArgs(["tasks/x", "--max-attempts", "3", "--model", "sonnet", "--timeout-ms", "60000", "--bin", "/usr/bin/fake"]);
    expect(a.maxAttempts).toBe(3);
    expect(a.model).toBe("sonnet");
    expect(a.timeoutMs).toBe(60000);
    expect(a.bin).toBe("/usr/bin/fake");
  });

  test("a missing task dir, an unknown flag, or a bad number is rejected", () => {
    expect(() => parseArgs([])).toThrow();
    expect(() => parseArgs(["a", "b"])).toThrow();
    expect(() => parseArgs(["t", "--nope"])).toThrow(/unknown flag/);
    expect(() => parseArgs(["t", "--max-attempts", "0"])).toThrow(/positive integer/);
  });
});

// --- helpers ---------------------------------------------------------------

import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
function makeFile(content: string): string {
  const p = join(mkdtempSync(join(tmpdir(), "aoi-worker-")), "candidate.json");
  writeFileSync(p, content, "utf8");
  return p;
}
