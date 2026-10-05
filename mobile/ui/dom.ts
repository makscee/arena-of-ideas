// Plain-DOM helpers shared by every screen of the phone client (mission #574).

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

export function button(label: string, onClick: () => void, cls = "", testid = ""): HTMLButtonElement {
  const b = h("button", { class: cls, ...(testid ? { "data-testid": testid } : {}) }, label);
  b.addEventListener("click", onClick);
  return b;
}

/** Replaces the screen; null kids are skipped. */
export function show(...kids: (Node | null)[]): void {
  app.replaceChildren(...kids.filter((k): k is Node => k !== null));
  window.scrollTo(0, 0);
}

/** A modal sheet over the current screen: the unit sheet, the rules, the
 * trace popup, why-I-lost. Tapping outside it closes it, and so does the next
 * show(). Returns close. */
export function overlay(...kids: Node[]): () => void {
  const sheet = h("div", { class: "sheet stack", role: "dialog" }, ...kids);
  const back = h("div", { class: "overlay", "data-testid": "overlay" }, sheet);
  const close = () => back.remove();
  back.addEventListener("click", (e) => e.target === back && close());
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

const FIT = ".card .name, .card .rates";
const MIN_PX = 9;

/** Shrinks each card name (and rates line) until its longest word fits the
 * card, measured, not guessed from letter counts; a word that still doesn't
 * fit at the smallest size ends in an ellipsis. Names break only between words. */
export function fitText(root: ParentNode = app): void {
  for (const el of root.querySelectorAll<HTMLElement>(FIT)) {
    if (!el.isConnected || el.clientWidth === 0) continue;
    el.style.fontSize = "";
    el.classList.remove("clip");
    let px = parseFloat(getComputedStyle(el).fontSize);
    while (el.scrollWidth > el.clientWidth + 0.5 && px > MIN_PX) {
      px = Math.max(MIN_PX, px - 0.5);
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
