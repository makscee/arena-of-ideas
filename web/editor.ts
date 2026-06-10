// Team editor — compose a team from the stress-set registry, live-validated.
// The client owns zero rules: unit/ability templates come off the kernel's
// public exports, validity comes off the kernel validator (real, path-addressed
// issues rendered inline), and persistence round-trips the CLI's team-file
// JSON exactly. What the team-file schema supports, this editor can express;
// it creates no new triggers or effects (that's the future creation pipeline).

import {
  Imp,
  Necromancer,
  Silencer,
  Summoner,
  TEAM_SIZE,
  Venomancer,
  stressRegistry,
  validateTeam,
  type Ability,
  type Amount,
  type Effect,
  type EventPattern,
  type UnitDef,
  type ValidationIssue,
} from "../src/index.js";
import { deleteTeam, loadTeams, parseTeamFileJson, saveTeam, toTeamFileJson } from "./teams.js";

// ---------------------------------------------------------------------------
// Templates — the shipped stress units as starting points, deep-cloned so
// edits never alias the kernel's exported defs.
// ---------------------------------------------------------------------------

const BLANK: UnitDef = { name: "Recruit", base: { hp: 5, pwr: 1 } };
const UNIT_TEMPLATES: UnitDef[] = [BLANK, Venomancer, Summoner, Silencer, Necromancer, Imp];

/** Attachable abilities: the ones the stress units ship with. */
const ABILITY_PALETTE: { label: string; ability: Ability }[] = [
  { label: "Venom strike (Venomancer)", ability: Venomancer.abilities![0]! },
  { label: "Summon Imp on death (Summoner)", ability: Summoner.abilities![0]! },
  { label: "Opening silence (Silencer)", ability: Silencer.abilities![0]! },
  { label: "Raise dead ally (Necromancer)", ability: Necromancer.abilities![0]! },
];

const clone = <T>(v: T): T => JSON.parse(JSON.stringify(v)) as T;

// ---------------------------------------------------------------------------
// Display-only formatting — a one-line gloss per ability so attached (or
// imported) abilities are readable. Tolerant of malformed data: the editor
// must still render while the validator is flagging it.
// ---------------------------------------------------------------------------

function amountDesc(a: Amount | undefined): string {
  if (!a || typeof a !== "object") return "?";
  if (a.kind === "const") return String(a.value);
  if (a.kind === "stat") return `holder ${a.stat}`;
  if (a.kind === "stacks") return "stacks";
  return "?";
}

function patternDesc(p: EventPattern | undefined): string {
  if (!p || typeof p !== "object") return "?";
  const extras = Object.entries(p)
    .filter(([k]) => k !== "on")
    .map(([k, v]) => `${k}: ${String(v)}`)
    .join(", ");
  return extras.length > 0 ? `${p.on} (${extras})` : String(p.on);
}

function effectDesc(e: Effect | undefined): string {
  if (!e || typeof e !== "object") return "?";
  switch (e.kind) {
    case "damage": return `damage ${amountDesc(e.amount)}`;
    case "heal": return `heal ${amountDesc(e.amount)}`;
    case "applyStatus": return `apply ${e.status} ×${amountDesc(e.stacks)}`;
    case "consumeStacks": return `consume ${e.status ?? "own"} ×${amountDesc(e.stacks)}`;
    case "summon": return `summon ${e.unit?.name ?? "?"}`;
    case "silence": return "silence";
    case "resurrect": return `resurrect at ${amountDesc(e.hp)} hp`;
    case "cancel": return "cancel";
    case "absorbHurt": return "absorb hurt";
    case "preventDeathHeal": return `prevent death, heal to ${amountDesc(e.toHp)}`;
    default: return String((e as { kind?: unknown }).kind ?? "?");
  }
}

function abilityDesc(ab: Ability): string {
  try {
    const whens = (ab.whens ?? []).map((w) => `${w.kind} on ${patternDesc(w.on)}`).join("; ");
    const sels = (ab.selectors ?? []).map((s) => s?.kind ?? "?").join(" + ");
    const effects = (ab.effects ?? []).map(effectDesc).join(", ");
    return `${whens} → ${effects} @ ${sels}`;
  } catch {
    return JSON.stringify(ab); // unreadably malformed — show the raw data
  }
}

// ---------------------------------------------------------------------------
// The editor
// ---------------------------------------------------------------------------

interface EditorEls {
  teamSelect: HTMLSelectElement;
  newButton: HTMLButtonElement;
  deleteButton: HTMLButtonElement;
  exportButton: HTMLButtonElement;
  importButton: HTMLButtonElement;
  fileInput: HTMLInputElement;
  nameInput: HTMLInputElement;
  unitsBox: HTMLElement;
  templateSelect: HTMLSelectElement;
  addButton: HTMLButtonElement;
  verdict: HTMLElement;
  issuesList: HTMLElement;
  importError: HTMLElement;
}

export interface Editor {
  /** Names of saved teams that currently pass the validator. */
  refresh(): void;
}

/** An empty number field must read as missing, never silently as 0. */
function numField(value: string): number {
  return value.trim() === "" ? Number.NaN : Number(value);
}

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

export function createEditor(els: EditorEls, onTeamsChanged: () => void): Editor {
  let teamName = "";
  let units: UnitDef[] = [];
  let issues: ValidationIssue[] = [];

  // ----- persistence -----

  function persist(): void {
    if (teamName.trim() === "") return; // a nameless draft lives only in the page
    saveTeam(teamName, units);
    refreshTeamSelect();
    onTeamsChanged();
  }

  function freshName(base: string): string {
    const taken = new Set(Object.keys(loadTeams()));
    if (!taken.has(base)) return base;
    for (let n = 2; ; n++) {
      if (!taken.has(`${base} ${n}`)) return `${base} ${n}`;
    }
  }

  function refreshTeamSelect(): void {
    const names = Object.keys(loadTeams()).sort();
    els.teamSelect.innerHTML = "";
    for (const name of names) {
      els.teamSelect.add(new Option(name, name, false, name === teamName));
    }
  }

  function loadByName(name: string): void {
    const saved = loadTeams()[name];
    if (!saved) return;
    teamName = name;
    units = clone(saved);
    els.nameInput.value = teamName;
    render();
  }

  // ----- validation: the kernel validator, on every change -----

  function validate(): void {
    issues = validateTeam(units, stressRegistry, "team");
    if (issues.length === 0) {
      els.verdict.textContent = `✓ valid — ${units.length} unit${units.length === 1 ? "" : "s"}, ready for battle`;
      els.verdict.className = "ok";
    } else {
      els.verdict.textContent = `✗ invalid — ${issues.length} issue${issues.length === 1 ? "" : "s"}; this team cannot battle until fixed`;
      els.verdict.className = "bad";
    }
    els.issuesList.innerHTML = issues
      .map((i) => `<li><code>${esc(i.path)}</code> ${esc(i.message)}</li>`)
      .join("");
    els.exportButton.disabled = issues.length > 0; // an exported file must run via the CLI
  }

  /** A field edit: update data + validate, no re-render (typing keeps focus). */
  function changed(): void {
    validate();
    persist();
  }

  /** A structural edit (add/remove/reorder/attach/detach): re-render too. */
  function structuralChange(): void {
    render();
    persist();
  }

  // ----- rendering -----

  function unitCard(u: UnitDef, i: number): string {
    const abilities = (u.abilities ?? [])
      .map(
        (ab, j) => `
          <div class="ed-row">
            <span class="ed-desc">${esc(abilityDesc(ab))}</span>
            <button type="button" data-act="detach-ability" data-i="${i}" data-j="${j}" title="Detach ability">✕</button>
          </div>`,
      )
      .join("");
    const statuses = (u.statuses ?? [])
      .map(
        (s, j) => `
          <div class="ed-row">
            <span class="ed-desc">${esc(String(s.status))}</span>
            <input type="number" value="${esc(String(s.stacks))}" data-field="stacks" data-i="${i}" data-j="${j}" title="Stacks" />
            <button type="button" data-act="detach-status" data-i="${i}" data-j="${j}" title="Detach status">✕</button>
          </div>`,
      )
      .join("");
    const statusOptions = Object.keys(stressRegistry)
      .map((name) => `<option value="${esc(name)}">${esc(name)}</option>`)
      .join("");
    const abilityOptions = ABILITY_PALETTE.map((p, j) => `<option value="${j}">${esc(p.label)}</option>`).join("");
    return `
      <div class="ed-unit" data-unit="${i}">
        <div class="ed-unit-head">
          <span class="ed-pos">${i + 1}${i === 0 ? " · front" : ""}</span>
          <input value="${esc(u.name ?? "")}" data-field="name" data-i="${i}" placeholder="name" />
          <button type="button" data-act="up" data-i="${i}" ${i === 0 ? "disabled" : ""} title="Move toward front">↑</button>
          <button type="button" data-act="down" data-i="${i}" ${i === units.length - 1 ? "disabled" : ""} title="Move toward back">↓</button>
          <button type="button" data-act="remove" data-i="${i}" title="Remove unit">✕</button>
        </div>
        <div class="ed-stats">
          <label>hp <input type="number" value="${esc(String(u.base?.hp ?? ""))}" data-field="hp" data-i="${i}" /></label>
          <label>pwr <input type="number" value="${esc(String(u.base?.pwr ?? ""))}" data-field="pwr" data-i="${i}" /></label>
        </div>
        <div class="ed-attach">
          <span class="ed-k">abilities</span>
          ${abilities || '<span class="ed-none">none</span>'}
          <div class="ed-row">
            <select data-pick="ability" data-i="${i}">${abilityOptions}</select>
            <button type="button" data-act="attach-ability" data-i="${i}">attach</button>
          </div>
        </div>
        <div class="ed-attach">
          <span class="ed-k">statuses</span>
          ${statuses || '<span class="ed-none">none</span>'}
          <div class="ed-row">
            <select data-pick="status" data-i="${i}">${statusOptions}</select>
            <button type="button" data-act="attach-status" data-i="${i}">attach</button>
          </div>
        </div>
      </div>`;
  }

  function render(): void {
    els.unitsBox.innerHTML = units.map(unitCard).join("");
    els.addButton.disabled = units.length >= TEAM_SIZE;
    validate();
  }

  // ----- events (delegated, so render can rebuild freely) -----

  els.unitsBox.addEventListener("input", (ev) => {
    const t = ev.target as HTMLInputElement;
    const field = t.getAttribute("data-field");
    if (!field) return;
    const u = units[Number(t.getAttribute("data-i"))];
    if (!u) return;
    if (field === "name") u.name = t.value;
    else if (field === "hp" || field === "pwr") {
      // An imported unit may lack a base object entirely (the validator flags
      // it) — editing a stat field must repair it, not throw.
      if (typeof u.base !== "object" || u.base === null) u.base = { hp: Number.NaN, pwr: Number.NaN };
      u.base[field] = numField(t.value);
    }
    else if (field === "stacks") {
      const s = u.statuses?.[Number(t.getAttribute("data-j"))];
      if (s) s.stacks = numField(t.value);
    }
    changed();
  });

  els.unitsBox.addEventListener("click", (ev) => {
    const btn = (ev.target as HTMLElement).closest("button[data-act]");
    if (!btn) return;
    const act = btn.getAttribute("data-act")!;
    const i = Number(btn.getAttribute("data-i"));
    const j = Number(btn.getAttribute("data-j"));
    const u = units[i];
    if (!u) return;
    if (act === "remove") units.splice(i, 1);
    else if (act === "up" && i > 0) [units[i - 1], units[i]] = [units[i]!, units[i - 1]!];
    else if (act === "down" && i < units.length - 1) [units[i], units[i + 1]] = [units[i + 1]!, units[i]!];
    else if (act === "detach-ability") u.abilities?.splice(j, 1);
    else if (act === "detach-status") u.statuses?.splice(j, 1);
    else if (act === "attach-ability") {
      const pick = els.unitsBox.querySelector<HTMLSelectElement>(`select[data-pick="ability"][data-i="${i}"]`);
      const tpl = ABILITY_PALETTE[Number(pick?.value ?? 0)];
      if (tpl) (u.abilities ??= []).push(clone(tpl.ability));
    } else if (act === "attach-status") {
      const pick = els.unitsBox.querySelector<HTMLSelectElement>(`select[data-pick="status"][data-i="${i}"]`);
      if (pick) (u.statuses ??= []).push({ status: pick.value, stacks: 1 });
    } else return;
    structuralChange();
  });

  els.addButton.addEventListener("click", () => {
    if (units.length >= TEAM_SIZE) return;
    const tpl = UNIT_TEMPLATES[Number(els.templateSelect.value)] ?? BLANK;
    units.push(clone(tpl));
    structuralChange();
  });

  // ----- team management -----

  els.newButton.addEventListener("click", () => {
    teamName = freshName("New Team");
    units = [clone(BLANK)];
    els.nameInput.value = teamName;
    structuralChange();
  });

  els.deleteButton.addEventListener("click", () => {
    if (teamName.trim() === "") return;
    deleteTeam(teamName);
    const rest = Object.keys(loadTeams()).sort();
    if (rest.length > 0) loadByName(rest[0]!);
    else {
      teamName = "";
      units = [];
      els.nameInput.value = "";
      render();
    }
    refreshTeamSelect();
    onTeamsChanged();
  });

  els.teamSelect.addEventListener("change", () => loadByName(els.teamSelect.value));

  // Renames commit on change (blur/enter), not per keystroke — otherwise every
  // intermediate string would persist as its own team.
  els.nameInput.addEventListener("change", () => {
    const next = els.nameInput.value.trim();
    if (next === "" || next === teamName) return;
    if (teamName.trim() !== "") deleteTeam(teamName);
    teamName = next;
    persist();
  });

  // ----- import / export: the CLI's team-file JSON, exactly -----

  els.exportButton.addEventListener("click", () => {
    const blob = new Blob([toTeamFileJson(units)], { type: "application/json" });
    const a = document.createElement("a");
    a.href = URL.createObjectURL(blob);
    a.download = `${(teamName.trim() || "team").replace(/[^\w.-]+/g, "-").toLowerCase()}.json`;
    a.click();
    URL.revokeObjectURL(a.href);
  });

  els.importButton.addEventListener("click", () => els.fileInput.click());

  els.fileInput.addEventListener("change", async () => {
    const file = els.fileInput.files?.[0];
    els.fileInput.value = ""; // re-importing the same file must re-fire
    if (!file) return;
    try {
      const imported = parseTeamFileJson(await file.text());
      teamName = freshName(file.name.replace(/\.json$/i, "") || "Imported Team");
      units = imported;
      els.nameInput.value = teamName;
      els.importError.hidden = true;
      structuralChange();
    } catch (err) {
      els.importError.textContent = `Import failed — ${(err as Error).message}`;
      els.importError.hidden = false;
    }
  });

  // ----- boot: templates dropdown, saved teams, an initial draft -----

  UNIT_TEMPLATES.forEach((tpl, i) => els.templateSelect.add(new Option(tpl.name, String(i))));
  refreshTeamSelect();
  const names = Object.keys(loadTeams()).sort();
  if (names.length > 0) loadByName(names[0]!);
  else render();

  return { refresh: refreshTeamSelect };
}
