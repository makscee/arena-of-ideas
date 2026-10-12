import { describe, expect, it } from "vitest";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { mvpRuntime } from "./runtime.js";
import { checkScenesEnv, SCENES } from "./scenes.js";

describe("test scenes (#1026)", () => {
  it("start only on a dev server with an in-memory world", () => {
    expect(checkScenesEnv({})).toBe(false);
    expect(checkScenesEnv({ ARENA_SCENES: "1", MVP_DEV: "1", MVP_DB: ":memory:" })).toBe(true);
    expect(() => checkScenesEnv({ ARENA_SCENES: "1", MVP_DB: ":memory:" })).toThrow(/MVP_DEV=1/);
    expect(() => checkScenesEnv({ ARENA_SCENES: "1", MVP_DEV: "1", MVP_DB: "data/arena-mvp.db" })).toThrow(/:memory:/);
    expect(() => checkScenesEnv({ ARENA_SCENES: "1", MVP_DEV: "1" })).toThrow(/:memory:/);
    expect(() => checkScenesEnv({ ARENA_SCENES: "1", MVP_DEV: "1", MVP_DB: ":memory:", MVP_INVITES: "1" })).toThrow(/MVP_INVITES/);
  });

  it("answer 404 on a dev server that isn't a scenes server", async () => {
    const app = createMvpApp(mvpRuntime({ content: mvpContent(), dev: true }));
    expect((await app.request("/api/v1/dev/scenes")).status).toBe(404);
    expect((await app.request("/api/v1/dev/scenes/home", { method: "POST" })).status).toBe(404);
  });

  it("list every scene and build a quick one", async () => {
    const app = createMvpApp(mvpRuntime({ content: mvpContent(), dev: true, scenes: true }));
    const list = (await (await app.request("/api/v1/dev/scenes")).json()) as { id: string }[];
    expect(list.map((s) => s.id)).toEqual(SCENES.map((s) => s.id));
    expect(new Set(list.map((s) => s.id)).size).toBe(list.length);
    const res = await app.request("/api/v1/dev/scenes/shop-r1", { method: "POST" });
    expect(res.status).toBe(200);
    const built = (await res.json()) as { player: { id: string }; entry: { screen: string; runId: string } };
    expect(built.entry.screen).toBe("shop");
    expect((await app.request(`/api/v1/runs/${built.entry.runId}`, { headers: { "X-Arena-Player": built.player.id } })).status).toBe(200);
    expect((await app.request("/api/v1/dev/scenes/vote", { method: "POST" })).status).toBe(200);
    expect((await app.request("/api/v1/dev/scenes/nope", { method: "POST" })).status).toBe(404);
  }, 120_000);
});
