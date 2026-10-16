import { describe, expect, it } from "vitest";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { corsAllows, corsOriginsFromEnv } from "./cors.js";
import { mvpRuntime } from "./runtime.js";
import { mvpServerApp, underBasePath } from "./server.js";

function fetchWith(corsOrigins: string[]) {
  const rt = mvpRuntime({ content: mvpContent() });
  const app = mvpServerApp(createMvpApp(rt), { staticRoot: ".", build: null, corsOrigins });
  const fetch = underBasePath(app, "/arena");
  return (method: string, path: string, headers: Record<string, string> = {}) => fetch(new Request(`http://localhost${path}`, { method, headers }));
}

describe("CORS for the CrazyGames build", () => {
  it("allows crazygames.com, its subdomains and the env list, nothing else", () => {
    for (const o of ["https://crazygames.com", "https://www.crazygames.com", "https://games.crazygames.com", "https://a-1.game.crazygames.com"]) expect(corsAllows(o, []), o).toBe(true);
    for (const o of ["http://crazygames.com", "https://evilcrazygames.com", "https://crazygames.com.evil.com", "https://example.com", "null"]) expect(corsAllows(o, []), o).toBe(false);
    expect(corsAllows("http://localhost:9123", ["http://localhost:9123"])).toBe(true);
    expect(corsOriginsFromEnv({ MVP_CORS_ORIGINS: " http://localhost:9123/, https://x.test ," })).toEqual(["http://localhost:9123", "https://x.test"]);
    expect(corsOriginsFromEnv({})).toEqual([]);
  });

  it("answers a preflight with the token and content-type headers, no credentials", async () => {
    const call = fetchWith(["http://localhost:9123"]);
    for (const [origin, path] of [["https://www.crazygames.com", "/api/v1/players"], ["http://localhost:9123", "/arena/api/v1/players"]] as const) {
      const res = await call("OPTIONS", path, { Origin: origin, "Access-Control-Request-Method": "POST", "Access-Control-Request-Headers": "content-type,x-arena-token" });
      expect(res.status, origin).toBe(204);
      expect(res.headers.get("access-control-allow-origin")).toBe(origin);
      expect(res.headers.get("access-control-allow-headers")?.toLowerCase()).toContain("x-arena-token");
      expect(res.headers.get("access-control-allow-headers")?.toLowerCase()).toContain("content-type");
      expect(res.headers.get("access-control-allow-credentials")).toBeNull();
    }
    expect((await call("OPTIONS", "/api/v1/players", { Origin: "https://example.com" })).status).toBe(403);
  });

  it("tags API responses for an allowed origin only", async () => {
    const call = fetchWith([]);
    const ok = await call("GET", "/api/v1/health", { Origin: "https://games.crazygames.com" });
    expect(ok.status).toBe(200);
    expect(ok.headers.get("access-control-allow-origin")).toBe("https://games.crazygames.com");
    const other = await call("GET", "/api/v1/health", { Origin: "https://example.com" });
    expect(other.status).toBe(200);
    expect(other.headers.get("access-control-allow-origin")).toBeNull();
    const same = await call("GET", "/api/v1/health");
    expect(same.headers.get("access-control-allow-origin")).toBeNull();
    const notFound = await call("GET", "/api/v1/nope", { Origin: "https://crazygames.com" });
    expect(notFound.status).toBe(404);
    expect(notFound.headers.get("access-control-allow-origin")).toBe("https://crazygames.com");
  });
});
