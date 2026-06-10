// Run screen — the shop/fight loop in the browser, over the kernel's run
// layer. It owns zero rules: every transition is initRun/buy/reroll/reorder/
// ladderFight, every battle is recomputed from its logged seed, and the whole
// thing persists through run-store.ts so an abandoned run resumes on reload —
// mid-shop or mid-battle-replay alike. The battle itself renders in the one
// existing viewer: its DOM (#result) is reparented in here while a run battle
// shows, and returned to the battle tab after.
//
// The champion chase reads off the ladder view (ladder-view.ts): pools and
// the champion, refreshed on every render so a fight's ghost shows at once.

import {
  BOOTSTRAP_RUN_ID,
  InvalidDecisionError,
  REROLL_COST,
  STACK_THRESHOLD,
  UNIT_COST,
  battle,
  buy,
  initRun,
  ladderFight,
  reorder,
  reroll,
  toBattleTeam,
  type LadderStore,
  type RunEvent,
  type RunState,
  type RunUnit,
  type StatusRegistry,
  type TeamSnapshot,
  type UnitDef,
} from "../src/index.js";
import { shapeSvg } from "./board-render.js";
import { renderUnitInspect } from "./inspect.js";
import { createLadderView, ghostLabel } from "./ladder-view.js";
import { clearRun, loadRun, nextRunId, saveRun, type KVStorage, type StoredBattle } from "./run-store.js";
import type { Viewer } from "./viewer.js";

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

const ofType = <T extends RunEvent["type"]>(log: readonly RunEvent[], t: T): Extract<RunEvent, { type: T }>[] =>
  log.filter((e): e is Extract<RunEvent, { type: T }> => e.type === t);

type Phase = "new" | "shop" | "battle" | "end";

interface RunScreenEls {
  newPanel: HTMLElement;
  newForm: HTMLFormElement;
  seed: HTMLInputElement;
  dice: HTMLButtonElement;
  newError: HTMLElement;
  champ: HTMLElement;
  warn: HTMLElement;
  shopPanel: HTMLElement;
  head: HTMLElement;
  next: HTMLElement;
  notice: HTMLElement;
  shopRow: HTMLElement;
  rerollButton: HTMLButtonElement;
  line: HTMLElement;
  fightButton: HTMLButtonElement;
  error: HTMLElement;
  inspect: HTMLElement;
  battlePanel: HTMLElement;
  battleHead: HTMLElement;
  battleMount: HTMLElement;
  outcome: HTMLElement;
  continueButton: HTMLButtonElement;
  endPanel: HTMLElement;
  endHead: HTMLElement;
  endStats: HTMLElement;
  endLine: HTMLElement;
  newRunButton: HTMLButtonElement;
  /** The ladder section (shown beside new/shop) and the view's render root. */
  ladderPanel: HTMLElement;
  ladderBody: HTMLElement;
}

export interface RunScreenDeps {
  storage: KVStorage;
  /** The localStorage-backed ladder, opened (bootstrap-seeded) by the caller. */
  store: LadderStore;
  /** The draftable pool new runs open with — a stable module-level object
   * (RunState holds it by reference; mutating it would desync stored runs). */
  pool: UnitDef[];
  /** Registry new runs fight with — threaded, never hardwired deeper. */
  registry: StatusRegistry;
  viewer: Viewer;
  /** The viewer's DOM (#result) — reparented here while a run battle shows. */
  viewerHost: HTMLElement;
  /** Where the viewer DOM lives otherwise (the battle view). */
  viewerHome: HTMLElement;
}

export interface RunScreen {
  /** The run tab was shown/hidden — mounts or returns the shared viewer DOM. */
  setVisible(visible: boolean): void;
}

export function createRunScreen(els: RunScreenEls, deps: RunScreenDeps): RunScreen {
  let state: RunState | undefined;
  let pending: StoredBattle | undefined; // the fought battle awaiting continue
  let phase: Phase = "new";
  let visible = false;
  let selected: { where: "offer" | "line"; index: number; status?: string } | undefined;
  let notice: string | undefined; // the last transition's level-up moment, if any

  const ladderView = createLadderView(els.ladderBody, { store: deps.store, registry: deps.registry });

  /** The champion as a phrase — bootstrap is shipped content, not a rival run. */
  const championPhrase = (c: TeamSnapshot): string =>
    c.runId === BOOTSTRAP_RUN_ID ? "the shipped champion" : `champion ${c.runId} (crowned at round ${c.round})`;

  // ---------- cards (reuse the board's card classes + shapes) ----------

  const chips = (statuses: readonly { status: string; stacks: number }[] | undefined): string =>
    (statuses ?? [])
      .map(
        (s) =>
          `<span class="chip" data-status="${esc(s.status)}" title="${esc(s.status)} ×${s.stacks}">${esc(s.status.slice(0, 3))}${s.stacks}</span>`,
      )
      .join("");

  function offerCard(def: UnitDef, i: number, gold: number): string {
    const sel = selected?.where === "offer" && selected.index === i;
    return `
      <div class="unit run-card${sel ? " sel" : ""}" data-offer="${i}" title="${esc(def.name)}">
        ${shapeSvg(def.name, false)}
        <span class="uname">${esc(def.name)}</span>
        <span class="unums"><span class="hp">${def.base.hp}</span><span class="pwr">${def.base.pwr}</span></span>
        <span class="chips">${chips(def.statuses)}</span>
        <button type="button" class="run-buy" data-buy="${i}"${gold < UNIT_COST ? " disabled" : ""}>buy ${UNIT_COST}g</button>
      </div>`;
  }

  function lineCard(u: RunUnit, i: number, last: number, buttons: boolean): string {
    const sel = selected?.where === "line" && selected.index === i;
    const stacks = u.stacks > 1 ? ` · ${u.stacks}/${STACK_THRESHOLD}` : "";
    const move = buttons
      ? `<span class="run-move">
          <button type="button" data-move="${i}:-1" title="Move toward the front"${i === 0 ? " disabled" : ""}>◂</button>
          <button type="button" data-move="${i}:1" title="Move toward the back"${i === last ? " disabled" : ""}>▸</button>
        </span>`
      : "";
    return `
      <div class="unit run-card${i === 0 ? " front" : ""}${sel ? " sel" : ""}" data-line="${i}" title="${esc(u.name)} — level ${u.level}, ${u.stacks}/${STACK_THRESHOLD} copies toward the next">
        ${i === 0 ? '<span class="front-tag">front</span>' : ""}
        ${shapeSvg(u.name, false)}
        <span class="uname">${esc(u.name)}</span>
        <span class="run-lvl">L${u.level}${stacks}</span>
        <span class="unums"><span class="hp">${u.base.hp}</span><span class="pwr">${u.base.pwr}</span></span>
        <span class="chips">${chips(u.def.statuses)}</span>
        ${move}
      </div>`;
  }

  // ---------- rendering ----------

  function show(which: Phase): void {
    phase = which;
    els.newPanel.hidden = which !== "new";
    els.shopPanel.hidden = which !== "shop";
    els.battlePanel.hidden = which !== "battle";
    els.endPanel.hidden = which !== "end";
    // The ladder rides along with the planning phases; a battle or an end
    // screen has its own focus. Refreshed on every show — pools visibly fill.
    els.ladderPanel.hidden = which === "battle" || which === "end";
    if (!els.ladderPanel.hidden) {
      ladderView.refresh(which === "shop" && state !== undefined ? { round: state.round, runId: state.runId } : undefined);
    }
    if (which === "battle") mountViewer();
    else unmountViewer();
  }

  function renderNew(): void {
    const champ = deps.store.champion();
    els.champ.textContent =
      champ === null
        ? "the champion spot is vacant — the first crown is free"
        : `holding the spot: ${championPhrase(champ)} — beat it to take the crown`;
    show("new");
  }

  function renderShop(): void {
    const s = state!;
    els.head.innerHTML =
      `<span class="run-round">round ${s.round}</span>` +
      `<span class="run-gold">${s.gold} gold</span>` +
      `<span class="run-lives">${s.lives} ${s.lives === 1 ? "life" : "lives"}</span>` +
      `<span class="run-id">${esc(s.runId)}</span>`;
    const rivals = deps.store.poolAt(s.round).filter((g) => g.runId !== s.runId).length;
    const champ = deps.store.champion();
    els.next.textContent =
      rivals > 0
        ? `next fight: a ghost from round ${s.round}'s pool — ${rivals} waiting (peek below)`
        : champ !== null
          ? `no ghosts left to fight at round ${s.round} — next fight challenges ${championPhrase(champ)} for the crown`
          : `no ghosts left at round ${s.round} and the spot is vacant — fighting takes the crown`;
    els.notice.textContent = notice ?? "";
    els.notice.hidden = notice === undefined;
    els.shopRow.innerHTML =
      s.offers.map((o, i) => offerCard(o, i, s.gold)).join("") ||
      '<span class="run-dim">the shop is empty — reroll or fight</span>';
    els.rerollButton.textContent = `reroll ${REROLL_COST}g`;
    els.rerollButton.disabled = s.gold < REROLL_COST;
    els.line.innerHTML =
      s.team.map((u, i) => lineCard(u, i, s.team.length - 1, true)).join("") ||
      '<span class="run-dim">no one yet — buy a unit</span>';
    els.fightButton.disabled = s.team.length === 0;
    els.inspect.hidden = selected === undefined;
    if (selected !== undefined) renderInspector();
    show("shop");
  }

  /** Inspector over a shop offer or line unit: the def's derived descriptions
   * (the same describe helpers the battle inspector uses) — players decide
   * buys by reading abilities, not by guessing from names. */
  function renderInspector(): void {
    const s = state!;
    const sel = selected!;
    const subject = sel.where === "offer" ? s.offers[sel.index] : s.team[sel.index];
    if (subject === undefined) {
      selected = undefined;
      els.inspect.hidden = true;
      return;
    }
    const def = "def" in subject ? subject.def : subject;
    const unit = "def" in subject ? subject : undefined;
    const base = unit?.base ?? def.base;
    renderUnitInspect(els.inspect, {
      title: def.name,
      state:
        `${base.hp} hp · ${base.pwr} pwr` +
        (unit !== undefined ? ` · L${unit.level}` : ` · ${UNIT_COST}g`),
      def,
      statuses: def.statuses ?? [],
      registry: s.statuses,
      ...(sel.status !== undefined ? { highlight: sel.status } : {}),
      noStatuses: "none to start with",
    });
  }

  function renderBattle(): void {
    const s = state!;
    const b = pending!;
    const fights = ofType(s.log, "FightFought");
    const last = fights[fights.length - 1]!;
    els.battleHead.textContent = `vs ${b.opponentLabel} — battle seed ${b.seed}`;
    els.outcome.textContent =
      last.winner === "A"
        ? "you won — no life lost"
        : last.winner === "B"
          ? `you lost — a life spent, ${s.lives} ${s.lives === 1 ? "life" : "lives"} left`
          : "a draw — no life lost";
    // Why the gold jumps on return: the new round's income rides the button.
    const started = ofType(s.log, "RoundStarted");
    const income = started[started.length - 1];
    els.continueButton.textContent =
      s.status === "over"
        ? s.endedBy === "crown"
          ? "claim the crown"
          : "see the end"
        : `continue to round ${s.round}${income !== undefined && income.round === s.round ? ` · +${income.income}g income` : ""}`;
    show("battle");
  }

  function renderEnd(): void {
    const s = state!;
    const crown = s.endedBy === "crown";
    els.endPanel.classList.toggle("crowned", crown);
    const dethroned = ofType(s.log, "Crowned")[0]?.dethroned;
    els.endHead.textContent = crown
      ? `👑 crowned at round ${s.round} — ` +
        (dethroned === undefined || dethroned === null
          ? "the spot was vacant; your team takes it"
          : dethroned === BOOTSTRAP_RUN_ID
            ? "the shipped champion falls; your team takes the spot"
            : `${dethroned} is dethroned; your team takes the spot`)
      : `out of lives at round ${s.round} — the run is over (your ghosts stay on the ladder)`;
    // The climb, derived from the run log: one fight per round, W/L/D per round.
    const fights = ofType(s.log, "FightFought");
    const won = fights.filter((f) => f.winner === "A").length;
    const lost = fights.filter((f) => f.winner === "B").length;
    const drawn = fights.length - won - lost;
    const marks = fights
      .map(
        (f) =>
          `<span class="end-mark ${f.winner === "A" ? "w" : f.winner === "B" ? "l" : "d"}" ` +
          `title="round ${f.round}: ${f.winner === "A" ? "won" : f.winner === "B" ? "lost" : "draw"}">${f.round}</span>`,
      )
      .join("");
    els.endStats.innerHTML =
      `<span class="end-record">${won}W/${lost}L/${drawn}D over ${fights.length} round${fights.length === 1 ? "" : "s"}` +
      `${crown ? ` · ${s.lives} ${s.lives === 1 ? "life" : "lives"} to spare` : ""}</span>` +
      `<span class="end-marks">${marks}</span>`;
    els.endLine.innerHTML = s.team.map((u, i) => lineCard(u, i, s.team.length - 1, false)).join("");
    show("end");
  }

  function render(): void {
    if (state === undefined) renderNew();
    else if (pending !== undefined) renderBattle();
    else if (state.status === "over") renderEnd();
    else renderShop();
  }

  // ---------- the shared viewer DOM ----------

  function mountViewer(): void {
    if (!visible || pending === undefined) return;
    els.battleMount.append(deps.viewerHost);
    deps.viewerHost.hidden = false;
    const s = state!;
    const log = battle({ teamA: pending.teamA, teamB: pending.teamB, seed: pending.seed, statuses: s.statuses });
    deps.viewer.load(log, { teams: { A: pending.teamA, B: pending.teamB }, registry: s.statuses });
  }

  function unmountViewer(): void {
    if (deps.viewerHost.parentElement !== els.battleMount) return;
    deps.viewer.stop();
    deps.viewerHost.hidden = true;
    deps.viewerHome.append(deps.viewerHost);
  }

  // ---------- transitions ----------

  function flag(message: string): void {
    els.error.textContent = message;
    els.error.hidden = false;
  }

  /** Apply a transition, persist, re-render; an InvalidDecisionError surfaces
   * on the error line (anything else propagates — it is a bug, not a play). */
  function transition(step: (s: RunState) => RunState): void {
    els.error.hidden = true;
    const before = state!.log.length;
    try {
      state = step(state!);
    } catch (err) {
      if (err instanceof InvalidDecisionError) {
        flag(err.message);
        return;
      }
      throw err;
    }
    // The fuse moment, surfaced: a level-up only changes a small "L2" badge,
    // so the shop says it happened — derived from the transition's own events.
    const ups = ofType(state.log.slice(before), "LeveledUp");
    const up = ups[ups.length - 1];
    notice =
      up === undefined
        ? undefined
        : `⬆ ${up.unit} fused ${STACK_THRESHOLD} copies into level ${up.level} — now ${up.hp} hp, ${up.pwr} pwr`;
    selected = undefined;
    saveRun(deps.storage, state, pending);
    render();
  }

  /** Fight the ladder, then reconstruct the battle for the viewer: the run log
   * records the battle seed and the drawn opponent; the teams pin it by value
   * (the champion may already be dethroned in the store by the time we look). */
  function fightLadder(): void {
    els.error.hidden = true;
    const before = state!;
    const championBefore = deps.store.champion();
    let next: RunState;
    try {
      next = ladderFight(before, deps.store);
    } catch (err) {
      if (err instanceof InvalidDecisionError) {
        flag(err.message);
        return;
      }
      throw err;
    }
    state = next;
    selected = undefined;
    notice = undefined;
    const fresh = next.log.slice(before.log.length);
    const fought = ofType(fresh, "FightFought")[0];
    if (fought === undefined) {
      // A vacant champion spot: crowned without a battle (slice-5 feel flag).
      pending = undefined;
      saveRun(deps.storage, next);
      render();
      return;
    }
    const drawn = ofType(fresh, "OpponentDrawn")[0];
    const opponent =
      drawn !== undefined
        ? deps.store.poolAt(before.round).find((g) => g.runId === drawn.opponent && g.seq === drawn.seq)!
        : championBefore!;
    pending = {
      teamA: toBattleTeam(before.team),
      teamB: opponent.team,
      seed: fought.battleSeed,
      opponentLabel:
        drawn !== undefined
          ? `the ghost of ${ghostLabel(opponent.runId)} from round ${before.round}`
          : championPhrase(opponent),
    };
    saveRun(deps.storage, next, pending);
    render();
  }

  // ---------- wiring ----------

  els.dice.addEventListener("click", () => {
    els.seed.value = String(Math.floor(Math.random() * 1_000_000));
    els.newError.hidden = true;
  });

  els.newForm.addEventListener("submit", (ev) => {
    ev.preventDefault();
    const raw = els.seed.value.trim();
    const seed = Number(raw);
    if (raw === "" || !Number.isInteger(seed) || seed < 0) {
      els.newError.textContent = "Seed must be a non-negative whole number.";
      els.newError.hidden = false;
      return;
    }
    els.newError.hidden = true;
    els.warn.hidden = true;
    state = initRun({ seed, runId: nextRunId(deps.storage), pool: deps.pool, statuses: deps.registry });
    pending = undefined;
    selected = undefined;
    notice = undefined;
    saveRun(deps.storage, state);
    render();
  });

  els.shopRow.addEventListener("click", (ev) => {
    const target = ev.target as HTMLElement;
    const buyBtn = target.closest("[data-buy]");
    if (buyBtn) {
      transition((s) => buy(s, Number(buyBtn.getAttribute("data-buy"))));
      return;
    }
    const card = target.closest("[data-offer]");
    if (!card) return;
    const index = Number(card.getAttribute("data-offer"));
    const chip = target.closest("[data-status]");
    if (chip) selected = { where: "offer", index, status: chip.getAttribute("data-status")! };
    else if (selected?.where === "offer" && selected.index === index && selected.status === undefined) selected = undefined;
    else selected = { where: "offer", index };
    renderShop();
  });

  els.line.addEventListener("click", (ev) => {
    const target = ev.target as HTMLElement;
    const moveBtn = target.closest("[data-move]");
    if (moveBtn) {
      const [from, dir] = moveBtn.getAttribute("data-move")!.split(":").map(Number) as [number, number];
      transition((s) => reorder(s, from, from + dir));
      return;
    }
    const card = target.closest("[data-line]");
    if (!card) return;
    const index = Number(card.getAttribute("data-line"));
    const chip = target.closest("[data-status]");
    if (chip) selected = { where: "line", index, status: chip.getAttribute("data-status")! };
    else if (selected?.where === "line" && selected.index === index && selected.status === undefined) selected = undefined;
    else selected = { where: "line", index };
    renderShop();
  });

  els.inspect.addEventListener("click", (ev) => {
    if (!(ev.target as HTMLElement).closest("#ins-close")) return;
    selected = undefined;
    renderShop();
  });

  els.rerollButton.addEventListener("click", () => transition(reroll));
  els.fightButton.addEventListener("click", fightLadder);

  els.continueButton.addEventListener("click", () => {
    pending = undefined;
    saveRun(deps.storage, state!);
    render();
  });

  els.newRunButton.addEventListener("click", () => {
    clearRun(deps.storage);
    state = undefined;
    pending = undefined;
    render();
  });

  // ---------- resume on load ----------

  try {
    const stored = loadRun(deps.storage);
    if (stored !== null) {
      state = stored.state;
      pending = stored.battle;
    }
  } catch (err) {
    // A corrupt stored run is refused loudly (never silently replayed wrong),
    // but it must not brick the screen: say so, offer a fresh start on top.
    els.warn.textContent = `Stored run could not be revived: ${(err as Error).message}. Starting a new run will overwrite it.`;
    els.warn.hidden = false;
  }
  render();

  return {
    setVisible(v: boolean): void {
      visible = v;
      if (v && phase === "battle") mountViewer();
      else if (!v) unmountViewer();
    },
  };
}
