import { describe, expect, test } from "vitest";
import { battle } from "./battle.js";
import { InMemoryLadderStore } from "./ladder.js";
import {
  applyDecision,
  awakenFusion,
  buy,
  challengeBoss,
  deserializeRun,
  fight,
  fuse,
  initRun,
  ladderFight,
  reorder,
  reroll,
  runToJSONL,
  serializeRun,
  toBattleTeam,
  type RunState,
} from "./run.js";
import type { AbilityRegistry, UnitDef } from "./types.js";

const abilities: AbilityRegistry = {
  AlphaAct: { name: "AlphaAct", family: "Strike", effects: [{ kind: "damage", amount: { kind: "const", value: 1 } }] },
  BetaAct: { name: "BetaAct", family: "Heal", effects: [{ kind: "heal", amount: { kind: "const", value: 1 } }] },
  GammaAct: { name: "GammaAct", family: "Shield", effects: [{ kind: "heal", amount: { kind: "const", value: 2 } }] },
};
const Alpha: UnitDef = {
  name: "Alpha", base: { pwr: 2, hp: 5 },
  triggers: [{ kind: "trigger", on: { on: "BattleStart" } }],
  selectors: [{ kind: "frontEnemy" }], abilities: ["AlphaAct"],
  statuses: [{ status: "Mark", stacks: 2 }, { status: "Guard", stacks: 1 }],
};
const Beta: UnitDef = {
  name: "Beta", base: { pwr: 3, hp: 7 },
  triggers: [{ kind: "trigger", on: { on: "TurnStart" } }],
  selectors: [{ kind: "holder" }], abilities: ["BetaAct"],
  statuses: [{ status: "Mark", stacks: 3 }, { status: "Haste", stacks: 2 }],
};
const statuses = {
  Mark: { name: "Mark", abilities: [] },
  Guard: { name: "Guard", abilities: [] },
  Haste: { name: "Haste", abilities: [] },
};
const Dummy: UnitDef = { name: "Dummy", base: { pwr: 0, hp: 50 }, triggers: [{ kind: "trigger", on: { on: "BattleStart" } }], selectors: [{ kind: "holder" }], abilities: ["AlphaAct"] };

function purchase(s: RunState, def: UnitDef): RunState {
  return buy({ ...s, gold: 999, offers: [def] }, 0);
}
function copies(s: RunState, def: UnitDef, total: number): RunState {
  for (let i = 0; i < total; i++) s = purchase(s, def);
  return s;
}
function ready(): RunState {
  let s = initRun({ seed: 41, pool: [Alpha, Beta], statuses, abilities });
  s = copies(s, Alpha, 3);
  s = copies(s, Beta, 3);
  return s;
}
function fused(order: "AB" | "BA" = "AB"): RunState {
  const s = ready();
  return order === "AB" ? fuse(s, 0, 1) : fuse(s, 1, 0);
}
function choicePending(): RunState {
  let s = fused();
  s = purchase(s, Alpha);
  s = purchase(s, Beta);
  return purchase(s, Alpha);
}

describe("AOI-61 base Awakening", () => {
  test("first copy is 1/3; every duplicate is literal +1 PWR/+2 HP; third Awakens once and later copies stack forever", () => {
    let s = initRun({ seed: 1, pool: [Alpha], statuses, abilities });
    s = purchase(s, Alpha);
    expect(s.team[0]).toMatchObject({ copies: 1, progression: "Base", base: { pwr: 2, hp: 5 } });
    s = purchase(s, Alpha);
    expect(s.team[0]).toMatchObject({ copies: 2, progression: "Base", base: { pwr: 3, hp: 7 } });
    s = purchase(s, Alpha);
    expect(s.team[0]).toMatchObject({ copies: 3, progression: "Awakened", base: { pwr: 4, hp: 9 } });
    s = copies(s, Alpha, 7);
    expect(s.team[0]).toMatchObject({ copies: 10, progression: "Awakened", base: { pwr: 11, hp: 23 } });
    expect(s.log.filter((e) => e.type === "Awakened")).toHaveLength(1);
    expect(s.log.filter((e) => e.type === "DuplicateGrown")).toHaveLength(9);
  });
});

describe("AOI-61 ordered fusion", () => {
  test("A+B and B+A are distinct ordered outcomes with summed current stats, merged statuses, provenance, axes, and Ability order", () => {
    const ab = fused("AB").team[0]!;
    const ba = fused("BA").team[0]!;
    expect(ab).toMatchObject({ name: "Alpha + Beta", base: { pwr: 9, hp: 20 }, kind: "composite" });
    expect(ab.def.statuses).toEqual([
      { status: "Mark", stacks: 5 }, { status: "Guard", stacks: 1 }, { status: "Haste", stacks: 2 },
    ]);
    expect(ab.fusion!.parents.map((p) => p.name)).toEqual(["Alpha", "Beta"]);
    expect(ab.def.triggers).toEqual(Alpha.triggers);
    expect(ab.def.selectors).toEqual(Beta.selectors);
    expect(ab.def.abilities).toEqual(["AlphaAct", "BetaAct"]);
    expect(ba.fusion!.parents.map((p) => p.name)).toEqual(["Beta", "Alpha"]);
    expect(ba.def.triggers).toEqual(Beta.triggers);
    expect(ba.def.selectors).toEqual(Alpha.selectors);
    expect(ba.def.abilities).toEqual(["BetaAct", "AlphaAct"]);
  });

  test("rejects unawakened, same-Ability, self, composite, and repeat fusion", () => {
    const base = initRun({ seed: 1, pool: [Alpha, Beta], statuses, abilities });
    const two = copies(copies(base, Alpha, 1), Beta, 1);
    expect(() => fuse(two, 0, 1)).toThrow(/both base Units must be Awakened/);
    const sameBeta = { ...Beta, name: "Beta2", abilities: ["AlphaAct"] };
    let same = initRun({ seed: 2, pool: [Alpha, sameBeta], statuses, abilities });
    same = copies(copies(same, Alpha, 3), sameBeta, 3);
    expect(() => fuse(same, 0, 1)).toThrow(/different Abilities/);
    expect(() => fuse(ready(), 0, 0)).toThrow(/itself/);
    const resolved = awakenFusion(choicePending(), "trigger");
    const compositePlusBase: RunState = { ...resolved, team: [...resolved.team, { name: Beta.name, base: { ...Beta.base }, kind: "base", copies: 3, progression: "Awakened", def: Beta }] };
    expect(() => fuse(compositePlusBase, 0, 1)).toThrow(/composite cannot fuse again/);
  });

  test("draft pools require exactly one Ability while direct battle composites retain two", () => {
    const multi: UnitDef = { ...Alpha, name: "Multi", abilities: ["AlphaAct", "BetaAct"] };
    expect(() => initRun({ seed: 3, pool: [multi], statuses, abilities })).toThrow(/draft-pool base Unit must carry exactly one Ability/);
    const legacy: UnitDef = { name: "Legacy", base: { pwr: 1, hp: 2 }, ability: "AlphaAct" };
    expect(initRun({ seed: 3, pool: [legacy], statuses, abilities }).pool).toEqual([legacy]);
    expect(() => battle({ teamA: [multi], teamB: [Dummy], seed: 3, statuses, abilities })).not.toThrow();
  });

  test("defensive fusion refuses malformed multi-Ability parents without dropping data", () => {
    const s = structuredClone(ready());
    s.team[0]!.def.abilities = ["AlphaAct", "GammaAct"];
    const before = JSON.stringify(s);
    expect(() => fuse(s, 0, 1)).toThrow(/carries 2 Abilities.*refused without changing or dropping/s);
    expect(JSON.stringify(s)).toBe(before);
  });

  test("battle projection executes inherited Abilities in parent order using the ordered axes", () => {
    const unit = toBattleTeam(fused("AB").team)[0]!;
    const log = battle({ teamA: [unit], teamB: [Dummy], seed: 3, statuses, abilities });
    const fired = log.filter((e) => e.source !== "kernel" && (e.type === "Hurt" || e.type === "Heal"));
    expect(fired.slice(0, 2).map((e) => e.source === "kernel" ? -1 : e.source.ability)).toEqual([0, 1]);
    expect(fired.slice(0, 2).map((e) => e.type)).toEqual(["Hurt", "Heal"]);
  });
});

describe("AOI-61 fusion Awakening", () => {
  test("either parent routes to a fresh shared 0/3 meter, every copy grows immediately, and third blocks", () => {
    let s = fused();
    expect(s.team[0]!.fusion!.meter).toBe(0);
    s = purchase(s, Alpha);
    s = purchase(s, Beta);
    s = purchase(s, Alpha);
    expect(s.team[0]).toMatchObject({ base: { pwr: 12, hp: 26 }, fusion: { meter: 3, doubled: false } });
    expect(s.log.filter((e) => e.type === "FusionAwakeningRequired")).toHaveLength(1);
    expect(() => reroll(s)).toThrow(/must choose Trigger path or Selector path/);
  });

  test.each(["trigger", "selector"] as const)("%s path appends the complete unused axis after existing, doubles once, then later increments stay literal", (path) => {
    const pending = choicePending();
    let s = awakenFusion(pending, path);
    const u = s.team[0]!;
    expect(u.base).toEqual({ pwr: 24, hp: 52 });
    expect(u.fusion).toMatchObject({ meter: 3, awakening: path, doubled: true });
    if (path === "trigger") expect(u.def.triggers).toEqual([...(Alpha.triggers ?? []), ...(Beta.triggers ?? [])]);
    else expect(u.def.selectors).toEqual([...(Beta.selectors ?? []), ...(Alpha.selectors ?? [])]);
    expect(() => awakenFusion(s, path)).toThrow(/no fusion Awakening choice is pending/);
    s = purchase(s, Beta);
    expect(s.team[0]!.base).toEqual({ pwr: 25, hp: 54 });
    expect(s.log.filter((e) => e.type === "FusionAwakened")).toHaveLength(1);
  });

  test("pending choice blocks buy, reroll, reorder, fuse, fight, ladderFight, challengeBoss, and direct applyDecision", () => {
    const s = choicePending();
    const ladder = new InMemoryLadderStore();
    const attempts = [
      () => purchase(s, Alpha), () => reroll(s), () => reorder(s, 0, 0), () => fuse(s, 0, 0),
      () => fight(s, [Dummy]), () => ladderFight(s, ladder), () => challengeBoss(s, ladder),
      () => applyDecision(s, { kind: "reroll" }),
    ];
    for (const attempt of attempts) expect(attempt).toThrow(/must choose Trigger path or Selector path/);
  });
});

describe("AOI-61 run persistence and replay", () => {
  test("versioned ordered fusion round-trips byte-stably and continues deterministically", () => {
    const mid = choicePending();
    const revived = deserializeRun(serializeRun(mid));
    const a = purchase(awakenFusion(mid, "selector"), Beta);
    const b = purchase(awakenFusion(revived, "selector"), Beta);
    expect(serializeRun(b)).toBe(serializeRun(a));
    expect(runToJSONL(b.log)).toBe(runToJSONL(a.log));
  });

  test("a valid pending choice survives resume and still blocks every ordinary action", () => {
    const revived = deserializeRun(serializeRun(choicePending()));
    const ladder = new InMemoryLadderStore();
    const attempts = [
      () => purchase(revived, Alpha), () => reroll(revived), () => reorder(revived, 0, 0), () => fuse(revived, 0, 0),
      () => fight(revived, [Dummy]), () => ladderFight(revived, ladder), () => challengeBoss(revived, ladder),
      () => applyDecision(revived, { kind: "reroll" }),
    ];
    for (const attempt of attempts) expect(attempt).toThrow(/must choose Trigger path or Selector path/);
  });

  test.each(["level", "stacks", "absorbed"] as const)("actionably rejects unversioned legacy team field %s", (field) => {
    const legacy = JSON.parse(serializeRun(ready()));
    delete legacy.runVersion;
    legacy.team[0][field] = field === "absorbed" ? ["BetaAct"] : 1;
    expect(() => deserializeRun(JSON.stringify(legacy))).toThrow(new RegExp(`team\\[0\\]\\.${field}.*Legacy level/stacks/absorbed.*Start a fresh run.*migration tool`, "s"));
  });

  test.each(["level", "stacks", "absorbed"] as const)("actionably rejects stamped-v2 legacy team field %s", (field) => {
    const legacy = JSON.parse(serializeRun(ready()));
    legacy.team[0][field] = field === "absorbed" ? ["BetaAct"] : 1;
    expect(() => deserializeRun(JSON.stringify(legacy))).toThrow(new RegExp(`team\\[0\\]\\.${field}.*Legacy level/stacks/absorbed.*Start a fresh run.*migration tool`, "s"));
  });

  test.each([
    ["null parents", (run: any) => { run.team[0].fusion.parents = [null, null]; }, /parents\[0\] must be an object/],
    ["invalid parent def", (run: any) => { run.team[0].fusion.parents[0].def = { nope: true }; }, /parent.*name must match def\.name|UnitDef/s],
    ["meter below range", (run: any) => { run.team[0].fusion.meter = -1; }, /meter must be an integer from 0 to 3/],
    ["meter above range", (run: any) => { run.team[0].fusion.meter = 4; }, /meter must be an integer from 0 to 3/],
    ["unknown path", (run: any) => { run.team[0].fusion.awakening = "bogus"; }, /path must be trigger or selector/],
    ["doubled pending state", (run: any) => { run.team[0].fusion.doubled = true; }, /cannot be doubled before a path is chosen/],
    ["chosen but not doubled", (run: any) => { run.team[0].fusion.awakening = "trigger"; }, /requires meter 3 and doubled true/],
    ["chosen below threshold", (run: any) => { run.team[0].fusion.meter = 2; run.team[0].fusion.awakening = "trigger"; run.team[0].fusion.doubled = true; }, /requires meter 3 and doubled true/],
    ["invalid composite progression", (run: any) => { run.team[0].progression = "Awakened"; }, /composite progression must be Base for its meter\/path state/],
  ] as const)("rejects malformed v2 fusion state: %s", (_label, mutate, expected) => {
    const malformed = JSON.parse(serializeRun(choicePending()));
    mutate(malformed);
    expect(() => deserializeRun(JSON.stringify(malformed))).toThrow(expected);
    expect(() => deserializeRun(JSON.stringify(malformed))).toThrow(/Start a fresh run or use an explicit migration tool/);
  });

  test("rejects malformed base progression and future versions with migration guidance", () => {
    const malformed = JSON.parse(serializeRun(ready()));
    malformed.team[0].progression = "Ascended";
    expect(() => deserializeRun(JSON.stringify(malformed))).toThrow(/progression must be Base or Awakened.*Start a fresh run/s);
    const future = { ...JSON.parse(serializeRun(ready())), runVersion: 99 };
    expect(() => deserializeRun(JSON.stringify(future))).toThrow(/version is 99; expected 2.*Start a fresh run/s);
  });
});
