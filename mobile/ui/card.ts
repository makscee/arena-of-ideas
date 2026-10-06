// The unit card and the unit sheet (mission #574), shared by the shop and
// result screens (slice 8), the battle viewer (slice 9) and the stats page
// (slice 11). Slice 8 owns the look of both; slices 9 and 11 only pass
// options (live numbers, rates, onOpen), so nobody reshapes these signatures.
//
// Round 2 (R2-7): the card is compact (64×84 on a phone): a trigger icon, the
// emoji, a one-line name, PWR/HP and one footer slot. The sheet shows only the
// form the unit has now; win and pick rates are its one dim last line.
import { termDef, termIcon, type TermId } from "../../src/glossary";
import { formSegments, formText as sharedFormText } from "../../src/mvp/form-text";
import { MVP_RULES, type BattleUnit, type LineUnit, type MvpContent, type UnitContent, type UnitForm } from "../../src/mvp/contract";
import type { Stats } from "../../src/types";
import { h } from "./dom";
import { discoveredLine } from "./fusion";
import { icon } from "./icon";
import { markChangedPieces, richText } from "./term";
import { unitStatsLine, type UnitRates } from "./unit-stats";

/** What a card needs to draw; LineUnit, BattleUnit and offers all fit. */
export type CardUnit = { emoji: string; name: string; stats: Stats } & Partial<Pick<LineUnit, "unitId" | "kind" | "form" | "copies" | "fusion" | "recipe">>;

export interface CardOptions {
  side: "you" | "ghost";
  /** The footer slot: copies pips, AWOKEN, FUSED, a price, or battle statuses. */
  extra?: (Node | null)[];
  testid?: string;
  /** The battle viewer's live state (slice 9): current stats, dead, the acting unit lit. */
  live?: { stats: Stats; dead?: boolean; acting?: boolean };
  /** An offer's tier, drawn as dots top-right. */
  tier?: number;
  /** Tapping the card opens this, usually overlay(unitSheet(...)). */
  onOpen?: () => void;
}

export function card(u: CardUnit, o: CardOptions): HTMLElement {
  const stats = o.live?.stats ?? u.stats;
  const el = h(
    "div",
    { class: `card ${o.side}`, ...(o.testid ? { "data-testid": o.testid } : {}) },
    triggerMark(u.recipe),
    o.tier ? h("span", { class: "tier", "aria-label": `tier ${o.tier}` }, "●".repeat(o.tier)) : null,
    h("div", { class: "emoji" }, u.emoji),
    // One line; ui/dom.ts fitText() shrinks a long name a little, then cuts it.
    h("div", { class: "name", title: u.name }, u.name),
    h("div", { class: "stats" }, h("span", { class: "p" }, `${stats.pwr}`), "/", h("span", { class: "h" }, `${stats.hp}`)),
    h("div", { class: "foot" }, ...(o.extra ?? [])),
  );
  if (u.kind === "fused") el.classList.add("fused");
  else if (u.form === "awoken") el.classList.add("awoken");
  if (o.live?.dead) el.classList.add("dead");
  if (o.live?.acting) el.classList.add("acting");
  if (o.onOpen) el.addEventListener("click", o.onOpen);
  return el;
}

/** The term for what wakes a form: its first When ("trigger:BattleStart"). */
function triggerTerm(form: UnitForm | undefined): { id: TermId; status?: string } | null {
  const on = form?.when[0]?.on;
  if (!on) return null;
  return { id: `trigger:${on.on}` as TermId, ...("status" in on && on.status ? { status: on.status } : {}) };
}

/** The card's top-left mark: the trigger's icon, in the When colour. */
function triggerMark(form: UnitForm | undefined): Node | null {
  const t = triggerTerm(form);
  if (!t) return null;
  const id = termIcon(t.id, t.status);
  const label = termDef(t.id)?.label ?? t.id;
  if (!id) return null;
  return h("span", { class: "trig tone-when", title: label, "aria-label": label, "data-testid": "card-trigger" }, icon(id, 12));
}

/** A form as one line of text: its authored text, else described from the
 * content's abilities (When → Who → Does, each Does in order). */
export function formText(form: UnitForm, content: MvpContent): string {
  return sharedFormText(form, content.abilities);
}

/** A form as highlighted text (R2-8): its terms tinted, iconed and tappable. */
export function formRich(form: UnitForm, content: MvpContent): Node[] {
  return richText(formSegments(form, content.abilities));
}

/** Everything about one unit: exact numbers and the form it has now. A
 * sleeping unit's sheet swaps in its awoken text behind "See Awoken" (what
 * changes underlined), a fused unit's parts open behind a tap, and the unit's
 * win and pick rates are one dim line at the bottom. opts.preview: a fusion
 * preview's credit line (./fusion.ts). Open it with overlay(unitSheet(...)) from ./dom. */
export function unitSheet(u: LineUnit | BattleUnit | UnitContent, content: MvpContent, opts: { rates?: UnitRates; from?: Stats; preview?: boolean } = {}): HTMLElement {
  const unitId = "forms" in u ? u.id : u.unitId;
  const unit = (id: string) => content.units.find((x) => x.id === id);
  // opts.from: your copy's stats now, when u is that copy after a buy (the shop's offer sheet).
  const statsLine = (s: Stats) =>
    opts.from
      ? h("div", { class: "num", "data-testid": "sheet-stats" }, `${opts.from.pwr} PWR / ${opts.from.hp} HP → ${s.pwr} PWR / ${s.hp} HP`)
      : h("div", { class: "num", "data-testid": "sheet-stats" }, `${s.pwr} PWR / ${s.hp} HP`);
  const fused = !("forms" in u) && u.kind === "fused";
  const c = fused ? undefined : unit(unitId);
  const now: UnitForm | undefined = "forms" in u ? u.forms.sleeping : u.recipe;
  const sleeping = !fused && ("forms" in u || u.form === "sleeping");
  const copies = "forms" in u ? 0 : u.copies;

  const box = h("div", { class: "sheet-form", "data-testid": "sheet-form" });
  const children: (Node | null)[] = [];
  if (fused && "fusion" in u && u.fusion) {
    const [a, b] = [unit(u.fusion.first), unit(u.fusion.second)];
    box.append(...formRich(u.recipe, content));
    const parts = h(
      "div",
      { class: "stack parts", "data-testid": "sheet-parts" },
      ...[
        [a, "When"],
        [b, "Who"],
      ].map(([p, role]) => (p ? h("div", { class: "sheet-form part" }, h("div", { class: "label" }, `${(p as UnitContent).emoji} ${(p as UnitContent).name} · Awoken · gives ${role}`), ...formRich((p as UnitContent).forms.awoken, content)) : null)),
    );
    parts.hidden = true;
    const label = `Made from ${a?.name ?? u.fusion.first} + ${b?.name ?? u.fusion.second}`;
    const toggle = h("button", { class: "small link", "data-testid": "sheet-parts-open" }, `${label} ▸`);
    toggle.addEventListener("click", () => {
      parts.hidden = !parts.hidden;
      toggle.textContent = `${label} ${parts.hidden ? "▸" : "▾"}`;
    });
    children.push(toggle, parts);
  } else if (now) {
    box.append(...formRich(now, content));
  }
  if (sleeping && c && now) {
    const left = Math.max(1, MVP_RULES.copiesToAwaken - copies);
    const sleepPieces = formSegments(now, content.abilities);
    const awokePieces = formSegments(c.forms.awoken, content.abilities);
    const see = `▸ See Awoken (${left} more ${left === 1 ? "copy" : "copies"})`;
    const back = "◂ Back to Sleeping (now)";
    const note = h("div", { class: "dim small" }, "What changes is underlined.");
    note.hidden = true;
    const btn = h("button", { class: "see-awoken", "data-testid": "see-awoken" }, see);
    let showing = false;
    btn.addEventListener("click", () => {
      showing = !showing;
      box.classList.toggle("other", showing);
      box.replaceChildren(...(showing ? [h("div", { class: "label" }, `Awoken · after copy ${MVP_RULES.copiesToAwaken}`), ...richText(awokePieces, { content: markChangedPieces(sleepPieces, awokePieces) })] : richText(sleepPieces)));
      btn.textContent = showing ? back : see;
      btn.dataset.testid = showing ? "see-sleeping" : "see-awoken";
      note.hidden = !showing;
    });
    children.push(btn, note);
  }

  return h(
    "div",
    { class: "stack", "data-testid": "unit-sheet" },
    h("div", { class: "row spread sheet-head" }, h("h2", {}, `${u.emoji} ${u.name}`), h("span", { class: "dim small", "data-testid": "sheet-state" }, "stats" in u ? sheetState(u) : `Sleeping · tier ${u.tier}`)),
    "forms" in u ? null : discoveredLine(u, { preview: opts.preview ?? false }),
    "stats" in u ? statsLine(u.stats) : h("div", { class: "num" }, `${u.base.pwr} PWR / ${u.base.hp} HP`),
    opts.from ? h("div", { class: "dim small" }, "Your copy now → after buying") : null,
    box,
    ...children,
    fused ? null : unitStatsLine(unitId, opts.rates),
  );
}

/** Form and copies; a fused unit says it is final (its credit is discoveredLine). */
function sheetState(u: LineUnit | BattleUnit): string {
  if (u.kind === "fused") return `Fused, final · ×${u.copies}`;
  return `${u.form === "awoken" ? "Awoken" : "Sleeping"} · ×${u.copies}`;
}
