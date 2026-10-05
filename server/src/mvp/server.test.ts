import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join, relative } from "node:path";
import { afterAll, describe, expect, it } from "vitest";
import type { PlayerRef, RunView } from "../../../src/mvp/contract.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { mvpRuntime } from "./runtime.js";
import { buildOf, mvpServerApp, underBasePath } from "./server.js";

const dir = mkdtempSync(join(tmpdir(), "arena-static-"));
writeFileSync(join(dir, "index.html"), "<!doctype html><title>arena</title>");
afterAll(() => rmSync(dir, { recursive: true, force: true }));

function server(build: string | null = "abc1234") {
  let n = 7;
  const rt = mvpRuntime({ content: mvpContent(), seed: () => (n = (n * 1103515245 + 12345) >>> 0) });
  const app = mvpServerApp(createMvpApp(rt), { staticRoot: relative(process.cwd(), dir) || ".", build });
  const fetch = underBasePath(app, "/arena");
  const call = async (method: string, path: string, body?: unknown, player?: string) => {
    const res = await fetch(
      new Request(`http://localhost${path}`, {
        method,
        headers: { "content-type": "application/json", ...(player ? { "X-Arena-Player": player } : {}) },
        ...(body !== undefined ? { body: JSON.stringify(body) } : {}),
      }),
    );
    return { status: res.status, type: res.headers.get("content-type") ?? "", text: await res.text() };
  };
  return { rt, call };
}

describe("the MVP's one origin", () => {
  it("reports the build on /health, with or without the base path", async () => {
    const { rt, call } = server();
    for (const path of ["/api/v1/health", "/arena/api/v1/health"]) {
      const res = await call("GET", path);
      expect(res.status).toBe(200);
      expect(JSON.parse(res.text)).toMatchObject({ ok: true, contentVersion: rt.content.version, build: "abc1234" });
    }
    const unknown = await server(null).call("GET", "/api/v1/health");
    expect(JSON.parse(unknown.text)).toMatchObject({ ok: true, build: null });
  });

  it("answers an unknown API path with a JSON 404, not the client's page", async () => {
    const { call } = server();
    for (const [method, path] of [["GET", "/api/v1/nope"], ["POST", "/api/v1/runs/x/nope"], ["GET", "/arena/api/v1/nope"], ["GET", "/api/v1"]] as const) {
      const res = await call(method, path);
      expect(res.status, `${method} ${path}`).toBe(404);
      expect(res.type).toContain("application/json");
      expect(JSON.parse(res.text).error).toMatch(/no such API path/);
    }
    // The API's own 404s keep their messages.
    const run = await call("GET", "/api/v1/runs/missing");
    expect([run.status, JSON.parse(run.text)]).toEqual([404, { error: "no such run" }]);
  });

  it("serves the client's index.html for the client's own routes", async () => {
    const { call } = server();
    for (const path of ["/", "/arena/", "/arena/run/123", "/stats"]) {
      const res = await call("GET", path);
      expect(res.status, path).toBe(200);
      expect(res.text).toContain("<title>arena</title>");
    }
  });

  it("answers 400 for a decision of an unknown kind, on /decisions and /preview", async () => {
    const { call } = server();
    const p = JSON.parse((await call("POST", "/api/v1/players", { name: "kind" })).text) as PlayerRef;
    const run = JSON.parse((await call("POST", "/api/v1/runs", undefined, p.id)).text) as RunView;
    for (const route of ["decisions", "preview"]) {
      const res = await call("POST", `/api/v1/runs/${run.runId}/${route}`, { kind: "zap" }, p.id);
      expect(res.status, route).toBe(400);
      expect(res.type).toContain("application/json");
      expect(JSON.parse(res.text)).toEqual({ error: 'unknown decision kind "zap"' });
    }
    // The run is untouched and still playable.
    const buy = await call("POST", `/api/v1/runs/${run.runId}/decisions`, { kind: "buy", slot: 0 }, p.id);
    expect(buy.status).toBe(200);
  });

  it("takes the build from MVP_BUILD first", () => {
    expect(buildOf({ MVP_BUILD: "deadbee" })).toBe("deadbee");
  });
});
