// The unit card and the unit sheet (mission #574), shared by the shop and
// result screens (slice 8), the battle viewer (slice 9) and the stats page
// (slice 11). Slice 8 owns the look of both; slices 9 and 11 only pass
// options (live numbers, rates, onOpen), so nobody reshapes these signatures.
import { describeAbility } from "../../src/describe";
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
    h("div", { class: "name" }, u.name),
    discoveredLine(u),
    h("div", { class: "stats" }, h("span", { class: "p" }, `${stats.pwr}`), " / ", h("span", { class: "h" }, `${stats.hp}`)),
    unitStatsLine(u.unitId, o.rates),
    ...(o.extra ?? []),
  );
  if (o.live?.dead) el.classList.add("dead");
  if (o.live?.acting) el.classList.add("acting");
  if (o.onOpen) el.addEventListener("click", o.onOpen);
  return el;
}

/** A form as one line of text: its authored text, else described from the
 * content's abilities (When → Who → Does, each Does in order). */
export function formText(form: UnitForm, content: MvpContent): string {
  if (form.text) return form.text;
  return form.does
    .map((id) => {
      const ab = content.abilities[id];
      return ab ? describeAbility({ ...ab, whens: form.when, selectors: form.who, ...(form.condition ? { condition: form.condition } : {}) }) : id;
    })
    .join(" ");
}

/** Everything about one unit: exact numbers and both forms; for a fused unit,
 * both parts. Slice 8 fills it in and opens it from the shop and Home; slice 9
 * only opens it from the battle; slice 11 only passes rates. Open it with
 * overlay(unitSheet(...)) from ./dom. The stub lists the forms as text. */
export function unitSheet(u: LineUnit | BattleUnit | UnitContent, content: MvpContent, opts: { rates?: UnitRates } = {}): HTMLElement {
  const unitId = "forms" in u ? u.id : u.unitId;
  return h(
    "div",
    { class: "stack", "data-testid": "unit-sheet" },
    h("h2", {}, `${u.emoji} ${u.name}`),
    "forms" in u ? null : discoveredLine(u),
    "stats" in u ? h("div", { class: "num" }, `${u.stats.pwr} PWR / ${u.stats.hp} HP`) : h("div", { class: "num" }, `${u.base.pwr} PWR / ${u.base.hp} HP`),
    ...sheetForms(u, content).map(([label, form]) => h("div", {}, h("div", { class: "label" }, label), formText(form, content))),
    unitStatsLine(unitId, opts.rates),
  );
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
