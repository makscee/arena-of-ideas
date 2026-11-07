// Home as a state of the table (arena#31, mission #26 move 1; the duel's mockup, home-1280x800.png):
// one stage like the shop's. The HUD's plates on top (the title, RATING n, the ≡ Esc menu), the gold
// banner "Today's champion · @name", the champion's line as TO BEAT (real pieces, centred, sized by
// ui/line-layout.ts), YOUR LINE as five empty slots with PLAY over them and its note under, and the
// tray: the Workshop card, the vote when one is open, and the rows Codex, Forge, Champions.
// This file lays the table out; main.ts makes what needs the game (the buttons, the banner's words).
import type { LineUnit, MvpContent } from "../../src/mvp/contract";
import { linePitch, pieceCap, spreadLine } from "../ui/line-layout";
import { lineToPiece, piece, pieceReach } from "../ui/piece";
import { h } from "../ui/dom";
import { orient, stage } from "../ui/stage";
import { fit, STAGE, type Orient } from "../ui/stage-fit";

export interface HomeParts {
  content: MvpContent;
  /** Today's champion's line: the TO BEAT row (never the next opponent's). Null before a champion is crowned. */
  champion: LineUnit[] | null;
  /** The HUD plates: RATING n (with the last run's change), and the ≡ Esc menu. */
  title: string;
  rating: HTMLElement;
  menu: HTMLElement;
  /** The gold banner, and a one-off notice under it (the dev "End day now"). */
  banner: HTMLElement;
  notice: HTMLElement | null;
  /** The labels of the two lines. */
  toBeat: string;
  yourLine: string;
  /** PLAY (RESUME), its note, and New run (a run waits). */
  play: HTMLElement;
  note: HTMLElement | null;
  newRun: HTMLElement | null;
  /** The tray. */
  workshop: HTMLElement;
  vote: HTMLElement;
  rows: (HTMLElement | null)[];
  err: HTMLElement;
  /** The window crossed between landscape and portrait, or a flexible stage changed a step: draw again. */
  redraw: () => void;
}

/** The table's text floor on the landscape stage (drawn at 0.625 on 800×450, where 18 reads 11). */
const FLOOR: Record<Orient, number> = { L: 18, P: 0 };
/** The line's piece on Home: the shop's, or the bigger one the stage's width allows (never past the cap). */
const PIECE_L = 84;

/** A taller window gives the landscape line bigger pieces, as the shop's does (84 on the 720 stage, up to the cap on a 960 one). */
export function landscapePiece(stageH: number): number {
  return Math.min(pieceCap("L", 5), PIECE_L + Math.round(((Math.min(960, Math.max(720, stageH)) - 720) / 240) * (pieceCap("L", 5) - PIECE_L)));
}

/** Where the line's five columns sit (stage x of each centre) and how large its piece and slot are. */
export function homeGeometry(o: Orient): { px: number; pitch: number; cx: number[]; above: number; rowH: number } {
  // The phone's line spreads over the stage's whole width (the tray is as wide), and its piece is the one that
  // leaves a counter chip or a star room beside its neighbour (ui/line-layout.ts spreadLine); the landscape line is the shop's pitch.
  const wide = spreadLine(STAGE.P.w, 5);
  const pitch = o === "L" ? linePitch(o, 5) : wide.pitch;
  const px = o === "L" ? landscapePiece(fit("L", window.innerWidth, window.innerHeight, true).h) : wide.px;
  const mid = o === "L" ? 640 : 195;
  const reach = pieceReach(px, FLOOR[o]);
  return { px, pitch, cx: [0, 1, 2, 3, 4].map((i) => mid + (i - 2) * pitch), above: reach.above, rowH: reach.above + reach.below + 4 };
}

/** The table: a stage holding Home. */
export function homeTable(p: HomeParts): HTMLElement {
  const o = orient();
  const g = homeGeometry(o);
  const floor = FLOOR[o];
  const box = (cx: number, top: number) => ({ style: `left:${Math.round(cx - g.px / 2)}px;top:${top}px;width:${g.px}px;height:${g.px}px` });

  // TO BEAT: the champion's units centred on the row, their order front first.
  const line = p.champion ?? [];
  const start = (5 - line.length) / 2;
  const theirs = h(
    "div",
    { class: "hm-row hm-theirs", "data-testid": "home-their", style: `height:${g.rowH}px` },
    ...line.slice(0, 5).map((u, i) => {
      const cell = h("div", { class: "hm-pbox", ...box(g.cx[0]! + (start + i) * g.pitch, g.above) });
      cell.append(piece(lineToPiece(u, p.content), { size: "M", side: "them", px: g.px, testid: `home-champ-${i}`, floor }));
      return cell;
    }),
  );
  // YOUR LINE: five empty slots, PLAY over their middle.
  const slots = h(
    "div",
    { class: "hm-row hm-slots", style: `height:${g.px}px` },
    ...g.cx.map((cx, i) => h("div", { class: "hm-slot", "data-testid": `home-slot-${i}`, ...box(cx, 0) }, h("span", {}, String(i + 1)))),
    h("div", { class: "hm-playbox" }, p.play),
  );
  const side = (cls: string, text: string, testid: string) => h("div", { class: `hm-side ${cls}`, "data-testid": testid, style: `top:${g.above + 22}px` }, text);

  const tray = h("div", { class: "hm-tray", "data-testid": "home-tray" }, p.workshop, p.vote, h("div", { class: "hm-rows" }, ...p.rows.filter((r): r is HTMLElement => r !== null)));
  const root = h(
    "div",
    { class: `home ${o}`, "data-testid": "home", style: `--px:${g.px}px;--slot:${g.px}px` },
    h("div", { class: "hm-hud", "data-testid": "hud" }, h("h1", { class: "hm-brand", "data-testid": "home-title" }, p.title), h("span", { class: "hud-gap" }), p.rating, p.menu),
    h("div", { class: "hm-gap" }),
    h("section", { class: "hm-sec hm-champion", "data-testid": "champion" }, p.banner, p.notice, h("div", { class: "hm-lines" }, side("them", p.toBeat, "home-side-them"), theirs)),
    h("div", { class: "hm-gap" }),
    h("section", { class: "hm-sec hm-you" }, h("div", { class: "hm-lines" }, side("you", p.yourLine, "home-side-you"), slots), h("div", { class: "hm-under" }, p.note, p.newRun, p.err)),
    h("div", { class: "hm-gap" }),
    tray,
  );
  const table = stage([root], p.redraw, o === "L");
  table.classList.add("t-home");
  return table;
}
