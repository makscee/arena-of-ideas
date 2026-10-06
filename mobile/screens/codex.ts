// The Codex (round 2, R2-11; docs/round2/ui.md (e), icons.md "Codex glossary
// entry", mechanics.md (4)). Reached from the title menu, the in-run menu and
// every term's "Open in Codex"; Back returns to the screen it opened over.
// Three tabs:
// - Units: all the content's units as compact cards, filtered by tier, by the
//   trigger icon a card shows and by a search over names and both forms'
//   text. A card opens its sheet (active form, See Awoken, the dim rates line).
// - Fusions: every discovered pair with its recipe and its credit, "N of 6,480
//   found"; picking a unit lists all its pairs as the first part, the ones
//   nobody has made as "?".
// - Keywords: every glossary term with its 48px icon, its rule and the units
//   whose text uses it; a term's deep link lands on its row.
// The icon credits (CC BY 3.0) sit at its foot. Reads /content and /fusions.
import { GLOSSARY, STATUS_TERMS, termDef, termGroup, termIcon, type FixedTermId, type IconId, type TermGroup, type TermId } from "../../src/glossary";
import type { FusionDiscovery, LineUnit, MvpContent, UnitContent, UnitId } from "../../src/mvp/contract";
import { formSegments, formText } from "../../src/mvp/form-text";
import { fuseUnits, lineUnitOf } from "../../src/mvp/forms";
import { api } from "../api";
import { card, formRich, unitSheet } from "../ui/card";
import { button, closable, h, onKeys, screen, show, who } from "../ui/dom";
import { icon } from "../ui/icon";
import { loadUnitRates } from "../ui/unit-stats";

export type CodexTab = "units" | "fusions" | "keywords";

export interface CodexState {
  tab: CodexTab;
  /** Units: one tier, or every tier. */
  tier: number | null;
  /** Units: the trigger icon a card shows top-left, or any. */
  trigger: IconId | null;
  query: string;
  /** Fusions: one unit's pairs (as the first part), or every discovery. */
  fusionsOf: UnitId | null;
  mine: boolean;
  /** Keywords: the row to land on. */
  term?: TermId | undefined;
}

const DEFAULTS: CodexState = { tab: "units", tier: null, trigger: null, query: "", fusionsOf: null, mine: false };

export async function codexScreen(a: { content: MvpContent; onBack: () => void; state?: Partial<CodexState> | undefined }): Promise<void> {
  void loadUnitRates(); // the dim rates line on unit sheets
  const st: CodexState = { ...DEFAULTS, ...a.state };
  const go = (next: Partial<CodexState>) => void codexScreen({ ...a, state: { ...st, term: undefined, ...next } });
  const back = button("Back", a.onBack, "primary grow", "codex-back");
  const tabBtn = (t: CodexTab, label: string) => button(label, () => go({ tab: t }), t === st.tab ? "on" : "", `codex-tab-${t}`);
  const body = h("div", { class: "stack codex-body" });
  show(
    h("div", { class: "row spread" }, h("h1", {}, "CODEX"), h("span", { class: "dim small" }, `${a.content.units.length} units`)),
    h("div", { class: "tabs", role: "tablist" }, tabBtn("units", "Units"), tabBtn("fusions", "Fusions"), tabBtn("keywords", "Keywords")),
    body,
    credits(),
    h("div", { class: "spacer" }),
    h("div", { class: "row footer" }, back),
  );
  screen("codex");
  onKeys((e) => {
    if (e.key !== "Escape" || document.querySelector(".overlay")) return false;
    a.onBack();
    return true;
  });
  if (st.tab === "units") body.append(...unitsTab(a.content, st, go));
  else if (st.tab === "keywords") {
    body.append(keywordsTab(a.content));
    if (st.term) landOn(body, st.term);
  } else {
    body.append(h("div", { class: "dim" }, "Loading fusions…"));
    let fusions: FusionDiscovery[];
    try {
      fusions = await api.fusions();
    } catch (e) {
      body.replaceChildren(h("div", { class: "error", "data-testid": "error" }, e instanceof Error ? e.message : String(e)));
      return;
    }
    if (!body.isConnected) return; // left meanwhile
    body.replaceChildren(...fusionsTab(a.content, fusions, st, go));
  }
}

// ---------- units ----------

/** The icon a unit's card shows top-left (card.ts triggerMark) and its label. */
function triggerOf(u: UnitContent): { icon: IconId; label: string } | null {
  const on = u.forms.sleeping.when[0]?.on;
  if (!on) return null;
  const id = `trigger:${on.on}` as TermId;
  const status = "status" in on ? on.status : undefined;
  const ic = termIcon(id, status);
  if (!ic) return null;
  const label = termDef(id)?.label ?? on.on;
  return { icon: ic, label: status && (on.on === "StatusApplied" || on.on === "StatusRemoved") ? `${status} ${on.on === "StatusApplied" ? "lands" : "leaves"}` : label };
}

const byTierName = (x: UnitContent, y: UnitContent) => x.tier - y.tier || x.name.localeCompare(y.name);

function unitsTab(content: MvpContent, st: CodexState, go: (next: Partial<CodexState>) => void): Node[] {
  const units = [...content.units].sort(byTierName);
  const tiers = [...new Set(units.map((u) => u.tier))].sort();
  // The trigger icons the picked tier has (the picked one always stays).
  const triggers = new Map<IconId, string>();
  for (const u of units.filter((x) => st.tier === null || x.tier === st.tier || triggerOf(x)?.icon === st.trigger)) {
    const t = triggerOf(u);
    if (t && !triggers.has(t.icon)) triggers.set(t.icon, t.label);
  }
  const text = new Map(units.map((u) => [u.id, `${u.name} ${formText(u.forms.sleeping, content.abilities)} ${formText(u.forms.awoken, content.abilities)}`.toLowerCase()]));

  const grid = h("div", { class: "slots codex-grid", "data-testid": "codex-units" });
  const count = h("div", { class: "dim small", "data-testid": "codex-count" });
  const draw = () => {
    const q = st.query.trim().toLowerCase();
    const shown = units.filter((u) => (st.tier === null || u.tier === st.tier) && (st.trigger === null || triggerOf(u)?.icon === st.trigger) && (!q || text.get(u.id)!.includes(q)));
    grid.replaceChildren(
      ...shown.map((u) =>
        card({ emoji: u.emoji, name: u.name, stats: u.base, recipe: u.forms.sleeping, unitId: u.id }, { side: "you", tier: u.tier, testid: "codex-unit", onOpen: () => closable(unitSheet(u, content)) }),
      ),
    );
    count.textContent = shown.length === units.length ? `All ${units.length} units, by tier. Tap one to read it.` : `${shown.length} of ${units.length} units`;
    if (!shown.length) grid.append(h("div", { class: "dim codex-none" }, "No unit matches."));
  };

  const tierRow = h(
    "div",
    { class: "row codex-filter", "data-testid": "codex-tiers" },
    h("span", { class: "label" }, "Tier"),
    ...[null, ...tiers].map((t) => button(t === null ? "All" : `${t}`, () => go({ tier: t }), st.tier === t ? "chip on" : "chip", `codex-tier-${t ?? "all"}`)),
  );
  const trigRow = h(
    "div",
    { class: "row codex-filter codex-triggers", "data-testid": "codex-triggers" },
    ...[...triggers].map(([ic, label]) => {
      const b = button("", () => go({ trigger: st.trigger === ic ? null : ic }), st.trigger === ic ? "chip icon on tone-when" : "chip icon tone-when", `codex-trigger-${ic}`);
      b.title = label;
      b.setAttribute("aria-label", label);
      b.setAttribute("aria-pressed", String(st.trigger === ic));
      b.append(icon(ic, 22));
      return b;
    }),
  );
  const search = h("input", { type: "search", placeholder: "Search names and text", "data-testid": "codex-search", "aria-label": "Search units" });
  search.value = st.query;
  search.addEventListener("input", () => {
    st.query = search.value;
    draw();
  });
  draw();
  const picked = st.trigger ? triggers.get(st.trigger) : null;
  return [search, tierRow, trigRow, picked ? h("div", { class: "dim small" }, `When: ${picked}`) : null, count, grid].filter((n): n is NonNullable<typeof n> => n !== null);
}

// ---------- fusions ----------

/** The fused unit an ordered pair makes, as fuseUnits builds it in a run. */
function fusedOf(first: UnitContent, second: UnitContent, f: FusionDiscovery | undefined, content: MvpContent): LineUnit {
  return fuseUnits(lineUnitOf(first, "a", 3), lineUnitOf(second, "b", 3), { name: f?.name ?? "???", discoveredBy: f?.discoveredBy ?? null }, content);
}

/** Rows drawn at once; "Show more" draws the next batch. */
const BATCH = 40;

function fusionsTab(content: MvpContent, fusions: FusionDiscovery[], st: CodexState, go: (next: Partial<CodexState>) => void): Node[] {
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
      closable(unitSheet(fused, content));
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
  ["trigger", "Triggers (When)"],
  ["condition", "Conditions"],
  ["target", "Targets (Who)"],
  ["effect", "Effects (Does)"],
  ["stat", "Stats"],
  ["state", "Unit states"],
  ["battle", "Battle"],
  ["term", "Words"],
];

/** Every unit whose text (either form) uses each term. */
function usersByTerm(content: MvpContent): Map<TermId, UnitContent[]> {
  const out = new Map<TermId, UnitContent[]>();
  for (const u of [...content.units].sort(byTierName)) {
    const ids = new Set<TermId>();
    for (const form of [u.forms.sleeping, u.forms.awoken]) for (const s of formSegments(form, content.abilities)) if (s.term) ids.add(s.term);
    for (const id of ids) out.set(id, [...(out.get(id) ?? []), u]);
  }
  return out;
}

/** The chips shown before "+N more". */
const CHIPS = 12;

function keywordsTab(content: MvpContent): HTMLElement {
  const users = usersByTerm(content);
  const statusIds = [...new Set([...Object.keys(STATUS_TERMS), ...Object.keys(content.statuses)])].map((s) => `status:${s}` as TermId);
  const ids: TermId[] = [...statusIds, ...(Object.keys(GLOSSARY) as FixedTermId[])];
  const row = (id: TermId): HTMLElement | null => {
    const def = termDef(id, content.statuses);
    if (!def) return null;
    const used = users.get(id) ?? [];
    const chip = (u: UnitContent) => {
      const b = button(`${u.emoji} ${u.name}`, () => closable(unitSheet(u, content)), "chip unit-chip", "codex-term-unit");
      return b;
    };
    const chips = h("div", { class: "chips" }, ...used.slice(0, CHIPS).map(chip));
    if (used.length > CHIPS) {
      const rest = button(`+${used.length - CHIPS} more`, () => rest.replaceWith(...used.slice(CHIPS).map(chip)), "chip", "codex-term-more");
      chips.append(rest);
    }
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
        used.length ? h("div", { class: "dim small" }, `Used by ${used.length} ${used.length === 1 ? "unit" : "units"}`) : null,
        used.length ? chips : null,
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

/** Scrolls a term's row into view and marks it, once it is laid out. */
function landOn(body: HTMLElement, id: TermId): void {
  const el = [...body.querySelectorAll<HTMLElement>(".kw-row")].find((r) => r.dataset.term === id);
  if (!el) return;
  // Now (the row is laid out), and once more after the next frame's fitText.
  el.scrollIntoView({ block: "center" });
  el.classList.add("landed");
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
