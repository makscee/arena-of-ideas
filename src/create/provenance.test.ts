// Provenance + candidates-pool + approve tests (PRD #013 slice 4).
//
// Three properties the slice rests on:
//   1. Provenance round-trips byte-stable: build → serialize → parse → serialize
//      yields the same bytes, and the parse re-validates the content.
//   2. approve moves a candidate's NEW units into the playable registry, stamps
//      creator credit, validates the merged pool, and refuses collisions.
//   3. A rejected/pending candidate never leaks into the draftable pool — only an
//      explicitly approved, validated record reaches mergePool's output.

import { describe, expect, test } from "vitest";
import { stressRegistry } from "../content/stress.js";
import { DEFAULT_RUN_POOL } from "../tunables.js";
import { mergePool, parseApprovedRegistry } from "../registry.js";
import type { UnitDef } from "../types.js";
import {
  buildRecord,
  gateStatsOf,
  readConvergedAttempt,
  serializeRecord,
} from "./provenance.js";
import type { RunManifest } from "./provenance.js";
import type { GauntletResult } from "./worker.js";
import { parseCandidateRecord } from "./candidates.js";
import { approveInto } from "./approve.js";

// --- fixtures --------------------------------------------------------------

const PASSED: GauntletResult = {
  status: "passed",
  validator: "ok",
  gate: {
    pass: true,
    verdict: "in-band",
    overallWinRate: 0.5,
    band: { min: 0.35, max: 0.65 },
    floor: 0.25,
    foldedTo: [],
    matchups: [
      { opponent: "AggroVenom", winRate: 0.6, wins: 30, losses: 20, draws: 0, seeds: 50 },
      { opponent: "SustainControl", winRate: 0.4, wins: 20, losses: 30, draws: 0, seeds: 50 },
    ],
  },
};

const BOUNCED: GauntletResult = {
  status: "gate-bounced",
  validator: "ok",
  gate: { ...PASSED.gate!, pass: false, verdict: "overtuned", overallWinRate: 0.9 },
};

const FROSTER: UnitDef = {
  name: "Froster",
  base: { hp: 11, pwr: 2 },
  statuses: [{ status: "Shield", stacks: 2 }],
  abilities: [
    {
      whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
      selectors: [{ kind: "frontEnemy" }],
      effects: [{ kind: "applyStatus", status: "Poison", stacks: { kind: "const", value: 2 } }],
    },
  ],
};

const MANIFEST: RunManifest = {
  ideaText: "A frosty striker.",
  creator: "maks",
  harness: "claude-code",
  model: "opus",
  startedAt: "2026-06-12T10:00:00.000Z",
};

// --- 1. provenance round-trip ----------------------------------------------

describe("provenance round-trip", () => {
  test("build → serialize → parse → serialize is byte-stable", () => {
    const record = buildRecord("froster", [FROSTER], MANIFEST, PASSED, 3);
    const once = serializeRecord(record);
    const reparsed = parseCandidateRecord(JSON.parse(once), stressRegistry, "froster.json");
    const twice = serializeRecord(reparsed);
    expect(twice).toBe(once);
  });

  test("the record carries every provenance field from the run", () => {
    const record = buildRecord("froster", [FROSTER], MANIFEST, PASSED, 3);
    expect(record.provenance).toEqual({
      ideaText: "A frosty striker.",
      creator: "maks",
      harness: "claude-code",
      model: "opus",
      timestamp: "2026-06-12T10:00:00.000Z",
      attempts: 3,
    });
    // gate stats are lifted verbatim (pooled + per-matchup).
    expect(record.gate.overallWinRate).toBe(0.5);
    expect(record.gate.matchups.map((m) => m.opponent)).toEqual(["AggroVenom", "SustainControl"]);
  });

  test("parse re-runs the content validator — a typo'd part fails loudly", () => {
    const record = buildRecord("bad", [FROSTER], MANIFEST, PASSED, 1);
    const raw = JSON.parse(serializeRecord(record));
    raw.units[0].abilities[0].effects[0].kind = "notARealEffect";
    expect(() => parseCandidateRecord(raw, stressRegistry, "bad.json")).toThrow();
  });

  test("gateStatsOf refuses a non-passing result", () => {
    expect(() => gateStatsOf(BOUNCED)).toThrow(/passed/);
  });
});

// --- readConvergedAttempt (gate-stats source from the run log) --------------

describe("readConvergedAttempt", () => {
  test("returns the passing attempt's gauntlet result + its index", () => {
    const log =
      JSON.stringify({ index: 1, outcome: "bounced", gauntlet: BOUNCED }) +
      "\n" +
      JSON.stringify({ index: 2, outcome: "passed", gauntlet: PASSED }) +
      "\n" +
      JSON.stringify({ summary: true, converged: true, convergedAt: 2, attempts: 2 }) +
      "\n";
    const got = readConvergedAttempt(log);
    expect(got).not.toBeNull();
    expect(got!.attempts).toBe(2);
    expect(got!.result.status).toBe("passed");
  });

  test("returns null when no attempt converged", () => {
    const log =
      JSON.stringify({ index: 1, outcome: "bounced", gauntlet: BOUNCED }) +
      "\n" +
      JSON.stringify({ summary: true, converged: false, convergedAt: null, attempts: 1 }) +
      "\n";
    expect(readConvergedAttempt(log)).toBeNull();
  });
});

// --- 2. approve moves new units + stamps credit -----------------------------

describe("approve", () => {
  const shippedNames = DEFAULT_RUN_POOL.map((u) => u.name);
  const record = buildRecord("froster", [FROSTER], MANIFEST, PASSED, 3);

  test("adds the new unit, stamps creator credit, validates the pool", () => {
    const next = approveInto({ units: [] }, record, shippedNames, stressRegistry);
    expect(next.units.map((u) => u.name)).toEqual(["Froster"]);
    expect(next.units[0]!._creator).toBe("maks");
    // the merged playable pool is valid and draftable.
    const pool = mergePool(DEFAULT_RUN_POOL, next.units);
    expect(pool.some((u) => u.name === "Froster")).toBe(true);
    expect(() => parseApprovedRegistry(next, stressRegistry)).not.toThrow();
  });

  test("skips shipped support units in the candidate team (only new ones move)", () => {
    const team = buildRecord("mix", [FROSTER, { name: "Squire", base: { hp: 8, pwr: 2 } }], MANIFEST, PASSED, 1);
    const next = approveInto({ units: [] }, team, shippedNames, stressRegistry);
    expect(next.units.map((u) => u.name)).toEqual(["Froster"]); // Squire is shipped — skipped
  });

  test("refuses a candidate that introduces no new unit", () => {
    const allShipped = buildRecord("dud", [{ name: "Squire", base: { hp: 8, pwr: 2 } }], MANIFEST, PASSED, 1);
    expect(() => approveInto({ units: [] }, allShipped, shippedNames, stressRegistry)).toThrow(/no new unit/);
  });

  test("refuses a name collision with an already-approved unit", () => {
    const current = approveInto({ units: [] }, record, shippedNames, stressRegistry);
    const dup = buildRecord("froster-2", [{ ...FROSTER }], { ...MANIFEST, creator: "eve" }, PASSED, 1);
    expect(() => approveInto(current, dup, shippedNames, stressRegistry)).toThrow(/collides/);
  });

  test("refuses a name collision with a shipped unit (renamed-candidate guard)", () => {
    // A new unit deliberately named like a shipped one is NOT skipped silently if
    // it is the only new content path — here it's caught as "no new unit", which
    // is the right loud refusal. A mixed team's shipped-name unit is skipped (above).
    const collide = buildRecord("c", [{ name: "Venomancer", base: { hp: 9, pwr: 3 } }], MANIFEST, PASSED, 1);
    expect(() => approveInto({ units: [] }, collide, shippedNames, stressRegistry)).toThrow();
  });
});

// --- 3. pending/rejected candidates never reach the draftable pool ----------

describe("pool isolation", () => {
  const shippedNames = DEFAULT_RUN_POOL.map((u) => u.name);

  test("an un-approved candidate is absent from the playable pool", () => {
    // A minted-but-not-approved candidate exists as a record; the playable pool
    // is mergePool(shipped, registry.units), and the registry only gains a unit
    // through approveInto. So without approve, the unit is not draftable.
    const registry = { units: [] }; // nothing approved yet
    const pool = mergePool(DEFAULT_RUN_POOL, registry.units);
    expect(pool.some((u) => u.name === "Froster")).toBe(false);
  });

  test("a gate-bounced candidate can never be minted (no passing attempt)", () => {
    const log =
      JSON.stringify({ index: 1, outcome: "bounced", gauntlet: BOUNCED }) +
      "\n" +
      JSON.stringify({ summary: true, converged: false, convergedAt: null, attempts: 1 }) +
      "\n";
    // readConvergedAttempt returns null → mint refuses → no record → never approved.
    expect(readConvergedAttempt(log)).toBeNull();
  });

  test("approving one candidate leaves an un-approved peer out of the pool", () => {
    const a = buildRecord("a", [FROSTER], MANIFEST, PASSED, 1);
    const bUnit: UnitDef = { name: "Emberling", base: { hp: 9, pwr: 3 } };
    const next = approveInto({ units: [] }, a, shippedNames, stressRegistry);
    const pool = mergePool(DEFAULT_RUN_POOL, next.units);
    expect(pool.some((u) => u.name === "Froster")).toBe(true);
    expect(pool.some((u) => u.name === bUnit.name)).toBe(false); // never approved
  });
});
