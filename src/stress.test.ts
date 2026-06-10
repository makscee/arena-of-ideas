// Stress set acceptance tests — SPEC §7.
// One test per ability; each verifies the correct engine behaviour.

import { describe, expect, test } from "vitest";
import { battle } from "./battle.js";
import type { BattleEvent, BattleInput, UnitDef } from "./types.js";
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
  Vitality,
} from "./content/stress.js";

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

const ofType = <T extends BattleEvent["type"]>(
  log: BattleEvent[],
  t: T,
): Extract<BattleEvent, { type: T }>[] =>
  log.filter((e): e is Extract<BattleEvent, { type: T }> => e.type === t);

function run(input: BattleInput): BattleEvent[] {
  return battle(input);
}

// ---------------------------------------------------------------------------
// 1. Strength
// ---------------------------------------------------------------------------

describe("stress: Strength", () => {
  test("strike Hurt has amount 3 and StatChanged delta+2 appears at battle start", () => {
    // Striker: 10/1 base + Strength 2 (each stack +1 pwr) → effective pwr = 3.
    // Passive target: 100/0 so it never kills the striker.
    const log = run({
      teamA: [
        {
          name: "Striker",
          base: { hp: 10, pwr: 1 },
          statuses: [{ status: "Strength", stacks: 2 }],
        },
      ],
      teamB: [{ name: "Target", base: { hp: 100, pwr: 0 } }],
      seed: 1,
      statuses: stressRegistry,
    });

    // StatChanged for pwr +2 emitted at turn 0 (battle start / initial status application).
    const statChanges = ofType(log, "StatChanged").filter(
      (e) => e.turn === 0 && e.stat === "pwr" && e.delta === 2,
    );
    expect(statChanges.length).toBeGreaterThan(0);
    expect(statChanges[0]!.now).toBe(3); // effective pwr after applying Strength 2

    // The first strike Hurt on the target carries amount = effective pwr = 3.
    const hurtsOnTarget = ofType(log, "Hurt").filter(
      (h) => h.unit.startsWith("B") && h.source === "kernel",
    );
    expect(hurtsOnTarget[0]!.amount).toBe(3);
  });
});

// ---------------------------------------------------------------------------
// 2. Vitality
// ---------------------------------------------------------------------------

describe("stress: Vitality", () => {
  test("defender survives turn 1, dies turn 2 (effective hp = base + stacks)", () => {
    // Defender base hp=3, Vitality 4 → effective hp = 7.
    // Attacker pwr=5: 7-5=2 hp remaining after turn 1 → alive.
    // Turn 2 strike deals 5 more → -3 ≤ 0 → Death.
    const log = run({
      teamA: [{ name: "Attacker", base: { hp: 100, pwr: 5 } }],
      teamB: [
        {
          name: "VitalityDef",
          base: { hp: 3, pwr: 0 },
          statuses: [{ status: "Vitality", stacks: 4 }],
        },
      ],
      seed: 1,
      statuses: stressRegistry,
    });

    const defId = ofType(log, "BattleStart")[0]!.teams.B[0]!.id;

    const turn1Deaths = ofType(log, "Death").filter(
      (d) => d.turn === 1 && d.unit === defId,
    );
    expect(turn1Deaths.length).toBe(0); // survives turn 1

    const turn2Deaths = ofType(log, "Death").filter(
      (d) => d.turn === 2 && d.unit === defId,
    );
    expect(turn2Deaths.length).toBe(1); // dies turn 2
  });
});

// ---------------------------------------------------------------------------
// 3. Curse
// ---------------------------------------------------------------------------

describe("stress: Curse", () => {
  test("effective pwr floored to 0 → strike Hurt amount is 0", () => {
    // Striker base pwr=2, Curse 5 (−1 pwr/stack) → max(0, 2−5) = 0.
    const log = run({
      teamA: [
        {
          name: "CursedStriker",
          base: { hp: 10, pwr: 2 },
          statuses: [{ status: "Curse", stacks: 5 }],
        },
      ],
      teamB: [{ name: "Target", base: { hp: 100, pwr: 0 } }],
      seed: 1,
      statuses: stressRegistry,
    });

    const firstHurtOnTarget = ofType(log, "Hurt")
      .filter((h) => h.unit.startsWith("B") && h.source === "kernel")[0];

    expect(firstHurtOnTarget).toBeDefined();
    expect(firstHurtOnTarget!.amount).toBe(0);
  });
});

// ---------------------------------------------------------------------------
// 4. Poison
// ---------------------------------------------------------------------------

describe("stress: Poison", () => {
  test("TurnEnd Hurts decay 3,2,1 on turns 1,2,3; fully consumed after turn 3", () => {
    // Both 20/0 so strikes do nothing. Poison 3 on unit A.
    // TurnEnd turn 1: Hurt 3 + StatusRemoved {stacks:1, remaining:2}.
    // TurnEnd turn 2: Hurt 2 + StatusRemoved {stacks:1, remaining:1}.
    // TurnEnd turn 3: Hurt 1 + StatusRemoved {stacks:1, remaining:0}.
    // Turn 4: no Poison Hurt.
    const poisonUnitDef: UnitDef = {
      name: "PoisonedUnit",
      base: { hp: 20, pwr: 0 },
      statuses: [{ status: "Poison", stacks: 3 }],
    };
    const log = run({
      teamA: [poisonUnitDef],
      teamB: [{ name: "Dummy", base: { hp: 20, pwr: 0 } }],
      seed: 1,
      statuses: stressRegistry,
    });

    const unitId = ofType(log, "BattleStart")[0]!.teams.A[0]!.id;

    const poisonSource = (h: BattleEvent) =>
      h.source !== "kernel" &&
      typeof h.source === "object" &&
      h.source.status === "Poison";

    // TurnEnd Hurts from Poison.
    const poisonHurts = ofType(log, "Hurt").filter(
      (h) => h.unit === unitId && poisonSource(h),
    );
    expect(poisonHurts[0]!.turn).toBe(1);
    expect(poisonHurts[0]!.amount).toBe(3);
    expect(poisonHurts[1]!.turn).toBe(2);
    expect(poisonHurts[1]!.amount).toBe(2);
    expect(poisonHurts[2]!.turn).toBe(3);
    expect(poisonHurts[2]!.amount).toBe(1);

    // StatusRemoved events — each consumes 1 stack.
    const removals = ofType(log, "StatusRemoved").filter(
      (r) => r.unit === unitId && r.status === "Poison",
    );
    expect(removals[0]!.stacks).toBe(1);
    expect(removals[0]!.remaining).toBe(2);
    expect(removals[1]!.stacks).toBe(1);
    expect(removals[1]!.remaining).toBe(1);
    expect(removals[2]!.stacks).toBe(1);
    expect(removals[2]!.remaining).toBe(0);

    // No Poison Hurt on turn 4 (status fully consumed).
    const turn4PoisonHurts = ofType(log, "Hurt").filter(
      (h) => h.turn === 4 && h.unit === unitId && poisonSource(h),
    );
    expect(turn4PoisonHurts.length).toBe(0);
  });
});

// ---------------------------------------------------------------------------
// 5. Shield
// ---------------------------------------------------------------------------

describe("stress: Shield", () => {
  test("partial absorption: Shield 3 vs 5-pwr → Hurt amount 2, absorbed 3, Shield removed", () => {
    const log = run({
      teamA: [{ name: "Attacker", base: { hp: 100, pwr: 5 } }],
      teamB: [
        {
          name: "ShieldUnit",
          base: { hp: 20, pwr: 0 },
          statuses: [{ status: "Shield", stacks: 3 }],
        },
      ],
      seed: 1,
      statuses: stressRegistry,
    });

    const unitId = ofType(log, "BattleStart")[0]!.teams.B[0]!.id;
    const firstHurt = ofType(log, "Hurt").filter((h) => h.unit === unitId)[0]!;

    expect(firstHurt.amount).toBe(2);
    expect(firstHurt.absorbed).toBe(3);

    const removal = ofType(log, "StatusRemoved").filter(
      (r) => r.unit === unitId && r.status === "Shield",
    )[0]!;
    expect(removal.stacks).toBe(3);
    expect(removal.remaining).toBe(0);
  });

  test("full absorption: Shield 9 vs 5-pwr → Hurt amount 0, absorbed 5, stacks 5 consumed remaining 4", () => {
    const log = run({
      teamA: [{ name: "Attacker", base: { hp: 100, pwr: 5 } }],
      teamB: [
        {
          name: "ShieldUnit",
          base: { hp: 20, pwr: 0 },
          statuses: [{ status: "Shield", stacks: 9 }],
        },
      ],
      seed: 1,
      statuses: stressRegistry,
    });

    const unitId = ofType(log, "BattleStart")[0]!.teams.B[0]!.id;
    const firstHurt = ofType(log, "Hurt").filter((h) => h.unit === unitId)[0]!;

    expect(firstHurt.amount).toBe(0);
    expect(firstHurt.absorbed).toBe(5);

    const removal = ofType(log, "StatusRemoved").filter(
      (r) => r.unit === unitId && r.status === "Shield",
    )[0]!;
    expect(removal.stacks).toBe(5);
    expect(removal.remaining).toBe(4);
  });
});

// ---------------------------------------------------------------------------
// 6. Freeze
// ---------------------------------------------------------------------------

describe("stress: Freeze", () => {
  test("Freeze 1: turn 1 Intercepted for Strike, zero Hurt on attacker; turn 2 attacker takes 2", () => {
    // FreezeUnit (B side, 20/2, Freeze 1) vs Attacker (A side, 20/1).
    // Regardless of who wins the first-strike coin:
    //   - FreezeUnit's strike is intercepted and cancelled (Freeze consumes 1 stack).
    //   - Zero Hurt events on the attacker in turn 1.
    //   - Turn 2: Freeze is gone; FreezeUnit strikes normally → attacker takes 2.
    const log = run({
      teamA: [{ name: "Attacker", base: { hp: 20, pwr: 1 } }],
      teamB: [
        {
          name: "FreezeUnit",
          base: { hp: 20, pwr: 2 },
          statuses: [{ status: "Freeze", stacks: 1 }],
        },
      ],
      seed: 1,
      statuses: stressRegistry,
    });

    const attackerId = ofType(log, "BattleStart")[0]!.teams.A[0]!.id;

    // Exactly one Intercepted event for original "Strike" in turn 1.
    const intercepted = ofType(log, "Intercepted").filter(
      (e) => e.turn === 1 && e.original === "Strike",
    );
    expect(intercepted.length).toBe(1);

    // Zero Hurt events on the attacker in turn 1.
    const turn1AttackerHurts = ofType(log, "Hurt").filter(
      (h) => h.turn === 1 && h.unit === attackerId,
    );
    expect(turn1AttackerHurts.length).toBe(0);

    // Turn 2: attacker takes a Hurt of 2 (FreezeUnit's pwr).
    const turn2AttackerHurts = ofType(log, "Hurt").filter(
      (h) => h.turn === 2 && h.unit === attackerId,
    );
    expect(turn2AttackerHurts.length).toBeGreaterThan(0);
    expect(turn2AttackerHurts[0]!.amount).toBe(2);
  });
});

// ---------------------------------------------------------------------------
// 7. Blessing
// ---------------------------------------------------------------------------

describe("stress: Blessing", () => {
  test("death-prevention: no Death turn 1, Intercepted+Heal to 2 hp, Blessing removed; dies turn 2", () => {
    // Defender 5hp + Blessing 2 vs 99-pwr attacker.
    // Turn 1: Strike → Hurt(99) → proposed Death → Blessing intercepts:
    //   Intercepted{original:"Death"}, then Heal bringing hp up to stacks(=2), Blessing removed.
    //   No Death event in turn 1.
    // Turn 2: 99-pwr strike again → dies for real.
    const log = run({
      teamA: [{ name: "Attacker", base: { hp: 100, pwr: 99 } }],
      teamB: [
        {
          name: "BlessedUnit",
          base: { hp: 5, pwr: 0 },
          statuses: [{ status: "Blessing", stacks: 2 }],
        },
      ],
      seed: 1,
      statuses: stressRegistry,
    });

    const defId = ofType(log, "BattleStart")[0]!.teams.B[0]!.id;

    // Turn 1: Intercepted for Death.
    const intercepted = ofType(log, "Intercepted").filter(
      (e) => e.turn === 1 && e.original === "Death",
    );
    expect(intercepted.length).toBe(1);

    // Turn 1: Heal brings the defender back to 2 hp (stacks = 2).
    const heals = ofType(log, "Heal").filter(
      (h) => h.turn === 1 && h.unit === defId,
    );
    expect(heals.length).toBeGreaterThan(0);
    // After heal: damage reduced so that curHp = 2. base hp = 5, damage was 99, heal = 96.
    expect(heals[0]!.amount).toBe(96);

    // Blessing removed in turn 1.
    const blessingRemoved = ofType(log, "StatusRemoved").filter(
      (r) => r.turn === 1 && r.unit === defId && r.status === "Blessing",
    );
    expect(blessingRemoved.length).toBe(1);

    // No Death event in turn 1.
    const turn1Deaths = ofType(log, "Death").filter(
      (d) => d.turn === 1 && d.unit === defId,
    );
    expect(turn1Deaths.length).toBe(0);

    // Dies in turn 2.
    const turn2Deaths = ofType(log, "Death").filter(
      (d) => d.turn === 2 && d.unit === defId,
    );
    expect(turn2Deaths.length).toBe(1);
  });
});

// ---------------------------------------------------------------------------
// 8. Summon
// ---------------------------------------------------------------------------

describe("stress: Summon", () => {
  test("Summoner dies, Imp summoned on A, PairFaced involves Imp, B wins", () => {
    // Summoner (6hp/1pwr) vs Ogre (30hp/3pwr).
    // Summoner dies → triggers: summon Imp on A. Imp fights Ogre, loses. B wins.
    const log = run({
      teamA: [Summoner],
      teamB: [{ name: "Ogre", base: { hp: 30, pwr: 3 } }],
      seed: 1,
      statuses: stressRegistry,
    });

    // Summoner dies.
    const summonerDeath = ofType(log, "Death").find((d) =>
      d.unit.includes("Summoner"),
    );
    expect(summonerDeath).toBeDefined();

    // A Summon event for the Imp appears on side A (not resurrected).
    const impSummon = ofType(log, "Summon").find(
      (s) => s.name === "Imp" && s.side === "A",
    );
    expect(impSummon).toBeDefined();
    expect(impSummon!.resurrected).toBeUndefined();

    // A later PairFaced involves the Imp's unit id.
    const impId = impSummon!.unit;
    const impPair = ofType(log, "PairFaced").find(
      (pf) => pf.id > summonerDeath!.id && (pf.a === impId || pf.b === impId),
    );
    expect(impPair).toBeDefined();

    // B wins.
    const end = ofType(log, "BattleEnd")[0]!;
    expect(end.winner).toBe("B");
  });
});

// ---------------------------------------------------------------------------
// 9. Silence
// ---------------------------------------------------------------------------

describe("stress: Silence", () => {
  test("(a) disabling: Silenced event exists, ability-sourced Hurts from silenced unit are zero", () => {
    // B has a unit with TurnEnd→damage frontEnemy 5.
    // A has Silencer (BattleStart→silence frontEnemy).
    // After silencing, B's TurnEnd ability cannot fire: zero ability-sourced Hurts on A.
    const abilityUnit: UnitDef = {
      name: "AbilityUnit",
      base: { hp: 20, pwr: 0 },
      abilities: [
        {
          whens: [{ kind: "trigger", on: { on: "TurnEnd" } }],
          selectors: [{ kind: "frontEnemy" }],
          effects: [{ kind: "damage", amount: { kind: "const", value: 5 } }],
        },
      ],
    };
    const log = run({
      teamA: [Silencer],
      teamB: [abilityUnit],
      seed: 1,
      statuses: stressRegistry,
    });

    // Silenced event exists.
    const silencedEvs = ofType(log, "Silenced");
    expect(silencedEvs.length).toBeGreaterThan(0);

    // No Hurt on A side sourced from the silenced ability.
    const abilityHurtsOnA = ofType(log, "Hurt").filter(
      (h) =>
        h.unit.startsWith("A") &&
        h.source !== "kernel" &&
        typeof h.source === "object" &&
        h.source.unit.startsWith("B"),
    );
    expect(abilityHurtsOnA.length).toBe(0);
  });

  test("(b) layering: Brute 2hp+Vitality 5 (eff 7), attacker 30/4 + TurnEnd-silence ally → Brute dies turn 1 via silence", () => {
    // Turn 1: Attacker hits Brute for 4 → curHp 3. TurnEnd: SilenceAlly fires, removes Vitality
    //   → effective hp drops from 7 to 2, damage still 4 → curHp = -2 → Death.
    // The Death's causal ancestry reaches StatusRemoved (from Silence).
    const silenceAlly: UnitDef = {
      name: "SilenceAlly",
      base: { hp: 100, pwr: 0 },
      abilities: [
        {
          whens: [{ kind: "trigger", on: { on: "TurnEnd" } }],
          selectors: [{ kind: "frontEnemy" }],
          effects: [{ kind: "silence" }],
        },
      ],
    };
    const brute: UnitDef = {
      name: "Brute",
      base: { hp: 2, pwr: 1 },
      statuses: [{ status: "Vitality", stacks: 5 }],
    };
    const log = run({
      teamA: [{ name: "Attacker", base: { hp: 30, pwr: 4 } }, silenceAlly],
      teamB: [brute],
      seed: 1,
      statuses: stressRegistry,
    });

    const bruteId = ofType(log, "BattleStart")[0]!.teams.B[0]!.id;

    // Brute dies in turn 1.
    const bruteDeath = ofType(log, "Death").find(
      (d) => d.unit === bruteId && d.turn === 1,
    )!;
    expect(bruteDeath).toBeDefined();

    // Walk the causal chain: Death ← StatChanged ← StatusRemoved.
    const statChanged = log[bruteDeath.causedBy!]!;
    expect(statChanged.type).toBe("StatChanged");
    const statusRemoved = log[statChanged.causedBy!]!;
    expect(statusRemoved.type).toBe("StatusRemoved");
    if (statusRemoved.type === "StatusRemoved") {
      expect(statusRemoved.status).toBe("Vitality");
    }
  });
});

// ---------------------------------------------------------------------------
// 10. Resurrect
// ---------------------------------------------------------------------------

describe("stress: Resurrect", () => {
  test("Fodder dies from Poison, Necromancer resurrects it once; no Poison Hurts after resurrection; A wins", () => {
    // Fodder 3/1 + Poison 3, Necromancer 20/2 vs Ogre 3/1.
    // Turn 1: Ogre strikes Fodder for 1 (Fodder 3hp → 2hp). Fodder strikes Ogre for 1 (Ogre 3hp → 2hp).
    //   TurnEnd: Poison 3 → Hurt 3 on Fodder → curHp = 2-3 = -1 → Death.
    //   Necromancer fires on Death of ally → resurrects Fodder at 1hp. Statuses cleared at death.
    // A continues: Fodder (no Poison) + Necromancer vs Ogre 2hp. A wins eventually.
    // Exactly 1 Death of Fodder (Ogre dies before Fodder can die again in this arrangement).
    const fodder: UnitDef = {
      name: "Fodder",
      base: { hp: 3, pwr: 1 },
      statuses: [{ status: "Poison", stacks: 3 }],
    };
    const log = run({
      teamA: [fodder, Necromancer],
      teamB: [{ name: "Ogre", base: { hp: 3, pwr: 1 } }],
      seed: 1,
      statuses: stressRegistry,
    });

    const fodderBaseId = ofType(log, "BattleStart")[0]!.teams.A[0]!.id;

    // Exactly 1 Death of Fodder.
    const fodderDeaths = ofType(log, "Death").filter(
      (d) => d.unit === fodderBaseId,
    );
    expect(fodderDeaths.length).toBe(1);

    // A Summon {resurrected: true} event for Fodder.
    const resurrection = ofType(log, "Summon").find(
      (s) => s.unit === fodderBaseId && s.resurrected === true,
    );
    expect(resurrection).toBeDefined();

    // No Poison-sourced Hurts on Fodder after the resurrection event.
    const isPoison = (h: Extract<BattleEvent, { type: "Hurt" }>) =>
      h.source !== "kernel" &&
      typeof h.source === "object" &&
      h.source.status === "Poison";

    const poisonHurtsAfterResurrect = ofType(log, "Hurt").filter(
      (h) =>
        h.unit === fodderBaseId &&
        h.id > (resurrection?.id ?? Infinity) &&
        isPoison(h),
    );
    expect(poisonHurtsAfterResurrect.length).toBe(0);

    // A wins.
    const end = ofType(log, "BattleEnd")[0]!;
    expect(end.winner).toBe("A");
  });
});
