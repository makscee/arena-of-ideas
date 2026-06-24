// Web shell — a thin, disposable client over the kernel's public API.
// It owns zero rules: pick teams, pick a seed, call battle(), hand the event
// log (and the content it ran on) to the battle screen — board, inline log,
// and inspector all read that one log.

import { DEFAULT_RUN_POOL, KERNEL_VERSION, battle, codexUnits, mergePool, openLadder, stressRegistry, type UnitDef } from "../src/index.js";
import { approvedUnits, committedApproved } from "./approved.js";
import { createArenaApi, type MeInfo } from "./api.js";
import { resolveUnits, teamOptions } from "./catalogue.js";
import { dismissInspectOverlay } from "./inspect.js";
import { createViewer } from "./viewer.js";
import { createBattleEditor, type BattleEditor } from "./battle-editor.js";
import { createGauntlet } from "./gauntlet.js";
import { createLogin } from "./login.js";
import { RemoteLadder } from "./remote-ladder.js";
import { createRunScreen, type RunScreen } from "./run-screen.js";
import {
  clearSession,
  loadDevMode,
  loadRun,
  loadSession,
  openLocalLadder,
  prefixedStorage,
  resetLadder,
  saveSession,
  setDevMode,
  type KVStorage,
} from "./run-store.js";
import { createCodex, type CodexScreen } from "./codex.js";
import { createLadderView, type LadderView, type LadderViewRun } from "./ladder-view.js";
import { createTitleScreen } from "./title-screen.js";

// ---------------------------------------------------------------------------
// DOM wiring
// ---------------------------------------------------------------------------

function el<T extends HTMLElement>(id: string): T {
  const node = document.getElementById(id);
  if (!node) throw new Error(`missing #${id}`);
  return node as T;
}

// The playable pool a new run drafts from: the shipped pool plus every approved
// creation-loop unit (PRD #013 slice 4). Computed once at load — the run screen
// and the codex both read it, so an approved unit is draftable AND catalogued.
const approved = approvedUnits();
const runPool = mergePool(DEFAULT_RUN_POOL, approved);

// ---------------------------------------------------------------------------
// Arena server session (#016 slice 3) — decided ONCE at boot: a stored token
// is verified against /v1/auth/me and the whole app wires remote (shared
// ladder, namespaced run storage) or local. Login/logout reload the page to
// re-run this — a mode switch, not a hot swap. With no stored token nothing
// here touches the network: logged-out play is byte-identical to before.
// ---------------------------------------------------------------------------

const api = createArenaApi();
const sessionToken = loadSession(window.localStorage);
let me: MeInfo | null = null;
let bootNetWarn: string | null = null;
if (sessionToken !== null) {
  const res = await api.me(sessionToken);
  if (res.ok) {
    me = res.value;
  } else if (res.kind === "unauthorized") {
    clearSession(window.localStorage); // dead token: a clean logged-out boot
  } else {
    bootNetWarn = "the arena server didn't answer — playing on this device's local ladder for now";
  }
}
let remote: RemoteLadder | null = null;
if (me !== null && sessionToken !== null) {
  const candidate = new RemoteLadder(api, sessionToken);
  const sync = await candidate.sync();
  if (sync.ok) remote = candidate;
  else bootNetWarn = `the shared ladder couldn't load (${sync.reason}) — playing locally this session`;
}
// Remote runs are pinned to the arena's committed content: the server rejects
// any other pool at submit, so the localStorage approved-override (a local
// playground affordance) joins local runs only.
const remoteRunPool = mergePool(DEFAULT_RUN_POOL, committedApproved());
// Logged-in runs live under namespaced keys: logging in never clobbers the
// local run, and a remote run never revives into a logged-out session.
const runStorage: KVStorage = remote !== null ? prefixedStorage(window.localStorage, "remote:") : window.localStorage;

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
    result.hidden = false; // visible before load — the board height lock measures real layout
    viewer.load(log, { teams: { A: teamA, B: teamB }, registry: stressRegistry });
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
// Views (#015 slice 3): title (the landing), run (the shop/fight loop),
// leaderboard (the ladder without a run), codex, and the dev tools — battle
// (pickers + replay), gauntlet (win-rate sweeps), editor — whose tab nav stays
// hidden until the title's "dev tools" entry reveals it. The editor refreshes
// both teams' pickers on every persisted change, so edited teams are always
// live everywhere. The run screen borrows the battle viewer's DOM while a run
// battle shows, so setVisible must hear every view change.
// ---------------------------------------------------------------------------

const views = {
  title: el<HTMLElement>("title-view"),
  settings: el<HTMLElement>("settings-view"),
  leaderboard: el<HTMLElement>("leaderboard-view"),
  run: el<HTMLElement>("run-view"),
  battle: el<HTMLElement>("battle-view"),
  gauntlet: el<HTMLElement>("gauntlet-view"),
  editor: el<HTMLElement>("editor-view"),
  codex: el<HTMLElement>("codex-view"),
};
const viewTabs = {
  run: el<HTMLButtonElement>("view-run"),
  battle: el<HTMLButtonElement>("view-battle"),
  gauntlet: el<HTMLButtonElement>("view-gauntlet"),
  editor: el<HTMLButtonElement>("view-editor"),
  codex: el<HTMLButtonElement>("view-codex"),
};
const viewsNav = el<HTMLElement>("views");
const homeButton = el<HTMLButtonElement>("home-button");
const titleDev = el<HTMLButtonElement>("title-dev");

// Dev-mode gate (#066 slice 1) — the dev surfaces (title's "dev tools" entry +
// the tab nav) are shown iff aoi.dev.v1 is on, hidden when off. Replaces the
// ungated #title-dev reveal: the gate is now the Settings toggle, with
// immediate effect. Off also tucks the nav away again, so toggling off while
// the nav happens to be revealed for the session re-hides it.
function reflectDevGate(): void {
  const on = loadDevMode(window.localStorage);
  titleDev.hidden = !on;
  if (!on) viewsNav.hidden = true;
}

let runScreen: RunScreen | undefined;
let leaderboardView: LadderView | undefined;
let battleEditor: BattleEditor | undefined;

// Codex: initialised once, lives in #codex-container. codexUnits() covers
// every unit a player can meet — shop pool, bootstrap ghosts/champion, summons.
const codexScreen: CodexScreen = createCodex(
  el("codex-container"),
  stressRegistry,
  codexUnits(approved),
);
codexScreen.setVisible(false);

// Deep-link navigation (e.g. #codex/status/Poison): handled on hash change,
// on cold load (the applyHashNav() call at the end of this module), and on
// every in-app codex-link click — the click handler navigates even when the
// hash is already the target, so a repeat click is never a dead click.
function applyHashNav(): void {
  const hash = window.location.hash.slice(1); // strip leading "#"
  if (hash.startsWith("codex/")) {
    showView("codex");
    codexScreen.navigate(hash);
  }
}
window.addEventListener("hashchange", applyHashNav);
document.addEventListener("click", (ev) => {
  const link = (ev.target as HTMLElement).closest<HTMLAnchorElement>('a[href^="#codex/"]');
  if (!link) return;
  ev.preventDefault(); // hashchange wouldn't fire when the hash is unchanged
  const fragment = link.getAttribute("href")!.slice(1);
  history.replaceState(null, "", `#${fragment}`);
  showView("codex");
  codexScreen.navigate(fragment);
});

/** The active climb, for the leaderboard's "you" markers — read from the
 * stored run (the same record the run screen revives from), not from the
 * ladder: the ladder store is #016's server-swap seam and knows nothing of
 * whose run is whose. A finished-but-undismissed run no longer climbs, so it
 * gets no marker; a corrupt stored run is the run screen's to surface. */
function activeRunMarker(): LadderViewRun | undefined {
  try {
    const stored = loadRun(runStorage);
    if (stored === null || stored.state.status !== "active") return undefined;
    return { round: stored.state.round, runId: stored.state.runId };
  } catch {
    return undefined;
  }
}

function showView(which: keyof typeof views): void {
  dismissInspectOverlay(); // an inspector never outlives its screen
  // Leaving the codex drops its deep-link hash (#015 slice 4 carry): a stale
  // #codex/... would otherwise put the NEXT reload back in the codex instead
  // of the title. replaceState, not location.hash = "" — no hashchange echo,
  // no extra history entry.
  if (which !== "codex" && window.location.hash.startsWith("#codex/")) {
    history.replaceState(null, "", window.location.pathname + window.location.search);
  }
  for (const key of Object.keys(views) as (keyof typeof views)[]) {
    views[key].hidden = key !== which;
  }
  for (const key of Object.keys(viewTabs) as (keyof typeof viewTabs)[]) {
    viewTabs[key].classList.toggle("active", key === which);
  }
  homeButton.hidden = which === "title"; // every other screen can walk home
  if (which !== "battle") viewer.stop();
  runScreen?.setVisible(which === "run");
  // The battle editor borrows the shared viewer DOM (#result) for its replay,
  // exactly as the run screen does — so it must hear every view change to mount
  // it on entry and return it on exit.
  battleEditor?.setVisible(which === "editor");
  codexScreen.setVisible(which === "codex");
  if (which === "title") {
    titleScreen.refresh(); // Play vs Continue, read fresh
    reflectDevGate(); // dev tools entry shown iff dev mode is on
  }
  if (which === "leaderboard") {
    leaderboardView?.refresh(activeRunMarker()); // pools fill live, own run marked
    // The shared ladder refreshes from the server on every show (#016 slice 3):
    // render what we have NOW, re-render when the sync lands — a dead server
    // costs freshness, never the screen.
    if (remote !== null) {
      void remote.sync().then((r) => {
        if (r.ok && !views.leaderboard.hidden) leaderboardView?.refresh(activeRunMarker());
      });
    }
  }
}
for (const key of Object.keys(viewTabs) as (keyof typeof viewTabs)[]) {
  viewTabs[key].addEventListener("click", () => showView(key));
}

// ---------------------------------------------------------------------------
// Title screen (#015 slice 3) — the landing. The Play entry reads the run
// screen's state at refresh time; dev tools hide behind one low-prominence
// entry that reveals the existing tab nav (it then works exactly as before).
// ---------------------------------------------------------------------------

const titleScreen = createTitleScreen(
  {
    ornament: el("title-ornament"),
    play: el<HTMLButtonElement>("title-play"),
  },
  {
    unitNames: runPool.map((u) => u.name),
    hasActiveRun: () => runScreen?.hasActiveRun() ?? false,
  },
);
el<HTMLButtonElement>("title-play").addEventListener("click", () => showView("run"));
el<HTMLButtonElement>("title-leaderboard").addEventListener("click", () => showView("leaderboard"));
el<HTMLButtonElement>("title-codex").addEventListener("click", () => showView("codex"));
// #016 slice 3: the login flow behind the title's Login entry. Reloads on
// success/logout — the session boot above re-decides the whole wiring.
createLogin(
  {
    loginButton: el<HTMLButtonElement>("title-login"),
    identity: el("title-id"),
    identityName: el("title-name"),
    logoutButton: el<HTMLButtonElement>("title-logout"),
    panel: el("login-panel"),
    form: el<HTMLFormElement>("login-panel"),
    blurb: el("login-blurb"),
    emailRow: el("login-email-row"),
    email: el<HTMLInputElement>("login-email"),
    codeRow: el("login-code-row"),
    code: el<HTMLInputElement>("login-code"),
    nameRow: el("login-name-row"),
    name: el<HTMLInputElement>("login-name"),
    submit: el<HTMLButtonElement>("login-submit"),
    cancel: el<HTMLButtonElement>("login-cancel"),
    error: el("login-error"),
  },
  {
    api,
    identity: me,
    token: sessionToken,
    saveToken: (t) => saveSession(window.localStorage, t),
    clearToken: () => clearSession(window.localStorage),
    reload: () => window.location.reload(),
  },
);
if (bootNetWarn !== null) {
  const warn = el("title-net-warn");
  warn.textContent = bootNetWarn;
  warn.hidden = false;
}
// #066 slice 2: the dev tools entry is gated by dev mode (reflectDevGate keeps
// it hidden when off). When shown, it reveals the tab nav and lands on the
// Battle Editor — the one dev surface (slice 2). The legacy battle/gauntlet
// tabs stay reachable for now (slice 5 deletes them).
titleDev.addEventListener("click", () => {
  viewsNav.hidden = false; // revealed for the session — tabs work as ever
  showView("editor");
});

// #066 slice 1: Settings (the ⚙ entry). The account block is the login state
// machine (wired above); the dev-mode toggle persists aoi.dev.v1 and takes
// effect the moment the player walks back to the title.
const settingsDevToggle = el<HTMLInputElement>("settings-dev-toggle");
el<HTMLButtonElement>("title-settings").addEventListener("click", () => {
  settingsDevToggle.checked = loadDevMode(window.localStorage); // reflect the stored state on open
  showView("settings");
});
settingsDevToggle.addEventListener("change", () => {
  setDevMode(window.localStorage, settingsDevToggle.checked);
});

homeButton.addEventListener("click", () => showView("title"));

// The run screen and the leaderboard share one ladder store: the shared
// server ladder when logged in (already synced above), the localStorage one
// otherwise. A failure to revive the stored ladder is loud (a silent fresh
// ladder would orphan its ghosts), but it must not take the other views down
// with it.
try {
  const ladderStore = remote ?? openLadder(openLocalLadder(window.localStorage), stressRegistry);
  leaderboardView = createLadderView(el("leaderboard-body"), {
    store: ladderStore,
    registry: stressRegistry,
    openFirstRound: true, // the screen opens showing teams, not closed drawers
    ...(remote !== null ? { holderName: () => remote!.holder() } : {}),
  });
  runScreen = createRunScreen(
    {
      newPanel: el("run-new"),
      newForm: el<HTMLFormElement>("run-new-form"),
      seed: el<HTMLInputElement>("run-seed"),
      dice: el<HTMLButtonElement>("run-seed-dice"),
      startButton: el<HTMLButtonElement>("run-start"),
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
      stakes: el("run-stakes"),
      error: el("run-error"),
      devPanel: el<HTMLDetailsElement>("run-dev"),
      devGoldPlus: el<HTMLButtonElement>("dev-gold-plus"),
      devGoldSetInput: el<HTMLInputElement>("dev-gold-set-input"),
      devGoldSet: el<HTMLButtonElement>("dev-gold-set"),
      devSpawnShop: el<HTMLButtonElement>("dev-spawn-shop"),
      devSpawnTeam: el<HTMLButtonElement>("dev-spawn-team"),
      devResetLadder: el<HTMLButtonElement>("dev-reset-ladder"),
      devResetConfirm: el("dev-reset-confirm"),
      devResetYes: el<HTMLButtonElement>("dev-reset-yes"),
      devResetNo: el<HTMLButtonElement>("dev-reset-no"),
      devNote: el("run-dev-note"),
      notice: el("run-notice"),
      battlePanel: el("run-battle"),
      battleHead: el("run-battle-head"),
      battleMount: el("run-battle-mount"),
      battleBar: el("run-battle-bar"),
      outcome: el("run-outcome"),
      continueButton: el<HTMLButtonElement>("run-continue"),
      skipButton: el<HTMLButtonElement>("run-skip"),
      endPanel: el("run-end"),
      endHead: el("run-end-head"),
      endStats: el("run-end-stats"),
      endLine: el("run-end-line"),
      endStatus: el("run-end-status"),
      newRunButton: el<HTMLButtonElement>("run-new-run"),
      ladderPanel: el("run-ladder"),
      ladderBody: el("run-ladder-body"),
      menuButton: el<HTMLButtonElement>("run-menu-button"),
      menuOverlay: el("run-menu-overlay"),
      menuClose: el<HTMLButtonElement>("run-menu-close"),
      abandonButton: el<HTMLButtonElement>("run-abandon"),
      abandonConfirm: el("run-abandon-confirm"),
      abandonYes: el<HTMLButtonElement>("run-abandon-yes"),
      abandonNo: el<HTMLButtonElement>("run-abandon-no"),
    },
    {
      storage: runStorage,
      store: ladderStore,
      pool: remote !== null ? remoteRunPool : runPool,
      devPool: () => codexUnits(approved), // #066 slice 4 spawn-any-unit — same pool as the editor's palette
      devEnabled: () => loadDevMode(window.localStorage), // device-wide gate, not the run's prefixed storage
      registry: stressRegistry,
      viewer,
      viewerHost: result,
      viewerHome: el("battle-view"),
      onExitToTitle: () => showView("title"), // abandon/run-end land here, reading "Play"
      ...(remote !== null ? { remote } : {}),
    },
  );
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

// The gauntlet (slice 3/5) still refreshes its picker on team changes; with the
// legacy editor gone the only remaining source is the saved-team store, kept
// fresh on each gauntlet show by its own refresh.
void (function keepGauntletFresh(): void {
  fillTeamPickers();
  gauntlet.refresh();
})();

// #066 slice 2: the Battle Editor — two team columns, a shared unit palette,
// per-slot overrides, quick loaders, and a locked-seed Fight that mounts the
// shared viewer (#result) as a black box. The pool the palette offers is the
// full set a player can meet: shipped run pool + approved + bootstrap + summons.
battleEditor = createBattleEditor(
  {
    columnA: el<HTMLElement>("be-col-a"),
    columnB: el<HTMLElement>("be-col-b"),
    seed: el<HTMLInputElement>("be-seed"),
    reroll: el<HTMLButtonElement>("be-reroll"),
    lock: el<HTMLInputElement>("be-lock"),
    fight: el<HTMLButtonElement>("be-fight"),
    runs: el<HTMLInputElement>("be-runs"),
    runN: el<HTMLButtonElement>("be-run-n"),
    band: el<HTMLElement>("be-band"),
    error: el<HTMLElement>("be-error"),
    mount: el<HTMLElement>("be-mount"),
  },
  {
    pool: () => codexUnits(approved),
    registry: stressRegistry,
    viewer,
    viewerHost: result,
    viewerHome: el("battle-view"),
  },
);

el<HTMLElement>("kernel-version").textContent = `kernel v${KERNEL_VERSION}`;

// The app opens on the title screen (#015 slice 3) — Play/Continue read from
// the freshly revived run state. Cold-load deep links override it: a shared
// #codex/... URL must land on the entry, not the title — routed FIRST, never
// through showView("title"), whose stale-hash sweep (slice 4) would eat the
// link before applyHashNav could read it. After every view is wired so
// showView can hide them all.
if (window.location.hash.startsWith("#codex/")) applyHashNav();
else showView("title");
