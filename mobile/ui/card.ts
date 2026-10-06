// The unit card and the unit sheet (mission #574), shared by the shop and
// result screens (slice 8), the battle viewer (slice 9) and the stats page
// (slice 11). Slice 8 owns the look of both; slices 9 and 11 only pass
// options (live numbers, rates, onOpen), so nobody reshapes these signatures.
import { formText as sharedFormText } from "../../src/mvp/form-text";
import type { BattleUnit, LineUnit, MvpContent, UnitContent, UnitForm } from "../../src/mvp/contract";
import type { Stats } from "../../src/types";
import { h } from "./dom";
import { discoveredLine } from "./fusion";
import { unitStatsLine, type UnitRates } from "./unit-stats";

/** What a card needs to draw; LineUnit, BattleUnit and offers all fit. */
export type CardUnit = { emoji: string; name: string; stats: Stats } & Partial<Pick<LineUnit, "unitId" | "kind" | "form" | "copies" | "fusion">>;

export interface CardOptions {
  side: "you" | "ghost";
  /** Extra rows under the stats (a cost, a copies badge). */
  extra?: (Node | null)[];
  testid?: string;
  /** The battle viewer's live state (slice 9): current stats, dead, the acting unit lit. */
  live?: { stats: Stats; dead?: boolean; acting?: boolean };
  /** Win and pick rate (slice 11); without them unitStatsLine looks them up by unitId. */
  rates?: UnitRates;
  /** Tapping the card opens this, usually overlay(unitSheet(...)). */
  onOpen?: () => void;
}

export function card(u: CardUnit, o: CardOptions): HTMLElement {
  const stats = o.live?.stats ?? u.stats;
  const el = h(
    "div",
    { class: `card ${o.side}`, ...(o.testid ? { "data-testid": o.testid } : {}) },
    h("div", { class: "emoji" }, u.emoji),
    // ui/dom.ts fitText() shrinks a long name to the card's measured width.
    h("div", { class: "name" }, u.name),
    h("div", { class: "stats" }, h("span", { class: "p" }, `${stats.pwr}`), " / ", h("span", { class: "h" }, `${stats.hp}`)),
    cardRates(u, o.rates),
    ...(o.extra ?? []),
  );
  if (u.kind === "fused") el.classList.add("fused");
  else if (u.form === "awoken") el.classList.add("awoken");
  if (o.live?.dead) el.classList.add("dead");
  if (o.live?.acting) el.classList.add("acting");
  if (o.onOpen) el.addEventListener("click", o.onOpen);
  return el;
}

/** The card's rates line, always one line so cards in a row line up: the
 * unit's rates, or "—" without any. A fused unit has no rates of its own (its
 * unitId is its first part's), so its line shows who discovered it instead
 * ("by you"; the sheet says it in full, with both parts' rates). */
function cardRates(u: CardUnit, rates?: UnitRates): Node {
  if (u.kind === "fused") return discoveredLine(u, true) ?? h("div", { class: "rates none" }, "—");
  return unitStatsLine(u.unitId, rates) ?? h("div", { class: "rates none" }, "—");
}

/** A form as one line of text: its authored text, else described from the
 * content's abilities (When → Who → Does, each Does in order). */
export function formText(form: UnitForm, content: MvpContent): string {
  return sharedFormText(form, content.abilities);
}

/** Everything about one unit: exact numbers and both forms; for a fused unit,
 * both parts. Slice 8 fills it in and opens it from the shop and Home; slice 9
 * only opens it from the battle; slice 11 only passes rates. Open it with
 * overlay(unitSheet(...)) from ./dom. The stub lists the forms as text. */
export function unitSheet(u: LineUnit | BattleUnit | UnitContent, content: MvpContent, opts: { rates?: UnitRates; from?: Stats } = {}): HTMLElement {
  const unitId = "forms" in u ? u.id : u.unitId;
  // opts.from: your copy's stats now, when u is that copy after a buy (the shop's offer sheet).
  const statsLine = (s: Stats) =>
    opts.from
      ? h("div", { class: "num", "data-testid": "sheet-stats" }, `${opts.from.pwr} PWR / ${opts.from.hp} HP → ${s.pwr} PWR / ${s.hp} HP`)
      : h("div", { class: "num", "data-testid": "sheet-stats" }, `${s.pwr} PWR / ${s.hp} HP`);
  return h(
    "div",
    { class: "stack", "data-testid": "unit-sheet" },
    h("h2", {}, `${u.emoji} ${u.name}`),
    "forms" in u ? null : discoveredLine(u),
    "stats" in u ? statsLine(u.stats) : h("div", { class: "num" }, `${u.base.pwr} PWR / ${u.base.hp} HP · tier ${u.tier}`),
    "stats" in u ? h("div", { class: "dim" }, opts.from ? `Your copy now → after buying: ${sheetState(u)}` : sheetState(u)) : null,
    ...sheetForms(u, content).map(([label, form]) =>
      h("div", { class: `sheet-form${"form" in u && u.kind !== "fused" && label.toLowerCase() === u.form ? " now" : ""}` }, h("div", { class: "label" }, label), formText(form, content)),
    ),
    ...sheetRates(u, unitId, content, opts.rates),
  );
}

/** The sheet's rates: the unit's own; for a fused unit, each part's, named. */
function sheetRates(u: LineUnit | BattleUnit | UnitContent, unitId: string, content: MvpContent, rates?: UnitRates): (Node | null)[] {
  if ("forms" in u || u.kind !== "fused" || !u.fusion) return [unitStatsLine(unitId, rates)];
  return [u.fusion.first, u.fusion.second].map((id) => {
    const line = unitStatsLine(id);
    const part = content.units.find((x) => x.id === id);
    return line ? h("div", { class: "row part-rates" }, h("span", { class: "dim small" }, `${part?.name ?? id}:`), line) : null;
  });
}

/** Form and copies; a fused unit says it is final (its credit is discoveredLine). */
function sheetState(u: LineUnit | BattleUnit): string {
  if (u.kind === "fused") return `Fused, final · ${u.copies} copies`;
  return `${u.form === "awoken" ? "Awoken" : "Sleeping"} · ${u.copies} ${u.copies === 1 ? "copy" : "copies"}`;
}

function sheetForms(u: LineUnit | BattleUnit | UnitContent, content: MvpContent): [string, UnitForm][] {
  const unit = (id: string) => content.units.find((x) => x.id === id);
  const both = (c: UnitContent, prefix = ""): [string, UnitForm][] => [
    [`${prefix}Sleeping`, c.forms.sleeping],
    [`${prefix}Awoken`, c.forms.awoken],
  ];
  if ("forms" in u) return both(u);
  if (u.kind === "fused" && u.fusion) {
    const parts = [unit(u.fusion.first), unit(u.fusion.second)].filter((c): c is UnitContent => c !== undefined);
    return [["Fused", u.recipe], ...parts.flatMap((c) => both(c, `${c.name} · `))];
  }
  const c = unit(u.unitId);
  return c ? both(c) : [["Now", u.recipe]];
}
