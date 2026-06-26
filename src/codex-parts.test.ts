// Codex Part-coverage acceptance (#078 slice 2).
// The codex must emit a card for EVERY registry Part atom — every Trigger,
// Interceptor, Condition, Selector, Effect the type space defines — plus the
// Status and Unit cards covered by codex.test.ts. Coverage is DERIVED from the
// type space (src/parts.ts), not a hand-maintained list, so a Part kind added
// to types.ts gets a card with no edit here.
//
// This test is a SECOND witness to the union: it lists every kind the DSL
// declares (types.ts) and asserts the codex carries a Part card for each. If a
// Part's card were stubbed out of buildCodex/partAtoms, the matching expectation
// below fails (must-fail-first). The union lists here and the sample records in
// parts.ts are kept in lockstep by the TS exhaustiveness guard in parts.ts —
// the records are typed `Record<…["kind"], …>`, so neither can silently drop a
// kind without a compile error.

import { describe, it, expect } from "vitest";
import { buildCodex, codexUnits } from "./codex.js";
import { partAtoms } from "./parts.js";
import { stressRegistry } from "./content/stress.js";
import type { Condition, Effect, EventPattern, Selector } from "./types.js";

const codex = buildCodex(stressRegistry, codexUnits());

// The full Part vocabulary, enumerated straight off the type space (types.ts).
// A `satisfies` keeps each array honest: every member must be a real union
// discriminant, so a typo or a removed kind is a compile error here too.
const EFFECT_KINDS = [
  "damage",
  "heal",
  "applyStatus",
  "consumeStacks",
  "summon",
  "silence",
  "resurrect",
  "cancel",
  "absorbHurt",
  "preventDeathHeal",
] as const satisfies readonly Effect["kind"][];

const SELECTOR_KINDS = [
  "holder",
  "eventUnit",
  "frontEnemy",
  "allEnemies",
  "allAllies",
  "randomEnemy",
  "lastDeadAlly",
] as const satisfies readonly Selector["kind"][];

const CONDITION_KINDS = ["holderHpAtMost"] as const satisfies readonly Condition["kind"][];

const EVENT_PATTERN_TAGS = [
  "BattleStart",
  "TurnStart",
  "TurnEnd",
  "Strike",
  "Hurt",
  "Heal",
  "Death",
  "Summon",
  "StatusApplied",
  "StatusRemoved",
] as const satisfies readonly EventPattern["on"][];

// Interceptor context: only proposable events (not the kernel lifecycle beats).
const INTERCEPTABLE_TAGS = [
  "Strike",
  "Hurt",
  "Heal",
  "Death",
  "Summon",
  "StatusApplied",
  "StatusRemoved",
] as const satisfies readonly EventPattern["on"][];

/** True iff the codex carries a Part card of this family + kind. */
const hasPart = (family: string, kind: string): boolean =>
  codex.parts.some((p) => p.family === family && p.kind === kind);

describe("codex — Part coverage is complete over the type space (#078)", () => {
  it("emits a Trigger card for every event pattern", () => {
    for (const tag of EVENT_PATTERN_TAGS) {
      expect(hasPart("trigger", tag), `trigger ${tag} has no card`).toBe(true);
    }
  });

  it("emits an Interceptor card for every proposable event pattern", () => {
    for (const tag of INTERCEPTABLE_TAGS) {
      expect(hasPart("interceptor", tag), `interceptor ${tag} has no card`).toBe(true);
    }
  });

  it("emits a Condition card for every condition kind", () => {
    for (const kind of CONDITION_KINDS) {
      expect(hasPart("condition", kind), `condition ${kind} has no card`).toBe(true);
    }
  });

  it("emits a Selector card for every selector kind", () => {
    for (const kind of SELECTOR_KINDS) {
      expect(hasPart("selector", kind), `selector ${kind} has no card`).toBe(true);
    }
  });

  it("emits an Effect card for every effect kind", () => {
    for (const kind of EFFECT_KINDS) {
      expect(hasPart("effect", kind), `effect ${kind} has no card`).toBe(true);
    }
  });

  it("has NO Part card beyond the type space (no hand-maintained extras)", () => {
    const expected = new Set<string>([
      ...EVENT_PATTERN_TAGS.map((t) => `trigger:${t}`),
      ...INTERCEPTABLE_TAGS.map((t) => `interceptor:${t}`),
      ...CONDITION_KINDS.map((k) => `condition:${k}`),
      ...SELECTOR_KINDS.map((k) => `selector:${k}`),
      ...EFFECT_KINDS.map((k) => `effect:${k}`),
    ]);
    const actual = new Set(codex.parts.map((p) => `${p.family}:${p.kind}`));
    expect(actual).toEqual(expected);
  });

  it("the exact card count is the size of the Part vocabulary", () => {
    const expectedCount =
      EVENT_PATTERN_TAGS.length +
      INTERCEPTABLE_TAGS.length +
      CONDITION_KINDS.length +
      SELECTOR_KINDS.length +
      EFFECT_KINDS.length;
    expect(codex.parts.length).toBe(expectedCount);
    // partAtoms() (the derivation buildCodex uses) agrees with the codex.
    expect(partAtoms().length).toBe(expectedCount);
  });

  it("every Part card has a name and a non-empty derived meaning", () => {
    for (const p of codex.parts) {
      expect(p.name.length, `${p.family}:${p.kind} name empty`).toBeGreaterThan(0);
      expect(p.meaning.length, `${p.family}:${p.kind} meaning empty`).toBeGreaterThan(0);
    }
  });
});
