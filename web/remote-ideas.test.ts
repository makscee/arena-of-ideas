// Remote ideas tests — the server-backed ideas store driven through a stub
// ArenaApi (no network, the remote-ladder stubApi pattern). The bar: list/
// submit/vote resolve to discriminated results carrying the server's view, and
// every failure becomes a player-shaped reason, never raw transport.

import { describe, expect, test } from "vitest";
import type { Idea, VoteMap } from "../src/index.js";
import type { ApiResult, ArenaApi } from "./api.js";
import { RemoteIdeas } from "./remote-ideas.js";

const idea = (id: string, seq: number, text: string, votes: VoteMap = {}): Idea => ({
  id,
  authorId: "ada",
  text,
  seq,
  votes,
});

/** An ArenaApi where every method fails loudly unless overridden — a test
 * touching an endpoint it didn't stub is asking the wrong question. */
function stubApi(overrides: Partial<ArenaApi>): ArenaApi {
  const die = (name: string) => () => {
    throw new Error(`unexpected api call: ${name}`);
  };
  return {
    startLogin: die("startLogin"),
    verifyLogin: die("verifyLogin"),
    me: die("me"),
    logout: die("logout"),
    setDisplayName: die("setDisplayName"),
    champion: die("champion"),
    pool: die("pool"),
    openRun: die("openRun"),
    servePool: die("servePool"),
    submitRun: die("submitRun"),
    listIdeas: die("listIdeas"),
    submitIdea: die("submitIdea"),
    voteIdea: die("voteIdea"),
    ideaCurrency: die("ideaCurrency"),
    ...overrides,
  } as ArenaApi;
}

const ok = <T,>(value: T): ApiResult<T> => ({ ok: true, value });

describe("RemoteIdeas over a stub api", () => {
  test("list returns the server's ranked ideas (public, no token used)", async () => {
    const ranked = [idea("idea-1", 1, "B", { x: "up", y: "up" }), idea("idea-0", 0, "A", { x: "up" })];
    const store = new RemoteIdeas(stubApi({ listIdeas: async () => ok({ ideas: ranked }) }), "token");
    const res = await store.list();
    expect(res).toEqual({ ok: true, value: ranked });
  });

  test("submit returns the stored idea the server assigned", async () => {
    const stored = idea("idea-0", 0, "draft mode");
    const store = new RemoteIdeas(
      stubApi({ submitIdea: async (token, text) => ok({ submitted: true as const, idea: { ...stored, text } }) }),
      "token",
    );
    const res = await store.submit("draft mode");
    expect(res).toEqual({ ok: true, value: { ...stored, text: "draft mode" } });
  });

  test("vote casts a direction and returns the post-cast idea and direction", async () => {
    const after = idea("idea-0", 0, "A", { ada: "down" });
    const seen: Array<{ ideaId: string; direction: string }> = [];
    const store = new RemoteIdeas(
      stubApi({
        voteIdea: async (_token, ideaId, direction) => {
          seen.push({ ideaId, direction });
          return ok({ cast: true as const, direction, idea: after });
        },
      }),
      "token",
    );
    const res = await store.vote("idea-0", "down");
    expect(res).toEqual({ ok: true, value: { direction: "down", idea: after } });
    expect(seen).toEqual([{ ideaId: "idea-0", direction: "down" }]); // the direction reached the api
  });

  test("currency returns the caller's vote footprint as a number", async () => {
    const store = new RemoteIdeas(stubApi({ ideaCurrency: async () => ok({ currency: 3 }) }), "token");
    expect(await store.currency()).toEqual({ ok: true, value: 3 });
  });

  test("a currency failure maps to a player-shaped reason", async () => {
    const store = new RemoteIdeas(
      stubApi({ ideaCurrency: async () => ({ ok: false, kind: "network", reason: "ECONNREFUSED" }) as ApiResult<never> }),
      "token",
    );
    expect(await store.currency()).toMatchObject({ ok: false, reason: expect.stringContaining("unreachable") });
  });

  test("failures map to player-shaped reasons, never raw transport", async () => {
    const store = new RemoteIdeas(
      stubApi({
        listIdeas: async () => ({ ok: false, kind: "network", reason: "ECONNREFUSED" }) as ApiResult<never>,
        submitIdea: async () => ({ ok: false, kind: "unauthorized" }) as ApiResult<never>,
        voteIdea: async () => ({ ok: false, kind: "rejected", status: 422, reason: "no idea with id idea-9" }) as ApiResult<never>,
      }),
      "token",
    );
    expect(await store.list()).toMatchObject({ ok: false, reason: expect.stringContaining("unreachable") });
    expect(await store.submit("x")).toMatchObject({ ok: false, reason: expect.stringContaining("log in") });
    expect(await store.vote("idea-9", "up")).toEqual({ ok: false, reason: "no idea with id idea-9" });
  });
});
