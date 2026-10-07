// The glossary covers every term a player reads (round 2, R2-6): every
// highlighted run of all 81 units' texts, both forms, and a sample of
// fusions, resolves to a glossary entry; no keyword hides in glue text; the
// runs join to the sentence; and every icon the glossary names ships in
// mobile/icons/ with its credit.

import { readFileSync, readdirSync } from "node:fs";
import { describe, expect, test } from "vitest";
import { describeAbility, describeStatus, describeStatusSegments, type DescribeSegment } from "./describe.js";
import { chainCappedTip, timeUpTip, GLOSSARY, ICON_IDS, STATUS_TERMS, scopedLabel, scopedTip, triggerLabel, termDef, termGroup, termIcon, type FixedTermId, type TermId } from "./glossary.js";
import { MVP_RULES } from "./mvp/contract.js";
import type { UnitForm } from "./mvp/contract.js";
import { formSegments, formText } from "./mvp/form-text.js";
import type { UnitFilter } from "./types.js";
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
  "hit", "strikes", "dies", "healed", "summoned", "gets", "gains", "start", "end", "would",
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
  test("a constant damage or heal number, a status's stacks and an HP amount are their own run, marked as the amount", () => {
    for (const [name, form] of forms) {
      const segs = formSegments(form, pool.abilities);
      for (const s of segs.filter((x) => x.amount)) {
        expect(s.text, name).toMatch(/^\d+$/);
        expect(s.term, name).toMatch(/^(effect:damage|effect:heal|status:.+|stat:hp)$/);
      }
      // R2-17: no number hides in a plain run before a status or a stat ("apply 2 " + "Poison").
      segs.forEach((s, i) => {
        if (!s.term && /\d $/.test(s.text) && /^(status|stat):/.test(segs[i + 1]?.term ?? "")) expect.fail(`${name}: "${s.text}" before ${segs[i + 1]!.term}`);
      });
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
    expect(all).toMatch(/^Hit:/m);
    expect(all).toMatch(/heal it for 1/);
  });

  test("stats read PWR and HP, never lower case", () => {
    expect(all).not.toMatch(/\b(pwr|hp)\b/);
  });

  test("silence, revive and summon read short", () => {
    expect(all).toMatch(/silence front enemy\./);
    expect(all).toMatch(/revive fallen ally at 2 HP\./);
    expect(all).toMatch(/summon Imp \(1\/2\)/);
    expect(all).not.toMatch(/strip its statuses|back of this unit's side/);
  });

  test("a trigger clause marks every run, a target its side", () => {
    const spike = pool.units.find((u) => u.name === "Spike")!;
    const segs = formSegments(spike.forms.sleeping, pool.abilities);
    expect(segs.filter((s) => s.clause === "when").map((s) => [s.text, s.term])).toEqual([
      ["Ally gets ", "trigger:StatusApplied"],
      ["Shield", "status:Shield"],
    ]);
    expect(segs.find((s) => s.term === "target:frontEnemy")?.side).toBe("enemy");
    expect(segs.filter((s) => s.term === "effect:damage").map((s) => [s.text, s.amount ?? false])).toEqual([["1", true], ["damage", false]]);
    const nurse = pool.units.find((u) => u.name === "Nurse")!;
    expect(formSegments(nurse.forms.sleeping, pool.abilities).find((s) => s.term === "target:eventUnit")).toMatchObject({
      text: "it",
      side: "ally",
    });
    const shield = describeStatusSegments(pool.statuses.Shield!);
    expect(shield.find((s) => s.term === "term:would")?.text).toBe("Would");
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

  test("a trigger's rule follows its scope: an enemy's death is not the holder's", () => {
    // The tip a trigger run shows comes from its scope, carried on the run.
    const scopeOf = (name: string) => formSegments(pool.units.find((u) => u.name === name)!.forms.sleeping, pool.abilities).find((s) => s.term === "trigger:Death")?.scope;
    const enemyDeath = pool.units.find((u) => u.forms.sleeping.when.some((w) => w.on.on === "Death" && "unit" in w.on && w.on.unit === "enemy"))!;
    expect(scopeOf(enemyDeath.name)).toBe("enemy");
    expect(scopedTip("trigger:Death", "enemy")).toBe("When an enemy dies.");
    expect(scopedTip("trigger:Death", "otherAlly")).toBe("When another ally (not self) dies.");
    expect(scopedTip("trigger:Death", "holder")).toBe(GLOSSARY["trigger:Death"].tip);
    expect(scopedTip("trigger:Death")).toBe(GLOSSARY["trigger:Death"].tip);
    expect(scopedTip("trigger:BattleStart", "any")).toBe(GLOSSARY["trigger:BattleStart"].tip);
  });
});

describe("keywords stand alone (R3-2, words.md (4))", () => {
  // A rule may name only core words (PWR, HP, stack, damage, heal, strike,
  // turn, ally, enemy, line, battle), never another keyword: if that keyword
  // is removed, the rule would lie. The names come from the tables, so a new
  // status, effect, state or battle rule is covered automatically. Triggers
  // and targets read as plain English ("dies", "self") and say nothing a
  // removal could break; core words and "would" (Blessing: "would die") stay.
  const CORE = new Set(["pwr", "hp", "damage", "heal", "stacks", "would"]);
  const named: [TermId, string][] = [
    ...Object.keys(STATUS_TERMS).map((k) => [`status:${k}` as TermId, k] as [TermId, string]),
    ...(Object.entries(GLOSSARY) as [FixedTermId, { label: string }][])
      .filter(([id]) => ["effect", "state", "battle", "condition", "term"].includes(termGroup(id)))
      .map(([id, d]) => [id, d.label] as [TermId, string]),
  ].filter(([, label]) => !CORE.has(label.toLowerCase()));
  const SCOPES: (UnitFilter | undefined)[] = [undefined, "ally", "otherAlly", "enemy", "any"];
  const rules: [TermId, string][] = [
    ...(Object.keys(GLOSSARY) as TermId[]).flatMap((id) => {
      const d = termDef(id)!;
      return [d.tip, d.more ?? "", ...SCOPES.map((sc) => scopedTip(id, sc) ?? "")].map((t) => [id, t] as [TermId, string]);
    }),
    ...Object.entries(STATUS_TERMS).flatMap(([k, d]) => [[`status:${k}`, d.tip], [`status:${k}`, d.more ?? ""]] as [TermId, string][]),
  ];
  // Fused is defined over Awoken: the core progression, not a keyword that may go.
  const allowed = (id: TermId, other: TermId) => id === other || (id === "state:fused" && other === "state:awoken");

  test("the list of names covers every status and the named rules", () => {
    const labels = named.map(([, l]) => l);
    for (const l of [...Object.keys(STATUS_TERMS), "Fatigue", "Summon", "Revive", "Silence", "Awoken", "Fused", "Chain stopped", "Time's up"]) expect(labels).toContain(l);
  });

  const unique = [...new Map(rules.filter(([, t]) => t !== "").map(([id, t]) => [`${id}|${t}`, [id, t] as [TermId, string]])).values()];
  test.each(unique)("%s: \"%s\" names no other keyword", (id, text) => {
    for (const [other, label] of named) {
      if (allowed(id, other)) continue;
      expect(text, `${id} names ${label}`).not.toMatch(new RegExp(`\\b${label}`, "i"));
    }
  });

  test("Damage says nothing about Shield; Shield says what it blocks", () => {
    expect(GLOSSARY["effect:damage"].tip).toBe("Takes away that much HP.");
    expect(STATUS_TERMS.Shield!.tip).toMatch(/Blocks damage of any kind/);
    expect(GLOSSARY["trigger:Hurt"].tip).toMatch(/counts even if all of it is blocked/);
  });

  test("Chain stopped reads the cap from the rules", () => {
    expect(GLOSSARY["battle:chainCapped"].tip).toContain(`${MVP_RULES.chainStepCap} steps`);
    // A run started under an older cap reads its own number (its ChainCapped event's steps).
    expect(chainCappedTip(64)).toContain("ran 64 steps");
    expect(GLOSSARY["battle:chainCapped"].tip).toBe(chainCappedTip(MVP_RULES.chainStepCap));
  });

  test("Time's up reads the turn cap from the rules (R3-26)", () => {
    expect(GLOSSARY["battle:timeUp"]).toMatchObject({ label: "Time's up", icon: "hourglass" });
    expect(GLOSSARY["battle:timeUp"].tip).toBe(timeUpTip(MVP_RULES.turnCap!));
    expect(GLOSSARY["battle:timeUp"].tip).toContain(`after turn ${MVP_RULES.turnCap}`);
    expect(GLOSSARY["battle:timeUp"].tip).toMatch(/draw/);
    // A run started before the cap (the kernel's 200) reads its own number.
    expect(timeUpTip(200)).toContain("after turn 200");
  });

  test("labels are the card's words", () => {
    expect(GLOSSARY["trigger:Hurt"].label).toBe("Hit");
    expect(GLOSSARY["target:holder"].label).toBe("Self");
    expect(GLOSSARY["target:eventUnit"].label).toBe("It");
    expect(GLOSSARY["target:allAllies"].label).toBe("All allies");
    expect(GLOSSARY["target:allEnemies"].label).toBe("All enemies");
    expect(GLOSSARY["target:allAllies"].tip).toMatch(/self included/);
    expect(scopedLabel("trigger:Death", "otherAlly")).toBe("Ally dies");
    expect(scopedLabel("trigger:Hurt", "enemy")).toBe("Enemy hit");
    expect(scopedLabel("trigger:StatChanged", "ally")).toBe("Ally gains PWR");
    expect(scopedLabel("trigger:Death", "holder")).toBe("Dies");
    expect(scopedLabel("trigger:BattleStart", "any")).toBe("Battle start");
    // A fired When in the battle (badge, Why) reads like the card (R3-19).
    expect(triggerLabel("trigger:StatusApplied", "Shield", "otherAlly")).toBe("Ally gets Shield");
    expect(triggerLabel("trigger:StatusRemoved", "Poison")).toBe("Loses Poison");
    expect(triggerLabel("trigger:Death", undefined, "ally")).toBe("Ally dies");
    expect(triggerLabel("trigger:Strike")).toBe("Strikes");
    // Every label a card's text opens with is a glossary label, scoped or not.
    const all = [...forms, ...fusions].flatMap(([, f]) => formSegments(f, pool.abilities));
    const labels = new Set(Object.keys(GLOSSARY).flatMap((id) => SCOPES.map((sc) => scopedLabel(id as TermId, sc)!.toLowerCase())));
    for (const s of all.filter((x) => x.clause === "when" && x.term?.startsWith("trigger:"))) {
      const words = s.text.trim().toLowerCase();
      if (/(gets|loses)$/.test(words)) continue; // "Ally gets " + the status: "Ally gets status"
      expect(labels, `"${s.text}"`).toContain(words);
    }
  });
});

