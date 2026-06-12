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
  INCOME_PER_ROUND,
  InvalidDecisionError,
  REROLL_COST,
  STACK_THRESHOLD,
  TEAM_SIZE,
  UNIT_COST,
  battle,
  buy,
  incomeForRound,
  initRun,
  ladderFight,
  reorder,
  reroll,
  serializeRun,
  shopSizeForRound,
  toBattleTeam,
  type LadderStore,
  type RunEvent,
  type RunState,
  type RunUnit,
  type StatusRegistry,
  type TeamSnapshot,
  type UnitDef,
} from "../src/index.js";
import { closeInspectOverlay, dismissInspectOverlay, openInspectOverlay, renderUnitInspect } from "./inspect.js";
import { createLadderView, ghostLabel } from "./ladder-view.js";
import { unitCardHtml } from "./unit-card.js";
import {
  clearRun,
  loadRun,
  loadSubmitResult,
  nextRunId,
  saveRun,
  saveSubmitResult,
  type KVStorage,
  type StoredBattle,
} from "./run-store.js";
import type { RemoteRun } from "./remote-ladder.js";
import type { Viewer } from "./viewer.js";

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

const ofType = <T extends RunEvent["type"]>(log: readonly RunEvent[], t: T): Extract<RunEvent, { type: T }>[] =>
  log.filter((e): e is Extract<RunEvent, { type: T }> => e.type === t);

// ---------- economy copy (IA-6) — pure, exported for the vitest suite ----------

/** Fusion pips on a line card: ● per copy held, ○ per copy still missing. */
export const fusionPips = (stacks: number): string =>
  "●".repeat(Math.min(stacks, STACK_THRESHOLD)) + "○".repeat(Math.max(0, STACK_THRESHOLD - stacks));

/** The shop header's income line — derived from the tunables' curve like the
 * codex is: a flat curve reads as one standing fact, a growing curve names
 * the next round's figure. Never a typed number. */
export const incomeLine = (round: number): string =>
  INCOME_PER_ROUND === 0
    ? `+${incomeForRound(round + 1)}g each round`
    : `+${incomeForRound(round + 1)}g next round`;

/** The stakes line at the fight button: what a loss costs, right now. */
export const stakesLine = (lives: number): string =>
  `a loss costs a life — ${lives} ${lives === 1 ? "life" : "lives"} left`;

/** Feel flag (PRD #012 slice 2 — a staging call, kept one boolean from
 * revert): false = the replay auto-plays and the outcome + continue stay
 * hidden until the playhead reaches the end (skip jumps straight there;
 * no spoiler at event 0 — audit GA-1). true = instant result: outcome and
 * continue shown from event 0, no autoplay. */
const INSTANT_RESULT = false;

type Phase = "new" | "shop" | "battle" | "end";

interface RunScreenEls {
  newPanel: HTMLElement;
  newForm: HTMLFormElement;
  seed: HTMLInputElement;
  dice: HTMLButtonElement;
  /** The form's submit button — disabled while a remote open is in flight. */
  startButton: HTMLButtonElement;
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
  stakes: HTMLElement;
  error: HTMLElement;
  battlePanel: HTMLElement;
  battleHead: HTMLElement;
  battleMount: HTMLElement;
  battleBar: HTMLElement;
  outcome: HTMLElement;
  continueButton: HTMLButtonElement;
  skipButton: HTMLButtonElement;
  endPanel: HTMLElement;
  endHead: HTMLElement;
  endStats: HTMLElement;
  endLine: HTMLElement;
  /** Shared-ladder submit verdict (#016 slice 3) — hidden for local runs. */
  endStatus: HTMLElement;
  newRunButton: HTMLButtonElement;
  /** The ladder section (shown beside new/shop) and the view's render root. */
  ladderPanel: HTMLElement;
  ladderBody: HTMLElement;
  /** Run menu (#014): a persistent control + an overlay with abandon behind a
   * two-step confirm. Both position: fixed — opening/closing never reflows. */
  menuButton: HTMLButtonElement;
  menuOverlay: HTMLElement;
  menuClose: HTMLButtonElement;
  abandonButton: HTMLButtonElement;
  abandonConfirm: HTMLElement;
  abandonYes: HTMLButtonElement;
  abandonNo: HTMLButtonElement;
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
  /** Called after the run is dropped (abandon, or the end screen's exit) —
   * the title screen is the landing now (#015 slice 3), so leaving a run
   * navigates there instead of squatting on the new-run form. */
  onExitToTitle?: () => void;
  /** The shared-ladder protocol (#016 slice 3) — present only when logged in.
   * With it, runs open before play, every ladder fight draws from a served
   * view, and a finished run is submitted for server-side re-derivation.
   * Without it, play is byte-identical to local-only: zero network. */
  remote?: RemoteRun;
}

export interface RunScreen {
  /** The run tab was shown/hidden — mounts or returns the shared viewer DOM. */
  setVisible(visible: boolean): void;
  /** Whether a run is in progress (including a finished one not yet
   * dismissed) — the title screen's Play/Continue label reads this. */
  hasActiveRun(): boolean;
}

export function createRunScreen(els: RunScreenEls, deps: RunScreenDeps): RunScreen {
  let state: RunState | undefined;
  let pending: StoredBattle | undefined; // the fought battle awaiting continue
  let phase: Phase = "new";
  let visible = false;
  let selected: { where: "offer" | "line" | "end"; index: number; status?: string } | undefined;
  let notice: string | undefined; // the last transition's level-up moment, if any
  let fused: string | undefined; // the just-fused unit's name — flashes once, consumed by the next shop render
  let revealed = false; // the pending battle's outcome has been shown (reset per fight)
  // The replay position the viewer held when it was last unmounted (#014): a
  // tab switch / menu open mid-battle unmounts the shared viewer, and the
  // re-mount must resume here, not reset to event 0. Undefined = start fresh
  // (a new fight, or the first mount of a reloaded pending battle).
  let battleResume: number | undefined;
  // The line's reserved card count, captured once per shop phase (keyed by
  // run + round): every card REACHABLE this phase, so mid-phase growth stays
  // inside the reserve and any height change lands on a phase render, where
  // scroll resets (slice-3 close). Gold only falls during a shop phase, so
  // the entry-time count is the phase's maximum.
  let lineReserve: { runId: string; round: number; count: number } | undefined;
  // The shared-ladder submit verdict for the current remote run (#016 slice 3).
  // "none" also covers local runs; "failed" (no server) may retry, the two
  // durable verdicts (accepted/rejected) never re-submit.
  let submitState:
    | { kind: "none" }
    | { kind: "pending" }
    | { kind: "accepted"; crowned: boolean }
    | { kind: "rejected"; reason: string }
    | { kind: "failed"; reason: string } = { kind: "none" };
  // The (runId, round) a shop-entry serve prefetch was already fired for —
  // once per round, so the "rivals waiting" line reads the served truth.
  let served: { runId: string; round: number } | undefined;

  const ladderView = createLadderView(els.ladderBody, { store: deps.store, registry: deps.registry });

  /** The champion as a phrase — bootstrap is shipped content, not a rival run. */
  const championPhrase = (c: TeamSnapshot): string =>
    c.runId === BOOTSTRAP_RUN_ID ? "the shipped champion" : `champion ${ghostLabel(c.runId)} (crowned at round ${c.round})`;

  // ---------- cards (the one shared unit card, run-screen flavoured) ----------

  function offerCard(def: UnitDef, i: number, gold: number): string {
    const sel = selected?.where === "offer" && selected.index === i;
    return unitCardHtml({
      artName: def.name,
      label: def.name,
      hp: def.base.hp,
      pwr: def.base.pwr,
      statuses: def.statuses,
      registry: state?.statuses ?? deps.registry,
      sel,
      classes: "run-card",
      attrs: `data-offer="${i}"`,
      title: `${def.name} — ${def.base.hp} hp, ${def.base.pwr} pwr · ${UNIT_COST}g · tap to inspect`,
      footer: `<button type="button" class="run-buy" data-buy="${i}"${gold < UNIT_COST ? " disabled" : ""}>buy ${UNIT_COST}g</button>`,
    });
  }

  function lineCard(u: RunUnit, i: number, last: number, buttons: boolean): string {
    const where = buttons ? "line" : "end";
    const sel = selected?.where === where && selected.index === i;
    const move = buttons
      ? `<span class="run-move">
          <button type="button" data-move="${i}:-1" title="Move toward the front"${i === 0 ? " disabled" : ""}>◂</button>
          <button type="button" data-move="${i}:1" title="Move toward the back"${i === last ? " disabled" : ""}>▸</button>
        </span>`
      : "";
    // Pips (IA-6): copies toward the next fuse, visible at a glance — the
    // title carries the words. The just-fused card flashes once (GA-7).
    return unitCardHtml({
      artName: u.name,
      label: u.name,
      hp: u.base.hp,
      pwr: u.base.pwr,
      statuses: u.def.statuses,
      registry: state?.statuses ?? deps.registry,
      level: u.level,
      pips: fusionPips(u.stacks),
      front: i === 0,
      sel,
      fused: u.name === fused,
      classes: "run-card",
      attrs: `data-line="${i}"`,
      title: `${u.name} — level ${u.level}, ${u.stacks}/${STACK_THRESHOLD} copies toward the next · tap to inspect`,
      footer: move,
    });
  }

  // ---------- rendering ----------

  function show(which: Phase): void {
    // A screen change takes the open inspector with it, whoever owns it.
    if (which !== phase) dismissInspectOverlay();
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
    // Place the menu button BEFORE mounting: in battle it docks into the bar,
    // so the reserveBattleBar() measure inside mountViewer() sees it (the bar
    // height stays honest even if the button ever changes it).
    syncMenuButton(); // the menu control rides the run, appearing/leaving with the phase
    if (which === "battle") mountViewer();
    else unmountViewer();
    // Phase transitions reset scroll so the new panel head is visible (LS-7,
    // GA-6): continue → shop lands on the gold/lives header, not mid-page;
    // run-end → end screen starts at top. Battle is handled in mountViewer()
    // via battlePanel.scrollIntoView. New-run leaves scroll wherever it is
    // (it's the first screen and always starts at the top anyway).
    if (which === "shop" || which === "end") window.scrollTo({ top: 0, behavior: "instant" });
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
    // Shown FIRST (slice-3 close): the reserves below measure real layout,
    // and a hidden panel measures 0 — a continue → shop render used to come
    // up with no reserve standing, so the next buy moved the fight button.
    // Mid-phase this is a no-op re-show; the scroll reset rides every shop
    // render either way, and nothing here paints between the two states.
    show("shop");
    els.head.innerHTML =
      `<span class="run-round">round ${s.round}</span>` +
      `<span class="run-gold">${s.gold} gold</span>` +
      `<span class="run-income">${esc(incomeLine(s.round))}</span>` +
      `<span class="run-lives">${s.lives} ${s.lives === 1 ? "life" : "lives"}</span>` +
      `<span class="run-id">${esc(s.runId)}</span>`;
    els.next.textContent = nextLine(s);
    prefetchServe(s);
    // Notice strip stays in flow at all times — content cleared when empty
    // so the reserved strip holds without showing stale text (LS-5). The
    // title carries the full string: at phone width the strip is a fixed
    // two-line clamp (slice-3 fix), so an over-long notice ellipsizes.
    els.notice.textContent = notice ?? "";
    els.notice.title = notice ?? "";
    renderShopRow(s);
    els.rerollButton.textContent = `reroll ${REROLL_COST}g`;
    els.rerollButton.disabled = s.gold < REROLL_COST;
    renderLine(s);
    els.fightButton.disabled = s.team.length === 0;
    els.stakes.textContent = stakesLine(s.lives);
    fused = undefined; // the flash renders once — re-renders must not replay it
    if (selected !== undefined) renderInspector();
    else closeInspectOverlay("run");
  }

  /** The "next fight" line — what the round's pool holds for THIS run. */
  function nextLine(s: RunState): string {
    const rivals = deps.store.poolAt(s.round).filter((g) => g.runId !== s.runId).length;
    const champ = deps.store.champion();
    return rivals > 0
      ? `next fight: a ghost from round ${s.round}'s pool — ${rivals} waiting (peek below)`
      : champ !== null
        ? `no ghosts left to fight at round ${s.round} — next fight challenges ${championPhrase(champ)} for the crown`
        : `no ghosts left at round ${s.round} and the spot is vacant — fighting takes the crown`;
  }

  /** Remote shops read the round's pool through the play endpoint (#016
   * slice 3), once per round: the public pool can't say which ghosts are the
   * player's own across runs, so the "N waiting" count is only honest off a
   * served (own-ghost-excluded) view. Re-reads are free server-side; only the
   * next-fight line re-renders, so a slow answer never yanks the scroll. */
  function prefetchServe(s: RunState): void {
    const remote = deps.remote;
    if (remote === undefined || s.status !== "active") return;
    if (served !== undefined && served.runId === s.runId && served.round === s.round) return;
    served = { runId: s.runId, round: s.round };
    void remote.serve(s.runId, s.round).then((r) => {
      if (!r.ok) {
        // Let a later shop render retry; the fight gate serves again anyway.
        if (served !== undefined && served.runId === s.runId && served.round === s.round) served = undefined;
        return;
      }
      if (state === undefined || state.runId !== s.runId || state.round !== s.round) return;
      if (phase === "shop" && pending === undefined) els.next.textContent = nextLine(state);
    });
  }

  /** Render the shop row and lock its min-height to the ROLLED offer count's
   * layout at the current width (slice-3 fix): three offers wrap to two rows
   * at 375px and a one-row min-height let buying collapse the row — the
   * fight button jumped −103px on the loop's most common action. The bought
   * slots are refilled with representative pool cards, the full rolled row is
   * measured invisibly within one task and removed (the reserveBattleBar()
   * pattern), and the measured height holds for the rest of the roll. The
   * measure is the fractional getBoundingClientRect height — offsetHeight
   * rounds down and the lost ~0.2px moved everything below on collapse.
   * Where layout doesn't run (height 0), the CSS one-row floor stands. */
  function renderShopRow(s: RunState): void {
    const real =
      s.offers.map((o, i) => offerCard(o, i, s.gold)).join("") ||
      '<span class="run-dim">the shop is empty — reroll or fight</span>';
    const fill = deps.pool.length === 0 ? 0 : Math.max(0, shopSizeForRound(s.round) - s.offers.length);
    const fillers = Array.from({ length: fill }, (_, k) =>
      offerCard(deps.pool[k % deps.pool.length]!, s.offers.length + k, s.gold),
    ).join("");
    els.shopRow.style.minHeight = "";
    els.shopRow.innerHTML = s.offers.map((o, i) => offerCard(o, i, s.gold)).join("") + fillers;
    const reserve = els.shopRow.getBoundingClientRect().height;
    els.shopRow.innerHTML = real;
    if (reserve > 0) els.shopRow.style.minHeight = `${reserve}px`;
  }

  /** Render the line and lock its min-height to the layout of every card
   * REACHABLE this shop phase (slice-3 close): the one-row CSS floor let the
   * third distinct buy wrap a new row — the fight button jumped +131px under
   * the tap at 375px. Reachable = min(TEAM_SIZE, units + gold/UNIT_COST),
   * captured at phase entry (gold only falls mid-phase, so that is the max;
   * a buy conserves it, a fuse or reroll only lowers it — the held reserve
   * always covers the row that growth can wrap). The renderShopRow() pattern:
   * fill to the reachable count with representative pool cards, measure the
   * fractional rect height invisibly within one task, hold it for the phase.
   * A small team with poor gold reserves only what it can reach — no burning
   * three rows while key info needs the fold. Where layout doesn't run
   * (height 0), the CSS one-row floor stands. */
  function renderLine(s: RunState): void {
    if (lineReserve === undefined || lineReserve.runId !== s.runId || lineReserve.round !== s.round) {
      lineReserve = {
        runId: s.runId,
        round: s.round,
        count: Math.min(TEAM_SIZE, s.team.length + Math.floor(s.gold / UNIT_COST)),
      };
    }
    const cards = s.team.map((u, i) => lineCard(u, i, s.team.length - 1, true)).join("");
    const real = cards || '<span class="run-dim">no one yet — buy a unit</span>';
    const fill = deps.pool.length === 0 ? 0 : Math.max(0, lineReserve.count - s.team.length);
    const last = s.team.length + fill - 1;
    const fillers = Array.from({ length: fill }, (_, k) => {
      const def = deps.pool[k % deps.pool.length]!;
      return lineCard({ name: def.name, base: { ...def.base }, level: 1, stacks: 1, def }, s.team.length + k, last, true);
    }).join("");
    els.line.style.minHeight = "";
    els.line.innerHTML = cards + fillers;
    const reserve = els.line.getBoundingClientRect().height;
    els.line.innerHTML = real;
    if (reserve > 0) els.line.style.minHeight = `${reserve}px`;
  }

  /** Inspector over a shop offer or line unit: the def's derived descriptions
   * (the same describe helpers the battle inspector uses) — players decide
   * buys by reading abilities, not by guessing from names. Shown in the one
   * overlay, pinned to the clicked card — never in the page flow. */
  function renderInspector(): void {
    const s = state!;
    const sel = selected!;
    const subject = sel.where === "offer" ? s.offers[sel.index] : s.team[sel.index];
    if (subject === undefined) {
      selected = undefined;
      closeInspectOverlay("run");
      return;
    }
    const def = "def" in subject ? subject.def : subject;
    const unit = "def" in subject ? subject : undefined;
    const base = unit?.base ?? def.base;
    const anchor =
      sel.where === "offer"
        ? els.shopRow.querySelector<HTMLElement>(`[data-offer="${sel.index}"]`)
        : (sel.where === "line" ? els.line : els.endLine).querySelector<HTMLElement>(`[data-line="${sel.index}"]`);
    openInspectOverlay("run", {
      anchor,
      onClose: () => {
        if (selected === undefined) return;
        selected = undefined;
        if (phase === "shop") renderShop();
        else if (phase === "end") renderEnd();
      },
      render: (body) =>
        renderUnitInspect(body, {
          title: def.name,
          state:
            `${base.hp} hp · ${base.pwr} pwr` +
            (unit !== undefined ? ` · L${unit.level}` : ` · ${UNIT_COST}g`),
          def,
          statuses: def.statuses ?? [],
          registry: s.statuses,
          ...(sel.status !== undefined ? { highlight: sel.status } : {}),
          noStatuses: "none to start with",
        }),
    });
  }

  /** The battle bar's staging: skip while the replay runs, outcome +
   * continue once it ends. Direct DOM toggles — no re-render, so revealing
   * never remounts (and restarts) the replay. */
  function syncBattleBar(): void {
    const shown = INSTANT_RESULT || revealed;
    els.outcome.hidden = !shown;
    els.continueButton.hidden = !shown;
    els.skipButton.hidden = shown;
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
    syncBattleBar();
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
    renderSubmitStatus();
    // Post-mortem (GA-5): the final team's cards open the same inspector the
    // shop uses — the pointer cursor they wear is a real affordance now.
    if (selected !== undefined) renderInspector();
    else closeInspectOverlay("run");
    show("end");
  }

  /** The shared-ladder verdict on the end screen (#016 slice 3) — honest in
   * every state: pending says so, a rejection says the run is NOT on the
   * ladder and why, a dead server offers a retry, and an accepted crown that
   * lapsed in the crown race is named, not glossed. Local runs show nothing. */
  function renderSubmitStatus(): void {
    const s = state;
    if (deps.remote === undefined || s === undefined) {
      els.endStatus.hidden = true;
      return;
    }
    els.endStatus.hidden = false;
    els.endStatus.classList.remove("ok", "bad");
    switch (submitState.kind) {
      case "none":
      case "pending":
        els.endStatus.textContent = "submitting this run to the shared ladder…";
        break;
      case "accepted":
        els.endStatus.classList.add("ok");
        els.endStatus.textContent =
          s.endedBy === "crown" && !submitState.crowned
            ? "your ghosts joined the shared ladder — but the champion you beat had already been dethroned, so the crown passed you by (the crown race)"
            : `your ghosts joined the shared ladder${submitState.crowned ? " — and the crown is yours" : " — they fight other players now"}`;
        break;
      case "rejected":
        els.endStatus.classList.add("bad");
        els.endStatus.textContent = `the server refused this run — it does not enter the shared ladder. Its reason: ${submitState.reason}`;
        break;
      case "failed":
        els.endStatus.classList.add("bad");
        els.endStatus.innerHTML =
          `couldn't reach the server — this run is not on the shared ladder yet (${esc(submitState.reason)}) ` +
          `<button type="button" id="run-submit-retry">try again</button>`;
        break;
    }
  }

  /** Submit the finished remote run for re-derivation. Fired the moment a run
   * ends and again from the retry button; the two durable verdicts persist so
   * a reload on the end screen neither re-submits nor forgets. */
  function submitRemote(): void {
    const remote = deps.remote;
    if (remote === undefined || state === undefined || state.status !== "over") return;
    if (submitState.kind !== "none" && submitState.kind !== "failed") return;
    const s = state;
    submitState = { kind: "pending" };
    void remote.submit(serializeRun(s)).then((r) => {
      if (state === undefined || state.runId !== s.runId) return; // run dropped while in flight
      if (r.ok) {
        submitState = { kind: "accepted", crowned: r.crowned };
        persistSubmit(s.runId, { accepted: true, crowned: r.crowned });
      } else if (r.kind === "rejected") {
        submitState = { kind: "rejected", reason: r.reason };
        persistSubmit(s.runId, { accepted: false, reason: r.reason });
      } else {
        submitState = { kind: "failed", reason: r.reason };
      }
      if (phase === "end") renderEnd();
    });
  }

  function persistSubmit(runId: string, verdict: { accepted: boolean; crowned?: boolean; reason?: string }): void {
    try {
      saveSubmitResult(deps.storage, { runId, ...verdict });
    } catch {
      // A quota failure only costs the don't-resubmit-after-reload guard;
      // the verdict on screen stands.
    }
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
    reserveBattleBar();
    const s = state!;
    const log = battle({ teamA: pending.teamA, teamB: pending.teamB, seed: pending.seed, statuses: s.statuses });
    // The replay is this phase's whole focus: start it at the top of the
    // viewport, so board + transport fit above the sticky bar at phone width.
    els.battlePanel.scrollIntoView({ block: "start" });
    deps.viewer.load(
      log,
      { teams: { A: pending.teamA, B: pending.teamB }, registry: s.statuses },
      {
        autoplay: !INSTANT_RESULT,
        // Resume where the player left off if this is a re-mount (#014 tab
        // switch); a fresh fight has no saved position and starts at the top.
        ...(battleResume !== undefined ? { resumeAt: battleResume } : {}),
        onEnded: () => {
          revealed = true;
          syncBattleBar();
        },
      },
    );
    battleResume = undefined; // consumed — the next unmount recaptures it
    // Nudge past the sticky bar: at 375px the auto-scrolled position left the
    // scrub 17px under it (slice-2 low) — scroll on until the whole transport
    // clears. The board top may slide off by the same few px; the controls win.
    const scrub = deps.viewerHost.querySelector<HTMLElement>("#scrub");
    if (scrub !== null) {
      const overlap = scrub.getBoundingClientRect().bottom - els.battleBar.getBoundingClientRect().top;
      if (overlap > 0) window.scrollBy(0, overlap + 8);
    }
  }

  /** Lock the battle bar at its revealed height before the replay starts:
   * both states (skip / outcome + continue) are measured invisibly within one
   * task and the max reserved, so the outcome reveal never grows the bar over
   * the transport (slice-2 low: 61→104px at 375px). */
  function reserveBattleBar(): void {
    const bar = els.battleBar;
    bar.style.minHeight = "";
    const stash = [els.skipButton.hidden, els.outcome.hidden, els.continueButton.hidden] as const;
    els.skipButton.hidden = false;
    els.outcome.hidden = true;
    els.continueButton.hidden = true;
    const skipState = bar.offsetHeight;
    els.skipButton.hidden = true;
    els.outcome.hidden = false;
    els.continueButton.hidden = false;
    const revealedState = bar.offsetHeight;
    [els.skipButton.hidden, els.outcome.hidden, els.continueButton.hidden] = stash;
    const max = Math.max(skipState, revealedState);
    if (max > 0) bar.style.minHeight = `${max}px`;
  }

  function unmountViewer(): void {
    if (deps.viewerHost.parentElement !== els.battleMount) return;
    // Capture the playhead before detaching, so the next mount resumes here
    // rather than resetting to event 0 (#014 tab-switch position preservation).
    if (phase === "battle" && pending !== undefined) {
      battleResume = deps.viewer.position();
      // And durably (#015 slice 4): a hard reload while parked on another
      // screen must still resume here — memory alone dies with the page.
      persist(state!, { ...pending, position: battleResume });
    }
    deps.viewer.stop();
    deps.viewerHost.hidden = true;
    deps.viewerHome.append(deps.viewerHost);
  }

  // ---------- transitions ----------

  function flag(message: string): void {
    // Error strip stays in flow (reserved, fixed-height at phone width) — set
    // text, no hidden toggle, so nothing below the fight button shifts (LS-5).
    // The title carries the full string past the phone strip's two-line clamp.
    els.error.textContent = message;
    els.error.title = message;
  }

  /** Persist the run; if localStorage is full, show a one-line warning and
   * carry on in-memory — the game is never blocked by a quota failure. */
  function persist(s: RunState, b?: StoredBattle): void {
    try {
      saveRun(deps.storage, s, b);
    } catch (err) {
      // QuotaExceededError (and any other write failure): warn once, keep playing.
      const reason = err instanceof Error ? err.message : String(err);
      els.warn.textContent = `progress is no longer being saved: ${reason}`;
      els.warn.hidden = false;
    }
  }

  /** Apply a transition, persist, re-render; an InvalidDecisionError surfaces
   * on the error line (anything else propagates — it is a bug, not a play). */
  function transition(step: (s: RunState) => RunState): void {
    els.error.textContent = ""; // clear in-flow error strip
    els.error.title = "";
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
    fused = up?.unit; // the card itself flashes too (GA-7)
    selected = undefined;
    persist(state, pending);
    render();
  }

  /** Fight the ladder. Remote runs (#016 slice 3) gate the fight behind the
   * play read: the server serves (and RECORDS) the round's view, the store
   * pins it, and only then does the kernel draw — submission replay accepts
   * only served views. A dead network refuses the fight with the reason and
   * changes nothing: the run state is untouched, the button comes back. */
  function fightLadder(): void {
    const remote = deps.remote;
    const s = state;
    if (remote === undefined || s === undefined) {
      doFightLadder();
      return;
    }
    els.error.textContent = "";
    els.error.title = "";
    els.fightButton.disabled = true;
    void remote.serve(s.runId, s.round).then((r) => {
      els.fightButton.disabled = false;
      // The run may have been abandoned (or already fought) while the serve
      // was in flight — a buy/reroll is fine, the view doesn't depend on it.
      if (state === undefined || state.runId !== s.runId || state.round !== s.round) return;
      if (state.status !== "active" || pending !== undefined) return;
      if (!r.ok) {
        flag(`the fight needs the server and it didn't answer: ${r.reason}`);
        return;
      }
      doFightLadder();
    });
  }

  /** The fight itself: draw from the store (for remote runs, the view the
   * serve above just pinned), then reconstruct the battle for the viewer: the
   * run log records the battle seed and the drawn opponent; the teams pin it
   * by value (the champion may already be dethroned in the store by the time
   * we look). */
  function doFightLadder(): void {
    els.error.textContent = ""; // clear in-flow error strip
    els.error.title = "";
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
    fused = undefined;
    const fresh = next.log.slice(before.log.length);
    const fought = ofType(fresh, "FightFought")[0];
    if (fought === undefined) {
      // A vacant champion spot: crowned without a battle (slice-5 feel flag).
      pending = undefined;
      persist(next);
      if (next.status === "over") submitRemote();
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
    revealed = false; // a fresh battle stages its outcome again
    persist(next, pending);
    // A run that just ended submits NOW, behind the replay — by the time the
    // player reaches the end screen the verdict is usually already in.
    if (next.status === "over") submitRemote();
    render();
  }

  // ---------- wiring ----------

  els.dice.addEventListener("click", () => {
    els.seed.value = String(Math.floor(Math.random() * 1_000_000));
    els.newError.hidden = true;
  });

  /** Start the run — shared tail of the local path and the remote open. */
  function startRun(seed: number, runId: string): void {
    els.warn.hidden = true;
    submitState = { kind: "none" };
    served = undefined;
    state = initRun({ seed, runId, pool: deps.pool, statuses: deps.registry });
    pending = undefined;
    selected = undefined;
    notice = undefined;
    persist(state);
    render();
  }

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
    const remote = deps.remote;
    if (remote === undefined) {
      startRun(seed, nextRunId(deps.storage));
      return;
    }
    // Remote runs open BEFORE play (the server's anti-forgery handshake, see
    // server/README.md): the run starts only once the server says so — a
    // refusal or dead network leaves the form standing with the reason, never
    // a half-open run. runIds are minted globally unique (`run-<n>` collides).
    const runId = remote.mintRunId();
    els.startButton.disabled = true;
    void remote.open(runId).then((r) => {
      els.startButton.disabled = false;
      if (state !== undefined) return; // a run appeared meanwhile — never clobber it
      if (!r.ok) {
        els.newError.textContent = `couldn't start an online run: ${r.reason}`;
        els.newError.hidden = false;
        return;
      }
      startRun(seed, runId);
    });
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

  // End-screen post-mortem: the final team's cards open the inspector (GA-5).
  els.endLine.addEventListener("click", (ev) => {
    const target = ev.target as HTMLElement;
    const card = target.closest("[data-line]");
    if (!card) return;
    const index = Number(card.getAttribute("data-line"));
    const chip = target.closest("[data-status]");
    if (chip) selected = { where: "end", index, status: chip.getAttribute("data-status")! };
    else if (selected?.where === "end" && selected.index === index && selected.status === undefined) selected = undefined;
    else selected = { where: "end", index };
    renderEnd();
  });

  els.rerollButton.addEventListener("click", () => transition(reroll));
  // The shop and line reserves are measured at the current width — a
  // rotation/resize re-measures in place (no full re-render: renderShop would
  // reset scroll, and phone browsers fire resize on every URL-bar collapse).
  // The line keeps its captured reachable count: a URL-bar resize after a
  // fuse must not shrink the reserve mid-phase.
  window.addEventListener("resize", () => {
    if (phase === "shop" && state !== undefined) {
      renderShopRow(state);
      renderLine(state);
    }
  });
  // Hard reload / tab close while the replay is on screen (#015 slice 4
  // carry): battleResume is captured on unmount, but a reload never unmounts —
  // write the live playhead into the stored battle on the way out. pagehide,
  // not beforeunload: it also covers bfcache entry and never blocks unload.
  window.addEventListener("pagehide", () => {
    if (state === undefined || pending === undefined) return;
    const position = visible && phase === "battle" ? deps.viewer.position() : battleResume;
    if (position === undefined) return;
    try {
      saveRun(deps.storage, state, { ...pending, position });
    } catch {
      // A quota failure on the way out has nowhere to surface — the run
      // itself was already persisted when the fight resolved.
    }
  });
  els.fightButton.addEventListener("click", fightLadder);
  els.skipButton.addEventListener("click", () => deps.viewer.toEnd()); // landing on the end reveals the bar via onEnded

  els.continueButton.addEventListener("click", () => {
    pending = undefined;
    revealed = false;
    persist(state!);
    render();
  });

  /** Drop the active run, stored run cleared — the seam both the end screen's
   * exit and the menu's "abandon" share. Ghosts already snapshotted into the
   * ladder stay (the snapshot is taken before each fight); an abandoned run
   * simply never reaches a crown. The screen resets to the new-run phase
   * (so the next Play opens on the seed form), then hands navigation to the
   * title (#015 slice 3) — which re-reads hasActiveRun() and shows "Play". */
  function clearActiveRun(): void {
    clearRun(deps.storage);
    state = undefined;
    pending = undefined;
    selected = undefined;
    notice = undefined;
    fused = undefined;
    battleResume = undefined;
    submitState = { kind: "none" };
    served = undefined;
    render();
    deps.onExitToTitle?.();
  }

  els.newRunButton.addEventListener("click", clearActiveRun);

  // Submit retry (#016 slice 3): only the "failed" state renders the button,
  // and submitRemote() re-fires only from there — a stray click is inert.
  els.endStatus.addEventListener("click", (ev) => {
    if (!(ev.target as HTMLElement).closest("#run-submit-retry")) return;
    submitRemote();
    if (phase === "end") renderEnd();
  });

  // ---------- run menu (#014) ----------

  /** The menu rides every in-run phase (shop/battle/end); the new-run screen
   * is its own start point and needs no abandon. Driven off the phase so the
   * button appears/disappears with the run, not the tab. */
  // The menu button's out-of-battle home: a fixed bottom-right control, the
  // same parent it ships in (#run-view). Captured once so dock/undock returns
  // it exactly where it started.
  const menuButtonHome = els.menuButton.parentElement!;

  /** Show the menu button by phase, and place it so it occludes nothing by
   * construction (Cass #014 round-2 finding): in battle it docks INSIDE the
   * sticky battle bar, right-aligned (skip/continue are left-aligned, the bar
   * owns the bottom edge — a control inside the bar can never steal the
   * transport's surface at any scroll offset). Out of battle there is no bar,
   * so it returns to its fixed bottom-right home. Docking happens before the
   * reserveBattleBar() measure in mountViewer(), so the reserve sees the
   * button's real contribution to bar height. */
  function syncMenuButton(): void {
    els.menuButton.hidden = state === undefined || phase === "new";
    const dock = phase === "battle" ? els.battleBar : menuButtonHome;
    if (els.menuButton.parentElement !== dock) dock.append(els.menuButton);
    els.menuButton.classList.toggle("in-bar", phase === "battle");
  }

  function closeMenu(): void {
    els.menuOverlay.hidden = true;
    els.menuButton.setAttribute("aria-expanded", "false");
    els.abandonConfirm.hidden = true; // a re-open starts at step one — no armed confirm
  }

  function openMenu(): void {
    els.abandonConfirm.hidden = true;
    els.menuOverlay.hidden = false;
    els.menuButton.setAttribute("aria-expanded", "true");
  }

  els.menuButton.addEventListener("click", () => {
    if (els.menuOverlay.hidden) openMenu();
    else closeMenu();
  });
  els.menuClose.addEventListener("click", closeMenu);
  // Outside tap and Escape close the menu (the overlay's own discipline, like
  // the inspector's) — never a half-open state left behind.
  document.addEventListener("click", (ev) => {
    if (els.menuOverlay.hidden) return;
    const target = ev.target as Node;
    if (els.menuOverlay.contains(target) || els.menuButton.contains(target)) return;
    closeMenu();
  });
  document.addEventListener("keydown", (ev) => {
    if (ev.key === "Escape" && !els.menuOverlay.hidden) closeMenu();
  });

  // Two-step confirm (a single click never destroys a run): the first tap arms
  // the confirm, the second abandons.
  els.abandonButton.addEventListener("click", () => {
    els.abandonConfirm.hidden = false;
  });
  els.abandonNo.addEventListener("click", () => {
    els.abandonConfirm.hidden = true;
  });
  els.abandonYes.addEventListener("click", () => {
    closeMenu();
    clearActiveRun(); // stored run cleared, lands on the title reading "Play"
  });

  // ---------- resume on load ----------

  try {
    const stored = loadRun(deps.storage);
    if (stored !== null) {
      state = stored.state;
      pending = stored.battle;
      // The parked replay position, revived (#015 slice 4) — fed through the
      // same resumeAt seam a tab switch uses; clamping is the viewer's job.
      if (typeof stored.battle?.position === "number") battleResume = stored.battle.position;
      // A finished remote run revives with its stored verdict; without one
      // (the reload beat the submit) it submits now — runIds are one-shot
      // server-side, so a race here resolves to one accepted copy.
      if (deps.remote !== undefined && state.status === "over") {
        const verdict = loadSubmitResult(deps.storage, state.runId);
        if (verdict !== null) {
          submitState = verdict.accepted
            ? { kind: "accepted", crowned: verdict.crowned ?? false }
            : { kind: "rejected", reason: verdict.reason ?? "the server refused this run" };
        } else {
          submitRemote();
        }
      }
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
      // The fixed menu button (and its overlay) must not float over OTHER tabs:
      // hide it (and close any open menu) when the run tab is hidden, restore it
      // by phase when the run tab returns (#014).
      if (!v) {
        els.menuButton.hidden = true;
        closeMenu();
      } else {
        syncMenuButton();
      }
      // The reserves measure 0 while the tab is display:none (the initial
      // render precedes the first show) — re-measure the moment layout is
      // real, so the line's reachable-count reserve is standing BEFORE the
      // first mid-phase tap, never applied by one (slice-3 close).
      if (v && phase === "shop" && state !== undefined) {
        renderShopRow(state);
        renderLine(state);
      }
    },
    hasActiveRun(): boolean {
      return state !== undefined;
    },
  };
}
