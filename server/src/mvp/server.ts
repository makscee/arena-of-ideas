// The MVP's one origin (mission #574), as main.ts serves it: the API
// (./app.ts) with the build on /health, a JSON 404 for any other /api/v1 path,
// then the built phone client, with index.html for every other GET (the
// client's own routes). Here rather than in main.ts so tests can call it.
import { execFileSync } from "node:child_process";
import { serveStatic } from "@hono/node-server/serve-static";
import { Hono } from "hono";
import { MVP_API_PREFIX } from "../../../src/mvp/contract.js";

export interface MvpServerOptions {
  /** The built client: serveStatic's root, relative to the cwd. */
  staticRoot: string;
  /** The deployed commit, reported as `build` on /health; null when unknown. */
  build: string | null;
}

export function mvpServerApp(api: Hono, o: MvpServerOptions): Hono {
  const app = new Hono();
  // GET /health is app.ts's; this adds the build to whatever it answers.
  app.use(`${MVP_API_PREFIX}/health`, async (c, next) => {
    await next();
    if (c.res.status !== 200 || !c.res.headers.get("content-type")?.includes("application/json")) return;
    const body = (await c.res.json()) as Record<string, unknown>;
    c.res = undefined; // a fresh response, not merged with the old one's headers
    c.res = c.json({ ...body, build: o.build });
  });
  app.route("/", api);
  // An API path no route answered is a 404 for the caller, not the client's page.
  app.all(MVP_API_PREFIX, (c) => c.json({ error: `no such API path: ${c.req.method} ${c.req.path}` }, 404));
  app.all(`${MVP_API_PREFIX}/*`, (c) => c.json({ error: `no such API path: ${c.req.method} ${c.req.path}` }, 404));
  // Hono knows no .m4a: the music (round 4) must not go out as a download,
  // or Safari won't play it.
  app.use("/*", async (c, next) => {
    await next();
    if (c.req.path.endsWith(".m4a") && c.res.headers.get("content-type") === "application/octet-stream") c.res.headers.set("Content-Type", "audio/mp4");
  });
  app.use("/*", serveStatic({ root: o.staticRoot }));
  app.get("/*", serveStatic({ root: o.staticRoot, path: "index.html" }));
  return app;
}

/** The app's fetch for requests that arrive with or without the public mount
 * path (BASE_PATH), depending on how the proxy forwards them. */
export function underBasePath(app: Hono, basePath: string): (req: Request) => Response | Promise<Response> {
  const base = basePath.replace(/\/$/, "");
  return (req) => {
    const url = new URL(req.url);
    if (base && (url.pathname === base || url.pathname.startsWith(base + "/"))) {
      url.pathname = url.pathname.slice(base.length) || "/";
      return app.fetch(new Request(url, req));
    }
    return app.fetch(req);
  };
}

/** The build to report: MVP_BUILD (scripts/mvp-redeploy.sh sets it to the
 * deployed commit), else the checkout's HEAD, else null. */
export function buildOf(env: NodeJS.ProcessEnv = process.env): string | null {
  if (env.MVP_BUILD) return env.MVP_BUILD;
  try {
    return execFileSync("git", ["rev-parse", "--short", "HEAD"], { encoding: "utf8", stdio: ["ignore", "pipe", "ignore"] }).trim() || null;
  } catch {
    return null;
  }
}
