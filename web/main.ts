// Web shell — a thin, disposable client over the kernel's public API.
// It owns zero rules: pick teams, pick a seed, call battle(), hand the event
// log to the visual viewer and the text replay. Both views read the same log.

import {
  KERNEL_VERSION,
  battle,
  renderReplay,
  assertValidContent,
  stressRegistry,
  validateTeam,
  Summoner,
  Silencer,
  Necromancer,
  Venomancer,
  type UnitDef,
} from "../src/index.js";
import teamAlphaJson from "../examples/team-alpha.json";
import teamBetaJson from "../examples/team-beta.json";
import { createViewer } from "./viewer.js";
import { createEditor } from "./editor.js";
import { loadTeams } from "./teams.js";

// ---------------------------------------------------------------------------
// Team catalogue — the shipped example files plus a squad of stress-set units.
// Everything goes through the validator before battle(), same as the CLI.
// Edited teams (localStorage) appear alongside, re-validated at run time —
// an invalid team can be edited but never reaches battle().
// ---------------------------------------------------------------------------

function loadTeam(name: string, units: unknown): UnitDef[] {
  assertValidContent(units, stressRegistry, name);
  return units;
}

const TEAMS: Record<string, UnitDef[]> = {
  "Team Alpha (aggro venom)": loadTeam("Team Alpha", teamAlphaJson.units),
  "Team Beta (control/sustain)": loadTeam("Team Beta", teamBetaJson.units),
  "Stress Squad (kernel units)": loadTeam("Stress Squad", [Venomancer, Summoner, Silencer, Necromancer]),
};

// Picker values are namespaced so a saved team may share a shipped team's name.
const SHIPPED_PREFIX = "shipped:";
const EDITED_PREFIX = "edited:";

// ---------------------------------------------------------------------------
// DOM wiring
// ---------------------------------------------------------------------------

function el<T extends HTMLElement>(id: string): T {
  const node = document.getElementById(id);
  if (!node) throw new Error(`missing #${id}`);
  return node as T;
}

const teamASelect = el<HTMLSelectElement>("team-a");
const teamBSelect = el<HTMLSelectElement>("team-b");
const seedInput = el<HTMLInputElement>("seed");
const seedError = el<HTMLElement>("seed-error");
const runError = el<HTMLElement>("run-error");
const randomizeButton = el<HTMLButtonElement>("randomize");
const form = el<HTMLFormElement>("controls");
const result = el<HTMLElement>("result");
const boardPanel = el<HTMLElement>("board-panel");
const replayBlock = el<HTMLPreElement>("replay");
const tabBoard = el<HTMLButtonElement>("tab-board");
const tabText = el<HTMLButtonElement>("tab-text");

const viewer = createViewer({
  board: el("board"),
  prev: el<HTMLButtonElement>("step-prev"),
  next: el<HTMLButtonElement>("step-next"),
  play: el<HTMLButtonElement>("step-play"),
  speed: el<HTMLSelectElement>("speed"),
  scrub: el<HTMLInputElement>("scrub"),
  stepLabel: el("step-label"),
  eventDesc: el("event-desc"),
  eventCause: el("event-cause"),
});

function fillTeamPickers(): void {
  for (const select of [teamASelect, teamBSelect]) {
    const kept = select.value; // keep the selection across editor saves when possible
    select.innerHTML = "";
    const shipped = document.createElement("optgroup");
    shipped.label = "shipped";
    for (const name of Object.keys(TEAMS)) shipped.append(new Option(name, SHIPPED_PREFIX + name));
    select.append(shipped);
    const editedNames = Object.keys(loadTeams()).sort();
    if (editedNames.length > 0) {
      const edited = document.createElement("optgroup");
      edited.label = "edited";
      for (const name of editedNames) edited.append(new Option(name, EDITED_PREFIX + name));
      select.append(edited);
    }
    if ([...select.options].some((o) => o.value === kept)) select.value = kept;
  }
}

fillTeamPickers();
teamBSelect.selectedIndex = 1; // default to an Alpha-vs-Beta matchup

function flagSeed(message: string): void {
  seedError.textContent = message;
  seedError.hidden = false;
  seedInput.setAttribute("aria-invalid", "true");
  seedInput.focus();
}

function clearSeedFlag(): void {
  seedError.hidden = true;
  seedInput.removeAttribute("aria-invalid");
}

seedInput.addEventListener("input", clearSeedFlag);

function flagRun(message: string): void {
  runError.textContent = message;
  runError.hidden = false;
}

function clearRunFlag(): void {
  runError.hidden = true;
}

randomizeButton.addEventListener("click", () => {
  seedInput.value = String(Math.floor(Math.random() * 1_000_000));
  clearSeedFlag();
});

// Tabs: the board is the watchable replay; the text replay stays ground truth.
function showTab(which: "board" | "text"): void {
  boardPanel.hidden = which !== "board";
  replayBlock.hidden = which !== "text";
  tabBoard.classList.toggle("active", which === "board");
  tabText.classList.toggle("active", which === "text");
  if (which !== "board") viewer.stop();
}
tabBoard.addEventListener("click", () => showTab("board"));
tabText.addEventListener("click", () => showTab("text"));

/** Resolve a picker value to units. Edited teams are re-validated here —
 * the gate before battle(), same role the CLI loader plays. */
function resolveTeam(value: string): UnitDef[] | null {
  if (value.startsWith(SHIPPED_PREFIX)) return TEAMS[value.slice(SHIPPED_PREFIX.length)] ?? null;
  const name = value.slice(EDITED_PREFIX.length);
  const units = loadTeams()[name];
  if (!units) {
    flagRun(`Team "${name}" is no longer saved.`);
    return null;
  }
  const issues = validateTeam(units, stressRegistry, name);
  if (issues.length > 0) {
    flagRun(
      `Team "${name}" is invalid (${issues.length} issue${issues.length === 1 ? "" : "s"}) — fix it in the editor. First: ${issues[0]!.path}: ${issues[0]!.message}`,
    );
    return null;
  }
  return units;
}

form.addEventListener("submit", (event) => {
  event.preventDefault();
  clearRunFlag();
  const teamA = resolveTeam(teamASelect.value);
  const teamB = resolveTeam(teamBSelect.value);
  if (!teamA || !teamB) return;
  // An empty or non-numeric seed flags the user — never a silent no-op.
  const raw = seedInput.value.trim();
  const seed = Number(raw);
  if (raw === "" || !Number.isInteger(seed)) {
    flagSeed(raw === "" ? "Seed is empty — type a whole number or roll the dice." : "Seed must be a whole number.");
    return;
  }
  clearSeedFlag();

  result.hidden = false;
  try {
    const log = battle({ teamA, teamB, seed, statuses: stressRegistry });
    viewer.load(log);
    replayBlock.textContent = `${renderReplay(log)}\n\n(seed ${seed})`;
    showTab("board");
  } catch (err) {
    replayBlock.textContent = `Battle failed: ${(err as Error).message}`;
    showTab("text");
  }
});

// ---------------------------------------------------------------------------
// Views: battle (pickers + replay) and editor. The editor refreshes the
// battle pickers on every persisted change, so edited teams are always live.
// ---------------------------------------------------------------------------

const battleView = el<HTMLElement>("battle-view");
const editorView = el<HTMLElement>("editor-view");
const viewBattle = el<HTMLButtonElement>("view-battle");
const viewEditor = el<HTMLButtonElement>("view-editor");

function showView(which: "battle" | "editor"): void {
  battleView.hidden = which !== "battle";
  editorView.hidden = which !== "editor";
  viewBattle.classList.toggle("active", which === "battle");
  viewEditor.classList.toggle("active", which === "editor");
  if (which !== "battle") viewer.stop();
}
viewBattle.addEventListener("click", () => showView("battle"));
viewEditor.addEventListener("click", () => showView("editor"));

createEditor(
  {
    teamSelect: el<HTMLSelectElement>("ed-team-select"),
    newButton: el<HTMLButtonElement>("ed-new"),
    deleteButton: el<HTMLButtonElement>("ed-delete"),
    exportButton: el<HTMLButtonElement>("ed-export"),
    importButton: el<HTMLButtonElement>("ed-import"),
    fileInput: el<HTMLInputElement>("ed-file"),
    nameInput: el<HTMLInputElement>("ed-name"),
    unitsBox: el<HTMLElement>("ed-units"),
    templateSelect: el<HTMLSelectElement>("ed-template"),
    addButton: el<HTMLButtonElement>("ed-add"),
    verdict: el<HTMLElement>("ed-verdict"),
    issuesList: el<HTMLElement>("ed-issues"),
    importError: el<HTMLElement>("ed-import-error"),
  },
  fillTeamPickers,
);

el<HTMLElement>("kernel-version").textContent = `kernel v${KERNEL_VERSION}`;
