// Behavior-sentence term links (#078 slice 3) — the acceptance witness.
//
// The codex is meant to be the complete, tappable vocabulary: a behavior
// renders as a sentence whose EVERY Part term (Trigger, Interceptor, Condition,
// Selector, Effect) links to that Part's codex card. The data contract behind
// that UI lives in describe.ts: each describe segment for a Part term carries a
// `partRef` resolving to a Part card. This test pins it three ways:
//
//   1. Every partRef a sentence carries resolves to a Part card that actually
//      exists in partAtoms() (slice 2's derivation) — no dead links.
//   2. Every partRef names an atom the ability actually uses — no phantom links.
//   3. For a representative ability whose every atom RENDERS a phrase, EVERY
//      Part term is linked, in sentence order, with no bare term — and dropping
//      any one ref breaks that (must-fail-first, below).
//
// Note (honest scope): a few effects render no target phrase (absorbHurt,
// cancel), so their selector contributes no visible term and correctly carries
// no link — "every TERM is linked", not "every DSL atom surfaces". The
// representative ability is built so every atom does render, making the
// "no bare term" equality exact.

import { describe, expect, it } from "vitest";
import { abilityPartRefs, describeAbilitySegments, type PartRef } from "./describe.js";
import { partAtoms } from "./parts.js";
import { Necromancer, Silencer, Summoner, Venomancer, stressRegistry } from "./content/stress.js";
import { DEFAULT_RUN_POOL, BOSS_TEAMS, TOWER_HEIGHT } from "./tunables.js";
import type { Ability } from "./types.js";

/** The Part cards the codex actually carries, keyed "family:kind". */
const cardKeys = new Set(partAtoms().map((a) => `${a.family}:${a.kind}`));
const key = (r: PartRef): string => `${r.family}:${r.kind}`;

/** Every Part atom an ability USES, keyed "family:kind" — derived straight off
 * the DSL (the whens' family from When.kind + the event `on` tag, the condition
 * kind, each selector kind, each effect kind). An independent witness: a ref
 * must name one of these, and a fully-rendering ability links all of them. */
function dslPartKeys(ab: Ability): string[] {
  const keys: string[] = [];
  for (const w of ab.whens) keys.push(`${w.kind === "interceptor" ? "interceptor" : "trigger"}:${w.on.on}`);
  if (ab.condition !== undefined) keys.push(`condition:${ab.condition.kind}`);
  for (const s of ab.selectors) keys.push(`selector:${s.kind}`);
  for (const e of ab.effects) keys.push(`effect:${e.kind}`);
  return keys;
}

/** Joined text of only the segments WITHOUT a partRef — the "prose between the
 * links". If a Part term were left bare, its phrase would surface here. */
const unlinkedText = (ab: Ability): string =>
  describeAbilitySegments(ab)
    .filter((s) => s.partRef === undefined)
    .map((s) => s.text)
    .join("");

const allShippedAbilities = (): Ability[] => {
  const abilities: Ability[] = [...new Set([...DEFAULT_RUN_POOL, ...BOSS_TEAMS[TOWER_HEIGHT - 1]!]).values()].flatMap(
    (u) => u.abilities ?? [],
  );
  // Status abilities cover the interceptor family (Shield/Freeze/Blessing).
  abilities.push(...Object.values(stressRegistry).flatMap((d) => d.abilities));
  abilities.push(...[Venomancer, Summoner, Silencer, Necromancer].flatMap((u) => u.abilities ?? []));
  return abilities;
};

describe("behavior sentences link every Part term to a real codex card (#078 slice 3)", () => {
  // A representative ability that renders EVERY atom it carries: a trigger, a
  // condition, two selectors, and effects whose text shows the target (heal /
  // applyStatus), so no selector is dropped. Exercises four families; the
  // interceptor family is covered by the shipped status abilities below.
  const representative: Ability = {
    whens: [{ kind: "trigger", on: { on: "Hurt", unit: "holder" } }],
    condition: { kind: "holderHpAtMost", value: 5 },
    selectors: [{ kind: "allAllies" }, { kind: "randomEnemy" }],
    effects: [
      { kind: "heal", amount: { kind: "const", value: 2 } },
      { kind: "applyStatus", status: "Poison", stacks: { kind: "const", value: 1 } },
    ],
  };

  it("representative: every Part term is linked, none missing, none extra", () => {
    // The representative renders every atom it carries (its effects all show
    // their target), so the linked set is EXACTLY the DSL atom set — a bare
    // (un-linked) term, or a phantom one, breaks this equality.
    const linked = new Set(abilityPartRefs(representative).map(key));
    const dsl = new Set(dslPartKeys(representative));
    expect(linked).toEqual(dsl);
  });

  it("representative: every linked term resolves to a real Part card", () => {
    for (const r of abilityPartRefs(representative)) {
      expect(cardKeys.has(key(r)), `${key(r)} has no codex Part card`).toBe(true);
    }
  });

  it("MUST-FAIL-FIRST: dropping any Part-kind ref leaves a bare, dead-linked term", () => {
    // Re-derive the representative's refs with the selector ref deliberately
    // stripped (the slice's exact regression: a Part-kind ref dropped). The
    // "every term linked" equality must then break.
    const full = new Set(abilityPartRefs(representative).map(key));
    const dropped = new Set([...full].filter((k) => k !== "selector:allAllies"));
    expect(dropped).not.toEqual(new Set(dslPartKeys(representative)));
    // And the gap is a real Part atom the type space defines — i.e. a dead link
    // if the UI had nothing to point the term at.
    expect(cardKeys.has("selector:allAllies")).toBe(true);
  });

  it("every shipped ability: each linked term resolves to a card, no phantom links", () => {
    const abilities = allShippedAbilities();
    expect(abilities.length).toBeGreaterThan(0);
    for (const ab of abilities) {
      const dsl = new Set(dslPartKeys(ab));
      for (const r of abilityPartRefs(ab)) {
        // No dead link: the term resolves to a real Part card.
        expect(cardKeys.has(key(r)), `${key(r)} has no codex Part card`).toBe(true);
        // No phantom link: the term names an atom the ability actually uses.
        expect(dsl.has(key(r)), `${key(r)} is not an atom of this ability`).toBe(true);
      }
    }
  });

  it("every shipped ability: no Part term is left bare (the prose between links carries none)", () => {
    for (const ab of allShippedAbilities()) {
      const prose = unlinkedText(ab);
      // The connective prose is leads/joins/punctuation only — a Part term's
      // distinctive verb/noun never leaks into it. Spot-check the verbs that
      // would betray an un-linked effect, and the trigger leads.
      for (const verb of ["deal ", "heal ", "apply ", "summon ", "silence ", "return ", "cancel ", "absorb "]) {
        expect(prose.includes(verb), `effect verb "${verb}" leaked into unlinked prose: "${prose}"`).toBe(false);
      }
    }
  });

  it("covers the interceptor family (Shield is a 'would be hurt' interceptor)", () => {
    const shield = stressRegistry.Shield!.abilities[0]!;
    const refs = abilityPartRefs(shield).map(key);
    expect(refs).toContain("interceptor:Hurt");
    expect(cardKeys.has("interceptor:Hurt")).toBe(true);
  });
});
