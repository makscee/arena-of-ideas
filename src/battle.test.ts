import { describe, expect, test } from "vitest";
import { battle, fatigueAmount, toJSONL } from "./battle.js";
import type { AbilityRegistry, BattleEvent, BattleInput, UnitDef } from "./types.js";
import {
  Blessing,
  Curse,
  Freeze,
  Necromancer,
  Poison,
  Shield,
  Silencer,
  Strength,
  stressRegistry,
  Summoner,
  Venomancer,
  Vitality,
} from "./content/stress.js";
import { stressAbilities } from "./content/stress.js";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const ofType = <T extends BattleEvent["type"]>(
  log: BattleEvent[],
  t: T,
): Extract<BattleEvent, { type: T }>[] =>
  log.filter((e): e is Extract<BattleEvent, { type: T }> => e.type === t);

/** Build a simple vanilla UnitDef with no abilities and no statuses. */
function vanilla(name: string, hp: number, pwr: number): UnitDef {
  return { name, base: { hp, pwr }, ability: "Strike" };
}

function runBattle(input: BattleInput): BattleEvent[] {
  // Vanilla fixtures reference the inert "Strike" ability (PRD #081); the
  // registry must be present for the ref to resolve. Inputs that pass their own
  // `abilities` (e.g. a local merged registry) override this default.
  return battle({ abilities: stressAbilities, ...input });
}

// ---------------------------------------------------------------------------
// 1. Determinism
// ---------------------------------------------------------------------------

describe("determinism", () => {
  test("byte-identical JSONL for identical inputs with stress content", () => {
    const teamA: UnitDef[] = [
      Venomancer,
      {
        name: "Summoner",
        base: Summoner.base,
        triggers: Summoner.triggers!,
        selectors: Summoner.selectors!,
        abilities: Summoner.abilities!,
      },
    ];
    const teamB: UnitDef[] = [
      Necromancer,
      {
        name: "ShieldPoison",
        base: { hp: 10, pwr: 1 },
        ability: "Strike",
        statuses: [
          { status: "Shield", stacks: 2 },
          { status: "Poison", stacks: 1 },
        ],
      },
    ];
    const input: BattleInput = { teamA, teamB, seed: 42, statuses: stressRegistry, abilities: stressAbilities };

    const log1 = toJSONL(battle(input));
    const log2 = toJSONL(battle(input));
    expect(log1).toBe(log2);
  });

  test("coin actually flips: PairFaced.first takes both values across seeds 1..10 for vanilla 1v1", () => {
    const teamA: UnitDef[] = [vanilla("Knight", 10, 2)];
    const teamB: UnitDef[] = [vanilla("Brute", 8, 3)];
    const firstSet = new Set<string>();
    for (let seed = 1; seed <= 10; seed++) {
      const log = runBattle({ teamA, teamB, seed });
      const pf = ofType(log, "PairFaced")[0];
      expect(pf).toBeDefined();
      firstSet.add(pf!.first);
    }
    // Both units must appear as first-striker across 10 seeds.
    expect(firstSet.size).toBe(2);
  });
});

// ---------------------------------------------------------------------------
// 2. Basic combat: Knight 10hp/2pwr vs Brute 8hp/3pwr
// ---------------------------------------------------------------------------

describe("basic combat", () => {
  test("first-striker side wins", () => {
    // Knight needs ceil(8/2)=4 hits to kill Brute; Brute needs ceil(10/3)=4 hits to kill Knight.
    // With alternating strikes the first-striker gets hits 1,3,5,... and kills on hit 4 (turn 4).
    // Second-striker gets hits 2,4,... but on turn 4 the first-striker kills on hit 4 before
    // second-striker gets another hit. So first-striker wins.
    const teamA: UnitDef[] = [vanilla("Knight", 10, 2)];
    const teamB: UnitDef[] = [vanilla("Brute", 8, 3)];

    // Try multiple seeds to find a decisive one — the winner depends on who goes first.
    // We assert winner === the side whose front unit id matches PairFaced.first.
    for (let seed = 1; seed <= 10; seed++) {
      const log = runBattle({ teamA, teamB, seed });
      const pf = ofType(log, "PairFaced")[0];
      const end = ofType(log, "BattleEnd")[0];
      expect(pf).toBeDefined();
      expect(end).toBeDefined();
      expect(end!.winner).not.toBe("draw");
      // PairFaced.first is either the A-front or B-front unit id.
      // The side whose unit appears in PairFaced.first should win.
      const firstId = pf!.first;
      // Determine which side firstId belongs to: it was created as "A1:Knight" or "B1:Brute".
      const winnerSide = firstId.startsWith("A") ? "A" : "B";
      expect(end!.winner).toBe(winnerSide);
    }
  });
});

// ---------------------------------------------------------------------------
// 3. Alternation: turn 1 has exactly 2 Strike events with different strikers
// ---------------------------------------------------------------------------

describe("alternation", () => {
  test("turn 1 has exactly 2 strikes with different strikers, first matching PairFaced.first", () => {
    const teamA: UnitDef[] = [vanilla("Knight", 10, 2)];
    const teamB: UnitDef[] = [vanilla("Brute", 10, 2)];
    const log = runBattle({ teamA, teamB, seed: 1 });

    const turn1Strikes = ofType(log, "Strike").filter((e) => e.turn === 1);
    expect(turn1Strikes.length).toBe(2);

    const s0 = turn1Strikes[0]!;
    const s1 = turn1Strikes[1]!;
    expect(s0.striker).not.toBe(s1.striker);

    const pf = ofType(log, "PairFaced")[0];
    expect(pf).toBeDefined();
    expect(s0.striker).toBe(pf!.first);
  });
});

// ---------------------------------------------------------------------------
// 2b. hpAfter: every hp-changing event carries the unit's hp after application
// ---------------------------------------------------------------------------

describe("hpAfter", () => {
  test("every hp-changing event (Hurt/Heal/hp StatChanged) carries hpAfter consistent with the hp deltas", () => {
    // Stress content exercises Heals (Blessing), absorbed Hurts (Shield), and StatChanged (Vitality).
    const teamA: UnitDef[] = [
      { name: "Blessed", base: { hp: 8, pwr: 2 }, ability: "Strike", statuses: [{ status: "Blessing", stacks: 3 }] },
      Venomancer,
    ];
    const teamB: UnitDef[] = [
      { name: "Shielded", base: { hp: 10, pwr: 2 }, ability: "Strike", statuses: [{ status: "Shield", stacks: 4 }] },
      { name: "Vital", base: { hp: 6, pwr: 1 }, ability: "Strike", statuses: [{ status: "Vitality", stacks: 2 }] },
    ];
    const log = runBattle({ teamA, teamB, seed: 7, statuses: stressRegistry, abilities: stressAbilities });

    // Re-derive current hp per unit (effective max via StatChanged, deltas via Hurt/Heal)
    // and check each event's hpAfter against it.
    const maxHp = new Map<string, number>();
    const damage = new Map<string, number>();
    for (const r of [...ofType(log, "BattleStart")[0]!.teams.A, ...ofType(log, "BattleStart")[0]!.teams.B]) {
      maxHp.set(r.id, r.hp);
      damage.set(r.id, 0);
    }
    let checked = 0;
    let hpStatChanges = 0;
    for (const e of log) {
      if (e.type === "StatChanged") {
        if (e.stat === "hp") {
          // An hp statMod moves current hp with the max — the kernel stamps the result.
          maxHp.set(e.unit, e.now);
          expect(e.hpAfter).toBe(maxHp.get(e.unit)! - damage.get(e.unit)!);
          hpStatChanges++;
        } else {
          expect(e.hpAfter).toBeUndefined(); // a pwr change moves no hp
        }
      }
      if (e.type === "Hurt") {
        damage.set(e.unit, (damage.get(e.unit) ?? 0) + e.amount);
        expect(e.hpAfter).toBe(maxHp.get(e.unit)! - damage.get(e.unit)!);
        checked++;
      }
      if (e.type === "Heal") {
        damage.set(e.unit, Math.max(0, (damage.get(e.unit) ?? 0) - e.amount));
        expect(e.hpAfter).toBe(maxHp.get(e.unit)! - damage.get(e.unit)!);
        checked++;
      }
    }
    expect(checked).toBeGreaterThan(0);
    expect(hpStatChanges).toBeGreaterThan(0); // Vitality fired, so hp StatChanged was checked too
    expect(ofType(log, "Heal").length).toBeGreaterThan(0); // Blessing fired, so Heals were checked too
  });
});

// ---------------------------------------------------------------------------
// 3b. Turn count: BattleEnd reports the deciding turn, not the loop's overshoot
// ---------------------------------------------------------------------------

describe("turn count", () => {
  test("BattleEnd.turns equals the turn the last Death happened on", () => {
    // Knight needs 4 hits, Brute needs 4 hits; with alternating strikes the
    // first-striker kills on its 4th hit — turn 4, whoever wins the coin.
    const teamA: UnitDef[] = [vanilla("Knight", 10, 2)];
    const teamB: UnitDef[] = [vanilla("Brute", 8, 3)];
    for (let seed = 1; seed <= 5; seed++) {
      const log = runBattle({ teamA, teamB, seed });
      const end = ofType(log, "BattleEnd")[0]!;
      const lastDeath = ofType(log, "Death").pop()!;
      const lastTurnStart = ofType(log, "TurnStart").pop()!;
      expect(end.turns).toBe(4);
      expect(end.turns).toBe(lastDeath.turn);
      expect(end.turns).toBe(lastTurnStart.turn);
      expect(end.turn).toBe(end.turns); // the event itself is stamped with the deciding turn
    }
  });
});

// ---------------------------------------------------------------------------
// 4. Fatigue / draw
// ---------------------------------------------------------------------------

describe("fatigue and draw", () => {
  test("two 0-pwr 30-hp units → draw with Fatigue events", () => {
    const teamA: UnitDef[] = [vanilla("Dummy", 30, 0)];
    const teamB: UnitDef[] = [vanilla("Dummy", 30, 0)];
    const log = runBattle({ teamA, teamB, seed: 1 });

    const fatigueEvents = ofType(log, "Fatigue");
    expect(fatigueEvents.length).toBeGreaterThan(0);

    const end = ofType(log, "BattleEnd")[0];
    expect(end).toBeDefined();
    expect(end!.winner).toBe("draw");
    // Both die simultaneously from fatigue, well before TURN_CAP=200.
    expect(end!.turns).toBeLessThan(200);
    // Decided by deaths, not by the clock: no timeUp.
    expect(end!.timeUp).toBeUndefined();
  });

  test("turnCap: both sides standing after the last turn is a draw marked timeUp (R3-26)", () => {
    const wall = vanilla("Wall", 100_000, 1);
    const log = runBattle({ teamA: [wall], teamB: [wall], seed: 1, turnCap: 12 });
    const end = ofType(log, "BattleEnd")[0]!;
    expect(end).toMatchObject({ winner: "draw", turns: 12, timeUp: true, turn: 12, source: "kernel", causedBy: null });
    expect(Math.max(...log.map((e) => e.turn))).toBe(12);
    // Fatigue still ran until the cap.
    expect(ofType(log, "Fatigue").map((e) => e.turn)).toEqual([10, 11, 12]);
    // Omitted = TURN_CAP (200).
    expect(ofType(runBattle({ teamA: [wall], teamB: [wall], seed: 1 }), "BattleEnd")[0]).toMatchObject({ winner: "draw", turns: 200, timeUp: true });
  });

  test("sudden death (R4-1): fatigue is linear to the turn before, then doubles every turn", () => {
    expect([10, 11, 19].map((t) => fatigueAmount(t))).toEqual([1, 2, 10]);
    expect([10, 19, 20, 21, 22].map((t) => fatigueAmount(t, 20))).toEqual([1, 10, 20, 40, 80]);
    expect(fatigueAmount(25)).toBe(16); // no sudden death: linear forever
  });

  test("sudden death pierces Shield and Blessing, and the battle ends by itself", () => {
    // Shield 1000 and Blessing 5 hold off linear fatigue; sudden death from turn 12 goes through both.
    const tank: UnitDef = { name: "Tank", base: { hp: 200, pwr: 0 }, ability: "Strike", statuses: [{ status: "Shield", stacks: 1000 }, { status: "Blessing", stacks: 5 }] };
    const log = runBattle({ teamA: [tank], teamB: [tank], seed: 1, statuses: stressRegistry, suddenDeathAt: 12 });
    const fat = ofType(log, "Fatigue");
    expect(fat.map((e) => [e.turn, e.amount, e.suddenDeath ?? false])).toEqual([
      [10, 1, false], [11, 2, false], [12, 4, true], [13, 8, true], [14, 16, true], [15, 32, true], [16, 64, true], [17, 128, true],
    ]);
    const hurts = ofType(log, "Hurt").filter((h) => log[h.causedBy!]!.type === "Fatigue");
    expect(hurts).toHaveLength(16);
    // Before sudden death Shield takes it all; from turn 12 nothing is absorbed.
    expect(hurts.filter((h) => h.turn < 12).every((h) => h.amount === 0 && h.absorbed! > 0 && !h.pierced)).toBe(true);
    expect(hurts.filter((h) => h.turn >= 12).every((h) => h.pierced && h.absorbed === undefined)).toBe(true);
    // Both fall on turn 17 (4+8+…+128 = 252 ≥ 200); Blessing never cancels the death.
    expect(ofType(log, "Death").map((d) => [d.turn, d.pierced])).toEqual([[17, true], [17, true]]);
    expect(ofType(log, "Intercepted")).toEqual([]);
    expect(ofType(log, "BattleEnd")[0]).toMatchObject({ winner: "draw", turns: 17 });
    expect(ofType(log, "BattleEnd")[0]!.timeUp).toBeUndefined();
    // The same tanks without sudden death last until linear fatigue wears through the Shield.
    expect(ofType(runBattle({ teamA: [tank], teamB: [tank], seed: 1, statuses: stressRegistry }), "BattleEnd")[0]!.turns).toBeGreaterThan(40);
  });

  test("in sudden death Summon and Revive do nothing, and the log says why", () => {
    const ogre = vanilla("Ogre", 1_000, 10);
    const teamA = [vanilla("Mouse", 1, 0), Summoner, Necromancer];
    const on = runBattle({ teamA, teamB: [ogre], seed: 1, suddenDeathAt: 1 });
    expect(ofType(on, "Summon")).toEqual([]);
    const failed = ofType(on, "SummonFailed");
    expect(failed.map((e) => [e.name, e.revive ?? false, e.reason])).toEqual(
      expect.arrayContaining([["Imp", false, "suddenDeath"], ["Mouse", true, "suddenDeath"]]),
    );
    for (const e of failed) expect(e.source).not.toBe("kernel"); // sourced to the ability that tried
    // Without it, the same line summons and revives.
    const off = runBattle({ teamA, teamB: [ogre], seed: 1 });
    expect(ofType(off, "Summon").length).toBeGreaterThan(0);
    expect(ofType(off, "SummonFailed")).toEqual([]);
    expect(() => runBattle({ teamA, teamB: [ogre], seed: 1, suddenDeathAt: 0 })).toThrow(/suddenDeathAt/);
  });

  test("turnCap: a battle decided on the last turn is a win, not a time-up", () => {
    // 10 hp against 1 pwr: the Ogre's tenth strike kills on turn 10.
    const log = runBattle({ teamA: [vanilla("Ogre", 1_000, 1)], teamB: [vanilla("Mouse", 10, 0)], seed: 1, turnCap: 10 });
    const end = ofType(log, "BattleEnd")[0]!;
    expect(end).toMatchObject({ winner: "A", turns: 10 });
    expect(end.timeUp).toBeUndefined();
  });

  test("turnCap must be a positive integer", () => {
    const wall = vanilla("Wall", 50, 1);
    expect(() => runBattle({ teamA: [wall], teamB: [wall], seed: 1, turnCap: 0 })).toThrow(/turnCap/);
    expect(() => runBattle({ teamA: [wall], teamB: [wall], seed: 1, turnCap: 2.5 })).toThrow(/turnCap/);
  });
});

// ---------------------------------------------------------------------------
// 5. Line collapse
// ---------------------------------------------------------------------------

describe("line collapse", () => {
  test("after Weak dies, a PairFaced involving Backup appears", () => {
    const weak = vanilla("Weak", 1, 1);
    const backup = vanilla("Backup", 10, 3);
    const ogre = vanilla("Ogre", 12, 2);

    const teamA: UnitDef[] = [weak, backup];
    const teamB: UnitDef[] = [ogre];

    const log = runBattle({ teamA, teamB, seed: 1 });

    // Weak must die.
    const deaths = ofType(log, "Death");
    const weakDeath = deaths.find((d) => d.unit.includes("Weak"));
    expect(weakDeath).toBeDefined();

    // After Weak's death, a PairFaced involving Backup's id must appear.
    const backupId = log
      .filter((e) => e.type === "BattleStart")
      .flatMap(() => {
        const bs = ofType(log, "BattleStart")[0];
        return bs ? bs.teams.A : [];
      })
      .find((r) => r.name === "Backup")?.id;

    expect(backupId).toBeDefined();

    const pairFaceds = ofType(log, "PairFaced");
    const backupPair = pairFaceds.find((pf) => pf.id > (weakDeath?.id ?? 0) && (pf.a === backupId || pf.b === backupId));
    expect(backupPair).toBeDefined();
  });
});

// ---------------------------------------------------------------------------
// 6. No-self-retrigger law
// ---------------------------------------------------------------------------

describe("no-self-retrigger law", () => {
  test("a unit that hurts itself when hurt is blocked by ChainBlocked after one Hurt per enemy strike", () => {
    // Unit with: when Hurt on holder → damage holder 1.
    const selfHurterAbilities: AbilityRegistry = {
      ...stressAbilities,
      SelfHurt: {
        name: "SelfHurt",
        family: "Strike",
        whens: [{ kind: "trigger", on: { on: "Hurt", unit: "holder" } }],
        selectors: [{ kind: "holder" }],
        effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }],
      },
    };
    const selfHurter: UnitDef = {
      name: "SelfHurter",
      base: { hp: 100, pwr: 0 },
      ability: "SelfHurt",
    };
    const attacker = vanilla("Attacker", 100, 2);

    const teamA: UnitDef[] = [selfHurter];
    const teamB: UnitDef[] = [attacker];

    const log = runBattle({ teamA, teamB, seed: 1, abilities: selfHurterAbilities });

    // ChainBlocked events must exist.
    const chainBlocked = ofType(log, "ChainBlocked");
    expect(chainBlocked.length).toBeGreaterThan(0);

    // For each enemy strike in turn 1, there is exactly one Hurt on SelfHurter sourced from the self-hurting ability.
    const turn1Strikes = ofType(log, "Strike").filter((e) => e.turn === 1 && e.striker.startsWith("B"));
    expect(turn1Strikes.length).toBeGreaterThan(0);

    // Find SelfHurter's unit id (A-side, front).
    const selfHurterId = ofType(log, "BattleStart")[0]?.teams.A[0]?.id;
    expect(selfHurterId).toBeDefined();

    // Find ability ref for the self-hurting ability.
    const abilityRef = { unit: selfHurterId!, ability: 0 };

    // Per strike: exactly one Hurt on selfHurter sourced from that ability.
    for (const strike of turn1Strikes) {
      const hurtsFromAbility = ofType(log, "Hurt").filter(
        (h) =>
          h.unit === selfHurterId &&
          h.causedBy !== null &&
          h.source !== "kernel" &&
          h.source.unit === abilityRef.unit &&
          h.source.ability === abilityRef.ability,
      );
      // We look at Hurts caused downstream of this particular Strike.
      // Since there's only one strike per turn from attacker and one self-hurt ability,
      // the total self-sourced Hurts should equal the number of attacker strikes (one per strike cycle).
      // More precisely: per enemy strike there is exactly 1 ability-sourced Hurt.
      // We'll just assert total = total strikes (1:1 ratio), tested turn-by-turn would be complex.
      // For turn 1 specifically:
      const t1AbilityHurts = ofType(log, "Hurt").filter(
        (h) =>
          h.turn === 1 &&
          h.unit === selfHurterId &&
          h.source !== "kernel" &&
          typeof h.source === "object" &&
          h.source.unit === selfHurterId &&
          h.source.ability === 0,
      );
      expect(t1AbilityHurts.length).toBe(1);
      break; // Only need to check once since we tested turn 1
    }
  });
});
