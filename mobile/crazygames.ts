// The CrazyGames SDK v3 in the cg build (#1218, mission #1214). index.html
// loads https://sdk.crazygames.com/crazygames-sdk-v3.js in the cg build only
// (vite.config.ts); boot inits it on the loading screen. Outside CrazyGames
// (no script, a failed init, the SDK's "local" or "disabled" environment, or
// `?cgsdk=local` in the address) there is no SDK: the game plays as a guest.
// Its user module only reads who is logged in: no logout, no auth prompt.
import { CG } from "./cg";

export interface CgUser {
  username: string;
}

interface CgSdk {
  environment?: string;
  init(): Promise<void>;
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

/** Inits the SDK once; null outside CrazyGames. Never throws, never waits past `ms`. */
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
  if (s.environment === "local" || s.environment === "disabled") return null;
  sdk = s;
  return s;
}

/** The logged-in CrazyGames user's token, or null (no SDK, no account, logged out). */
export async function cgUserToken(): Promise<string | null> {
  const u = sdk?.user;
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
    sdk?.user?.addAuthListener((user) => user && fn());
  } catch (err) {
    console.warn("CrazyGames auth listener:", err);
  }
}
