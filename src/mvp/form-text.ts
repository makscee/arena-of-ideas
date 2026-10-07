// A unit form as one sentence (When → Who → Does), shared by the phone client,
// the tests and anything else that shows a unit's text. Display-only, like
// describe.ts: the segments carry glossary terms, and joining their text gives
// the plain sentence.

import { describeAbilitySegments, type DescribeSegment } from "../describe.js";
import type { AbilityRegistry, Effect } from "../types.js";
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
    return describeAbilitySegments({ ...abs[0]!, whens: form.when, selectors: form.who, effects: sumStatuses(abs.flatMap((ab) => ab!.effects)), ...cond });
  }
  const out: DescribeSegment[] = [];
  form.does.forEach((id, i) => {
    if (i > 0) out.push({ text: " " });
    const ab = abilities[id];
    out.push(...(ab ? describeAbilitySegments({ ...ab, whens: form.when, selectors: form.who, ...cond }) : [{ text: id }]));
  });
  return out;
}

/** A fused unit's Does can give the same status twice ("1 Shield …, then
 * 1 Shield"): the text says it once, with the stacks added up ("2 Shield"),
 * which is what the target ends up with. Display only: the fight still
 * applies each one. */
function sumStatuses(effects: Effect[]): Effect[] {
  const out: Effect[] = [];
  for (const e of effects) {
    const same = e.kind === "applyStatus" && e.stacks.kind === "const" ? out.find((o) => o.kind === "applyStatus" && o.status === e.status && o.stacks.kind === "const") : undefined;
    if (same && same.kind === "applyStatus" && same.stacks.kind === "const" && e.kind === "applyStatus" && e.stacks.kind === "const") {
      out[out.indexOf(same)] = { ...same, stacks: { kind: "const", value: same.stacks.value + e.stacks.value } };
    } else out.push(e);
  }
  return out;
}

/** formSegments joined: the form's plain sentence. */
export const formText = (form: UnitForm, abilities: AbilityRegistry): string =>
  formSegments(form, abilities).map((s) => s.text).join("");
