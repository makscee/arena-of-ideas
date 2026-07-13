import { primaryAbilityIdOf, statusActionsOf, unitActionsOf } from "../src/types.js";
// Codex screen — fully generated from registry + tunables via buildCodex().
// Renders statuses, units, and rule entries as responsive card grids (#015
// slice 2): units AND statuses now wear the ONE shared card (unit-card.ts) at
// the same fixed size (#078 — a Status is the same shape as a Unit, #074); the
// old codex-local status lookalike is gone. Rules stay a prose card.
// Presentation only — every sentence comes from buildCodex()/describe output.
// Supports deep-link anchors of the form #codex/status/<name>,
// #codex/unit/<name>, #codex/rule/<key>.

import { buildCodex } from "../src/codex.js";
import { abilityChips, describeAbilitySegments, describeStatusSegments } from "../src/describe.js";
import type { DescribeSegment } from "../src/describe.js";
import type { Ability, AbilityDef, AbilityRegistry, Family, StatusDef, StatusRegistry, UnitDef } from "../src/types.js";
import { openInspectOverlay, renderEntityInspect, type InspectEntityKind } from "./inspect.js";
import { unitCardHtml } from "./unit-card.js";

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

function abilityLine(ab: AbilityDef | undefined): {
  abilityLabel?: string | undefined;
  trigger?: string | undefined;
  triggerGlyph?: string | undefined;
  target?: string | undefined;
  action?: string | undefined;
} {
  return ab === undefined ? {} : { abilityLabel: ab.name, ...abilityChips(ab) };
}

/** A derived behavior sentence → HTML where every term links to its codex
 * card (#078 slice 3): a status name links to its Status card, every Part term
 * (trigger/interceptor/condition/selector/effect) to its Part card. statusRef
 * wins when a term is both (an applyStatus name) — the status card is the
 * closer answer. In the codex the links are real anchors (the global #codex/
 * handler in main.ts navigates within the open codex); inside the inspector
 * statusRef reveals in-panel instead, so the two renderers differ on status. */
function segmentLinksHtml(segs: DescribeSegment[], registry: StatusRegistry): string {
  return segs
    .map((s) => {
      if (s.statusRef !== undefined && registry[s.statusRef] !== undefined)
        return `<button type="button" class="entity-ref" data-entity-ref="status:${esc(s.statusRef)}">${esc(s.text)}</button>`;
      if (s.partRef !== undefined)
        return `<a class="codex-termref" href="#codex/part/${esc(s.partRef.family)}/${esc(s.partRef.kind)}" data-part="${esc(s.partRef.family)}:${esc(s.partRef.kind)}">${esc(s.text)}</a>`;
      return esc(s.text);
    })
    .join("");
}

export function createCodex(
  container: HTMLElement,
  registry: StatusRegistry,
  units: UnitDef[],
  abilities: AbilityRegistry,
): CodexScreen {
  const data = buildCodex(registry, units, abilities);
  // The raw def behind each derived entry (first occurrence wins — the same
  // dedup buildCodex applies): the unit card needs the statuses' stacks and
  // the level, which the derived entry doesn't carry.
  const defByName = new Map<string, UnitDef>();
  for (const u of units) if (!defByName.has(u.name)) defByName.set(u.name, u);
  const summonByName = new Map<string, { unit: UnitDef; source: string }>();
  for (const ability of Object.values(abilities)) {
    for (const effect of ability.effects) {
      if (effect.kind === "summon" && !summonByName.has(effect.unit.name)) summonByName.set(effect.unit.name, { unit: effect.unit, source: ability.name });
    }
  }

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

  // Section: statuses — the SAME shared card as units (#078): a Status is the
  // same shape as a Unit (a bundle of Parts, #074), so it wears the same card at
  // the same fixed size, framing its per-stack statMods where a unit frames
  // hp/pwr. The kernel-derived sentence rides below, exactly as a unit's
  // abilities do. No codex-local lookalike anymore.
  const statusCards = data.statuses.map((s) => {
    const card = unitCardHtml({
      surface: "codex/reference",
      kind: "status",
      artName: s.name,
      label: s.name,
      hp: s.hp,
      pwr: s.pwr,
      registry,
      variant: "reference",
      tag: "stacks",
      abilityLabel: "Status",
      action: s.description,
      classes: "codex-unit",
      attrs: `data-inspect-entity="status:${esc(s.name)}"`,
      title: s.name,
    });
    // The description's behavior sentence links every term to its codex card
    // (#078 slice 3): a referenced status to its Status card, every Part term to
    // its Part card. Derived from the registry def's segments; the plain string
    // still feeds search.
    const sdef: StatusDef | undefined = registry[s.name];
    const descHtml = sdef !== undefined ? segmentLinksHtml(describeStatusSegments(sdef), registry) : esc(s.description);
    return (
      `<div class="codex-entry codex-status-entry" id="codex-status-${encodeId(s.name)}"` +
      ` data-search="${esc(`${s.name} ${s.description}`.toLowerCase())}">` +
      anchorHtml(`codex/status/${s.name}`) +
      card +
      `<div class="codex-entry-desc">${descHtml}</div></div>`
    );
  });
  container.append(sectionEl("statuses", "Statuses", grid(statusCards, "codex-grid-statuses")));

  // Section: abilities — the Ability catalogue (PRD #081), beside Statuses. Each
  // shipped ability lists with its family swatch (the colour axis, hex from the
  // one palette) and its kernel-derived description. This is the source #080's
  // card and #082/#083's displays read colour + ability-line from.
  const abilityCards = data.abilities.map((a) => {
    const card = unitCardHtml({
      surface: "codex/reference",
      kind: "ability",
      artName: a.name,
      label: a.name,
      hp: "",
      pwr: "",
      registry,
      family: a.family,
      variant: "reference",
      tag: a.family,
      action: a.description,
      classes: "codex-unit",
      attrs: `data-inspect-entity="ability:${esc(a.name)}"`,
      title: `${a.name} — Ability`,
    });
    return (
      `<div class="codex-entry codex-ability-entry" id="codex-ability-${encodeId(a.name)}"` +
      ` data-search="${esc(`${a.name} ${a.family} ${a.description}`.toLowerCase())}">` +
      anchorHtml(`codex/ability/${a.name}`) + card +
      `<div class="codex-entry-desc">${esc(a.description)}</div></div>`
    );
  });
  container.append(sectionEl("abilities", "Abilities", grid(abilityCards, "codex-grid-abilities")));

  // Section: units — the shared card (art, framed stats, chips) over the
  // derived ability sentences; credit line for approved creation-loop units.
  const unitCards = data.units.filter((u) => !summonByName.has(u.name)).map((u) => {
    const def = defByName.get(u.name);
    const level = def?.level ?? 1;
    const search = [u.name, `${u.hp}hp`, `${u.pwr}pwr`, ...u.abilities, ...u.statuses].join(" ").toLowerCase();
    const card = unitCardHtml({
      surface: "codex/reference",
      artName: u.name,
      label: u.name,
      hp: u.hp,
      pwr: u.pwr,
      registry,
      statuses: def?.statuses,
      family: u.family,
      variant: "reference",
      ...abilityLine(abilities[def !== undefined ? primaryAbilityIdOf(def)! : u.ability]),
      ...(level > 1 ? { level } : {}),
      classes: "codex-unit",
      attrs: `data-inspect-entity="unit:${esc(u.name)}"`,
      title: u.name,
    });
    // Render from the def's abilities as segments so every term is a tappable
    // codex link (#078 slice 3); the derived u.abilities strings still feed
    // search. A unit not in defByName (shouldn't happen) falls back to plain.
    const abilityDefs: Ability[] = def === undefined ? [] : unitActionsOf(def, abilities);
    const abilitiesHtml =
      abilityDefs.length > 0
        ? abilityDefs
            .map((ab) => `<div class="codex-entry-desc">${segmentLinksHtml(describeAbilitySegments(ab), registry)}</div>`)
            .join("")
        : u.abilities.length > 0
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
      abilitiesHtml +
      credit +
      `</div>`
    );
  });
  container.append(sectionEl("units", "Units", grid(unitCards, "codex-grid-units")));

  // Summons are first-class entities, not a subtype hidden in the Unit grid.
  const summonCards = [...summonByName.values()].map(({ unit, source }) => {
    const ability = abilities[primaryAbilityIdOf(unit)!];
    const card = unitCardHtml({
      surface: "codex/reference",
      kind: "summon",
      artName: unit.name,
      label: unit.name,
      hp: unit.base.hp,
      pwr: unit.base.pwr,
      registry,
      family: ability?.family ?? "Summon",
      variant: "reference",
      tag: `from ${source}`,
      abilityLabel: source,
      ...(ability !== undefined ? abilityLine(ability) : {}),
      classes: "codex-unit",
      attrs: `data-inspect-entity="summon:${esc(unit.name)}"`,
      title: `${unit.name} — Summon from ${source}`,
    });
    return `<div class="codex-entry codex-summon-entry" id="codex-summon-${encodeId(unit.name)}" data-search="${esc(`${unit.name} summon ${source}`.toLowerCase())}">${anchorHtml(`codex/summon/${unit.name}`)}${card}<div class="codex-entry-desc">Created by ${esc(source)}.</div></div>`;
  });
  container.append(sectionEl("summons", "Summons", grid(summonCards, "codex-grid-summons")));

  // Trigger, Selector, Condition, Interceptor and low-level Effect are glossary
  // rows. They are intentionally not card-bearing entities.
  const FAMILY_LABELS: Record<string, string> = { trigger: "Trigger", interceptor: "Interceptor", condition: "Condition", selector: "Selector", effect: "Effect" };
  const partRows = data.parts.map((p) => {
    const id = `codex-part-${encodeId(p.family)}-${encodeId(p.kind)}`;
    const fragment = `codex/part/${p.family}/${p.kind}`;
    const search = `${p.name} ${p.family} ${p.meaning}`.toLowerCase();
    return `<div class="codex-entry codex-part-row" id="${id}" data-non-card="${esc(p.family)}" data-search="${esc(search)}">${anchorHtml(fragment)}<span class="codex-part-kind">${esc(FAMILY_LABELS[p.family] ?? p.family)}</span><b>${esc(p.name)}</b><span>${esc(p.meaning)}</span></div>`;
  });
  container.append(sectionEl("parts", "Grammar · non-card", `<div class="codex-part-list">${partRows.join("")}</div>`));

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

  // Every card and entity-reference chip opens the one app-wide inspector.
  container.addEventListener("click", (event) => {
    const target = event.target as HTMLElement;
    const hit = target.closest<HTMLElement>("[data-inspect-entity],[data-entity-ref]");
    if (hit === null) return;
    const raw = hit.dataset.inspectEntity ?? hit.dataset.entityRef;
    if (raw === undefined) return;
    event.preventDefault();
    const split = raw.indexOf(":");
    const kind = raw.slice(0, split) as InspectEntityKind;
    const name = raw.slice(split + 1);
    const unit = defByName.get(name);
    const ability = abilities[name];
    const status = registry[name];
    const summon = summonByName.get(name);
    const summary =
      kind === "unit" && unit !== undefined ? `${unit.base.pwr} PWR · ${unit.base.hp} HP` :
      kind === "ability" && ability !== undefined ? data.abilities.find((a) => a.name === name)?.description ?? name :
      kind === "status" && status !== undefined ? data.statuses.find((s) => s.name === name)?.description ?? name :
      kind === "summon" && summon !== undefined ? `${summon.unit.base.pwr} PWR · ${summon.unit.base.hp} HP · created by ${summon.source}` : name;
    const recipeUnit = kind === "summon" ? summon?.unit : unit;
    const boundUnitAbility = recipeUnit !== undefined ? unitActionsOf(recipeUnit, abilities)[0] : undefined;
    const unitChips = boundUnitAbility !== undefined ? abilityChips(boundUnitAbility) : undefined;
    const boundStatusAbility = status !== undefined ? statusActionsOf(status)[0] : undefined;
    const statusChips = boundStatusAbility !== undefined ? abilityChips(boundStatusAbility) : undefined;
    openInspectOverlay("entity", {
      anchor: hit.closest<HTMLElement>("[data-card-entity]") ?? hit,
      render: (body) => renderEntityInspect(body, {
        kind,
        name,
        summary,
        registry,
        ...(kind === "ability" && ability !== undefined ? {
          family: ability.family,
          tag: ability.family,
          action: data.abilities.find((a) => a.name === name)?.description ?? name,
        } : {}),
        ...(kind === "status" && status !== undefined ? {
          family: undefined,
          tag: `stacks${status.statMods?.pwr ? ` · ${status.statMods.pwr > 0 ? "+" : ""}${status.statMods.pwr} PWR/stack` : ""}${status.statMods?.hp ? ` · ${status.statMods.hp > 0 ? "+" : ""}${status.statMods.hp} HP/stack` : ""}`,
          trigger: statusChips?.trigger,
          target: statusChips?.target,
          action: statusChips?.action,
        } : {}),
        ...((kind === "unit" || kind === "summon") && recipeUnit !== undefined ? {
          hp: recipeUnit.base.hp,
          pwr: recipeUnit.base.pwr,
          statuses: recipeUnit.statuses,
          family: abilities[primaryAbilityIdOf(recipeUnit)!]?.family,
          trigger: unitChips?.trigger,
          triggerGlyph: unitChips?.triggerGlyph,
          target: unitChips?.target,
          action: unitChips?.action ?? primaryAbilityIdOf(recipeUnit) ?? "Strike",
          ...(kind === "summon" && summon !== undefined ? { tag: `from ${summon.source}` } : {}),
        } : {}),
      }),
      onClose: () => {},
    });
  });

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
      // fragment: "codex/status/Poison", "codex/rule/fatigue", or a part's
      // "codex/part/<family>/<kind>" (a Part keys on family AND kind, #078).
      const parts = fragment.split("/");
      if (parts.length < 3) return;
      const [, kind, ...rest] = parts as [string, string, ...string[]];
      const key = rest.map(encodeId).join("-");
      const id = `codex-${kind}-${key}`;
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
