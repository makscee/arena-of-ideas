// Codex acceptance tests
// (a) Every registry status and every shipped unit appears in the codex data.
// (b) Each named rule section is present.
// (c) Numbers in codex output match tunables (change-detector against drift).

import { describe, it, expect } from "vitest";
import { buildCodex } from "./codex.js";
import { stressRegistry } from "./content/stress.js";
import { DEFAULT_RUN_POOL } from "./tunables.js";
import {
  INCOME_BASE,
  STARTING_LIVES,
  STACK_THRESHOLD,
  SHOP_SIZE_BASE,
  SHOP_SIZE_MAX,
  SHOP_SIZE_STEP,
  UNIT_COST,
  REROLL_COST,
} from "./tunables.js";
import { FATIGUE_START, FATIGUE_RAMP, TURN_CAP } from "./battle.js";

const codex = buildCodex(stressRegistry, DEFAULT_RUN_POOL);

describe("codex — status coverage", () => {
  const registryNames = Object.keys(stressRegistry).sort();
  const codexNames = codex.statuses.map((s) => s.name).sort();

  it("contains every registry status", () => {
    expect(codexNames).toEqual(registryNames);
  });

  it("each status has a non-empty description", () => {
    for (const s of codex.statuses) {
      expect(s.description.length, `${s.name} description empty`).toBeGreaterThan(0);
    }
  });

  it("Poison entry is present and describes turn-end damage", () => {
    const poison = codex.statuses.find((s) => s.name === "Poison");
    expect(poison).toBeDefined();
    // describeStatus derives this from the ability — "end of each turn" + "damage"
    expect(poison!.description.toLowerCase()).toMatch(/turn/);
    expect(poison!.description.toLowerCase()).toMatch(/damage/);
  });
});

describe("codex — unit coverage", () => {
  const poolNames = [...new Set(DEFAULT_RUN_POOL.map((u) => u.name))].sort();
  const codexNames = codex.units.map((u) => u.name).sort();

  it("contains every unit from DEFAULT_RUN_POOL", () => {
    expect(codexNames).toEqual(poolNames);
  });

  it("each unit entry carries hp and pwr", () => {
    for (const u of codex.units) {
      expect(u.hp, `${u.name} hp`).toBeGreaterThan(0);
      expect(u.pwr, `${u.name} pwr`).toBeGreaterThanOrEqual(0);
    }
  });

  it("Venomancer entry describes its Poison application", () => {
    const v = codex.units.find((u) => u.name === "Venomancer");
    expect(v).toBeDefined();
    expect(v!.abilities.length).toBeGreaterThan(0);
    // describeAbility produces "apply 2 Poison to the front enemy"
    const text = v!.abilities.join(" ").toLowerCase();
    expect(text).toMatch(/poison/);
  });
});

describe("codex — rule sections", () => {
  const ruleKeys = codex.rules.map((r) => r.key);

  for (const key of ["fatigue", "income", "lives", "fusion", "shop", "strike-order", "ghosts", "draws"]) {
    it(`rule "${key}" is present`, () => {
      expect(ruleKeys).toContain(key);
    });
  }

  it("each rule has a non-empty title and text", () => {
    for (const r of codex.rules) {
      expect(r.title.length, `${r.key} title empty`).toBeGreaterThan(0);
      expect(r.text.length, `${r.key} text empty`).toBeGreaterThan(0);
    }
  });
});

describe("codex — tunable/constant drift detection", () => {
  it("fatigue rule cites FATIGUE_START", () => {
    const rule = codex.rules.find((r) => r.key === "fatigue")!;
    expect(rule.text).toContain(String(FATIGUE_START));
  });

  it("fatigue rule cites FATIGUE_RAMP", () => {
    const rule = codex.rules.find((r) => r.key === "fatigue")!;
    expect(rule.text).toContain(String(FATIGUE_RAMP));
  });

  it("fatigue rule cites TURN_CAP", () => {
    const rule = codex.rules.find((r) => r.key === "fatigue")!;
    expect(rule.text).toContain(String(TURN_CAP));
  });

  it("income rule cites INCOME_BASE", () => {
    const rule = codex.rules.find((r) => r.key === "income")!;
    expect(rule.text).toContain(String(INCOME_BASE));
  });

  it("lives rule cites STARTING_LIVES", () => {
    const rule = codex.rules.find((r) => r.key === "lives")!;
    expect(rule.text).toContain(String(STARTING_LIVES));
  });

  it("fusion rule cites STACK_THRESHOLD", () => {
    const rule = codex.rules.find((r) => r.key === "fusion")!;
    expect(rule.text).toContain(String(STACK_THRESHOLD));
  });

  it("fusion rule cites UNIT_COST", () => {
    const rule = codex.rules.find((r) => r.key === "fusion")!;
    expect(rule.text).toContain(String(UNIT_COST));
  });

  it("fusion rule cites REROLL_COST", () => {
    const rule = codex.rules.find((r) => r.key === "fusion")!;
    expect(rule.text).toContain(String(REROLL_COST));
  });

  it("shop rule cites SHOP_SIZE_BASE", () => {
    const rule = codex.rules.find((r) => r.key === "shop")!;
    expect(rule.text).toContain(String(SHOP_SIZE_BASE));
  });

  it("shop rule cites SHOP_SIZE_MAX", () => {
    const rule = codex.rules.find((r) => r.key === "shop")!;
    expect(rule.text).toContain(String(SHOP_SIZE_MAX));
  });

  it("shop rule cites SHOP_SIZE_STEP", () => {
    const rule = codex.rules.find((r) => r.key === "shop")!;
    expect(rule.text).toContain(String(SHOP_SIZE_STEP));
  });
});
