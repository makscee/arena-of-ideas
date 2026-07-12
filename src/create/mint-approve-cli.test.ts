// CLI surface tests for mint-candidate + approve (PRD #013 slice 4): argument
// parsing, actual mint persistence, and the candidates loader.

import { describe, expect, test } from "vitest";
import { mkdtempSync, mkdirSync, readFileSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { spawnSync } from "node:child_process";
import { battle } from "../battle.js";
import { validateTeamFile } from "../cli.js";
import { stressAbilities, stressRegistry } from "../content/stress.js";
import { renderReplay } from "../replay.js";
import { buildRecord, serializeRecord } from "./provenance.js";
import type { RunManifest } from "./provenance.js";
import { parseCandidateRecord } from "./candidates.js";
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

describe("mint-candidate", () => {
  test("actual mint preserves custom nested Summon abilities through write and reread", () => {
    const root = mkdtempSync(join(tmpdir(), "aoi-mint-nested-"));
    const task = join(root, "task");
    const out = join(task, "out");
    const candidates = join(root, "candidates");
    mkdirSync(out, { recursive: true });
    const legacyTeam = {
      units: [{ name: "Spawner", base: { hp: 5, pwr: 0 }, ability: "Spawn" }],
      abilities: {
        Spawn: {
          name: "Spawn", family: "Summon",
          whens: [{ kind: "trigger", on: { on: "BattleStart" } }],
          selectors: [{ kind: "holder" }],
          effects: [{ kind: "summon", unit: { name: "Child", base: { hp: 2, pwr: 0 }, ability: "ChildAct" } }],
        },
        ChildAct: {
          name: "ChildAct", family: "Strike",
          whens: [{ kind: "trigger", on: { on: "TurnStart" } }],
          selectors: [{ kind: "frontEnemy" }],
          effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }],
        },
        Strike: stressAbilities.Strike,
      },
    };
    writeFileSync(join(out, "candidate.json"), JSON.stringify(legacyTeam));
    writeFileSync(join(out, "run-log.jsonl"), JSON.stringify({ index: 1, outcome: "passed", gauntlet: PASSED }) + "\n");

    const result = spawnSync(process.execPath, [
      "--import", "tsx/esm", join(import.meta.dirname, "mint-candidate.ts"),
      task, "--creator", "test", "--id", "nested", "--idea", "nested summon", "--out", candidates,
    ], { cwd: join(import.meta.dirname, "..", ".."), encoding: "utf8" });
    expect(result.status, result.stderr).toBe(0);

    const written = JSON.parse(readFileSync(join(candidates, "nested.json"), "utf8"));
    const record = parseCandidateRecord(written, stressRegistry, stressAbilities, "nested.json");
    expect(Object.keys(record.abilities)).toEqual(["Spawn", "ChildAct"]);
    expect(record.abilities.Spawn!.effects[0]).toMatchObject({
      kind: "summon", unit: { abilities: ["ChildAct"] },
    });

    const before = validateTeamFile(legacyTeam, "legacy nested team");
    const enemy = validateTeamFile({ units: [{ name: "Target", base: { hp: 10, pwr: 0 }, ability: "Strike" }] });
    const replay = (units: typeof record.units, abilities: typeof record.abilities) => renderReplay(battle({
      teamA: units, teamB: enemy.units, seed: 7, statuses: stressRegistry,
      abilities: { ...stressAbilities, ...abilities },
    }));
    expect(replay(record.units, record.abilities)).toBe(replay(before.units, before.abilities));
    expect(replay(record.units, record.abilities)).toContain("Child");
  });

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
    const good = buildRecord("good", [{ name: "Froster", base: { hp: 11, pwr: 2 }, ability: "Strike" }], MANIFEST, PASSED, 2);
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
