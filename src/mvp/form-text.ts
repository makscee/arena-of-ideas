// A unit form as one sentence (When → Who → Does), shared by the phone client,
// the tests and anything else that shows a unit's text. Display-only, like
// describe.ts: the segments carry glossary terms, and joining their text gives
// the plain sentence.

import { describeAbilitySegments, type DescribeSegment } from "../describe.js";
import type { AbilityRegistry } from "../types.js";
import type { UnitForm } from "./contract.js";

/** A form as segments: its authored text (one plain segment), else described
 * from its abilities. Every Does shares the form's When and Who: one sentence,
 * the Does joined by "then" ("When the battle begins: summon …, then apply 3
 * Vitality to every ally."), unless a Does carries a condition of its own. */
export function formSegments(form: UnitForm, abilities: AbilityRegistry): DescribeSegment[] {
  if (form.text) return [{ text: form.text }];
  const abs = form.does.map((id) => abilities[id]);
  const cond = form.condition ? { condition: form.condition } : {};
  if (abs.length > 1 && abs.every((ab) => ab !== undefined && ab.condition === undefined)) {
    return describeAbilitySegments({ ...abs[0]!, whens: form.when, selectors: form.who, effects: abs.flatMap((ab) => ab!.effects), ...cond });
  }
  const out: DescribeSegment[] = [];
  form.does.forEach((id, i) => {
    if (i > 0) out.push({ text: " " });
    const ab = abilities[id];
    out.push(...(ab ? describeAbilitySegments({ ...ab, whens: form.when, selectors: form.who, ...cond }) : [{ text: id }]));
  });
  return out;
}

/** formSegments joined: the form's plain sentence. */
export const formText = (form: UnitForm, abilities: AbilityRegistry): string =>
  formSegments(form, abilities).map((s) => s.text).join("");
