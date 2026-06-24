// Unit palette (#066 slice 2) — a reusable picker over a unit pool, rendered as
// a popover anchored to the control that opened it. It owns zero domain rules:
// the caller hands it the pool (shipped + approved + codex, whatever it wants)
// and a registry for the chips, and gets back a single callback per pick. Built
// decoupled on purpose — slice 4's spawn-any-unit reuses this exact component by
// passing a different pool and a different onPick, no editor knowledge baked in.
//
// The picked def is deep-cloned before it leaves: a placed/spawned unit must
// never alias the pool entry (an in-place stat edit would otherwise mutate the
// shared pool object and desync everything that holds it by reference).

import type { StatusRegistry, UnitDef } from "../src/index.js";
import { unitCardHtml } from "./unit-card.js";

const clone = <T>(v: T): T => JSON.parse(JSON.stringify(v)) as T;

export interface PaletteDeps {
  /** The pool to pick from — read live via a function so the caller can grow it
   * (approved overrides, etc.) without rebuilding the palette. */
  pool: () => readonly UnitDef[];
  /** Registry for the cards' status chips. */
  registry: StatusRegistry;
  /** Fired once per pick with a fresh deep clone of the chosen def. */
  onPick: (def: UnitDef) => void;
}

export interface Palette {
  /** Open the popover, anchored under `anchor`. A second open re-anchors it. */
  open(anchor: HTMLElement): void;
  close(): void;
  /** The popover element — the caller appends it once into its own subtree. */
  readonly element: HTMLElement;
}

export function createPalette(deps: PaletteDeps): Palette {
  const root = document.createElement("div");
  root.className = "palette";
  root.hidden = true;
  root.setAttribute("role", "dialog");
  root.setAttribute("aria-label", "Pick a unit");

  const grid = document.createElement("div");
  grid.className = "palette-grid";
  root.append(grid);

  let anchor: HTMLElement | undefined;

  function render(): void {
    const pool = deps.pool();
    grid.innerHTML =
      pool.length === 0
        ? '<span class="palette-empty">the pool is empty</span>'
        : pool
            .map((def, i) =>
              unitCardHtml({
                artName: def.name,
                label: def.name,
                hp: def.base.hp,
                pwr: def.base.pwr,
                statuses: def.statuses,
                registry: deps.registry,
                classes: "palette-card",
                attrs: `data-pick="${i}"`,
                title: `${def.name} — ${def.base.hp} hp, ${def.base.pwr} pwr · place`,
              }),
            )
            .join("");
  }

  /** Position the popover just under its anchor, clamped into the viewport so a
   * right-edge control never opens it off-screen. position: fixed (set in CSS),
   * so the coords are viewport-relative and no parent scroll shifts it. */
  function place(): void {
    if (anchor === undefined) return;
    const a = anchor.getBoundingClientRect();
    root.style.visibility = "hidden";
    root.hidden = false;
    const w = root.offsetWidth;
    const h = root.offsetHeight;
    let left = a.left;
    if (left + w > window.innerWidth - 8) left = Math.max(8, window.innerWidth - 8 - w);
    let top = a.bottom + 4;
    if (top + h > window.innerHeight - 8) top = Math.max(8, a.top - 4 - h); // flip above if it would overflow
    root.style.left = `${left}px`;
    root.style.top = `${top}px`;
    root.style.visibility = "";
  }

  function open(next: HTMLElement): void {
    anchor = next;
    render();
    place();
  }

  function close(): void {
    root.hidden = true;
    anchor = undefined;
  }

  grid.addEventListener("click", (ev) => {
    const card = (ev.target as HTMLElement).closest<HTMLElement>("[data-pick]");
    if (card === null) return;
    const def = deps.pool()[Number(card.getAttribute("data-pick"))];
    if (def === undefined) return;
    deps.onPick(clone(def));
    close();
  });

  // Outside tap and Escape close it (the inspector/menu discipline) — never a
  // half-open popover floating over the page.
  document.addEventListener("click", (ev) => {
    if (root.hidden) return;
    const target = ev.target as Node;
    if (root.contains(target) || (anchor !== undefined && anchor.contains(target))) return;
    close();
  });
  document.addEventListener("keydown", (ev) => {
    if (ev.key === "Escape" && !root.hidden) close();
  });

  return {
    open,
    close,
    element: root,
  };
}
