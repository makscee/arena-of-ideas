import { describe, expect, test } from "vitest";
import { battle, winnerOf } from "./battle.js";
import { Necromancer, Silencer, stressRegistry, Summoner, Venomancer } from "./content/stress.js";
import {
  applyDecision,
  buy,
  fight,
  initRun,
  InvalidDecisionError,
  playRun,
  reorder,
  reroll,
  runToJSONL,
  toBattleTeam,
} from "./run.js";
import type { RunDecision, RunEvent, RunInput, RunState, RunUnit } from "./run.js";
import {
  incomeForRound,
  LEVEL_HP_GROWTH,
  LEVEL_PWR_GROWTH,
  REROLL_COST,
  SHOP_SIZE_MAX,
  shopSizeForRound,
  STACK_THRESHOLD,
  STARTING_GOLD,
  STARTING_LIVES,
  UNIT_COST,
} from "./tunables.js";
import { ValidationError, validateTeam } from "./validate.js";
import type { UnitDef } from "./types.js";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const ofType = <T extends RunEvent["type"]>(
  log: RunEvent[],
  t: T,
): Extract<RunEvent, { type: T }>[] =>
  log.filter((e): e is Extract<RunEvent, { type: T }> => e.type === t);

function vanilla(name: string, hp: number, pwr: number): UnitDef {
  return { name, base: { hp, pwr } };
}

const POOL: UnitDef[] = [Summoner, Silencer, Necromancer, Venomancer];
const INPUT: RunInput = { seed: 7, pool: POOL, statuses: stressRegistry };

// Loses to any striker regardless of the coin (1 hp, can't strike back for damage).
const PUSHOVER: UnitDef[] = [vanilla("Pushover", 1, 0)];
// Kills any pool unit in one strike and outlasts everything — a guaranteed loss.
const WALL: UnitDef[] = [vanilla("Wall", 100, 99)];

// ---------------------------------------------------------------------------
// 1. Determinism: the run log is the determinism artifact
// ---------------------------------------------------------------------------

describe("determinism", () => {
  test("byte-identical run-log JSONL for the same seed + decision sequence", () => {
    const decisions: RunDecision[] = [
      { kind: "buy", offer: 0 },
      { kind: "reroll" },
      { kind: "buy", offer: 1 },
      { kind: "reorder", from: 0, to: 0 },
      { kind: "fight", opponent: [vanilla("Brute", 8, 3)] },
      { kind: "buy", offer: 0 },
      { kind: "fight", opponent: WALL },
    ];
    const run1 = playRun(INPUT, decisions);
    const run2 = playRun(INPUT, decisions);
    const jsonl1 = runToJSONL(run1.log);
    expect(jsonl1).toBe(runToJSONL(run2.log));
    // The sequence actually exercised the economy: both fights are in the artifact.
    expect(ofType(run1.log, "FightFought").length).toBe(2);
  });

  test("a transition never mutates its input state", () => {
    const s0 = applyDecision(initRun(INPUT), { kind: "buy", offer: 0 });
    const snapshot = JSON.stringify(s0);
    buy(s0, 0);
    reroll(s0);
    reorder(s0, 0, 0);
    fight(s0, PUSHOVER);
    expect(JSON.stringify(s0)).toBe(snapshot);
  });
});

// ---------------------------------------------------------------------------
// 2. Gold economy: starting gold, costs, carryover + income
// ---------------------------------------------------------------------------

describe("gold economy", () => {
  test("a run opens with starting gold, lives, and a rolled round-1 shop", () => {
    const s = initRun(INPUT);
    expect(s.round).toBe(1);
    expect(s.gold).toBe(STARTING_GOLD);
    expect(s.lives).toBe(STARTING_LIVES);
    expect(s.team).toEqual([]);
    expect(s.offers.length).toBe(shopSizeForRound(1));
    expect(s.log[0]).toMatchObject({ type: "RunStart", seed: 7, gold: STARTING_GOLD, lives: STARTING_LIVES });
    expect(s.log[1]).toMatchObject({ type: "ShopRolled" });
  });

  test("buying costs UNIT_COST; rerolling costs REROLL_COST", () => {
    let s = initRun(INPUT);
    s = buy(s, 0);
    expect(s.gold).toBe(STARTING_GOLD - UNIT_COST);
    s = reroll(s);
    expect(s.gold).toBe(STARTING_GOLD - UNIT_COST - REROLL_COST);
  });

  test("gold carries over and income lands when the fight turns the round", () => {
    let s = initRun(INPUT);
    s = buy(s, 0); // leave a carryover smaller than the income
    const carry = s.gold;
    s = fight(s, PUSHOVER);
    expect(s.round).toBe(2);
    expect(s.gold).toBe(carry + incomeForRound(2));
    expect(ofType(s.log, "RoundStarted")[0]).toMatchObject({ round: 2, income: incomeForRound(2), gold: s.gold });
  });
});

// ---------------------------------------------------------------------------
// 3. Shop offers: pool-drawn, 3–6 by round, reroll/buy mechanics
// ---------------------------------------------------------------------------

describe("shop offers", () => {
  test("offers are drawn from the pool, 3–6 per round, growing to the cap", () => {
    const names = new Set(POOL.map((u) => u.name));
    let s = initRun(INPUT);
    for (let i = 0; i < 12; i++) {
      // As rolled, before any purchase shrinks it.
      expect(s.offers.length).toBe(shopSizeForRound(s.round));
      expect(s.offers.length).toBeGreaterThanOrEqual(3);
      expect(s.offers.length).toBeLessThanOrEqual(6);
      for (const o of s.offers) expect(names.has(o.name)).toBe(true);
      if (s.team.length === 0) s = buy(s, 0); // fight needs a line
      s = fight(s, PUSHOVER);
    }
    expect(s.offers.length).toBe(SHOP_SIZE_MAX); // the size curve actually reaches its cap
  });

  test("reroll replaces the offers with a fresh seeded draw", () => {
    const s0 = initRun(INPUT);
    const s1 = reroll(s0);
    expect(s1.offers.length).toBe(shopSizeForRound(1));
    expect(s1.rng).not.toBe(s0.rng); // the draw advanced the run's one stream
    expect(ofType(s1.log, "ShopRolled").length).toBe(2);
  });

  test("buying removes the bought offer from the shop", () => {
    const s0 = initRun(INPUT);
    const s1 = buy(s0, 0);
    expect(s1.offers.length).toBe(s0.offers.length - 1);
    expect(s1.offers).toEqual(s0.offers.slice(1));
  });
});

// ---------------------------------------------------------------------------
// 4. Duplicate stacking → level-up with stat growth
// ---------------------------------------------------------------------------

describe("duplicate stacking and level-up", () => {
  // A single-unit pool makes every offer a copy — the stacking path is forced.
  const soloInput: RunInput = { seed: 3, pool: [vanilla("Grunt", 5, 2)] };

  test("a first copy joins the line; a second stacks instead of taking a slot", () => {
    let s = buy(initRun(soloInput), 0);
    expect(s.team.length).toBe(1);
    expect(s.team[0]).toMatchObject({ name: "Grunt", level: 1, stacks: 1 });
    s = buy(s, 0);
    expect(s.team.length).toBe(1);
    expect(s.team[0]).toMatchObject({ stacks: 2 });
  });

  test("at STACK_THRESHOLD copies the unit levels up and its base grows", () => {
    let s = initRun(soloInput);
    for (let i = 0; i < STACK_THRESHOLD; i++) s = buy(s, 0);
    expect(s.team[0]).toMatchObject({ name: "Grunt", level: 2, stacks: 1 });
    expect(s.team[0]!.base).toEqual({ hp: 5 + LEVEL_HP_GROWTH, pwr: 2 + LEVEL_PWR_GROWTH });
    expect(ofType(s.log, "LeveledUp")[0]).toMatchObject({
      unit: "Grunt",
      level: 2,
      hp: 5 + LEVEL_HP_GROWTH,
      pwr: 2 + LEVEL_PWR_GROWTH,
    });
    // Growth lives on the run unit — the flat content def is untouched.
    expect(soloInput.pool[0]!.base).toEqual({ hp: 5, pwr: 2 });
  });

  test("the grown unit fights with its grown base and level", () => {
    let s = initRun(soloInput);
    for (let i = 0; i < STACK_THRESHOLD; i++) s = buy(s, 0);
    expect(toBattleTeam(s.team)[0]).toMatchObject({
      name: "Grunt",
      level: 2,
      base: { hp: 5 + LEVEL_HP_GROWTH, pwr: 2 + LEVEL_PWR_GROWTH },
    });
  });
});

// ---------------------------------------------------------------------------
// 5. level in DSL magnitude expressions (the way pwr references work)
// ---------------------------------------------------------------------------

describe("level in magnitude expressions", () => {
  const veteran: UnitDef = {
    name: "Veteran",
    base: { hp: 20, pwr: 0 },
    level: 3,
    abilities: [
      {
        whens: [{ kind: "trigger", on: { on: "TurnStart" } }],
        selectors: [{ kind: "frontEnemy" }],
        effects: [{ kind: "damage", amount: { kind: "level", of: "holder" } }],
      },
    ],
  };

  test("a level amount evaluates to the holder's level", () => {
    const log = battle({ teamA: [veteran], teamB: [vanilla("Dummy", 30, 0)], seed: 1 });
    const abilityHurt = log.find((e) => e.type === "Hurt" && e.source !== "kernel");
    expect(abilityHurt).toMatchObject({ amount: 3 });
  });

  test("the validator accepts a level amount and pins it to the holder", () => {
    expect(validateTeam([veteran], {})).toEqual([]);
    const bad = JSON.parse(JSON.stringify(veteran)) as Record<string, unknown>;
    (bad as { abilities: { effects: { amount: { of: string } }[] }[] }).abilities[0]!.effects[0]!.amount.of = "target";
    const issues = validateTeam([bad], {});
    expect(issues.length).toBe(1);
    expect(issues[0]!.message).toMatch(/level amounts read the holder/);
  });
});

// ---------------------------------------------------------------------------
// 6. fight: delegates to battle(), records the outcome, lives on loss
// ---------------------------------------------------------------------------

describe("fight", () => {
  test("delegates to battle() — the outcome is reproducible from the logged seed", () => {
    const s = fight(buy(initRun(INPUT), 0), PUSHOVER);
    const f = ofType(s.log, "FightFought")[0]!;
    expect(f.winner).toBe("A");
    // The team is unchanged by the fight, so the logged seed replays the same battle.
    const replay = battle({ teamA: toBattleTeam(s.team), teamB: PUSHOVER, seed: f.battleSeed, statuses: stressRegistry });
    expect(winnerOf(replay)).toBe(f.winner);
  });

  test("a loss decrements lives; a win does not", () => {
    const s = buy(initRun(INPUT), 0);
    const won = fight(s, PUSHOVER);
    expect(won.lives).toBe(STARTING_LIVES);
    const lost = fight(s, WALL);
    expect(lost.lives).toBe(STARTING_LIVES - 1);
    expect(ofType(lost.log, "FightFought")[0]).toMatchObject({ winner: "B", lives: STARTING_LIVES - 1 });
  });

  test("a draw costs no life", () => {
    // Two 0-pwr 30-hp units fatigue out together — the battle suite's draw setup.
    let s = initRun({ seed: 5, pool: [vanilla("Pacifist", 30, 0)] });
    s = fight(buy(s, 0), [vanilla("Dummy", 30, 0)]);
    expect(ofType(s.log, "FightFought")[0]).toMatchObject({ winner: "draw", lives: STARTING_LIVES });
  });
});

// ---------------------------------------------------------------------------
// 7. reorder: line positions
// ---------------------------------------------------------------------------

describe("reorder", () => {
  test("moves a unit to a new line position", () => {
    let s = initRun(INPUT);
    while (new Set(s.offers.map((o) => o.name)).size < 2) s = reroll(s);
    s = buy(s, 0);
    s = buy(s, s.offers.findIndex((o) => o.name !== s.team[0]!.name));
    expect(s.team.length).toBe(2);
    const [front, back] = s.team.map((u) => u.name);
    s = reorder(s, 1, 0);
    expect(s.team.map((u) => u.name)).toEqual([back, front]);
    expect(ofType(s.log, "Reordered")[0]).toMatchObject({ from: 1, to: 0 });
  });
});

// ---------------------------------------------------------------------------
// 8. Invalid decisions are rejected loudly, never silently ignored
// ---------------------------------------------------------------------------

describe("invalid decisions", () => {
  /** A hand-built line unit for edge states transitions can't cheaply reach. */
  function lineUnit(name: string): RunUnit {
    return { name, base: { hp: 5, pwr: 1 }, level: 1, stacks: 1, def: vanilla(name, 5, 1) };
  }

  test("buying broke, and rerolling broke, both throw", () => {
    let s = initRun(INPUT);
    for (let i = 0; i < STARTING_GOLD / REROLL_COST; i++) s = reroll(s);
    expect(s.gold).toBe(0);
    expect(() => buy(s, 0)).toThrow(InvalidDecisionError);
    expect(() => buy(s, 0)).toThrow(/gold/);
    expect(() => reroll(s)).toThrow(InvalidDecisionError);
    expect(() => reroll(s)).toThrow(/reroll costs/);
  });

  test("buying a missing offer index throws", () => {
    const s = initRun(INPUT);
    expect(() => buy(s, 99)).toThrow(InvalidDecisionError);
    expect(() => buy(s, -1)).toThrow(/no offer at index/);
  });

  test("buying to a full line without a copy to stack onto throws; with one it stacks", () => {
    const base = initRun({ seed: 1, pool: [vanilla("Other", 5, 1)] }); // every offer is "Other"
    const full: RunState = { ...base, team: ["A", "B", "C", "D", "E"].map(lineUnit) };
    expect(() => buy(full, 0)).toThrow(InvalidDecisionError);
    expect(() => buy(full, 0)).toThrow(/line is full/);
    const withCopy: RunState = { ...full, team: [...full.team.slice(0, 4), lineUnit("Other")] };
    const after = buy(withCopy, 0);
    expect(after.team.length).toBe(5);
    expect(after.team[4]).toMatchObject({ name: "Other", stacks: 2 });
  });

  test("reordering outside the line throws", () => {
    const s = buy(initRun(INPUT), 0);
    expect(() => reorder(s, 0, 1)).toThrow(InvalidDecisionError);
    expect(() => reorder(s, -1, 0)).toThrow(/outside the line/);
  });

  test("fighting with an empty line throws", () => {
    const s = initRun(INPUT);
    expect(() => fight(s, PUSHOVER)).toThrow(InvalidDecisionError);
    expect(() => fight(s, PUSHOVER)).toThrow(/line is empty/);
  });

  test("an invalid opponent is rejected by the content gate", () => {
    const s = buy(initRun(INPUT), 0);
    const bad = [{ name: "Bad", base: { hp: -1, pwr: 1 } }] as UnitDef[];
    expect(() => fight(s, bad)).toThrow(ValidationError);
  });
});
