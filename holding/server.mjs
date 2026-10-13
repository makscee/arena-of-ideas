// The holding server (#1044, mission 1043): what arena.makscee.ru/arena shows
// while Arena is rebuilt. Plain node, no dependencies, and it never opens the
// world's database: the live DB stays exactly as it was (no wipe, no
// migration, no day end, no rotation) until the reopen decides what to do with it.
// It serves the built phone client (mobile/dist), which draws the holding page
// when /api/v1/health says `dark`, and answers 503 on every other API route.
//
//   node holding/server.mjs        env: PORT (8791), HOST (127.0.0.1),
//                                  BASE_PATH (/arena), MVP_BUILD, DIST (mobile/dist)
// scripts/arena-dark.sh puts it on the live port under launchd.
import { createServer } from "node:http";
import { readFile, stat } from "node:fs/promises";
import { dirname, extname, join, normalize, resolve, sep } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const port = Number(process.env.PORT ?? 8791);
const host = process.env.HOST ?? "127.0.0.1";
const base = (process.env.BASE_PATH ?? "/arena").replace(/\/$/, "");
const build = process.env.MVP_BUILD ?? null;
const dist = resolve(process.env.DIST ?? join(here, "..", "mobile", "dist"));

const TYPES = {
  ".html": "text/html; charset=utf-8", ".js": "text/javascript; charset=utf-8", ".mjs": "text/javascript; charset=utf-8",
  ".css": "text/css; charset=utf-8", ".json": "application/json", ".svg": "image/svg+xml", ".png": "image/png",
  ".jpg": "image/jpeg", ".webp": "image/webp", ".ico": "image/x-icon", ".woff2": "font/woff2",
  ".mp3": "audio/mpeg", ".ogg": "audio/ogg", ".wav": "audio/wav", ".txt": "text/plain; charset=utf-8", ".webmanifest": "application/manifest+json",
};

const json = (res, status, body) => {
  res.writeHead(status, { "content-type": "application/json", "cache-control": "no-store" });
  res.end(JSON.stringify(body));
};

async function file(path) {
  const full = normalize(join(dist, path));
  if (full !== dist && !full.startsWith(dist + sep)) return null;
  try {
    if ((await stat(full)).isFile()) return full;
  } catch {
    /* no such file */
  }
  return null;
}

export const server = createServer(async (req, res) => {
  const url = new URL(req.url ?? "/", "http://holding");
  let path;
  try {
    path = decodeURIComponent(url.pathname);
  } catch {
    return json(res, 400, { error: "bad path" });
  }
  // Requests arrive with the base path or without it (tailscale serve strips its mount path), as in server/src/mvp/main.ts.
  const underBase = path === base || path.startsWith(base + "/");
  const rest = underBase ? path.slice(base.length) || "/" : path;
  if (rest === "/api/v1/health") return json(res, 200, { ok: true, dark: true, build });
  if (rest.startsWith("/api/")) return json(res, 503, { error: "Arena is being rebuilt" });
  if (req.method !== "GET" && req.method !== "HEAD") return json(res, 405, { error: "method not allowed" });
  if (path === base) {
    res.writeHead(301, { location: base + "/" + url.search });
    return res.end();
  }
  const asset = rest === "/" ? null : await file(rest);
  const target = asset ?? (await file("index.html"));
  if (!target) return json(res, 503, { error: "the client isn't built (npm run mvp:build)" });
  const body = await readFile(target);
  res.writeHead(200, {
    "content-type": TYPES[extname(target)] ?? "application/octet-stream",
    // The page itself is never cached, so the reopen takes effect at once; built assets are hashed.
    "cache-control": asset && rest.startsWith("/assets/") ? "public, max-age=31536000, immutable" : "no-cache",
  });
  res.end(req.method === "HEAD" ? undefined : body);
});

if (process.argv[1] === fileURLToPath(import.meta.url)) {
  server.listen(port, host, () => console.log(`holding server on http://${host}:${port}${base}/ (dist ${dist}, build ${build ?? "unknown"})`));
}
