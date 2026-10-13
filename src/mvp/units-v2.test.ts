// The rules v2 pool (#1046): the eight units build into content that
// validates, and their lines fight through the MVP's own path.

import { describe, expect, it } from "vitest";
import { contentOf } from "../../server/src/mvp/content.js";
import { contentFormProblems, lineUnitOf } from "./forms.js";
import { fightLines, toBattleDef } from "./fight.js";
import { MVP_RULES } from "./contract.js";
import { ROWS_V2 } from "./units-v2.js";
import { traceLines } from "../words/trace.js";
import { formSegments } from "./form-text.js";

const content = contentOf(ROWS_V2);

describe("rules v2 pool", () => {
  it("the eight §4 units build into content with no form problems", () => {
    expect(content.units.map((u) => u.name).slice(0, 8)).toEqual(["Hourglass", "Mirror", "Candle", "Gambler", "Nautilus", "Magnet", "Raven", "Seed"]);
    expect(content.units).toHaveLength(20);
    expect(new Set(content.units.map((u) => u.tier))).toEqual(new Set([1, 2, 3, 4]));
    expect(contentFormProblems(content)).toEqual([]);
    expect(Object.keys(content.statuses)).toEqual(expect.arrayContaining(["Grains", "Sands", "Memory", "Memories", "Shield", "Strength"]));
  });

  it("every unit of the batch fights, sleeping and awoken, and says both forms in EN and RU", () => {
    const p = { id: "bot", name: "bot", bot: true };
    for (const [i, u] of content.units.entries()) {
      const mate = content.units[(i + 3) % content.units.length]!;
      const foes = [content.units[(i + 7) % content.units.length]!, content.units[(i + 11) % content.units.length]!, content.units[(i + 13) % content.units.length]!];
      for (const copies of [1, 3]) {
        const a = [lineUnitOf(u, "a1", copies), lineUnitOf(mate, "a2")];
        const b = foes.map((f, j) => lineUnitOf(f, `b${j}`));
        const rec = fightLines({ player: p, line: a }, { player: p, line: b }, { battleId: `${u.id}-${copies}`, seed: 3, kind: "round", round: 1, runId: null, at: "2026-10-13T00:00:00.000Z", content, rules: MVP_RULES });
        expect(rec.log.length).toBeGreaterThan(2);
      }
      for (const lang of ["en", "ru"] as const) for (const form of ["sleeping", "awoken"] as const) {
        const text = formSegments(u.forms[form], content.abilities, lang).map((s) => s.text).join("");
        expect(text, `${u.name} ${form} ${lang}`).toMatch(/\S/);
        expect(text, `${u.name} ${form} ${lang}`).not.toMatch(/undefined|\[object/);
      }
    }
  });

  it("a v2 line fights the MVP way: Gambler flips, Raven starts with its Memory", () => {
    const unit = (name: string, copies = 1, uid = name) => lineUnitOf(content.units.find((u) => u.name === name)!, uid, copies);
    const a = [unit("Gambler"), unit("Raven", 3)];
    const b = [unit("Nautilus"), unit("Hourglass")];
    expect(toBattleDef(a[1]!).statuses).toEqual([{ status: "Memories", stacks: 1 }]);
    const p = { id: "bot", name: "bot", bot: true };
    const rec = fightLines({ player: p, line: a }, { player: p, line: b }, { battleId: "x", seed: 1, kind: "round", round: 1, runId: null, at: "2026-10-13T00:00:00.000Z", content, rules: MVP_RULES });
    const lines = traceLines({ teamA: a.map(toBattleDef), teamB: b.map(toBattleDef), seed: 1, statuses: content.statuses, abilities: content.abilities }, rec.log);
    expect(lines.some((l) => /^Gambler flips: (heads|tails)$/.test(l))).toBe(true);
  });
});
