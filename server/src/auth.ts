/**
 * Session auth middleware. Adapted from void-auth (src/middleware/auth.ts):
 * bearer header only — the arena client stores the token itself, no cookies.
 *
 * Extracts `Authorization: Bearer <token>`, verifies it against the sessions
 * table, and either attaches the resolved session to the context or returns
 * 401. Factory so tests can inject a fixed clock and DB. Later slices mount
 * this in front of the ladder API.
 */
import type { Context, MiddlewareHandler, Next } from "hono";
import type { DB } from "./db.js";
import { verify, type SessionInfo } from "./sessions.js";

export interface AuthDeps {
  db: DB;
  /** Unix seconds. */
  clock: () => number;
}

/** Hono env carrying the verified session for downstream handlers. */
export type AuthEnv = { Variables: { session: SessionInfo } };

export function createAuthMiddleware(deps: AuthDeps): MiddlewareHandler<AuthEnv> {
  return async (c: Context<AuthEnv>, next: Next) => {
    const header = c.req.header("authorization");
    if (!header || !header.startsWith("Bearer ")) {
      return c.json({ error: "unauthorized" }, 401);
    }
    const token = header.slice("Bearer ".length).trim();
    if (!token) {
      return c.json({ error: "unauthorized" }, 401);
    }
    const session = verify(deps.db, token, deps.clock);
    if (!session) {
      return c.json({ error: "unauthorized" }, 401);
    }
    c.set("session", session);
    await next();
    return;
  };
}
