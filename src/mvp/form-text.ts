// A unit form as one sentence (When → Who → Does), shared by the phone client,
// the tests and anything else that shows a unit's text. Display-only, like
// describe.ts: the segments carry glossary terms, and joining their text gives
// the plain sentence.

import { describeAbilitySegments, type DescribeSegment, type Lang } from "../describe.js";
import { withStatusNames } from "../status-names.js";
import type { AbilityRegistry, Effect, StatusDef, StatusRegistry, UnitDef } from "../types.js";
import type { UnitForm } from "./contract.js";

/** A form as segments: its authored text (one plain segment), else described
 * from its abilities. The form's own Does share its When and Who: one sentence,
 * the Does joined by "then" ("When the battle begins: summon …, then apply 3
 * Vitality to every ally."), unless a Does carries a condition of its own. */
export function formSegments(form: UnitForm, abilities: AbilityRegistry, lang?: Lang, statuses?: StatusRegistry, carries: readonly string[] = (form.statuses ?? []).map((s) => s.status)): DescribeSegment[] {
  if (form.text) return [{ text: form.text }];
  return verbFirst(withStatusNames(statuses, () => {
    const out = ruleSegments(form, abilities, lang);
    // A unit it becomes (rules v2: Seed's Tree, Chrysalis's Butterfly) brings
    // a rule of its own: the card says it, named, so the whole unit is read.
    for (const body of bodies(form, abilities)) out.push({ text: ` ${lang === "ru" ? (body.names?.ru.name ?? body.name) : body.name}: ` }, ...ruleSegments(bodyForm(body), abilities, lang));
    // A second rule lives on a counter status (rules v2: Hourglass's grains,
    // Raven's memory): the card reads it on after the form's own, so the
    // whole rule is said. `carries`: the statuses the unit starts with.
    for (const id of counterRules(form, abilities, statuses ?? {}, carries)) out.push({ text: " " }, ...statusRuleSegments(id, statuses![id]!, lang));
    return out;
  }), lang, isV2Statuses(statuses));
}

const CLAUSE_ENDS = [".", ", ", ", and ", ", then ", ", и ", ", затем "];

/** Rules v2 content: a counter status carries a rule of its own. */
const isV2Statuses = (statuses?: StatusRegistry): boolean => Object.values(statuses ?? {}).some((d) => d.counter && d.abilities.length > 0);

/** Rules v2 (#1047, Maks's note on batch 1): a clause never opens on a bare
 * number. "Ally strikes: 1 Beats to self." reads "Ally strikes: gain 1
 * Beats.", "1 Strength to every ally behind" reads "give every ally behind 1
 * Strength" (RU "получить 1 Такт", "дать всем союзникам позади 1 Силу").
 * Only a clause's opening status, and only on v2 content (`on`), so the old
 * pool's goldens stay. */
function verbFirst(segs: DescribeSegment[], lang: Lang | undefined, on: boolean): DescribeSegment[] {
  if (!on) return segs;
  const ru = lang === "ru";
  const opens = ru ? [": ", ", и ", ", затем "] : [": ", ", and ", ", then "];
  const out: DescribeSegment[] = [];
  for (let i = 0; i < segs.length; i++) {
    const amount = segs[i]!;
    const name = segs[i + 2];
    const to = segs[i + 3];
    const isStatus = (amount as { amount?: boolean }).amount && amount.term?.startsWith("status:") && /^\d+$/.test(amount.text) && name?.term === amount.term && segs[i + 1]?.text === " ";
    if (!isStatus || !opens.includes(out[out.length - 1]?.text ?? "") || to?.text !== (ru ? " " : " to ")) {
      out.push(amount);
      continue;
    }
    let j = i + 4;
    const target: DescribeSegment[] = [];
    // the target runs to the clause's end (v2 position selectors carry no term)
    while (j < segs.length && !CLAUSE_ENDS.includes(segs[j]!.text)) target.push(segs[j++]!);
    if (!target.length) {
      out.push(amount);
      continue;
    }
    const n = Number(amount.text);
    // RU: one of a feminine status reads in the accusative ("1 Силу", "1 Песчинку").
    const said = ru && n === 1 ? { ...name!, text: name!.text.replace(/а$/, "у").replace(/я$/, "ю") } : name!;
    const self = target.length === 1 && target[0]!.term === "target:holder";
    if (self) out.push({ text: ru ? "получить " : "gain " }, amount, { text: " " }, said);
    else out.push({ text: ru ? "дать " : "give " }, ...target, { text: " " }, amount, { text: " " }, said);
    i = j - 1;
  }
  return out;
}

/** The form's own rule: its clause, then its "and" clauses. */
function ruleSegments(form: UnitForm, abilities: AbilityRegistry, lang?: Lang): DescribeSegment[] {
  const out = clauseSegments(form, abilities, lang);
  // Each "and" clause (R4-7) reads on after the form's own Does, with its own
  // Who and without the When again: "Ally dies: 2 Strength to self, and
  // silence front enemy."
  for (const clause of form.also ?? []) {
    const segs = clauseSegments({ ...form, who: clause.who, does: clause.does, also: [] }, abilities, lang);
    const colon = segs.findIndex((s) => s.text === ": ");
    if (colon < 0 || out[out.length - 1]?.text !== ".") {
      out.push({ text: " " }, ...segs);
      continue;
    }
    out.pop();
    out.push({ text: lang === "ru" ? ", и " : ", and " }, ...segs.slice(colon + 1));
  }
  return out;
}

/** The counter statuses with a rule of their own that the form gives (or
 * the unit starts with), in order, once each. */
function counterRules(form: UnitForm, abilities: AbilityRegistry, statuses: StatusRegistry, carries: readonly string[]): string[] {
  const ids: string[] = [];
  const visit = (e: Effect): void => {
    if (e.kind === "applyStatus") ids.push(e.status);
    else if (e.kind === "delay") visit(e.effect);
  };
  for (const id of [...form.does, ...(form.also ?? []).flatMap((c) => c.does)]) for (const e of abilities[id]?.effects ?? []) visit(e);
  ids.push(...carries);
  return [...new Set(ids)].filter((id) => statuses[id]?.counter && statuses[id]!.abilities.length > 0);
}

/** The units the form's effects transform into, in order, once each. */
function bodies(form: UnitForm, abilities: AbilityRegistry): UnitDef[] {
  const out: UnitDef[] = [];
  const visit = (e: Effect): void => {
    if (e.kind === "transform" && e.into.abilities?.length && !out.some((b) => b.name === e.into.name)) out.push(e.into);
    else if (e.kind === "delay") visit(e.effect);
  };
  for (const id of [...form.does, ...(form.also ?? []).flatMap((c) => c.does)]) for (const e of abilities[id]?.effects ?? []) visit(e);
  return out;
}

/** A transform's body as a form: its When, Who, Does and "and" clauses. */
const bodyForm = (b: UnitDef): UnitForm => ({
  when: b.triggers ?? [],
  who: b.selectors ?? [],
  does: b.abilities ?? [],
  ...(b.condition ? { condition: b.condition } : {}),
  ...(b.also ? { also: b.also.map((c) => ({ who: c.selectors, does: c.abilities })) } : {}),
});

/** A counter status's rule as the form's own would read: its first ability
 * the clause, the rest its "and" clauses. Its own stacks are named ("spend 3
 * Grains", not "spend 3 stacks"), since the card is the unit's, not the status's. */
function statusRuleSegments(id: string, def: StatusDef, lang?: Lang): DescribeSegment[] {
  const named = (e: Effect): Effect => (e.kind === "consumeStacks" && e.status === undefined ? { ...e, status: id } : e);
  const abilities: AbilityRegistry = {};
  def.abilities.forEach((ab, i) => (abilities[`${id}#${i}`] = { name: id, family: "Arcane", ...ab, effects: ab.effects.map(named) }));
  const [first, ...rest] = def.abilities.map((ab, i) => ({ who: def.selectors ?? ab.selectors ?? [], does: [`${id}#${i}`] }));
  const condition = def.condition ?? def.abilities[0]?.condition;
  return ruleSegments({ when: def.triggers ?? def.abilities[0]?.whens ?? [], ...(condition ? { condition } : {}), who: first!.who, does: first!.does, also: rest }, abilities, lang);
}

/** One clause (a form's Who and its Does) as one sentence. */
function clauseSegments(form: UnitForm, abilities: AbilityRegistry, lang?: Lang): DescribeSegment[] {
  const abs = form.does.map((id) => abilities[id]);
  const cond = form.condition ? { condition: form.condition } : {};
  const opts = lang ? { lang } : {};
  if (abs.length > 1 && abs.every((ab) => ab !== undefined && ab.condition === undefined)) {
    return describeAbilitySegments({ ...abs[0]!, whens: form.when, selectors: form.who, effects: sumStatuses(abs.flatMap((ab) => ab!.effects)), ...cond }, opts);
  }
  const out: DescribeSegment[] = [];
  form.does.forEach((id, i) => {
    if (i > 0) out.push({ text: " " });
    const ab = abilities[id];
    out.push(...(ab ? describeAbilitySegments({ ...ab, whens: form.when, selectors: form.who, ...cond }, opts) : [{ text: id }]));
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
export const formText = (form: UnitForm, abilities: AbilityRegistry, lang?: Lang, statuses?: StatusRegistry, carries?: readonly string[]): string =>
  formSegments(form, abilities, lang, statuses, carries).map((s) => s.text).join("");

/** The card's sentence (#1047): the form's authored `say` in `lang` when it
 * has one, else formSegments. The word chain's glossary terms that the
 * sentence names word for word stay tappable. */
export function sentenceSegments(form: UnitForm, abilities: AbilityRegistry, lang?: Lang, statuses?: StatusRegistry): DescribeSegment[] {
  const chain = formSegments(form, abilities, lang, statuses);
  if (!form.say) return chain;
  let rest = lang === "ru" ? form.say.ru : form.say.en;
  const out: DescribeSegment[] = [];
  for (const term of chain.filter((s) => s.term && s.text.trim())) {
    const at = rest.indexOf(term.text);
    if (at < 0) continue;
    if (at > 0) out.push({ text: rest.slice(0, at) });
    out.push(term);
    rest = rest.slice(at + term.text.length);
  }
  if (rest) out.push({ text: rest });
  return out;
}

/** sentenceSegments joined. */
export const sentenceText = (form: UnitForm, abilities: AbilityRegistry, lang?: Lang, statuses?: StatusRegistry): string =>
  sentenceSegments(form, abilities, lang, statuses).map((s) => s.text).join("");
