/**
 * Arena server entrypoint. Config via env only:
 *
 *   MAIL_BASE_URL  required — void-mail base URL (e.g. https://mail.example.com)
 *   MAIL_TOKEN     required — void-mail bearer token
 *   DB_PATH        optional — SQLite file path (default ./data/arena.db)
 *   PORT           optional — listen port (default 8787)
 *   STATIC_DIR     optional — directory of the built web client (vite dist/);
 *                  when set, the server serves it same-origin so one container
 *                  ships the whole game. API routes keep precedence (they are
 *                  registered first); unknown non-API GETs fall back to
 *                  index.html. Unset = API-only (dev: vite proxies instead).
 *   MOCK_MODE      optional — "1" swaps the real mailer for an in-memory mock
 *                  and mounts GET /_mock/last-code?email=… returning the last
 *                  OTP code sent to that address (the void-mail _mock/last
 *                  pattern). For the e2e harness ONLY: no mail leaves the
 *                  process, and the codes endpoint must never exist in prod —
 *                  hence env-gated at boot, not a runtime flag.
 *
 * Run: `npm start --workspace server` (tsx, matching the arena toolchain).
 */
import { serve } from "@hono/node-server";
import { serveStatic } from "@hono/node-server/serve-static";
import { join, relative } from "node:path";
import { seedBootstrapTower } from "../../src/index.js";
import { createApp, defaultRateLimiters } from "./app.js";
import { startCleanupTimer } from "./cleanup.js";
import { defaultArenaContent } from "./content.js";
import { openDb } from "./db.js";
import { SqliteLadderStore } from "./ladder-store.js";
import { createMailClient, createMockMailClient } from "./mail.js";

function requireEnv(name: string): string {
  const v = process.env[name];
  if (!v) {
    console.error(`Missing required env var ${name}`);
    process.exit(1);
  }
  return v;
}

/** A garbage PORT must die at boot, not become NaN and a bind surprise. */
function parsePort(raw: string | undefined): number {
  if (raw === undefined || raw === "") return 8787;
  const n = Number(raw);
  if (!Number.isInteger(n) || n < 1 || n > 65535) {
    console.error(`Invalid PORT "${raw}" — expected an integer in 1..65535`);
    process.exit(1);
  }
  return n;
}

const mockMode = process.env.MOCK_MODE === "1";
const mockMailer = mockMode ? createMockMailClient() : null;
const mailClient =
  mockMailer ?? createMailClient({ baseUrl: requireEnv("MAIL_BASE_URL"), token: requireEnv("MAIL_TOKEN") });
const dbPath = process.env.DB_PATH ?? "./data/arena.db";
const port = parsePort(process.env.PORT);

const clock = (): number => Math.floor(Date.now() / 1000);
const { db } = openDb(dbPath);
const app = createApp({
  db,
  clock,
  mailClient,
  rateLimiters: defaultRateLimiters(),
});
startCleanupTimer(db, clock);

// Production launches EMPTY (PRD #085: createApp → openEmptyLadder; play founds
// the tower). The solo-playtest / e2e harness, which needs a populated tower up
// front (a champion to read, a ladder to climb), opts into the #075 full-tower
// seed via AOI_SEED_BOOTSTRAP — the explicit solo-playtest seam, never set in
// production. One player (or a probe suite) cannot found a whole tower alone.
if (process.env.AOI_SEED_BOOTSTRAP === "1") {
  const content = defaultArenaContent();
  seedBootstrapTower(new SqliteLadderStore(db), content.statuses, content.abilities);
  console.log("AOI_SEED_BOOTSTRAP: seeded the bootstrap tower (solo-playtest / e2e)");
}

if (mockMailer !== null) {
  console.log("MOCK_MODE: mail is mocked; /_mock/last-code is mounted");
  app.get("/_mock/last-code", (c) => {
    const email = c.req.query("email");
    const last = [...mockMailer.sent].reverse().find((m) => m.to === email);
    const code = last?.text.match(/\b(\d{6})\b/)?.[1];
    if (code === undefined) return c.json({ error: "no_code_sent" }, 404);
    return c.json({ email, code });
  });
}

// Static web client (deployment: one container ships the whole game). The
// client talks same-origin relative URLs ("/v1/..."), so serving it here
// needs no CORS at all. Registered after every API route — Hono dispatches
// in registration order, so /v1/*, /healthz and /_mock keep precedence.
const staticDir = process.env.STATIC_DIR;
if (staticDir !== undefined && staticDir !== "") {
  // serve-static resolves roots against cwd; normalize whatever was given.
  const root = relative(process.cwd(), staticDir) || ".";
  app.use("*", serveStatic({ root }));
  // SPA fallback for non-API GETs (the client routes via location.hash, so
  // this mostly covers "/" and stray deep links). API misses stay JSON 404s.
  app.get("*", (c, next) => {
    if (c.req.path.startsWith("/v1/")) return next();
    return serveStatic({ path: join(root, "index.html") })(c, next);
  });
  console.log(`serving static client from ${staticDir}`);
}

serve({ fetch: app.fetch, port }, (info) => {
  console.log(`arena server listening on :${info.port} (db: ${dbPath})`);
});
