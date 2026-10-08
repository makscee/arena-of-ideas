// Arena MVP kernel: forms, copies, Awoken and fusion (mission #574, slice 2).
// Pure functions over the contract's LineUnit; run.ts, the server, bots and
// content simulation call these instead of growing units themselves.
//
// - A unit has two forms in its content, sleeping and awoken. The awoken form
//   keeps the same When and upgrades the Who and/or the Does.
// - Each copy merged in adds rules.copyGrowth (+1 PWR / +2 HP); the
//   rules.copiesToAwaken-th copy swaps the recipe to the awoken form.
// - Fusion takes two Awoken units in tap order: the When of the first, the Who
//   of the second, the Does of both (first's, then second's). Its PWR and HP
//   are the stronger part's plus one copy's growth (rules.copyGrowth), not the
//   sum (round 3, note 5). A fused unit is final; further copies of either part
//   merge into it for stats.
//
// Slice 2 owns these bodies. The run (slice 4), bots (slice 6) and content
// tuning (slice 7) only call them. A fusion's name and credit come from the
// server's namer (slice 10, server/src/mvp/fusions.ts), which also owns the
// portmanteau fallback and the rule that bots are never credited.

import { validateTeam } from "../validate.js";
import type { Stats, UnitDef } from "../types.js";
import {
  MVP_RULES,
  type FuseContext,
  type LineUnit,
  type MvpContent,
  type MvpRules,
  type UnitContent,
  type UnitForm,
  type UnitId,
} from "./contract.js";

function grown(base: Stats, extraCopies: number, rules: MvpRules): Stats {
  return { pwr: base.pwr + extraCopies * rules.copyGrowth.pwr, hp: base.hp + extraCopies * rules.copyGrowth.hp };
}

/** A fresh line unit from content with `copies` merged in: stats grown per
 * copy, already awoken at rules.copiesToAwaken copies. */
export function lineUnitOf(u: UnitContent, uid: string, copies = 1, rules: MvpRules = MVP_RULES): LineUnit {
  if (!Number.isInteger(copies) || copies < 1) throw new Error(`copies must be a positive integer, got ${copies}`);
  const form = copies >= rules.copiesToAwaken ? "awoken" : "sleeping";
  return {
    uid,
    kind: "unit",
    unitId: u.id,
    name: u.name,
    emoji: u.emoji,
    copies,
    form,
    stats: grown(u.base, copies - 1, rules),
    recipe: structuredClone(u.forms[form]),
  };
}

/** Where a new copy of `unitId` merges: the unit itself, or a fused unit with
 * it as either part. -1 means it takes a new slot. */
export function mergeTarget(line: LineUnit[], unitId: UnitId): number {
  return line.findIndex((u) =>
    u.kind === "fused" ? u.fusion?.first === unitId || u.fusion?.second === unitId : u.unitId === unitId,
  );
}

/** Merge one more copy in: +copyGrowth, and the copiesToAwaken-th copy of a
 * plain unit swaps it to its awoken form. Fused units only grow. */
export function addCopy(u: LineUnit, content: MvpContent, rules: MvpRules = MVP_RULES): LineUnit {
  const next: LineUnit = structuredClone(u);
  next.copies += 1;
  next.stats = grown(u.stats, 1, rules);
  if (next.kind === "unit" && next.form === "sleeping" && next.copies >= rules.copiesToAwaken) {
    const c = content.units.find((x) => x.id === u.unitId);
    if (!c) throw new Error(`unknown unit ${u.unitId}`);
    next.form = "awoken";
    next.recipe = structuredClone(c.forms.awoken);
  }
  return next;
}

/** Why these two can't fuse, or null when both are Awoken and neither is fused. */
export function fuseCheck(a: LineUnit, b: LineUnit): string | null {
  if (a.uid === b.uid) return "a unit can't fuse with itself";
  if (a.kind === "fused" || b.kind === "fused") return "a fused unit is final";
  if (a.form !== "awoken" || b.form !== "awoken") return "both units must be Awoken";
  return null;
}

/** Fuse two Awoken units in tap order: the When of `first`, the Who of
 * `second`, the Does of both (first's, then second's). PWR and HP are each
 * the higher of the two plus one copy's growth; copies are summed. The result
 * keeps first's uid and is final. `ctx` carries the name and the credit,
 * which the server looks up (slice 10). */
export function fuseUnits(
  first: LineUnit,
  second: LineUnit,
  ctx: FuseContext,
  _content: MvpContent,
  rules: MvpRules = MVP_RULES,
): LineUnit {
  const why = fuseCheck(first, second);
  if (why) throw new Error(`can't fuse ${first.name} + ${second.name}: ${why}`);
  const recipe: UnitForm = {
    when: structuredClone(first.recipe.when),
    ...(first.recipe.condition ? { condition: structuredClone(first.recipe.condition) } : {}),
    who: structuredClone(second.recipe.who),
    does: [...first.recipe.does, ...second.recipe.does],
  };
  // Each part's "and" clauses keep their own Who: the first's, then the second's.
  const also = [...(first.recipe.also ?? []), ...(second.recipe.also ?? [])];
  if (also.length) recipe.also = structuredClone(also);
  return {
    uid: first.uid,
    kind: "fused",
    unitId: first.unitId,
    name: ctx.name,
    emoji: `${first.emoji}${second.emoji}`,
    copies: first.copies + second.copies,
    form: "awoken",
    stats: {
      pwr: Math.max(first.stats.pwr, second.stats.pwr) + rules.copyGrowth.pwr,
      hp: Math.max(first.stats.hp, second.stats.hp) + rules.copyGrowth.hp,
    },
    recipe,
    fusion: {
      first: first.unitId,
      second: second.unitId,
      name: ctx.name,
      // The namer already applied the credit rule (FusionDiscovery); copy it.
      discoveredBy: ctx.discoveredBy,
    },
  };
}

function formDef(u: UnitContent, form: UnitForm, key: string): UnitDef {
  return {
    name: `${u.name} (${key})`,
    base: { ...u.base },
    triggers: form.when,
    selectors: form.who,
    abilities: form.does,
    ...(form.condition ? { condition: form.condition } : {}),
    ...(form.also?.length ? { also: form.also.map((c) => ({ selectors: c.who, abilities: c.does })) } : {}),
  };
}

const same = (a: unknown, b: unknown) => JSON.stringify(a) === JSON.stringify(b);

/** Problems with a unit's two forms; empty when it can ship. */
export function formProblems(u: UnitContent, content: Pick<MvpContent, "abilities" | "statuses">): string[] {
  const out: string[] = [];
  const { sleeping, awoken } = u.forms ?? ({} as UnitContent["forms"]);
  if (!sleeping || !awoken) return [`${u.id}: needs both a sleeping and an awoken form`];
  if (sleeping.does.length !== 1) out.push(`${u.id}: the sleeping form must Do exactly one thing, got ${sleeping.does.length}`);
  if (sleeping.also?.length) out.push(`${u.id}: the sleeping form has no "and" clause`);
  if (awoken.does.length < 1) out.push(`${u.id}: the awoken form must Do something`);
  if (!same(sleeping.when, awoken.when) || !same(sleeping.condition, awoken.condition)) {
    out.push(`${u.id}: the awoken form must keep the sleeping form's When`);
  }
  if (same(sleeping.who, awoken.who) && same(sleeping.does, awoken.does) && !awoken.also?.length) {
    out.push(`${u.id}: the awoken form must upgrade the Who and/or the Does`);
  }
  for (const [key, form] of [["sleeping", sleeping], ["awoken", awoken]] as const) {
    // "The attacker" is who dealt a hit: only a When on a hit (hurt, allyHurt) has one.
    const usesAttacker = [form.who, ...(form.also ?? []).map((c) => c.who)].some((who) => who.some((w) => w.kind === "attacker"));
    if (usesAttacker && !(form.when.length > 0 && form.when.every((w) => w.kind === "trigger" && w.on.on === "Hurt"))) {
      out.push(`${u.id}.${key}: "the attacker" needs a When on a hit (hurt or allyHurt)`);
    }
    for (const issue of validateTeam([formDef(u, form, key)], content.statuses, content.abilities, `${u.id}.${key}`)) {
      out.push(`${issue.path}: ${issue.message}`);
    }
  }
  return out;
}

/** Problems across a content pack's units (duplicate ids, bad forms). */
export function contentFormProblems(content: MvpContent): string[] {
  const out: string[] = [];
  const ids = new Set<string>();
  for (const u of content.units) {
    if (ids.has(u.id)) out.push(`${u.id}: duplicate unit id`);
    ids.add(u.id);
    out.push(...formProblems(u, content));
  }
  return out;
}
