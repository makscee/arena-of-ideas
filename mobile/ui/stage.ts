// The table's stage (arena#28, mission #26 move 1): every state is one fixed
// stage, drawn at its own size and scaled to fit the window: 1280×720 for a
// landscape window (the desktop, CrazyGames' 800×450), 390×844 for a portrait
// one (a phone). Everything inside is laid out in stage pixels, so a window
// never reflows it: the same table, smaller or larger.
import { h } from "./dom";

import { fit, orientOf, type Orient } from "./stage-fit";

/** Landscape or portrait for the window now. */
export const orient = (): Orient => orientOf(window.innerWidth, window.innerHeight);

/** A stage holding `kids`, fitted to the window now and on every resize. `onFlip`
 * runs when the window crosses between landscape and portrait (the screen draws
 * itself again at the other stage). The returned stage drops its listener when
 * it leaves the page. */
export function stage(kids: (Node | null)[], onFlip: () => void): HTMLElement {
  const o = orient();
  const el = h("div", { class: `stage ${o}`, "data-testid": "stage", "data-orient": o }, ...kids);
  const place = () => {
    if (!el.isConnected) return removeEventListener("resize", place);
    if (orient() !== o) {
      removeEventListener("resize", place);
      return onFlip();
    }
    const f = fit(o, window.innerWidth, window.innerHeight);
    apply();
  };
  const apply = () => {
    const f = fit(o, window.innerWidth, window.innerHeight);
    el.style.setProperty("--k", String(f.k));
    el.style.left = `${f.left}px`;
    el.style.top = `${f.top}px`;
    el.style.height = `${f.h}px`;
    el.style.transform = `scale(${f.k})`;
    // A portrait stage under 780 tall is the compact table (their line's empty wells make room).
    el.classList.toggle("compact", o === "P" && f.h < 780);
  };
  apply();
  addEventListener("resize", place);
  return el;
}
