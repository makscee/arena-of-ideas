// Web shell — a thin, disposable client over the kernel's public API.
// It owns zero rules: pick teams, pick a seed, call battle(), hand the event
// log to the visual viewer and the text replay. Both views read the same log.

import {
  KERNEL_VERSION,
  battle,
  renderReplay,
  assertValidContent,
  stressRegistry,
  Summoner,
  Silencer,
  Necromancer,
  Venomancer,
  type UnitDef,
} from "../src/index.js";
import teamAlphaJson from "../examples/team-alpha.json";
import teamBetaJson from "../examples/team-beta.json";
import { createViewer } from "./viewer.js";

// ---------------------------------------------------------------------------
// Team catalogue — the shipped example files plus a squad of stress-set units.
// Everything goes through the validator before battle(), same as the CLI.
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

for (const select of [teamASelect, teamBSelect]) {
  for (const name of Object.keys(TEAMS)) {
    select.add(new Option(name, name));
  }
}
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

form.addEventListener("submit", (event) => {
  event.preventDefault();
  const teamA = TEAMS[teamASelect.value];
  const teamB = TEAMS[teamBSelect.value];
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

el<HTMLElement>("kernel-version").textContent = `kernel v${KERNEL_VERSION}`;
