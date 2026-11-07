// The table's stage geometry (arena#28), free of the DOM so it can be tested: ui/stage.ts draws with it.
import { pieceReach } from "./reach";

export type Orient = "L" | "P";

/** The stage's size per orientation. */
export const STAGE: Record<Orient, { w: number; h: number }> = { L: { w: 1280, h: 720 }, P: { w: 390, h: 844 } };

/** Landscape or portrait stage for a window (the duel's rule: narrower than 0.8 is portrait). */
export const orientOf = (w: number, hh: number): Orient => (w / hh < 0.8 ? "P" : "L");

/** The shortest portrait stage, px: a window shorter than this at its width scales the stage down instead. */
export const P_MIN_H = 690;

/** The tallest a flexible landscape stage grows (4:3 windows): 1280×960. */
export const L_MAX_H = 960;

/** The scale that fits the stage in a window, where its corner goes and its height. The landscape
 * stage is 1280×720; `flex` (the shop's table) lets it grow taller to use a window that is not
 * 16:9 (1440×900 is 1280×800, 1024×768 is 1280×960) instead of leaving bands. The portrait stage
 * is 390 wide and as tall as the window allows (its height is `h`, from 690 up): a phone's width
 * sets the scale, so a 360×640 phone keeps nearly 1:1 and its taps stay 44px; only a window
 * shorter than that scales it down. */
export function fit(o: Orient, w: number, hh: number, flex = false): { k: number; left: number; top: number; h: number } {
  const s = STAGE[o];
  if (o === "P") {
    const k = Math.min(w / s.w, hh / P_MIN_H);
    return { k, left: (w - s.w * k) / 2, top: 0, h: hh / k };
  }
  const k = Math.min(w / s.w, hh / s.h);
  const h = flex ? Math.min(L_MAX_H, Math.max(s.h, hh / k)) : s.h;
  return { k, left: (w - s.w * k) / 2, top: (hh - h * k) / 2, h };
}

/** A cell's height for a piece of side `s` at the landscape stage's text floor: the tab above, the
 * rim numbers, the name and (unless bare) the tier and price row below. */
export function cellHeight(s: number, bare = false, floor = 18): { top: number; h: number } {
  const r = pieceReach(s, floor);
  const top = Math.max(15, r.above + 2);
  return { top, h: Math.round(top + r.below + 7 + (bare ? 0 : floor ? 30 : 20)) };
}

/** The shop's field on the flexible landscape stage of height `H` (stage px). The tray keeps its
 * size at the foot; their wells, your line and the bench share what is above it, and the line's
 * pieces grow into the height a taller window gives (they never shrink under 84). Rows are
 * spaced to use the rest. All the columns are the same five. */
export interface FieldLayout {
  /** The line's piece, and the bench's; `sDock` is the line's piece while a panel narrows the field. */
  s: number;
  sb: number;
  sDock: number;
  wellsTop: number;
  wellsH: number;
  lineTop: number;
  lineH: number;
  benchTop: number;
  benchH: number;
  trayTop: number;
}
export function fieldLayout(H: number): FieldLayout {
  const trayTop = H - 244;
  const wellsTop = 76;
  const wellsH = 50;
  const room = trayTop - 6 - (wellsTop + wellsH + 8);
  let s = 84;
  for (let t = 116; t >= 84; t -= 2) {
    const sb = Math.round(t * 0.68);
    if (cellHeight(t).h + 4 + cellHeight(sb, true).h <= room) {
      s = t;
      break;
    }
  }
  const sb = Math.round(s * 0.68);
  const lineH = cellHeight(s).h;
  const benchH = cellHeight(sb, true).h;
  const spare = Math.max(0, room - lineH - 4 - benchH);
  const lineTop = wellsTop + wellsH + 8 + Math.round(spare / 2);
  const benchTop = lineTop + lineH + 4 + Math.round(spare / 2);
  return { s, sb, sDock: Math.min(s, 86), wellsTop, wellsH, lineTop, lineH, benchTop, benchH, trayTop };
}
