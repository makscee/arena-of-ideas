// Codex acceptance tests
// (a) Every registry status and every meetable unit appears in the codex data.
// (b) Each named rule section is present.
// (c) Numbers in codex output match the code that runs the game — formulas are
//     asserted by CALLING them (fatigueAmount, incomeForRound), so retuning a
//     knob cannot silently leave the codex lying.
// (d) The ghosts rule is asserted against the actual ladder semantics
//     (one random draw per round; pre-seeded champion) — the codex must never
//     re-grow the "beat every ghost" / "vacant spot" misreadings.

import { describe, it, expect } from "vitest";
import { buildCodex, codexUnits } from "./codex.js";
import { stressRegistry } from "./content/stress.js";
import { stressAbilities } from "./content/stress.js";
import {
  TOWER_HEIGHT,
  DEFAULT_RUN_POOL,
  INCOME_PER_ROUND,
  INCOME_CAP,
  LEVEL_HP_GROWTH,
  LEVEL_PWR_GROWTH,
  REROLL_COST,
  SHOP_SIZE_BASE,
  SHOP_SIZE_MAX,
  SHOP_SIZE_STEP,
  STACK_THRESHOLD,
  STARTING_LIVES,
  UNIT_COST,
  incomeForRound,
  FAMILY_HEX,
  familyHex,
} from "./tunables.js";
import { FATIGUE_RAMP, FATIGUE_START, TURN_CAP, fatigueAmount } from "./battle.js";

const codex = buildCodex(stressRegistry, codexUnits([], stressAbilities), stressAbilities);
const rule = (key: string) => {
  const r = codex.rules.find((r) => r.key === key);
  expect(r, `rule "${key}" missing`).toBeDefined();
  return r!;
};

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
  const codexNames = codex.units.map((u) => u.name);

  it("contains every unit from DEFAULT_RUN_POOL", () => {
    for (const u of DEFAULT_RUN_POOL) expect(codexNames, `${u.name} missing`).toContain(u.name);
  });

  it("contains the units a player faces but cannot buy (bootstrap + summons)", () => {
    // Warden/Warlord live only in bootstrap ghosts/champion; Imp only as a summon.
    for (const name of ["Warden", "Warlord", "Imp"]) {
      expect(codexNames, `${name} missing`).toContain(name);
    }
  });

  it("dedup favors the buyable variant (shop pool first)", () => {
    // Brawler is 12/2 in the shop pool and 14/3, 16/4 as bootstrap ghosts —
    // the codex must show what the shop sells.
    const brawler = codex.units.find((u) => u.name === "Brawler")!;
    const poolBrawler = DEFAULT_RUN_POOL.find((u) => u.name === "Brawler")!;
    expect(brawler.hp).toBe(poolBrawler.base.hp);
    expect(brawler.pwr).toBe(poolBrawler.base.pwr);
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

  it("Warlord entry carries its starting Strength stacks", () => {
    const w = codex.units.find((u) => u.name === "Warlord")!;
    expect(w.statuses.join(" ")).toMatch(/Strength ×\d/);
  });
});

// ---------------------------------------------------------------------------
// The Ability catalogue + colour-from-family derive (PRD #081 slice 4)
// ---------------------------------------------------------------------------

describe("codex — ability catalogue (#081)", () => {
  it("lists every shipped ability with its family and the palette hex", () => {
    const byName = new Map(codex.abilities.map((a) => [a.name, a]));
    // Every entry in the shipped registry has a catalogue card.
    for (const [id, def] of Object.entries(stressAbilities)) {
      const entry = byName.get(id);
      expect(entry, `ability ${id} catalogued`).toBeDefined();
      expect(entry!.family).toBe(def.family);
      // The hex is derived from the one palette, never re-typed.
      expect(entry!.hex).toBe(FAMILY_HEX[def.family]);
    }
    expect(codex.abilities.length).toBe(Object.keys(stressAbilities).length);
  });

  it("every catalogued ability's family hex matches the pinned palette", () => {
    expect(FAMILY_HEX).toEqual({
      Poison: "#a06bff",
      Strike: "#ff7a4d",
      Shield: "#4d9bff",
      Summon: "#25e6d4",
      Arcane: "#e056fd",
      Control: "#6b8cff",
      Heal: "#33d98a",
    });
  });

  it("derives a unit's colour from its ability's family — unit → ability → family → hex", () => {
    // Venomancer references Venom (Poison family) → #a06bff.
    const venom = codex.units.find((u) => u.name === "Venomancer")!;
    expect(venom.ability).toBe("Venom");
    expect(venom.family).toBe("Poison");
    expect(venom.hex).toBe(familyHex("Poison"));
    expect(venom.hex).toBe("#a06bff");
    // A plain body references the inert Strike ability → Strike family → #ff7a4d.
    const brawler = codex.units.find((u) => u.name === "Brawler")!;
    expect(brawler.ability).toBe("Strike");
    expect(brawler.family).toBe("Strike");
    expect(brawler.hex).toBe("#ff7a4d");
  });

  it("every shipped unit derives a family and hex (no colourless unit)", () => {
    for (const u of codex.units) {
      expect(u.family, `${u.name} family`).toBeDefined();
      expect(u.hex, `${u.name} hex`).toBe(familyHex(u.family!));
    }
  });
});

describe("codex — rule sections", () => {
  for (const key of ["fatigue", "income", "lives", "fusion", "shop", "strike-order", "ghosts", "draws"]) {
    it(`rule "${key}" is present`, () => {
      rule(key);
    });
  }

  it("each rule has a non-empty title and text", () => {
    for (const r of codex.rules) {
      expect(r.title.length, `${r.key} title empty`).toBeGreaterThan(0);
      expect(r.text.length, `${r.key} text empty`).toBeGreaterThan(0);
    }
  });
});

describe("codex — fatigue derives from the kernel's formula", () => {
  it("cites FATIGUE_START and the first three ramp values from fatigueAmount()", () => {
    const r = rule("fatigue");
    expect(r.text).toContain(`From turn ${FATIGUE_START}`);
    // The sequence is asserted as one string so an additive re-write of a
    // multiplicative ramp (e.g. RAMP=2 → "2, 3, 4" instead of "2, 4, 6") fails.
    const seq = `${fatigueAmount(FATIGUE_START)}, ${fatigueAmount(FATIGUE_START + 1)}, ${fatigueAmount(FATIGUE_START + 2)}`;
    expect(r.text).toContain(seq);
  });

  it("cites TURN_CAP", () => {
    expect(rule("fatigue").text).toContain(String(TURN_CAP));
  });

  it("growth phrasing derives from FATIGUE_RAMP — 'without limit' only while the ramp ramps", () => {
    const text = rule("fatigue").text;
    if (FATIGUE_RAMP > 0) {
      expect(text).toContain("grows every turn");
    } else {
      expect(text).toContain("holds steady");
      expect(text).not.toContain("without limit");
    }
  });
});

describe("codex — income derives from incomeForRound()", () => {
  it("cites round-1 income from the curve, not a raw constant", () => {
    expect(rule("income").text).toContain(`${incomeForRound(1)} gold`);
  });

  it("matches the curve's shape: flat text iff INCOME_PER_ROUND is 0", () => {
    const text = rule("income").text;
    if (INCOME_PER_ROUND === 0) {
      expect(text).toContain("each new round");
      expect(text).not.toContain("grows");
    } else {
      expect(text).toContain(`grows by ${INCOME_PER_ROUND}`);
      expect(text).toContain(String(INCOME_CAP));
    }
  });
});

describe("codex — lives / fusion / shop tunables", () => {
  it("lives rule cites STARTING_LIVES", () => {
    expect(rule("lives").text).toContain(`${STARTING_LIVES} lives`);
  });

  it("fusion rule cites STACK_THRESHOLD and the level growth", () => {
    const text = rule("fusion").text;
    expect(text).toContain(`${STACK_THRESHOLD} copies`);
    expect(text).toContain(`+${LEVEL_HP_GROWTH} base hp`);
    expect(text).toContain(`+${LEVEL_PWR_GROWTH} base pwr`);
    // Levels continue: the count resets, THRESHOLD−1 more copies fuse again.
    expect(text).toContain(`${STACK_THRESHOLD - 1} more copies`);
  });

  it("fusion rule cites UNIT_COST and REROLL_COST", () => {
    const text = rule("fusion").text;
    expect(text).toContain(`${UNIT_COST}g`);
    expect(text).toContain(`${REROLL_COST}g`);
  });

  it("shop rule cites SHOP_SIZE_BASE/STEP/MAX", () => {
    const text = rule("shop").text;
    expect(text).toContain(`${SHOP_SIZE_BASE} offers`);
    expect(text).toContain(`every ${SHOP_SIZE_STEP} rounds`);
    expect(text).toContain(`${SHOP_SIZE_MAX} offers`);
  });
});

describe("codex — ghosts rule matches ladder semantics", () => {
  it("states the one-random-draw rule, not pool-clearing", () => {
    const text = rule("ghosts").text;
    expect(text).toContain("one ghost drawn at random");
    // The misreadings the first version shipped with — pinned dead:
    expect(text.toLowerCase()).not.toMatch(/beat every ghost/);
    expect(text.toLowerCase()).not.toMatch(/every ghost in your round/);
  });

  it("states the pre-seeded boss-per-floor tower, never a fresh-ladder free crown", () => {
    const text = rule("ghosts").text;
    expect(text).toContain("the top floor's boss being the champion");
    // The fixed tower's guard is the overshoot rule, stated plainly:
    expect(text.toLowerCase()).toMatch(/overshoot/);
    expect(text.toLowerCase()).toMatch(/no boss, no crown/);
  });

  it("cites TOWER_HEIGHT for the fixed tower's floor count", () => {
    expect(rule("ghosts").text).toContain(`fixed ${TOWER_HEIGHT}-floor tower`);
  });

  it("states that the floor advances win or lose, and the boss challenge is terminal", () => {
    const text = rule("ghosts").text;
    expect(text).toContain("win or lose");
    expect(text).toContain("terminal move");
  });
});
