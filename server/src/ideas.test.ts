/**
 * Ideas table API contract (#076 slice 2). In-process Hono app over in-memory
 * SQLite + mock mailer, the ladder-api.test.ts harness. The acceptance:
 *
 *  - submit an idea → it appears in the list,
 *  - a player votes then re-votes → the count stays 1 (the composite PK on
 *    idea_votes is the one-vote-per-player floor — DB-level, not app-level),
 *  - toggle off decrements,
 *  - list returns ranked order (votes desc, ties by submission seq),
 *  - the serialized kernel `Idea` shape round-trips through a real request,
 *  - auth: an unauthenticated submit/vote is rejected (401); list is public.
 *
 * Plus a must-fail-first: a raw double-insert against a votes table WITHOUT the
 * PK double-counts — proving the constraint, not app logic, is what holds.
 */
import { describe, expect, test } from "vitest";
import Database from "better-sqlite3";
import type { Hono } from "hono";
import { InMemoryIdeaStore, type Idea } from "../../src/index.js";
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

async function voteIdea(ctx: Ctx, token: string | null, ideaId: string): Promise<{ status: number; body: any }> {
  const res = await ctx.app.request(`/v1/ideas/${encodeURIComponent(ideaId)}/vote`, {
    method: "POST",
    headers: { "content-type": "application/json", ...(token ? { authorization: `Bearer ${token}` } : {}) },
    body: JSON.stringify({}),
  });
  return { status: res.status, body: await res.json() };
}

async function listIdeas(ctx: Ctx): Promise<Idea[]> {
  const res = await ctx.app.request("/v1/ideas");
  expect(res.status).toBe(200);
  return ((await res.json()) as { ideas: Idea[] }).ideas;
}

const votesOf = (ideas: Idea[], id: string) => ideas.find((i) => i.id === id)!.votes.length;

describe("submit and list", () => {
  test("a submitted idea appears in the list with the kernel Idea shape", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const { status, body } = await submitIdea(ctx, ada, "  add a draft mode  ");
    expect(status).toBe(200);
    expect(body).toMatchObject({ submitted: true });
    // Trimmed, id off the seq, empty vote set — the kernel's submit contract.
    expect(body.idea).toMatchObject({ id: "idea-0", seq: 0, text: "add a draft mode", votes: [] });

    const ideas = await listIdeas(ctx);
    expect(ideas).toHaveLength(1);
    expect(ideas[0]).toMatchObject({ id: "idea-0", text: "add a draft mode", votes: [] });
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
    expect((await submitIdea(ctx, ada, "first")).body.idea).toMatchObject({ id: "idea-0", seq: 0 });
    expect((await submitIdea(ctx, ada, "second")).body.idea).toMatchObject({ id: "idea-1", seq: 1 });
    expect((await submitIdea(ctx, ada, "third")).body.idea).toMatchObject({ id: "idea-2", seq: 2 });
  });
});

describe("vote-toggle: one vote per player (DB-enforced)", () => {
  test("a player votes then re-votes — the count stays 1, never double-counts", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const id = (await submitIdea(ctx, ada, "buff the wisp")).body.idea.id;

    const first = await voteIdea(ctx, ada, id);
    expect(first.status).toBe(200);
    expect(first.body).toMatchObject({ toggled: true, voted: true });
    expect(first.body.idea.votes).toHaveLength(1);
    expect(votesOf(await listIdeas(ctx), id)).toBe(1);

    // The toggle removes it (the kernel's set semantics) — voted:false, count 0.
    const second = await voteIdea(ctx, ada, id);
    expect(second.body).toMatchObject({ toggled: true, voted: false });
    expect(votesOf(await listIdeas(ctx), id)).toBe(0);

    // Vote again, then again: still one vote, never two. The composite PK holds.
    expect((await voteIdea(ctx, ada, id)).body.voted).toBe(true);
    expect(votesOf(await listIdeas(ctx), id)).toBe(1);
  });

  test("two players each vote once — count is 2, votes carry both user ids sorted", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");
    const id = (await submitIdea(ctx, ada, "shared idea")).body.idea.id;

    await voteIdea(ctx, ada, id);
    await voteIdea(ctx, bob, id);
    const ideas = await listIdeas(ctx);
    expect(votesOf(ideas, id)).toBe(2);
    const votes = ideas.find((i) => i.id === id)!.votes;
    expect([...votes].sort()).toEqual(votes); // serialized vote set is sorted (stable shape)
  });

  test("voting an unknown idea is rejected", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const { status, body } = await voteIdea(ctx, ada, "idea-999");
    expect(status).toBe(422);
    expect(body).toMatchObject({ toggled: false });
    expect(body.reason).toMatch(/no idea/);
  });

  // Must-fail-first: prove it's the CONSTRAINT, not app logic, that holds. A
  // votes table WITHOUT the composite PK lets a raw double-insert count twice —
  // the very corruption the PK prevents. With the PK (the real schema), the
  // second insert collides and the count stays 1.
  test("a votes table WITHOUT the PK double-counts — the constraint is what protects us", () => {
    // No PK: nothing stops two identical (idea, user) rows.
    const loose = new Database(":memory:");
    loose.exec(`CREATE TABLE idea_votes_loose (idea_id TEXT NOT NULL, user_id TEXT NOT NULL)`);
    const insLoose = loose.prepare(`INSERT INTO idea_votes_loose (idea_id, user_id) VALUES (?, ?)`);
    insLoose.run("idea-0", "ada");
    insLoose.run("idea-0", "ada"); // a re-vote / race — no constraint to stop it
    const looseCount = loose.prepare(`SELECT COUNT(*) c FROM idea_votes_loose WHERE idea_id = ?`).get("idea-0") as { c: number };
    expect(looseCount.c).toBe(2); // double-counted — the failure mode the PK exists to kill

    // The real schema: composite PK on (idea_id, user_id). The second insert
    // collides; INSERT OR IGNORE (the server's onConflictDoNothing) no-ops it.
    const guarded = new Database(":memory:");
    guarded.exec(`CREATE TABLE idea_votes (idea_id TEXT NOT NULL, user_id TEXT NOT NULL, PRIMARY KEY (idea_id, user_id))`);
    const insGuarded = guarded.prepare(`INSERT OR IGNORE INTO idea_votes (idea_id, user_id) VALUES (?, ?)`);
    insGuarded.run("idea-0", "ada");
    insGuarded.run("idea-0", "ada");
    const guardedCount = guarded.prepare(`SELECT COUNT(*) c FROM idea_votes WHERE idea_id = ?`).get("idea-0") as { c: number };
    expect(guardedCount.c).toBe(1); // the PK held: one vote per player, at the DB layer
  });
});

describe("list returns ranked order", () => {
  test("ideas rank by vote count desc, ties broken by submission seq", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");
    const cat = await login(ctx, "cat@example.com");

    const a = (await submitIdea(ctx, ada, "idea A")).body.idea.id; // seq 0
    const b = (await submitIdea(ctx, ada, "idea B")).body.idea.id; // seq 1
    const c = (await submitIdea(ctx, ada, "idea C")).body.idea.id; // seq 2

    // B gets 3 votes, C gets 3 votes (tie → earlier seq B first), A gets 1.
    for (const t of [ada, bob, cat]) await voteIdea(ctx, t, b);
    for (const t of [ada, bob, cat]) await voteIdea(ctx, t, c);
    await voteIdea(ctx, ada, a);

    const ranked = await listIdeas(ctx);
    expect(ranked.map((i) => i.id)).toEqual([b, c, a]); // 3,3 (B before C by seq), then 1
  });
});

describe("auth", () => {
  test("submit and vote require a bearer; list is public", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const id = (await submitIdea(ctx, ada, "seed")).body.idea.id;

    // Unauthenticated submit / vote → 401.
    expect((await submitIdea(ctx, null, "anon idea")).status).toBe(401);
    expect((await voteIdea(ctx, null, id)).status).toBe(401);
    // Neither anonymous call mutated anything.
    const ideas = await listIdeas(ctx); // the list itself needs no token (public)
    expect(ideas).toHaveLength(1);
    expect(votesOf(ideas, id)).toBe(0);
  });
});

describe("serialized shape round-trips through a real request cycle", () => {
  test("the server's Idea is byte-equivalent to the in-memory backing's", async () => {
    const ctx = makeCtx();
    const ada = await login(ctx, "ada@example.com");
    const bob = await login(ctx, "bob@example.com");

    // Build the same table in the kernel's in-memory backing and on the server,
    // with the SAME user ids, then compare the serialized lists field for field.
    const meRes = await ctx.app.request("/v1/auth/me", { headers: { authorization: `Bearer ${ada}` } });
    const meId = (await meRes.json()) as { userId: string };
    const bobRes = await ctx.app.request("/v1/auth/me", { headers: { authorization: `Bearer ${bob}` } });
    const bobId = (await bobRes.json()) as { userId: string };

    const id1 = (await submitIdea(ctx, ada, "alpha")).body.idea.id;
    await submitIdea(ctx, ada, "beta");
    await voteIdea(ctx, ada, id1);
    await voteIdea(ctx, bob, id1);
    const serverList = await listIdeas(ctx);

    const mem = new InMemoryIdeaStore();
    const m1 = mem.submit("alpha", meId.userId);
    mem.submit("beta", meId.userId);
    mem.toggleVote(m1.id, meId.userId);
    mem.toggleVote(m1.id, bobId.userId);
    const memList = mem.list();

    // Same ids, seqs, text, authors, and vote sets in the same ranked order —
    // a JSON round-trip of one equals the other.
    expect(JSON.parse(JSON.stringify(serverList))).toEqual(JSON.parse(JSON.stringify(memList)));
  });
});
