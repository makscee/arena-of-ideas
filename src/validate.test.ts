// Validator tests — every rejection here corresponds to content the kernel
// would otherwise accept and silently never fire.

import { describe, expect, test } from "vitest";
import { MAX_DEFINITION_COMPLEXITY, ValidationError, assertValidContent, complexityOf, validateRegistry, validateTeam } from "./validate.js";
import { Necromancer, Silencer, Summoner, Venomancer, stressRegistry } from "./content/stress.js";
import { BOSS_TEAMS, BOOTSTRAP_TEAMS, DEFAULT_RUN_POOL, TOWER_HEIGHT } from "./tunables.js";
import { loadTeamFile } from "./cli.js";
import { join } from "node:path";
import { tmpdir } from "node:os";
import { writeFileSync } from "node:fs";
import type { UnitDef } from "./types.js";

const vanilla: UnitDef = { name: "Grunt", base: { hp: 5, pwr: 1 } };

// ---------------------------------------------------------------------------
// 1. Good content passes
// ---------------------------------------------------------------------------

describe("valid content passes", () => {
  test("the stress registry itself is valid", () => {
    expect(validateRegistry(stressRegistry)).toEqual([]);
  });

  test("the stress units and a status-carrying unit are valid", () => {
    const team: UnitDef[] = [
      Venomancer,
      Summoner,
      Silencer,
      Necromancer,
      { name: "Shielded", base: { hp: 10, pwr: 1 }, statuses: [{ status: "Shield", stacks: 2 }] },
    ];
    expect(validateTeam(team, stressRegistry)).toEqual([]);
    expect(() => assertValidContent(team, stressRegistry)).not.toThrow();
  });

  test("the example team files load and validate", () => {
    const EXAMPLES = join(new URL(".", import.meta.url).pathname, "..", "examples");
    expect(() => loadTeamFile(join(EXAMPLES, "team-alpha.json"))).not.toThrow();
    expect(() => loadTeamFile(join(EXAMPLES, "team-beta.json"))).not.toThrow();
  });
});

// ---------------------------------------------------------------------------
// 2. Wrong-context parts are rejected (the headline check)
// ---------------------------------------------------------------------------

describe("wrong-context parts", () => {
  test("an interceptor-context effect on a trigger-only ability is rejected", () => {
    // "Freeze the enemy when I strike" — but cancel only runs in interceptor
    // context; today's kernel would run this battle and never fire it.
    const unit: UnitDef = {
      name: "Miswired",
      base: { hp: 5, pwr: 1 },
      abilities: [
        {
          whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
          selectors: [{ kind: "frontEnemy" }],
          effects: [{ kind: "cancel" }],
        },
      ],
    };
    const issues = validateTeam([unit], stressRegistry, "teamA");
    expect(issues).toHaveLength(1);
    expect(issues[0]!.path).toBe("teamA[0].abilities[0].effects[0]");
    expect(issues[0]!.message).toMatch(/interceptor-context effect.*only trigger whens.*can never run/);
    expect(() => assertValidContent([unit], stressRegistry, "teamA")).toThrow(ValidationError);
  });

  test("a trigger-context effect on an interceptor-only ability is rejected", () => {
    const unit: UnitDef = {
      name: "Miswired",
      base: { hp: 5, pwr: 1 },
      abilities: [
        {
          whens: [{ kind: "interceptor", on: { on: "Hurt", unit: "holder" } }],
          selectors: [{ kind: "holder" }],
          effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }],
        },
      ],
    };
    const issues = validateTeam([unit], stressRegistry);
    expect(issues.map((i) => i.message).join("\n")).toMatch(/trigger-context effect.*only interceptor whens/);
  });

  test("a mixed-when ability may carry both atom families", () => {
    const unit: UnitDef = {
      name: "Mixed",
      base: { hp: 5, pwr: 1 },
      abilities: [
        {
          whens: [
            { kind: "trigger", on: { on: "TurnEnd" } },
            { kind: "interceptor", on: { on: "Hurt", unit: "holder" } },
          ],
          selectors: [{ kind: "holder" }],
          effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }, { kind: "cancel" }],
        },
      ],
    };
    expect(validateTeam([unit], stressRegistry)).toEqual([]);
  });
});

// ---------------------------------------------------------------------------
// 3. Unknown kinds and bad references
// ---------------------------------------------------------------------------

describe("unknown kinds", () => {
  const withAbility = (ability: unknown): unknown[] => [
    { name: "Typo", base: { hp: 5, pwr: 1 }, abilities: [ability] },
  ];

  test("misspelled effect kind is rejected", () => {
    const issues = validateTeam(
      withAbility({
        whens: [{ kind: "trigger", on: { on: "TurnEnd" } }],
        selectors: [{ kind: "holder" }],
        effects: [{ kind: "dmage", amount: { kind: "const", value: 1 } }],
      }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/unknown effect kind "dmage"/);
  });

  test("misspelled trigger event name is rejected", () => {
    const issues = validateTeam(
      withAbility({
        whens: [{ kind: "trigger", on: { on: "TurnEnded" } }],
        selectors: [{ kind: "holder" }],
        effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }],
      }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/unknown event "TurnEnded"/);
  });

  test("unknown selector kind is rejected", () => {
    const issues = validateTeam(
      withAbility({
        whens: [{ kind: "trigger", on: { on: "TurnEnd" } }],
        selectors: [{ kind: "nearestEnemy" }],
        effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }],
      }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/unknown selector kind "nearestEnemy"/);
  });

  test("bad target context: unknown unit filter on a pattern is rejected", () => {
    const issues = validateTeam(
      withAbility({
        whens: [{ kind: "trigger", on: { on: "Hurt", unit: "enemey" } }],
        selectors: [{ kind: "holder" }],
        effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }],
      }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/unknown unit filter "enemey"/);
  });

  test("typo'd pattern field (strikr) is rejected, not silently ignored", () => {
    const issues = validateTeam(
      withAbility({
        whens: [{ kind: "trigger", on: { on: "Strike", strikr: "holder" } }],
        selectors: [{ kind: "holder" }],
        effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }],
      }),
      stressRegistry,
    );
    expect(issues.map((i) => i.message).join("\n")).toMatch(/"Strike" patterns take no "strikr" field/);
  });
});

describe("bad references and malformed bundles", () => {
  test("unknown status in a unit's status bundle is rejected", () => {
    const unit = { name: "Poisoned", base: { hp: 5, pwr: 1 }, statuses: [{ status: "Poson", stacks: 2 }] };
    const issues = validateTeam([unit], stressRegistry);
    expect(issues.map((i) => i.message).join("\n")).toMatch(/unknown status "Poson"/);
  });

  test("non-positive stacks in a status bundle are rejected", () => {
    const unit = { name: "Zero", base: { hp: 5, pwr: 1 }, statuses: [{ status: "Shield", stacks: 0 }] };
    const issues = validateTeam([unit], stressRegistry);
    expect(issues.map((i) => i.message).join("\n")).toMatch(/stacks must be a positive integer/);
  });

  test("applyStatus to an unregistered status is rejected", () => {
    const unit: UnitDef = {
      name: "Caster",
      base: { hp: 5, pwr: 1 },
      abilities: [
        {
          whens: [{ kind: "trigger", on: { on: "TurnEnd" } }],
          selectors: [{ kind: "frontEnemy" }],
          effects: [{ kind: "applyStatus", status: "Doom", stacks: { kind: "const", value: 1 } }],
        },
      ],
    };
    const issues = validateTeam([unit], stressRegistry);
    expect(issues.map((i) => i.message).join("\n")).toMatch(/unknown status "Doom"/);
  });

  test("a stacks amount on a unit ability (no owning status) is rejected", () => {
    const unit: UnitDef = {
      name: "Stackless",
      base: { hp: 5, pwr: 1 },
      abilities: [
        {
          whens: [{ kind: "trigger", on: { on: "TurnEnd" } }],
          selectors: [{ kind: "holder" }],
          effects: [{ kind: "damage", amount: { kind: "stacks" } }],
        },
      ],
    };
    const issues = validateTeam([unit], stressRegistry);
    expect(issues.map((i) => i.message).join("\n")).toMatch(/owning status.*evaluates to 0/);
  });

  test("negative base stats are rejected", () => {
    const issues = validateTeam([{ name: "Broken", base: { hp: -1, pwr: 1 } }], stressRegistry);
    expect(issues.map((i) => i.message).join("\n")).toMatch(/hp must be a non-negative integer/);
  });

  test("team size 0 and 6 are rejected", () => {
    expect(validateTeam([], stressRegistry).map((i) => i.message).join("\n")).toMatch(/1\.\.5 units/);
    const six = Array.from({ length: 6 }, () => vanilla);
    expect(validateTeam(six, stressRegistry).map((i) => i.message).join("\n")).toMatch(/1\.\.5 units/);
  });

  test("a summoned unit is validated recursively", () => {
    const unit: UnitDef = {
      name: "BadSummoner",
      base: { hp: 5, pwr: 1 },
      abilities: [
        {
          whens: [{ kind: "trigger", on: { on: "Death", unit: "holder" } }],
          selectors: [{ kind: "holder" }],
          effects: [{ kind: "summon", unit: { name: "Imp", base: { hp: -2, pwr: 1 } } }],
        },
      ],
    };
    const issues = validateTeam([unit], stressRegistry, "teamA");
    expect(issues[0]!.path).toBe("teamA[0].abilities[0].effects[0].unit.base.hp");
  });
});

// ---------------------------------------------------------------------------
// 4. The CLI fails loudly on a bad team file
// ---------------------------------------------------------------------------

describe("CLI team loading rejects bad content", () => {
  test("a team file with a wrong-context part fails to load with a specific error", () => {
    const path = join(tmpdir(), `aoi-validate-bad-${Date.now()}.json`);
    writeFileSync(
      path,
      JSON.stringify({
        units: [
          {
            name: "Miswired",
            base: { hp: 5, pwr: 1 },
            abilities: [
              {
                whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
                selectors: [{ kind: "frontEnemy" }],
                effects: [{ kind: "cancel" }],
              },
            ],
          },
        ],
      }),
      "utf8",
    );
    expect(() => loadTeamFile(path)).toThrow(/interceptor-context effect.*can never run/);
  });
});

// ---------------------------------------------------------------------------
// 5. The complexity cap — a card is a fixed budget (PRD #078 slice 4)
// ---------------------------------------------------------------------------
//
// A card has ONE fixed size, and its size IS the behavior budget: a Unit or
// Status whose composed Parts overflow the card cannot exist. Enforced as
// content here, so the cap holds against authored AND AI-generated content
// alike. Cap = MAX_DEFINITION_COMPLEXITY (8); the metric is complexityOf.

describe("complexity cap", () => {
  // All shipped content must keep fitting — the cap must not retroactively
  // reject a unit/status the game already ships.
  test("every shipped status fits the budget", () => {
    for (const [name, def] of Object.entries(stressRegistry)) {
      expect(complexityOf(def as unknown as Record<string, unknown>),
        `status ${name}`).toBeLessThanOrEqual(MAX_DEFINITION_COMPLEXITY);
    }
    expect(validateRegistry(stressRegistry)).toEqual([]);
  });

  test("the richest shipped status (Poison) scores 4, well under the cap", () => {
    expect(complexityOf(stressRegistry["Poison"] as unknown as Record<string, unknown>)).toBe(4);
  });

  test("the shipped run pool and bootstrap teams all fit", () => {
    for (const u of DEFAULT_RUN_POOL) {
      expect(validateTeam([u], stressRegistry)).toEqual([]);
    }
    for (const team of BOOTSTRAP_TEAMS.flat()) {
      expect(validateTeam([...team], stressRegistry)).toEqual([]);
    }
    expect(validateTeam([...BOSS_TEAMS[TOWER_HEIGHT - 1]!], stressRegistry)).toEqual([]);
  });

  // A unit assembled exactly to the budget: 1 when + 1 selector + 6 effects = 8.
  const atCap: UnitDef = {
    name: "AtCap",
    base: { hp: 9, pwr: 2 },
    abilities: [
      {
        whens: [{ kind: "trigger", on: { on: "BattleStart" } }],
        selectors: [{ kind: "holder" }],
        effects: [
          { kind: "heal", amount: { kind: "const", value: 1 } },
          { kind: "heal", amount: { kind: "const", value: 1 } },
          { kind: "heal", amount: { kind: "const", value: 1 } },
          { kind: "heal", amount: { kind: "const", value: 1 } },
          { kind: "heal", amount: { kind: "const", value: 1 } },
          { kind: "heal", amount: { kind: "const", value: 1 } },
        ],
      },
    ],
  };

  test("an at-cap unit (complexity 8) passes", () => {
    expect(complexityOf(atCap as unknown as Record<string, unknown>)).toBe(MAX_DEFINITION_COMPLEXITY);
    expect(validateTeam([atCap], stressRegistry)).toEqual([]);
    expect(() => assertValidContent([atCap], stressRegistry)).not.toThrow();
  });

  // Just-over the boundary: the at-cap unit plus one more effect = complexity 9.
  const justOver: UnitDef = {
    ...atCap,
    name: "JustOver",
    abilities: [
      {
        ...atCap.abilities![0]!,
        effects: [...atCap.abilities![0]!.effects, { kind: "heal", amount: { kind: "const", value: 1 } }],
      },
    ],
  };

  test("a just-over-cap unit (complexity 9) is rejected with a clear error", () => {
    expect(complexityOf(justOver as unknown as Record<string, unknown>)).toBe(MAX_DEFINITION_COMPLEXITY + 1);
    const issues = validateTeam([justOver], stressRegistry, "teamA");
    expect(issues).toHaveLength(1);
    expect(issues[0]!.path).toBe("teamA[0]");
    expect(issues[0]!.message).toMatch(/complexity 9 exceeds the card budget of 8/);
    expect(() => assertValidContent([justOver], stressRegistry, "teamA")).toThrow(ValidationError);
  });

  // A clearly over-cap unit: a single 2-when × 2-selector × 2-effect ability,
  // the multiplicative depth the budget exists to stop. Parts = 6, surcharge =
  // 2·2·2 − 2 = 6, complexity = 12.
  const wayOver: UnitDef = {
    name: "WayOver",
    base: { hp: 9, pwr: 2 },
    abilities: [
      {
        whens: [
          { kind: "trigger", on: { on: "BattleStart" } },
          { kind: "trigger", on: { on: "TurnStart" } },
        ],
        selectors: [{ kind: "holder" }, { kind: "allEnemies" }],
        effects: [
          { kind: "damage", amount: { kind: "const", value: 1 } },
          { kind: "heal", amount: { kind: "const", value: 1 } },
        ],
      },
    ],
  };

  test("a multiplicative-depth unit (complexity 12) is rejected", () => {
    expect(complexityOf(wayOver as unknown as Record<string, unknown>)).toBe(12);
    const issues = validateTeam([wayOver], stressRegistry, "teamA");
    expect(issues.some((i) => /complexity 12 exceeds the card budget/.test(i.message))).toBe(true);
    expect(() => assertValidContent([wayOver], stressRegistry, "teamA")).toThrow(ValidationError);
  });

  test("an over-cap StatusDef is rejected at registry validation", () => {
    const fatRegistry = {
      ...stressRegistry,
      Overloaded: {
        name: "Overloaded",
        abilities: [justOver.abilities![0]!],
      },
    };
    const issues = validateRegistry(fatRegistry as never);
    expect(issues.some((i) => i.path === "registry.Overloaded" && /complexity 9 exceeds/.test(i.message))).toBe(true);
  });

  // Must-fail-first: the over-cap unit is otherwise content-valid — every part
  // is a real, correctly-wired atom. Before the cap predicate existed, all the
  // shape/context/reference checks pass it (no issues), so ONLY the complexity
  // check rejects it. We prove that by stripping the complexity issues and
  // confirming nothing else complained.
  test("must-fail-first: the over-cap unit is rejected by the cap alone, not pre-existing checks", () => {
    const issues = validateTeam([justOver], stressRegistry, "teamA");
    const nonComplexity = issues.filter((i) => !/exceeds the card budget/.test(i.message));
    expect(nonComplexity).toEqual([]); // would have been the entire result pre-predicate → no failure
  });
});
