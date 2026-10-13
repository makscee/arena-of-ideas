// The holding server (#1044): dark health, 503 on the API, the client for everything else, no escape from dist.
import type { AddressInfo } from "node:net";
import { mkdirSync, mkdtempSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, beforeAll, describe, expect, it } from "vitest";

let url = "";
let close = () => {};

beforeAll(async () => {
  const root = mkdtempSync(join(tmpdir(), "holding-"));
  const dist = join(root, "dist");
  mkdirSync(dist);
  mkdirSync(join(dist, "assets"));
  writeFileSync(join(dist, "index.html"), "<!doctype html><title>client</title>");
  writeFileSync(join(dist, "assets", "app-abc.js"), "console.log(1)");
  writeFileSync(join(root, "secret.txt"), "outside dist");
  process.env.DIST = dist;
  process.env.MVP_BUILD = "abc123 (holding)";
  const { server } = await import("./server.mjs");
  await new Promise<void>((ok) => server.listen(0, "127.0.0.1", ok));
  url = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
  close = () => server.close();
});
afterAll(() => close());

describe("holding server", () => {
  it("says it is dark on /health", async () => {
    const res = await fetch(`${url}/arena/api/v1/health`);
    expect(res.status).toBe(200);
    expect(await res.json()).toEqual({ ok: true, dark: true, build: "abc123 (holding)" });
  });

  it("closes every other API route with 503, whatever the method", async () => {
    for (const [method, path] of [["GET", "/api/v1/home"], ["POST", "/api/v1/runs"], ["POST", "/api/v1/players"], ["GET", "/api/v1/dev/scenes"]]) {
      const res = await fetch(`${url}/arena${path}`, { method });
      expect(res.status, `${method} ${path}`).toBe(503);
    }
  });

  it("serves the client at the base and under any path, assets as they are", async () => {
    expect(await (await fetch(`${url}/arena/`)).text()).toContain("<title>client</title>");
    expect(await (await fetch(`${url}/arena/some/deep/link`)).text()).toContain("<title>client</title>");
    const js = await fetch(`${url}/arena/assets/app-abc.js`);
    expect(js.headers.get("content-type")).toContain("javascript");
    expect(await js.text()).toBe("console.log(1)");
    const bare = await fetch(`${url}/arena`, { redirect: "manual" });
    expect(bare.status).toBe(301);
    expect(bare.headers.get("location")).toBe("/arena/");
  });

  it("answers the same without the base path, as when tailscale serve strips it", async () => {
    expect(await (await fetch(`${url}/api/v1/health`)).json()).toMatchObject({ dark: true });
    expect((await fetch(`${url}/api/v1/home`)).status).toBe(503);
    expect(await (await fetch(`${url}/`)).text()).toContain("<title>client</title>");
    expect((await fetch(`${url}/assets/app-abc.js`)).status).toBe(200);
  });

  it("never reads outside dist", async () => {
    const res = await fetch(`${url}/arena/..%2fsecret.txt`);
    expect(await res.text()).not.toContain("outside dist");
  });

  it("takes no writes on the client routes", async () => {
    expect((await fetch(`${url}/arena/`, { method: "POST" })).status).toBe(405);
  });
});
