// The stats page (mission #574). Slice 11 owns this file. Home's "Stats"
// button opens it: your records (HomeView.rating, which slices 4 and 5
// write) and the champion history; a champion opens its team. Round 2
// (R2-11): units with their rates and the discovered fusions moved to the
// Codex (./codex.ts), where a unit's rates are the dim line on its sheet.
import type { Champion, HomeView, MvpContent, StatsView } from "../../src/mvp/contract";
import { api } from "../api";
import { card, unitSheet } from "../ui/card";
import { t } from "../i18n";
import { button, h, isDesktop, onKeys, overlay, screen, show, who } from "../ui/dom";
import { keepUnitRates } from "../ui/unit-stats";

export async function statsScreen(a: { content: MvpContent; onBack: () => void; onCodex?: () => void }): Promise<void> {
  const back = button(t("stats.back"), a.onBack, "primary grow", "stats-back");
  // Esc goes back, once any sheet over the page is closed (ui/dom.ts).
  const escBack = (e: KeyboardEvent) => (e.key === "Escape" ? (a.onBack(), true) : false);
  let data: [StatsView, HomeView];
  try {
    data = await Promise.all([api.stats(), api.home()]);
  } catch (e) {
    show(h("h1", {}, t("stats.title")), h("div", { class: "error", "data-testid": "error" }, e instanceof Error ? e.message : String(e)), h("div", { class: "spacer" }), h("div", { class: "row footer" }, back));
    onKeys(escBack);
    return;
  }
  const [stats, home] = data;
  keepUnitRates(stats.units);
  const { content } = a;
  show(
    h("h1", {}, t("stats.title")),
    recordsPanel(home),
    h("div", { class: "label" }, t("stats.champions")),
    championsPanel(stats.champions, content),
    a.onCodex ? h("div", { class: "dim small" }, t("stats.codexBefore"), codexLink(a.onCodex), t("stats.codexAfter")) : null,
    h("div", { class: "spacer" }),
    // Stuck to the bottom: the lists grow long, and Back is the only way home.
    h("div", { class: "row footer" }, back),
  );
  screen("stats");
  onKeys(escBack);
}

const codexLink = (open: () => void): HTMLElement => {
  const b = h("button", { type: "button", class: "small link", "data-testid": "stats-codex" }, t("stats.codex"));
  b.addEventListener("click", open);
  return b;
};

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
        return el;
      }),
    ),
  );
}
