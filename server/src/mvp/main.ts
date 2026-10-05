/**
 * Arena MVP server (mission #574): the API plus the built phone client, one
 * origin, and the background jobs (./jobs.ts) on the same runtime. Env:
 *   PORT        listen port (default 8791; 8787 is /r on m1)
 *   HOST        bind address (default 127.0.0.1; tailscale serve fronts it)
 *   STATIC_DIR  built mobile client (default mobile/dist)
 *   BASE_PATH   public mount path (default /arena); requests may arrive with
 *               or without it, depending on how the proxy forwards them.
 *   MVP_DEV     1 serves the dev tools (/api/v1/dev/*, "end day now")
 *   MVP_DB      the SQLite file (default data/arena-mvp.db); ":memory:" keeps nothing
 * Run: npm run mvp:server
 */
import { serve } from "@hono/node-server";
import { serveStatic } from "@hono/node-server/serve-static";
import { Hono } from "hono";
import { mkdirSync } from "node:fs";
import { dirname, relative, resolve } from "node:path";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { startMvpJobs } from "./jobs.js";
import { mvpRuntime } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";

const port = Number(process.env.PORT ?? 8791);
const host = process.env.HOST ?? "127.0.0.1";
const basePath = (process.env.BASE_PATH ?? "/arena").replace(/\/$/, "");
const staticDir = resolve(process.env.STATIC_DIR ?? "mobile/dist");
const root = relative(process.cwd(), staticDir) || ".";

const dbPath = process.env.MVP_DB ?? "data/arena-mvp.db";
if (dbPath !== ":memory:") mkdirSync(dirname(resolve(dbPath)), { recursive: true });
const store = new SqliteMvpStore(dbPath);
const content = mvpContent();
const rt = mvpRuntime({ content, store, dev: process.env.MVP_DEV === "1" });
const api = createMvpApp(rt);
const app = new Hono();
app.route("/", api);
app.use("/*", serveStatic({ root }));
app.get("/*", serveStatic({ root, path: "index.html" }));

serve({
  port,
  hostname: host,
  fetch: (req) => {
    const url = new URL(req.url);
    if (basePath && (url.pathname === basePath || url.pathname.startsWith(basePath + "/"))) {
      url.pathname = url.pathname.slice(basePath.length) || "/";
      return app.fetch(new Request(url, req));
    }
    return app.fetch(req);
  },
});
startMvpJobs(rt);
console.log(`arena mvp on http://${host}:${port} (base ${basePath}, content ${content.version}, db ${dbPath}, static ${staticDir}${rt.dev ? ", dev" : ""})`);
