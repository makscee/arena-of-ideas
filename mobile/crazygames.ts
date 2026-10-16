// The CrazyGames SDK v3 in the cg build (#1218, mission #1214). index.html
// loads https://sdk.crazygames.com/crazygames-sdk-v3.js in the cg build only
// (vite.config.ts); boot inits it on the loading screen. Outside CrazyGames
// (no script, a failed init, the SDK's "local" or "disabled" environment, or
// `?cgsdk=local` in the address) there is no SDK: the game plays as a guest.
// Its user module only reads who is logged in: no logout, no auth prompt.
// #1219: the SDK's lifecycle (loading, gameplay), the ad break hook and the
// portal's settings (muteAudio) are here too.
import { CG } from "./cg";
import { onAdFinished, onAdStarted, setPortalMute } from "./ui/sound";

export interface CgUser {
  username: string;
}

type AdType = "midgame" | "rewarded";

interface CgSdk {
  environment?: string;
  init(): Promise<void>;
  game?: {
    loadingStart?(): void;
    loadingStop?(): void;
    gameplayStart?(): void;
    gameplayStop?(): void;
    settings?: { muteAudio?: boolean; disableChat?: boolean };
    addSettingsChangeListener?(fn: (s: { muteAudio?: boolean; disableChat?: boolean }) => void): void;
  };
  ad?: { requestAd?(type: AdType, cb: { adStarted?: () => void; adFinished?: () => void; adError?: (e: unknown) => void }): void };
  user?: {
    isUserAccountAvailable?: boolean;
    getUser(): Promise<CgUser | null>;
    getUserToken(): Promise<string>;
    addAuthListener(fn: (user: CgUser | null) => void): void;
  };
}

declare global {
  interface Window {
    CrazyGames?: { SDK?: CgSdk };
  }
}

let sdk: CgSdk | null = null;

/** Inits the SDK once; null outside CrazyGames (and where it is disabled). Never throws, never waits past `ms`. */
export async function cgInit(ms = 8000): Promise<CgSdk | null> {
  if (!CG || new URLSearchParams(location.search).get("cgsdk") === "local") return null;
  const s = window.CrazyGames?.SDK;
  if (!s) return null;
  try {
    await Promise.race([s.init(), new Promise((_, no) => setTimeout(() => no(new Error("CrazyGames SDK init timed out")), ms))]);
  } catch (err) {
    console.warn("CrazyGames SDK:", err);
    return null;
  }
  // "disabled": every call throws. "local" (`?useLocalSdk=true`, or localhost):
  // the lifecycle, ads and settings work and only log, but its user is a demo.
  if (s.environment === "disabled") return null;
  sdk = s;
  return s;
}

/** Runs an SDK call that must never break the game. */
function sdkCall(what: string, fn: (s: CgSdk) => void): void {
  if (!sdk) return;
  try {
    fn(sdk);
  } catch (err) {
    console.warn(`CrazyGames SDK ${what}:`, err);
  }
}

/** The loading screen: loadingStart when boot begins, loadingStop once Home is up. */
export function cgLoading(on: boolean): void {
  sdkCall(on ? "loadingStart" : "loadingStop", (s) => (on ? s.game?.loadingStart?.() : s.game?.loadingStop?.()));
}

let gameplayOn = false;
/** gameplayStart / gameplayStop, each only when the state changes. */
export function cgGameplay(on: boolean): void {
  if (on === gameplayOn || !sdk) return;
  gameplayOn = on;
  sdkCall(on ? "gameplayStart" : "gameplayStop", (s) => (on ? s.game?.gameplayStart?.() : s.game?.gameplayStop?.()));
}

/** The screens where the player is playing: a run's shop and a battle. Home,
 * the Codex, Forge, ideas, results and the run menu are not. */
const PLAYING = new Set(["shop", "battle"]);

/** Keeps gameplayStart/Stop in line with the screen `app` shows (its
 * data-screen) and the run menu over it: no screen has to say so. Redraws
 * that leave the state alone call nothing. */
export function cgWatchGameplay(app: HTMLElement): void {
  const sync = () => cgGameplay(PLAYING.has(app.dataset.screen ?? "") && !app.querySelector('[data-testid="run-menu"]'));
  new MutationObserver(sync).observe(app, { childList: true, attributes: true, attributeFilter: ["data-screen"] });
  sync();
}

/** The portal's settings: muteAudio keeps the game silent now and when its
 * menu changes it. There is no chat and no text from other players but
 * names (guest names, or a CrazyGames username the portal moderates; the
 * server's profanity filter is on both), so disableChat has nothing to hide. */
export function cgApplySettings(): void {
  sdkCall("settings", (s) => {
    setPortalMute(!!s.game?.settings?.muteAudio);
    s.game?.addSettingsChangeListener?.((now) => setPortalMute(!!now.muteAudio));
  });
}

/** An ad break (Full Launch; Basic Launch requests none): the game mutes
 * while it plays and unmutes when it ends or fails. */
export function cgRequestAd(type: AdType = "midgame"): void {
  if (!sdk?.ad?.requestAd) return;
  sdkCall("requestAd", (s) => s.ad!.requestAd!(type, { adStarted: onAdStarted, adFinished: onAdFinished, adError: onAdFinished }));
}

/** The playtest page's frame (scripts/arena-cg-playtest.sh, `?playtest=1`)
 * asks for an ad break by message; no other build listens. */
export function cgPlaytestHooks(): void {
  if (new URLSearchParams(location.search).get("playtest") !== "1") return;
  addEventListener("message", (e) => {
    if (e.source === parent && (e.data as { type?: string } | null)?.type === "arena-playtest-ad") cgRequestAd("midgame");
  });
}

/** The logged-in CrazyGames user's token, or null (no SDK, no account, logged out). */
export async function cgUserToken(): Promise<string | null> {
  const u = sdk?.environment === "local" ? undefined : sdk?.user;
  if (!u || u.isUserAccountAvailable === false) return null;
  try {
    return (await u.getUser()) ? await u.getUserToken() : null;
  } catch (err) {
    console.warn("CrazyGames user:", err);
    return null;
  }
}

/** Calls `fn` when the player logs in to CrazyGames while playing (a logout reloads the page). */
export function onCgLogin(fn: () => void): void {
  try {
    if (sdk?.environment !== "local") sdk?.user?.addAuthListener((user) => user && fn());
  } catch (err) {
    console.warn("CrazyGames auth listener:", err);
  }
}
