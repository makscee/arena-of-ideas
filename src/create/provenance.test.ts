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
import { readFileSync } from "node:fs";
import { join } from "node:path";
import { stressAbilities, stressRegistry } from "../content/stress.js";
import { DEFAULT_RUN_POOL } from "../tunables.js";
import { mergePool, parseApprovedRegistry } from "../registry.js";
import type { UnitDef } from "../types.js";
import {
  buildRecord,
  gateStatsOf,
  readConvergedAttempt,
  serializeRecord,
} from "./provenance.js";
import type { CandidateRecord, RunManifest } from "./provenance.js";
import type { GauntletResult } from "./worker.js";
import { parseCandidateRecord } from "./candidates.js";
import { approveInto, reSimCandidate, ResimRejectedError, RESIM_WIN_RATE_TOLERANCE } from "./approve.js";

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

// PRD #081: a unit references one shipped ability by id. Venom IS "apply 2
// Poison to the front enemy after it strikes" — the same mechanic this fixture
// carried inline before the migration.
const FROSTER: UnitDef = {
  name: "Froster",
  base: { hp: 11, pwr: 2 },
  statuses: [{ status: "Shield", stacks: 2 }],
  ability: "Venom",
};

// The real committed frostbite-striker team + its true recorded gate stats. Used
// by the approve / pool-isolation blocks because PRD #067 slice 3 makes approve
// RE-SIM: a record only approves if its units genuinely re-sim in-band AND its
// recorded win-rate matches the re-sim. FROSTER alone re-sims underpowered (0%),
// so the bookkeeping tests below drive the honest, in-band team the gate accepts.
// (Loaded from the committed candidate so the fixture and the real artifact never
// drift — the frostbite-striker convergence is the slice's honest reference.)
const HONEST: CandidateRecord = parseCandidateRecord(
  JSON.parse(readFileSync(join(import.meta.dirname, "..", "..", "candidates", "frostbite-striker.json"), "utf8")),
  stressRegistry,
  stressAbilities,
  "frostbite-striker.json",
);
/** The honest team's NEW unit (Glacier and Squire are shipped/skipped). */
const HONEST_NEW_NAME = "Frostbiter";

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
    const reparsed = parseCandidateRecord(JSON.parse(once), stressRegistry, stressAbilities, "froster.json");
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
    raw.units[0].ability = "notARealAbility"; // a dangling ability ref (#081) is content the validator must catch
    expect(() => parseCandidateRecord(raw, stressRegistry, stressAbilities, "bad.json")).toThrow();
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
  // The bookkeeping tests drive the HONEST team — approve RE-SIMS (PRD #067 s3),
  // so only a record whose units genuinely land in-band reaches the merge logic.
  // Frostbiter + Glacier are new; Squire is shipped (skipped).
  const honest = (id: string, creator = "maks"): CandidateRecord => ({
    ...HONEST,
    id,
    provenance: { ...HONEST.provenance, creator },
  });

  test("adds the new units, stamps creator credit, validates the pool", () => {
    const next = approveInto({ units: [] }, honest("frostbite-striker"), shippedNames, stressRegistry, stressAbilities);
    expect(next.units.map((u) => u.name)).toEqual(["Frostbiter", "Glacier"]); // Squire shipped → skipped
    expect(next.units.every((u) => u._creator === "maks")).toBe(true);
    // the merged playable pool is valid and draftable.
    const pool = mergePool(DEFAULT_RUN_POOL, next.units);
    expect(pool.some((u) => u.name === HONEST_NEW_NAME)).toBe(true);
    expect(() => parseApprovedRegistry(next, stressRegistry, stressAbilities)).not.toThrow();
  });

  test("skips shipped support units in the candidate team (only new ones move)", () => {
    // Squire is part of the honest team and is shipped — it is skipped, not moved.
    const next = approveInto({ units: [] }, honest("frostbite-striker"), shippedNames, stressRegistry, stressAbilities);
    expect(next.units.map((u) => u.name)).not.toContain("Squire");
  });

  test("refuses a candidate that introduces no new unit", () => {
    const allShipped = buildRecord("dud", [{ name: "Squire", base: { hp: 8, pwr: 2 }, ability: "Strike" }], MANIFEST, PASSED, 1);
    expect(() => approveInto({ units: [] }, allShipped, shippedNames, stressRegistry, stressAbilities)).toThrow(/no new unit/);
  });

  test("refuses a name collision with an already-approved unit", () => {
    const current = approveInto({ units: [] }, honest("frostbite-striker"), shippedNames, stressRegistry, stressAbilities);
    // A second honest record (same in-band team, different id/creator) collides on Frostbiter.
    expect(() => approveInto(current, honest("frostbite-2", "eve"), shippedNames, stressRegistry, stressAbilities)).toThrow(/collides/);
  });

  test("refuses a name collision with a shipped unit (renamed-candidate guard)", () => {
    // A new unit deliberately named like a shipped one is NOT skipped silently if
    // it is the only new content path — here it's caught as "no new unit", which
    // is the right loud refusal. A mixed team's shipped-name unit is skipped (above).
    const collide = buildRecord("c", [{ name: "Venomancer", base: { hp: 9, pwr: 3 }, ability: "Strike" }], MANIFEST, PASSED, 1);
    expect(() => approveInto({ units: [] }, collide, shippedNames, stressRegistry, stressAbilities)).toThrow();
  });
});

// --- 2b. approve RE-SIMS — a forged in-band record cannot self-approve --------
//
// PRD #067 slice 3. The hole (Cass, #013 slice 4): approve TRUSTED the candidate
// file's recorded gate stats and promoted with no re-sim, so a hand-forged
// "in-band" stat over truly-overtuned data (a 999-power unit) got approved. This
// block reproduces that forge and proves approve now re-sims and refuses it.

describe("approve re-sims (forge containment)", () => {
  const shippedNames = DEFAULT_RUN_POOL.map((u) => u.name);

  // THE FORGE: a 999-power unit whose recorded gate LIES — it claims the honest
  // in-band stats (PASSED, 0.5) while the unit data is wildly overtuned. Content-
  // valid (a big base stat is legal DSL), so it sails past the validator; only a
  // re-sim catches that it is not actually in-band.
  const OVERTUNED: UnitDef = { name: "OvertunedOgre", base: { hp: 999, pwr: 999 }, ability: "Strike" };
  const forged = buildRecord("forged-ogre", [OVERTUNED], MANIFEST, PASSED, 1);

  test("the forged record CLAIMS in-band (the lie the old approve trusted)", () => {
    // The record on disk asserts an in-band verdict — exactly what pre-slice
    // approve read and believed. The data underneath does not earn it.
    expect(forged.gate.verdict).toBe("in-band");
    expect(forged.gate.overallWinRate).toBe(0.5);
  });

  test("PRE-slice behaviour: trusting the record would approve the forge", () => {
    // Simulate the OLD approve (bookkeeping only, no re-sim): the structural guards
    // pass (OvertunedOgre is new + non-colliding), so a trust-the-record approve
    // would have promoted it. This is the hole the slice closes.
    const newUnits = forged.units.filter((u) => !new Set(shippedNames).has(u.name));
    expect(newUnits.map((u) => u.name)).toEqual(["OvertunedOgre"]); // would have been promoted
  });

  test("POST-slice: approve re-sims and REJECTS the forge loudly with the real numbers", () => {
    let thrown: unknown;
    try {
      approveInto({ units: [] }, forged, shippedNames, stressRegistry, stressAbilities);
    } catch (err) {
      thrown = err;
    }
    expect(thrown).toBeInstanceOf(ResimRejectedError);
    const e = thrown as ResimRejectedError;
    // Loud: the message names the out-of-band verdict and the re-sim win-rate.
    expect(e.message).toMatch(/OUT OF BAND/);
    expect(e.message).toMatch(/overtuned/);
    // The re-sim numbers are the truth, not the forged 0.5 — a 999-power unit
    // crushes the reference meta, so the real win-rate is far above the band.
    expect(e.report.verdict).toBe("overtuned");
    expect(e.report.pass).toBe(false);
    expect(e.report.overallWinRate).toBeGreaterThan(forged.gate.band.max);
    expect(e.report.overallWinRate).not.toBe(forged.gate.overallWinRate);
  });

  test("the forged unit never reaches the registry", () => {
    expect(() => approveInto({ units: [] }, forged, shippedNames, stressRegistry, stressAbilities)).toThrow(ResimRejectedError);
    // and nothing was promoted (approveInto is pure; it throws before returning).
  });

  // A subtler forge: honest-LOOKING units but a recorded win-rate that disagrees
  // with the truth. Even if the verdict were in-band, a lied-about number is
  // rejected by the tolerance check — the record must match the re-sim.
  test("rejects a record whose recorded win-rate disagrees with the re-sim (the record lied)", () => {
    // Start from the honest team (re-sims in-band at 0.6467) but tamper the
    // recorded win-rate to a different in-band value (0.40). Verdict would pass,
    // but the recorded number no longer matches the re-sim → rejected.
    const lied: CandidateRecord = {
      ...HONEST,
      id: "honest-but-lied",
      gate: { ...HONEST.gate, overallWinRate: 0.4 },
    };
    let thrown: unknown;
    try {
      approveInto({ units: [] }, lied, shippedNames, stressRegistry, stressAbilities);
    } catch (err) {
      thrown = err;
    }
    expect(thrown).toBeInstanceOf(ResimRejectedError);
    expect((thrown as Error).message).toMatch(/disagrees with the re-sim|record lied/);
  });

  test("an honest in-band candidate still approves (the re-sim matches its receipt)", () => {
    // The honest frostbite-striker convergence: its recorded stats ARE the re-sim,
    // so it passes both checks and promotes. The honest path is unaffected.
    const report = reSimCandidate(HONEST, stressRegistry, stressAbilities);
    expect(report.pass).toBe(true);
    expect(report.verdict).toBe("in-band");
    // recorded == re-sim, within tolerance (deterministic sim → exact match).
    expect(Math.abs(HONEST.gate.overallWinRate - report.overallWinRate)).toBeLessThanOrEqual(
      RESIM_WIN_RATE_TOLERANCE,
    );
    const next = approveInto({ units: [] }, HONEST, shippedNames, stressRegistry, stressAbilities);
    expect(next.units.map((u) => u.name)).toEqual(["Frostbiter", "Glacier"]);
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
    // Approve the honest (re-simmable) team; an un-approved peer stays out.
    const bUnit: UnitDef = { name: "Emberling", base: { hp: 9, pwr: 3 }, ability: "Strike" };
    const next = approveInto({ units: [] }, HONEST, shippedNames, stressRegistry, stressAbilities);
    const pool = mergePool(DEFAULT_RUN_POOL, next.units);
    expect(pool.some((u) => u.name === HONEST_NEW_NAME)).toBe(true);
    expect(pool.some((u) => u.name === bUnit.name)).toBe(false); // never approved
  });
});
