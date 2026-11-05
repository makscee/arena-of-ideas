// The first screen (arena#31, mission #26 move 3; round 4: the duel's mockup, first-1280x800.png):
// one stage, like the table. The title large at the top left, the EN/РУ toggle at the top
// right, the chain demo filling the middle as two rows (today's champion's line above, a
// player's line below) at the table's piece size, one step's wire and caption at a time
// (Gambler strikes → Drummer counts that strike → the line behind powers up), the headline,
// PLAY with its note, and a quiet name field under it. PLAY skips the name form: the caller
// makes a generated name and opens round 1.
import type { LineUnit, MvpContent, UnitContent } from "../../src/mvp/contract";
import { chooseLang, LANGS, uiLang } from "../lang";
import { t } from "../i18n";
import { faceOfUnit, lineToPiece, piece, type PieceUnit } from "../ui/piece";
import { wireMiddle, type Pt } from "../ui/chain-wire";
import { button, h, onGone, screen, show, withKey } from "../ui/dom";
import { orient, stage } from "../ui/stage";

/** A player's line in the demo, by name; the pool's first units stand in when one has left it. */
const MINE = ["Gambler", "Drummer", "Hornet", "Lantern"] as const;
/** Their line when no champion is crowned yet: the mockup's four. */
const THEIRS = ["Thornback", "Bodyguard", "Archmage", "Phoenix"] as const;
const STEP_MS = 1500;

/** What the caller does to the screen while PLAY works (arena#31's fix round: a press answers at once). */
export interface StartControl {
  /** The press is acknowledged now: PLAY reads "Starting…", the chain stops, the field is off. */
  starting: () => void;
  /** The start failed: PLAY is ready again. */
  ready: () => void;
}

export interface StartOptions {
  content: MvpContent | null;
  /** Today's champion's line (GET /day), when there is one: the demo's top row. */
  champion?: LineUnit[] | null;
  /** PLAY: make a generated name, open round 1. */
  onPlay: () => void;
  /** A typed name: the name form's path. */
  onName: (name: string) => void;
  /** Shown under the buttons. */
  extra?: (HTMLElement | null)[];
  err: HTMLElement;
}

const reduced = (): boolean => matchMedia("(prefers-reduced-motion: reduce)").matches;

const byName = (pool: UnitContent[], names: readonly string[], skip = 0): UnitContent[] => {
  const named = names.map((n) => pool.find((u) => u.name === n));
  return named.every(Boolean) ? (named as UnitContent[]) : pool.slice(skip, skip + 4);
};

const asPiece = (u: UnitContent): PieceUnit => ({ name: u.name, emoji: u.emoji, stats: u.base, face: faceOfUnit(u), tier: u.tier, copies: 1, ...(u.portrait ? { portrait: u.portrait } : {}) });

/** The geometry of one orientation, in stage pixels. */
interface Layout {
  /** The demo's box: width, height, and where each row's piece boxes start. */
  w: number;
  h: number;
  cx: number[];
  theirTop: number;
  mineTop: number;
  px: number;
}

const LAYOUT_L: Layout = { w: 1280, h: 720, cx: [340, 535, 730, 925], theirTop: 100, mineTop: 280, px: 80 };
const layoutP = (compact: boolean): Layout => ({ w: 390, h: compact ? 270 : 345, cx: [52, 149, 246, 343], theirTop: compact ? 34 : 38, mineTop: compact ? 168 : 210, px: 60 });

/** One piece, placed by its box's top left. */
function placed(u: PieceUnit, side: "you" | "them", cx: number, top: number, px: number, testid: string, floor: number): HTMLElement {
  const box = h("div", { class: "st-piece", style: `left:${cx - px / 2}px;top:${top}px;width:${px}px;height:${px}px` });
  box.append(piece(u, { size: "L", side, px, testid, floor }));
  return box;
}

/** The demo: the two rows, the three steps' wires, numbers and floats, and the caption. `step()` shows a step. */
function demo(lay: Layout, mine: UnitContent[], theirs: PieceUnit[], word: boolean, narrow: boolean): { root: HTMLElement; light: (step: number) => void; caption: HTMLElement } {
  const { px, cx } = lay;
  // The table's text floor: the landscape stage is drawn at 0.625 on the 800×450 frame, where 18 is 11.
  const floor = word ? 18 : 0;
  const root = h("div", { class: "st-demo", "data-testid": "start-chain", style: `width:${lay.w}px;height:${lay.h}px` });
  theirs.forEach((u, i) => root.append(placed(u, "them", cx[i]!, lay.theirTop, px, `start-their-${i}`, floor)));
  mine.forEach((u, i) => root.append(placed(asPiece(u), "you", cx[i]!, lay.mineTop, px, `start-piece-${i}`, floor)));

  // The wires, by step. Points are on the pieces' boxes: a = Gambler, t = their front piece, b = Drummer, c/d = the two behind it.
  const mc = (i: number): Pt => ({ x: cx[i]!, y: lay.mineTop + px / 2 });
  const tc: Pt = { x: cx[0]!, y: lay.theirTop + px / 2 };
  const wires: { s: number; d: string; mid: Pt }[] = [];
  const quad = (s: number, p0: Pt, c: Pt, p1: Pt) => wires.push({ s, d: `M${p0.x} ${p0.y} Q${c.x} ${c.y} ${p1.x} ${p1.y}`, mid: wireMiddle(p0, c, p1) });
  if (narrow) {
    // The portrait stage has no room beside a column: the first wire runs straight up between the rows.
    const p0 = { x: mc(0).x, y: lay.mineTop - 14 }, p1 = { x: tc.x, y: lay.theirTop + px + 40 };
    quad(1, p0, { x: p0.x, y: (p0.y + p1.y) / 2 }, p1);
  } else {
    const p0 = { x: mc(0).x - px * 0.5, y: mc(0).y - px * 0.4 }, p1 = { x: tc.x - px * 0.55, y: tc.y + px * 0.3 };
    const c = { x: cx[0]! - px * 1.5, y: (p0.y + p1.y) / 2 };
    quad(1, p0, c, p1);
  }
  {
    const p0 = { x: mc(0).x + px * 0.5, y: mc(0).y + px * 0.5 }, p1 = { x: mc(1).x - px * 0.5, y: mc(1).y + px * 0.5 };
    quad(2, p0, { x: (p0.x + p1.x) / 2, y: mc(0).y + px * 1.25 }, p1);
  }
  for (const k of [2, 3]) {
    const p0 = { x: mc(1).x + px * 0.45, y: mc(1).y - px * 0.55 }, p1 = { x: mc(k).x - px * 0.4, y: mc(k).y - px * 0.6 };
    quad(3, p0, { x: (p0.x + p1.x) / 2, y: mc(1).y - px * (0.95 + k * 0.08) }, p1);
  }
  const svg = document.createElementNS("http://www.w3.org/2000/svg", "svg");
  svg.setAttribute("class", "st-wires");
  svg.setAttribute("viewBox", `0 0 ${lay.w} ${lay.h}`);
  svg.setAttribute("aria-hidden", "true");
  const wireEls = wires.map((w) => {
    const p = document.createElementNS("http://www.w3.org/2000/svg", "path");
    p.setAttribute("d", w.d);
    p.setAttribute("class", `st-wire s${w.s}`);
    p.setAttribute("data-testid", "start-wire");
    p.dataset.s = String(w.s);
    svg.append(p);
    return p;
  });
  root.append(svg);
  // One number per step, on its first wire.
  const discs = [1, 2, 3].map((s) => {
    const m = wires.find((w) => w.s === s)!.mid;
    const d = h("div", { class: `st-n s${s}`, "data-testid": "start-wire-n", "data-s": String(s), style: `left:${m.x}px;top:${m.y}px` }, String(s));
    root.append(d);
    return d;
  });
  // The steps' results: the hit, and the power-up on the two behind.
  const floats = [
    h("div", { class: "st-float hit s1", "data-s": "1", style: `left:${cx[0]! + px * 0.62}px;top:${lay.theirTop - 6}px` }, "−4"),
    h("div", { class: "st-float buff s3", "data-s": "3", style: `left:${cx[2]! - px * 0.35}px;top:${lay.mineTop - px * 0.45}px` }, "+1"),
    h("div", { class: "st-float buff s3", "data-s": "3", style: `left:${cx[3]! - px * 0.35}px;top:${lay.mineTop - px * 0.45}px` }, "+1"),
  ];
  root.append(...floats);
  const caption = h("div", { class: "st-caption", "data-testid": "start-caption", "aria-live": "polite" });
  const names = [mine[0]?.name ?? "", mine[1]?.name ?? ""];
  const words = [t("start.cap1", { name: names[0]! }), t("start.cap2", { name: names[1]! }), t("start.cap3")];
  const pieces = [...root.querySelectorAll<HTMLElement>(".st-piece")];
  // Who acts and who is hit at each step.
  const act: Record<number, number[]> = { 1: [4], 2: [5], 3: [5] };
  const hurt: Record<number, number[]> = { 1: [0] };
  const buff: Record<number, number[]> = { 3: [6, 7] };
  const light = (step: number) => {
    root.dataset.step = String(step);
    for (const el of [...wireEls, ...discs, ...floats]) el.dataset.on = Number(el.dataset.s) === step ? "1" : "0";
    pieces.forEach((p, i) => {
      p.classList.toggle("act", !!act[step]?.includes(i));
      p.classList.toggle("hurt", !!hurt[step]?.includes(i));
      p.classList.toggle("buff", !!buff[step]?.includes(i));
    });
    caption.textContent = words[step - 1]!;
  };
  return { root, light, caption };
}

/** The EN/РУ toggle, each in its own words; the other one keeps itself on this device and redraws the page. */
function langToggle(): HTMLElement {
  const now = uiLang();
  const pick = (code: (typeof LANGS)[number]) => {
    const b = button(t(`lang.${code}`), () => code !== now && chooseLang(code), code === now ? "lang-on" : "", `lang-${code}`);
    b.setAttribute("aria-pressed", String(code === now));
    b.lang = code;
    return b;
  };
  return h("div", { class: "st-lang", role: "group", "aria-label": t("lang.label"), "data-testid": "start-lang" }, ...LANGS.map(pick));
}

export function startScreen(o: StartOptions): StartControl {
  const pool = o.content?.units ?? [];
  const mine = byName(pool, MINE);
  const champ = o.champion && o.champion.length >= 4 && o.content ? o.champion.slice(0, 4).map((u) => lineToPiece(u, o.content!)) : null;
  const theirs = champ ?? byName(pool, THEIRS, 4).map(asPiece);
  let typed = "";
  let busy = false;
  let ui = draw();

  function draw(): { starting: () => void; ready: () => void } {
    const o2 = orient();
    const L = o2 === "L";
    const input = h("input", { placeholder: t("start.ownName"), maxlength: "24", autocomplete: "nickname", "aria-label": t("start.ownName"), "data-testid": "name-input" });
    input.value = typed;
    input.addEventListener("input", () => (typed = input.value));
    const play = withKey(button(t("start.play"), () => o.onPlay(), "primary play-start", "start-play"), "Enter");
    const label = [...play.childNodes];
    const named = button(t("name.enter"), () => o.onName(input.value.trim()), "small", "name-submit");
    // Enter in the name field plays under that name; anywhere else it is PLAY.
    input.addEventListener("keydown", (e) => {
      if (e.key !== "Enter") return;
      e.stopPropagation();
      e.preventDefault();
      if (input.value.trim()) o.onName(input.value.trim());
      else o.onPlay();
    });
    const onKey = (e: KeyboardEvent) => {
      if (e.key === "Enter" && !(e.target instanceof HTMLInputElement) && !(e.target instanceof HTMLButtonElement)) o.onPlay();
    };
    addEventListener("keydown", onKey);

    const compact = !L && window.innerHeight < 780;
    const lay = L ? LAYOUT_L : layoutP(compact);
    const d = demo(lay, mine, theirs, L, !L);
    const brand = h("h1", { class: "st-brand", "data-testid": "start-brand" }, t("name.title"));
    const headline = h("h2", { class: "st-title", "data-testid": "start-title" }, t("start.title"));
    const lead = h("p", { class: "st-lead" }, t("start.lead"));
    const note = h("div", { class: "st-note" }, t("start.note"));
    const progress = h("div", { class: "start-progress", "aria-hidden": "true" }, h("i"));
    const nameRow = h("div", { class: "st-name" }, input, named);
    const sides = [
      h("div", { class: "st-side them", "data-testid": "start-side", style: `top:${lay.theirTop + (L ? 30 : -34)}px` }, t("start.sideTheirs")),
      h("div", { class: "st-side you", "data-testid": "start-side", style: `top:${lay.mineTop + (L ? 30 : -34)}px` }, t("start.sideYours")),
    ];
    d.root.append(...sides);
    const root = h("div", { class: `start ${L ? "L" : "P"}${compact ? " compact" : ""}`, "data-testid": "start" });
    const extra = (o.extra ?? []).filter((x): x is HTMLElement => x !== null);
    const playRow = h("div", { class: "st-play" }, play, note, o.err);
    if (L) root.append(brand, langToggle(), d.root, d.caption, headline, lead, playRow, progress, nameRow, ...extra);
    else root.append(h("div", { class: "st-top" }, brand, langToggle()), d.root, h("div", { class: "st-mid" }, d.caption, headline, lead), h("div", { class: "st-foot" }, playRow, progress, nameRow, ...extra));

    show(stage([root], () => (ui = draw())));
    screen("start");
    // The chain plays itself, looping; reduced motion shows the last step, still.
    let step = 0;
    let timer: ReturnType<typeof setInterval> | undefined;
    if (reduced()) d.light(3);
    else {
      d.light(1);
      timer = setInterval(() => d.light((step = (step + 1) % 3) + 1), STEP_MS);
    }
    const stop = () => timer !== undefined && (clearInterval(timer), (timer = undefined));
    onGone(() => (stop(), removeEventListener("keydown", onKey)));
    const set = (on: boolean) => {
      root.classList.toggle("starting", on);
      if (on) play.setAttribute("aria-busy", "true");
      else play.removeAttribute("aria-busy");
      play.replaceChildren(...(on ? [t("start.starting")] : label));
      input.disabled = named.disabled = on;
    };
    if (busy) (stop(), set(true));
    else play.focus();
    return {
      starting: () => (stop(), set(true)),
      ready: () => (set(false), play.focus()),
    };
  }

  return {
    starting: () => ((busy = true), ui.starting()),
    ready: () => ((busy = false), ui.ready()),
  };
}
