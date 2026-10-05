// The stats page (mission #574). Slice 11 owns this file: the champion
// history, the discovered fusions (api.fusions(), slice 10's list), unit win
// and pick rates and your records (api.stats()). Home's "Stats" button opens
// it; a past battle opens in battleScreen (./battle.ts).
import type { MvpContent } from "../../src/mvp/contract";
import { button, h, show } from "../ui/dom";

export function statsScreen(a: { content: MvpContent; onBack: () => void }): void {
  // Slice 11 replaces this stub.
  show(
    h("h1", {}, "STATS"),
    h("div", { class: "panel dim", "data-testid": "stats" }, "Stats arrive in slice 11."),
    h("div", { class: "spacer" }),
    button("Back", a.onBack, "primary", "stats-back"),
  );
}
