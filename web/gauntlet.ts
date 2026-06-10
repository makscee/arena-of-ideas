// Gauntlet — pit a challenger team against the shipped teams (and optionally
// the other saved teams) as a win-rate sweep per matchup. The browser embryo
// of the sim gate: a matchup is a distribution, not a memorizable result.
// All distribution math is the kernel's sweep helper (the same one the CLI's
// --sweep mode runs); this file owns only the chunked drain that keeps the
// page responsive, the results table, and the jump-to-viewer hooks.

import {
  stressRegistry,
  summarizeSweep,
  sweepSeeds,
  type SweepOutcome,
  type UnitDef,
} from "../src/index.js";
import { resolveUnits, teamOptions, type TeamOption } from "./catalogue.js";

// Drain the sweep generator in ~frame-sized slices: battles run until the
// budget is spent, then the loop yields to the browser so input and paint
// stay live. Simpler than a worker and responsive enough at kernel speeds.
const CHUNK_BUDGET_MS = 12;
const yieldToBrowser = (): Promise<void> => new Promise((resolve) => setTimeout(resolve, 0));

function esc(s: string): string {
  return s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");
}

const pct = (x: number, n: number): string => ((x / n) * 100).toFixed(1) + "%";

interface MatchupRow {
  opponent: TeamOption;
  outcomes: SweepOutcome[];
}

interface GauntletEls {
  form: HTMLFormElement;
  challenger: HTMLSelectElement;
  seeds: HTMLInputElement;
  includeSaved: HTMLInputElement;
  error: HTMLElement;
  progress: HTMLElement;
  results: HTMLElement;
  tableBody: HTMLElement;
  tableFoot: HTMLElement;
}

export interface Gauntlet {
  /** Re-read the catalogue (saved teams changed in the editor). */
  refresh(): void;
}

export function createGauntlet(
  els: GauntletEls,
  watch: (challengerValue: string, opponentValue: string, seed: number) => void,
): Gauntlet {
  // A re-run supersedes the in-flight one: the old loop sees a newer token
  // after its next yield and quietly stops.
  let runToken = 0;

  function refreshChallengerPicker(): void {
    const kept = els.challenger.value;
    els.challenger.innerHTML = "";
    for (const opt of teamOptions()) els.challenger.add(new Option(opt.label, opt.value));
    if ([...els.challenger.options].some((o) => o.value === kept)) els.challenger.value = kept;
  }

  function flag(message: string): void {
    els.error.textContent = message;
    els.error.hidden = false;
  }

  /** First outcome of each result class — the representative seed to watch. */
  function exampleSeeds(outcomes: SweepOutcome[]): Partial<Record<"A" | "B" | "draw", number>> {
    const seeds: Partial<Record<"A" | "B" | "draw", number>> = {};
    for (const o of outcomes) seeds[o.winner] ??= o.seed;
    return seeds;
  }

  // The challenger is baked into each button at render time, so a later
  // change of the picker can't redirect an old row's watch.
  function watchButtons(challengerValue: string, opponentValue: string, outcomes: SweepOutcome[]): string {
    const seeds = exampleSeeds(outcomes);
    const labels = { A: "win", B: "loss", draw: "draw" } as const;
    return (Object.keys(labels) as (keyof typeof labels)[])
      .filter((k) => seeds[k] !== undefined)
      .map(
        (k) =>
          `<button type="button" class="g-watch" data-challenger="${esc(challengerValue)}" data-opponent="${esc(opponentValue)}" data-seed="${seeds[k]}">▶ ${labels[k]}</button>`,
      )
      .join(" ");
  }

  function matchupTr(row: MatchupRow, challengerValue: string): string {
    const s = summarizeSweep(row.outcomes);
    return `<tr>
      <td class="g-name">${esc(row.opponent.name)}</td>
      <td>${s.aWins}/${s.bWins}/${s.draws}</td>
      <td>${pct(s.aWins, s.n)}</td>
      <td>${(s.totalTurns / s.n).toFixed(1)}</td>
      <td class="g-actions">${watchButtons(challengerValue, row.opponent.value, row.outcomes)}</td>
    </tr>`;
  }

  function overallTr(rows: MatchupRow[]): string {
    const s = summarizeSweep(rows.flatMap((r) => r.outcomes));
    return `<tr>
      <td class="g-name">overall (${rows.length} matchups)</td>
      <td>${s.aWins}/${s.bWins}/${s.draws}</td>
      <td>${pct(s.aWins, s.n)}</td>
      <td>${(s.totalTurns / s.n).toFixed(1)}</td>
      <td></td>
    </tr>`;
  }

  async function run(): Promise<void> {
    const token = ++runToken;
    els.error.hidden = true;

    const challengerValue = els.challenger.value;
    const challenger = resolveUnits(challengerValue);
    if ("error" in challenger) return flag(challenger.error);

    const raw = els.seeds.value.trim();
    const n = Number(raw);
    if (raw === "" || !Number.isInteger(n) || n < 1) {
      return flag("Seeds per matchup must be a positive whole number.");
    }

    // Opponents: every shipped team, plus the other saved teams when asked.
    // The challenger never fights itself; invalid saved teams can't battle.
    const opponents = teamOptions().filter(
      (o) => o.value !== challengerValue && !o.invalid && (o.shipped || els.includeSaved.checked),
    );
    if (opponents.length === 0) return flag("No opponents to fight.");

    els.results.hidden = false;
    els.tableBody.innerHTML = "";
    els.tableFoot.innerHTML = "";
    els.progress.hidden = false;

    const rows: MatchupRow[] = [];
    for (const opponent of opponents) {
      const resolved = resolveUnits(opponent.value);
      if ("error" in resolved) return flag(resolved.error); // raced a deletion mid-run
      const input = { teamA: challenger.units, teamB: resolved.units as UnitDef[], statuses: stressRegistry };

      const outcomes: SweepOutcome[] = [];
      let deadline = performance.now() + CHUNK_BUDGET_MS;
      for (const outcome of sweepSeeds(input, n)) {
        outcomes.push(outcome);
        if (performance.now() >= deadline) {
          els.progress.textContent = `vs ${opponent.name}: seed ${outcomes.length}/${n}…`;
          await yieldToBrowser();
          if (token !== runToken) return; // superseded by a newer run
          deadline = performance.now() + CHUNK_BUDGET_MS;
        }
      }

      rows.push({ opponent, outcomes });
      els.tableBody.innerHTML = rows.map((r) => matchupTr(r, challengerValue)).join("");
      console.log(`gauntlet: vs ${opponent.name} — ${outcomes.length} seeds done`);
      await yieldToBrowser();
      if (token !== runToken) return;
    }

    els.tableFoot.innerHTML = overallTr(rows);
    els.progress.textContent = `done — ${rows.length} matchups × ${n} seeds`;
  }

  els.form.addEventListener("submit", (event) => {
    event.preventDefault();
    void run();
  });

  // Watch a battle from a row: jump to the viewer at that matchup and seed.
  els.results.addEventListener("click", (ev) => {
    const btn = (ev.target as HTMLElement).closest<HTMLElement>("button.g-watch");
    if (!btn) return;
    watch(
      btn.getAttribute("data-challenger")!,
      btn.getAttribute("data-opponent")!,
      Number(btn.getAttribute("data-seed")),
    );
  });

  refreshChallengerPicker();
  return { refresh: refreshChallengerPicker };
}
