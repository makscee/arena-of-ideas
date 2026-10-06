// The words on every card (R3-1): a golden file of all unit texts, both forms,
// and the card-speak rules they follow (words.md (6)).

import { describe, expect, it } from "vitest";
import type { DescribeSegment } from "../describe.js";
import { formSegments, formText } from "./form-text.js";
import { unitTexts } from "./unit-texts.js";
import { mvpPool } from "./units.js";

const pool = mvpPool();
const forms = pool.units.flatMap((u) => [
  [`${u.name}`, u.forms.sleeping] as const,
  [`${u.name} awoken`, u.forms.awoken] as const,
]);

describe("unit texts", () => {
  it("match the golden file (`npm run -s mvp:texts`; update with -u)", async () => {
    await expect(unitTexts(pool)).toMatchFileSnapshot("./unit-texts.golden.txt");
  });

  it("carry no filler", () => {
    // "every ally" where "all allies" is meant, "this unit" where "self" is, …
    const FILLER = [/\bthe\b/i, /this unit/i, /\bevery\b/i, /each turn/i, /\bafter\b/i, /\bwhen\b/i, /another ally/i, /\ban? [A-Z]/, /\bapply\b/i, /\bdeal\b/i, /\bits\b/i];
    for (const [name, form] of forms) {
      const text = formText(form, pool.abilities);
      for (const f of FILLER) expect(text, `${name}: "${text}" has filler ${f}`).not.toMatch(f);
    }
  });

  it("merge with 'and' only effects that share one target", () => {
    let merged = 0;
    for (const [name, form] of forms) {
      const segs = formSegments(form, pool.abilities);
      // The Does part, split into its ", then " steps.
      const does = segs.slice(segs.findIndex((s) => s.text === ": ") + 1);
      const steps: DescribeSegment[][] = [[]];
      for (const s of does) {
        if (s.term === undefined && s.text === ", then ") steps.push([]);
        else steps.at(-1)!.push(s);
      }
      for (const step of steps) {
        // Three or more: commas, then a final "and" ("1 Strength, 1 Vitality and 4 damage to …").
        const ands = step.filter((s, i) => s.term === undefined && s.partRef === undefined && (s.text === " and " || s.text === ", ") && /^(effect|status):/.test(step[i - 1]?.term ?? ""));
        if (ands.length === 0) continue;
        expect(ands.at(-1)!.text, `${name}: the last join is "and"`).toBe(" and ");
        expect(ands.slice(0, -1).every((s) => s.text === ", "), `${name}: the others are commas`).toBe(true);
        merged++;
        const text = step.map((s) => s.text).join("");
        // One target, named once, after the last merged effect.
        expect(step.filter((s) => s.text === " to ").length, `${name}: "${text}"`).toBe(1);
        expect(step.findIndex((s) => s.text === " to "), `${name}: "${text}"`).toBeGreaterThan(step.lastIndexOf(ands.at(-1)!));
        // Each merged part is "amount, word": a number run then a damage or status run.
        expect(step.filter((s) => s.amount).length, `${name}: "${text}"`).toBe(ands.length + 1);
      }
    }
    expect(merged).toBeGreaterThan(10);
  });

  it("join three or more merged effects with commas and a final 'and' (a fusion shows it)", () => {
    const awoken = (name: string) => pool.units.find((u) => u.name === name)!.forms.awoken;
    const [a, b] = [awoken("Fighter"), awoken("Taser")];
    // The When of the first, the Who of the second, the Does of both.
    expect(formText({ when: a.when, who: b.who, does: [...a.does, ...b.does] }, pool.abilities)).toBe("Strikes: 1 damage, 1 Freeze and 2 damage to front enemy.");
  });

  it("read as Maks's examples", () => {
    const text = (name: string, form: "sleeping" | "awoken") => formText(pool.units.find((u) => u.name === name)!.forms[form], pool.abilities);
    expect(text("Medic", "sleeping")).toBe("Turn end: heal all allies for 1.");
    expect(text("Taser", "awoken")).toBe("Battle start: 1 Freeze and 2 damage to front enemy.");
    expect(text("Gardener", "awoken")).toBe("Ally summoned: 2 Vitality and 2 Shield to self.");
    expect(text("Spike", "sleeping")).toBe("Ally gets Shield: 1 damage to front enemy.");
    expect(text("Necromancer", "sleeping")).toBe("Ally dies: revive fallen ally at 2 HP.");
    expect(text("Leech", "awoken")).toBe("Strikes: heal self for PWR, then 1 Shield to self.");
    expect(text("Rose", "sleeping")).toBe("Hit: 2 damage to front enemy.");
    expect(text("Planter", "awoken")).toBe("Battle start: summon Treant (1/8), then 2 Vitality and 2 Shield to self.");
  });
});
