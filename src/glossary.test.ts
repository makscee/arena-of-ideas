// The glossary covers every term a player reads (round 2, R2-6): every
// highlighted run of all 81 units' texts, both forms, and a sample of
// fusions, resolves to a glossary entry; no keyword hides in glue text; the
// runs join to the sentence; and every icon the glossary names ships in
// mobile/icons/ with its credit.

import { readFileSync, readdirSync } from "node:fs";
import { describe, expect, test } from "vitest";
import { describeAbility, describeStatus, describeStatusSegments, type DescribeSegment } from "./describe.js";
import { GLOSSARY, ICON_IDS, STATUS_TERMS, termDef, termIcon, type TermId } from "./glossary.js";
import type { UnitForm } from "./mvp/contract.js";
import { formSegments, formText } from "./mvp/form-text.js";
import { mvpPool } from "./mvp/units.js";

const pool = mvpPool();
const join = (segs: DescribeSegment[]): string => segs.map((s) => s.text).join("");

/** A fusion's recipe, as fuseUnits builds it: the When of the first, the Who
 * of the second, the Does of both. */
const fuse = (a: UnitForm, b: UnitForm): UnitForm => ({
  when: a.when,
  ...(a.condition ? { condition: a.condition } : {}),
  who: b.who,
  does: [...a.does, ...b.does],
});

const forms: [string, UnitForm][] = pool.units.flatMap((u) => [
  [`${u.name} (sleeping)`, u.forms.sleeping] as [string, UnitForm],
  [`${u.name} (awoken)`, u.forms.awoken] as [string, UnitForm],
]);
// A spread of ordered pairs: every unit fused once as the first part, with a
// partner far down the pool, so every When meets a different Who.
const n = pool.units.length;
const fusions: [string, UnitForm][] = pool.units.map((a, i) => {
  const b = pool.units[(i * 13 + 5) % n]!;
  return [`${a.name} + ${b.name}`, fuse(a.forms.awoken, b.forms.awoken)];
});

/** Words that are glossary terms: they must sit in a tagged run, never in
 * glue text. */
const KEYWORDS = [
  ...Object.keys(pool.statuses),
  "PWR", "HP", "stack", "damage", "heal", "summon", "silence", "revive",
  "hit", "strikes", "dies", "healed", "summoned", "lands", "gains", "battle begins", "each turn", "would",
];
const keywordIn = (text: string): string | undefined =>
  KEYWORDS.find((k) => new RegExp(`\\b${k}`, "i").test(text));

function checkSegments(label: string, segs: DescribeSegment[]): void {
  for (const s of segs) {
    expect(s.text, `${label}: an empty run`).not.toBe("");
    if (s.term === undefined) {
      expect(keywordIn(s.text), `${label}: "${s.text}" holds a keyword but no term`).toBeUndefined();
      expect(s.statusRef, `${label}: a status run without a term`).toBeUndefined();
    } else {
      expect(termDef(s.term, pool.statuses), `${label}: ${s.term} has no glossary entry`).toBeDefined();
    }
    if (s.statusRef !== undefined) expect(s.term).toBe(`status:${s.statusRef}`);
  }
}

describe("glossary covers every unit's text", () => {
  test("there are 81 units, both forms described", () => {
    expect(pool.units).toHaveLength(81);
  });

  test.each(forms)("%s", (label, form) => {
    const segs = formSegments(form, pool.abilities);
    checkSegments(label, segs);
    expect(join(segs)).toBe(formText(form, pool.abilities));
  });

  test.each(fusions)("fusion %s", (label, form) => {
    checkSegments(label, formSegments(form, pool.abilities));
  });

  test("the runs join to the plain sentence describeAbility gives", () => {
    for (const [, form] of forms) {
      if (form.does.length !== 1) continue;
      const ab = pool.abilities[form.does[0]!]!;
      expect(join(formSegments(form, pool.abilities))).toBe(describeAbility({ ...ab, whens: form.when, selectors: form.who }));
    }
  });

  test.each(Object.entries(pool.statuses))("status %s", (name, def) => {
    const segs = describeStatusSegments(def);
    checkSegments(name, segs);
    expect(join(segs)).toBe(describeStatus(def));
    expect(termDef(`status:${name}`, pool.statuses)?.icon).toBeDefined();
  });
});

describe("amounts (R2-8)", () => {
  test("a constant damage or heal number is its own run, marked as the amount", () => {
    for (const [name, form] of forms) {
      const segs = formSegments(form, pool.abilities);
      for (const s of segs.filter((x) => x.amount)) {
        expect(s.text, name).toMatch(/^\d+$/);
        expect(["effect:damage", "effect:heal"], name).toContain(s.term);
      }
      // No number hides inside a damage or heal verb run.
      for (const s of segs.filter((x) => (x.term === "effect:damage" || x.term === "effect:heal") && !x.amount)) expect(s.text, name).not.toMatch(/\d/);
    }
  });

  test("Damage reads deal [2] damage", () => {
    const fighter = pool.units.find((u) => formText(u.forms.sleeping, pool.abilities).includes(" damage to "))!;
    const segs = formSegments(fighter.forms.sleeping, pool.abilities);
    const i = segs.findIndex((s) => s.amount);
    expect(segs[i]!.term).toBe("effect:damage");
    expect(segs[i + 2]).toMatchObject({ text: "damage", term: "effect:damage" });
  });
});

describe("wording", () => {
  const all = [...forms, ...fusions].map(([, f]) => formText(f, pool.abilities)).join("\n");

  test("no text promises lost HP for a hit Shield may block, or names the event's unit", () => {
    expect(all).not.toMatch(/is hurt|be hurt|event's unit|most recently dead/);
    expect(all).toMatch(/After this unit is hit:/);
    expect(all).toMatch(/heal that ally for 1/);
  });

  test("stats read PWR and HP, never lower case", () => {
    expect(all).not.toMatch(/\b(pwr|hp)\b/);
  });

  test("silence, revive and summon read short", () => {
    expect(all).toMatch(/silence the front enemy\./);
    expect(all).toMatch(/revive the last fallen ally at 2 HP\./);
    expect(all).toMatch(/summon an Imp \(1\/2\)/);
    expect(all).not.toMatch(/strip its statuses|back of this unit's side/);
  });

  test("a trigger clause marks every run, a target its side", () => {
    const spike = pool.units.find((u) => u.name === "Spike")!;
    const segs = formSegments(spike.forms.sleeping, pool.abilities);
    expect(segs.filter((s) => s.clause === "when").map((s) => [s.text, s.term])).toEqual([
      ["After ", "trigger:StatusApplied"],
      ["Shield", "status:Shield"],
      [" lands on an ally", "trigger:StatusApplied"],
    ]);
    expect(segs.find((s) => s.term === "target:frontEnemy")?.side).toBe("enemy");
    expect(segs.filter((s) => s.term === "effect:damage").map((s) => [s.text, s.amount ?? false])).toEqual([["1", true], ["damage", false]]);
    const nurse = pool.units.find((u) => u.name === "Nurse")!;
    expect(formSegments(nurse.forms.sleeping, pool.abilities).find((s) => s.term === "target:eventUnit")).toMatchObject({
      text: "that ally",
      side: "ally",
    });
    const shield = describeStatusSegments(pool.statuses.Shield!);
    expect(shield.find((s) => s.term === "term:would")?.text).toBe("would");
  });
});

describe("glossary entries and icons", () => {
  const entries = [...Object.entries(GLOSSARY), ...Object.entries(STATUS_TERMS).map(([k, v]) => [`status:${k}`, v] as const)];
  const iconDir = new URL("../mobile/icons/", import.meta.url);

  test("every entry has a label and a one-line tip", () => {
    for (const [id, def] of entries) {
      expect(def.label, id).not.toBe("");
      expect(def.tip, id).toMatch(/\S/);
      expect(def.tip, id).not.toMatch(/\n/);
    }
  });

  test("every shipped status has its entry", () => {
    expect(Object.keys(STATUS_TERMS).sort()).toEqual(Object.keys(pool.statuses).sort());
  });

  test("a status nobody listed takes its tip from its definition", () => {
    const def = { name: "Thorns", abilities: [] };
    expect(termDef("status:Thorns", { Thorns: def })).toMatchObject({ label: "Thorns", tip: describeStatus(def) });
    expect(termDef("status:Nope" as TermId)).toBeUndefined();
  });

  test("every icon named ships as a currentColor SVG with its credit", () => {
    const files = readdirSync(iconDir).filter((f) => f.endsWith(".svg")).map((f) => f.slice(0, -4)).sort();
    expect(files).toEqual([...ICON_IDS].sort());
    const credits = readFileSync(new URL("CREDITS.txt", iconDir), "utf8");
    for (const id of ICON_IDS) {
      expect(readFileSync(new URL(`${id}.svg`, iconDir), "utf8"), id).toMatch(/fill="currentColor"/);
      expect(credits, id).toMatch(new RegExp(`^${id}\\s`, "m"));
    }
    for (const [id, def] of entries) if (def.icon) expect(ICON_IDS, id).toContain(def.icon);
  });

  test("a status-lands trigger shows the status's own icon", () => {
    expect(termIcon("trigger:StatusApplied", "Shield")).toBe("shield");
    expect(termIcon("trigger:Hurt")).toBe("broken-heart");
  });
});
