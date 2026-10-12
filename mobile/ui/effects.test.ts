// The battle's effect language (mission 1007, #1011).
import { expect, test } from "vitest";
import type { BattleEvent } from "../../src/types";
import { BIG_HIT_MIN, changeOf, type Change } from "../../src/mvp/trace";
import { BIG_HIT, LANGUAGE, effectsOf, statNets, type EffectKind } from "./effects";
import { cue, waveSound } from "./sound-map";

const logOf = (...evs: object[]): BattleEvent[] => evs.map((e, id) => ({ id, ...e })) as unknown as BattleEvent[];
const changesOf = (log: BattleEvent[]): Change[] => log.flatMap((e) => { const c = changeOf(e); return c ? [c] : []; });

test("any two kinds differ in at least two of colour, motion and sound", () => {
  const kinds = Object.keys(LANGUAGE) as EffectKind[];
  for (const a of kinds) for (const b of kinds) {
    if (a >= b) continue;
    const [x, y] = [LANGUAGE[a], LANGUAGE[b]];
    const diff = Number(x.colour !== y.colour) + Number(x.motion !== y.motion) + Number(x.sound !== y.sound);
    expect(diff, `${a} vs ${b}`).toBeGreaterThanOrEqual(2);
  }
});

test("each kind's sound is the cue a wave of it makes, and no two kinds share a file (heal is its own)", () => {
  expect(cue("heal").file).not.toBe(cue("buff").file);
  const log = logOf(
    { type: "Heal", unit: "a1", amount: 2 },
    { type: "StatChanged", unit: "a1", stat: "pwr", delta: 1, now: 3 },
    { type: "StatChanged", unit: "a1", stat: "pwr", delta: -1, now: 2 },
    { type: "StatusApplied", unit: "a1", status: "Poison", stacks: 1, total: 1 },
    { type: "Death", unit: "a1" },
  );
  const step = (i: number) => ({ eventIds: [i], changes: [], caption: "", turn: 1, actor: null, subject: null }) as never;
  expect(waveSound(step(0), log, 0)?.key).toBe(LANGUAGE.heal.sound);
  expect(waveSound(step(1), log, 0)?.key).toBe(LANGUAGE.buff.sound);
  expect(waveSound(step(2), log, 0)?.key).toBe(LANGUAGE.debuff.sound);
  expect(waveSound(step(3), log, 0)?.key).toBe(LANGUAGE.status.sound);
  expect(waveSound(step(4), log, 0)?.key).toBe(LANGUAGE.death.sound);
});

test("the big hit is trace.ts's", () => expect(BIG_HIT).toBe(BIG_HIT_MIN));

test("a wave's changes become one effect each: damage, blocked, heal, buff, debuff, status, death, summon", () => {
  const log = logOf(
    { type: "Hurt", unit: "b1", amount: 5, source: "kernel", causedBy: null },
    { type: "Hurt", unit: "b1", amount: 0, absorbed: 3, source: "kernel", causedBy: null },
    { type: "Hurt", unit: "b2", amount: 2, absorbed: 1, source: "kernel", causedBy: null },
    { type: "Heal", unit: "a1", amount: 3 },
    { type: "StatChanged", unit: "a1", stat: "hp", delta: 2, now: 6, hpAfter: 6 },
    { type: "StatChanged", unit: "a2", stat: "pwr", delta: -1, now: 1 },
    { type: "StatusApplied", unit: "b3", status: "Poison", stacks: 2, total: 2 },
    { type: "Death", unit: "b1" },
    { type: "Summon", unit: "a9", name: "Imp", side: "A", hp: 1, pwr: 1, resurrected: true },
  );
  const fx = effectsOf(changesOf(log), log);
  expect(fx.map((f) => `${f.kind}:${f.unit}:${f.text ?? "-"}${f.stat ? `:${f.stat}` : ""}${f.big ? ":big" : ""}`)).toEqual([
    "damage:b1:−5:big",
    "blocked:b1:0",
    "damage:b2:−2",
    "blocked:b2:-",
    "heal:a1:+3",
    "buff:a1:-:hp",
    "debuff:a2:-:pwr",
    "status:b3:×2",
    "death:b1:-",
    "summon:a9:returns",
  ]);
});

test("the turn's net stat change per unit keeps a digit gold or grey until the turn ends", () => {
  const log = logOf(
    { type: "StatChanged", unit: "a1", stat: "pwr", delta: 2, now: 3 },
    { type: "StatChanged", unit: "a1", stat: "pwr", delta: -1, now: 2 },
    { type: "StatChanged", unit: "a1", stat: "hp", delta: -3, now: 1, hpAfter: 1 },
    { type: "Heal", unit: "a1", amount: 2 },
  );
  expect([...statNets([0, 1, 2, 3], log)]).toEqual([["a1", { pwr: 1, hp: -3 }]]);
  expect([...statNets([], log)]).toEqual([]);
});
