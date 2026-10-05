// The unit card (mission #574), shared by the shop and result screens
// (slice 8) and the battle viewer (slice 9).
import type { Stats } from "../../src/types";
import { h } from "./dom";

/** What a card needs to draw; LineUnit, BattleUnit and offers all fit. */
export interface CardUnit {
  emoji: string;
  name: string;
  stats: Stats;
}

export function card(u: CardUnit, side: "you" | "ghost", extra: (Node | null)[] = [], testid = ""): HTMLElement {
  return h(
    "div",
    { class: `card ${side}`, ...(testid ? { "data-testid": testid } : {}) },
    h("div", { class: "emoji" }, u.emoji),
    h("div", { class: "name" }, u.name),
    h("div", { class: "stats" }, h("span", { class: "p" }, `${u.stats.pwr}`), " / ", h("span", { class: "h" }, `${u.stats.hp}`)),
    ...extra,
  );
}
