// Remote ideas (#076 slice 2) — the server-backed ideas table behind one
// store surface, so slice 3's UI talks to the same shape whether the backing is
// local (PersistedIdeaStore over localStorage, web/ideas-store.ts) or this
// remote one over the server. The kernel's IdeaStore is synchronous (it reads
// in-process state); a server backing is necessarily async, so this mirrors
// RemoteLadder's split: async methods that always resolve to a discriminated
// result — a network failure is data the UI degrades on, never a thrown reject.
//
// Identity is the session token: submit and vote carry the bearer (the server
// keys the one-vote-per-player rule on session.userId), list is public. The
// server is the source of truth for ranking and vote sets — every method that
// mutates returns the server's post-mutation view, so the UI never guesses.

import type { Idea, VoteDir } from "../src/index.js";
import type { ArenaApi } from "./api.js";

export type IdeasResult<T> = { ok: true; value: T } | { ok: false; reason: string };

/** The remote ideas seam: list (public) + submit/vote (authed). All methods
 * resolve, never reject — failures arrive as player-shaped reasons. */
export class RemoteIdeas {
  private readonly api: ArenaApi;
  private readonly token: string;

  constructor(api: ArenaApi, token: string) {
    this.api = api;
    this.token = token;
  }

  /** Every idea, ranked by votes (server order) — public, no token. */
  async list(): Promise<IdeasResult<Idea[]>> {
    const res = await this.api.listIdeas();
    if (!res.ok) return { ok: false, reason: failureReason(res) };
    return { ok: true, value: res.value.ideas };
  }

  /** Submit an idea as the caller. Returns the stored idea (id, seq, empty
   * vote set) the server assigned. */
  async submit(text: string): Promise<IdeasResult<Idea>> {
    const res = await this.api.submitIdea(this.token, text);
    if (!res.ok) return { ok: false, reason: failureReason(res) };
    return { ok: true, value: res.value.idea };
  }

  /** Cast the caller's directional vote on `ideaId` (switch-only: up/down, never
   * removed). Returns the idea in its post-cast state plus the direction now
   * held — the server is the source of truth, so the UI renders the returned
   * vote map, never a local guess. */
  async vote(ideaId: string, direction: VoteDir): Promise<IdeasResult<{ direction: VoteDir; idea: Idea }>> {
    const res = await this.api.voteIdea(this.token, ideaId, direction);
    if (!res.ok) return { ok: false, reason: failureReason(res) };
    return { ok: true, value: { direction: res.value.direction, idea: res.value.idea } };
  }
}

function failureReason(res: { ok: false; kind: string; reason?: string }): string {
  if (res.kind === "unauthorized") return "your session has expired — log in again";
  if (res.kind === "network") return `the server is unreachable (${res.reason ?? "no answer"})`;
  return res.reason ?? "the server refused";
}
