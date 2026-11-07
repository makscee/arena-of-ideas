// Home as a state of the table (arena#31, mission #26 move 1; arena#51, mission #44 move 6): one stage, the shop's own places.
// The HUD's plates on top (the title, RATING n, the ≡ Esc menu), a dark banner "Today's champion · @name" where the shop has its
// rail, the champion's line as TO BEAT in the shop's line frames, YOUR LINE as five empty frames in the shop's bench row (no
// fictional round, hearts or gold before a run starts), and the tray: the Workshop card where the detail is, the vote when
// one is open, the rows Codex, Forge and Champions where the offers are, and PLAY where Fight is.
// This file lays the table out; main.ts makes what needs the game (the buttons, the banner's words).
import type { LineUnit, MvpContent } from "../../src/mvp/contract";
import { lineToPiece, piece } from "../ui/piece";
import { h } from "../ui/dom";
import { orient, stage } from "../ui/stage";
import { cellHeight, fieldLayout, fit, type Orient } from "../ui/stage-fit";

export interface HomeParts {
  content: MvpContent;
  /** Today's champion's line: the TO BEAT row (never the next opponent's). Null before a champion is crowned. */
  champion: LineUnit[] | null;
  /** The HUD plates: RATING n (with the last run's change), and the ≡ Esc menu. */
  title: string;
  rating: HTMLElement;
  menu: HTMLElement;
  /** The banner, and a one-off notice under it (the dev "End day now"). */
  banner: HTMLElement;
  notice: HTMLElement | null;
  /** The labels of the two lines, and what your line says before a run. */
  toBeat: string;
  yourLine: string;
  yourLineSub: string;
  /** PLAY (RESUME; its note is inside it) and New run (a run waits). */
  play: HTMLElement;
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

/** The piece sizes of the shop's line and bench per stage (main.ts pieceSizes): Home's frames are the same boxes. */
export function homeSizes(o: Orient, stageH: number): { line: number; bench: number } {
  if (o === "P") return { line: 42, bench: 34 };
  const f = fieldLayout(stageH);
  return { line: f.s, bench: f.sb };
}

/** A frame's height for a piece of side `s`: the piece's own reach plus the tier row (`bare`: none). */
const frameStyle = (s: number, floor: number, bare = false): string => {
  const c = cellHeight(s, bare, floor);
  return `--s:${s}px;padding-top:${c.top}px;height:${c.h}px`;
};

/** The table: a stage holding Home. */
export function homeTable(p: HomeParts): HTMLElement {
  const o = orient();
  const floor = FLOOR[o];
  const lay = o === "L" ? fieldLayout(fit("L", window.innerWidth, window.innerHeight, true).h) : null;
  const sz = homeSizes(o, lay ? fit("L", window.innerWidth, window.innerHeight, true).h : 844);

  // TO BEAT: the champion's units in the shop's line frames, front first; a short line leaves plain frames.
  const line = p.champion ?? [];
  const theirs = h(
    "div",
    { class: "hm-frames hm-theirs", "data-testid": "home-their" },
    ...Array.from({ length: 5 }, (_, i) => {
      const u = line[i];
      if (!u) return h("div", { class: "hm-cell empty blank", "aria-hidden": "true", style: frameStyle(sz.line, floor) });
      return h("div", { class: "hm-cell them", style: frameStyle(sz.line, floor) }, piece(lineToPiece(u, p.content), { size: o === "L" ? "L" : "M", side: "them", px: o === "L" ? sz.line : sz.line - 6, testid: `home-champ-${i}`, floor }));
    }),
  );
  // YOUR LINE: five empty frames, numbered, where the shop's bench is.
  const slots = h(
    "div",
    { class: "hm-frames hm-slots" },
    ...Array.from({ length: 5 }, (_, i) => h("div", { class: "hm-cell empty", "data-testid": `home-slot-${i}`, style: frameStyle(sz.bench, floor) }, h("span", { class: "hm-num" }, String(i + 1)))),
  );
  const side = (cls: string, text: string, testid: string, sub?: string) =>
    h("div", { class: `hm-side ${cls}`, "data-testid": testid }, h("div", { class: "label" }, text), sub ? h("div", { class: "sub" }, sub) : null);

  const tray = h(
    "div",
    { class: "hm-tray", "data-testid": "home-tray" },
    p.workshop,
    p.vote,
    h("div", { class: "hm-rows" }, ...p.rows.filter((r): r is HTMLElement => r !== null)),
    // Where Reroll and Fight are: New run (while a run waits) above, PLAY below.
    h("div", { class: "hm-go" }, p.newRun, p.play),
  );
  const root = h(
    "div",
    { class: `home ${o}`, "data-testid": "home" },
    h("div", { class: "hm-hud", "data-testid": "hud" }, h("h1", { class: "hm-brand", "data-testid": "home-title" }, p.title), h("span", { class: "hud-gap" }), p.rating, p.menu),
    h("section", { class: "hm-champion", "data-testid": "champion" }, p.banner),
    p.notice,
    side("them", p.toBeat, "home-side-them"),
    theirs,
    side("you", p.yourLine, "home-side-you", p.yourLineSub),
    slots,
    tray,
    p.err,
  );
  const table = stage([root], p.redraw, o === "L");
  table.classList.add("t-home");
  if (lay) for (const [k, v] of Object.entries({ "wells-top": lay.wellsTop, "line-top": lay.lineTop, "bench-top": lay.benchTop, "tray-top": lay.trayTop })) table.style.setProperty(`--${k}`, `${v}px`);
  return table;
}
