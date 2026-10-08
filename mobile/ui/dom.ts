// Plain-DOM helpers shared by every screen of the phone client (mission #574).
import { backStays, escStep } from "./esc";
import { toggleSound } from "./sound";

export const app = document.getElementById("app")!;

export function h<K extends keyof HTMLElementTagNameMap>(tag: K, attrs: Record<string, string> = {}, ...kids: (Node | string | null)[]): HTMLElementTagNameMap[K] {
  const el = document.createElement(tag);
  for (const [k, v] of Object.entries(attrs)) {
    if (k === "class") el.className = v;
    else el.setAttribute(k, v);
  }
  for (const kid of kids) if (kid !== null) el.append(kid);
  return el;
}

/** A player's "@name", kept on one line: it never splits at a hyphen or a
 * space, and where its place is too narrow it ends in "…" (style.css .who).
 * The full name is its title. */
export function who(name: string, cls = ""): HTMLSpanElement {
  return h("span", { class: cls ? `who ${cls}` : "who", title: name }, `@${name}`);
}

export function button(label: string, onClick: () => void, cls = "", testid = ""): HTMLButtonElement {
  const b = h("button", { class: cls, ...(testid ? { "data-testid": testid } : {}) }, label);
  b.addEventListener("click", onClick);
  return b;
}

/** Replaces the screen; null kids are skipped. It forgets the last screen's
 * name (screen()) and keys (onKeys()): a screen that has a desktop layout
 * says so again after show(). */
export function show(...kids: (Node | null)[]): void {
  // The screen it replaces is gone for good: it frees what it holds.
  const free = gone;
  gone = null;
  free?.();
  delete app.dataset.screen;
  keys = null;
  leaving = null;
  app.replaceChildren(...kids.filter((k): k is Node => k !== null));
  window.scrollTo(0, 0);
}

/** Keeps the current screen as it is (its nodes, name, keys and scroll, not
 * its sheets) and returns a function that puts it back: the Codex opens over
 * any screen, a run's shop or a battle included, and Back returns to it
 * untouched. */
export function keepScreen(): () => void {
  // A battle stops playing out of sight; Back brings it back paused.
  leaving?.();
  const pauseAgain = leaving;
  // Set aside, not gone: the next show() (the Codex) mustn't free it.
  const freeLater = gone;
  gone = null;
  const kids = [...app.childNodes].filter((n) => !(n instanceof HTMLElement && n.classList.contains("overlay")));
  const name = app.dataset.screen;
  const kept = keys;
  const y = window.scrollY;
  return () => {
    show(...kids);
    if (name) app.dataset.screen = name;
    keys = kept;
    leaving = pauseAgain;
    gone = freeLater;
    window.scrollTo(0, y);
  };
}

/** What the current screen does when keepScreen() sets it aside (the
 * battle: pause). show() clears it. */
let leaving: (() => void) | null = null;
export function onLeave(fn: () => void): void {
  leaving = fn;
}

/** What the current screen frees when show() replaces it for good (the
 * battle: its timer, its resize listener, its timeline's observer). A
 * screen keepScreen() set aside keeps it until it is shown and replaced. */
let gone: (() => void) | null = null;
export function onGone(fn: () => void): void {
  gone = fn;
}

// ---------- desktop (round 2, R2-9) ----------

/** 1024px and wider: the desktop layout (style.css, the same breakpoint).
 * Below it the phone layout stays as it is. */
export const desktopQuery = matchMedia("(min-width: 1024px)");
export const isDesktop = (): boolean => desktopQuery.matches;

/** Names the screen on #app (data-screen), which turns on its desktop layout
 * in style.css; a screen without a name (the name
 * screen) stays a phone column at every width. */
export function screen(name: "home" | "shop" | "result" | "over" | "stats" | "codex" | "battle" | "ideas"): void {
  app.dataset.screen = name;
}

/** The current screen's keyboard, at every width: show() clears it. A key
 * typed in a field, or with Ctrl/Cmd/Alt, never reaches it; nor does an Esc
 * that closed something (escStep). Returning true means handled (its
 * default, like Space clicking a focused button, is stopped). */
let keys: ((e: KeyboardEvent) => boolean) | null = null;
export function onKeys(fn: (e: KeyboardEvent) => boolean): void {
  keys = fn;
}

// ---------- Esc (round 3, note 9) ----------

/** The term popover (./term.ts) says how to close it, if one is open: Esc
 * closes it before anything under it. */
let popover: (() => (() => void) | null) | null = null;
export function onPopoverEsc(fn: () => (() => void) | null): void {
  popover = fn;
}

const TEXT_INPUTS = new Set(["text", "search", "email", "url", "tel", "password", "number", ""]);
/** A field keys type into (a range or a checkbox is not one). */
function textField(t: EventTarget | null): HTMLElement | null {
  if (!(t instanceof HTMLElement)) return null;
  if (t instanceof HTMLInputElement) return TEXT_INPUTS.has(t.type) ? t : null;
  return t.tagName === "TEXTAREA" || t.isContentEditable ? t : null;
}

/** One Esc's first steps (escStep): closes the popover or the top sheet, or
 * blurs the field. "screen" and "native" are left to the screen's keys and
 * the browser. */
function escClose(field: HTMLElement | null): ReturnType<typeof escStep> {
  const closePop = popover?.() ?? null;
  const top = topLayer();
  const clearable = field instanceof HTMLInputElement && field.type === "search" && field.value !== "";
  const step = escStep({ popover: closePop !== null, overlays: top ? 1 : 0, field: field ? (clearable ? "clearable" : "plain") : "none" });
  if (step === "popover") closePop!();
  else if (step === "overlay") top!.dismiss();
  else if (step === "blur") field!.blur();
  return step;
}

// Esc first, in capture: it closes the top-most thing before anything else
// sees the key. Left to the screen ("screen"), it goes on like any key.
addEventListener(
  "keydown",
  (e) => {
    if (e.key !== "Escape" || e.ctrlKey || e.metaKey || e.altKey || e.repeat) return;
    const step = escClose(textField(e.target));
    if (step === "screen" || step === "native") return;
    e.preventDefault();
    e.stopImmediatePropagation();
  },
  { capture: true },
);

// ---------- Back (R3-26) ----------

// The phone's Back gesture (Android's Esc) and the browser's Back are one Esc
// while Esc has something to do (esc.ts backStays: a sheet, the shop, a
// battle, the Codex...): one history entry of ours sits on top, and Back
// pops it instead of leaving the game. Home has no Esc, so Back leaves there.
const GUARD = "arenaBack";
/** The top history entry is ours. */
let armed = false;
/** Our own history.back() calls whose popstate is still to come. */
let ownBacks = 0;
function syncBack(): void {
  if (ownBacks > 0) return; // pushing now would race the pending back
  const stays = backStays({ overlays: topLayer() ? 1 : 0, screen: app.dataset.screen ?? "" });
  if (stays && !armed) {
    history.pushState({ [GUARD]: true }, "");
    armed = true;
  } else if (!stays && armed) {
    // A sheet closed by a tap, or Home: the entry goes, so the next Back leaves.
    armed = false;
    ownBacks++;
    history.back();
  }
}
addEventListener("popstate", (e) => {
  if (ownBacks > 0) return void (ownBacks--, syncBack());
  armed = (e.state as Record<string, unknown> | null)?.[GUARD] === true;
  if (armed) return; // Forward, back onto our entry: nothing to undo
  // The Esc path: the popover or the top sheet, else the screen's own Esc.
  const step = escClose(null);
  if (step === "screen" && !topLayer()) keys?.(new KeyboardEvent("keydown", { key: "Escape" }));
  syncBack();
});
let backQueued = false;
new MutationObserver(() => {
  if (backQueued) return;
  backQueued = true;
  queueMicrotask(() => ((backQueued = false), syncBack()));
}).observe(app, { childList: true, attributes: true, attributeFilter: ["data-screen"] });
addEventListener("keydown", (e) => {
  if (e.ctrlKey || e.metaKey || e.altKey || e.repeat) return;
  if (textField(e.target)) return;
  // M: sound on/off, on every screen (round 3, note 16).
  if (e.key === "m" || e.key === "M") return void (toggleSound(), e.preventDefault());
  // A sheet is open: the screen under it takes no keys.
  if (topLayer()) return;
  if (keys && keys(e)) e.preventDefault();
});

// ---------- overlays ----------

/** The open overlays, bottom first. Each one's `dismiss` is what a tap
 * outside it and Esc do: close it, or for the run menu, Resume. */
type Layer = { el: HTMLElement; dismiss: () => void };
const layers: Layer[] = [];
/** The top-most overlay still on the screen (show() and keepScreen() drop
 * overlays without closing them: those are forgotten here). */
function topLayer(): Layer | null {
  for (let i = layers.length - 1; i >= 0; i--) if (!layers[i]!.el.isConnected) layers.splice(i, 1);
  return layers[layers.length - 1] ?? null;
}

/** A modal sheet over the current screen: the unit sheet, the rules, the
 * trace popup, why-I-lost. Tapping outside it closes it, and so do Esc and
 * the next show(). Returns close. */
export function overlay(...kids: Node[]): () => void {
  return dismissable(null, ...kids);
}

/** overlay() whose tap outside and Esc do `dismiss(close)` instead of a plain
 * close: the run menu resumes the battle under it. */
export function dismissable(dismiss: ((close: () => void) => void) | null, ...kids: Node[]): () => void {
  const sheet = h("div", { class: "sheet stack", role: "dialog" }, ...kids);
  const back = h("div", { class: "overlay", "data-testid": "overlay" }, sheet);
  const layer: Layer = { el: back, dismiss: () => (dismiss ? dismiss(close) : close()) };
  function close(): void {
    back.remove();
    const i = layers.indexOf(layer);
    if (i >= 0) layers.splice(i, 1);
  }
  back.addEventListener("click", (e) => e.target === back && layer.dismiss());
  topLayer();
  layers.push(layer);
  app.append(back);
  return close;
}

/** overlay() with a Close button pinned to the sheet's bottom, for a sheet
 * with nothing else to tap (a unit sheet, the rules, a why-I-lost row). */
export function closable(...kids: Node[]): () => void {
  const closeBtn = button("Close", () => close(), "grow", "sheet-close");
  const close = overlay(...kids, h("div", { class: "row sheet-actions" }, closeBtn));
  return close;
}

// ---------- names that fit ----------

// A battle chip's label (a status's name runs long: "Strength ×1") shrinks too.
const FIT = ".card .name, .bv-pill:not(.two), .bv-pill.two .bv-l";
/** The smallest a fitted line goes. A card name (one line on a compact card)
 * stops at 10px on a battle card, 9px on a line or shop card ("War Drummer",
 * "Plague Doctor" on a 64px phone card, R2-17: two lines don't fit its
 * height), and ends in an ellipsis; the sheet has the full name. */
const MIN_PX = 8;
const NAME_MIN_PX = 10;
const CARD_NAME_MIN_PX = 9;

/** Shrinks each card name (and battle chip) until it fits the card, measured,
 * not guessed from letter counts; one that still doesn't fit at the smallest
 * size ends in an ellipsis. */
export function fitText(root: ParentNode = app): void {
  for (const el of root.querySelectorAll<HTMLElement>(FIT)) {
    if (!el.isConnected || el.clientWidth === 0) continue;
    el.style.fontSize = "";
    el.classList.remove("clip");
    let px = parseFloat(getComputedStyle(el).fontSize);
    const min = !el.classList.contains("name") ? MIN_PX : el.closest(".bv-card") ? NAME_MIN_PX : CARD_NAME_MIN_PX;
    while (el.scrollWidth > el.clientWidth + 0.5 && px > min) {
      px = Math.max(min, px - 0.5);
      el.style.fontSize = `${px}px`;
    }
    if (el.scrollWidth > el.clientWidth + 0.5) el.classList.add("clip");
  }
}

let fitQueued = false;
const queueFit = () => {
  if (fitQueued) return;
  fitQueued = true;
  requestAnimationFrame(() => {
    fitQueued = false;
    fitText();
  });
};
new MutationObserver(queueFit).observe(app, { childList: true, subtree: true });
addEventListener("resize", queueFit);
void document.fonts?.ready.then(queueFit);
// A name measured in a fallback font is cut wrongly: measure again once a font arrives.
document.fonts?.addEventListener?.("loadingdone", queueFit);
