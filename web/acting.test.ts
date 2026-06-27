// #082 slice D — the acting-card model: the centre card derives the acting
// unit, its RESULT effects, and the reactive CHAINS straight from the kernel
// log's structured `source`/`causedBy`, never from narrated text. Side-card
// state (ACTING/TARGET/USED) and the per-beat trace strip come off the same
// projection. These tests pin the mapping a DOM probe can't see cheaply.

import { describe, expect, test } from "vitest";
import {
  battle,
  beatsOf,
  displayNames,
  type AbilityDef,
  type AbilityRegistry,
  type Beat,
  type BattleEvent,
  type UnitDef,
} from "../src/index.js";
import { Poison, Strength, stressRegistry } from "../src/content/stress.js";
import {
  actingCardHtml,
  actingModelAt,
  actingUnitAt,
  traceChipsAt,
  traceStripHtml,
  usedThisTurnAt,
  type ActingCtx,
} from "./acting.js";
import { familySigil } from "./unit-card.js";
import { battleHtml, type BattleAnnotations } from "./board-render.js";
import { boardAt } from "../src/index.js";

// Venomancer (Poison family): on strike, poisons the front enemy.
const Venom: AbilityDef = {
  name: "Toxic Strike",
  family: "Poison",
  whens: [{ kind: "trigger", on: { on: "Strike", striker: "holder" } }],
  selectors: [{ kind: "frontEnemy" }],
  effects: [{ kind: "applyStatus", status: "Poison", stacks: { kind: "const", value: 2 } }],
};
// Brawler (Strike family): on being damaged, gains Strength (the reactive CHAIN).
const Enrage: AbilityDef = {
  name: "Enrage",
  family: "Strike",
  whens: [{ kind: "trigger", on: { on: "Hurt", unit: "holder" } }],
  selectors: [{ kind: "holder" }],
  effects: [{ kind: "applyStatus", status: "Strength", stacks: { kind: "const", value: 1 } }],
};
const abilities: AbilityRegistry = { Toxic_Strike: Venom, Enrage, Strike: { name: "Strike", family: "Strike", whens: [{ kind: "trigger", on: { on: "BattleStart" } }], selectors: [{ kind: "holder" }], effects: [{ kind: "heal", amount: { kind: "const", value: 0 } }] } };
const registry = { ...stressRegistry, Poison, Strength };

const Venomancer: UnitDef = { name: "Venomancer", base: { hp: 6, pwr: 1 }, ability: "Toxic_Strike" };
const Brawler: UnitDef = { name: "Brawler", base: { hp: 12, pwr: 1 }, ability: "Enrage" };

function run(): { log: BattleEvent[]; beats: Beat[]; ctx: ActingCtx } {
  const log = battle({ teamA: [Venomancer], teamB: [Brawler], seed: 1, statuses: registry, abilities });
  const beats = beatsOf(log);
  const name = displayNames(log);
  const defs = new Map<string, UnitDef>([
    ["A1:Venomancer", Venomancer],
    ["B1:Brawler", Brawler],
  ]);
  const sideOf = (id: string): "A" | "B" | undefined =>
    id.startsWith("A") ? "A" : id.startsWith("B") ? "B" : undefined;
  return { log, beats, ctx: { defs, abilities, registry, name, sideOf } };
}

/** The step (beat.end) where Venomancer strikes Brawler. */
function venomBeat(beats: Beat[], ctx: ActingCtx): Beat {
  const beat = beats.find((b) => b.root.type === "Strike" && ctx.name((b.root as { striker: string }).striker) === "Venomancer");
  if (!beat) throw new Error("no Venomancer strike beat");
  return beat;
}

describe("actingModelAt", () => {
  test("an acting step yields a card for the striker with its named ability", () => {
    const { log, beats, ctx } = run();
    const beat = venomBeat(beats, ctx);
    const m = actingModelAt(log, beats, beat.end, ctx);
    expect(m.kind).toBe("card");
    expect(m.acting?.name).toBe("Venomancer");
    expect(m.acting?.family).toBe("Poison");
    expect(m.acting?.abilityLabel).toBe("Toxic Strike");
    // The NOW chip names the live target and the action.
    expect(m.acting?.now.targetName).toBe("Brawler");
    expect(m.acting?.now.action).toContain("Poison");
  });

  test("RESULT lists the step's DIRECT effects (the strike's hurt + the poison)", () => {
    const { log, beats, ctx } = run();
    const beat = venomBeat(beats, ctx);
    const m = actingModelAt(log, beats, beat.end, ctx);
    const text = m.result.map((r) => r.html).join(" | ");
    expect(text).toMatch(/takes/); // the kernel Hurt on Brawler
    expect(text).toMatch(/Poison 2/); // Venomancer's own ability — a DIRECT effect
    // The reactive Strength is NOT in RESULT (it's a chain).
    expect(text).not.toMatch(/Strength/);
  });

  test("CHAINS box appears only when a different unit's ability fired", () => {
    const { log, beats, ctx } = run();
    const beat = venomBeat(beats, ctx);
    const m = actingModelAt(log, beats, beat.end, ctx);
    expect(m.chains.length).toBe(1);
    expect(m.chains[0]!.unit).toBe("Brawler");
    expect(m.chains[0]!.trigger).toBe("ON DAMAGED");
    expect(m.chains[0]!.rows.map((r) => r.html).join(" ")).toMatch(/Strength/);
  });

  test("CHAINS box is absent before the chain event is revealed", () => {
    const { log, beats, ctx } = run();
    const beat = venomBeat(beats, ctx);
    // The Strike root alone (no caused events revealed yet) → no chain.
    const m = actingModelAt(log, beats, beat.start, ctx);
    expect(m.chains.length).toBe(0);
  });

  test("a beat with no actor renders a phase caption, not a card", () => {
    const { log, beats, ctx } = run();
    const m = actingModelAt(log, beats, 0, ctx); // BattleStart
    expect(m.kind).toBe("phase");
    expect(m.caption).toBe("Battle begins");
  });
});

describe("side-card state + trace strip", () => {
  test("the acting unit, its target, and a used unit get the right tags", () => {
    const { log, beats, ctx } = run();
    const beat = venomBeat(beats, ctx);
    const { acting, target } = actingUnitAt(beats, beat.end);
    expect(ctx.name(acting!)).toBe("Venomancer");
    expect(ctx.name(target!)).toBe("Brawler");

    // Render the board markup for this step and assert the ribbons land.
    const used = usedThisTurnAt(log, beats, beat.end);
    const anno: BattleAnnotations = { acting, target, used };
    const html = battleHtml({
      board: boardAt(log, beat.end),
      ctx,
      anno,
      registry,
      centerHtml: "",
      traceHtml: "",
      header: { turn: 1 },
    });
    expect(html).toContain(">ACTING<");
    expect(html).toContain(">TARGET<");
    expect(html).toContain("is-acting");
    expect(html).toContain("is-target");
  });

  test("a unit that already struck this turn is marked USED", () => {
    const { log, beats, ctx } = run();
    // The SECOND strike beat of turn 1: the first striker is now USED.
    const strikes = beats.filter((b) => b.root.type === "Strike" && b.root.turn === 1);
    if (strikes.length >= 2) {
      const second = strikes[1]!;
      const used = usedThisTurnAt(log, beats, second.end);
      expect(used.size).toBeGreaterThanOrEqual(1);
      // The first striker is in the used set.
      const firstStriker = (strikes[0]!.root as { striker: string }).striker;
      expect(used.has(firstStriker)).toBe(true);
    }
  });

  test("the trace strip has one chip per beat, exactly one current", () => {
    const { log, beats, ctx } = run();
    const beat = venomBeat(beats, ctx);
    const chips = traceChipsAt(log, beats, beat.end, ctx);
    expect(chips.length).toBe(beats.length);
    expect(chips.filter((c) => c.current).length).toBe(1);
    expect(chips.find((c) => c.current)!.id).toBe(beat.end);
    // A Strike beat's chip reads "<ABBR> · <UNIT>".
    const venomChip = chips.find((c) => c.primary.includes("VENOMANCER"));
    expect(venomChip).toBeDefined();
    const stripHtml = traceStripHtml(chips);
    expect((stripHtml.match(/class="tr-chip/g) ?? []).length).toBe(beats.length);
    expect((stripHtml.match(/is-cur/g) ?? []).length).toBe(1);
  });
});

describe("actingCardHtml", () => {
  test("renders the NOW chip, RESULT rows, and a CHAINS callout for a chained step", () => {
    const { log, beats, ctx } = run();
    const beat = venomBeat(beats, ctx);
    const m = actingModelAt(log, beats, beat.end, ctx);
    const html = actingCardHtml(m, familySigil(m.acting!.family, m.acting!.hex));
    expect(html).toContain("● NOW");
    expect(html).toContain(">RESULT<");
    expect(html).toContain("↳ CHAINS");
    expect(html).toContain("Venomancer");
    // The trigger/action/effect marks render as inline `currentColor` SVG icons
    // (#086), NOT the unicode glyphs the vendored fonts drop to the wrong char.
    expect(html).toMatch(/class="ac-now-act"><svg class="gly" data-glyph="[a-z-]+"/);
    expect(html).toMatch(/class="ac-g [^"]*"><svg class="gly" data-glyph="[a-z-]+"/);
    // the named-ability star is the inline SVG, not the ✦ font glyph
    expect(html).toContain('class="ac-spark" data-glyph="ability-star"');
    // the CHAINS header's reactive mark is the burst SVG, not the ✸ font glyph
    expect(html).toMatch(/CHAINS ·[\s\S]*?· <svg class="gly" data-glyph="damaged"/);
    // no raw fallback-prone unicode marks leak into the markup
    for (const ch of ["⚔", "☠", "✦", "☣", "◆", "✶", "⛨", "✸"]) expect(html).not.toContain(ch);
  });

  test("a phase step renders a caption, not a card", () => {
    const { log, beats, ctx } = run();
    const m = actingModelAt(log, beats, 0, ctx);
    const html = actingCardHtml(m, "");
    expect(html).toContain("acting-phase");
    expect(html).toContain("Battle begins");
    expect(html).not.toContain("acting-card");
  });
});
