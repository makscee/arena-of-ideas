/**
 * Arena server entrypoint. Config via env only:
 *
 *   MAIL_BASE_URL  required — void-mail base URL (e.g. https://mail.example.com)
 *   MAIL_TOKEN     required — void-mail bearer token
 *   DB_PATH        optional — SQLite file path (default ./data/arena.db)
 *   PORT           optional — listen port (default 8787)
 *
 * Run: `npm start --workspace server` (tsx, matching the arena toolchain).
 */
import { serve } from "@hono/node-server";
import { createApp, defaultRateLimiters } from "./app.js";
import { startCleanupTimer } from "./cleanup.js";
import { openDb } from "./db.js";
import { createMailClient } from "./mail.js";

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

const mailBaseUrl = requireEnv("MAIL_BASE_URL");
const mailToken = requireEnv("MAIL_TOKEN");
const dbPath = process.env.DB_PATH ?? "./data/arena.db";
const port = parsePort(process.env.PORT);

const clock = (): number => Math.floor(Date.now() / 1000);
const { db } = openDb(dbPath);
const app = createApp({
  db,
  clock,
  mailClient: createMailClient({ baseUrl: mailBaseUrl, token: mailToken }),
  rateLimiters: defaultRateLimiters(),
});
startCleanupTimer(db, clock);

serve({ fetch: app.fetch, port }, (info) => {
  console.log(`arena server listening on :${info.port} (db: ${dbPath})`);
});
