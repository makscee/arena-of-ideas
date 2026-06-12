/**
 * Arena server entrypoint. Config via env only:
 *
 *   MAIL_BASE_URL  required — void-mail base URL (e.g. https://mail.example.com)
 *   MAIL_TOKEN     required — void-mail bearer token
 *   DB_PATH        optional — SQLite file path (default ./data/arena.db)
 *   PORT           optional — listen port (default 8787)
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
import { createApp, defaultRateLimiters } from "./app.js";
import { startCleanupTimer } from "./cleanup.js";
import { openDb } from "./db.js";
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

serve({ fetch: app.fetch, port }, (info) => {
  console.log(`arena server listening on :${info.port} (db: ${dbPath})`);
});
