// History screen (PRD #077 slice 3) — the player-facing READ surface over the
// season archive. Past seasons' final leaderboards, kept from day one, read back
// here: a list of completed seasons (number + content version + champion), and —
// on opening one — that season's final tower, floor by floor. Same shape as
// ideas-screen.ts / ladder-view.ts: DOM in `els`, behaviour in `deps`, a
// refresh() main.ts calls on every show.
//
// History is PUBLIC, like the ladder and the ideas list — no auth: it reads the
// device's local archive directly (openLocalArchive), no server round-trip, no
// token. The archive is written by the season transition (slice 2); until a
// season ends it is empty, so the screen's empty state is the common first read,
// not an error.
//
// READ ONLY: the screen calls list()/seasonAt() and renders their output through
// the kernel's shared formatter (summarizeSeasons / seasonChampion), so the web
// and the CLI can never disagree on what a season reads as. It never archives.

import { championLabel, seasonChampion, summarizeSeasons } from "../src/index.js";
import type { SeasonArchiveStore, SeasonRecord } from "../src/index.js";

export interface HistoryScreenEls {
  /** The completed-seasons list mount — one row per archived season. */
  list: HTMLElement;
  /** The single-season final-tower panel, shown when a season row is opened. */
  detail: HTMLElement;
  /** Back-to-list control inside the detail panel. */
  back: HTMLButtonElement;
}

export interface HistoryScreenDeps {
  /** The archive backing — the device's local archive (openLocalArchive). Read
   * only here: the screen lists and reads, never archives. */
  archive: SeasonArchiveStore;
}

export interface HistoryScreen {
  /** Re-read the archive and re-render the seasons list (back to the list view
   * if a season detail was open). Called every time the screen shows, so the
   * list reflects any season that ended since last time. */
  refresh(): void;
}

export function createHistoryScreen(els: HistoryScreenEls, deps: HistoryScreenDeps): HistoryScreen {
  els.back.addEventListener("click", () => showList());

  /** Render the completed-seasons list — one row per archived season, newest
   * (highest number) first so the latest season reads at the top. Each row
   * carries the season number, the content version it ran on, and a one-line
   * champion summary; clicking it opens that season's final tower. */
  function showList(): void {
    els.detail.hidden = true;
    els.list.hidden = false;
    els.list.textContent = "";

    const records = deps.archive.list();
    if (records.length === 0) {
      const empty = document.createElement("p");
      empty.className = "history-empty";
      empty.textContent = "No seasons have ended yet — past seasons' final leaderboards will appear here.";
      els.list.append(empty);
      return;
    }

    // Completion order is ascending (season 1 first); show newest first.
    const summaries = summarizeSeasons(records).slice().reverse();
    for (const summary of summaries) {
      const row = document.createElement("button");
      row.type = "button";
      row.className = "history-row";
      row.dataset.season = String(summary.season);

      const head = document.createElement("span");
      head.className = "history-row-head";
      head.textContent = `Season ${summary.season}`;

      const meta = document.createElement("span");
      meta.className = "history-row-meta";
      meta.textContent = `content v${summary.version}`;

      const champ = document.createElement("span");
      champ.className = "history-row-champ";
      champ.textContent = championLabel(summary.champion);

      row.append(head, meta, champ);
      row.addEventListener("click", () => showSeason(summary.season));
      els.list.append(row);
    }
  }

  /** Render one season's final tower — the frozen leaderboard, floor by floor
   * from the top (champion) down. Reads the archived record fresh through
   * seasonAt; a number with no record (race against a cleared archive) drops
   * back to the list rather than showing a blank panel. */
  function showSeason(season: number): void {
    const record = deps.archive.seasonAt(season);
    if (record === null) {
      showList();
      return;
    }
    els.list.hidden = true;
    els.detail.hidden = false;
    renderTower(record);
  }

  function renderTower(record: SeasonRecord): void {
    // Keep the back button (the first child); rebuild everything after it.
    while (els.detail.lastChild && els.detail.lastChild !== els.back) {
      els.detail.removeChild(els.detail.lastChild);
    }

    const champion = seasonChampion(record);
    const heading = document.createElement("h3");
    heading.className = "history-detail-head";
    heading.textContent = `Season ${record.season} — final tower (content v${record.version})`;

    const champLine = document.createElement("p");
    champLine.className = "history-detail-champ";
    champLine.textContent = `Champion: ${championLabel(champion)}`;

    els.detail.append(heading, champLine);

    const floors = Object.keys(record.finalTower.bosses)
      .map(Number)
      .sort((a, b) => b - a); // top floor (champion) first

    if (floors.length === 0) {
      const none = document.createElement("p");
      none.className = "history-empty";
      none.textContent = "This season's tower seated no boss.";
      els.detail.append(none);
      return;
    }

    const tower = document.createElement("div");
    tower.className = "history-tower";
    for (const floor of floors) {
      const boss = record.finalTower.bosses[String(floor)]!;
      const team = boss.team.map((u) => u.name).join(", ");
      const item = document.createElement("div");
      item.className = "history-floor";
      if (floor === floors[0]) item.classList.add("history-floor-champ");
      item.textContent = `Floor ${floor}: ${boss.runId} — ${team}` + (floor === floors[0] ? " ★ champion" : "");
      tower.append(item);
    }
    els.detail.append(tower);
  }

  return { refresh: showList };
}
