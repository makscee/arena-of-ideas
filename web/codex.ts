// Codex screen — fully generated from registry + tunables via buildCodex().
// Renders statuses, units, and rule entries; supports deep-link anchors of
// the form #codex/status/<name>, #codex/unit/<name>, #codex/rule/<key>.

import { buildCodex } from "../src/codex.js";
import type { StatusRegistry, UnitDef } from "../src/types.js";

// ---------------------------------------------------------------------------
// Public API
// ---------------------------------------------------------------------------

export interface CodexScreen {
  /** Show or hide the codex panel. */
  setVisible(visible: boolean): void;
  /** Navigate to a deep-link fragment (e.g. "codex/status/Poison"). */
  navigate(fragment: string): void;
}

export function createCodex(
  container: HTMLElement,
  registry: StatusRegistry,
  units: UnitDef[],
): CodexScreen {
  const data = buildCodex(registry, units);

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

  // Section: statuses
  const statusSection = buildSection("statuses", "Statuses");
  const statusList = document.createElement("div");
  statusList.className = "codex-list";
  for (const s of data.statuses) {
    const card = document.createElement("div");
    card.className = "codex-entry";
    card.id = `codex-status-${encodeId(s.name)}`;
    card.dataset.search = s.name.toLowerCase() + " " + s.description.toLowerCase();

    const heading = document.createElement("div");
    heading.className = "codex-entry-head";
    const nameEl = document.createElement("span");
    nameEl.className = "codex-entry-name";
    nameEl.textContent = s.name;
    const anchor = anchorLink(`codex/status/${s.name}`);
    heading.append(nameEl, anchor);

    const desc = document.createElement("div");
    desc.className = "codex-entry-desc";
    desc.textContent = s.description;

    card.append(heading, desc);
    statusList.append(card);
  }
  statusSection.append(statusList);
  container.append(statusSection);

  // Section: units
  const unitSection = buildSection("units", "Units");
  const unitList = document.createElement("div");
  unitList.className = "codex-list";
  for (const u of data.units) {
    const card = document.createElement("div");
    card.className = "codex-entry";
    card.id = `codex-unit-${encodeId(u.name)}`;
    const searchText = [u.name, `${u.hp}hp`, `${u.pwr}pwr`, ...u.abilities, ...u.statuses].join(" ").toLowerCase();
    card.dataset.search = searchText;

    const heading = document.createElement("div");
    heading.className = "codex-entry-head";
    const nameEl = document.createElement("span");
    nameEl.className = "codex-entry-name";
    nameEl.textContent = u.name;
    const stats = document.createElement("span");
    stats.className = "codex-entry-stats";
    stats.textContent =
      `${u.hp} hp · ${u.pwr} pwr` + (u.statuses.length > 0 ? ` · starts with ${u.statuses.join(", ")}` : "");
    const anchor = anchorLink(`codex/unit/${u.name}`);
    heading.append(nameEl, stats, anchor);

    card.append(heading);
    for (const ab of u.abilities) {
      const row = document.createElement("div");
      row.className = "codex-entry-desc";
      row.textContent = ab;
      card.append(row);
    }
    if (u.abilities.length === 0) {
      const row = document.createElement("div");
      row.className = "codex-entry-desc codex-dim";
      row.textContent = "No abilities.";
      card.append(row);
    }
    // Authorship credit for an approved creation-loop unit (PRD #013 slice 4).
    if (u.creator !== undefined) {
      const credit = document.createElement("div");
      credit.className = "codex-entry-credit codex-dim";
      credit.dataset.creator = u.creator;
      credit.textContent = `made by ${u.creator}`;
      card.append(credit);
    }

    unitList.append(card);
  }
  unitSection.append(unitList);
  container.append(unitSection);

  // Section: rules
  const rulesSection = buildSection("rules", "Rules");
  const ruleList = document.createElement("div");
  ruleList.className = "codex-list";
  for (const r of data.rules) {
    const card = document.createElement("div");
    card.className = "codex-entry";
    card.id = `codex-rule-${encodeId(r.key)}`;
    card.dataset.search = (r.title + " " + r.text).toLowerCase();

    const heading = document.createElement("div");
    heading.className = "codex-entry-head";
    const titleEl = document.createElement("span");
    titleEl.className = "codex-entry-name";
    titleEl.textContent = r.title;
    const anchor = anchorLink(`codex/rule/${r.key}`);
    heading.append(titleEl, anchor);

    const desc = document.createElement("div");
    desc.className = "codex-entry-desc";
    desc.textContent = r.text;

    card.append(heading, desc);
    ruleList.append(card);
  }
  rulesSection.append(ruleList);
  container.append(rulesSection);

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

function buildSection(id: string, title: string): HTMLElement {
  const section = document.createElement("div");
  section.className = "codex-section";
  section.id = `codex-sec-${id}`;
  const heading = document.createElement("div");
  heading.className = "pane-k codex-section-head";
  heading.textContent = title;
  section.append(heading);
  return section;
}

function anchorLink(fragment: string): HTMLAnchorElement {
  const a = document.createElement("a");
  a.href = `#${fragment}`;
  a.className = "codex-anchor";
  a.textContent = "#";
  a.title = `Deep link: #${fragment}`;
  return a;
}

function encodeId(name: string): string {
  return name.replace(/[^a-zA-Z0-9_-]/g, "_");
}
