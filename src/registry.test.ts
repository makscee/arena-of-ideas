// Approved-units registry tests (PRD #013 slice 4): parse/validate, merge by
// name with a loud collision guard, credit extraction, and that the committed
// registry/approved-units.json parses against the live stress registry.

import { describe, expect, test } from "vitest";
import { readFileSync } from "node:fs";
import { fileURLToPath } from "node:url";
import { dirname, join } from "node:path";
import { stressAbilities, stressRegistry } from "./content/stress.js";
import { DEFAULT_RUN_POOL } from "./tunables.js";
import { creditsOf, mergePool, parseApprovedRegistry } from "./registry.js";
import type { ApprovedUnit } from "./registry.js";
import { deserializeRun, initRun, serializeRun } from "./run.js";

const here = dirname(fileURLToPath(import.meta.url));

const FROSTER: ApprovedUnit = { name: "Froster", base: { hp: 11, pwr: 2 }, ability: "Strike", _creator: "maks" };

describe("parseApprovedRegistry", () => {
  test("an empty registry is valid", () => {
    expect(parseApprovedRegistry({ units: [] }, stressRegistry, stressAbilities).units).toEqual([]);
  });

  test("a non-object or missing units array fails loudly", () => {
    expect(() => parseApprovedRegistry(null, stressRegistry, stressAbilities)).toThrow();
    expect(() => parseApprovedRegistry({}, stressRegistry, stressAbilities)).toThrow(/units/);
    expect(() => parseApprovedRegistry({ units: "no" }, stressRegistry, stressAbilities)).toThrow(/units/);
  });

  test("invalid DSL in a unit fails loudly (content gate)", () => {
    const bad = { units: [{ name: "X", base: { hp: 5, pwr: 1 }, abilities: [{ whens: [], selectors: [], effects: [] }] }] };
    expect(() => parseApprovedRegistry(bad, stressRegistry, stressAbilities)).toThrow();
  });

  test("a non-string _creator is rejected", () => {
    expect(() => parseApprovedRegistry({ units: [{ ...FROSTER, _creator: 7 }] }, stressRegistry, stressAbilities)).toThrow(/_creator/);
  });

  test("the committed registry file parses against the live registry", () => {
    const raw = readFileSync(join(here, "..", "registry", "approved-units.json"), "utf8");
    expect(() => parseApprovedRegistry(JSON.parse(raw), stressRegistry, stressAbilities, "approved-units.json")).not.toThrow();
  });
});

describe("mergePool", () => {
  test("appends new units after the base, base order preserved", () => {
    const pool = mergePool(DEFAULT_RUN_POOL, [FROSTER]);
    expect(pool.slice(0, DEFAULT_RUN_POOL.length)).toEqual(DEFAULT_RUN_POOL);
    expect(pool[pool.length - 1]!.name).toBe("Froster");
  });

  test("a name collision with the base is rejected loudly", () => {
    const dup: ApprovedUnit = { name: DEFAULT_RUN_POOL[0]!.name, base: { hp: 1, pwr: 1 } };
    expect(() => mergePool(DEFAULT_RUN_POOL, [dup])).toThrow(/collides/);
  });

  test("does not mutate its inputs", () => {
    const baseLen = DEFAULT_RUN_POOL.length;
    mergePool(DEFAULT_RUN_POOL, [FROSTER]);
    expect(DEFAULT_RUN_POOL.length).toBe(baseLen);
  });
});

describe("creditsOf", () => {
  test("maps credited unit names to their creators", () => {
    expect(creditsOf([FROSTER, { name: "Plain", base: { hp: 5, pwr: 1 } }])).toEqual({ Froster: "maks" });
  });
});

describe("stored runs survive an approval", () => {
  test("a run saved BEFORE an approve (shipped-only pool) still revives", () => {
    // A run persists its own pool BY VALUE (run.ts serializeRun). An approval
    // grows the registry merged into NEW runs only — a stored run revives against
    // its own captured pool, unaffected. This pins that an approval can never
    // brick a run started before it (Cass's localStorage-continuity probe).
    const before = serializeRun(initRun({ seed: 7, pool: [...DEFAULT_RUN_POOL], statuses: stressRegistry, abilities: stressAbilities }));
    // …an approval happens (the registry now has Froster) — does not touch `before`.
    const merged = mergePool(DEFAULT_RUN_POOL, [FROSTER]);
    expect(merged.some((u) => u.name === "Froster")).toBe(true);
    // the old run still deserializes and holds no approved unit.
    const revived = deserializeRun(before);
    expect(revived.pool.some((u) => u.name === "Froster")).toBe(false);
    expect(revived.status).toBe("active");
  });
});
