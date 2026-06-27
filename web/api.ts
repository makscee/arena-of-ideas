// Arena server client (PRD #016 slice 3) — the web shell's one fetch surface.
// All URLs are relative ("/v1/..."): vite dev proxies them to the server (see
// vite.config.ts), and a same-origin deployment needs no CORS. Every call
// resolves to a discriminated result — a network failure is data, never an
// unhandled rejection, because the UI must degrade visibly, not brick.
//
// fetch is injected so vitest drives this module with a stub (the run-store
// storage pattern); the browser passes nothing and gets window.fetch.

import type { Idea, TeamSnapshot, VoteDir } from "../src/index.js";

/** What a call can come back as: the payload, a server refusal (4xx/422 with
 * its reason), or no server at all. `unauthorized` is split out because the
 * boot path treats a dead token (drop it) unlike a dead network (keep it). */
export type ApiResult<T> =
  | { ok: true; value: T }
  | { ok: false; kind: "rejected"; status: number; reason: string }
  | { ok: false; kind: "unauthorized" }
  | { ok: false; kind: "network"; reason: string };

export interface MeInfo {
  userId: string;
  email: string;
  displayName: string | null;
}

export interface VerifyInfo {
  token: string;
  sessionId: string;
  expiresAt: number;
}

export interface ChampionInfo {
  champion: TeamSnapshot | null;
  /** The owning user's display name; null for the bootstrap seat. */
  holder: string | null;
}

export interface ServedView {
  round: number;
  pool: TeamSnapshot[];
  champion: TeamSnapshot;
}

export interface SubmitInfo {
  runId: string;
  endedBy: string;
  finalRound: number;
  crowned: boolean;
}

export type FetchLike = (input: string, init?: RequestInit) => Promise<Response>;

export interface ArenaApi {
  startLogin(email: string): Promise<ApiResult<{ sent: true }>>;
  verifyLogin(email: string, code: string): Promise<ApiResult<VerifyInfo>>;
  me(token: string): Promise<ApiResult<MeInfo>>;
  logout(token: string): Promise<ApiResult<{ ok: true }>>;
  setDisplayName(token: string, displayName: string): Promise<ApiResult<{ displayName: string }>>;
  champion(): Promise<ApiResult<ChampionInfo>>;
  pool(round: number): Promise<ApiResult<{ round: number; pool: TeamSnapshot[] }>>;
  openRun(token: string, runId: string): Promise<ApiResult<{ opened: true; runId: string }>>;
  servePool(token: string, runId: string, round: number): Promise<ApiResult<ServedView>>;
  submitRun(token: string, run: string): Promise<ApiResult<SubmitInfo>>;
  listIdeas(): Promise<ApiResult<{ ideas: Idea[] }>>;
  submitIdea(token: string, text: string): Promise<ApiResult<{ submitted: true; idea: Idea }>>;
  voteIdea(token: string, ideaId: string, direction: VoteDir): Promise<ApiResult<{ cast: true; direction: VoteDir; idea: Idea }>>;
  ideaCurrency(token: string): Promise<ApiResult<{ currency: number }>>;
}

/** A refusal body's reason, best-effort: the server's `reason`/`error` field,
 * or the bare status — never raw JSON in a player's face. */
function reasonOf(body: unknown, status: number): string {
  if (typeof body === "object" && body !== null) {
    const b = body as Record<string, unknown>;
    if (typeof b.reason === "string") return b.reason;
    if (typeof b.error === "string") return b.error.replace(/_/g, " ");
  }
  return `the server answered ${status}`;
}

export function createArenaApi(fetchImpl?: FetchLike): ArenaApi {
  const doFetch: FetchLike = fetchImpl ?? ((input, init) => fetch(input, init));

  async function call<T>(path: string, init?: RequestInit & { token?: string }): Promise<ApiResult<T>> {
    let res: Response;
    try {
      res = await doFetch(path, {
        ...init,
        headers: {
          ...(init?.body !== undefined ? { "content-type": "application/json" } : {}),
          ...(init?.token !== undefined ? { authorization: `Bearer ${init.token}` } : {}),
        },
      });
    } catch (err) {
      return { ok: false, kind: "network", reason: err instanceof Error ? err.message : String(err) };
    }
    let body: unknown = null;
    try {
      body = await res.json();
    } catch {
      // A bodyless or non-JSON answer (a dead proxy's 502) is a refusal below.
    }
    if (res.ok) return { ok: true, value: body as T };
    if (res.status === 401) return { ok: false, kind: "unauthorized" };
    // A 5xx is the server failing, not refusing — the caller's degrade path
    // is the network one (retry later), not the rejection one (give up).
    if (res.status >= 500) return { ok: false, kind: "network", reason: reasonOf(body, res.status) };
    return { ok: false, kind: "rejected", status: res.status, reason: reasonOf(body, res.status) };
  }

  return {
    startLogin: (email) => call("/v1/auth/login/email/start", { method: "POST", body: JSON.stringify({ email }) }),
    verifyLogin: (email, code) => call("/v1/auth/login/email/verify", { method: "POST", body: JSON.stringify({ email, code }) }),
    me: (token) => call("/v1/auth/me", { token }),
    logout: (token) => call("/v1/auth/logout", { method: "POST", body: JSON.stringify({}), token }),
    setDisplayName: (token, displayName) =>
      call("/v1/auth/display-name", { method: "POST", body: JSON.stringify({ displayName }), token }),
    champion: () => call("/v1/ladder/champion"),
    pool: (round) => call(`/v1/ladder/pool/${round}`),
    openRun: (token, runId) => call("/v1/runs/open", { method: "POST", body: JSON.stringify({ runId }), token }),
    servePool: (token, runId, round) =>
      call(`/v1/runs/${encodeURIComponent(runId)}/pool/${round}`, { token }),
    submitRun: (token, run) => call("/v1/runs", { method: "POST", body: JSON.stringify({ run }), token }),
    listIdeas: () => call("/v1/ideas"),
    submitIdea: (token, text) => call("/v1/ideas", { method: "POST", body: JSON.stringify({ text }), token }),
    voteIdea: (token, ideaId, direction) =>
      call(`/v1/ideas/${encodeURIComponent(ideaId)}/vote`, { method: "POST", body: JSON.stringify({ direction }), token }),
    ideaCurrency: (token) => call("/v1/ideas/currency", { token }),
  };
}
