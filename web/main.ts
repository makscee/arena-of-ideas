// Web shell — a thin, disposable client over the kernel's public API.
// It owns zero rules: pick teams, pick a seed, call battle(), hand the event
// log (and the content it ran on) to the battle screen — board, inline log,
// and inspector all read that one log.

import { DEFAULT_RUN_POOL, KERNEL_VERSION, battle, openLadder, stressRegistry, type UnitDef } from "../src/index.js";
import { resolveUnits, teamOptions } from "./catalogue.js";
import { createViewer } from "./viewer.js";
import { createEditor } from "./editor.js";
import { createGauntlet } from "./gauntlet.js";
import { createRunScreen, type RunScreen } from "./run-screen.js";
import { openLocalLadder, resetLadder } from "./run-store.js";

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
const runError = el<HTMLElement>("battle-run-error");
const randomizeButton = el<HTMLButtonElement>("randomize");
const form = el<HTMLFormElement>("controls");
const result = el<HTMLElement>("result");

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
  log: el("battle-log"),
  inspect: el("inspect-panel"),
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

  try {
    const log = battle({ teamA, teamB, seed, statuses: stressRegistry });
    viewer.load(log, { teams: { A: teamA, B: teamB }, registry: stressRegistry });
    result.hidden = false;
  } catch (err) {
    result.hidden = true;
    flagRun(`Battle failed: ${(err as Error).message}`);
  }
}

form.addEventListener("submit", (event) => {
  event.preventDefault();
  runFromControls();
});

// ---------------------------------------------------------------------------
// Views: run (the primary screen — shop/fight loop), battle (pickers +
// replay), gauntlet (win-rate sweeps), editor. The editor refreshes both
// teams' pickers on every persisted change, so edited teams are always live
// everywhere. The run screen borrows the battle viewer's DOM while a run
// battle shows, so setVisible must hear every tab change.
// ---------------------------------------------------------------------------

const views = {
  run: el<HTMLElement>("run-view"),
  battle: el<HTMLElement>("battle-view"),
  gauntlet: el<HTMLElement>("gauntlet-view"),
  editor: el<HTMLElement>("editor-view"),
};
const viewTabs = {
  run: el<HTMLButtonElement>("view-run"),
  battle: el<HTMLButtonElement>("view-battle"),
  gauntlet: el<HTMLButtonElement>("view-gauntlet"),
  editor: el<HTMLButtonElement>("view-editor"),
};

let runScreen: RunScreen | undefined;

function showView(which: keyof typeof views): void {
  for (const key of Object.keys(views) as (keyof typeof views)[]) {
    views[key].hidden = key !== which;
    viewTabs[key].classList.toggle("active", key === which);
  }
  if (which !== "battle") viewer.stop();
  runScreen?.setVisible(which === "run");
}
for (const key of Object.keys(viewTabs) as (keyof typeof viewTabs)[]) {
  viewTabs[key].addEventListener("click", () => showView(key));
}

// The run screen — the app opens on it. A failure to revive the stored
// ladder is loud (a silent fresh ladder would orphan its ghosts), but it
// must not take the other views down with it.
try {
  runScreen = createRunScreen(
    {
      newPanel: el("run-new"),
      newForm: el<HTMLFormElement>("run-new-form"),
      seed: el<HTMLInputElement>("run-seed"),
      dice: el<HTMLButtonElement>("run-seed-dice"),
      newError: el("run-new-error"),
      champ: el("run-champ"),
      warn: el("run-warn"),
      shopPanel: el("run-shop"),
      head: el("run-head"),
      next: el("run-next"),
      shopRow: el("run-shop-row"),
      rerollButton: el<HTMLButtonElement>("run-reroll"),
      line: el("run-line"),
      fightButton: el<HTMLButtonElement>("run-fight"),
      error: el("run-error"),
      inspect: el("run-inspect"),
      notice: el("run-notice"),
      battlePanel: el("run-battle"),
      battleHead: el("run-battle-head"),
      battleMount: el("run-battle-mount"),
      outcome: el("run-outcome"),
      continueButton: el<HTMLButtonElement>("run-continue"),
      endPanel: el("run-end"),
      endHead: el("run-end-head"),
      endStats: el("run-end-stats"),
      endLine: el("run-end-line"),
      newRunButton: el<HTMLButtonElement>("run-new-run"),
      ladderPanel: el("run-ladder"),
      ladderBody: el("run-ladder-body"),
    },
    {
      storage: window.localStorage,
      store: openLadder(openLocalLadder(window.localStorage), stressRegistry),
      pool: DEFAULT_RUN_POOL,
      registry: stressRegistry,
      viewer,
      viewerHost: result,
      viewerHome: el("battle-view"),
    },
  );
  runScreen.setVisible(true); // the app opens on the run tab
} catch (err) {
  // Loud, but not a dead end: the error stays on screen, and an explicit
  // two-step reset is the way out — deleting every ghost and the champion is
  // destructive, so nothing happens on a single stray click.
  const view = el("run-view");
  view.innerHTML = "";
  const msg = document.createElement("p");
  msg.className = "run-warn";
  msg.setAttribute("role", "alert");
  msg.textContent = `The run screen could not open: ${(err as Error).message}`;
  const actions = document.createElement("div");
  actions.className = "run-bar";
  const offerReset = (): void => {
    actions.innerHTML = "";
    const reset = document.createElement("button");
    reset.type = "button";
    reset.textContent = "reset ladder…";
    reset.addEventListener("click", () => {
      actions.innerHTML = "";
      const warn = document.createElement("span");
      warn.className = "run-warn";
      warn.textContent = "This deletes every ghost, the champion, and the active run — there is no undo.";
      const really = document.createElement("button");
      really.type = "button";
      really.className = "danger";
      really.textContent = "really reset";
      really.addEventListener("click", () => {
        resetLadder(window.localStorage);
        window.location.reload(); // reopen everything over the fresh (re-bootstrapped) ladder
      });
      const keep = document.createElement("button");
      keep.type = "button";
      keep.textContent = "keep it";
      keep.addEventListener("click", offerReset);
      actions.append(warn, really, keep);
    });
    actions.append(reset);
  };
  offerReset();
  view.append(msg, actions);
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
