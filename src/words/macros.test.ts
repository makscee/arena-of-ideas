import { describe, expect, it } from "vitest";
import { battle, toJSONL } from "../battle.js";
import { describeAbilitySegments } from "../describe.js";
import { describeAbilitySegmentsRu } from "../describe-ru.js";
import { unitActionsOf } from "../types.js";
import { validateAbilityRegistry } from "../validate.js";
import { FIXTURES, FIXTURE_ABILITIES, FIXTURE_STATUSES } from "./fixtures.js";
import { expandAbilities, expandEffects, MACROS } from "./macros.js";

describe("composed words (#1043 §7)", () => {
  it("echo expands to remember + delay + performRemembered, and only where used", () => {
    expect(expandEffects([{ kind: "macro", name: "echo" }])).toEqual(MACROS.echo!.effects);
    const plain = { Zap: FIXTURE_ABILITIES.Zap! };
    expect(expandAbilities(plain)).toBe(plain);
    const out = expandAbilities(FIXTURE_ABILITIES);
    expect(out).not.toBe(FIXTURE_ABILITIES);
    expect(out.Echo!.effects.map((e) => e.kind)).toEqual(["remember", "delay"]);
    expect(out.Zap).toBe(FIXTURE_ABILITIES.Zap);
    expect(() => expandEffects([{ kind: "macro", name: "nope" }])).toThrow(/unknown macro/);
  });

  it("Déjà Vu through the macro fights the same golden as its primitives", () => {
    const { input } = FIXTURES.dejaVu!;
    const expanded = { ...input, abilities: expandAbilities(input.abilities ?? {}) };
    expect(toJSONL(battle(input))).toBe(toJSONL(battle(expanded)));
    expect(input.abilities!.Echo!.effects[0]).toEqual({ kind: "macro", name: "echo" });
  });

  it("the validator accepts a registered macro and names an unknown one", () => {
    expect(validateAbilityRegistry(FIXTURE_ABILITIES, FIXTURE_STATUSES)).toEqual([]);
    const bad = { Nope: { name: "Nope", family: "Arcane" as const, effects: [{ kind: "macro" as const, name: "steal" }] } };
    const issues = validateAbilityRegistry(bad, FIXTURE_STATUSES);
    expect(issues.map((i) => i.message).join("\n")).toMatch(/unknown composed word "steal"/);
  });

  it("a macro says itself in the card sentence", () => {
    const dejaVu = FIXTURES.dejaVu!.input.teamA[0]!;
    const action = unitActionsOf(dejaVu, FIXTURE_ABILITIES).find((a) => JSON.stringify(a).includes('"macro"'))!;
    expect(describeAbilitySegments(action).map((s) => s.text).join("")).toMatch(/echoes .* a turn later/);
    expect(describeAbilitySegmentsRu(action, {}).map((s) => s.text).join("")).toMatch(/повторяет .* ходом позже/);
  });
});
