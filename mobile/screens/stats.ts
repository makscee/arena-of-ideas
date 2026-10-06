// The stats page (mission #574). Slice 11 owns this file. Home's "Stats"
// button opens it: your records (HomeView.rating, which slices 4 and 5
// write), unit win and pick rates (api.stats()), the champion history and the
// discovered fusions (api.fusions(), slice 10's list). Every unit opens its
// sheet; a champion opens its team.
import type { Champion, FusionDiscovery, HomeView, MvpContent, StatsView, UnitContent } from "../../src/mvp/contract";
import { api } from "../api";
import { card, unitSheet } from "../ui/card";
import { button, h, overlay, show, who } from "../ui/dom";
import { keepUnitRates, pct } from "../ui/unit-stats";

type Tab = "units" | "champions" | "fusions";

export async function statsScreen(a: { content: MvpContent; onBack: () => void; tab?: Tab }): Promise<void> {
  const back = button("Back", a.onBack, "primary grow", "stats-back");
  let data: [StatsView, FusionDiscovery[], HomeView];
  try {
    data = await Promise.all([api.stats(), api.fusions(), api.home()]);
  } catch (e) {
    show(h("h1", {}, "STATS"), h("div", { class: "error", "data-testid": "error" }, e instanceof Error ? e.message : String(e)), h("div", { class: "spacer" }), h("div", { class: "row footer" }, back));
    return;
  }
  const [stats, fusions, home] = data;
  keepUnitRates(stats.units);
  const { content } = a;
  const tab = a.tab ?? "units";
  const tabBtn = (t: Tab, label: string) => button(label, () => void statsScreen({ ...a, tab: t }), t === tab ? "on" : "", `stats-tab-${t}`);
  const body = tab === "units" ? unitsPanel(stats, content) : tab === "champions" ? championsPanel(stats.champions, content) : fusionsPanel(fusions, content);
  show(
    h("h1", {}, "STATS"),
    recordsPanel(home),
    h("div", { class: "tabs" }, tabBtn("units", "Units"), tabBtn("champions", "Champions"), tabBtn("fusions", "Fusions")),
    body,
    h("div", { class: "spacer" }),
    // Stuck to the bottom: the lists grow long, and Back is the only way home.
    h("div", { class: "row footer" }, back),
  );
}

function recordsPanel(home: HomeView): HTMLElement {
  const r = home.rating;
  const record = (label: string, value: number) => h("div", { class: "record" }, h("div", { class: "num" }, `${value}`), h("div", { class: "label" }, label));
  return h(
    "div",
    { class: "stack" },
    h("div", { class: "label" }, "Your records"),
    h(
      "div",
      { class: "panel records", "data-testid": "stats-records" },
      record("Rating", r?.rating ?? home.rules.ratingStart),
      record("Runs", r?.runs ?? 0),
      record("Slays", r?.slays ?? 0),
      record("Days 👑", r?.daysAsChampion ?? 0),
      record("Playoff W", r?.playoffWins ?? 0),
    ),
  );
}

function unitsPanel(stats: StatsView, content: MvpContent): HTMLElement {
  const byId = new Map(content.units.map((u) => [u.id, u]));
  const rows = stats.units.flatMap((s) => {
    const u = byId.get(s.unitId);
    if (!u) return [];
    return [
      h(
        "div",
        { class: "stat-row", "data-testid": "stats-unit" },
        h("span", { class: "emoji" }, u.emoji),
        h("span", { class: "grow" }, u.name),
        h("span", { class: "num w" }, pct(s.winRate)),
        h("span", { class: "num" }, pct(s.pickRate)),
        h("span", { class: "num dim" }, `${s.runs}`),
      ),
    ].map((row) => (row.addEventListener("click", () => overlay(unitSheet(u, content, { rates: s }))), row));
  });
  return h(
    "div",
    { class: "panel stack", "data-testid": "stats-units" },
    rows.length
      ? h("div", { class: "stat-head" }, h("span", { class: "grow" }, "Unit"), h("span", {}, "Win"), h("span", {}, "Pick"), h("span", {}, "Runs"))
      : h("div", { class: "dim" }, "No finished runs on today's units yet."),
    ...rows,
    rows.length ? h("div", { class: "dim small" }, "Win: fights its team won. Pick: finished runs that ended with it on the line. Tap a unit to read it.") : null,
  );
}

function championsPanel(champions: Champion[], content: MvpContent): HTMLElement {
  const rows = [...champions].reverse().map((c) => {
    const row = h(
      "div",
      { class: "stat-row", "data-testid": "stats-champion" },
      h("span", { class: "emoji" }, "👑"),
      h("span", { class: "grow" }, h("div", {}, `Day ${c.seq} · `, who(c.player.name), c.player.bot ? " 🤖" : ""), h("div", { class: "dim small" }, `${c.day} · ${c.line.map((u) => u.emoji).join(" ")}`)),
    );
    row.addEventListener("click", () => overlay(championSheet(c, content)));
    return row;
  });
  return h("div", { class: "panel stack", "data-testid": "stats-champions" }, ...(rows.length ? rows : [h("div", { class: "dim" }, "No champion yet.")]));
}

function championSheet(c: Champion, content: MvpContent): HTMLElement {
  return h(
    "div",
    { class: "stack", "data-testid": "champion-sheet" },
    h("h2", {}, `👑 Day ${c.seq} · `, who(c.player.name), c.player.bot ? " 🤖" : ""),
    h("div", { class: "dim" }, `${c.day}, front first. Tap a unit to read it.`),
    h(
      "div",
      { class: "slots" },
      ...c.line.map((u) => {
        const el = card(u, { side: "ghost" });
        el.addEventListener("click", () => overlay(unitSheet(u, content)));
        return el;
      }),
    ),
  );
}

/** The newest fusions shown; the rest are counted. */
const FUSIONS_SHOWN = 60;

function fusionsPanel(fusions: FusionDiscovery[], content: MvpContent): HTMLElement {
  const byId = new Map<string, UnitContent>(content.units.map((u) => [u.id, u]));
  const part = (id: string) => byId.get(id);
  const me = api.player?.id;
  const rows = [...fusions]
    .sort((x, y) => (x.discoveredAt < y.discoveredAt ? 1 : x.discoveredAt > y.discoveredAt ? -1 : 0))
    .slice(0, FUSIONS_SHOWN)
    .map((f) => {
      const a = part(f.first);
      const b = part(f.second);
      const by = f.discoveredBy ? (f.discoveredBy.id === me ? "you" : who(f.discoveredBy.name)) : null;
      return h(
        "div",
        { class: "stat-row", "data-testid": "stats-fusion" },
        h("span", { class: "emoji" }, `${a?.emoji ?? "?"}`),
        h(
          "span",
          { class: "grow" },
          h("div", {}, f.name),
          h("div", { class: "dim small" }, `${a?.name ?? f.first} → ${b?.name ?? f.second}`),
          h("div", { class: "discovered" }, ...(by ? ["discovered by ", by] : ["made by bots, unclaimed"])),
        ),
        h("span", { class: "emoji" }, `${b?.emoji ?? "?"}`),
      );
    });
  return h(
    "div",
    { class: "panel stack", "data-testid": "stats-fusions" },
    ...(rows.length ? rows : [h("div", { class: "dim" }, "No fusions yet. Fuse two Awoken units to discover one.")]),
    fusions.length > rows.length ? h("div", { class: "dim small" }, `The newest ${rows.length} of ${fusions.length} fusions.`) : null,
    rows.length ? h("div", { class: "dim small" }, "Order matters: the first part gives the When, the second the Who.") : null,
  );
}
