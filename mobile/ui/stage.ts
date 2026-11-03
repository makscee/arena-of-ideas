// The table's stage (arena#28, mission #26 move 1): every state is one fixed
// stage, drawn at its own size and scaled to fit the window: 1280×720 for a
// landscape window (the desktop, CrazyGames' 800×450), 390×844 for a portrait
// one (a phone). Everything inside is laid out in stage pixels, so a window
// never reflows it: the same table, smaller or larger.
import { h } from "./dom";

export type Orient = "L" | "P";

/** The stage's size per orientation. */
export const STAGE: Record<Orient, { w: number; h: number }> = { L: { w: 1280, h: 720 }, P: { w: 390, h: 844 } };

/** Landscape or portrait stage for a window (the duel's rule: narrower than 0.8 is portrait). */
export const orientOf = (w: number, hh: number): Orient => (w / hh < 0.8 ? "P" : "L");
export const orient = (): Orient => orientOf(window.innerWidth, window.innerHeight);

/** The scale that fits the stage in a window, and where its corner goes. */
export function fit(o: Orient, w: number, hh: number): { k: number; left: number; top: number } {
  const s = STAGE[o];
  const k = Math.min(w / s.w, hh / s.h);
  return { k, left: (w - s.w * k) / 2, top: (hh - s.h * k) / 2 };
}

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
    el.style.setProperty("--k", String(f.k));
    el.style.left = `${f.left}px`;
    el.style.top = `${f.top}px`;
    el.style.transform = `scale(${f.k})`;
  };
  const f = fit(o, window.innerWidth, window.innerHeight);
  el.style.setProperty("--k", String(f.k));
  el.style.left = `${f.left}px`;
  el.style.top = `${f.top}px`;
  el.style.transform = `scale(${f.k})`;
  addEventListener("resize", place);
  return el;
}
