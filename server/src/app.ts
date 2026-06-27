/**
 * Arena server app factory. Endpoint contract follows void-auth's login
 * routes (src/routes/login.api.ts), minus cookies and registration:
 *
 *   POST /v1/auth/login/email/start   issue a 6-digit code, send via mail
 *   POST /v1/auth/login/email/verify  verify code, mint session, return token
 *   POST /v1/auth/logout              revoke the current session
 *   GET  /v1/auth/me                  current session + user
 *   POST /v1/auth/display-name        set the caller's display name (slice 3:
 *                                     the first-login name pick; what the
 *                                     leaderboard and creator credit show)
 *   GET  /healthz                     liveness
 *
 * Shared ladder (slice 2) — one ladder per server instance, opened from the
 * kernel's bootstrap at factory time (idempotent: a played-on DB is never
 * reseeded), users identified by the session's user id:
 *
 *   GET  /v1/ladder/champion          public — current champion + holder name
 *   GET  /v1/ladder/pool/:round       public — the round's ghost pool;
 *                                     ?exclude=me (bearer) = the pool as the
 *                                     caller's runs see it, own ghosts out
 *   POST /v1/runs/open                bearer — open a run BEFORE playing it
 *                                     (one-shot, expires after the open TTL)
 *   GET  /v1/runs/:runId/pool/:round  bearer — THE play read: the round's
 *                                     pool as this run sees it + the seated
 *                                     champion; the server records the view
 *                                     and replay accepts only recorded views
 *   POST /v1/runs                     bearer — submit a finished run for
 *                                     re-derivation (runs.ts)
 *
 * Ideas table (#076 slice 2) — the free-text idea queue, ranked by votes, one
 * vote per player (DB-enforced on the session's user id):
 *
 *   GET  /v1/ideas                    public — every idea, ranked by votes
 *   POST /v1/ideas                    bearer — submit an idea (the author)
 *   POST /v1/ideas/:id/vote           bearer — toggle the caller's vote
 *
 * Listing is public for the same reason the leaderboard reads are: reading the
 * ranked queue logged-out is reasonable, and the ideas are not secrets. Only
 * submitting and voting need identity — both key on session.userId.
 *
 * Leaderboard reads are deliberately public: the title screen shows the
 * champion to logged-out players, and ghost teams are not secrets — they are
 * the opponents everyone fights. Only submission (writing) needs identity.
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
import { openLadder } from "../../src/index.js";
import { createAuthMiddleware, type AuthEnv } from "./auth.js";
import { defaultArenaContent, type ArenaContent } from "./content.js";
import type { DB } from "./db.js";
import { createEmailCodes, OTP_TTL_SECONDS } from "./email-codes.js";
import { SqliteLadderStore } from "./ladder-store.js";
import type { MailClient } from "./mail.js";
import { renderOtpEmail } from "./otp-email.js";
import { createRateLimiter, type RateLimiter } from "./rate-limit.js";
import { listIdeas, submitIdea, toggleIdeaVote } from "./ideas.js";
import { openRun, servePool, submitRun } from "./runs.js";
import { users } from "./schema.js";
import { mint, revoke, verify } from "./sessions.js";

const SESSION_LIFETIME_DAYS = 30;
const SESSION_LABEL = "arena-web";
const OTP_FROM_NAME = "Arena of Ideas";

export interface AppDeps {
  db: DB;
  /** Unix seconds. */
  clock: () => number;
  mailClient: MailClient;
  rateLimiters: { ipStart: RateLimiter; emailStart: RateLimiter; poolServe: RateLimiter };
  /** The run content submissions are pinned to; defaults to the arena's
   * shipped pool + approved registry. Tests inject a tiny deterministic pool. */
  content?: ArenaContent;
}

/** Prod limiters: 5 starts per IP and 5 per email, per 10 minutes; 300 pool
 * serves per session per 10 minutes. The serve limit is sized off honest play
 * — one serve per ladder fight, so even a speedrun reads a few per minute;
 * 300/10min (one every 2s) is an order of magnitude of headroom, while a
 * round-sweeping client (each distinct round = one recorded row) gets cut off
 * three decimal orders below the old MAX_POOL_ROUND amplification. */
export function defaultRateLimiters(): AppDeps["rateLimiters"] {
  const windowMs = 10 * 60 * 1000;
  return {
    ipStart: createRateLimiter({ limit: 5, windowMs }),
    emailStart: createRateLimiter({ limit: 5, windowMs }),
    poolServe: createRateLimiter({ limit: 300, windowMs }),
  };
}

// Pragmatic shape check, not RFC 5322 — void-mail does the real bounce.
const EMAIL_RE = /^[^\s@]+@[^\s@]+\.[^\s@]+$/;
const CODE_RE = /^\d{6}$/;

/** Display names: 2–24 chars, letters/digits to start, then also spaces and
 * light punctuation. Unicode letters allowed — names are identity, not slugs;
 * control characters, angle brackets and the like have nowhere to live here. */
const DISPLAY_NAME_RE = /^[\p{L}\p{N}][\p{L}\p{N} _.'-]{1,23}$/u;

/** Pool reads are bounded: no real run climbs anywhere near this, so a bigger
 * :round is junk input, not a query. */
const MAX_POOL_ROUND = 10_000;

/** How far past the ladder's deepest ghost round a play read may go. Honest
 * play needs at most frontier + 1: a round with no visible ghosts is the
 * kernel's outran-every-ghost champion challenge and the run ends there, so
 * serves beyond the frontier are a sweep, not a game — each distinct round
 * writes a run_pool_serves row, and without this bound one open run could
 * record MAX_POOL_ROUND of them. */
const SERVE_ROUND_MARGIN = 16;

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
  const content = deps.content ?? defaultArenaContent();
  const emailCodes = createEmailCodes(db, clock);
  const auth = createAuthMiddleware({ db, clock });
  // The shared ladder, bootstrap-seated at open (kernel rule: a vacant
  // champion spot is a free crown — openLadder never leaves one).
  const store = new SqliteLadderStore(db);
  openLadder(store, content.statuses, content.abilities);

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
    // response shape is constant by design. The operator still sees them.
    const sent = await mailClient.send({
      to: email,
      subject: rendered.subject,
      html: rendered.html,
      text: rendered.text,
      fromName: OTP_FROM_NAME,
    });
    if (!sent.ok) {
      console.error(`[mail] OTP send to ${email} failed: ${sent.error}`);
    }

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

  // The first-login name pick (slice 3). Setting it again just renames — the
  // name is display identity, not a key; ghosts and crowns hang off user id.
  app.post("/v1/auth/display-name", auth, async (c) => {
    const body = await jsonBody(c.req.raw);
    const raw = body?.displayName;
    const name = typeof raw === "string" ? raw.trim() : "";
    if (!DISPLAY_NAME_RE.test(name)) {
      return c.json({ error: "invalid_display_name" }, 400);
    }
    const session = c.get("session");
    db.update(users)
      .set({ displayName: name, updatedAt: clock() })
      .where(eq(users.id, session.userId))
      .run();
    return c.json({ displayName: name }, 200);
  });

  app.get("/v1/ladder/champion", (c) => {
    const rec = store.championRecord();
    if (rec === null) return c.json({ champion: null, holder: null });
    const holder =
      rec.userId === null
        ? null
        : (db.select().from(users).where(eq(users.id, rec.userId)).limit(1).all()[0]?.displayName ?? null);
    return c.json({ champion: rec.snap, holder });
  });

  app.get("/v1/ladder/pool/:round", (c) => {
    const round = Number(c.req.param("round"));
    if (!Number.isInteger(round) || round < 1 || round > MAX_POOL_ROUND) {
      return c.json({ error: "invalid_round" }, 400);
    }
    if (c.req.query("exclude") !== "me") {
      return c.json({ round, pool: store.poolAt(round) });
    }
    // Play reads: the pool as this user's runs see it — own ghosts (across
    // all the user's runs) excluded. Needs identity, hence the bearer.
    const header = c.req.header("authorization");
    const token = header?.startsWith("Bearer ") ? header.slice("Bearer ".length).trim() : "";
    const session = token === "" ? null : verify(db, token, clock);
    if (session === null) return c.json({ error: "unauthorized" }, 401);
    return c.json({ round, pool: store.poolVisibleTo(round, session.userId) });
  });

  app.post("/v1/runs/open", auth, async (c) => {
    const body = await jsonBody(c.req.raw);
    const runId = body?.runId;
    if (typeof runId !== "string") {
      return c.json({ error: "invalid_body" }, 400);
    }
    const session = c.get("session");
    const outcome = openRun({ db, store, clock }, session.userId, runId);
    return c.json(outcome, outcome.opened ? 200 : 422);
  });

  app.get("/v1/runs/:runId/pool/:round", auth, (c) => {
    const round = Number(c.req.param("round"));
    if (!Number.isInteger(round) || round < 1 || round > MAX_POOL_ROUND) {
      return c.json({ error: "invalid_round" }, 400);
    }
    const session = c.get("session");
    // Authed write-amplification guards (each serve can record a row): a
    // per-session rate limit with generous honest headroom, and a frontier
    // bound — no run plays rounds the ladder's ghosts don't reach.
    const limited = rateLimiters.poolServe.check(`serve:${session.sessionId}`);
    if (!limited.ok) {
      return c.json({ error: "rate_limited", retryAfterMs: limited.retryAfterMs }, 429);
    }
    const frontier = store.maxPoolRound() + SERVE_ROUND_MARGIN;
    if (round > frontier) {
      return c.json(
        { served: false, reason: `round ${round} is beyond the ladder's frontier — no pool to serve past round ${frontier}` },
        422,
      );
    }
    const outcome = servePool({ db, store, clock }, session.userId, c.req.param("runId"), round);
    return c.json(outcome, outcome.served ? 200 : 422);
  });

  app.post("/v1/runs", auth, async (c) => {
    const body = await jsonBody(c.req.raw);
    const run = body?.run;
    if (typeof run !== "string") {
      return c.json({ error: "invalid_body" }, 400);
    }
    const session = c.get("session");
    const outcome = submitRun({ db, store, content, clock }, session.userId, run);
    return c.json(outcome, outcome.accepted ? 200 : 422);
  });

  // Ideas table — public list, authed submit/vote (one vote per session user).
  app.get("/v1/ideas", (c) => {
    return c.json({ ideas: listIdeas({ db, clock }) });
  });

  app.post("/v1/ideas", auth, async (c) => {
    const body = await jsonBody(c.req.raw);
    const text = body?.text;
    if (typeof text !== "string") {
      return c.json({ error: "invalid_body" }, 400);
    }
    const session = c.get("session");
    const outcome = submitIdea({ db, clock }, session.userId, text);
    return c.json(outcome, outcome.submitted ? 200 : 422);
  });

  app.post("/v1/ideas/:id/vote", auth, (c) => {
    const session = c.get("session");
    const outcome = toggleIdeaVote({ db, clock }, session.userId, c.req.param("id"));
    return c.json(outcome, outcome.toggled ? 200 : 422);
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
