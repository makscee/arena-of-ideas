// Validator tests — every rejection here corresponds to content the kernel
// would otherwise accept and silently never fire.

import { describe, expect, test } from "vitest";
import { ValidationError, assertValidContent, validateRegistry, validateTeam } from "./validate.js";
import { Necromancer, Silencer, Summoner, Venomancer, stressRegistry } from "./content/stress.js";
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
