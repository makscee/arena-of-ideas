// Battle Editor (#066 slice 2) — the one dev surface: build two teams, fight
// them, edit, replay. It owns zero rules — every fight is battle() over the
// kernel, the replay is the existing viewer mounted as a BLACK BOX (the same
// load() seam run-screen uses; no battle-internal file is touched here). It is
// a free sandbox: its fights are never runs and never submitted.
//
// The loop's point is isolation: the seed is LOCKED by default, so editing a
// team and pressing Fight again replays the SAME seed against the changed
// teams — the only thing that moved is the edit, so the outcome reflects it.
// Unlock (or reroll) to check variance. Slice 3 adds Run ×N on top of this.

import {
  TEAM_SIZE,
  battle,
  codexUnits,
  stressRegistry,
  type StatusRegistry,
  type UnitDef,
} from "../src/index.js";
import { SHIPPED_TEAMS } from "./catalogue.js";
import { createPalette, type Palette } from "./unit-palette.js";
import { loadTeams } from "./teams.js";
import { unitCardHtml } from "./unit-card.js";
import type { Viewer } from "./viewer.js";

const clone = <T>(v: T): T => JSON.parse(JSON.stringify(v)) as T;

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

/** An empty number field reads as missing (NaN), never silently as 0 — same
 * discipline as the legacy editor's stat fields. */
const numField = (value: string): number => (value.trim() === "" ? Number.NaN : Number(value));

type SideKey = "A" | "B";

export interface BattleEditorEls {
  /** The two column roots — slots render inside these. */
  columnA: HTMLElement;
  columnB: HTMLElement;
  /** The seed control. */
  seed: HTMLInputElement;
  reroll: HTMLButtonElement;
  lock: HTMLInputElement;
  fight: HTMLButtonElement;
  error: HTMLElement;
  /** Where the shared viewer DOM is reparented while a fight plays. */
  mount: HTMLElement;
}

export interface BattleEditorDeps {
  /** The pool the palette offers — shipped + approved + codex (summons etc.). */
  pool: () => readonly UnitDef[];
  registry: StatusRegistry;
  viewer: Viewer;
  /** The viewer's shared DOM (#result) — reparented into `mount` for a fight. */
  viewerHost: HTMLElement;
  /** Where the viewer DOM lives otherwise (the battle view) — returned on hide. */
  viewerHome: HTMLElement;
}

export interface BattleEditor {
  /** The editor view was shown/hidden — mounts or returns the shared viewer. */
  setVisible(visible: boolean): void;
}

export function createBattleEditor(els: BattleEditorEls, deps: BattleEditorDeps): BattleEditor {
  // Each side is a list of placed units (deep clones, so an in-place edit never
  // aliases the pool, a saved team, or a shipped template). Defaults give the
  // sandbox something to fight on first open.
  const teams: Record<SideKey, UnitDef[]> = {
    A: clone(SHIPPED_TEAMS["Team Alpha (aggro venom)"] ?? []),
    B: clone(SHIPPED_TEAMS["Team Beta (control/sustain)"] ?? []),
  };
  let visible = false;
  let mounted = false; // the shared viewer is currently parented into our mount
  let fought = false; // a battle has been mounted at least once this session

  // One palette, shared by both columns' "add unit" buttons. The target slot is
  // captured on open and consumed on pick.
  let pickTarget: SideKey | undefined;
  const palette: Palette = createPalette({
    pool: deps.pool,
    registry: deps.registry,
    onPick: (def) => {
      if (pickTarget === undefined) return;
      const team = teams[pickTarget];
      pickTarget = undefined;
      if (team.length >= TEAM_SIZE) return;
      team.push(def);
      renderBoth();
    },
  });
  document.body.append(palette.element);

  // ----- rendering -----

  function statusRow(side: SideKey, i: number, j: number, name: string, stacks: number): string {
    return `
      <div class="be-row">
        <span class="be-status-name">${esc(name)}</span>
        <input type="number" value="${esc(String(stacks))}" data-field="stacks" data-side="${side}" data-i="${i}" data-j="${j}" title="Stacks" />
        <button type="button" data-act="remove-status" data-side="${side}" data-i="${i}" data-j="${j}" title="Remove status">✕</button>
      </div>`;
  }

  function slot(side: SideKey, u: UnitDef, i: number): string {
    const statusOptions = Object.keys(deps.registry)
      .map((name) => `<option value="${esc(name)}">${esc(name)}</option>`)
      .join("");
    const statuses = (u.statuses ?? []).map((s, j) => statusRow(side, i, j, String(s.status), Number(s.stacks))).join("");
    const card = unitCardHtml({
      artName: u.name ?? "",
      label: u.name ?? "",
      hp: u.base?.hp ?? "",
      pwr: u.base?.pwr ?? "",
      statuses: u.statuses,
      registry: deps.registry,
      ...(u.level !== undefined ? { level: u.level } : {}),
      classes: "be-card",
      attrs: `data-slot="${i}"`,
      title: `${u.name ?? "?"} — slot ${i + 1}${i === 0 ? " · front" : ""}`,
    });
    return `
      <div class="be-slot" data-side="${side}" data-i="${i}">
        ${card}
        <div class="be-edit">
          <label>hp <input type="number" value="${esc(String(u.base?.hp ?? ""))}" data-field="hp" data-side="${side}" data-i="${i}" /></label>
          <label>pwr <input type="number" value="${esc(String(u.base?.pwr ?? ""))}" data-field="pwr" data-side="${side}" data-i="${i}" /></label>
          <label>lvl <input type="number" value="${esc(String(u.level ?? ""))}" data-field="level" data-side="${side}" data-i="${i}" placeholder="1" /></label>
          <button type="button" data-act="up" data-side="${side}" data-i="${i}" ${i === 0 ? "disabled" : ""} title="Move toward front">↑</button>
          <button type="button" data-act="down" data-side="${side}" data-i="${i}" title="Move toward back">↓</button>
          <button type="button" data-act="remove" data-side="${side}" data-i="${i}" title="Remove unit">✕</button>
        </div>
        <div class="be-statuses">
          ${statuses || '<span class="be-none">no statuses</span>'}
          <div class="be-row">
            <select data-pick-status data-side="${side}" data-i="${i}">${statusOptions}</select>
            <button type="button" data-act="add-status" data-side="${side}" data-i="${i}">add status</button>
          </div>
        </div>
      </div>`;
  }

  // The column's stable chrome is built ONCE per side (head, slots region, the
  // actions bar with its loader <select>). Only the slots region is re-rendered
  // on a team change — so the loader/add/clear controls stay the SAME DOM nodes
  // across edits (a full innerHTML swap would replace the <select> mid-action
  // and lose a just-fired selectOption; it also drops palette focus).
  const skel: Record<SideKey, { count: HTMLElement; slots: HTMLElement; add: HTMLButtonElement; load: HTMLSelectElement }> =
    {} as never;

  function buildSkeleton(side: SideKey): void {
    const root = side === "A" ? els.columnA : els.columnB;
    root.innerHTML =
      `<div class="be-col-head"><span class="be-col-name">Team ${side}</span>` +
      `<span class="be-col-count" data-count></span></div>` +
      `<div class="be-slots" data-slots></div>` +
      `<div class="be-col-actions">` +
      `<button type="button" class="be-add" data-add="${side}">+ unit</button>` +
      `<select class="be-load" data-load="${side}"></select>` +
      `<button type="button" class="be-clear" data-clear="${side}">clear</button>` +
      `</div>`;
    skel[side] = {
      count: root.querySelector("[data-count]")!,
      slots: root.querySelector("[data-slots]")!,
      add: root.querySelector(".be-add")!,
      load: root.querySelector(".be-load")!,
    };
    refreshLoaderOptions(side);
  }

  /** (Re)fill a side's quick-loader options: copy-from-other, shipped templates,
   * saved teams. The <select> node itself is preserved. Called on build and on
   * every editor show (saved teams may have changed elsewhere). */
  function refreshLoaderOptions(side: SideKey): void {
    const other = side === "A" ? "B" : "A";
    const shipped = Object.keys(SHIPPED_TEAMS)
      .map((name) => `<option value="shipped:${esc(name)}">▸ ${esc(name)}</option>`)
      .join("");
    const saved = Object.keys(loadTeams())
      .sort()
      .map((name) => `<option value="saved:${esc(name)}">★ ${esc(name)}</option>`)
      .join("");
    skel[side].load.innerHTML =
      `<option value="">load…</option>` +
      `<option value="copy:${other}">⇄ copy Team ${other} → ${side}</option>` +
      (shipped !== "" ? `<optgroup label="shipped templates">${shipped}</optgroup>` : "") +
      (saved !== "" ? `<optgroup label="saved teams">${saved}</optgroup>` : "");
    skel[side].load.value = "";
  }

  function renderColumn(side: SideKey): void {
    const team = teams[side];
    const s = skel[side];
    s.slots.innerHTML =
      team.map((u, i) => slot(side, u, i)).join("") || '<span class="be-empty">empty — add a unit</span>';
    s.count.textContent = `${team.length}/${TEAM_SIZE}`;
    s.add.disabled = team.length >= TEAM_SIZE;
  }

  function renderBoth(): void {
    renderColumn("A");
    renderColumn("B");
    // Fight needs at least one unit a side — an empty team can't battle.
    els.fight.disabled = teams.A.length === 0 || teams.B.length === 0;
  }

  // ----- editing (delegated, so a re-render rebuilds freely) -----

  function onInput(ev: Event): void {
    const t = ev.target as HTMLInputElement;
    const field = t.getAttribute("data-field");
    if (field === null) return;
    const side = t.getAttribute("data-side") as SideKey;
    const u = teams[side]?.[Number(t.getAttribute("data-i"))];
    if (u === undefined) return;
    if (field === "hp" || field === "pwr") {
      if (typeof u.base !== "object" || u.base === null) u.base = { hp: Number.NaN, pwr: Number.NaN };
      u.base[field] = numField(t.value);
    } else if (field === "level") {
      const v = numField(t.value);
      if (Number.isNaN(v)) delete u.level;
      else u.level = v;
    } else if (field === "stacks") {
      const s = u.statuses?.[Number(t.getAttribute("data-j"))];
      if (s !== undefined) s.stacks = numField(t.value);
    }
    // Re-render the edited side so the card's chips/badge reflect the override.
    renderColumn(side);
    renderBoth();
  }

  function onClick(ev: Event): void {
    const target = ev.target as HTMLElement;
    const addBtn = target.closest<HTMLElement>("[data-add]");
    if (addBtn !== null) {
      pickTarget = addBtn.getAttribute("data-add") as SideKey;
      palette.open(addBtn);
      return;
    }
    const clearBtn = target.closest<HTMLElement>("[data-clear]");
    if (clearBtn !== null) {
      teams[clearBtn.getAttribute("data-clear") as SideKey] = [];
      renderBoth();
      return;
    }
    const btn = target.closest<HTMLElement>("button[data-act]");
    if (btn === null) return;
    const act = btn.getAttribute("data-act")!;
    const side = btn.getAttribute("data-side") as SideKey;
    const i = Number(btn.getAttribute("data-i"));
    const j = Number(btn.getAttribute("data-j"));
    const team = teams[side];
    const u = team?.[i];
    if (u === undefined) return;
    if (act === "remove") team.splice(i, 1);
    else if (act === "up" && i > 0) [team[i - 1], team[i]] = [team[i]!, team[i - 1]!];
    else if (act === "down" && i < team.length - 1) [team[i], team[i + 1]] = [team[i + 1]!, team[i]!];
    else if (act === "remove-status") u.statuses?.splice(j, 1);
    else if (act === "add-status") {
      const pick = (side === "A" ? els.columnA : els.columnB).querySelector<HTMLSelectElement>(
        `select[data-pick-status][data-side="${side}"][data-i="${i}"]`,
      );
      if (pick !== null && pick.value !== "") (u.statuses ??= []).push({ status: pick.value, stacks: 1 });
    } else return;
    renderColumn(side);
    renderBoth();
  }

  function onLoad(ev: Event): void {
    const sel = (ev.target as HTMLElement).closest<HTMLSelectElement>("select[data-load]");
    if (sel === null || sel.value === "") return;
    const side = sel.getAttribute("data-load") as SideKey;
    const value = sel.value;
    sel.value = ""; // reset to the resting label; loading the same option must re-fire
    if (value.startsWith("copy:")) {
      const from = value.slice("copy:".length) as SideKey;
      teams[side] = clone(teams[from]);
    } else if (value.startsWith("shipped:")) {
      const units = SHIPPED_TEAMS[value.slice("shipped:".length)];
      if (units !== undefined) teams[side] = clone(units);
    } else if (value.startsWith("saved:")) {
      const units = loadTeams()[value.slice("saved:".length)];
      if (units !== undefined) teams[side] = clone(units);
    }
    renderBoth();
  }

  for (const col of [els.columnA, els.columnB]) {
    col.addEventListener("input", onInput);
    col.addEventListener("click", onClick);
    col.addEventListener("change", onLoad);
  }

  // ----- seed control -----

  els.reroll.addEventListener("click", () => {
    els.seed.value = String(Math.floor(Math.random() * 1_000_000));
    clearError();
  });

  /** Read the seed box; an empty/non-integer value flags rather than silently
   * fighting at NaN. */
  function readSeed(): number | undefined {
    const raw = els.seed.value.trim();
    const seed = Number(raw);
    if (raw === "" || !Number.isInteger(seed)) {
      flag(raw === "" ? "Seed is empty — type a whole number or reroll." : "Seed must be a whole number.");
      return undefined;
    }
    return seed;
  }

  function flag(message: string): void {
    els.error.textContent = message;
    els.error.hidden = false;
  }
  function clearError(): void {
    els.error.hidden = true;
    els.error.textContent = "";
  }

  // ----- fight: assemble both teams + the seed, run battle(), mount the viewer -----

  function fight(): void {
    clearError();
    if (teams.A.length === 0 || teams.B.length === 0) {
      flag("both teams need at least one unit.");
      return;
    }
    // Locked (default) keeps the seed where it is, so the only thing that moved
    // since the last fight is the edit; unlocked rerolls each fight so variance
    // shows. The box always carries the seed that was actually fought.
    if (!els.lock.checked) els.seed.value = String(Math.floor(Math.random() * 1_000_000));
    const seed = readSeed();
    if (seed === undefined) return;
    const teamA = clone(teams.A);
    const teamB = clone(teams.B);
    let log;
    try {
      log = battle({ teamA, teamB, seed, statuses: deps.registry });
    } catch (err) {
      flag(`Battle failed: ${(err as Error).message}`);
      return;
    }
    mountViewer();
    deps.viewer.load(log, { teams: { A: teamA, B: teamB }, registry: deps.registry }, { autoplay: true });
    fought = true;
    els.mount.scrollIntoView({ block: "nearest" });
  }

  els.fight.addEventListener("click", fight);

  // ----- the shared viewer DOM (borrowed exactly like run-screen) -----

  function mountViewer(): void {
    if (!visible) return;
    els.mount.append(deps.viewerHost);
    deps.viewerHost.hidden = false;
    mounted = true;
  }

  function unmountViewer(): void {
    if (deps.viewerHost.parentElement !== els.mount) return;
    deps.viewer.stop();
    deps.viewerHost.hidden = true;
    deps.viewerHome.append(deps.viewerHost);
    mounted = false;
  }

  // ----- boot -----

  buildSkeleton("A");
  buildSkeleton("B");
  renderBoth();

  return {
    setVisible(v: boolean): void {
      visible = v;
      if (v) {
        // Walking back into the editor after a fight re-shows the last replay;
        // loaders re-read saved teams that may have changed elsewhere.
        refreshLoaderOptions("A");
        refreshLoaderOptions("B");
        renderBoth();
        if (fought && !mounted) {
          els.mount.append(deps.viewerHost);
          deps.viewerHost.hidden = false;
          mounted = true;
        }
      } else {
        unmountViewer();
        palette.close();
      }
    },
  };
}
