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

const mailBaseUrl = requireEnv("MAIL_BASE_URL");
const mailToken = requireEnv("MAIL_TOKEN");
const dbPath = process.env.DB_PATH ?? "./data/arena.db";
const port = Number(process.env.PORT ?? 8787);

const { db } = openDb(dbPath);
const app = createApp({
  db,
  clock: () => Math.floor(Date.now() / 1000),
  mailClient: createMailClient({ baseUrl: mailBaseUrl, token: mailToken }),
  rateLimiters: defaultRateLimiters(),
});

serve({ fetch: app.fetch, port }, (info) => {
  console.log(`arena server listening on :${info.port} (db: ${dbPath})`);
});
