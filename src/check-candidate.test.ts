// Creation candidate checker tests — the sim gate and the validator path that
// stand in front of the kernel as the creation contract (slice 1). Exercises
// the gate verdicts (in-band passes, out-of-band bounces both ways), the
// loud-validator-failure path, gate-config loading/merging, and determinism —
// without spawning a subprocess.

import { describe, expect, test } from "vitest";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { mkdtempSync, writeFileSync } from "node:fs";
import {
  checkCandidate,
  defaultGateConfig,
  loadCandidate,
  loadGateConfig,
  machineReport,
  mergeGateConfig,
} from "./check-candidate.js";
import { runGate, formatGateReport } from "./gate.js";
import { REFERENCE_META } from "./content/reference-meta.js";
import { stressRegistry } from "./content/stress.js";
import { ValidationError } from "./validate.js";
import { GATE_BAND_MIN, GATE_BAND_MAX, GATE_MATCHUP_FLOOR } from "./tunables.js";
import type { UnitDef } from "./types.js";

const TASK = join(new URL(".", import.meta.url).pathname, "..", "tasks", "frostbite-striker");

// The committed fixtures — the hand-proven contract. They live in fixtures/ so
// the task root's candidate.json / out/ are free for a worker's emitted output
// to land without clobbering the golden inputs (slice 2).
const SANE = join(TASK, "fixtures", "candidate.json");
const OVERTUNED = join(TASK, "fixtures", "candidate-overtuned.json");

function writeTmp(name: string, content: string): string {
  const p = join(mkdtempSync(join(tmpdir(), "aoi-cand-")), name);
  writeFileSync(p, content, "utf8");
  return p;
}

// ---------------------------------------------------------------------------
// 1. Gate verdicts — the three outcomes the loop must distinguish
// ---------------------------------------------------------------------------

describe("checkCandidate verdicts", () => {
  const cfg = defaultGateConfig();

  test("an in-band candidate passes (exit 0) — pooled in band AND every matchup above the floor", () => {
    const result = checkCandidate(loadCandidate(SANE), cfg);
    expect(result.status).toBe("passed");
    expect(result.exitCode).toBe(0);
    expect(result.gate!.pass).toBe(true);
    expect(result.gate!.verdict).toBe("in-band");
    expect(result.gate!.overallWinRate).toBeGreaterThanOrEqual(cfg.bandMin);
    expect(result.gate!.overallWinRate).toBeLessThanOrEqual(cfg.bandMax);
    // Passing means broadly viable, not lucky on the average: no matchup folds.
    expect(result.gate!.foldedTo).toEqual([]);
    for (const m of result.gate!.matchups) {
      expect(m.winRate).toBeGreaterThanOrEqual(cfg.floor);
    }
  });

  test("a pooled-in-band candidate that folds one matchup is bounced as counter-folded (the floor closes the gameable-pool hole)", () => {
    // The pre-floor "sane" fixture: curse-only striker fronted by a body. It
    // pools to ~54% (in [35,65]) yet folds StatStack to 0% — it would have
    // passed a pooled-only gate. The per-matchup floor catches it.
    const gameable: UnitDef[] = [
      { name: "Vanguard", base: { hp: 11, pwr: 3 } },
      {
        name: "Frostmage",
        base: { hp: 8, pwr: 2 },
        abilities: [
          {
            whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
            selectors: [{ kind: "frontEnemy" }],
            effects: [{ kind: "applyStatus", status: "Curse", stacks: { kind: "const", value: 1 } }],
          },
        ],
      },
      { name: "Squire", base: { hp: 7, pwr: 2 } },
    ];
    const result = checkCandidate(gameable, cfg);
    // Pooled win-rate sits inside the band — a pooled-only gate would pass it.
    expect(result.gate!.overallWinRate).toBeGreaterThanOrEqual(cfg.bandMin);
    expect(result.gate!.overallWinRate).toBeLessThanOrEqual(cfg.bandMax);
    // But the floor bounces it: at least one matchup is below the floor.
    expect(result.status).toBe("gate-bounced");
    expect(result.exitCode).toBe(2);
    expect(result.gate!.pass).toBe(false);
    expect(result.gate!.verdict).toBe("counter-folded");
    expect(result.gate!.foldedTo.length).toBeGreaterThan(0);
    // The bounce names which opponents folded — the signal the loop adjusts to.
    for (const name of result.gate!.foldedTo) {
      const m = result.gate!.matchups.find((x) => x.opponent === name)!;
      expect(m.winRate).toBeLessThan(cfg.floor);
    }
  });

  test("the overtuned candidate ('deal 999 damage') is bounced (exit 2) with its win-rate data", () => {
    const result = checkCandidate(loadCandidate(OVERTUNED), cfg);
    expect(result.status).toBe("gate-bounced");
    expect(result.exitCode).toBe(2);
    expect(result.gate!.pass).toBe(false);
    expect(result.gate!.verdict).toBe("overtuned");
    expect(result.gate!.overallWinRate).toBeGreaterThan(cfg.bandMax);
    // The bounce carries the numbers the loop adjusts against.
    expect(result.gate!.totalSeeds).toBeGreaterThan(0);
    expect(result.gate!.matchups.length).toBe(REFERENCE_META.length);
  });

  test("an underpowered candidate is bounced as such", () => {
    // A lone 1/1 body loses to every reference team → 0% win-rate.
    const filler: UnitDef[] = [{ name: "Filler", base: { hp: 1, pwr: 1 } }];
    const result = checkCandidate(filler, cfg);
    expect(result.status).toBe("gate-bounced");
    expect(result.gate!.verdict).toBe("underpowered");
    expect(result.gate!.overallWinRate).toBeLessThan(cfg.bandMin);
  });
});

// ---------------------------------------------------------------------------
// 2. Validator path — a typo'd candidate must fail loudly, never reach the gate
// ---------------------------------------------------------------------------

describe("validator path", () => {
  test("a typo'd effect kind throws ValidationError (loud), before any sim", () => {
    const typo = writeTmp(
      "typo.json",
      JSON.stringify({
        units: [
          {
            name: "Typo",
            base: { hp: 8, pwr: 2 },
            abilities: [
              {
                whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
                selectors: [{ kind: "frontEnemy" }],
                effects: [{ kind: "damgae", amount: { kind: "const", value: 3 } }],
              },
            ],
          },
        ],
      }),
    );
    expect(() => loadCandidate(typo)).toThrow(ValidationError);
    expect(() => loadCandidate(typo)).toThrow(/unknown effect kind "damgae"/);
  });

  test("a dangling status reference fails the validator", () => {
    const bad = writeTmp(
      "dangling.json",
      JSON.stringify({ units: [{ name: "Ghost", base: { hp: 5, pwr: 1 }, statuses: [{ status: "Nope", stacks: 1 }] }] }),
    );
    expect(() => loadCandidate(bad)).toThrow(/unknown status "Nope"/);
  });

  test("malformed JSON fails loudly", () => {
    const broken = writeTmp("broken.json", "{ not json }");
    expect(() => loadCandidate(broken)).toThrow(/not valid JSON/);
  });

  test("a missing units field fails loudly", () => {
    const empty = writeTmp("empty.json", JSON.stringify({}));
    expect(() => loadCandidate(empty)).toThrow(/"units" field/);
  });
});

// ---------------------------------------------------------------------------
// 3. Gate config — tunable defaults, per-task override, validation
// ---------------------------------------------------------------------------

describe("gate config", () => {
  test("defaults come from the tunables, never hardcoded prose", () => {
    expect(defaultGateConfig()).toEqual({
      bandMin: GATE_BAND_MIN,
      bandMax: GATE_BAND_MAX,
      floor: GATE_MATCHUP_FLOOR,
      seeds: 50,
    });
  });

  test("the task dir's gate.json is read", () => {
    const cfg = loadGateConfig(TASK);
    expect(cfg.bandMin).toBe(0.35);
    expect(cfg.bandMax).toBe(0.65);
    expect(cfg.floor).toBe(0.25);
    expect(cfg.seeds).toBe(50);
  });

  test("a task dir without gate.json falls back to the defaults", () => {
    const dir = mkdtempSync(join(tmpdir(), "aoi-nogate-"));
    expect(loadGateConfig(dir)).toEqual(defaultGateConfig());
  });

  test("an override merges over the defaults", () => {
    const merged = mergeGateConfig(defaultGateConfig(), { bandMin: 0.4, seeds: 10 });
    expect(merged).toEqual({ bandMin: 0.4, bandMax: GATE_BAND_MAX, floor: GATE_MATCHUP_FLOOR, seeds: 10 });
  });

  test("the per-matchup floor is overridable like the band", () => {
    const merged = mergeGateConfig(defaultGateConfig(), { floor: 0.3 });
    expect(merged.floor).toBe(0.3);
    expect(() => mergeGateConfig(defaultGateConfig(), { floor: 1.5 })).toThrow(/in \[0, 1\]/);
  });

  test("an out-of-range knob is rejected", () => {
    expect(() => mergeGateConfig(defaultGateConfig(), { bandMin: 1.5 })).toThrow(/in \[0, 1\]/);
    expect(() => mergeGateConfig(defaultGateConfig(), { seeds: 0 })).toThrow(/in \[1,/);
    expect(() => mergeGateConfig(defaultGateConfig(), { bandMin: 0.8, bandMax: 0.6 })).toThrow(/must not exceed/);
  });
});

// ---------------------------------------------------------------------------
// 4. Determinism — same candidate + same config → byte-identical report
// ---------------------------------------------------------------------------

describe("determinism", () => {
  test("the gate report is byte-identical across two runs", () => {
    const candidate = loadCandidate(SANE);
    const cfg = loadGateConfig(TASK);
    const a = JSON.stringify(runGate(candidate, REFERENCE_META, cfg, stressRegistry));
    const b = JSON.stringify(runGate(candidate, REFERENCE_META, cfg, stressRegistry));
    expect(a).toBe(b);
  });

  test("the machine report line is stable and parseable", () => {
    const result = checkCandidate(loadCandidate(SANE), loadGateConfig(TASK));
    const line = machineReport(result);
    const parsed = JSON.parse(line);
    expect(parsed.status).toBe("passed");
    expect(parsed.gate.overallWinRate).toBe(result.gate!.overallWinRate);
  });
});

// ---------------------------------------------------------------------------
// 5. Report rendering — human transcript carries the verdict and the band
// ---------------------------------------------------------------------------

describe("formatGateReport", () => {
  test("a passing report reads PASS with the band and matchups", () => {
    const report = runGate(loadCandidate(SANE), REFERENCE_META, loadGateConfig(TASK), stressRegistry);
    const text = formatGateReport(report);
    expect(text).toContain("Sim gate: PASS (in-band)");
    expect(text).toContain("band 35.0%–65.0%");
    for (const m of REFERENCE_META) expect(text).toContain(`vs ${m.name}:`);
  });
});
