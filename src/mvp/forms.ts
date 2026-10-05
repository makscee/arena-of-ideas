// Unit forms, copies, awakening and fusion (mission #574). Pure functions on
// line units. Slice 2 owns the bodies; slice 4 (the run), slice 6 (bots) and
// slice 7 (content tuning) only call them, so the run never inlines these rules.
// Every function returns a new unit and leaves its inputs alone.

import { MVP_RULES, type FuseContext, type LineUnit, type MvpContent, type MvpRules, type UnitContent, type UnitId } from "./contract.js";
import { MvpDecisionError } from "./errors.js";

/** Until slice 2 lands, every fusion is refused with this (the API answers 501). */
const FUSION_NOT_YET = "fusion arrives in slice 2";

/** A fresh line unit of `u` with `copies` copies merged in (1 = just bought).
 * Slice 2: at rules.copiesToAwaken copies it starts in the awoken form. */
export function lineUnitOf(u: UnitContent, uid: string, copies = 1, rules: MvpRules = MVP_RULES): LineUnit {
  let unit: LineUnit = { uid, kind: "unit", unitId: u.id, name: u.name, emoji: u.emoji, copies: 1, form: "sleeping", stats: { ...u.base }, recipe: u.forms.sleeping };
  for (let n = 1; n < copies; n++) unit = mergeCopy(unit, u, rules);
  return unit;
}

/** Where a bought copy of `unitId` merges: the index of that unit, or of a
 * fused unit with it as either part; -1 when it needs a new slot. */
export function mergeTarget(line: LineUnit[], unitId: UnitId): number {
  return line.findIndex((x) => (x.kind === "fused" ? x.fusion?.first === unitId || x.fusion?.second === unitId : x.unitId === unitId));
}

/** `u` with one more copy merged in: +rules.copyGrowth.
 * Slice 2 adds the awakening at rules.copiesToAwaken. */
export function addCopy(u: LineUnit, content: MvpContent, rules: MvpRules): LineUnit {
  return mergeCopy(u, u.kind === "unit" ? content.units.find((x) => x.id === u.unitId) : undefined, rules);
}

/** One copy into `unit`; `src` is its content (undefined for a fused unit,
 * which only grows). Slice 2: the copiesToAwaken-th copy swaps a sleeping
 * unit to src.forms.awoken. */
function mergeCopy(unit: LineUnit, _src: UnitContent | undefined, rules: MvpRules): LineUnit {
  return {
    ...unit,
    copies: unit.copies + 1,
    stats: { pwr: unit.stats.pwr + rules.copyGrowth.pwr, hp: unit.stats.hp + rules.copyGrowth.hp },
  };
}

/** Why `a` and `b` (in tap order) may not fuse, or null when they may.
 * Slice 2: both awoken, neither fused. */
export function fuseCheck(_a: LineUnit, _b: LineUnit): string | null {
  return FUSION_NOT_YET;
}

/** The fused unit: When of `first`, Who of `second`, Does of both (first's,
 * then second's), stats summed; it keeps first's uid. `ctx` carries the name
 * and the credit, which the server looks up (slice 10). Call it only after
 * fuseCheck(first, second) returned null. Slice 2 fills it in. */
export function fuseUnits(_first: LineUnit, _second: LineUnit, _ctx: FuseContext, _content: MvpContent): LineUnit {
  throw new MvpDecisionError("fuse", FUSION_NOT_YET);
}
