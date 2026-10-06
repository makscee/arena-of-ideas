// The stats page (mission #574). Slice 11 owns this file. Home's "Stats"
// button opens it: your records (HomeView.rating, which slices 4 and 5
// write) and the champion history; a champion opens its team. Round 2
// (R2-11): units with their rates and the discovered fusions moved to the
// Codex (./codex.ts), where a unit's rates are the dim line on its sheet.
import type { Champion, HomeView, MvpContent, StatsView } from "../../src/mvp/contract";
import { api } from "../api";
import { card, unitSheet } from "../ui/card";
import { button, h, overlay, screen, show, who } from "../ui/dom";
import { keepUnitRates } from "../ui/unit-stats";

export async function statsScreen(a: { content: MvpContent; onBack: () => void; onCodex?: () => void }): Promise<void> {
  const back = button("Back", a.onBack, "primary grow", "stats-back");
  let data: [StatsView, HomeView];
  try {
    data = await Promise.all([api.stats(), api.home()]);
  } catch (e) {
    show(h("h1", {}, "STATS"), h("div", { class: "error", "data-testid": "error" }, e instanceof Error ? e.message : String(e)), h("div", { class: "spacer" }), h("div", { class: "row footer" }, back));
    return;
  }
  const [stats, home] = data;
  keepUnitRates(stats.units);
  const { content } = a;
  show(
    h("h1", {}, "STATS"),
    recordsPanel(home),
    h("div", { class: "label" }, "Champions"),
    championsPanel(stats.champions, content),
    a.onCodex ? h("div", { class: "dim small" }, "Units, their win and pick rates, and every fusion found are in the ", codexLink(a.onCodex), ".") : null,
    h("div", { class: "spacer" }),
    // Stuck to the bottom: the lists grow long, and Back is the only way home.
    h("div", { class: "row footer" }, back),
  );
  screen("stats");
}

const codexLink = (open: () => void): HTMLElement => {
  const b = h("button", { type: "button", class: "small link", "data-testid": "stats-codex" }, "Codex");
  b.addEventListener("click", open);
  return b;
};

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
