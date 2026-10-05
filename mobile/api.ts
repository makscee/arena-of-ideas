/// <reference types="vite/client" />
// Typed client for the MVP API (src/mvp/contract.ts). Paths are relative to
// the app's base (/arena/ on the test instance).
import {
  MVP_API_PREFIX,
  PLAYER_HEADER,
  type BattleRecord,
  type Decision,
  type DecisionResponse,
  type HomeView,
  type MvpContent,
  type PlayerRef,
  type RunView,
} from "../src/mvp/contract";

const BASE = (import.meta.env.BASE_URL ?? "/").replace(/\/$/, "") + MVP_API_PREFIX;
const PLAYER_KEY = "arena.player";

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

export class ApiError extends Error {
  constructor(readonly status: number, message: string) {
    super(message);
  }
}

async function call<T>(method: string, path: string, body?: unknown): Promise<T> {
  const res = await fetch(BASE + path, {
    method,
    headers: { "content-type": "application/json", ...(player ? { [PLAYER_HEADER]: player.id } : {}) },
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
  async register(name: string): Promise<PlayerRef> {
    player = await call<PlayerRef>("POST", "/players", { name });
    savePlayer(player);
    return player;
  },
  forget(): void {
    player = null;
    savePlayer(null);
  },
  content: () => call<MvpContent>("GET", "/content"),
  home: () => call<HomeView>("GET", "/home"),
  startRun: () => call<RunView>("POST", "/runs"),
  run: (id: string) => call<RunView>("GET", `/runs/${id}`),
  decide: (id: string, d: Decision) => call<DecisionResponse>("POST", `/runs/${id}/decisions`, d),
  battle: (id: string) => call<BattleRecord>("GET", `/battles/${id}`),
};
