/**
 * Arena server app factory. Endpoint contract follows void-auth's login
 * routes (src/routes/login.api.ts), minus cookies and registration:
 *
 *   POST /v1/auth/login/email/start   issue a 6-digit code, send via mail
 *   POST /v1/auth/login/email/verify  verify code, mint session, return token
 *   POST /v1/auth/logout              revoke the current session
 *   GET  /v1/auth/me                  current session + user
 *   GET  /healthz                     liveness
 *
 * Pure factory — all dependencies injected; no module-level globals. Tests
 * wire the mock mailer + tiny-window rate limiters; prod (main.ts) wires the
 * real void-mail client + 5/10min limiters.
 *
 * No-enumeration property: /start behaves identically whether or not the
 * email belongs to a user — it always issues a code, always sends mail, and
 * always answers `{"sent":true}`. Users are created lazily on first
 * successful /verify.
 */
import { Hono } from "hono";
import { randomUUID } from "node:crypto";
import { eq } from "drizzle-orm";
import { createAuthMiddleware, type AuthEnv } from "./auth.js";
import type { DB } from "./db.js";
import { createEmailCodes, OTP_TTL_SECONDS } from "./email-codes.js";
import type { MailClient } from "./mail.js";
import { renderOtpEmail } from "./otp-email.js";
import { createRateLimiter, type RateLimiter } from "./rate-limit.js";
import { users } from "./schema.js";
import { mint, revoke } from "./sessions.js";

const SESSION_LIFETIME_DAYS = 30;
const SESSION_LABEL = "arena-web";

export interface AppDeps {
  db: DB;
  /** Unix seconds. */
  clock: () => number;
  mailClient: MailClient;
  rateLimiters: { ipStart: RateLimiter; emailStart: RateLimiter };
}

/** Prod limiters: 5 starts per IP and 5 per email, per 10 minutes. */
export function defaultRateLimiters(): AppDeps["rateLimiters"] {
  const windowMs = 10 * 60 * 1000;
  return {
    ipStart: createRateLimiter({ limit: 5, windowMs }),
    emailStart: createRateLimiter({ limit: 5, windowMs }),
  };
}

// Pragmatic shape check, not RFC 5322 — void-mail does the real bounce.
const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const CODE_RE = /^\d{6}$/;

async function jsonBody(req: Request): Promise<Record<string, unknown> | null> {
  try {
    const body: unknown = await req.json();
    if (typeof body !== "object" || body === null || Array.isArray(body)) return null;
    return body as Record<string, unknown>;
  } catch {
    return null;
  }
}

export function createApp(deps: AppDeps): Hono<AuthEnv> {
  const { db, clock, mailClient, rateLimiters } = deps;
  const emailCodes = createEmailCodes(db, clock);
  const auth = createAuthMiddleware({ db, clock });

  const app = new Hono<AuthEnv>();

  app.get("/healthz", (c) => c.json({ ok: true }));

  app.post("/v1/auth/login/email/start", async (c) => {
    const body = await jsonBody(c.req.raw);
    const email = body?.email;
    if (typeof email !== "string" || !EMAIL_RE.test(email)) {
      return c.json({ error: "invalid_email" }, 400);
    }

    // Process-local limiter keyed on the XFF header; fine while one proxy
    // hop fronts one container (same caveat as void-auth's phase-1 limiter).
    const ip = c.req.header("x-forwarded-for") ?? "unknown";
    const ipResult = rateLimiters.ipStart.check(`ip:${ip}`);
    if (!ipResult.ok) {
      return c.json({ error: "rate_limited", retryAfterMs: ipResult.retryAfterMs }, 429);
    }
    const emailResult = rateLimiters.emailStart.check(`email:${email}`);
    if (!emailResult.ok) {
      return c.json({ error: "rate_limited", retryAfterMs: emailResult.retryAfterMs }, 429);
    }

    const { code } = emailCodes.issue(email);
    const rendered = renderOtpEmail({ code, ttl: OTP_TTL_SECONDS });
    // Send failures are not leaked to clients (still 200 sent:true) — the
    // response shape is constant by design.
    await mailClient.send({
      to: email,
      subject: rendered.subject,
      html: rendered.html,
      text: rendered.text,
    });

    return c.json({ sent: true as const }, 200);
  });

  app.post("/v1/auth/login/email/verify", async (c) => {
    const body = await jsonBody(c.req.raw);
    const email = body?.email;
    const code = body?.code;
    if (typeof email !== "string" || !EMAIL_RE.test(email)) {
      return c.json({ error: "invalid_email" }, 400);
    }
    if (typeof code !== "string" || !CODE_RE.test(code)) {
      return c.json({ error: "invalid_code" }, 400);
    }

    const result = emailCodes.verify(email, code);
    if (!result.ok) {
      return c.json({ error: result.reason }, 401);
    }

    // Auto-create user on first login; displayName stays null until the
    // first-login name pick (later slice).
    const existing = db.select().from(users).where(eq(users.email, email)).limit(1).all()[0];
    let userId: string;
    if (existing) {
      userId = existing.id;
    } else {
      userId = randomUUID();
      const now = clock();
      db.insert(users)
        .values({ id: userId, email, displayName: null, createdAt: now, updatedAt: now })
        .run();
    }

    const minted = mint(db, { userId, label: SESSION_LABEL, lifetimeDays: SESSION_LIFETIME_DAYS }, clock);
    return c.json(
      { token: minted.token, sessionId: minted.sessionId, expiresAt: minted.expiresAt },
      200,
    );
  });

  app.post("/v1/auth/logout", auth, (c) => {
    const session = c.get("session");
    revoke(db, session.sessionId);
    return c.json({ ok: true as const }, 200);
  });

  app.get("/v1/auth/me", auth, (c) => {
    const session = c.get("session");
    return c.json(
      {
        userId: session.userId,
        email: session.email,
        displayName: session.displayName,
        sessionId: session.sessionId,
        expiresAt: session.expiresAt,
      },
      200,
    );
  });

  return app;
}
