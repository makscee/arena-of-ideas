/// <reference types="vite/client" />
// Icons for glossary terms (src/glossary.ts): the SVGs in ../icons/ (from
// game-icons.net, credited in ../icons/CREDITS.txt) go into the page as one
// hidden sprite, and icon(id) draws one with <use>. They are filled with
// currentColor, so a term's tone class colours them. No icon font, no fetch:
// the bundler inlines the files.
import type { IconId } from "../../src/glossary";

const files = import.meta.glob("../icons/*.svg", { query: "?raw", import: "default", eager: true }) as Record<string, string>;

/** One <symbol id="i-<id>"> per icon file, keeping its viewBox and paths. */
export function spriteMarkup(svgs: Record<string, string> = files): string {
  const symbols = Object.entries(svgs).map(([path, svg]) => {
    const id = path.slice(path.lastIndexOf("/") + 1, -".svg".length);
    const viewBox = /viewBox="([^"]+)"/.exec(svg)?.[1] ?? "0 0 512 512";
    const body = svg.replace(/^[\s\S]*?<svg[^>]*>/, "").replace(/<\/svg>\s*$/, "");
    return `<symbol id="i-${id}" viewBox="${viewBox}">${body}</symbol>`;
  });
  return `<svg xmlns="http://www.w3.org/2000/svg" aria-hidden="true" style="display:none">${symbols.join("")}</svg>`;
}

let installed = false;

/** Puts the sprite into the page once; icon() calls it, so nothing loads
 * until a screen draws its first icon. */
export function installIconSprite(): void {
  if (installed) return;
  installed = true;
  document.body.insertAdjacentHTML("afterbegin", spriteMarkup());
}

const SVG = "http://www.w3.org/2000/svg";

/** An icon as an inline <svg>, `size` px square (20 reads well; 16 is the
 * floor, for compact card lines). Decorative: the term's words carry the
 * meaning, so it is hidden from screen readers. */
export function icon(id: IconId, size = 20, cls = ""): SVGSVGElement {
  installIconSprite();
  const svg = document.createElementNS(SVG, "svg");
  svg.setAttribute("class", cls ? `ic ${cls}` : "ic");
  svg.setAttribute("width", String(size));
  svg.setAttribute("height", String(size));
  svg.setAttribute("aria-hidden", "true");
  const use = document.createElementNS(SVG, "use");
  use.setAttribute("href", `#i-${id}`);
  svg.append(use);
  return svg;
}
