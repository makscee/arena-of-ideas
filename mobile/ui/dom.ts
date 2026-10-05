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
