// Web shell — a thin, disposable client over the kernel's public API.
// It owns zero rules: pick teams, pick a seed, call battle(), hand the event
// log (and the content it ran on) to the battle screen — board, inline log,
// and inspector all read that one log.

import { DEFAULT_RUN_POOL, KERNEL_VERSION, codexUnits, mergePool, openEmptyLadder, seedBootstrapTower, stressAbilities, stressRegistry } from "../src/index.js";
import { approvedUnits } from "./approved.js";
import { createArenaApi, type ContentInfo, type MeInfo } from "./api.js";
import { dismissInspectOverlay } from "./inspect.js";
import { createViewer } from "./viewer.js";
import { createBattleEditor, type BattleEditor } from "./battle-editor.js";
import { createLogin } from "./login.js";
import { RemoteLadder } from "./remote-ladder.js";
import { createRunScreen, type RunScreen } from "./run-screen.js";
import {
  clearSession,
  loadDevMode,
  loadRun,
  loadSession,
  openLocalArchive,
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
import { RemoteIdeas } from "./remote-ideas.js";
import { createIdeasScreen, ideasLadderHtml, type IdeasScreen } from "./ideas-screen.js";
import { createHistoryScreen, type HistoryScreen } from "./history-screen.js";

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
let serverContent: ContentInfo | null = null;
if (me !== null && sessionToken !== null) {
  const contentResult = await api.content();
  if (!contentResult.ok) {
    bootNetWarn = `the active season content couldn't load — playing locally this session`;
  } else {
    serverContent = contentResult.value;
    const candidate = new RemoteLadder(api, sessionToken, contentResult.value.contentVersion);
    const sync = await candidate.sync();
    if (sync.ok) remote = candidate;
    else bootNetWarn = `the shared ladder couldn't load (${sync.reason}) — playing locally this session`;
  }
}
// Authenticated play consumes the server's canonical active payload. The
// localStorage playground override remains local/offline only.
const remoteRunPool = serverContent?.pool ?? runPool;
const activeStatuses = remote !== null && serverContent !== null ? serverContent.statuses : stressRegistry;
const activeAbilities = remote !== null && serverContent !== null ? serverContent.abilities : stressAbilities;
const activeSeason = remote !== null && serverContent !== null ? { season: serverContent.season, contentVersion: serverContent.contentVersion } : null;
// Logged-in runs live under namespaced keys: logging in never clobbers the
// local run, and a remote run never revives into a logged-out session.
const runStorage: KVStorage = remote !== null ? prefixedStorage(window.localStorage, "remote:") : window.localStorage;

// #battle-view is no longer a navigable surface (#066 slice 5 deleted the inline
// picker); it survives only as the home for #result, the shared viewer DOM the
// run screen and the Battle Editor reparent into their own mounts.
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
});

// ---------------------------------------------------------------------------
// Views (#015 slice 3): title (the landing), run (the shop/fight loop),
// leaderboard (the ladder without a run), codex, and the one dev surface —
// the Battle Editor, reached via the dev-gated title entry. #066 slice 5
// removed the legacy dev surfaces (inline battle picker, gauntlet view, and the
// tab-nav reveal); #battle-view stays only as the home for the shared viewer
// DOM (#result), which the run screen and the editor reparent. The run screen
// borrows that viewer DOM while a run battle shows, so setVisible must hear
// every view change.
// ---------------------------------------------------------------------------

const views = {
  title: el<HTMLElement>("title-view"),
  settings: el<HTMLElement>("settings-view"),
  leaderboard: el<HTMLElement>("leaderboard-view"),
  run: el<HTMLElement>("run-view"),
  battle: el<HTMLElement>("battle-view"),
  editor: el<HTMLElement>("editor-view"),
  codex: el<HTMLElement>("codex-view"),
  ideas: el<HTMLElement>("ideas-view"),
  history: el<HTMLElement>("history-view"),
};
const homeButton = el<HTMLButtonElement>("home-button");
const titleDev = el<HTMLButtonElement>("title-dev");

// Dev-mode gate (#066 slice 1) — the dev entry (the title's "dev tools" button,
// the only door to the Battle Editor) is shown iff aoi.dev.v1 is on, hidden when
// off. The gate is the Settings toggle, with immediate effect on every title
// show. With the tab nav deleted (#066 slice 5), this is the whole gate.
function reflectDevGate(): void {
  titleDev.hidden = !loadDevMode(window.localStorage);
}

let runScreen: RunScreen | undefined;
let leaderboardView: LadderView | undefined;
let battleEditor: BattleEditor | undefined;
let ideasScreen: IdeasScreen | undefined;
let historyScreen: HistoryScreen | undefined;
// The title hub owns a read-only Ideas synopsis and the live tower render.
let hubTower: LadderView | undefined;

// Codex: initialised once, lives in #codex-container. codexUnits() covers
// every unit a player can meet — shop pool, bootstrap ghosts/champion, summons.
const codexScreen: CodexScreen = createCodex(
  el("codex-container"),
  activeStatuses,
  codexUnits(remote !== null ? remoteRunPool : approved, activeAbilities),
  activeAbilities,
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
  homeButton.hidden = which === "title"; // every other screen can walk home
  if (which !== "battle") viewer.stop();
  runScreen?.setVisible(which === "run");
  // The battle editor borrows the shared viewer DOM (#result) for its replay,
  // exactly as the run screen does — so it must hear every view change to mount
  // it on entry and return it on exit.
  battleEditor?.setVisible(which === "editor");
  codexScreen.setVisible(which === "codex");
  if (which === "title") {
    titleScreen.refresh(); // New Run always, Continue gated on an active run
    reflectDevGate(); // dev tools entry shown iff dev mode is on
    refreshHub(); // the live ideas ladder + Arena Tower in the hub columns
  }
  if (which === "ideas") {
    // Re-pull the public table on every show — list() is the source of truth
    // for ranking, so a vote/submit made elsewhere (or by another player) is
    // reflected the moment the screen opens.
    void ideasScreen?.refresh();
  }
  if (which === "history") {
    // Re-read the local archive on every show — a season that ended since the
    // last visit appears the moment the screen opens (and we land on the list,
    // not a stale season detail).
    historyScreen?.refresh();
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

/** Re-render the hub's live columns (B·Arena slice B) and the Strategy/Creation
 * counters. Called on every title show so the landing is never stale. The
 * counters are derived from what the columns actually rendered — the tower's
 * floor count is the Strategy number, the ideas count is the Creation number —
 * so they never drift from the live data (never a hardcoded figure). */
function refreshHub(): void {
  hubTower?.refresh(activeRunMarker());
  const floors = el("hub-tower-body").querySelectorAll(".tower-floor").length;
  el("hub-strategy").textContent = String(floors);
  el("hub-season").textContent = activeSeason === null
    ? "Local play · committed content"
    : `Season ${activeSeason.season} · content v${activeSeason.contentVersion}`;
  const list = el("hub-ideas-list");
  if (me === null) {
    list.innerHTML = '<p class="ideas-empty">Open Ideas to read the public governance table.</p>';
    return;
  }
  void api.listIdeas().then((result) => {
    if (!result.ok) {
      list.innerHTML = '<p class="ideas-empty">Ideas are unavailable right now.</p>';
      return;
    }
    list.innerHTML = ideasLadderHtml(result.value.ideas.slice(0, 4), { userId: me?.userId ?? null, mode: "top", actionable: false });
  });
}

// ---------------------------------------------------------------------------
// Title screen (#015 slice 3) — the landing. The Play entry reads the run
// screen's state at refresh time; the dev tools hide behind one low-prominence,
// dev-gated entry that opens the Battle Editor (#066 slice 5: the one dev door).
// ---------------------------------------------------------------------------

const titleScreen = createTitleScreen(
  {
    ornament: el("title-ornament"),
    newRun: el<HTMLButtonElement>("title-play"),
    continueRun: el<HTMLButtonElement>("title-continue"),
  },
  {
    unitNames: runPool.map((u) => u.name),
    hasActiveRun: () => runScreen?.hasActiveRun() ?? false,
    // The Continue label's round comes from the SAME stored record the
    // leaderboard's own-run marker reads (active runs only) — never a guess.
    activeRound: () => activeRunMarker()?.round ?? null,
  },
);
// New Run and Continue both lead to the run view; the run screen decides start
// vs resume (a fresh device shows the new-run form, an active run resumes).
el<HTMLButtonElement>("title-play").addEventListener("click", () => showView("run"));
el<HTMLButtonElement>("title-continue").addEventListener("click", () => showView("run"));
// The hub's magenta "Create an idea" opens the full ideas screen on its submit
// box (the reveal is a login nudge when logged out, exactly like every other
// authed action) — the same funnel the column's footer CTA uses.
el<HTMLButtonElement>("title-create-idea").addEventListener("click", () => {
  showView("ideas");
  el<HTMLButtonElement>("ideas-reveal").click();
});
// The column headers double as the doors to the full-screen views; Codex is a
// center action. Leaderboard/ideas/history keep the old menu's nav ids so every
// live nav probe still reaches its view by id.
el<HTMLButtonElement>("title-leaderboard").addEventListener("click", () => showView("leaderboard"));
el<HTMLButtonElement>("title-ideas").addEventListener("click", () => showView("ideas"));
el<HTMLButtonElement>("hub-open-ideas").addEventListener("click", () => showView("ideas"));
el<HTMLButtonElement>("title-history").addEventListener("click", () => showView("history"));
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
// it hidden when off). When shown, it opens the Battle Editor — the one dev
// surface (#066 slice 5 deleted the legacy battle/gauntlet tabs).
titleDev.addEventListener("click", () => showView("editor"));

// #066 slice 6: Settings (the ⚙ entry) holds ONLY the dev-mode toggle now —
// the account/login block moved back to the title (Maks's gate call). The
// toggle persists aoi.dev.v1 and takes effect the moment the player walks back
// to the title.
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
  // Local solo play seeds the #075 bootstrap tower so a lone session has a
  // ladder to climb. `?ladder=empty` (PRD #085) instead opens the tower EMPTY —
  // the production genesis path — so the cold-start synth climb and the
  // found-floor-1 end are reachable (and reviewable) in local play. It seeds
  // NOTHING over the persisted local store (so a first/cleared visit is a true
  // empty launch); it never touches the active run. The remote (logged-in)
  // store is the shared server ladder.
  const emptyLadder = new URLSearchParams(window.location.search).get("ladder") === "empty";
  const ladderStore =
    remote ??
    (emptyLadder
      ? openEmptyLadder(openLocalLadder(window.localStorage))
      : seedBootstrapTower(openLocalLadder(window.localStorage), stressRegistry, stressAbilities));
  leaderboardView = createLadderView(el("leaderboard-body"), {
    store: ladderStore,
    registry: activeStatuses,
    abilities: activeAbilities,
    variant: "tower", // the leaderboard wears the compact Arena Tower (slice C)
    ...(remote !== null ? { holderName: () => remote!.holder() } : {}),
  });
  // The hub's right column (B·Arena slice B): the SAME Arena Tower render over
  // the SAME store, dropped into the title. Its own header is hidden in the hub
  // (the column supplies the mockup header that opens the full leaderboard); the
  // floors and chip inspector are the live ones.
  hubTower = createLadderView(el("hub-tower-body"), {
    store: ladderStore,
    registry: activeStatuses,
    abilities: activeAbilities,
    variant: "tower",
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
      bossPanel: el("run-boss"),
      bossHead: el("run-boss-head"),
      bossTeam: el("run-boss-team"),
      challengeButton: el<HTMLButtonElement>("run-challenge"),
      challengeCancel: el<HTMLButtonElement>("run-challenge-cancel"),
      challengeNote: el("run-challenge-note"),
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
      devPool: () => codexUnits(remote !== null ? remoteRunPool : approved, activeAbilities), // same active entities as the Codex
      devEnabled: () => loadDevMode(window.localStorage), // device-wide gate, not the run's prefixed storage
      registry: activeStatuses,
      abilities: activeAbilities,
      ...(activeSeason !== null ? { contentVersion: activeSeason.contentVersion } : {}),
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

// #066 slice 5: the standalone gauntlet view is gone — its win-rate sweep lives
// in the Battle Editor's Run ×N (slice 3, on the pure sweep in src/sweep.ts).

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
    pool: () => codexUnits(remote !== null ? remoteRunPool : approved, activeAbilities),
    registry: activeStatuses,
    abilities: activeAbilities,
    viewer,
    viewerHost: result,
    viewerHome: el("battle-view"),
  },
);

// #076 slice 3: the ideas screen. The backing is RemoteIdeas (slice 2) over
// the same arena api — list() is public (a logged-out player reads the ranked
// table), submit/vote carry the session token. Logged out we still construct
// the backing for its public list(); a submit/vote tap routes to login instead
// of the server (onNeedLogin), the same nudge every other authed action gives.
const ideasBacking = new RemoteIdeas(api, sessionToken ?? "");
ideasScreen = createIdeasScreen(
  {
    form: el<HTMLFormElement>("ideas-form"),
    text: el<HTMLInputElement>("ideas-text"),
    submit: el<HTMLButtonElement>("ideas-submit"),
    reveal: el<HTMLButtonElement>("ideas-reveal"),
    sortTop: el<HTMLButtonElement>("ideas-sort-top"),
    sortNew: el<HTMLButtonElement>("ideas-sort-new"),
    status: el("ideas-status"),
    list: el("ideas-list"),
    loginNote: el("ideas-login-note"),
    currency: el("ideas-currency"),
  },
  {
    ideas: ideasBacking,
    userId: me?.userId ?? null,
    // A logged-out submit/vote walks home and opens the login panel — the same
    // door the title's Login entry opens, so the player completes auth and
    // comes back to take part.
    onNeedLogin: () => {
      showView("title");
      el<HTMLButtonElement>("title-login").click();
    },
  },
);

// #077 slice 3: the season-history screen. It reads the device's local season
// archive (openLocalArchive) — public, no auth, like the ladder. The archive is
// written by the season transition (slice 2); until a season ends it is empty,
// which is the screen's empty state, not an error. A corrupt stored archive
// throws loudly (a silent fresh archive would erase finished history) — surfaced
// inline, not as a dead app: the rest of the title still works.
try {
  historyScreen = createHistoryScreen(
    {
      list: el("history-list"),
      detail: el("history-detail"),
      back: el<HTMLButtonElement>("history-back"),
    },
    remote !== null && sessionToken !== null
      ? {
          load: async () => {
            const result = await api.history(sessionToken);
            if (!result.ok) throw new Error(result.kind === "network" ? result.reason : "season history request was refused");
            return result.value.seasons;
          },
        }
      : { archive: openLocalArchive(window.localStorage) },
  );
} catch (err) {
  const list = el("history-list");
  list.textContent = "";
  const msg = document.createElement("p");
  msg.className = "history-empty";
  msg.setAttribute("role", "alert");
  msg.textContent = `Season history could not open: ${(err as Error).message}`;
  list.append(msg);
}

el("ideas-season").textContent = activeSeason === null
  ? "Local play · committed content"
  : `Season ${activeSeason.season} · content v${activeSeason.contentVersion}`;
el<HTMLElement>("kernel-version").textContent = `kernel v${KERNEL_VERSION}`;

// The app opens on the title screen (#015 slice 3) — Play/Continue read from
// the freshly revived run state. Cold-load deep links override it: a shared
// #codex/... URL must land on the entry, not the title — routed FIRST, never
// through showView("title"), whose stale-hash sweep (slice 4) would eat the
// link before applyHashNav could read it. After every view is wired so
// showView can hide them all.
if (window.location.hash.startsWith("#codex/")) applyHashNav();
else showView("title");
