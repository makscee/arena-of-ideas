// The unit card and the unit sheet (mission #574), shared by the shop and
// result screens (slice 8), the battle viewer (slice 9) and the stats page
// (slice 11). Slice 8 owns the look of both; slices 9 and 11 only pass
// options (live numbers, rates, onOpen), so nobody reshapes these signatures.
//
// Round 2 (R2-7): the card is compact (64×84 on a phone): a top row, the
// emoji, a one-line name, PWR/HP and one footer slot. Round 3 (R3-4): the top
// row is the form's When · Who · Does icon line, the tier numeral top right. The sheet shows only the
// form the unit has now; win and pick rates are its one dim last line.
import { cardIcons, type Pip } from "../../src/mvp/card-icons";
import { formSegments, formText as sharedFormText } from "../../src/mvp/form-text";
import { MVP_RULES, type BattleUnit, type UnitCredit, type VersionCredit, type LineUnit, type MvpContent, type SummonContent, type UnitContent, type UnitForm } from "../../src/mvp/contract";
import { summonId } from "../../src/describe";
import type { AbilityRegistry, Stats } from "../../src/types";
import { t } from "../i18n";
import { closable, h, who } from "./dom";
import { discoveredLine } from "./fusion";
import { icon } from "./icon";
import { roman } from "./roman";
import { markChangedPieces, richText } from "./term";
import { unitStatsLine, type UnitRates } from "./unit-stats";
import { rulesLangOpt } from "../lang";
import { unitName } from "../unit-names";

export { roman };

/** What a card needs to draw; LineUnit, BattleUnit and offers all fit. */
export type CardUnit = { emoji: string; name: string; stats: Stats } & Partial<Pick<LineUnit, "unitId" | "kind" | "form" | "copies" | "fusion" | "recipe">>;

export interface CardOptions {
  side: "you" | "ghost";
  /** The footer slot: copies pips, AWOKEN, FUSED, a price, or battle statuses. */
  extra?: (Node | null)[];
  testid?: string;
  /** The battle viewer's live state (slice 9): current stats, dead, the acting
   * unit lit. With maxHp (R2-13) the card adds an HP bar and draws PWR and HP
   * big, each with its icon, HP red once the unit is hurt. */
  live?: { stats: Stats; maxHp?: number; dead?: boolean; acting?: boolean };
  /** An offer's tier, drawn as a Roman numeral top-right; "S" marks a summoned unit (R3-5). */
  tier?: number | "S";
  /** Tapping the card opens this, usually overlay(unitSheet(...)). */
  onOpen?: () => void;
  /** M2-6: a candidate's card: its PWR / HP are set by simulation, so it shows none. */
  unset?: boolean;
  /** M3-8: a unit the live credits don't know (the Library's): its archetype
   * version, and no live credit (NEW, 💡). */
  version?: number;
}

/** A tier's colour class (R4-4): .t1–.t4 on the --tier-1..4 tokens, .ts for a summoned unit. */
export const tierClass = (tier: number | "S") => (tier === "S" ? "ts" : `t${tier}`);

// A pool unit's sheet head, "Sleeping · Tier II", the numeral in its tier's colour.
const poolState = (form: string, tier: number) => [t("card.tierOf", { form }), h("span", { class: `tier ${tierClass(tier)}` }, roman(tier))];
// An idea's candidate (M2-6) has no tier yet: the simulation sets it with the numbers.
const SET_BY_SIM = () => t("card.setBySim");

/** The live units' credits (M2-9: who the idea was, NEW), set once the
 * content loads (../content.ts) and again after the dev tool credits one. */
let credits = new Map<string, UnitCredit>();
export function setCardCredits(list: UnitCredit[]): void {
  credits = new Map(list.map((c) => [c.unitId, c]));
}
export const creditOf = (unitId: string | undefined): UnitCredit | undefined => (unitId ? credits.get(unitId) : undefined);

/** The live unit a stored version id is served as (M3-8: a version keeps its
 * root's name, so the pool may serve it under its root's id). */
export function liveIdOf(storedId: string): string | undefined {
  for (const c of credits.values()) if (c.versions[c.version - 1]?.unitId === storedId) return c.unitId;
  return credits.has(storedId) ? storedId : undefined;
}

export const daysLive = (n: number): string => t("card.daysLive", { n });

/** A unit's credit (M2-9, M3-8), one dim line:
 * "NEW 💡 idea by @a, evolved by @b · v2 · 23 days live". A seed root evolved
 * says "evolved by @b" alone; a seed unit only its days live. */
export function creditText(c: VersionCredit & { isNew?: boolean; liveDays: number | null }, testid = "sheet-credit"): HTMLElement | null {
  const by: (Node | string)[] = [
    ...(c.by ? [t("card.ideaBy"), who(c.by.name)] : []),
    ...(c.evolvedBy ? [c.by ? t("card.andEvolvedBy") : t("card.evolvedBy"), who(c.evolvedBy.name)] : []),
  ];
  const parts: (Node | string)[][] = [by, c.version > 1 ? [`v${c.version}`] : [], c.liveDays !== null ? [h("span", { "data-testid": "sheet-days" }, daysLive(c.liveDays))] : []].filter((p) => p.length);
  if (!parts.length && !c.isNew) return null;
  return h(
    "div",
    { class: "dim small credit-line", "data-testid": testid },
    ...(c.isNew ? [h("span", { class: "new-badge inline", "data-testid": "sheet-new" }, "NEW")] : []),
    ...parts.flatMap((p, i) => (i ? [" · ", ...p] : p)),
  );
}

/** A pool unit's sheet line (creditText); none before the credits load. */
function creditLine(unitId: string): HTMLElement | null {
  const c = creditOf(unitId);
  return c ? creditText(c) : null;
}

export function card(u: CardUnit, o: CardOptions): HTMLElement {
  // M4-4: a line or battle unit comes named in English; a fused one keeps its name.
  if (u.kind !== "fused") u = { ...u, name: unitName(u.unitId, u.name) };
  const stats = o.live?.stats ?? u.stats;
  // A fused unit is its finders' (discoveredLine); its parts' credits stay on theirs.
  // A Library card's unit isn't live: no live credit, even when a live version shares its id.
  const credit = u.kind === "fused" || o.version !== undefined ? undefined : creditOf(u.unitId);
  const version = o.version ?? credit?.version ?? 1;
  const el = h(
    "div",
    { class: `card ${o.side}`, ...(o.testid ? { "data-testid": o.testid } : {}) },
    iconLine(u.recipe, !!o.tier),
    o.tier ? h("span", { class: `tier ${tierClass(o.tier)}`, "aria-label": o.tier === "S" ? t("card.summonedAria") : t("card.tierAria", { tier: o.tier }) }, o.tier === "S" ? "S" : roman(o.tier)) : null,
    credit?.isNew ? h("span", { class: "new-badge", "data-testid": "card-new" }, "NEW") : null,
    h("div", { class: "emoji" }, u.emoji),
    // One line; ui/dom.ts fitText() shrinks a long name a little, then cuts it.
    // A version that isn't its archetype's first says so beside it (M3-8).
    h("div", { class: "name", title: u.name }, u.name, ...(version > 1 ? [h("span", { class: "ver", "data-testid": "card-version" }, ` v${version}`)] : [])),
    ...(o.unset
      ? [h("div", { class: "stats unset", title: SET_BY_SIM() }, h("span", { class: "p" }, "?"), "/", h("span", { class: "h" }, "?"))]
      : o.live?.maxHp !== undefined
        ? liveStats(stats, o.live.maxHp)
        : [h("div", { class: "stats" }, h("span", { class: "p" }, `${stats.pwr}`), "/", h("span", { class: "h" }, `${stats.hp}`))]),
    h("div", { class: "foot" }, ...(o.extra ?? [])),
  );
  if (u.kind === "fused") el.classList.add("fused");
  else if (u.form === "awoken") el.classList.add("awoken");
  if (o.live?.dead) el.classList.add("dead");
  if (o.live?.acting) el.classList.add("acting");
  if (credit?.by) {
    // No room on a 64px card for the words: a 💡 mark, the words on hover and for readers.
    el.dataset.by = credit.by.name;
    el.title = `${u.name}: idea by @${credit.by.name}`;
    el.append(h("span", { class: "by-mark", "data-testid": "card-by", "aria-label": t("card.ideaByAria", { name: credit.by.name }) }, "💡"));
  }
  if (o.onOpen) el.addEventListener("click", o.onOpen);
  return el;
}

/** A battle card's numbers (R2-13): an HP bar, then PWR and HP, big. */
function liveStats(stats: Stats, maxHp: number): Node[] {
  const max = Math.max(1, maxHp, stats.hp);
  const pct = Math.round((Math.max(0, stats.hp) / max) * 100);
  const bar = h("div", { class: "hpbar", "data-testid": "hp-bar", role: "meter", "aria-valuemin": "0", "aria-valuemax": String(max), "aria-valuenow": String(stats.hp), "aria-label": `${stats.hp} of ${max} HP` }, h("i", {}));
  (bar.firstChild as HTMLElement).style.width = `${pct}%`;
  if (pct <= 34) bar.classList.add("low");
  return [
    bar,
    h(
      "div",
      { class: "stats big" },
      h("span", { class: "p", title: "PWR" }, icon("broadsword", 11, "stat-ic"), `${stats.pwr}`),
      h("span", { class: `h${stats.hp < max ? " hurt" : ""}`, title: "HP" }, icon("hearts", 11, "stat-ic"), `${stats.hp}`),
    ),
  ];
}

/** The content's abilities, so a card can read its form's Does (set once the
 * content loads: ../content.ts). */
let abilities: AbilityRegistry = {};
export function setCardAbilities(a: AbilityRegistry): void {
  abilities = a;
}
/** Adds abilities the content lacks (a vote card's candidate, M2-8). */
export function addCardAbilities(a: AbilityRegistry): void {
  abilities = { ...abilities, ...a };
}

/** Icons past these counts fold into "+" (phone) or "+n" (desktop; "+n" past
 * 3 on the 1024–1279px shop, whose cards are narrower); style.css hides the
 * rest by width (docs/round3/words.md (8)). */
const PHONE_ICONS = 3;
const DESKTOP_ICONS = 5;

/** A 4px pip on a When icon's corner: whose event it is (teal an ally's, pink
 * an enemy's, dim either side's; none its own). Shared with sentence pills
 * and the Codex trigger filter. */
export function withPip(ic: Element, pip: Pip | undefined): Element {
  if (!pip) return ic;
  return h("span", { class: "pipped" }, ic, h("i", { class: `pip pip-${pip}`, "aria-hidden": "true" }));
}

/** The card's top row: When, Who, then each Does, one icon per idea, each in
 * its tone. Hovering names them all ("Battle start · Front enemy · Freeze"). */
function iconLine(form: UnitForm | undefined, tiered: boolean): Node | null {
  const icons = form ? cardIcons(form, abilities, rulesLangOpt()) : [];
  if (!icons.length) return null;
  const names = icons.map((c) => c.label).join(" · ");
  return h(
    "span",
    { class: tiered ? "icons tiered" : "icons", title: names, "aria-label": names, "data-testid": "card-icons" },
    ...icons.map((c) => h("span", { class: `ci tone-${c.tone}`, title: c.label, "data-icon": c.icon, ...(c.pip ? { "data-pip": c.pip } : {}) }, withPip(icon(c.icon, 11), c.pip))),
    icons.length > PHONE_ICONS ? h("small", { class: "more phone-more", "aria-hidden": "true" }, "+") : null,
    icons.length > DESKTOP_ICONS ? h("small", { class: "more desk-more", "aria-hidden": "true" }, `+${icons.length - DESKTOP_ICONS}`) : null,
    icons.length > PHONE_ICONS ? h("small", { class: "more mid-more", "aria-hidden": "true" }, `+${icons.length - PHONE_ICONS}`) : null,
  );
}

/** The card's icons, each with its name ("Battle start", "Front enemy",
 * "Damage"): a phone has no hover, so the sheet says what they mean. */
export function iconKey(form: UnitForm | undefined): HTMLElement | null {
  const icons = form ? cardIcons(form, abilities, rulesLangOpt()) : [];
  if (!icons.length) return null;
  return h("div", { class: "sheet-icons", "data-testid": "sheet-icons" }, ...icons.map((c) => h("span", { class: `ik tone-${c.tone}` }, withPip(icon(c.icon, 14), c.pip), h("span", { class: "ik-name" }, c.label))));
}

/** A form as one line of text: its authored text, else described from the
 * content's abilities (When → Who → Does, each Does in order). */
export function formText(form: UnitForm, content: MvpContent): string {
  return sharedFormText(form, content.abilities, rulesLangOpt());
}

/** A form as highlighted text (R2-8): its terms tinted, iconed and tappable. */
export function formRich(form: UnitForm, content: MvpContent): Node[] {
  return richText(formSegments(form, content.abilities, rulesLangOpt()));
}

/** Everything about one unit: exact numbers and the form it has now. A
 * sleeping unit's sheet swaps in its awoken text behind "See Awoken" (what
 * changes underlined), a fused unit's parts open behind a tap, and the unit's
 * win and pick rates are one dim line at the bottom. opts.preview: a fusion
 * preview's credit line (./fusion.ts). opts.credit: a Library unit's credit
 * line. Open it with overlay(unitSheet(...)) from ./dom. */
export function unitSheet(
  u: LineUnit | BattleUnit | UnitContent,
  content: MvpContent,
  opts: { rates?: UnitRates; from?: Stats; preview?: boolean; candidate?: boolean; credit?: Parameters<typeof creditText>[0] } = {},
): HTMLElement {
  const unitId = "forms" in u ? u.id : u.unitId;
  // A unit that left the pool (a champion's, a replay's) is in content.left (M2-2).
  const unit = (id: string) => content.units.find((x) => x.id === id) ?? content.left?.find((x) => x.id === id);
  // opts.from: your copy's stats now, when u is that copy after a buy (the shop's offer sheet).
  const statsLine = (s: Stats) =>
    opts.from
      ? h("div", { class: "num", "data-testid": "sheet-stats" }, `${t("card.stats", { pwr: opts.from.pwr, hp: opts.from.hp })} → ${t("card.stats", { pwr: s.pwr, hp: s.hp })}`)
      : h("div", { class: "num", "data-testid": "sheet-stats" }, t("card.stats", { pwr: s.pwr, hp: s.hp }));
  const fused = !("forms" in u) && u.kind === "fused";
  const c = fused ? undefined : unit(unitId);
  const now: UnitForm | undefined = "forms" in u ? u.forms.sleeping : u.recipe;
  const sleeping = !fused && ("forms" in u || u.form === "sleeping");
  const copies = "forms" in u ? 0 : u.copies;

  const box = h("div", { class: "sheet-form", "data-testid": "sheet-form" });
  const headState = (form: "sleeping" | "awoken") => (opts.candidate ? [formName(form)] : "stats" in u ? [sheetState(u)] : poolState(formName(form), u.tier));
  const state = h("span", { class: "dim small", "data-testid": "sheet-state" }, ...headState("sleeping"));
  const children: (Node | null)[] = [];
  // What the form shown summons, under its text (R3-5); swapped by See Awoken.
  const summons = h("div", { class: "stack" });
  const shownForm = fused && "recipe" in u ? u.recipe : now;
  const icons = h("div", {}, ...[iconKey(shownForm)].filter((n): n is HTMLElement => n !== null));
  if (shownForm) summons.replaceChildren(...[summonsBlock(shownForm, content)].filter((n): n is HTMLElement => n !== null));
  if (fused && "fusion" in u && u.fusion) {
    const [a, b] = [unit(u.fusion.first), unit(u.fusion.second)];
    box.append(...formRich(u.recipe, content));
    const parts = h(
      "div",
      { class: "stack parts", "data-testid": "sheet-parts" },
      ...[
        [a, t("card.when")],
        [b, t("card.who")],
      ].map(([p, role]) => (p ? h("div", { class: "sheet-form part" }, h("div", { class: "label" }, t("card.partGives", { unit: `${(p as UnitContent).emoji} ${(p as UnitContent).name}`, role: role as string })), ...formRich((p as UnitContent).forms.awoken, content)) : null)),
    );
    parts.hidden = true;
    const label = t("card.madeFrom", { a: a?.name ?? u.fusion.first, b: b?.name ?? u.fusion.second });
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
    const sleepPieces = formSegments(now, content.abilities, rulesLangOpt());
    const awokePieces = formSegments(c.forms.awoken, content.abilities, rulesLangOpt());
    // Short enough for one line in the 1024px inspector (R2-17).
    const see = opts.candidate ? t("card.seeAwoken") : t("card.awokenIn", { n: left });
    const back = t("card.backToSleeping");
    const note = h("div", { class: "dim small" }, t("card.changesUnderlined"));
    note.hidden = true;
    const btn = h("button", { class: "see-awoken", "data-testid": "see-awoken" }, see);
    let showing = false;
    btn.addEventListener("click", () => {
      showing = !showing;
      // The Summons block follows the form shown: Planter's Imp, its awoken Treant.
      summons.replaceChildren(...[summonsBlock(showing ? c.forms.awoken : now, content)].filter((n): n is HTMLElement => n !== null));
      icons.replaceChildren(...[iconKey(showing ? c.forms.awoken : now)].filter((n): n is HTMLElement => n !== null));
      box.classList.toggle("other", showing);
      box.replaceChildren(...(showing ? [h("div", { class: "label" }, t("card.awokenAfter", { n: MVP_RULES.copiesToAwaken })), ...richText(awokePieces, { content: markChangedPieces(sleepPieces, awokePieces) })] : richText(sleepPieces)));
      btn.textContent = showing ? back : see;
      // A unit from the pool (the Codex, an offer) heads its sheet with the form shown.
      if (!("stats" in u)) state.replaceChildren(...headState(showing ? "awoken" : "sleeping"));
      btn.dataset.testid = showing ? "see-sleeping" : "see-awoken";
      note.hidden = !showing;
    });
    children.push(btn, note);
  }

  return h(
    "div",
    { class: "stack", "data-testid": "unit-sheet" },
    h("div", { class: "row spread sheet-head" }, h("h2", {}, `${u.emoji} ${fused ? u.name : unitName(unitId, u.name)}`), state),
    // What the unit is about, in one sentence (R4-8); a fused unit has none.
    c?.archetype ? h("div", { class: "archetype", "data-testid": "sheet-archetype" }, c.archetype) : null,
    // opts.credit: a Library unit's own (M3-8), not a live version's that shares its id.
    fused ? null : opts.credit ? creditText(opts.credit) : creditLine(unitId),
    "forms" in u ? null : discoveredLine(u, { preview: opts.preview ?? false }),
    opts.candidate ? h("div", { class: "dim small", "data-testid": "sheet-unset" }, SET_BY_SIM()) : "stats" in u ? statsLine(u.stats) : h("div", { class: "num" }, t("card.stats", { pwr: u.base.pwr, hp: u.base.hp })),
    opts.from ? h("div", { class: "dim small" }, t("card.yourCopyNow")) : null,
    icons.childNodes.length ? icons : null,
    box,
    ...children,
    summons.childNodes.length || (sleeping && c) ? summons : null,
    fused || opts.candidate ? null : unitStatsLine(unitId, opts.rates),
  );
}

// ---------- summoned units (R3-5, docs/round3/words.md (3)) ----------

/** A summon by its id; the content's summons (none in content before round 3). */
export function summonById(content: MvpContent, id: string): SummonContent | undefined {
  return content.summons?.find((s) => s.id === id);
}

/** What a form summons, in its Does order, once each. */
export function summonsOf(form: UnitForm, content: MvpContent): SummonContent[] {
  const out: SummonContent[] = [];
  for (const d of [...form.does, ...(form.also ?? []).flatMap((c) => c.does)])
    for (const e of content.abilities[d]?.effects ?? []) {
      const s = e.kind === "summon" ? summonById(content, summonId(e.unit.name)) : undefined;
      if (s && !out.includes(s)) out.push(s);
    }
  return out;
}

/** The units that summon it, and in which of their forms. */
export function summonersOf(content: MvpContent, id: string): { unit: UnitContent; forms: ("sleeping" | "awoken")[] }[] {
  return content.units.flatMap((unit) => {
    const forms = (["sleeping", "awoken"] as const).filter((f) => summonsOf(unit.forms[f], content).some((s) => s.id === id));
    return forms.length ? [{ unit, forms: [...forms] }] : [];
  });
}

/** A summon drawn as a card: its emoji, name, numbers and icon line. */
export function summonCard(s: SummonContent, o: CardOptions): HTMLElement {
  return card({ emoji: s.emoji, name: s.name, stats: s.base, ...(s.form ? { recipe: s.form } : {}) }, o);
}

/** A summon's text, or the line that says it has none. */
export function summonText(s: SummonContent, content: MvpContent): Node[] {
  return s.form ? formRich(s.form, content) : [h("b", {}, t("card.noAbility"))];
}

/** A summoner's "Summons" block: each summoned unit's compact card and its
 * text, inline, so nothing needs a second tap. Null when the form summons nothing. */
function summonsBlock(form: UnitForm, content: MvpContent): HTMLElement | null {
  const list = summonsOf(form, content);
  if (!list.length) return null;
  return h(
    "div",
    { class: "stack summons", "data-testid": "sheet-summons" },
    h("div", { class: "label" }, t("card.summons")),
    ...list.map((s) =>
      h(
        "div",
        { class: "summon-row", "data-testid": "sheet-summon", "data-summon": s.id },
        summonCard(s, { side: "you", testid: "summon-card", onOpen: () => openSummon(s, content) }),
        h("div", { class: "sheet-form" }, ...summonText(s, content)),
      ),
    ),
  );
}

/** Opens a summon's sheet over whatever is open. */
export function openSummon(s: SummonContent, content: MvpContent): void {
  closable(summonSheet(s, content));
}

/** A summoned unit's own sheet: name, emoji, PWR / HP, its text (or the
 * no-ability line), and "Summoned by" chips that open those units. */
export function summonSheet(s: SummonContent, content: MvpContent): HTMLElement {
  const by = summonersOf(content, s.id);
  return h(
    "div",
    { class: "stack", "data-testid": "summon-sheet", "data-summon": s.id },
    h("div", { class: "row spread sheet-head" }, h("h2", {}, `${s.emoji} ${s.name}`), h("span", { class: "dim small", "data-testid": "sheet-state" }, t("card.summoned"))),
    h("div", { class: "num", "data-testid": "sheet-stats" }, t("card.stats", { pwr: s.base.pwr, hp: s.base.hp })),
    h("div", { class: "sheet-form", "data-testid": "sheet-form" }, ...summonText(s, content)),
    by.length ? h("div", { class: "label" }, t("card.summonedBy")) : null,
    by.length
      ? h(
          "div",
          { class: "chips", "data-testid": "summoned-by" },
          ...by.map(({ unit, forms }) => {
            const b = h("button", { class: "chip unit-chip", "data-testid": "summoned-by-unit", "data-unit": unit.id }, `${unit.emoji} ${unit.name}${forms.length === 1 && forms[0] === "awoken" ? t("card.awokenSuffix") : ""}`);
            b.addEventListener("click", () => closable(unitSheet(unit, content)));
            return b;
          }),
        )
      : null,
  );
}

/** Form and copies; a fused unit says it is final (its credit is discoveredLine). */
function sheetState(u: LineUnit | BattleUnit): string {
  if (u.kind === "fused") return t("card.fusedFinal", { n: u.copies });
  return t("card.formCopies", { form: formName(u.form), n: u.copies });
}

/** A form's name: "Sleeping", "Awoken". */
export function formName(form: "sleeping" | "awoken"): string {
  return form === "awoken" ? t("card.awoken") : t("card.sleeping");
}
