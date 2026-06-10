// Web shell — a thin, disposable client over the kernel's public API.
// It owns zero rules: pick teams, pick a seed, call battle(), print the replay.

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
const randomizeButton = el<HTMLButtonElement>("randomize");
const form = el<HTMLFormElement>("controls");
const replayBlock = el<HTMLPreElement>("replay");

for (const select of [teamASelect, teamBSelect]) {
  for (const name of Object.keys(TEAMS)) {
    select.add(new Option(name, name));
  }
}
teamBSelect.selectedIndex = 1; // default to an Alpha-vs-Beta matchup

randomizeButton.addEventListener("click", () => {
  seedInput.value = String(Math.floor(Math.random() * 1_000_000));
});

form.addEventListener("submit", (event) => {
  event.preventDefault();
  const teamA = TEAMS[teamASelect.value];
  const teamB = TEAMS[teamBSelect.value];
  const seed = Number.parseInt(seedInput.value, 10);
  if (!teamA || !teamB || !Number.isFinite(seed)) return;

  replayBlock.hidden = false;
  try {
    const log = battle({ teamA, teamB, seed, statuses: stressRegistry });
    replayBlock.textContent = `${renderReplay(log)}\n\n(seed ${seed})`;
  } catch (err) {
    replayBlock.textContent = `Battle failed: ${(err as Error).message}`;
  }
});

el<HTMLElement>("kernel-version").textContent = `kernel v${KERNEL_VERSION}`;
