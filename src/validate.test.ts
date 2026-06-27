// Validator tests — every rejection here corresponds to content the kernel
// would otherwise accept and silently never fire (or crash on at resolve time).
//
// PRD #081 — a Unit references exactly ONE Ability by id (`ability`) into the
// ability registry; inline `abilities[]` on a unit is retired, an ability-less
// unit is rejected, and a dangling ref is rejected. The ability BODIES (the
// wrong-context / typo / reference checks) are validated once, at the ability
// registry (validateAbilityRegistry), in unit-ability context.

import { describe, expect, test } from "vitest";
import {
  MAX_DEFINITION_COMPLEXITY,
  ValidationError,
  assertValidContent,
  complexityOf,
  validateAbilityRegistry,
  validateRegistry,
  validateTeam,
} from "./validate.js";
import { Necromancer, Silencer, Summoner, Venomancer, stressAbilities, stressRegistry } from "./content/stress.js";
import { BOSS_TEAMS, BOOTSTRAP_TEAMS, DEFAULT_RUN_POOL, TOWER_HEIGHT } from "./tunables.js";
import { loadTeamFile } from "./cli.js";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { writeFileSync } from "node:fs";
import type { AbilityRegistry, UnitDef } from "./types.js";

const vanilla: UnitDef = { name: "Grunt", base: { hp: 5, pwr: 1 }, ability: "Strike" };

/** Build a single-entry ability registry for an ability-body test. */
const reg = (def: unknown): AbilityRegistry => ({ Probe: { name: "Probe", family: "Strike", ...(def as object) } as never });

// ---------------------------------------------------------------------------
// 0. The #081 one-ability invariant (the headline of slice 2)
// ---------------------------------------------------------------------------

describe("the one-ability invariant (#081)", () => {
  test("a unit with exactly one registered ability ref is accepted", () => {
    expect(validateTeam([Venomancer], stressRegistry, stressAbilities)).toEqual([]);
    expect(validateTeam([vanilla], stressRegistry, stressAbilities)).toEqual([]);
  });

  test("must-fail-first: an inline `abilities[]` array on a unit is rejected", () => {
    const unit = {
      name: "Inline",
      base: { hp: 5, pwr: 1 },
      abilities: [
        { whens: [{ kind: "trigger", on: { on: "TurnEnd" } }], selectors: [{ kind: "holder" }], effects: [{ kind: "heal", amount: { kind: "const", value: 1 } }] },
      ],
    };
    const issues = validateTeam([unit], stressRegistry, stressAbilities, "teamA");
    expect(issues.some((i) => i.path === "teamA[0].abilities" && /inline `abilities\[\]`.*retired/.test(i.message))).toBe(true);
  });

  test("must-fail-first: an ability-less unit is rejected (no stray effects)", () => {
    const unit = { name: "Bare", base: { hp: 5, pwr: 1 } };
    const issues = validateTeam([unit], stressRegistry, stressAbilities, "teamA");
    expect(issues.some((i) => i.path === "teamA[0].ability" && /must reference exactly one Ability/.test(i.message))).toBe(true);
  });

  test("must-fail-first: a dangling ability ref is rejected", () => {
    const unit = { name: "Ghostly", base: { hp: 5, pwr: 1 }, ability: "Nonexistent" };
    const issues = validateTeam([unit], stressRegistry, stressAbilities, "teamA");
    expect(issues.some((i) => i.path === "teamA[0].ability" && /unknown ability "Nonexistent"/.test(i.message))).toBe(true);
  });

  test("must-fail-first: an AbilityDef with an unknown family is rejected", () => {
    const issues = validateAbilityRegistry(
      { Plasma: { name: "Plasma", family: "Plasma" as never, whens: [{ kind: "trigger", on: { on: "TurnEnd" } }], selectors: [{ kind: "holder" }], effects: [{ kind: "heal", amount: { kind: "const", value: 1 } }] } },
      stressRegistry,
    );
    expect(issues.some((i) => i.path === "abilities.Plasma.family" && /unknown family.*one of/.test(i.message))).toBe(true);
  });

  test("an AbilityDef.name that disagrees with its registry key is rejected", () => {
    const issues = validateAbilityRegistry(
      { Key: { name: "Other", family: "Strike", whens: [{ kind: "trigger", on: { on: "TurnEnd" } }], selectors: [{ kind: "holder" }], effects: [{ kind: "heal", amount: { kind: "const", value: 1 } }] } },
      stressRegistry,
    );
    expect(issues.some((i) => i.path === "abilities.Key" && /must equal its registry key/.test(i.message))).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// 1. Good content passes
// ---------------------------------------------------------------------------

describe("valid content passes", () => {
  test("the stress registry and ability registry are valid", () => {
    expect(validateRegistry(stressRegistry)).toEqual([]);
    expect(validateAbilityRegistry(stressAbilities, stressRegistry)).toEqual([]);
  });

  test("the stress units and a status-carrying unit are valid", () => {
    const team: UnitDef[] = [
      Venomancer,
      Summoner,
      Silencer,
      Necromancer,
      { name: "Shielded", base: { hp: 10, pwr: 1 }, ability: "Strike", statuses: [{ status: "Shield", stacks: 2 }] },
    ];
    expect(validateTeam(team, stressRegistry, stressAbilities)).toEqual([]);
    expect(() => assertValidContent(team, stressRegistry, stressAbilities)).not.toThrow();
  });

  test("the example team files load and validate", () => {
    const EXAMPLES = join(new URL(".", import.meta.url).pathname, "..", "examples");
    expect(() => loadTeamFile(join(EXAMPLES, "team-alpha.json"))).not.toThrow();
    expect(() => loadTeamFile(join(EXAMPLES, "team-beta.json"))).not.toThrow();
  });
});

// ---------------------------------------------------------------------------
// 2. Wrong-context parts are rejected — now at the ability registry
// ---------------------------------------------------------------------------

describe("wrong-context parts (ability bodies)", () => {
  test("an interceptor-context effect on a trigger-only ability is rejected", () => {
    const issues = validateAbilityRegistry(
      reg({
        whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
        selectors: [{ kind: "frontEnemy" }],
        effects: [{ kind: "cancel" }],
      }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/interceptor-context effect.*only trigger whens.*can never run/);
  });

  test("a trigger-context effect on an interceptor-only ability is rejected", () => {
    const issues = validateAbilityRegistry(
      reg({
        whens: [{ kind: "interceptor", on: { on: "Hurt", unit: "holder" } }],
        selectors: [{ kind: "holder" }],
        effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }],
      }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/trigger-context effect.*only interceptor whens/);
  });

  test("a mixed-when ability may carry both atom families", () => {
    const issues = validateAbilityRegistry(
      reg({
        whens: [
          { kind: "trigger", on: { on: "TurnEnd" } },
          { kind: "interceptor", on: { on: "Hurt", unit: "holder" } },
        ],
        selectors: [{ kind: "holder" }],
        effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }, { kind: "cancel" }],
      }),
      stressRegistry,
    );
    expect(issues).toEqual([]);
  });
});

// ---------------------------------------------------------------------------
// 3. Unknown kinds and bad references — in ability bodies
// ---------------------------------------------------------------------------

describe("unknown kinds (ability bodies)", () => {
  test("misspelled effect kind is rejected", () => {
    const issues = validateAbilityRegistry(
      reg({ whens: [{ kind: "trigger", on: { on: "TurnEnd" } }], selectors: [{ kind: "holder" }], effects: [{ kind: "dmage", amount: { kind: "const", value: 1 } }] }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/unknown effect kind "dmage"/);
  });

  test("misspelled trigger event name is rejected", () => {
    const issues = validateAbilityRegistry(
      reg({ whens: [{ kind: "trigger", on: { on: "TurnEnded" } }], selectors: [{ kind: "holder" }], effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }] }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/unknown event "TurnEnded"/);
  });

  test("unknown selector kind is rejected", () => {
    const issues = validateAbilityRegistry(
      reg({ whens: [{ kind: "trigger", on: { on: "TurnEnd" } }], selectors: [{ kind: "nearestEnemy" }], effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }] }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/unknown selector kind "nearestEnemy"/);
  });

  test("unknown unit filter on a pattern is rejected", () => {
    const issues = validateAbilityRegistry(
      reg({ whens: [{ kind: "trigger", on: { on: "Hurt", unit: "enemey" } }], selectors: [{ kind: "holder" }], effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }] }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/unknown unit filter "enemey"/);
  });

  test("typo'd pattern field (strikr) is rejected, not silently ignored", () => {
    const issues = validateAbilityRegistry(
      reg({ whens: [{ kind: "trigger", on: { on: "Strike", strikr: "holder" } }], selectors: [{ kind: "holder" }], effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }] }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/"Strike" patterns take no "strikr" field/);
  });

  test("applyStatus to an unregistered status is rejected", () => {
    const issues = validateAbilityRegistry(
      reg({ whens: [{ kind: "trigger", on: { on: "TurnEnd" } }], selectors: [{ kind: "frontEnemy" }], effects: [{ kind: "applyStatus", status: "Doom", stacks: { kind: "const", value: 1 } }] }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/unknown status "Doom"/);
  });

  test("a stacks amount on a unit ability (no owning status) is rejected", () => {
    const issues = validateAbilityRegistry(
      reg({ whens: [{ kind: "trigger", on: { on: "TurnEnd" } }], selectors: [{ kind: "holder" }], effects: [{ kind: "damage", amount: { kind: "stacks" } }] }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/owning status.*evaluates to 0/);
  });

  test("a summoned unit is validated recursively (and needs its own ability ref)", () => {
    const issues = validateAbilityRegistry(
      reg({
        whens: [{ kind: "trigger", on: { on: "Death", unit: "holder" } }],
        selectors: [{ kind: "holder" }],
        effects: [{ kind: "summon", unit: { name: "Imp", base: { hp: -2, pwr: 1 }, ability: "Strike" } }],
      }),
      stressRegistry,
    );
    expect(issues.some((i) => i.path === "abilities.Probe.effects[0].unit.base.hp")).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// 4. Bad status references and malformed bundles — still on the unit
// ---------------------------------------------------------------------------

describe("bad references and malformed bundles", () => {
  test("unknown status in a unit's status bundle is rejected", () => {
    const unit = { name: "Poisoned", base: { hp: 5, pwr: 1 }, ability: "Strike", statuses: [{ status: "Poson", stacks: 2 }] };
    const issues = validateTeam([unit], stressRegistry, stressAbilities);
    expect(issues.map((i) => i.message).join("\n")).toMatch(/unknown status "Poson"/);
  });

  test("non-positive stacks in a status bundle are rejected", () => {
    const unit = { name: "Zero", base: { hp: 5, pwr: 1 }, ability: "Strike", statuses: [{ status: "Shield", stacks: 0 }] };
    const issues = validateTeam([unit], stressRegistry, stressAbilities);
    expect(issues.map((i) => i.message).join("\n")).toMatch(/stacks must be a positive integer/);
  });

  test("negative base stats are rejected", () => {
    const issues = validateTeam([{ name: "Broken", base: { hp: -1, pwr: 1 }, ability: "Strike" }], stressRegistry, stressAbilities);
    expect(issues.map((i) => i.message).join("\n")).toMatch(/hp must be a non-negative integer/);
  });

  test("team size 0 and 6 are rejected", () => {
    expect(validateTeam([], stressRegistry, stressAbilities).map((i) => i.message).join("\n")).toMatch(/1\.\.5 units/);
    const six = Array.from({ length: 6 }, () => vanilla);
    expect(validateTeam(six, stressRegistry, stressAbilities).map((i) => i.message).join("\n")).toMatch(/1\.\.5 units/);
  });
});

// ---------------------------------------------------------------------------
// 5. The CLI fails loudly on a bad team file
// ---------------------------------------------------------------------------

describe("CLI team loading rejects bad content", () => {
  test("a team file with an ability-less unit fails to load with a specific error", () => {
    const path = join(tmpdir(), `aoi-validate-bad-${Date.now()}.json`);
    writeFileSync(path, JSON.stringify({ units: [{ name: "Bare", base: { hp: 5, pwr: 1 } }] }), "utf8");
    expect(() => loadTeamFile(path)).toThrow(/must reference exactly one Ability/);
  });

  test("a team file with a dangling ability ref fails to load", () => {
    const path = join(tmpdir(), `aoi-validate-dangle-${Date.now()}.json`);
    writeFileSync(path, JSON.stringify({ units: [{ name: "Ghost", base: { hp: 5, pwr: 1 }, ability: "Nope" }] }), "utf8");
    expect(() => loadTeamFile(path)).toThrow(/unknown ability "Nope"/);
  });
});

// ---------------------------------------------------------------------------
// 6. The complexity cap — a card is a fixed budget (PRD #078 slice 4 / #081)
// ---------------------------------------------------------------------------
//
// A card has ONE fixed size, and its size IS the behavior budget. With #081 a
// unit's whole behavior is its one Ability, so the cap is enforced on each
// AbilityDef (and on each StatusDef, content on a card too).

describe("complexity cap", () => {
  test("every shipped status fits the budget", () => {
    for (const [name, def] of Object.entries(stressRegistry)) {
      expect(complexityOf(def as unknown as Record<string, unknown>), `status ${name}`).toBeLessThanOrEqual(MAX_DEFINITION_COMPLEXITY);
    }
    expect(validateRegistry(stressRegistry)).toEqual([]);
  });

  test("every shipped ability fits the budget", () => {
    expect(validateAbilityRegistry(stressAbilities, stressRegistry)).toEqual([]);
  });

  test("the richest shipped status (Poison) scores 4, well under the cap", () => {
    expect(complexityOf(stressRegistry["Poison"] as unknown as Record<string, unknown>)).toBe(4);
  });

  test("the shipped run pool and bootstrap teams all fit", () => {
    for (const u of DEFAULT_RUN_POOL) {
      expect(validateTeam([u], stressRegistry, stressAbilities)).toEqual([]);
    }
    for (const team of BOOTSTRAP_TEAMS.flat()) {
      expect(validateTeam([...team], stressRegistry, stressAbilities)).toEqual([]);
    }
    expect(validateTeam([...BOSS_TEAMS[TOWER_HEIGHT - 1]!], stressRegistry, stressAbilities)).toEqual([]);
  });

  // An ability assembled to exactly the budget: 1 when + 1 selector + 6 effects = 8.
  const atCapBody = {
    whens: [{ kind: "trigger", on: { on: "BattleStart" } }],
    selectors: [{ kind: "holder" }],
    effects: Array.from({ length: 6 }, () => ({ kind: "heal", amount: { kind: "const", value: 1 } })),
  };

  test("an at-cap ability (complexity 8) passes", () => {
    expect(validateAbilityRegistry(reg(atCapBody), stressRegistry)).toEqual([]);
  });

  test("a just-over-cap ability (complexity 9) is rejected with a clear error", () => {
    const body = { ...atCapBody, effects: [...atCapBody.effects, { kind: "heal", amount: { kind: "const", value: 1 } }] };
    const issues = validateAbilityRegistry(reg(body), stressRegistry);
    expect(issues.some((i) => i.path === "abilities.Probe" && /complexity 9 exceeds the card budget of 8/.test(i.message))).toBe(true);
  });

  test("a multiplicative-depth ability (complexity 12) is rejected", () => {
    const body = {
      whens: [
        { kind: "trigger", on: { on: "BattleStart" } },
        { kind: "trigger", on: { on: "TurnStart" } },
      ],
      selectors: [{ kind: "holder" }, { kind: "allEnemies" }],
      effects: [
        { kind: "damage", amount: { kind: "const", value: 1 } },
        { kind: "heal", amount: { kind: "const", value: 1 } },
      ],
    };
    const issues = validateAbilityRegistry(reg(body), stressRegistry);
    expect(issues.some((i) => /complexity 12 exceeds the card budget/.test(i.message))).toBe(true);
  });

  test("an over-cap StatusDef is rejected at registry validation", () => {
    const fatRegistry = {
      ...stressRegistry,
      Overloaded: {
        name: "Overloaded",
        abilities: [{ ...atCapBody, effects: [...atCapBody.effects, { kind: "heal", amount: { kind: "const", value: 1 } }] }],
      },
    };
    const issues = validateRegistry(fatRegistry as never);
    expect(issues.some((i) => i.path === "registry.Overloaded" && /complexity 9 exceeds/.test(i.message))).toBe(true);
  });

  test("must-fail-first: the over-cap ability is rejected by the cap alone, not pre-existing checks", () => {
    const body = { ...atCapBody, effects: [...atCapBody.effects, { kind: "heal", amount: { kind: "const", value: 1 } }] };
    const issues = validateAbilityRegistry(reg(body), stressRegistry);
    const nonComplexity = issues.filter((i) => !/exceeds the card budget/.test(i.message));
    expect(nonComplexity).toEqual([]);
  });
});
