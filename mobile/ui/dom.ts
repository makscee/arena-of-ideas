// Plain-DOM helpers shared by every screen of the phone client (mission #574).
import { t } from "../i18n";
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

/** A clickable element that is not a <button> (a card) made reachable by
 * keyboard (#1064): Tab reaches it, Enter clicks it (the keydown below), and
 * style.css shows its focus. `label` names it for a screen reader. */
export function actionable<T extends HTMLElement>(el: T, label = ""): T {
  el.tabIndex = 0;
  el.setAttribute("role", "button");
  if (label) el.setAttribute("aria-label", label);
  return el;
}

/** Puts a key on a button, as a keycap after its label, on desktop (the shop's
 * "Reroll R"): the key is visible where it is used (#1064). */
export function withKey<T extends HTMLElement>(el: T, key: string): T {
  if (keysQuery.matches) el.append(" ", h("kbd", {}, keyLabel(key)));
  return el;
}

/** Notes which control the keyboard is on (its test id, and its place among
 * those) and returns a function that puts the focus back on the same one
 * after a redraw (#1064). */
export function rememberFocus(): () => void {
  const a = document.activeElement;
  const scope: ParentNode = a instanceof HTMLElement ? a.closest(".overlay") ?? app : app;
  const key = a instanceof HTMLElement && app.contains(a) ? a.dataset.testid ?? "" : "";
  const same = () => (key ? [...scope.querySelectorAll<HTMLElement>("[data-testid]")].filter((e) => e.dataset.testid === key) : []);
  const nth = a instanceof HTMLElement ? same().indexOf(a) : -1;
  return () => {
    if (nth < 0 || (document.activeElement && document.activeElement !== document.body && app.contains(document.activeElement))) return;
    const els = same();
    (els[nth] ?? els[els.length - 1])?.focus({ preventScroll: true });
  };
}

/** Runs a redraw and puts the keyboard back on the same control. */
export function keepFocus(redraw: () => void): void {
  const back = rememberFocus();
  redraw();
  back();
}

/** A screen restored by keepScreen() or redrawn by show() keeps the keyboard
 * where it was: the focused control's test id (the shop redraws after every
 * decision; Tab must not jump back to the top each time). */
let focusKey = "";
addEventListener("focusin", (e) => {
  const el = e.target;
  focusKey = el instanceof HTMLElement && app.contains(el) && !el.closest(".overlay") ? (el.dataset.testid ?? "") : focusKey;
});

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
  const had = app.contains(document.activeElement) && document.activeElement !== app ? focusKey : "";
  app.replaceChildren(...kids.filter((k): k is Node => k !== null));
  window.scrollTo(0, 0);
  if (had) {
    const again = [...app.querySelectorAll<HTMLElement>("[data-testid]")].find((e) => e.dataset.testid === had && !e.closest(".overlay"));
    if (again && !(again as HTMLButtonElement).disabled) again.focus({ preventScroll: true });
  }
  focusLost = byKey && (document.activeElement === document.body || !app.contains(document.activeElement));
}
/** The last input was a key, not the pointer: a focus lost then is the keyboard's. */
let byKey = false;
addEventListener("keydown", () => (byKey = true), true);
addEventListener("pointerdown", () => (byKey = false), true);
/** The last show() found the control the keyboard was on gone (a confirm
 * button that did its work): the new screen puts the focus somewhere itself. */
let focusLost = false;
export const keyboardLost = (): boolean => focusLost;

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

/** Where the screens take keys (#1217): the desktop, and a landscape window
 * with a mouse below it (CrazyGames' 821×462). A phone takes none. */
const keysQuery = matchMedia("(min-width: 1024px), (orientation: landscape) and (pointer: fine)");
export const hasKeys = (): boolean => keysQuery.matches;

/** The CrazyGames build (VITE_CG, mission #1214): its fullscreen owns Esc,
 * so Backspace (and the on-screen button) is Back there. */
export const cgBuild = ((import.meta.env.VITE_CG as string | undefined) ?? "") !== "" && import.meta.env.VITE_CG !== "0";

/** A key as the screens read it (#1217): by its place on the keyboard
 * (KeyboardEvent.code), so WASD, the digits and the letters are the same
 * keys on a Russian or an AZERTY layout. A key without a code (one a screen
 * makes itself, the browser's Back) is its `key`. In the CrazyGames build
 * Esc is nothing and Backspace outside a field is Esc. */
export function keyOf(e: KeyboardEvent): string {
  const c = e.code;
  if (/^Key[A-Z]$/.test(c)) return c.slice(3).toLowerCase();
  if (/^(Digit|Numpad)[0-9]$/.test(c)) return c.slice(-1);
  if (c === "Space") return " ";
  if (c === "Slash" || c === "NumpadDivide") return "/";
  if (cgBuild && c === "Escape") return "";
  if (cgBuild && c === "Backspace" && !textField(e.target)) return "Escape";
  return e.key;
}

/** A key's name on a keycap: Esc is ⌫ in the CrazyGames build. */
export const keyLabel = (k: string): string => (cgBuild && k === "Esc" ? "⌫" : k);

/** Names the screen on #app (data-screen), which turns on its desktop layout
 * in style.css; a screen without a name (the name
 * screen) stays a phone column at every width. */
export function screen(name: "home" | "shop" | "result" | "over" | "stats" | "codex" | "battle" | "ideas" | "forge"): void {
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
    if (keyOf(e) !== "Escape" || e.ctrlKey || e.metaKey || e.altKey || e.repeat) return;
    const step = escClose(textField(e.target));
    if (step === "screen" || step === "native") return;
    e.preventDefault();
    e.stopImmediatePropagation();
  },
  { capture: true },
);

// ---------- Back and Forward (R3-26, #1146) ----------

// Every screen change is a history entry of its own, so the browser's Back
// and Forward (and the phone's Back gesture) step through the screens. A
// screen's entry is named by its group: the shop, a fight, its result and the
// run's end are one group ("run"), the Codex's tabs are one. Over an entry
// sits at most one guard entry while Back must close something first: a
// sheet is open, or a battle is playing (Back is its Esc).
const GUARD = "arenaBack";
const GROUP: Record<string, string> = { home: "home", shop: "run", result: "run", battle: "run", over: "run", codex: "codex", stats: "codex", ideas: "ideas", forge: "forge" };
const groupOf = (name: string | undefined): string | undefined => (name ? GROUP[name] : undefined);
type NavState = { at: number; group: string; [GUARD]?: true };
const navState = (): NavState | null => {
  const s = history.state as Partial<NavState> | null;
  return s && typeof s.at === "number" && typeof s.group === "string" ? (s as NavState) : null;
};
/** The group of each entry of this page's trail, by position. */
const trail: string[] = [];
/** Where we are in the trail. A reload keeps the entry it was on. */
let at = navState()?.at ?? 0;
const was = navState();
if (was) trail[at] = was.group;
/** The top history entry is the guard over entry `at`. A reload with a sheet
 * open (the language switch in the Throne's settings, M5-3) keeps it on top:
 * it counts as armed, so the first sync takes it off instead of stacking a second one. */
let armed = (history.state as Record<string, unknown> | null)?.[GUARD] === true;
/** Our own history moves whose popstate is still to come. */
let ownBacks = 0;
/** Rebuilds a screen group from the server (main.ts): Back onto a screen that is gone. */
let restore: ((group: string) => Promise<void>) | null = null;
export function onRestore(fn: (group: string) => Promise<void>): void {
  restore = fn;
}
/** A restore is out: the old screen may redraw meanwhile, and that is no new screen. */
let restoring = false;
function syncBack(): void {
  if (ownBacks > 0 || restoring) return; // pushing now would race the pending move
  const name = app.dataset.screen;
  const group = groupOf(name);
  if (group && group !== trail[at]) {
    if (trail[at] === undefined) {
      // The first screen of the page takes the entry the page loaded with.
      trail[at] = group;
      history.replaceState({ at, group }, "");
      armed = false;
    } else if (trail[at - 1] === group) {
      // An in-app Back button onto the screen before: the browser's Back.
      const steps = armed ? 2 : 1;
      armed = false;
      at--;
      ownBacks++;
      history.go(-steps);
      return;
    } else {
      // A new screen: a new entry (it takes the guard's place if one is up).
      at++;
      trail.length = at;
      trail[at] = group;
      if (armed) history.replaceState({ at, group }, "");
      else history.pushState({ at, group }, "");
      armed = false;
    }
  }
  const stays = backStays({ overlays: topLayer() ? 1 : 0, screen: name ?? "" });
  if (stays && !armed) {
    history.pushState({ at, group: trail[at] ?? "", [GUARD]: true }, "");
    armed = true;
  } else if (!stays && armed) {
    // A sheet closed by a tap, or the battle over: the guard goes.
    armed = false;
    ownBacks++;
    history.back();
  }
}
addEventListener("popstate", (e) => {
  if (ownBacks > 0) return void (ownBacks--, syncBack());
  const s = e.state as Partial<NavState> | null;
  const moved = typeof s?.at === "number" && s.at !== at;
  if (s?.[GUARD] === true) {
    // Forward, back onto a guard: nothing to undo, and it goes if nothing needs it.
    armed = true;
    if (typeof s.at === "number") at = s.at;
    return syncBack();
  }
  const wasArmed = armed;
  armed = false;
  if (wasArmed && !moved) {
    // The guard popped: the Esc path, the popover or the top sheet, else the screen's own Esc.
    const step = escClose(null);
    if (step === "screen" && !topLayer()) keys?.(new KeyboardEvent("keydown", { key: "Escape" }));
    return syncBack();
  }
  if (!moved || !s || typeof s.at !== "number" || typeof s.group !== "string") return syncBack();
  // Another screen's entry (Back or Forward over a screen change).
  at = s.at;
  trail[at] = s.group;
  const target = s.group;
  // The Codex and the Forge sit over a kept screen: their Back puts it back as it was.
  const shown = groupOf(app.dataset.screen);
  if (shown !== target && (shown === "codex" || shown === "forge")) keys?.(new KeyboardEvent("keydown", { key: "Escape" }));
  if (groupOf(app.dataset.screen) !== target && restore) {
    restoring = true;
    void restore(target).finally(() => ((restoring = false), syncBack()));
    return;
  }
  syncBack();
});
let backQueued = false;
new MutationObserver(() => {
  if (backQueued) return;
  backQueued = true;
  queueMicrotask(() => ((backQueued = false), syncBack()));
}).observe(app, { childList: true, attributes: true, attributeFilter: ["data-screen"] });
/** Something Enter presses: a button, a link, or a card made actionable(). */
const pressable = (t: EventTarget | null): t is HTMLElement => t instanceof HTMLElement && t.matches("button, a[href], [role='button'][tabindex]");
const native = (el: HTMLElement) => el.matches("button, a[href]");
const FOCUSABLE = "button:not(:disabled), a[href], input:not(:disabled), select:not(:disabled), textarea:not(:disabled), [tabindex]:not([tabindex='-1'])";
const shown = (el: HTMLElement) => el.getClientRects().length > 0;

// Tab stays inside the top sheet (#1064): the screen under it is not reachable
// while it is open, and Tab wraps from its last control to its first.
addEventListener(
  "keydown",
  (e) => {
    if (e.key !== "Tab" || e.ctrlKey || e.metaKey || e.altKey) return;
    const top = topLayer();
    if (!top) return;
    const els = [...top.el.querySelectorAll<HTMLElement>(FOCUSABLE)].filter(shown);
    e.preventDefault();
    if (els.length === 0) return;
    const at = els.indexOf(document.activeElement as HTMLElement);
    const to = e.shiftKey ? (at <= 0 ? els.length - 1 : at - 1) : at < 0 || at === els.length - 1 ? 0 : at + 1;
    els[to]!.focus();
  },
  { capture: true },
);

addEventListener("keydown", (e) => {
  if (e.ctrlKey || e.metaKey || e.altKey || e.repeat) return;
  if (textField(e.target)) return;
  // M: sound on/off, on every screen (round 3, note 16).
  if (keyOf(e) === "m") return void (toggleSound(), e.preventDefault());
  // Enter on a focused button, link or card presses it, before the screen's
  // own Enter (Continue and the like): the key does what the focus says.
  if (e.key === "Enter" && pressable(e.target)) {
    if (!native(e.target)) (e.preventDefault(), e.target.click());
    return;
  }
  // Space on a button reached by Tab presses that button (what the focus ring
  // says); after a mouse click the focus is not visible, and Space is the
  // screen's (Fight in the shop).
  if (e.key === " " && pressable(e.target) && native(e.target) && e.target.matches(":focus-visible")) return;
  // A sheet is open: the screen under it takes no keys.
  if (topLayer()) return;
  if (keys && keys(e)) return void e.preventDefault();
  // Space on a card made actionable() presses it too (a button does by itself).
  if (e.key === " " && pressable(e.target) && !native(e.target)) return void (e.preventDefault(), e.target.click());
});

// ---------- overlays ----------

/** The open overlays, bottom first. Each one's `dismiss` is what a tap
 * outside it and Esc do: close it, or for the run menu, Resume. */
type Layer = { el: HTMLElement; dismiss: () => void; restore?: () => void };
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
    const wasOn = back.contains(document.activeElement);
    back.remove();
    const i = layers.indexOf(layer);
    if (i >= 0) layers.splice(i, 1);
    if (wasOn) layer.restore?.();
  }
  back.addEventListener("click", (e) => e.target === back && layer.dismiss());
  topLayer();
  layers.push(layer);
  // The keyboard goes into the sheet (its primary button, else its first
  // control) and comes back to what it was on when the sheet closes (#1064).
  const from = document.activeElement instanceof HTMLElement ? document.activeElement : null;
  const fromKey = from?.dataset.testid ?? "";
  app.append(back);
  queueMicrotask(() => (sheet.querySelector<HTMLElement>("button.primary:not(:disabled)") ?? sheet.querySelector<HTMLElement>(FOCUSABLE))?.focus({ preventScroll: true }));
  layer.restore = () => {
    const to = from?.isConnected ? from : fromKey ? [...app.querySelectorAll<HTMLElement>("[data-testid]")].find((e) => e.dataset.testid === fromKey && !e.closest(".overlay")) : undefined;
    to?.focus({ preventScroll: true });
  };
  return close;
}

/** overlay() with a Close button pinned to the sheet's bottom, for a sheet
 * with nothing else to tap (a unit sheet, the rules, a why-I-lost row). */
export function closable(...kids: Node[]): () => void {
  const closeBtn = withKey(button(t("common.close"), () => close(), "grow", "sheet-close"), "Esc");
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
  // Read every width first, in one layout, then write (#913): measuring each
  // line after the last one's write forced a layout per name, every redraw.
  const els = [...root.querySelectorAll<HTMLElement>(FIT)].map((el) => ({ el, w: el.isConnected ? el.clientWidth : 0 }));
  const misses: { el: HTMLElement; key: string }[] = [];
  for (const { el, w } of els) {
    if (w === 0) continue;
    // A line fitted before at this width with this text and font fits the
    // same: a battle redraws the same names every wave.
    const key = `${el.className}|${el.style.fontSize}|${w}|${el.closest(".bv-card, .card")?.className ?? ""}|${el.textContent}`;
    const hit = fitted.get(key);
    if (hit) {
      el.style.fontSize = hit.px;
      el.classList.toggle("clip", hit.clip);
    } else misses.push({ el, key });
  }
  for (const { el, key } of misses) {
    el.style.fontSize = "";
    el.classList.remove("clip");
    let px = parseFloat(getComputedStyle(el).fontSize);
    const min = !el.classList.contains("name") ? MIN_PX : el.closest(".bv-card") ? NAME_MIN_PX : CARD_NAME_MIN_PX;
    while (el.scrollWidth > el.clientWidth + 0.5 && px > min) {
      px = Math.max(min, px - 0.5);
      el.style.fontSize = `${px}px`;
    }
    if (el.scrollWidth > el.clientWidth + 0.5) el.classList.add("clip");
    if (fitted.size > 2000) fitted.clear();
    fitted.set(key, { px: el.style.fontSize, clip: el.classList.contains("clip") });
  }
}
/** Each fitted line's size, by its text, classes, card and width (#913). */
const fitted = new Map<string, { px: string; clip: boolean }>();
// A font arriving changes every measure.
const refit = () => fitted.clear();

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
addEventListener("resize", () => { refit(); queueFit(); });
void document.fonts?.ready.then(() => { refit(); queueFit(); });
// A name measured in a fallback font is cut wrongly: measure again once a font arrives.
document.fonts?.addEventListener?.("loadingdone", () => { refit(); queueFit(); });
