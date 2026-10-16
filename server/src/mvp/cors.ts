// CORS for the portable CrazyGames build (mission #1214): the game's zip runs
// on CrazyGames' hosts and calls this server from there. The token travels in
// the X-Arena-Token header, not a cookie, so no credentials are allowed.
import type { MiddlewareHandler } from "hono";
import { MVP_API_PREFIX, PLAYER_HEADER, TOKEN_HEADER } from "../../../src/mvp/contract.js";

/** CrazyGames' web origins: the site and every subdomain (the QA/preview host
 * and the game-files hosts live under it). Its docs name no exact hosts. */
const CRAZYGAMES = /^https:\/\/([a-z0-9-]+\.)*crazygames\.com$/;

/** `MVP_CORS_ORIGINS`: exact origins, comma-separated. */
export function corsOriginsFromEnv(env: NodeJS.ProcessEnv = process.env): string[] {
  return (env.MVP_CORS_ORIGINS ?? "").split(",").map((s) => s.trim().replace(/\/$/, "")).filter(Boolean);
}

export function corsAllows(origin: string, extra: readonly string[]): boolean {
  return CRAZYGAMES.test(origin) || extra.includes(origin);
}

/** Answers the API paths' preflights and tags their responses for an allowed
 * Origin; any other caller gets no CORS headers (the browser refuses it). */
export function cors(extra: readonly string[]): MiddlewareHandler {
  return async (c, next) => {
    const origin = c.req.header("Origin");
    const api = c.req.path === MVP_API_PREFIX || c.req.path.startsWith(`${MVP_API_PREFIX}/`);
    if (!origin || !api) return next();
    const allowed = corsAllows(origin, extra);
    if (c.req.method === "OPTIONS") {
      if (!allowed) return c.body(null, 403);
      return c.body(null, 204, {
        "Access-Control-Allow-Origin": origin,
        "Access-Control-Allow-Methods": "GET, POST, PUT, PATCH, DELETE, OPTIONS",
        "Access-Control-Allow-Headers": `Content-Type, ${TOKEN_HEADER}, ${PLAYER_HEADER}, Accept-Language`,
        "Access-Control-Max-Age": "86400",
        Vary: "Origin",
      });
    }
    await next();
    if (allowed) c.res.headers.set("Access-Control-Allow-Origin", origin);
    c.res.headers.append("Vary", "Origin");
  };
}
