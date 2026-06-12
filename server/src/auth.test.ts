import { createHash } from "node:crypto";
import { describe, expect, test, vi } from "vitest";
import type { Hono } from "hono";
import { createApp } from "./app.js";
import type { AuthEnv } from "./auth.js";
import { openDb } from "./db.js";
import { createMockMailClient, type MailClient, type MockMailClient } from "./mail.js";
import { createRateLimiter } from "./rate-limit.js";
import { emailCodes, sessions, users } from "./schema.js";

// ---------------------------------------------------------------------------
// Harness: in-process Hono app, in-memory SQLite, mock mailer, fake clock.
// ---------------------------------------------------------------------------

const WINDOW_MS = 10 * 60 * 1000;
const DAY = 86400;

interface Ctx {
  app: Hono<AuthEnv>;
  db: ReturnType<typeof openDb>["db"];
  mailer: MockMailClient;
  /** Advance the fake clock by `seconds` (drives TTLs and rate windows). */
  advance: (seconds: number) => void;
}

function makeCtx(): Ctx {
  const { db } = openDb(":memory:");
  const mailer = createMockMailClient();
  let nowSec = 1_750_000_000;
  const clock = () => nowSec;
  const clockMs = () => nowSec * 1000;
  const app = createApp({
    db,
    clock,
    mailClient: mailer,
    rateLimiters: {
      ipStart: createRateLimiter({ limit: 5, windowMs: WINDOW_MS, clock: clockMs }),
      emailStart: createRateLimiter({ limit: 5, windowMs: WINDOW_MS, clock: clockMs }),
      poolServe: createRateLimiter({ limit: 100, windowMs: WINDOW_MS, clock: clockMs }),
    },
  });
  return { app, db, mailer, advance: (s) => (nowSec += s) };
}

function post(
  app: Hono<AuthEnv>,
  path: string,
  body: unknown,
  headers: Record<string, string> = {},
): Promise<Response> {
  return Promise.resolve(
    app.request(path, {
      method: "POST",
      headers: { "content-type": "application/json", ...headers },
      body: JSON.stringify(body),
    }),
  );
}

function me(app: Hono<AuthEnv>, token: string): Promise<Response> {
  return Promise.resolve(
    app.request("/v1/auth/me", { headers: { authorization: `Bearer ${token}` } }),
  );
}

/** The 6-digit code from the most recent mock-mailed message. */
function lastCode(mailer: MockMailClient): string {
  const last = mailer.sent[mailer.sent.length - 1];
  if (!last) throw new Error("no mail sent");
  const m = last.text.match(/\b(\d{6})\b/);
  if (!m) throw new Error(`no code in mail text: ${last.text}`);
  return m[1]!;
}

/** A 6-digit code guaranteed different from `code`. */
function notThe(code: string): string {
  return code === "000000" ? "111111" : "000000";
}

const sha256 = (s: string) => createHash("sha256").update(s).digest("hex");

async function login(ctx: Ctx, email: string, ip = "10.0.0.1"): Promise<string> {
  const start = await post(ctx.app, "/v1/auth/login/email/start", { email }, { "x-forwarded-for": ip });
  expect(start.status).toBe(200);
  const verify = await post(ctx.app, "/v1/auth/login/email/verify", {
    email,
    code: lastCode(ctx.mailer),
  });
  expect(verify.status).toBe(200);
  const body = (await verify.json()) as { token: string };
  return body.token;
}

// ---------------------------------------------------------------------------
// Happy path
// ---------------------------------------------------------------------------

describe("OTP login happy path", () => {
  test("start sends a code, verify mints a session, bearer authenticates", async () => {
    const ctx = makeCtx();

    const start = await post(ctx.app, "/v1/auth/login/email/start", { email: "maks@example.com" });
    expect(start.status).toBe(200);
    expect(await start.json()).toEqual({ sent: true });
    expect(ctx.mailer.sent).toHaveLength(1);
    expect(ctx.mailer.sent[0]!.to).toBe("maks@example.com");

    const verify = await post(ctx.app, "/v1/auth/login/email/verify", {
      email: "maks@example.com",
      code: lastCode(ctx.mailer),
    });
    expect(verify.status).toBe(200);
    const session = (await verify.json()) as { token: string; sessionId: string; expiresAt: number };
    expect(session.token).toBeTruthy();
    expect(session.expiresAt).toBe(1_750_000_000 + 30 * DAY);

    // User auto-created, displayName null until the first-login name pick.
    const rows = ctx.db.select().from(users).all();
    expect(rows).toHaveLength(1);
    expect(rows[0]!.email).toBe("maks@example.com");
    expect(rows[0]!.displayName).toBeNull();

    const authed = await me(ctx.app, session.token);
    expect(authed.status).toBe(200);
    const meBody = (await authed.json()) as { userId: string; email: string; displayName: null };
    expect(meBody.userId).toBe(rows[0]!.id);
    expect(meBody.email).toBe("maks@example.com");
    expect(meBody.displayName).toBeNull();
  });

  test("second login reuses the existing user", async () => {
    const ctx = makeCtx();
    await login(ctx, "maks@example.com", "10.0.0.1");
    await login(ctx, "maks@example.com", "10.0.0.2");
    expect(ctx.db.select().from(users).all()).toHaveLength(1);
  });

  test("malformed bodies are rejected with 400", async () => {
    const ctx = makeCtx();
    expect((await post(ctx.app, "/v1/auth/login/email/start", { email: "not-an-email" })).status).toBe(400);
    expect((await post(ctx.app, "/v1/auth/login/email/verify", { email: "a@b.co", code: "12345" })).status).toBe(400);
    expect((await post(ctx.app, "/v1/auth/login/email/verify", { email: "a@b.co", code: 123456 })).status).toBe(400);
  });
});

// ---------------------------------------------------------------------------
// Code lifecycle: wrong / exhausted / expired
// ---------------------------------------------------------------------------

describe("code lifecycle", () => {
  test("wrong code is rejected and increments attempts", async () => {
    const ctx = makeCtx();
    await post(ctx.app, "/v1/auth/login/email/start", { email: "a@b.co" });
    const code = lastCode(ctx.mailer);

    const res = await post(ctx.app, "/v1/auth/login/email/verify", { email: "a@b.co", code: notThe(code) });
    expect(res.status).toBe(401);
    expect(await res.json()).toEqual({ error: "wrong" });
    expect(ctx.db.select().from(emailCodes).all()[0]!.attempts).toBe(1);

    // Still valid: correct code succeeds within the attempt cap.
    const ok = await post(ctx.app, "/v1/auth/login/email/verify", { email: "a@b.co", code });
    expect(ok.status).toBe(200);
  });

  test("5 wrong attempts exhaust the code; the right code no longer works", async () => {
    const ctx = makeCtx();
    await post(ctx.app, "/v1/auth/login/email/start", { email: "a@b.co" });
    const code = lastCode(ctx.mailer);

    for (let i = 1; i <= 5; i++) {
      const res = await post(ctx.app, "/v1/auth/login/email/verify", { email: "a@b.co", code: notThe(code) });
      expect(res.status).toBe(401);
      expect(((await res.json()) as { error: string }).error).toBe(i < 5 ? "wrong" : "exhausted");
    }
    expect(ctx.db.select().from(emailCodes).all()[0]!.attempts).toBe(5);

    const res = await post(ctx.app, "/v1/auth/login/email/verify", { email: "a@b.co", code });
    expect(res.status).toBe(401);
    expect(await res.json()).toEqual({ error: "exhausted" });
  });

  test("code expires after the 10-minute TTL", async () => {
    const ctx = makeCtx();
    await post(ctx.app, "/v1/auth/login/email/start", { email: "a@b.co" });
    const code = lastCode(ctx.mailer);

    ctx.advance(10 * 60 + 1);
    const res = await post(ctx.app, "/v1/auth/login/email/verify", { email: "a@b.co", code });
    expect(res.status).toBe(401);
    expect(await res.json()).toEqual({ error: "expired" });
  });

  test("verify with no code ever issued", async () => {
    const ctx = makeCtx();
    const res = await post(ctx.app, "/v1/auth/login/email/verify", { email: "no@one.co", code: "123456" });
    expect(res.status).toBe(401);
    expect(await res.json()).toEqual({ error: "no_code" });
  });
});

// ---------------------------------------------------------------------------
// Rate limits
// ---------------------------------------------------------------------------

describe("start rate limits", () => {
  test("per IP: 6th start from one IP is 429; window expiry frees it", async () => {
    const ctx = makeCtx();
    const ip = { "x-forwarded-for": "1.2.3.4" };
    for (let i = 0; i < 5; i++) {
      const res = await post(ctx.app, "/v1/auth/login/email/start", { email: `u${i}@b.co` }, ip);
      expect(res.status).toBe(200);
    }
    const blocked = await post(ctx.app, "/v1/auth/login/email/start", { email: "u5@b.co" }, ip);
    expect(blocked.status).toBe(429);
    const body = (await blocked.json()) as { error: string; retryAfterMs: number };
    expect(body.error).toBe("rate_limited");
    expect(body.retryAfterMs).toBeGreaterThan(0);

    // Other IPs are unaffected.
    const other = await post(ctx.app, "/v1/auth/login/email/start", { email: "u6@b.co" }, { "x-forwarded-for": "5.6.7.8" });
    expect(other.status).toBe(200);

    ctx.advance(10 * 60 + 1);
    const after = await post(ctx.app, "/v1/auth/login/email/start", { email: "u7@b.co" }, ip);
    expect(after.status).toBe(200);
  });

  test("per email: 6th start for one email is 429 even across IPs", async () => {
    const ctx = makeCtx();
    for (let i = 0; i < 5; i++) {
      const res = await post(
        ctx.app,
        "/v1/auth/login/email/start",
        { email: "hot@b.co" },
        { "x-forwarded-for": `10.0.0.${i}` },
      );
      expect(res.status).toBe(200);
    }
    const blocked = await post(
      ctx.app,
      "/v1/auth/login/email/start",
      { email: "hot@b.co" },
      { "x-forwarded-for": "10.0.0.99" },
    );
    expect(blocked.status).toBe(429);
    expect(((await blocked.json()) as { error: string }).error).toBe("rate_limited");
  });
});

// ---------------------------------------------------------------------------
// No enumeration
// ---------------------------------------------------------------------------

describe("no email-existence enumeration", () => {
  test("start responds identically for existing and unknown emails", async () => {
    const ctx = makeCtx();
    await login(ctx, "existing@b.co"); // existing@b.co now has a user row
    ctx.mailer.reset();

    const known = await post(ctx.app, "/v1/auth/login/email/start", { email: "existing@b.co" });
    const unknown = await post(ctx.app, "/v1/auth/login/email/start", { email: "stranger@b.co" });

    expect(known.status).toBe(200);
    expect(unknown.status).toBe(200);
    expect(await known.text()).toBe(await unknown.text());
    // Both get a real code mail — behavior, not just the response, matches.
    expect(ctx.mailer.sent.map((m) => m.to)).toEqual(["existing@b.co", "stranger@b.co"]);
  });
});

// ---------------------------------------------------------------------------
// Session verification
// ---------------------------------------------------------------------------

describe("session verification", () => {
  test("accepts a valid bearer, rejects garbage/missing/expired/revoked", async () => {
    const ctx = makeCtx();
    const token = await login(ctx, "a@b.co");

    expect((await me(ctx.app, token)).status).toBe(200);
    expect((await me(ctx.app, "not-a-token")).status).toBe(401);
    expect((await ctx.app.request("/v1/auth/me")).status).toBe(401);

    // Logout revokes; the token dies immediately.
    const logout = await post(ctx.app, "/v1/auth/logout", {}, { authorization: `Bearer ${token}` });
    expect(logout.status).toBe(200);
    expect((await me(ctx.app, token)).status).toBe(401);

    // A fresh session expires after the 30-day lifetime.
    const token2 = await login(ctx, "a@b.co", "10.0.0.2");
    expect((await me(ctx.app, token2)).status).toBe(200);
    ctx.advance(30 * DAY + 1);
    expect((await me(ctx.app, token2)).status).toBe(401);
  });
});

// ---------------------------------------------------------------------------
// Display name (slice 3): the first-login name pick
// ---------------------------------------------------------------------------

describe("display name", () => {
  test("starts null, set persists to /me, renaming overwrites", async () => {
    const ctx = makeCtx();
    const token = await login(ctx, "ada@b.co");
    const auth = { authorization: `Bearer ${token}` };

    let who = (await (await me(ctx.app, token)).json()) as { displayName: string | null };
    expect(who.displayName).toBeNull(); // first login: no name until the pick

    const set = await post(ctx.app, "/v1/auth/display-name", { displayName: "Ada L." }, auth);
    expect(set.status).toBe(200);
    expect(await set.json()).toEqual({ displayName: "Ada L." });
    who = (await (await me(ctx.app, token)).json()) as { displayName: string | null };
    expect(who.displayName).toBe("Ada L.");

    // Renaming just renames — display identity, not a key.
    const again = await post(ctx.app, "/v1/auth/display-name", { displayName: "Ada" }, auth);
    expect(again.status).toBe(200);
    who = (await (await me(ctx.app, token)).json()) as { displayName: string | null };
    expect(who.displayName).toBe("Ada");
  });

  test("requires auth and rejects bad shapes/lengths/charsets", async () => {
    const ctx = makeCtx();
    const token = await login(ctx, "ada@b.co");
    const auth = { authorization: `Bearer ${token}` };

    expect((await post(ctx.app, "/v1/auth/display-name", { displayName: "Ada" })).status).toBe(401);

    const bad = [
      undefined, // missing
      42, // not a string
      "A", // too short
      "x".repeat(25), // too long
      " leading-dash-after-trim-is-fine-but-this-starts-with-punct".slice(0, 24).replace(/^ /, "-"),
      "<script>alert(1)</script>",
      "two\nlines",
    ];
    for (const displayName of bad) {
      const res = await post(ctx.app, "/v1/auth/display-name", { displayName }, auth);
      expect(res.status, `displayName ${JSON.stringify(displayName)}`).toBe(400);
    }

    // Whitespace trims before validation; unicode letters are fine.
    const ok = await post(ctx.app, "/v1/auth/display-name", { displayName: "  Максим Ч.  " }, auth);
    expect(ok.status).toBe(200);
    expect(await ok.json()).toEqual({ displayName: "Максим Ч." });
  });
});

// ---------------------------------------------------------------------------
// Storage hygiene: raw codes and tokens never touch the DB
// ---------------------------------------------------------------------------

describe("storage hygiene", () => {
  test("codes and tokens are stored sha256-only", async () => {
    const ctx = makeCtx();
    await post(ctx.app, "/v1/auth/login/email/start", { email: "a@b.co" });
    const code = lastCode(ctx.mailer);
    const verify = await post(ctx.app, "/v1/auth/login/email/verify", { email: "a@b.co", code });
    const { token } = (await verify.json()) as { token: string };

    const codeRow = ctx.db.select().from(emailCodes).all()[0]!;
    expect(codeRow.codeHash).toBe(sha256(code));
    expect(codeRow.codeHash).toMatch(/^[0-9a-f]{64}$/);
    // No column anywhere in the row carries the raw code.
    for (const value of Object.values(codeRow)) {
      expect(value).not.toBe(code);
    }

    const sessionRow = ctx.db.select().from(sessions).all()[0]!;
    expect(sessionRow.tokenHash).toBe(sha256(token));
    expect(sessionRow.tokenHash).toMatch(/^[0-9a-f]{64}$/);
    for (const value of Object.values(sessionRow)) {
      expect(value).not.toBe(token);
    }
  });
});

// ---------------------------------------------------------------------------
// Mail send failure: constant response shape, but never silent server-side
// ---------------------------------------------------------------------------

describe("mail send failure", () => {
  test("/start still answers {sent:true}; the failure lands in the server log", async () => {
    const { db } = openDb(":memory:");
    const failingMailer: MailClient = { send: async () => ({ ok: false, error: "smtp boom" }) };
    const clockMs = () => 1_750_000_000_000;
    const app = createApp({
      db,
      clock: () => 1_750_000_000,
      mailClient: failingMailer,
      rateLimiters: {
        ipStart: createRateLimiter({ limit: 5, windowMs: WINDOW_MS, clock: clockMs }),
        emailStart: createRateLimiter({ limit: 5, windowMs: WINDOW_MS, clock: clockMs }),
        poolServe: createRateLimiter({ limit: 100, windowMs: WINDOW_MS, clock: clockMs }),
      },
    });
    const errSpy = vi.spyOn(console, "error").mockImplementation(() => {});
    try {
      const res = await post(app, "/v1/auth/login/email/start", { email: "ada@example.com" });
      expect(res.status).toBe(200);
      expect(await res.json()).toEqual({ sent: true }); // shape stays constant — no enumeration, no leak
      expect(errSpy).toHaveBeenCalledWith(expect.stringContaining("smtp boom"));
    } finally {
      errSpy.mockRestore();
    }
  });
});
