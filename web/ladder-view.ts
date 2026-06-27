// Ladder view — the champion chase made visible. The champion team sits at
// the top, then every round's pool of ghosts, expandable down to a unit's
// abilities: the same cards and the same derived-description inspector the
// shop uses, so a ghost reads exactly like a shop offer. Display only — it
// owns zero rules and never writes the store; refresh() re-reads it, so the
// pools visibly fill as runs play.

import {
  BOOTSTRAP_RUN_ID,
  type AbilityRegistry,
  type LadderStore,
  type StatusRegistry,
  type TeamSnapshot,
  type UnitDef,
} from "../src/index.js";
import { closeInspectOverlay, openInspectOverlay, renderUnitInspect } from "./inspect.js";
import { unitCardHtml } from "./unit-card.js";

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

/** Rounds scanned past the last non-empty pool — pools are contiguous from
 * round 1 (every run fights every round on its way up), so 1 is enough; the
 * cap only guards against a hand-corrupted store looping forever. */
const ROUND_SCAN_CAP = 200;

export interface LadderViewDeps {
  store: LadderStore;
  registry: StatusRegistry;
  /** The ability registry a unit's `ability` ref resolves through (PRD #081). */
  abilities: AbilityRegistry;
  /** Start with round 1's pool expanded — the leaderboard screen (#015
   * slice 4) opens showing teams, not a list of closed drawers; the run
   * screen's side ladder keeps its compact default. */
  openFirstRound?: boolean;
  /** The champion holder's display name, when the backing knows one (#016
   * slice 3: the shared ladder names its players; local backings don't). */
  holderName?: () => string | null;
}

/** The active run, when one is in the shop — marks "you are here" and "yours". */
export interface LadderViewRun {
  round: number;
  runId: string;
}

export interface LadderView {
  /** Re-read the store and re-render; `run` marks the active run's row and ghosts. */
  refresh(run?: LadderViewRun): void;
}

/** A selected unit: the champion's (round < 0) or a ghost's, by pool address. */
interface Selection {
  champ: boolean;
  round: number;
  seq: number;
  unit: number;
  status?: string;
}

/** A ghost's display name: bootstrap ghosts are shipped content, not a run.
 * Shared-ladder runIds are minted globally unique (#016: `web-<uuid>`) and
 * read as noise past a prefix — the label keeps enough to tell runs apart. */
export function ghostLabel(runId: string): string {
  if (runId === BOOTSTRAP_RUN_ID) return "shipped";
  return runId.length > 20 ? `${runId.slice(0, 12)}…` : runId;
}

export function createLadderView(root: HTMLElement, deps: LadderViewDeps): LadderView {
  const open = new Set<number>(); // expanded rounds survive re-renders
  if (deps.openFirstRound === true) open.add(1);
  let sel: Selection | undefined;
  let run: LadderViewRun | undefined;

  function unitCard(u: UnitDef, addr: string, i: number, selected: boolean): string {
    const level = u.level ?? 1;
    return unitCardHtml({
      artName: u.name,
      label: u.name,
      hp: u.base.hp,
      pwr: u.base.pwr,
      statuses: u.statuses,
      registry: deps.registry,
      ...(level > 1 ? { level } : {}),
      sel: selected,
      classes: "run-card lv-unit",
      attrs: `data-lv="${addr}:${i}"`,
      title: u.name,
    });
  }

  function teamRow(snap: TeamSnapshot, addr: string, isChamp: boolean): string {
    const cards = snap.team
      .map((u, i) =>
        unitCard(
          u,
          addr,
          i,
          sel !== undefined && sel.champ === isChamp && sel.round === snap.round && sel.seq === snap.seq && sel.unit === i,
        ),
      )
      .join("");
    return `<div class="lv-team">${cards}</div>`;
  }

  function championHtml(): string {
    const champ = deps.store.champion();
    if (champ === null) {
      return `<div class="lv-champ"><span class="lv-k">champion</span> <span class="run-dim">the spot is vacant — the next crown is free</span></div>`;
    }
    const holder = deps.holderName?.() ?? null;
    const who =
      champ.runId === BOOTSTRAP_RUN_ID
        ? "the shipped champion — dethrone it to take the crown"
        : `${holder !== null ? `${esc(holder)} · ` : ""}${esc(ghostLabel(champ.runId))} — crowned at round ${champ.round}`;
    return `
      <div class="lv-champ">
        <div><span class="lv-k">👑 champion</span> <span class="lv-who">${who}</span></div>
        ${teamRow(champ, `c:${champ.round}:${champ.seq}`, true)}
      </div>`;
  }

  function roundsHtml(): string {
    const rows: string[] = [];
    for (let r = 1; r <= ROUND_SCAN_CAP; r++) {
      const pool = deps.store.poolAt(r);
      if (pool.length === 0) break;
      const here = run !== undefined && run.round === r;
      const head =
        `<button type="button" class="lv-round-head${here ? " here" : ""}" data-lvround="${r}">` +
        `<span class="lv-tri">${open.has(r) ? "▾" : "▸"}</span> round ${r} — ${pool.length} ghost${pool.length === 1 ? "" : "s"}` +
        `${here ? '<span class="lv-here">you fight here</span>' : ""}</button>`;
      const body = open.has(r)
        ? `<div class="lv-pool-body">` +
          pool
            .map((g) => {
              const yours = run !== undefined && g.runId === run.runId;
              return (
                `<div class="lv-ghost">` +
                `<span class="lv-gid">${esc(ghostLabel(g.runId))}${yours ? ' <span class="lv-you">(you)</span>' : ""}</span>` +
                teamRow(g, `g:${g.round}:${g.seq}`, false) +
                `</div>`
              );
            })
            .join("") +
          `</div>`
        : "";
      rows.push(`<div class="lv-round${here ? " here" : ""}">${head}${body}</div>`);
    }
    return rows.join("");
  }

  /** The selected unit looked up fresh from the store — pools only grow and
   * the champion only swaps, so a stale selection simply drops. */
  function selectedUnit(): { def: UnitDef; snap: TeamSnapshot } | undefined {
    if (sel === undefined) return undefined;
    const snap = sel.champ
      ? deps.store.champion()
      : (deps.store.poolAt(sel.round).find((g) => g.seq === sel!.seq) ?? null);
    if (snap === null || snap === undefined) return undefined;
    if (sel.champ && (snap.round !== sel.round || snap.seq !== sel.seq)) return undefined; // champion swapped
    const def = snap.team[sel.unit];
    return def === undefined ? undefined : { def, snap };
  }

  function render(): void {
    const found = selectedUnit();
    if (found === undefined) sel = undefined;
    root.innerHTML = championHtml() + `<div class="lv-rounds">${roundsHtml()}</div>`;
    if (found !== undefined && sel !== undefined) {
      const { def, snap } = found;
      const which = sel;
      const addr = `${which.champ ? "c" : "g"}:${which.round}:${which.seq}:${which.unit}`;
      openInspectOverlay("ladder", {
        anchor: root.querySelector<HTMLElement>(`[data-lv="${addr}"]`),
        onClose: () => {
          if (sel === undefined) return;
          sel = undefined;
          render();
        },
        render: (body) =>
          renderUnitInspect(body, {
            title: def.name,
            state:
              `${def.base.hp} hp · ${def.base.pwr} pwr` +
              ((def.level ?? 1) > 1 ? ` · L${def.level}` : "") +
              ` · ${which.champ ? "champion" : `ghost of ${esc(ghostLabel(snap.runId))}, round ${snap.round}`}`,
            def,
            statuses: def.statuses ?? [],
            registry: deps.registry,
            abilities: deps.abilities,
            ...(which.status !== undefined ? { highlight: which.status } : {}),
            noStatuses: "none to start with",
          }),
      });
    } else {
      closeInspectOverlay("ladder");
    }
  }

  root.addEventListener("click", (ev) => {
    const target = ev.target as HTMLElement;
    const head = target.closest("[data-lvround]");
    if (head) {
      const r = Number(head.getAttribute("data-lvround"));
      if (open.has(r)) open.delete(r);
      else open.add(r);
      render();
      return;
    }
    const card = target.closest("[data-lv]");
    if (!card) return;
    const [kind, round, seq, unit] = card.getAttribute("data-lv")!.split(":");
    const next: Selection = { champ: kind === "c", round: Number(round), seq: Number(seq), unit: Number(unit) };
    const chip = target.closest("[data-status]");
    if (chip) next.status = chip.getAttribute("data-status")!;
    const same =
      sel !== undefined &&
      sel.champ === next.champ &&
      sel.round === next.round &&
      sel.seq === next.seq &&
      sel.unit === next.unit &&
      next.status === undefined &&
      sel.status === undefined;
    sel = same ? undefined : next; // re-click closes, the shop inspector's way
    render();
  });

  return {
    refresh(r?: LadderViewRun): void {
      run = r;
      render();
    },
  };
}
