/**
 * Ideas table API contract (#076 slice 2, evolved to directional votes in #082
 * slice 1). In-process Hono app over in-memory SQLite + mock mailer, the
 * ladder-api.test.ts harness. The acceptance:
 *
 *  - submit an idea → it appears in the list,
 *  - a player casts up then switches to down → ONE vote, flipped (the composite
 *    PK on idea_votes is the one-vote-per-player floor — DB-level, not
 *    app-level); re-casting the same direction is a no-op,
 *  - SWITCH-ONLY: no request returns a player to neutral (no un-vote path),
 *  - list returns ranked order (up raises, down lowers, ties by submission seq),
 *  - the serialized kernel `Idea` shape round-trips through a real request,
 *  - auth: an unauthenticated submit/vote is rejected (401); list is public.
 *
 * Plus a must-fail-first: a votes table WITHOUT the PK lets a second (idea,user)
 * row exist — proving the constraint, not app logic, is what holds one vote per
 * player.
 */
import { describe, expect, test } from "vitest";
import Database from "better-sqlite3";
import type { Hono } from "hono";
import { InMemoryIdeaStore, voteScore, type Idea, type VoteDir } from "../../src/index.js";
import {
  castIdeaVote,
  listIdeas as listIdeasFn,
  recordBuildOutcome,
  submitIdea as submitIdeaFn,
} from "./ideas.js";
import { createApp } from "./app.js";
import type { AuthEnv } from "./auth.js";
import { openDb } from "./db.js";
import { createMockMailClient, type MockMailClient } from "./mail.js";
import { createRateLimiter } from "./rate-limit.js";

interface Ctx {
  app: Hono<AuthEnv>;
  mailer: MockMailClient;
  now: { sec: number };
}

function makeCtx(): Ctx {
  const { db } = openDb(":memory:");
  const mailer = createMockMailClient();
  const now = { sec: 1_750_000_000 };
  const clock = () => now.sec;
  const app = createApp({
    db,
    clock,
    mailClient: mailer,
    rateLimiters: {
      ipStart: createRateLimiter({ limit: 100, windowMs: 60_000, clock: () => clock() * 1000 }),
      emailStart: createRateLimiter({ limit: 100, windowMs: 60_000, clock: () => clock() * 1000 }),
      poolServe: createRateLimiter({ limit: 10_000, windowMs: 60_000, clock: () => clock() * 1000 }),
    },
  });
  return { app, mailer, now };
}

async function login(ctx: Ctx, email: string): Promise<string> {
  const start = await ctx.app.request("/v1/auth/login/email/start", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ email }),
  });
  expect(start.status).toBe(200);
  const code = ctx.mailer.sent[ctx.mailer.sent.length - 1]!.text.match(/\b(\d{6})\b/)![1]!;
  const verify = await ctx.app.request("/v1/auth/login/email/verify", {
    method: "POST",
    headers: { "content-type": "application/json" },
    body: JSON.stringify({ email, code }),
  });
  expect(verify.status).toBe(200);
  return ((await verify.json()) as { token: string }).token;
}

async function submitIdea(ctx: Ctx, token: string | null, text: string): Promise<{ status: number; body: any }> {
  const res = await ctx.app.request("/v1/ideas", {
    method: "POST",
    headers: { "content-type": "application/json", ...(token ? { authorization: `Bearer ${token}` } : {}) },
    body: JSON.stringify({ text }),
  });
  return { status: res.status, body: await res.json() };
}

async function voteIdea(
  ctx: Ctx,
  token: string | null,
  ideaId: string,
  direction: VoteDir | null = "up",
): Promise<{ status: number; body: any }> {
  const res = await ctx.app.request(`/v1/ideas/${encodeURIComponent(ideaId)}/vote`, {
    method: "POST",
    headers: { "content-type": "application/json", ...(token ? { authorization: `Bearer ${token}` } : {}) },
    body: JSON.stringify(direction === null ? {} : { direction }),
  });
  return { status: res.status, body: await res.json() };
}

async function listIdeas(ctx: Ctx): Promise<Idea[]> {
  const res = await ctx.app.request("/v1/ideas");
  expect(res.status).toBe(200);
  return ((await res.json()) as { ideas: Idea[] }).ideas;
}

const ideaOf = (ideas: Idea[], id: string) => ideas.find((i) => i.id === id)!;
const voterCount = (ideas: Idea[], id: string) => Object.keys(ideaOf(ideas, id).votes).length;
const scoreOf = (ideas: Idea[], id: string) => voteScore(ideaOf(ideas, id).votes);

describe("submit and list", () => {
  test("a submitted idea appears in the list with the kernel Idea shape", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const { status, body } = await submitIdea(ctx, ada, "  add a draft mode  ");
    expect(status).toBe(200);
    expect(body).toMatchObject({ submitted: true });
    // Trimmed, id off the seq, empty vote map — the kernel's submit contract.
    expect(body.idea).toMatchObject({ id: "idea-0", seq: 0, text: "add a draft mode", votes: {} });

    const ideas = await listIdeas(ctx);
    expect(ideas).toHaveLength(1);
    expect(ideas[0]).toMatchObject({ id: "idea-0", text: "add a draft mode", votes: {} });
  });

  test("empty / whitespace-only text is rejected (the kernel's shared rule)", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const { status, body } = await submitIdea(ctx, ada, "   ");
    expect(status).toBe(422);
    expect(body).toMatchObject({ submitted: false });
    expect(body.reason).toMatch(/empty/);
    expect(await listIdeas(ctx)).toHaveLength(0);
  });

  test("seq increments per submission — ids stay stable and unique", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    // One idea per author per day — step the clock a day per submit so the same
    // author may seed several ideas (the seq/id contract is what's under test).
    expect((await submitIdea(ctx, ada, "first")).body.idea).toMatchObject({ id: "idea-0", seq: 0 });
    ctx.now.sec += DAY;
    expect((await submitIdea(ctx, ada, "second")).body.idea).toMatchObject({ id: "idea-1", seq: 1 });
    ctx.now.sec += DAY;
    expect((await submitIdea(ctx, ada, "third")).body.idea).toMatchObject({ id: "idea-2", seq: 2 });
  });
});

const DAY = 86_400;

describe("one idea per player per day", () => {
  test("first submit of a day succeeds; a second is refused (422, not stored); next day succeeds", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");

    const first = await submitIdea(ctx, ada, "idea one");
    expect(first.status).toBe(200);
    expect(first.body).toMatchObject({ submitted: true });

    // Second submit the SAME UTC day → refused with a player-shaped reason, 422.
    const second = await submitIdea(ctx, ada, "idea two");
    expect(second.status).toBe(422);
    expect(second.body).toMatchObject({ submitted: false });
    expect(second.body.reason).toMatch(/one idea per day/i);
    expect(await listIdeas(ctx)).toHaveLength(1); // idea two was NOT stored

    // Advance the injected clock past the UTC day boundary → allowed again.
    ctx.now.sec += DAY;
    const next = await submitIdea(ctx, ada, "idea three");
    expect(next.status).toBe(200);
    expect(next.body).toMatchObject({ submitted: true });
    expect(await listIdeas(ctx)).toHaveLength(2);
  });

  test("the limit is per-player — another author may submit the same day", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");
    expect((await submitIdea(ctx, ada, "ada's idea")).status).toBe(200);
    expect((await submitIdea(ctx, bob, "bob's idea")).status).toBe(200); // different author, same day → fine
    expect((await submitIdea(ctx, ada, "ada again")).status).toBe(422); // ada's second → refused
    expect(await listIdeas(ctx)).toHaveLength(2);
  });
});

describe("directional vote: one vote per player (DB-enforced), switch-only", () => {
  test("cast up then switch to down — one vote, flipped, never a second row", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const id = (await submitIdea(ctx, ada, "buff the wisp")).body.idea.id;
    const meId = (await meOf(ctx, ada)).userId;

    const first = await voteIdea(ctx, ada, id, "up");
    expect(first.status).toBe(200);
    expect(first.body).toMatchObject({ cast: true, direction: "up" });
    expect(first.body.idea.votes).toEqual({ [meId]: "up" });
    expect(voterCount(await listIdeas(ctx), id)).toBe(1);

    // Switch to down: the SAME single row flips direction — never a second row.
    const second = await voteIdea(ctx, ada, id, "down");
    expect(second.body).toMatchObject({ cast: true, direction: "down" });
    expect(second.body.idea.votes).toEqual({ [meId]: "down" });
    expect(voterCount(await listIdeas(ctx), id)).toBe(1); // still ONE vote
    expect(scoreOf(await listIdeas(ctx), id)).toBe(-1); // now a down

    // Re-cast the same direction: an idempotent no-op, still one row.
    const third = await voteIdea(ctx, ada, id, "down");
    expect(third.body.direction).toBe("down");
    expect(voterCount(await listIdeas(ctx), id)).toBe(1);
  });

  test("switch-only: no request returns a player to neutral (the vote persists)", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const id = (await submitIdea(ctx, ada, "permanent")).body.idea.id;
    // Cast, flip, flip back — through every transition the vote stays present.
    await voteIdea(ctx, ada, id, "up");
    await voteIdea(ctx, ada, id, "down");
    await voteIdea(ctx, ada, id, "up");
    expect(voterCount(await listIdeas(ctx), id)).toBe(1); // never un-voted
    // The route rejects anything that isn't a direction — there is no neutral.
    const bad = await voteIdea(ctx, ada, id, null);
    expect(bad.status).toBe(400);
    expect(bad.body).toMatchObject({ error: "invalid_direction" });
    expect(voterCount(await listIdeas(ctx), id)).toBe(1); // the bad request changed nothing
  });

  test("two players each cast their own direction — count 2, each its own sense", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");
    const adaId = (await meOf(ctx, ada)).userId;
    const bobId = (await meOf(ctx, bob)).userId;
    const id = (await submitIdea(ctx, ada, "shared idea")).body.idea.id;

    await voteIdea(ctx, ada, id, "up");
    await voteIdea(ctx, bob, id, "down");
    const ideas = await listIdeas(ctx);
    expect(voterCount(ideas, id)).toBe(2);
    expect(ideaOf(ideas, id).votes).toEqual({ [adaId]: "up", [bobId]: "down" });
    expect(scoreOf(ideas, id)).toBe(0); // one up, one down → net neutral
    // The serialized map's keys are sorted (stable shape).
    const keys = Object.keys(ideaOf(ideas, id).votes);
    expect([...keys].sort()).toEqual(keys);
  });

  test("voting an unknown idea is rejected", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const { status, body } = await voteIdea(ctx, ada, "idea-999", "up");
    expect(status).toBe(422);
    expect(body).toMatchObject({ cast: false });
    expect(body.reason).toMatch(/no idea/);
  });

  // Must-fail-first: prove it's the CONSTRAINT, not app logic, that holds one
  // vote per player. A votes table WITHOUT the composite PK lets a second
  // (idea, user) row exist — the very corruption the PK prevents. With the PK
  // (the real schema), the second cast switches the existing row in place.
  test("a votes table WITHOUT the PK keeps two rows — the constraint is what protects us", () => {
    // No PK: nothing stops two rows for the same (idea, user).
    const loose = new Database(":memory:");
    loose.exec(`CREATE TABLE idea_votes_loose (idea_id TEXT NOT NULL, user_id TEXT NOT NULL, direction TEXT NOT NULL)`);
    const insLoose = loose.prepare(`INSERT INTO idea_votes_loose (idea_id, user_id, direction) VALUES (?, ?, ?)`);
    insLoose.run("idea-0", "ada", "up");
    insLoose.run("idea-0", "ada", "down"); // a switch / race — no constraint to upsert against
    const looseCount = loose.prepare(`SELECT COUNT(*) c FROM idea_votes_loose WHERE idea_id = ?`).get("idea-0") as { c: number };
    expect(looseCount.c).toBe(2); // two rows — the failure mode the PK exists to kill

    // The real schema: composite PK on (idea_id, user_id). The second cast
    // collides; ON CONFLICT DO UPDATE (the server's onConflictDoUpdate) switches
    // the existing row's direction in place — one row, never two.
    const guarded = new Database(":memory:");
    guarded.exec(`CREATE TABLE idea_votes (idea_id TEXT NOT NULL, user_id TEXT NOT NULL, direction TEXT NOT NULL, PRIMARY KEY (idea_id, user_id))`);
    const insGuarded = guarded.prepare(
      `INSERT INTO idea_votes (idea_id, user_id, direction) VALUES (?, ?, ?)
       ON CONFLICT (idea_id, user_id) DO UPDATE SET direction = excluded.direction`,
    );
    insGuarded.run("idea-0", "ada", "up");
    insGuarded.run("idea-0", "ada", "down");
    const guardedRows = guarded.prepare(`SELECT direction FROM idea_votes WHERE idea_id = ?`).all("idea-0") as { direction: string }[];
    expect(guardedRows).toHaveLength(1); // the PK held: one vote per player, at the DB layer
    expect(guardedRows[0]!.direction).toBe("down"); // switched in place, not duplicated
  });
});

async function currencyOf(ctx: Ctx, token: string): Promise<number> {
  const res = await ctx.app.request("/v1/ideas/currency", { headers: { authorization: `Bearer ${token}` } });
  expect(res.status).toBe(200);
  return ((await res.json()) as { currency: number }).currency;
}

describe("vote-currency (derived, no stored counter)", () => {
  test("counts distinct ideas voted, unchanged by a flip or a re-cast", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    // One idea per author per day — step the clock per submit to seed three.
    const a = (await submitIdea(ctx, ada, "idea A")).body.idea.id;
    ctx.now.sec += DAY;
    const b = (await submitIdea(ctx, ada, "idea B")).body.idea.id;
    ctx.now.sec += DAY;
    const c = (await submitIdea(ctx, ada, "idea C")).body.idea.id;

    expect(await currencyOf(ctx, ada)).toBe(0); // voted on nothing yet
    await voteIdea(ctx, ada, a, "up");
    await voteIdea(ctx, ada, b, "down");
    expect(await currencyOf(ctx, ada)).toBe(2); // two distinct ideas, either direction counts
    await voteIdea(ctx, ada, a, "down"); // flip a's direction
    expect(await currencyOf(ctx, ada)).toBe(2); // a flip never changes the count
    await voteIdea(ctx, ada, b, "down"); // re-cast the same direction
    expect(await currencyOf(ctx, ada)).toBe(2); // a no-op re-cast never changes it
    await voteIdea(ctx, ada, c, "up"); // a third distinct idea
    expect(await currencyOf(ctx, ada)).toBe(3);
  });

  test("currency is per-player and needs a bearer", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");
    const id = (await submitIdea(ctx, ada, "shared")).body.idea.id;
    await voteIdea(ctx, ada, id, "up");
    expect(await currencyOf(ctx, ada)).toBe(1);
    expect(await currencyOf(ctx, bob)).toBe(0); // bob's footprint is his own
    // Public/anonymous read is refused — the currency is per-session-user.
    expect((await ctx.app.request("/v1/ideas/currency")).status).toBe(401);
  });
});

describe("list returns ranked order", () => {
  test("ideas rank by directional score desc (up raises, down lowers), ties by seq", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");
    const cat = await login(ctx, "cat@example.com");

    // One idea per author per day — step the clock a day per submit so ada may
    // seed the three ideas under test.
    const a = (await submitIdea(ctx, ada, "idea A")).body.idea.id; // seq 0
    ctx.now.sec += DAY;
    const b = (await submitIdea(ctx, ada, "idea B")).body.idea.id; // seq 1
    ctx.now.sec += DAY;
    const c = (await submitIdea(ctx, ada, "idea C")).body.idea.id; // seq 2

    // B gets 3 up (+3), A gets 1 up (+1), C gets a down (−1).
    for (const t of [ada, bob, cat]) await voteIdea(ctx, t, b, "up");
    await voteIdea(ctx, ada, a, "up");
    await voteIdea(ctx, bob, c, "down");

    const ranked = await listIdeas(ctx);
    // B (+3) first, A (+1) next, C (−1) LAST — a down pushes it below neutral.
    expect(ranked.map((i) => i.id)).toEqual([b, a, c]);
  });
});

describe("auth", () => {
  test("submit and vote require a bearer; list is public", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const id = (await submitIdea(ctx, ada, "seed")).body.idea.id;

    // Unauthenticated submit / vote → 401.
    expect((await submitIdea(ctx, null, "anon idea")).status).toBe(401);
    expect((await voteIdea(ctx, null, id, "up")).status).toBe(401);
    // Neither anonymous call mutated anything.
    const ideas = await listIdeas(ctx); // the list itself needs no token (public)
    expect(ideas).toHaveLength(1);
    expect(voterCount(ideas, id)).toBe(0);
  });
});

describe("serialized shape round-trips through a real request cycle", () => {
  test("the server's Idea is byte-equivalent to the in-memory backing's", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");

    // Build the same table in the kernel's in-memory backing and on the server,
    // with the SAME user ids, then compare the serialized lists field for field.
    const meId = (await meOf(ctx, ada)).userId;
    const bobId = (await meOf(ctx, bob)).userId;

    const id1 = (await submitIdea(ctx, ada, "alpha")).body.idea.id;
    ctx.now.sec += DAY; // one idea per author per day — step to seed beta
    await submitIdea(ctx, ada, "beta");
    await voteIdea(ctx, ada, id1, "up");
    await voteIdea(ctx, bob, id1, "down");
    const serverList = await listIdeas(ctx);

    const mem = new InMemoryIdeaStore();
    const m1 = mem.submit("alpha", meId);
    mem.submit("beta", meId);
    mem.castVote(m1.id, meId, "up");
    mem.castVote(m1.id, bobId, "down");
    const memList = mem.list();

    // Same ids, seqs, text, authors, and directional vote maps in the same
    // ranked order — a JSON round-trip of one equals the other.
    expect(JSON.parse(JSON.stringify(serverList))).toEqual(JSON.parse(JSON.stringify(memList)));
  });
});

async function meOf(ctx: Ctx, token: string): Promise<{ userId: string }> {
  const res = await ctx.app.request("/v1/auth/me", { headers: { authorization: `Bearer ${token}` } });
  return (await res.json()) as { userId: string };
}

// ---------------------------------------------------------------------------
// Recorded lifecycle outcomes (#083 slice 2) — shipped/bounced written back to
// the idea, surfaced through the same list() the remote reads. Driven at the
// pure-function level over an in-memory DB (the recording is a build-tool path,
// not a public route in v1).
// ---------------------------------------------------------------------------

describe("recorded lifecycle outcomes (#083 slice 2)", () => {
  const depsOf = () => ({ db: openDb(":memory:").db, clock: () => 1_750_000_000 });

  test("a fresh idea is on-table with no bounce-reason key (byte-equivalent to in-memory)", () => {
    const deps = depsOf();
    submitIdeaFn(deps, "ada", "draft mode");
    const [idea] = listIdeasFn(deps);
    expect(idea!.status).toBe("on-table");
    expect("bounceReason" in idea!).toBe(false); // no stray null key
  });

  test("a bounce round-trips with its reason; votes intact and the idea stays re-votable", () => {
    const deps = depsOf();
    const sub = submitIdeaFn(deps, "ada", "draft mode");
    const id = (sub as { idea: Idea }).idea.id;
    castIdeaVote(deps, "bob", id, "up"); // a vote before the bounce

    // Selected → bounced with a visible reason (candidacy ≠ guarantee).
    const rec = recordBuildOutcome(deps, id, "bounced", "sim gauntlet: folds to poison");
    expect(rec).toMatchObject({ recorded: true });

    // Surfaced through the public list (what RemoteIdeas reads): status + reason.
    const bounced = listIdeasFn(deps).find((i) => i.id === id)!;
    expect(bounced.status).toBe("bounced");
    expect(bounced.bounceReason).toBe("sim gauntlet: folds to poison");
    expect(bounced.votes).toEqual({ bob: "up" }); // votes untouched by the bounce

    // Re-votable: a new vote still lands on the bounced idea.
    castIdeaVote(deps, "cat", id, "up");
    expect(listIdeasFn(deps).find((i) => i.id === id)!.votes).toEqual({ bob: "up", cat: "up" });

    // Shipping clears the stale bounce reason.
    recordBuildOutcome(deps, id, "shipped");
    const shipped = listIdeasFn(deps).find((i) => i.id === id)!;
    expect(shipped.status).toBe("shipped");
    expect(shipped.bounceReason).toBeUndefined();
  });

  test("recording an outcome on an unknown idea is rejected", () => {
    const res = recordBuildOutcome(depsOf(), "idea-999", "shipped");
    expect(res).toMatchObject({ recorded: false });
    expect((res as { reason: string }).reason).toMatch(/no idea/);
  });
});
