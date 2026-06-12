// CLI surface tests for mint-candidate + approve (PRD #013 slice 4): argument
// parsing and the candidates loader. The end-to-end filesystem behaviour is
// proven by the e2e walk and the manual transcript in the slice report; here we
// pin the parse contract and the loader's tolerance of a malformed pool entry.

import { describe, expect, test } from "vitest";
import { mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { stressRegistry } from "../content/stress.js";
import { buildRecord, serializeRecord } from "./provenance.js";
import type { RunManifest } from "./provenance.js";
import { parseArgs as parseMintArgs } from "./mint-candidate.js";
import { loadCandidates } from "./approve-cli.js";

const MANIFEST: RunManifest = {
  ideaText: "x",
  creator: "maks",
  harness: "claude-code",
  model: "opus",
  startedAt: "2026-06-12T10:00:00.000Z",
};
const PASSED = {
  status: "passed" as const,
  validator: "ok",
  gate: {
    pass: true,
    verdict: "in-band",
    overallWinRate: 0.5,
    band: { min: 0.35, max: 0.65 },
    floor: 0.25,
    foldedTo: [],
    matchups: [{ opponent: "AggroVenom", winRate: 0.5, wins: 25, losses: 25, draws: 0, seeds: 50 }],
  },
};

describe("mint-candidate parseArgs", () => {
  test("requires a task dir and --creator; defaults harness/model", () => {
    const a = parseMintArgs(["tasks/x", "--creator", "maks"]);
    expect(a.taskDir).toBe("tasks/x");
    expect(a.creator).toBe("maks");
    expect(a.harness).toBe("claude-code");
    expect(a.model).toBe("(default)");
  });

  test("flags parse; missing creator or task dir is rejected", () => {
    const a = parseMintArgs(["t", "--creator", "c", "--harness", "raw-chat", "--model", "deepseek", "--id", "z"]);
    expect(a.harness).toBe("raw-chat");
    expect(a.model).toBe("deepseek");
    expect(a.id).toBe("z");
    expect(() => parseMintArgs(["t"])).toThrow(/creator/);
    expect(() => parseMintArgs(["--creator", "c"])).toThrow();
    expect(() => parseMintArgs(["t", "--creator", "c", "--nope"])).toThrow(/unknown flag/);
  });
});

describe("loadCandidates", () => {
  test("parses valid records, isolates a malformed entry as an error", () => {
    const dir = mkdtempSync(join(tmpdir(), "aoi-cand-"));
    const good = buildRecord("good", [{ name: "Froster", base: { hp: 11, pwr: 2 } }], MANIFEST, PASSED, 2);
    writeFileSync(join(dir, "good.json"), serializeRecord(good));
    writeFileSync(join(dir, "bad.json"), "{ not json");
    const { records, errors } = loadCandidates(dir);
    expect(records.map((r) => r.id)).toEqual(["good"]);
    expect(errors.length).toBe(1);
    expect(errors[0]).toMatch(/bad\.json/);
  });

  test("a missing directory yields no candidates, no throw", () => {
    const { records, errors } = loadCandidates(join(tmpdir(), "aoi-nope-" + Math.random()));
    expect(records).toEqual([]);
    expect(errors).toEqual([]);
  });
});
