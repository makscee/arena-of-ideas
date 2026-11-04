// The table's stage geometry (arena#28), free of the DOM so it can be tested: ui/stage.ts draws with it.
export type Orient = "L" | "P";

/** The stage's size per orientation. */
export const STAGE: Record<Orient, { w: number; h: number }> = { L: { w: 1280, h: 720 }, P: { w: 390, h: 844 } };

/** Landscape or portrait stage for a window (the duel's rule: narrower than 0.8 is portrait). */
export const orientOf = (w: number, hh: number): Orient => (w / hh < 0.8 ? "P" : "L");

/** The shortest portrait stage, px: a window shorter than this at its width scales the stage down instead. */
export const P_MIN_H = 690;

/** The scale that fits the stage in a window, where its corner goes and its height. The landscape
 * stage is always 1280×720. The portrait stage is 390 wide and as tall as the window allows (its
 * height is `h`, from 690 up): a phone's width sets the scale, so a 360×640 phone keeps nearly
 * 1:1 and its taps stay 44px; only a window shorter than that scales it down. */
export function fit(o: Orient, w: number, hh: number): { k: number; left: number; top: number; h: number } {
  const s = STAGE[o];
  if (o === "P") {
    const k = Math.min(w / s.w, hh / P_MIN_H);
    return { k, left: (w - s.w * k) / 2, top: 0, h: hh / k };
  }
  const k = Math.min(w / s.w, hh / s.h);
  return { k, left: (w - s.w * k) / 2, top: (hh - s.h * k) / 2, h: s.h };
}
