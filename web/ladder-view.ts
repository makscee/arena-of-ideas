// Ladder view — the champion chase made visible. The champion team sits at
// the top, then every round's pool of ghosts, expandable down to a unit's
// abilities: the same cards and the same derived-description inspector the
// shop uses, so a ghost reads exactly like a shop offer. Display only — it
// owns zero rules and never writes the store; refresh() re-reads it, so the
// pools visibly fill as runs play.

import {
  BOOTSTRAP_RUN_ID,
  FAMILY_HEX,
  type AbilityRegistry,
  type Family,
  type LadderStore,
  type StatusRegistry,
  type TeamSnapshot,
  type UnitDef,
} from "../src/index.js";
import { closeInspectOverlay, openInspectOverlay, renderUnitInspect } from "./inspect.js";
import { familySigil, nameFamily, unitCardHtml } from "./unit-card.js";

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
  /** Which face the view wears. `accordion` (default) is the run-screen's
   * champion-over-expandable-pools drawer; `tower` is the leaderboard's compact
   * Arena Tower floor-list (slice C, the mockup's strategy ladder). Both read the
   * same store; the tower is a presentation over `arenaTowerHtml`. */
  variant?: "accordion" | "tower";
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

// ---------- Arena Tower (slice C): the compact strategy-ladder floor list -----
// A reusable render over plain data (TowerRung[]), so the leaderboard screen AND
// the title hub can each drop the tower into their own container. The view below
// derives the rungs from the ladder store; the markup is this pure function.

/** A unit reduced to what a sigil chip needs — its name (chip title + inspect
 * anchor) and its ability family (the chip's colour). */
export interface TowerUnit {
  name: string;
  family: Family;
}

/** One rung of the tower: a team on the ladder, with its rank from the top
 * (1 = champion), the handle that holds it, and the champion / own-run flags
 * that drive the gold / teal treatment. */
export interface TowerRung {
  rank: number;
  handle: string;
  isChamp: boolean;
  isYou: boolean;
  /** The floor (round) the team was fielded at — carried for the inspect line. */
  round: number;
  units: TowerUnit[];
}

/** The Arena Tower markup (mockup line 130): a header, then one row per rung
 * (rank · family-coloured 22px sigil chips · @handle), highest rung — the
 * champion — first, then a plinth. Pure presentation over TowerRung[]: takes its
 * data, returns markup to drop into any container. Each chip carries
 * `data-tower-unit="rung:unit"` so a tap opens the shared unit inspector. */
export function arenaTowerHtml(rungs: readonly TowerRung[], sel?: { rung: number; unit: number }): string {
  const head =
    `<div class="tower-head">` +
    `<div><div class="tower-title">Arena Tower</div><div class="tower-eyebrow">Strategy ladder</div></div>` +
    `<div class="tower-hint">climb ▲</div>` +
    `</div>`;
  if (rungs.length === 0) {
    return `<div class="tower">${head}<div class="tower-empty">the tower is empty — the first crown is free</div></div>`;
  }
  const floors = rungs
    .map((rung, ri) => {
      const cls = ["tower-floor", rung.isChamp && "is-champ", rung.isYou && "is-you"].filter(Boolean).join(" ");
      const chips = rung.units
        .map((u, ui) => {
          const hex = FAMILY_HEX[u.family];
          const selected = sel !== undefined && sel.rung === ri && sel.unit === ui;
          return (
            `<button type="button" class="tower-chip${selected ? " sel" : ""}" style="--fam:${hex}" ` +
            `data-tower-unit="${ri}:${ui}" title="${esc(u.name)}" aria-label="${esc(u.name)}">` +
            familySigil(u.family, hex, "tower-sigil") +
            `</button>`
          );
        })
        .join("");
      return (
        `<div class="${cls}" data-round="${rung.round}">` +
        `<div class="tower-rank">${rung.rank}</div>` +
        `<div class="tower-sigils">${chips}</div>` +
        `<div class="tower-handle">${esc(rung.handle)}</div>` +
        `</div>`
      );
    })
    .join("");
  return `<div class="tower">${head}<div class="tower-floors">${floors}</div><div class="tower-plinth"></div></div>`;
}

export function createLadderView(root: HTMLElement, deps: LadderViewDeps): LadderView {
  const variant = deps.variant ?? "accordion";
  const open = new Set<number>(); // expanded rounds survive re-renders
  if (deps.openFirstRound === true) open.add(1);
  let sel: Selection | undefined;
  let run: LadderViewRun | undefined;
  // Tower-variant state: the chip the inspector is anchored to (rung/unit index
  // into the ordered rungs), and the ordered snapshots that index resolves
  // through (recomputed each render — pools only grow, so a stale index drops).
  let towerSel: { rung: number; unit: number } | undefined;

  /** A unit's family — its ability's family (PRD #081), the same axis the cards
   * colour by, with `nameFamily` as the pre-081 degrade so a chip always colours. */
  function unitFamily(u: UnitDef): Family {
    return deps.abilities[u.ability]?.family ?? nameFamily(u.name);
  }

  /** The tower's rungs, top (champion) first: the champion, then every ghost in
   * the pools ordered by floor (round) descending, then submission order. The
   * champion's own boss-ghost is de-duped out of the pool sweep so it appears
   * once, at the summit. */
  function orderedTowerSnaps(): TeamSnapshot[] {
    const champ = deps.store.champion();
    const ghosts: TeamSnapshot[] = [];
    for (let r = 1; r <= ROUND_SCAN_CAP; r++) {
      const pool = deps.store.poolAt(r);
      if (pool.length === 0) break; // pools are contiguous from round 1
      ghosts.push(...pool);
    }
    const isChampSnap = (g: TeamSnapshot): boolean =>
      champ !== null && g.runId === champ.runId && g.round === champ.round && g.seq === champ.seq;
    const rest = ghosts.filter((g) => !isChampSnap(g)).sort((a, b) => b.round - a.round || a.seq - b.seq);
    return champ !== null ? [champ, ...rest] : rest;
  }

  function towerRung(snap: TeamSnapshot, i: number): TowerRung {
    const isChamp = i === 0; // the summit is always the champion (snaps[0])
    const mine = run !== undefined && snap.runId === run.runId;
    const isYou = mine && !isChamp;
    const handle = isChamp
      ? `@${deps.holderName?.() ?? ghostLabel(snap.runId)}`
      : isYou
        ? "you"
        : `@${ghostLabel(snap.runId)}`;
    return {
      rank: i + 1,
      handle,
      isChamp,
      isYou,
      round: snap.round,
      units: snap.team.map((u) => ({ name: u.name, family: unitFamily(u) })),
    };
  }

  function renderTower(): void {
    const snaps = orderedTowerSnaps();
    if (towerSel !== undefined && snaps[towerSel.rung]?.team[towerSel.unit] === undefined) towerSel = undefined;
    root.innerHTML = arenaTowerHtml(
      snaps.map((snap, i) => towerRung(snap, i)),
      towerSel,
    );
    if (towerSel === undefined) {
      closeInspectOverlay("ladder");
      return;
    }
    const which = towerSel;
    const snap = snaps[which.rung]!;
    const def = snap.team[which.unit]!;
    openInspectOverlay("ladder", {
      anchor: root.querySelector<HTMLElement>(`[data-tower-unit="${which.rung}:${which.unit}"]`),
      onClose: () => {
        if (towerSel === undefined) return;
        towerSel = undefined;
        renderTower();
      },
      render: (body) =>
        renderUnitInspect(body, {
          title: def.name,
          state:
            `${def.base.hp} hp · ${def.base.pwr} pwr` +
            ((def.level ?? 1) > 1 ? ` · L${def.level}` : "") +
            ` · ${which.rung === 0 ? "champion" : `floor ${snap.round}`}`,
          def,
          statuses: def.statuses ?? [],
          registry: deps.registry,
          abilities: deps.abilities,
          noStatuses: "none to start with",
        }),
    });
  }

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
    if (variant === "tower") {
      renderTower();
      return;
    }
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
    if (variant === "tower") {
      const chip = target.closest("[data-tower-unit]");
      if (!chip) return;
      const [rung, unit] = chip.getAttribute("data-tower-unit")!.split(":").map(Number);
      const same = towerSel !== undefined && towerSel.rung === rung && towerSel.unit === unit;
      towerSel = same ? undefined : { rung: rung!, unit: unit! }; // re-click closes
      renderTower();
      return;
    }
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
