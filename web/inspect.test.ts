// unitDefs tests — the inspector's id → UnitDef resolution. Roster units map
// by line order; a summoned unit's def is recovered from the summon effect on
// its source ability, so a mid-battle arrival can still show its abilities.

import { describe, expect, test } from "vitest";
import { Imp, Summoner, Venomancer, battle, stressRegistry, type UnitDef } from "../src/index.js";
import { chipsHtml, renderUnitInspect, unitDefs } from "./inspect.js";

const dummy = (name: string, hp = 10, pwr = 3): UnitDef => ({ name, base: { hp, pwr } });

describe("unitDefs", () => {
  test("roster units map to their defs by line order", () => {
    const teams = { A: [dummy("One"), dummy("Two")], B: [dummy("Three")] };
    const log = battle({ teamA: teams.A, teamB: teams.B, seed: 0 });
    const defs = unitDefs(log, teams, stressRegistry);
    expect(defs.get("A1:One")).toBe(teams.A[0]);
    expect(defs.get("A2:Two")).toBe(teams.A[1]);
    expect(defs.get("B1:Three")).toBe(teams.B[0]);
  });

  test("a summoned unit's def is recovered from the summoning ability", () => {
    const teams = { A: [Summoner], B: [dummy("Bruiser", 20, 7)] };
    const log = battle({ teamA: teams.A, teamB: teams.B, seed: 0, statuses: stressRegistry });
    const summon = log.find((e) => e.type === "Summon");
    expect(summon).toBeDefined();
    const defs = unitDefs(log, teams, stressRegistry);
    expect(defs.get((summon as { unit: string }).unit)?.name).toBe(Imp.name);
  });
});

// renderUnitInspect only writes innerHTML, so a bare object stands in for the
// panel element — no DOM needed to assert what the inspector says.
const fakeRoot = (): HTMLElement => ({ innerHTML: "" }) as HTMLElement;

describe("renderUnitInspect status refs", () => {
  test("a status name in an ability sentence renders as a tappable ref with a hidden definition row", () => {
    const root = fakeRoot();
    renderUnitInspect(root, {
      title: Venomancer.name,
      state: "10 hp · 2 pwr",
      def: Venomancer,
      statuses: [],
      registry: stressRegistry,
    });
    expect(root.innerHTML).toContain('data-status-ref="Poison"');
    expect(root.innerHTML).toContain('class="ins-ref"');
    // The definition is in the same panel, hidden until the ref is tapped.
    expect(root.innerHTML).toContain('data-status-def="Poison" hidden');
    // The Poison definition's sentence is present — its Part terms are now
    // tappable codex links (#078 slice 3), so the phrase spans anchors: the
    // effect verb and the selector noun each carry their own Part link.
    expect(root.innerHTML).toContain("deal damage equal to its stacks to ");
    expect(root.innerHTML).toContain('href="#codex/part/effect/damage"');
    expect(root.innerHTML).toContain('href="#codex/part/selector/holder"');
    expect(root.innerHTML).toContain(">the holder</a>");
  });

  test("a carried status renders its row (no duplicate hidden definition), unknown names stay plain", () => {
    const root = fakeRoot();
    renderUnitInspect(root, {
      title: Venomancer.name,
      state: "10 hp · 2 pwr",
      def: Venomancer,
      statuses: [{ status: "Poison", stacks: 2 }],
      registry: stressRegistry,
    });
    expect(root.innerHTML).toContain('data-status-row="Poison"');
    expect(root.innerHTML).not.toContain('data-status-def="Poison"');
  });

  test("a status the registry cannot resolve is not tappable", () => {
    const root = fakeRoot();
    renderUnitInspect(root, {
      title: "Mystery",
      state: "1 hp · 1 pwr",
      def: {
        name: "Mystery",
        base: { hp: 1, pwr: 1 },
        abilities: [
          {
            whens: [{ kind: "trigger", on: { on: "TurnEnd" } }],
            selectors: [{ kind: "holder" }],
            effects: [{ kind: "applyStatus", status: "Unregistered", stacks: { kind: "const", value: 1 } }],
          },
        ],
      },
      statuses: [],
      registry: stressRegistry,
    });
    expect(root.innerHTML).toContain("Unregistered");
    expect(root.innerHTML).not.toContain("data-status-ref");
    expect(root.innerHTML).not.toContain("data-status-def");
    // An unresolved status name stays plain — but its surrounding Part terms
    // (the trigger, the effect, the selector) are still tappable codex links.
    expect(root.innerHTML).toContain('href="#codex/part/trigger/TurnEnd"');
    expect(root.innerHTML).toContain('href="#codex/part/effect/applyStatus"');
  });

  test("every Part term in an ability sentence renders as a codex Part link (#078 slice 3)", () => {
    const root = fakeRoot();
    renderUnitInspect(root, {
      title: Venomancer.name,
      state: "10 hp · 2 pwr",
      def: Venomancer,
      statuses: [],
      registry: stressRegistry,
    });
    // Venomancer: "After this unit strikes: apply 2 Poison to the front enemy."
    // The trigger, the applyStatus effect, and the frontEnemy selector each link
    // to their Part card; Poison stays a status ref (the in-panel reveal wins).
    expect(root.innerHTML).toContain('href="#codex/part/trigger/Strike"');
    expect(root.innerHTML).toContain('href="#codex/part/effect/applyStatus"');
    expect(root.innerHTML).toContain('href="#codex/part/selector/frontEnemy"');
    expect(root.innerHTML).toContain('data-status-ref="Poison"');
  });
});

describe("chipsHtml", () => {
  test("the chip title carries the derived definition, not just name×count", () => {
    const html = chipsHtml([{ status: "Poison", stacks: 2 }], stressRegistry);
    expect(html).toContain("Poi2");
    expect(html).toContain("Poison ×2 — At the end of each turn:");
  });

  test("an unknown status falls back to name×count", () => {
    const html = chipsHtml([{ status: "Mystery", stacks: 1 }], stressRegistry);
    expect(html).toContain('title="Mystery ×1"');
  });
});
