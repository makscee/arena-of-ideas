/// <reference types="vite/client" />
// Typed client for the MVP API (src/mvp/contract.ts). Paths are relative to
// the app's base (/arena/ on the test instance).
import {
  MVP_API_PREFIX,
  PLAYER_HEADER,
  TOKEN_HEADER,
  type BattleRecord,
  type CreditsView,
  type DayView,
  type Decision,
  type DecisionResponse,
  type FusionDiscovery,
  type HomeView,
  type IdeasView,
  type MyIdeasView,
  type JoinSession,
  type LibraryView,
  type MvpContent,
  type PlayerRef,
  type PlayerSession,
  type RunView,
  type StatsView,
} from "../src/mvp/contract";

const BASE = (import.meta.env.BASE_URL ?? "/").replace(/\/$/, "") + MVP_API_PREFIX;
const PLAYER_KEY = "arena.player";
/** Slice 13: the session token from an invite link, kept on the device. */
const TOKEN_KEY = "arena.token";
/** R4-20: the invite code of a player who joined through the open link on
 * this device, for Home's "Your link (for another device)". */
const OWN_INVITE_KEY = "arena.ownInvite";

export function savedPlayer(): PlayerRef | null {
  try {
    const raw = localStorage.getItem(PLAYER_KEY);
    return raw ? (JSON.parse(raw) as PlayerRef) : null;
  } catch {
    return null;
  }
}

function savePlayer(p: PlayerRef | null): void {
  try {
    if (p) localStorage.setItem(PLAYER_KEY, JSON.stringify(p));
    else localStorage.removeItem(PLAYER_KEY);
  } catch {
    /* private mode: the name lasts this tab only */
  }
}

let player: PlayerRef | null = savedPlayer();
let token: string | null = (() => {
  try {
    return localStorage.getItem(TOKEN_KEY);
  } catch {
    return null;
  }
})();

let ownInvite: string | null = (() => {
  try {
    return localStorage.getItem(OWN_INVITE_KEY);
  } catch {
    return null;
  }
})();

function saveOwnInvite(code: string | null): void {
  ownInvite = code;
  try {
    if (code) localStorage.setItem(OWN_INVITE_KEY, code);
    else localStorage.removeItem(OWN_INVITE_KEY);
  } catch {
    /* private mode: the link shows this tab only */
  }
}

function saveToken(t: string | null): void {
  try {
    if (t) localStorage.setItem(TOKEN_KEY, t);
    else localStorage.removeItem(TOKEN_KEY);
  } catch {
    /* private mode: the session lasts this tab only */
  }
}

export class ApiError extends Error {
  constructor(readonly status: number, message: string) {
    super(message);
  }
}

async function call<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(BASE + path, {
    method,
    headers: { "content-type": "application/json", ...(token ? { [TOKEN_HEADER]: token } : player ? { [PLAYER_HEADER]: player.id } : {}) },
    ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
  });
  const json = (await res.json().catch(() => ({ error: res.statusText }))) as T & { error?: string };
  if (!res.ok) throw new ApiError(res.status, json.error ?? `HTTP ${res.status}`);
  return json;
}

export const api = {
  get player() {
    return player;
  },
  /** True once an invite link gave this device a session token (slice 13). */
  get hasToken() {
    return token !== null;
  },
  async register(name: string): Promise<PlayerRef> {
    player = await call<PlayerRef>("POST", "/players", { name });
    savePlayer(player);
    return player;
  },
  /** Whose invite link this is, without opening it (slice 13). */
  invitePlayer: (code: string) => call<{ player: PlayerRef }>("POST", "/invites/lookup", { code }).then((r) => r.player),
  /** Opens an invite link (slice 13): this device becomes its player. */
  async redeem(code: string): Promise<PlayerRef> {
    const s = await call<PlayerSession>("POST", "/invites/redeem", { code });
    if (s.player.id !== player?.id) saveOwnInvite(null);
    player = s.player;
    token = s.token;
    savePlayer(player);
    saveToken(token);
    return player;
  },
  /** R4-20: the invite code this device's player got by joining (their link), or null. */
  get ownInvite() {
    return ownInvite;
  },
  /** Is `code` the open join link (R4-20)? 404 ApiError if not. */
  joinCheck: (code: string) => call<{ ok: true }>("POST", "/join/check", { code }),
  /** Joins through the open link as a new player named `name`: this device becomes that player. */
  async join(code: string, name: string): Promise<JoinSession> {
    const s = await call<JoinSession>("POST", "/join", { code, name });
    player = s.player;
    token = s.token;
    savePlayer(player);
    saveToken(token);
    saveOwnInvite(s.invite);
    return s;
  },
  forget(): void {
    saveOwnInvite(null);
    player = null;
    token = null;
    savePlayer(null);
    saveToken(null);
  },
  /** `invites`: the server is invite-only, so the name screen asks for a link. */
  health: () => call<{ invites?: boolean; open?: boolean }>("GET", "/health"),
  content: () => call<MvpContent>("GET", "/content"),
  home: () => call<HomeView>("GET", "/home"),
  startRun: () => call<RunView>("POST", "/runs"),
  run: (id: string) => call<RunView>("GET", `/runs/${id}`),
  decide: (id: string, d: Decision) => call<DecisionResponse>("POST", `/runs/${id}/decisions`, d),
  /** What a shop decision would do, without doing it (the awakening and fusion result cards). */
  preview: (id: string, d: Decision) => call<DecisionResponse>("POST", `/runs/${id}/preview`, d),
  /** Gives the run up (slice R2-2): it ends "abandoned", every heart left
   * rated a lost fight. A bare RunView, not a DecisionResponse. */
  abandon: (id: string) => call<RunView>("POST", `/runs/${id}/abandon`),
  battle: (id: string) => call<BattleRecord>("GET", `/battles/${id}`),
  /** The day, its champion and the last playoff (slice 5). */
  day: () => call<DayView>("GET", "/day"),
  /** Dev "end day now": 404 unless the server runs with MVP_DEV=1, 501 until slice 5. */
  endDay: () => call<DayView>("POST", "/dev/end-day"),
  /** Dev "+1 idea" (M2-3): 404 unless MVP_DEV=1. */
  grantIdea: () => call<IdeasView>("POST", "/dev/grant-idea"),
  /** My ideas (M2-4): the ideas held and the player's own sent ones. */
  myIdeas: () => call<MyIdeasView>("GET", "/ideas"),
  /** Sends an idea, spending one held (400 its length, 409 none held). */
  writeIdea: (text: string) => call<MyIdeasView>("POST", "/ideas", { text }),
  /** Takes back a `written` idea, refunding it. */
  cancelIdea: (ideaId: string) => call<MyIdeasView>("POST", `/ideas/${encodeURIComponent(ideaId)}/cancel`),
  /** 501 until slice 11. */
  stats: () => call<StatsView>("GET", "/stats"),
  /** Discovered fusions (slice 10). */
  fusions: () => call<FusionDiscovery[]>("GET", "/fusions"),
  /** Who each live unit's idea was, NEW, your creator number (M2-9). */
  credits: () => call<CreditsView>("GET", "/credits"),
  /** The units that have left the pool (M2-9). */
  library: () => call<LibraryView>("GET", "/library"),
  /** Dev (M2-9): the unit becomes your idea, entered today. 404 unless MVP_DEV=1. */
  creditUnit: (unitId?: string) => call<CreditsView>("POST", "/dev/credit-unit", unitId ? { unitId } : {}),
};
