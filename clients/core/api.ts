// The protocol's fetch client (#957): one call shape for every client (the
// phone app, `npm run mvp:bot`, the MCP server, the Claude Code mod). Only
// `fetch`, so it runs in a browser, Node 18+, Bun or Deno. The routes are
// protocol/openapi.json (src/mvp/routes.ts).
import {
  MVP_API_PREFIX,
  PLAYER_HEADER,
  TOKEN_HEADER,
  type BattleRecord,
  type DayView,
  type Decision,
  type DecisionResponse,
  type EmailVerify,
  type FusionDiscovery,
  type HealthView,
  type HomeView,
  type LibraryView,
  type MvpContent,
  type PlayerRef,
  type RunView,
} from "../../src/mvp/contract.js";

/** A 4xx or 5xx answer: `{ error }` and its status. */
export class ApiError extends Error {
  constructor(
    readonly status: number,
    message: string,
  ) {
    super(message);
  }
}

/** Where and as whom a call goes. `api` is the API's base URL, prefix
 * included (apiBase); `token` (a PlayerSession's) wins over `player` (a
 * device id, open servers only). */
export interface Connection {
  api: string;
  token?: string | null;
  player?: string | null;
  /** Sent as Accept-Language, so server text (refusals) follows it. */
  lang?: string;
  fetch?: typeof fetch;
}

/** The API base for a server's root: "https://arena.makscee.ru" → ".../api/v1". */
export const apiBase = (root: string): string => root.replace(/\/$/, "") + MVP_API_PREFIX;

/** One call: JSON in, JSON out; a non-2xx answer throws ApiError. */
export async function apiCall<T>(c: Connection, method: string, path: string, body?: unknown): Promise<T> {
  const res = await (c.fetch ?? fetch)(c.api + path, {
    method,
    headers: {
      "content-type": "application/json",
      ...(c.lang ? { "accept-language": c.lang } : {}),
      ...(c.token ? { [TOKEN_HEADER]: c.token } : c.player ? { [PLAYER_HEADER]: c.player } : {}),
    },
    ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
  });
  const json = (await res.json().catch(() => ({ error: res.statusText }))) as T & { error?: string };
  if (!res.ok) throw new ApiError(res.status, json.error ?? `HTTP ${res.status}`);
  return json;
}

/** The game's calls, typed. Anything else: `call(method, path, body)`. */
export class ArenaClient {
  constructor(readonly conn: Connection) {}
  call<T>(method: "GET" | "POST", path: string, body?: unknown): Promise<T> {
    return apiCall<T>(this.conn, method, path, body);
  }
  health = () => this.call<HealthView>("GET", "/health");
  content = () => this.call<MvpContent>("GET", "/content");
  /** A device identity on an open server; this client calls as it from then on. */
  register = async (name: string) => {
    const p = await this.call<PlayerRef>("POST", "/players", { name });
    this.conn.player = p.id;
    return p;
  };
  /** Email login (#929): mails a code; `verify` then makes this client the player. */
  emailStart = (email: string, name?: string) => this.call<{ ok: true; expiresAt: string }>("POST", "/auth/email/start", { email, ...(name ? { name } : {}) });
  emailVerify = async (email: string, code: string, name?: string) => {
    const r = await this.call<EmailVerify>("POST", "/auth/email/verify", { email, code, ...(name ? { name } : {}) });
    if (r.status === "done") this.conn.token = r.token;
    return r;
  };
  home = () => this.call<HomeView>("GET", "/home");
  /** Starts a run, or returns the active one. */
  startRun = () => this.call<RunView>("POST", "/runs");
  run = (runId: string) => this.call<RunView>("GET", `/runs/${encodeURIComponent(runId)}`);
  decide = (runId: string, d: Decision) => this.call<DecisionResponse>("POST", `/runs/${encodeURIComponent(runId)}/decisions`, d);
  preview = (runId: string, d: Decision) => this.call<DecisionResponse>("POST", `/runs/${encodeURIComponent(runId)}/preview`, d);
  abandon = (runId: string) => this.call<RunView>("POST", `/runs/${encodeURIComponent(runId)}/abandon`);
  battle = (battleId: string) => this.call<BattleRecord>("GET", `/battles/${encodeURIComponent(battleId)}`);
  day = () => this.call<DayView>("GET", "/day");
  fusions = () => this.call<FusionDiscovery[]>("GET", "/fusions");
  library = () => this.call<LibraryView>("GET", "/library");
}
