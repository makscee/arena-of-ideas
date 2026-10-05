/**
 * Arena MVP server (mission #574): the API plus the built phone client, one
 * origin. Env:
 *   PORT        listen port (default 8791; 8787 is /r on m1)
 *   HOST        bind address (default 127.0.0.1; tailscale serve fronts it)
 *   STATIC_DIR  built mobile client (default mobile/dist)
 *   BASE_PATH   public mount path (default /arena); requests may arrive with
 *               or without it, depending on how the proxy forwards them.
 * Run: npm run mvp:server
 */
import { serve } from "@hono/node-server";
import { serveStatic } from "@hono/node-server/serve-static";
import { Hono } from "hono";
import { relative, resolve } from "node:path";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";

const port = Number(process.env.PORT ?? 8791);
const host = process.env.HOST ?? "127.0.0.1";
const basePath = (process.env.BASE_PATH ?? "/arena").replace(/\/$/, "");
const staticDir = resolve(process.env.STATIC_DIR ?? "mobile/dist");
const root = relative(process.cwd(), staticDir) || ".";

const content = mvpContent();
const api = createMvpApp({ content });
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
console.log(`arena mvp on http://${host}:${port} (base ${basePath}, content ${content.version}, static ${staticDir})`);
