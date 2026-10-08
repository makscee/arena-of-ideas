/**
 * Arena MVP server (mission #574): the API plus the built phone client, one
 * origin, and the background jobs (./jobs.ts) on the same runtime. Env:
 *   PORT        listen port (default 8791; 8787 is /r on m1)
 *   HOST        bind address (default 127.0.0.1; tailscale serve fronts it)
 *   STATIC_DIR  built mobile client (default mobile/dist)
 *   BASE_PATH   public mount path (default /arena); requests may arrive with
 *               or without it, depending on how the proxy forwards them.
 *   MVP_DEV     1 serves the dev tools (/api/v1/dev/*, "end day now")
 *   MVP_INVITES 1 makes it invite-only (slice 13): players come from invite
 *               links (npm run mvp:invite), the dev tools only for admin invites
 *   MVP_DB      the SQLite file (default data/arena-mvp.db); ":memory:" keeps nothing
 *   ARENA_NAMER_URL  the fusion namer (OpenAI-compatible chat endpoint, slice
 *               10); without it every fusion gets the portmanteau
 *   MVP_BUILD   the deployed commit, `build` on /api/v1/health (default: the
 *               checkout's HEAD); scripts/mvp-redeploy.sh sets it
 * Run: npm run mvp:server
 */
import { serve } from "@hono/node-server";
import { mkdirSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import { createMvpApp } from "./app.js";
import { startMvpJobs } from "./jobs.js";
import { poolContent, seedUnits } from "./pool.js";
import { mvpRuntime } from "./runtime.js";
import { buildOf, mvpServerApp, underBasePath } from "./server.js";
import { SqliteMvpStore } from "./sqlite-store.js";

const port = Number(process.env.PORT ?? 8791);
const host = process.env.HOST ?? "127.0.0.1";
const basePath = (process.env.BASE_PATH ?? "/arena").replace(/\/$/, "");
const staticDir = resolve(process.env.STATIC_DIR ?? "mobile/dist");
const root = relative(process.cwd(), staticDir) || ".";

const dbPath = process.env.MVP_DB ?? "data/arena-mvp.db";
if (dbPath !== ":memory:") mkdirSync(dirname(resolve(dbPath)), { recursive: true });
const store = new SqliteMvpStore(dbPath);
// M2-1: the pool lives in the DB; the first start seeds it from the code pool.
seedUnits(store, new Date());
const content = poolContent(store);
const rt = mvpRuntime({ content, store, dev: process.env.MVP_DEV === "1", invites: process.env.MVP_INVITES === "1", open: process.env.MVP_OPEN === "1" });
const build = buildOf();
const app = mvpServerApp(createMvpApp(rt), { staticRoot: root, build });

serve({ port, hostname: host, fetch: underBasePath(app, basePath) });
startMvpJobs(rt);
console.log(`arena mvp on http://${host}:${port} (base ${basePath}, build ${build ?? "unknown"}, content ${content.version}, db ${dbPath}, static ${staticDir}${rt.dev ? ", dev" : ""}${rt.invites ? (rt.open ? ", open to all" : ", invite-only") : ""})`);
