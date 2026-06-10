// Web shell — a thin, disposable client over the kernel's public API.
// It owns zero rules: pick teams, pick a seed, call battle(), hand the event
// log to the visual viewer and the text replay. Both views read the same log.

import { KERNEL_VERSION, battle, renderReplay, stressRegistry, type UnitDef } from "../src/index.js";
import { resolveUnits, teamOptions } from "./catalogue.js";
import { createViewer } from "./viewer.js";
import { createEditor } from "./editor.js";
import { createGauntlet } from "./gauntlet.js";

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
  const options = teamOptions(); // invalid saved teams come pre-marked in the label
  for (const select of [teamASelect, teamBSelect]) {
    const kept = select.value; // keep the selection across editor saves when possible
    select.innerHTML = "";
    const shipped = document.createElement("optgroup");
    shipped.label = "shipped";
    const edited = document.createElement("optgroup");
    edited.label = "edited";
    for (const opt of options) (opt.shipped ? shipped : edited).append(new Option(opt.label, opt.value));
    select.append(shipped);
    if (edited.children.length > 0) select.append(edited);
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

/** Resolve a picker value to units via the catalogue's gate; a failure
 * surfaces as the run error (the validator's own message for saved teams). */
function resolveTeam(value: string): UnitDef[] | null {
  const resolved = resolveUnits(value);
  if ("error" in resolved) {
    flagRun(resolved.error);
    return null;
  }
  return resolved.units;
}

function runFromControls(): void {
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
}

form.addEventListener("submit", (event) => {
  event.preventDefault();
  runFromControls();
});

// ---------------------------------------------------------------------------
// Views: battle (pickers + replay), gauntlet (win-rate sweeps), editor.
// The editor refreshes both teams' pickers on every persisted change, so
// edited teams are always live everywhere.
// ---------------------------------------------------------------------------

const views = {
  battle: el<HTMLElement>("battle-view"),
  gauntlet: el<HTMLElement>("gauntlet-view"),
  editor: el<HTMLElement>("editor-view"),
};
const viewTabs = {
  battle: el<HTMLButtonElement>("view-battle"),
  gauntlet: el<HTMLButtonElement>("view-gauntlet"),
  editor: el<HTMLButtonElement>("view-editor"),
};

function showView(which: keyof typeof views): void {
  for (const key of Object.keys(views) as (keyof typeof views)[]) {
    views[key].hidden = key !== which;
    viewTabs[key].classList.toggle("active", key === which);
  }
  if (which !== "battle") viewer.stop();
}
for (const key of Object.keys(viewTabs) as (keyof typeof viewTabs)[]) {
  viewTabs[key].addEventListener("click", () => showView(key));
}

// Watch a gauntlet matchup: load it into the battle controls and run — the
// viewer shows exactly the battle that produced that row's result.
const gauntlet = createGauntlet(
  {
    form: el<HTMLFormElement>("gauntlet-controls"),
    challenger: el<HTMLSelectElement>("g-challenger"),
    seeds: el<HTMLInputElement>("g-seeds"),
    includeSaved: el<HTMLInputElement>("g-include-saved"),
    error: el<HTMLElement>("g-error"),
    progress: el<HTMLElement>("g-progress"),
    results: el<HTMLElement>("g-results"),
    tableBody: el<HTMLElement>("g-rows"),
    tableFoot: el<HTMLElement>("g-overall"),
  },
  (challengerValue, opponentValue, seed) => {
    teamASelect.value = challengerValue;
    teamBSelect.value = opponentValue;
    seedInput.value = String(seed);
    showView("battle");
    runFromControls();
  },
);

function onTeamsChanged(): void {
  fillTeamPickers();
  gauntlet.refresh();
}

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
  onTeamsChanged,
);

el<HTMLElement>("kernel-version").textContent = `kernel v${KERNEL_VERSION}`;
