// The Champions (mission #574, slice 11; M5-6 folded the stats page into the
// Codex's Champions tab): your records (HomeView.rating, which slices 4 and 5
// write) and the champion history; a champion opens its team. Round 2
// (R2-11): units with their rates and the discovered fusions moved to the
// Codex (./codex.ts), where a unit's rates are the dim line on its sheet.
import type { Champion, HomeView, MvpContent, StatsView } from "../../src/mvp/contract";
import { card, unitSheet } from "../ui/card";
import { t } from "../i18n";
import { actionable, h, isDesktop, overlay, who } from "../ui/dom";
import { keepUnitRates } from "../ui/unit-stats";

/** The Codex's Champions tab (M5-6): the champions, newest first, then your records. */
export function championsTab(stats: StatsView, home: HomeView, content: MvpContent): Node[] {
  keepUnitRates(stats.units);
  return [h("div", { class: "label" }, t("stats.champions")), championsPanel(stats.champions, content), recordsPanel(home)];
}

function recordsPanel(home: HomeView): HTMLElement {
  const r = home.rating;
  const record = (label: string, value: number) => h("div", { class: "record" }, h("div", { class: "num" }, `${value}`), h("div", { class: "label" }, label));
  return h(
    "div",
    { class: "stack" },
    h("div", { class: "label" }, t("stats.records")),
    h(
      "div",
      { class: "panel records", "data-testid": "stats-records" },
      record(t("stats.rating"), r?.rating ?? home.rules.ratingStart),
      record(t("stats.runs"), r?.runs ?? 0),
      record(t("stats.slays"), r?.slays ?? 0),
      record(t("stats.daysChampion"), r?.daysAsChampion ?? 0),
      record(t("stats.playoffWins"), r?.playoffWins ?? 0),
    ),
  );
}

function championsPanel(champions: Champion[], content: MvpContent): HTMLElement {
  const rows = [...champions].reverse().map((c) => {
    const row = h(
      "div",
      { class: "stat-row", "data-testid": "stats-champion" },
      h("span", { class: "emoji" }, "👑"),
      h("span", { class: "grow" }, h("div", {}, t("stats.dayPrefix", { seq: c.seq }), who(c.player.name), c.player.bot ? " 🤖" : ""), h("div", { class: "dim small" }, `${c.day} · ${c.line.map((u) => u.emoji).join(" ")}`)),
    );
    row.addEventListener("click", () => overlay(championSheet(c, content)));
    actionable(row);
    return row;
  });
  return h("div", { class: "panel stack", "data-testid": "stats-champions" }, ...(rows.length ? rows : [h("div", { class: "dim" }, t("stats.noChampion"))]));
}

function championSheet(c: Champion, content: MvpContent): HTMLElement {
  return h(
    "div",
    { class: "stack", "data-testid": "champion-sheet" },
    h("h2", {}, t("stats.sheetDayPrefix", { seq: c.seq }), who(c.player.name), c.player.bot ? " 🤖" : ""),
    h("div", { class: "dim" }, isDesktop() ? t("stats.sheetHintDesktop", { day: c.day }) : t("stats.sheetHintPhone", { day: c.day })),
    h(
      "div",
      { class: "slots" },
      ...c.line.map((u) => {
        const el = card(u, { side: "ghost" });
        el.addEventListener("click", () => overlay(unitSheet(u, content)));
        return actionable(el, u.name);
      }),
    ),
  );
}
