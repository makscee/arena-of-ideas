// Codex screen — fully generated from registry + tunables via buildCodex().
// Renders statuses, units, and rule entries as responsive card grids (#015
// slice 2): units wear the one shared unit card (unit-card.ts), statuses get
// a matching card with a hash-stable colour identity, rules a prose card.
// Presentation only — every sentence comes from buildCodex()/describe output.
// Supports deep-link anchors of the form #codex/status/<name>,
// #codex/unit/<name>, #codex/rule/<key>.

import { buildCodex } from "../src/codex.js";
import type { StatusRegistry, UnitDef } from "../src/types.js";
import { nameHue, unitCardHtml } from "./unit-card.js";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export interface CodexScreen {
  /** Show or hide the codex panel. */
  setVisible(visible: boolean): void;
  /** Navigate to a deep-link fragment (e.g. "codex/status/Poison"). */
  navigate(fragment: string): void;
}

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

export function createCodex(
  container: HTMLElement,
  registry: StatusRegistry,
  units: UnitDef[],
): CodexScreen {
  const data = buildCodex(registry, units);
  // The raw def behind each derived entry (first occurrence wins — the same
  // dedup buildCodex applies): the unit card needs the statuses' stacks and
  // the level, which the derived entry doesn't carry.
  const defByName = new Map<string, UnitDef>();
  for (const u of units) if (!defByName.has(u.name)) defByName.set(u.name, u);

  // ---- shell ---------------------------------------------------------------
  container.innerHTML = "";
  container.className = "codex-panel";

  // Search input
  const searchRow = document.createElement("div");
  searchRow.className = "codex-search-row";
  const searchInput = document.createElement("input");
  searchInput.type = "search";
  searchInput.placeholder = "filter…";
  searchInput.className = "codex-search";
  searchInput.setAttribute("aria-label", "Filter codex entries");
  searchRow.append(searchInput);
  container.append(searchRow);

  // Section: statuses — name + colour identity + the kernel-derived sentence.
  const statusCards = data.statuses.map((s) => {
    const hue = nameHue(s.name);
    return (
      `<div class="codex-entry codex-status-entry" id="codex-status-${encodeId(s.name)}"` +
      ` data-search="${esc(`${s.name} ${s.description}`.toLowerCase())}" style="--codex-hue: ${hue.toFixed(0)}">` +
      anchorHtml(`codex/status/${s.name}`) +
      `<div class="codex-entry-head"><span class="codex-swatch" aria-hidden="true"></span>` +
      `<span class="codex-entry-name">${esc(s.name)}</span></div>` +
      `<div class="codex-entry-desc">${esc(s.description)}</div></div>`
    );
  });
  container.append(sectionEl("statuses", "Statuses", grid(statusCards, "codex-grid-statuses")));

  // Section: units — the shared card (art, framed stats, chips) over the
  // derived ability sentences; credit line for approved creation-loop units.
  const unitCards = data.units.map((u) => {
    const def = defByName.get(u.name);
    const level = def?.level ?? 1;
    const search = [u.name, `${u.hp}hp`, `${u.pwr}pwr`, ...u.abilities, ...u.statuses].join(" ").toLowerCase();
    const card = unitCardHtml({
      artName: u.name,
      label: u.name,
      hp: u.hp,
      pwr: u.pwr,
      registry,
      statuses: def?.statuses,
      ...(level > 1 ? { level } : {}),
      classes: "codex-unit",
      attrs: "",
      title: u.name,
    });
    const abilities =
      u.abilities.length > 0
        ? u.abilities.map((ab) => `<div class="codex-entry-desc">${esc(ab)}</div>`).join("")
        : `<div class="codex-entry-desc codex-dim">No abilities.</div>`;
    // Authorship credit for an approved creation-loop unit (PRD #013 slice 4).
    const credit =
      u.creator !== undefined
        ? `<div class="codex-entry-credit codex-dim" data-creator="${esc(u.creator)}">made by ${esc(u.creator)}</div>`
        : "";
    return (
      `<div class="codex-entry codex-unit-entry" id="codex-unit-${encodeId(u.name)}" data-search="${esc(search)}">` +
      anchorHtml(`codex/unit/${u.name}`) +
      card +
      abilities +
      credit +
      `</div>`
    );
  });
  container.append(sectionEl("units", "Units", grid(unitCards, "codex-grid-units")));

  // Section: rules — prose cards.
  const ruleCards = data.rules.map(
    (r) =>
      `<div class="codex-entry codex-rule-entry" id="codex-rule-${encodeId(r.key)}"` +
      ` data-search="${esc(`${r.title} ${r.text}`.toLowerCase())}">` +
      anchorHtml(`codex/rule/${r.key}`) +
      `<div class="codex-entry-head"><span class="codex-entry-name">${esc(r.title)}</span></div>` +
      `<div class="codex-entry-desc">${esc(r.text)}</div></div>`,
  );
  container.append(sectionEl("rules", "Rules", grid(ruleCards, "codex-grid-rules")));

  // ---- search filter -------------------------------------------------------
  searchInput.addEventListener("input", () => {
    const q = searchInput.value.toLowerCase().trim();
    for (const entry of container.querySelectorAll<HTMLElement>(".codex-entry")) {
      const match = q === "" || (entry.dataset.search ?? "").includes(q);
      entry.hidden = !match;
    }
    // Show/hide section headers based on whether any children are visible
    for (const section of container.querySelectorAll<HTMLElement>(".codex-section")) {
      const hasVisible = [...section.querySelectorAll<HTMLElement>(".codex-entry")].some((e) => !e.hidden);
      section.hidden = !hasVisible;
    }
  });

  // ---- public interface ----------------------------------------------------
  return {
    setVisible(visible: boolean) {
      container.hidden = !visible;
    },
    navigate(fragment: string) {
      // fragment: "codex/status/Poison" or "codex/rule/fatigue" etc.
      const parts = fragment.split("/");
      if (parts.length < 3) return;
      const [, kind, key] = parts as [string, string, string];
      const id = `codex-${kind}-${encodeId(key)}`;
      const target = container.querySelector<HTMLElement>(`#${CSS.escape(id)}`);
      if (target) {
        container.hidden = false;
        // If a search filter is hiding the target, clear it so the card is
        // actually visible — the user asked for this specific card.
        if (target.hidden) {
          searchInput.value = "";
          searchInput.dispatchEvent(new Event("input"));
        }
        target.scrollIntoView({ block: "center", behavior: "smooth" });
        target.classList.add("codex-highlight");
        setTimeout(() => target.classList.remove("codex-highlight"), 1800);
      }
    },
  };
}

// ---------------------------------------------------------------------------
// Helpers
// ---------------------------------------------------------------------------

function grid(cards: string[], cls: string): string {
  return `<div class="codex-grid ${cls}">${cards.join("")}</div>`;
}

function sectionEl(id: string, title: string, bodyHtml: string): HTMLElement {
  const section = document.createElement("div");
  section.className = "codex-section";
  section.id = `codex-sec-${id}`;
  section.innerHTML = `<div class="pane-k codex-section-head">${esc(title)}</div>${bodyHtml}`;
  return section;
}

function anchorHtml(fragment: string): string {
  return `<a href="#${esc(fragment)}" class="codex-anchor" title="Deep link: #${esc(fragment)}">#</a>`;
}

function encodeId(name: string): string {
  return name.replace(/[^a-zA-Z0-9_-]/g, "_");
}
