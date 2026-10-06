// The Codex (round 2, R2-11; docs/round2/ui.md (e), icons.md "Codex glossary
// entry", mechanics.md (4)). Reached from the title menu, the in-run menu and
// every term's "Open in Codex"; Back returns to the screen it opened over.
// Three tabs:
// - Units: all the content's units as compact cards, filtered by tier, by the
//   When a card's icon line leads with (its pip tells "Dies" from "Ally
//   dies") and by a search over names and both forms'
//   text. A card opens its sheet (active form, See Awoken, the dim rates line).
//   A quiet Sort (tier, win rate, pick rate) compares the rates: only while a
//   rate sort is on does each card show its number, dim.
// - Fusions: every discovered pair with its recipe and its credit, "N of 6,480
//   found"; picking a unit lists all its pairs as the first part, the ones
//   nobody has made as "?".
// - Keywords: every glossary term with its 48px icon, its rule and the units
//   whose text uses it; a term's deep link lands on its row. A trigger said of
//   someone else ("After an enemy dies") has its own line there.
// On desktop a sheet opens in the inspector on the right, not an overlay.
// The icon credits (CC BY 3.0) sit at its foot. Reads /content, /stats and
// /fusions, each once while it stays open (CodexCache).
import { GLOSSARY, STATUS_TERMS, scopedLabel, scopedTip, termDef, termGroup, type FixedTermId, type IconId, type TermGroup, type TermId } from "../../src/glossary";
import type { FusionDiscovery, LineUnit, MvpContent, StatsView, UnitContent, UnitId } from "../../src/mvp/contract";
import type { UnitFilter } from "../../src/types";
import { formSegments, formText } from "../../src/mvp/form-text";
import { fuseUnits, lineUnitOf } from "../../src/mvp/forms";
import { cardIcons, type Pip } from "../../src/mvp/card-icons";
import type { AbilityRegistry } from "../../src/types";
import { api } from "../api";
import { card, formRich, roman, summonCard, summonSheet, unitSheet, withPip } from "../ui/card";
import { app, button, closable, h, isDesktop, onKeys, screen, show, who } from "../ui/dom";
import { icon } from "../ui/icon";
import { loadUnitRates, pct } from "../ui/unit-stats";

export type CodexTab = "units" | "fusions" | "keywords";
export type CodexSort = "tier" | "win" | "pick";

export interface CodexState {
  tab: CodexTab;
  /** Units: one tier, the summoned units (R3-5), or every tier. */
  tier: number | "summoned" | null;
  /** Units: the When a card's icon line leads with (its icon, plus its pip:
   * "death-skull.ally"), or any. */
  trigger: string | null;
  query: string;
  /** Units: by tier, or by win or pick rate (the rates show on the cards only then). */
  sort: CodexSort;
  /** Fusions: one unit's pairs (as the first part), or every discovery. */
  fusionsOf: UnitId | null;
  mine: boolean;
  /** Keywords: the row to land on, and a trigger's scope line in it. */
  term?: TermId | undefined;
  scope?: UnitFilter | undefined;
}

const DEFAULTS: CodexState = { tab: "units", tier: null, trigger: null, query: "", sort: "tier", fusionsOf: null, mine: false };

/** What the Codex fetched, kept while it stays open (main.ts makes a new one
 * each time it opens over another screen): a filter tap or a tab switch
 * never refetches. */
export interface CodexCache {
  stats?: Promise<StatsView | null>;
  fusions?: Promise<FusionDiscovery[]>;
  /** Each tab's window scroll, put back on a switch. */
  scroll: Partial<Record<CodexTab, number>>;
  /** Where the open Codex is (its tab and filters), for a redraw at 1024px. */
  state?: CodexState;
}
export const newCodexCache = (): CodexCache => ({ scroll: {} });

/** Draws the Codex once; a tab or filter change redraws only its body and
 * keeps the scroll. On desktop (R2-9) a unit or a fusion opens in the
 * inspector on the right, never in an overlay (ui.md (e)). */
export async function codexScreen(a: { content: MvpContent; onBack: () => void; state?: Partial<CodexState> | undefined; cache?: CodexCache }): Promise<void> {
  const cache = a.cache ?? newCodexCache();
  cache.stats ??= loadUnitRates(); // the dim rates line on unit sheets, and the rate sorts
  const st: CodexState = { ...DEFAULTS, ...a.state };
  cache.state = st;
  const desk = isDesktop();
  const back = button("Back", a.onBack, "primary grow", "codex-back");
  const tabs = h("div", { class: "tabs", role: "tablist" });
  const body = h("div", { class: "stack codex-body" });
  const inspector = desk ? h("aside", { class: "codex-insp stack", "data-testid": "inspector" }) : null;
  /** A unit or a fusion is in the inspector: Esc clears it before Back. */
  let inspected = false;
  const idle = () => {
    inspected = false;
    for (const el of app.querySelectorAll(".codex-body .inspected")) el.classList.remove("inspected");
    inspector?.replaceChildren(h("div", { class: "dim" }, "Pick a unit or a fusion to read it here."));
  };
  idle();
  /** A unit's (or a fused unit's) sheet: the inspector on desktop, an overlay on a phone. */
  const open = (node: HTMLElement, from?: HTMLElement): void => {
    if (!inspector) return void closable(node);
    for (const el of app.querySelectorAll(".codex-body .inspected")) el.classList.remove("inspected");
    from?.classList.add("inspected");
    inspector.replaceChildren(node);
    inspector.scrollTop = 0;
    inspected = true;
  };

  let drawing = 0;
  const draw = async (): Promise<void> => {
    const n = ++drawing;
    tabs.replaceChildren(tabBtn("units", "Units"), tabBtn("fusions", "Fusions"), tabBtn("keywords", "Keywords"));
    // The body's height stays while it redraws, so the window keeps its scroll.
    body.style.minHeight = `${body.offsetHeight}px`;
    const y = window.scrollY;
    let kids: Node[];
    try {
      if (st.tab === "units") kids = unitsTab(a.content, st, set, open, st.sort === "tier" ? null : await rates(), retryRates);
      else if (st.tab === "keywords") kids = [keywordsTab(a.content, open)];
      else {
        // The first draw of Fusions waits on /fusions: say so meanwhile (R2-17).
        if (!body.hasChildNodes()) body.replaceChildren(h("div", { class: "dim", "data-testid": "codex-loading" }, "Loading fusions…"));
        kids = fusionsTab(a.content, await (cache.fusions ??= api.fusions()), st, set, open);
      }
    } catch (e) {
      delete cache.fusions; // a failed fetch is tried again on the next draw
      kids = [h("div", { class: "error", "data-testid": "error" }, e instanceof Error ? e.message : String(e))];
    }
    if (n !== drawing || !body.isConnected) return; // redrawn or left meanwhile
    body.replaceChildren(...kids);
    window.scrollTo(0, y);
    body.style.minHeight = "";
    if (st.tab === "keywords" && st.term) landOn(body, st.term, st.scope);
  };
  /** The rates for a rate sort; a fetch that failed (null) is tried again (R2-17). */
  const rates = async (): Promise<StatsView | null> => (await cache.stats!) ?? (await (cache.stats = loadUnitRates()));
  const retryRates = () => {
    delete cache.stats;
    cache.stats = loadUnitRates();
    void draw();
  };
  /** A filter, sort or fusions change: the same tab, redrawn where it is. */
  const set = (next: Partial<CodexState>): void => {
    Object.assign(st, { term: undefined, scope: undefined }, next);
    void draw();
  };
  const tabBtn = (t: CodexTab, label: string) =>
    button(
      label,
      () => {
        if (t === st.tab) return;
        cache.scroll[st.tab] = window.scrollY;
        Object.assign(st, { tab: t, term: undefined, scope: undefined });
        body.style.minHeight = "";
        body.replaceChildren();
        void draw().then(() => window.scrollTo(0, cache.scroll[t] ?? 0));
      },
      t === st.tab ? "on" : "",
      `codex-tab-${t}`,
    );

  const main = h(
    "div",
    { class: "stack codex-main" },
    h("div", { class: "row spread" }, h("h1", {}, "CODEX"), h("span", { class: "dim small" }, `${a.content.units.length} units`)),
    tabs,
    body,
    credits(),
    h("div", { class: "spacer" }),
    h("div", { class: "row footer" }, back),
  );
  show(main, inspector);
  screen("codex");
  // Esc (after a sheet, closed in ui/dom.ts): clears the inspector, then Back.
  onKeys((e) => {
    if (e.key !== "Escape") return false;
    if (inspected) idle();
    else a.onBack();
    return true;
  });
  await draw();
}

// ---------- units ----------

/** The When icon a unit's card leads with (card.ts iconLine), its pip and
 * its label. The key tells "Dies" from "Ally dies" ("death-skull.ally"). */
function triggerOf(u: UnitContent, abilities: AbilityRegistry): { key: string; icon: IconId; pip?: Pip; label: string } | null {
  const w = cardIcons(u.forms.sleeping, abilities)[0];
  if (!w || w.role !== "when") return null;
  return { key: w.pip ? `${w.icon}.${w.pip}` : w.icon, icon: w.icon, ...(w.pip ? { pip: w.pip } : {}), label: w.label };
}

const byTierName = (x: UnitContent, y: UnitContent) => x.tier - y.tier || x.name.localeCompare(y.name);

const SORTS: [CodexSort, string][] = [
  ["tier", "Tier"],
  ["win", "Win rate"],
  ["pick", "Pick rate"],
];

function unitsTab(
  content: MvpContent,
  st: CodexState,
  set: (next: Partial<CodexState>) => void,
  open: (node: HTMLElement, from?: HTMLElement) => void,
  stats: StatsView | null,
  retry: () => void,
): Node[] {
  // A rate sort: highest first, units no run has counted yet last (by tier).
  const rates = new Map((stats?.units ?? []).filter((r) => r.runs > 0).map((r) => [r.unitId, r]));
  const rateOf = (u: UnitContent): number | undefined => {
    const r = rates.get(u.id);
    return r && (st.sort === "win" ? r.winRate : r.pickRate);
  };
  const units = [...content.units].sort(byTierName);
  if (st.sort !== "tier") units.sort((x, y) => (rateOf(y) ?? -1) - (rateOf(x) ?? -1));
  const tiers = [...new Set(units.map((u) => u.tier))].sort();
  // The trigger icons the picked tier has (the picked one always stays).
  const trig = new Map(units.map((u) => [u.id, triggerOf(u, content.abilities)]));
  const triggers = new Map<string, NonNullable<ReturnType<typeof triggerOf>>>();
  for (const u of [...units].sort(byTierName).filter((x) => st.tier === null || x.tier === st.tier || trig.get(x.id)?.key === st.trigger)) {
    const t = trig.get(u.id);
    if (t && !triggers.has(t.key)) triggers.set(t.key, t);
  }
  const text = new Map(units.map((u) => [u.id, `${u.name} ${formText(u.forms.sleeping, content.abilities)} ${formText(u.forms.awoken, content.abilities)}`.toLowerCase()]));

  const summons = content.summons ?? [];
  const summonText = new Map(summons.map((x) => [x.id, `${x.name} ${x.form ? formText(x.form, content.abilities) : ""}`.toLowerCase()]));
  const grid = h("div", { class: "slots codex-grid", "data-testid": "codex-units" });
  const count = h("div", { class: "dim small", "data-testid": "codex-count" });
  const order = st.sort === "tier" ? "by tier" : `by ${st.sort === "win" ? "win" : "pick"} rate, highest first`;
  const draw = () => {
    const q = st.query.trim().toLowerCase();
    const shown = units.filter((u) => st.tier !== "summoned" && (st.tier === null || u.tier === st.tier) && (st.trigger === null || trig.get(u.id)?.key === st.trigger) && (!q || text.get(u.id)!.includes(q)));
    grid.replaceChildren(
      ...shown.map((u) => {
        const r = st.sort === "tier" ? undefined : rateOf(u);
        // The rate shows on a card only while a rate sort is on: a quiet number under it.
        const extra = st.sort === "tier" ? [] : [h("span", { class: "codex-rate", "data-testid": "codex-rate" }, r === undefined ? "–" : pct(r))];
        const el = card({ emoji: u.emoji, name: u.name, stats: u.base, recipe: u.forms.sleeping, unitId: u.id }, { side: "you", tier: u.tier, testid: "codex-unit", extra, onOpen: () => open(unitSheet(u, content), el) });
        return el;
      }),
    );
    // The summoned units, a group after tier IV, tagged "S" (R3-5). A When
    // filter hides them: they have none of their own.
    const sums = st.tier === null || st.tier === "summoned" ? summons.filter((x) => st.trigger === null && (!q || summonText.get(x.id)!.includes(q))) : [];
    if (sums.length)
      grid.append(
        h("div", { class: "label codex-group", "data-testid": "codex-summoned" }, "Summoned"),
        ...sums.map((x) => {
          const el = summonCard(x, { side: "you", tier: "S", testid: "codex-summon", onOpen: () => open(summonSheet(x, content), el) });
          el.dataset.summon = x.id;
          return el;
        }),
      );
    count.textContent =
      st.tier === "summoned"
        ? `${sums.length} summoned units: other units bring them into battle. Tap one to read it.`
        : shown.length === units.length
          ? `All ${units.length} units, ${order}, then ${summons.length} summoned. Tap one to read it.`
          : `${shown.length} of ${units.length} units, ${order}`;
    if (!shown.length && !sums.length) grid.append(h("div", { class: "dim codex-none" }, "No unit matches."));
  };

  const tierRow = h(
    "div",
    { class: "row codex-filter", "data-testid": "codex-tiers" },
    h("span", { class: "label" }, "Tier"),
    ...[null, ...tiers].map((t) => button(t === null ? "All" : roman(t), () => set({ tier: t }), st.tier === t ? "chip on" : "chip", `codex-tier-${t ?? "all"}`)),
    summons.length ? button("Summoned", () => set({ tier: "summoned", trigger: null }), st.tier === "summoned" ? "chip on" : "chip", "codex-tier-summoned") : null,
  );
  const trigRow = h(
    "div",
    { class: "row codex-filter codex-triggers", "data-testid": "codex-triggers" },
    ...[...triggers].map(([key, t]) => {
      const b = button("", () => set({ trigger: st.trigger === key ? null : key }), st.trigger === key ? "chip icon on tone-when" : "chip icon tone-when", `codex-trigger-${key.replace(".", "-")}`);
      b.title = t.label;
      b.setAttribute("aria-label", t.label);
      b.setAttribute("aria-pressed", String(st.trigger === key));
      b.append(withPip(icon(t.icon, 22), t.pip));
      return b;
    }),
  );
  // Sort: quiet, at the end; rates are a hint (docs/round2/README.md).
  const sortRow = h(
    "div",
    { class: "row codex-filter codex-sort", "data-testid": "codex-sort" },
    h("span", { class: "label" }, "Sort"),
    ...SORTS.map(([s, label]) => button(label, () => set({ sort: s }), st.sort === s ? "chip quiet on" : "chip quiet", `codex-sort-${s}`)),
  );
  const search = h("input", { type: "search", placeholder: "Search names and text", "data-testid": "codex-search", "aria-label": "Search units" });
  search.value = st.query;
  search.addEventListener("input", () => {
    st.query = search.value;
    draw();
  });
  draw();
  const picked = st.trigger ? triggers.get(st.trigger) : null;
  const note =
    st.sort !== "tier"
      ? stats
        ? h("div", { class: "dim small" }, "Win: how often its team won the fight. Picked: how often it was on a finished line. Since the units last changed.")
        : h("div", { class: "row dim small", "data-testid": "codex-rates-error" }, "Rates aren't available right now.", button("Try again", retry, "small", "codex-rates-retry"))
      : null;
  return [search, tierRow, trigRow, sortRow, picked ? h("div", { class: "dim small" }, `When: ${picked.label}`) : null, count, note, grid].filter((n): n is NonNullable<typeof n> => n !== null);
}

// ---------- fusions ----------

/** The fused unit an ordered pair makes, as fuseUnits builds it in a run. */
function fusedOf(first: UnitContent, second: UnitContent, f: FusionDiscovery | undefined, content: MvpContent): LineUnit {
  return fuseUnits(lineUnitOf(first, "a", 3), lineUnitOf(second, "b", 3), { name: f?.name ?? "???", discoveredBy: f?.discoveredBy ?? null }, content);
}

/** Rows drawn at once; "Show more" draws the next batch. */
const BATCH = 40;

function fusionsTab(
  content: MvpContent,
  fusions: FusionDiscovery[],
  st: CodexState,
  go: (next: Partial<CodexState>) => void,
  open: (node: HTMLElement, from?: HTMLElement) => void,
): Node[] {
  const byId = new Map(content.units.map((u) => [u.id, u]));
  const me = api.player?.id;
  const n = content.units.length;
  const total = n * (n - 1);
  const known = new Map(fusions.map((f) => [`${f.first}>${f.second}`, f]));
  const mine = fusions.filter((f) => f.discoveredBy?.id === me);

  const pick = h("select", { "data-testid": "codex-fusions-of", "aria-label": "Pairs of one unit" }) as HTMLSelectElement;
  pick.append(h("option", { value: "" }, "Every discovery"), ...[...content.units].sort((x, y) => x.name.localeCompare(y.name)).map((u) => h("option", { value: u.id }, `${u.emoji} ${u.name} + …`)));
  pick.value = st.fusionsOf ?? "";
  pick.addEventListener("change", () => go({ fusionsOf: pick.value || null }));
  const mineBtn = button(`Mine · ${mine.length}`, () => go({ mine: !st.mine }), st.mine ? "chip on" : "chip", "codex-fusions-mine");

  type Row = { a: UnitContent; b: UnitContent; f?: FusionDiscovery | undefined };
  let rows: Row[];
  const first = st.fusionsOf ? byId.get(st.fusionsOf) : undefined;
  if (first) {
    rows = content.units.filter((b) => b.id !== first.id).map((b) => ({ a: first, b, f: known.get(`${first.id}>${b.id}`) }))
      // The found pairs first, then the "?" ones, each by name.
      .sort((x, y) => Number(!x.f) - Number(!y.f) || x.b.name.localeCompare(y.b.name));
    if (st.mine) rows = rows.filter((r) => r.f?.discoveredBy?.id === me);
  } else {
    rows = [...(st.mine ? mine : fusions)]
      .sort((x, y) => (x.discoveredAt < y.discoveredAt ? 1 : x.discoveredAt > y.discoveredAt ? -1 : 0))
      .flatMap((f) => {
        const a = byId.get(f.first);
        const b = byId.get(f.second);
        return a && b ? [{ a, b, f }] : [];
      });
  }

  const row = ({ a, b, f }: Row): HTMLElement => {
    const fused = fusedOf(a, b, f, content);
    const by = f?.discoveredBy ? (f.discoveredBy.id === me ? "you" : who(f.discoveredBy.name)) : null;
    const el = h(
      "div",
      { class: `stat-row fusion-row${f ? "" : " unknown"}`, "data-testid": f ? "codex-fusion" : "codex-fusion-unknown" },
      h("span", { class: "emoji pair" }, `${a.emoji}${b.emoji}`),
      h(
        "span",
        { class: "grow" },
        h("div", { class: "fusion-name" }, f ? f.name : "?"),
        h("div", { class: "dim small" }, `When of ${a.name} · Who of ${b.name}`),
        f ? h("div", { class: "fusion-recipe small" }, ...formRich(fused.recipe, content)) : null,
        f ? h("div", { class: "discovered" }, ...(by ? ["discovered by ", by] : ["made by bots, unclaimed"])) : h("div", { class: "discovered" }, "nobody has made it yet"),
      ),
      h("span", { class: "num dim" }, `${fused.stats.pwr}/${fused.stats.hp}`),
    );
    el.addEventListener("click", (e) => {
      if ((e.target as HTMLElement).closest("button.t")) return; // a term opens its own rule
      open(unitSheet(fused, content), el);
    });
    return el;
  };

  const list = h("div", { class: "panel stack", "data-testid": "codex-fusions" });
  let drawn = 0;
  const more = button("Show more", () => drawMore(), "small", "codex-fusions-more");
  const drawMore = () => {
    list.insertBefore(h("div", { class: "contents" }, ...rows.slice(drawn, drawn + BATCH).map(row)), more);
    drawn = Math.min(rows.length, drawn + BATCH);
    more.hidden = drawn >= rows.length;
  };
  list.append(more);
  if (rows.length) drawMore();
  else {
    more.hidden = true;
    list.prepend(h("div", { class: "dim" }, st.mine ? "You haven't discovered a fusion yet. Fuse two Awoken units in a run to name one." : "No fusions yet. Fuse two Awoken units in a run to discover one."));
  }

  const found = first ? rows.filter((r) => r.f).length : null;
  return [
    h(
      "div",
      { class: "row spread" },
      h("div", { class: "num", "data-testid": "codex-fusions-found" }, `${fusions.length.toLocaleString("en")} of ${total.toLocaleString("en")} found`),
      mineBtn,
    ),
    pick,
    first ? h("div", { class: "dim small" }, `${first.name} first: ${found} of ${n - 1} pairs found. Order matters: the first part gives the When, the second the Who.`) : h("div", { class: "dim small" }, "Newest first. Pick a unit to see all its pairs, the ones nobody has found as \"?\"."),
    list,
  ];
}

// ---------- keywords ----------

const GROUPS: [TermGroup, string][] = [
  ["status", "Statuses"],
  ["trigger", "When"],
  ["condition", "Conditions"],
  ["target", "Who"],
  ["effect", "Does"],
  ["stat", "Stats"],
  ["state", "Unit states"],
  ["battle", "Battle"],
  ["term", "Words"],
];

/** Rows that show only when a unit's text uses them: the rest (statuses,
 * stats, unit states, battle rules, Stacks, which their rules say) always
 * show. Kernel jargon no unit says (Absorb, Cheat death, Cancel, Spend,
 * would) stays hidden until content starts using it. */
const SHOWN_IF_USED: ReadonlySet<TermGroup> = new Set(["trigger", "condition", "target", "effect"]);
const shownIfUsed = (id: TermId): boolean => SHOWN_IF_USED.has(termGroup(id)) || id === "term:would";

/** A trigger's scopes other than its holder's, in the order its row lists them. */
const SCOPES: Exclude<UnitFilter, "holder">[] = ["ally", "otherAlly", "enemy", "any"];
/** A term, or a trigger term said of someone else ("After an enemy dies"). */
const useKey = (id: TermId, scope?: UnitFilter) => (scope && scope !== "holder" && id.startsWith("trigger:") ? `${id}|${scope}` : id);

/** Every unit whose text (either form) uses each term, a scoped trigger
 * under its own key (useKey). */
function usersByTerm(content: MvpContent): Map<string, UnitContent[]> {
  const out = new Map<string, UnitContent[]>();
  for (const u of [...content.units].sort(byTierName)) {
    const keys = new Set<string>();
    for (const form of [u.forms.sleeping, u.forms.awoken]) for (const s of formSegments(form, content.abilities)) if (s.term) keys.add(useKey(s.term, s.scope));
    for (const k of keys) out.set(k, [...(out.get(k) ?? []), u]);
  }
  return out;
}

/** The chips shown before "+N more". */
const CHIPS = 12;

function keywordsTab(content: MvpContent, open: (node: HTMLElement, from?: HTMLElement) => void): HTMLElement {
  const users = usersByTerm(content);
  const statusIds = [...new Set([...Object.keys(STATUS_TERMS), ...Object.keys(content.statuses)])].map((s) => `status:${s}` as TermId);
  const ids: TermId[] = [...statusIds, ...(Object.keys(GLOSSARY) as FixedTermId[])];
  const chipsOf = (used: UnitContent[]): HTMLElement => {
    const chip = (u: UnitContent) => {
      const b = button(`${u.emoji} ${u.name}`, () => open(unitSheet(u, content), b), "chip unit-chip", "codex-term-unit");
      return b;
    };
    const chips = h("div", { class: "chips" }, ...used.slice(0, CHIPS).map(chip));
    if (used.length > CHIPS) {
      const rest = button(`+${used.length - CHIPS} more`, () => rest.replaceWith(...used.slice(CHIPS).map(chip)), "chip", "codex-term-more");
      chips.append(rest);
    }
    return chips;
  };
  const usedBy = (n: number) => h("div", { class: "dim small" }, `Used by ${n} ${n === 1 ? "unit" : "units"}`);
  const row = (id: TermId): HTMLElement | null => {
    const def = termDef(id, content.statuses);
    if (!def) return null;
    const used = users.get(id) ?? [];
    // A trigger said of someone else gets its own line, with its own label,
    // rule and units: "Enemy dies / When an enemy dies." lists Wither, not
    // "When it dies."
    const scoped = (id.startsWith("trigger:") ? SCOPES : []).flatMap((sc) => {
      const them = users.get(useKey(id, sc));
      const tip = them && scopedTip(id, sc);
      return them && tip
        ? [
            h(
              "div",
              { class: "stack kw-scope", "data-scope": sc, "data-testid": "codex-term-scope" },
              h("b", { class: `tone-${def.tone}`, "data-testid": "codex-term-scope-label" }, scopedLabel(id, sc) ?? def.label),
              h("div", { class: "kw-tip" }, tip),
              usedBy(them.length),
              chipsOf(them),
            ),
          ]
        : [];
    });
    if (shownIfUsed(id) && used.length === 0 && scoped.length === 0) return null;
    return h(
      "div",
      { class: "kw-row", "data-term": id, "data-testid": "codex-term", id: `codex-${id.replace(/[^a-zA-Z0-9]+/g, "-")}` },
      h("div", { class: `kw-icon tone-${def.tone}` }, def.icon ? icon(def.icon, 48) : h("span", { class: "kw-noicon" }, def.label.slice(0, 1))),
      h(
        "div",
        { class: "stack kw-text" },
        h("b", { class: `tone-${def.tone}` }, def.label),
        h("div", { class: "kw-tip" }, def.tip),
        def.more ? h("div", { class: "dim small" }, def.more) : null,
        used.length ? usedBy(used.length) : null,
        used.length ? chipsOf(used) : null,
        ...scoped,
      ),
    );
  };
  return h(
    "div",
    { class: "stack keywords", "data-testid": "codex-keywords" },
    ...GROUPS.flatMap(([g, title]) => {
      const rows = ids.filter((id) => termGroup(id) === g).map(row).filter((r): r is HTMLElement => r !== null);
      return rows.length ? [h("div", { class: "label kw-group" }, title), h("div", { class: "panel stack kw-list" }, ...rows)] : [];
    }),
  );
}

/** Scrolls a term's row (a scoped trigger's line in it) into view and marks
 * it, once it is laid out. */
function landOn(body: HTMLElement, id: TermId, scope?: UnitFilter): void {
  const row = [...body.querySelectorAll<HTMLElement>(".kw-row")].find((r) => r.dataset.term === id);
  if (!row) return;
  const line = scope && scope !== "holder" ? row.querySelector<HTMLElement>(`.kw-scope[data-scope="${scope}"]`) : null;
  const el = line ?? row;
  // Now (the row is laid out), and once more after the next frame's fitText.
  el.scrollIntoView({ block: "center" });
  row.classList.add("landed");
  line?.classList.add("landed");
  requestAnimationFrame(() => el.isConnected && el.scrollIntoView({ block: "center" }));
}

// ---------- credits ----------

/** The icon credits CC BY 3.0 asks for (docs/round2/icons.md, icons/CREDITS.txt). */
function credits(): HTMLElement {
  const a = (href: string, text: string) => h("a", { href, target: "_blank", rel: "noopener" }, text);
  return h(
    "div",
    { class: "dim small credits", "data-testid": "icon-credits" },
    "Icons made by Delapouite, Lorc, Sbed and Skoll from ",
    a("https://game-icons.net", "game-icons.net"),
    ", licensed ",
    a("https://creativecommons.org/licenses/by/3.0/", "CC BY 3.0"),
    ". Heart-plus by Zeromancer (CC0). Icons were recoloured and their background removed.",
  );
}
