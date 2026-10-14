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

  it("open the forge in each Codex state, with the Codex on (#1049)", async () => {
    const app = createMvpApp(mvpRuntime({ content: mvpContent(), dev: true, scenes: true, codex: true }));
    for (const state of ["unforged", "ready", "discovered"]) {
      const built = (await (await app.request(`/api/v1/dev/scenes/forge-${state}`, { method: "POST" })).json()) as { player: { id: string }; entry: { runId: string; forge: [number, number] } };
      const forge = (await (await app.request(`/api/v1/runs/${built.entry.runId}/forge`, { headers: { "X-Arena-Player": built.player.id } })).json()) as { pairs: { first: number; second: number; state: string; candidates?: unknown[] }[] };
      const slot = forge.pairs.find((p) => p.first === built.entry.forge[0] && p.second === built.entry.forge[1]);
      expect(slot?.state).toBe(state);
      if (state === "ready") expect(slot?.candidates).toHaveLength(3);
    }
    const claim = (await (await app.request("/api/v1/dev/scenes/forge-claim", { method: "POST" })).json()) as { entry: { click?: string } };
    expect(claim.entry.click).toBe("forge-claim");
    // Off, the forge scenes say why instead of building.
    const off = createMvpApp(mvpRuntime({ content: mvpContent(), dev: true, scenes: true }));
    expect((await off.request("/api/v1/dev/scenes/forge-ready", { method: "POST" })).status).not.toBe(200);
  }, 240_000);

  it("set the shop's fusion marks: none, one now, one a buy away (#1049 reopened)", async () => {
    const app = createMvpApp(mvpRuntime({ content: mvpContent(), dev: true, scenes: true, codex: true }));
    type Forge = { pairs: { first: number; second: number; state: string }[]; offers: { slot: number; partner: number; state: string; awakens: boolean }[] };
    const forgeOf = async (id: string) => {
      const built = (await (await app.request(`/api/v1/dev/scenes/${id}`, { method: "POST" })).json()) as { player: { id: string }; entry: { runId: string } };
      return (await (await app.request(`/api/v1/runs/${built.entry.runId}/forge`, { headers: { "X-Arena-Player": built.player.id } })).json()) as Forge;
    };
    const now = await forgeOf("fusion-now");
    expect(now.pairs).toEqual([expect.objectContaining({ first: 0, second: 1, state: "discovered" })]);
    // Built after fusion-now, its pair still has no design (each scene its own units).
    const none = await forgeOf("fusion-none");
    expect(none.pairs.map((p) => p.state)).toEqual(["unforged"]);
    expect(none.offers).toEqual([]);
    const shop = await forgeOf("fusion-shop");
    expect(shop.pairs).toEqual([]);
    expect(shop.offers).toEqual([expect.objectContaining({ slot: 0, partner: 0, state: "ready", awakens: true }), expect.objectContaining({ slot: 1, partner: 0, state: "discovered", awakens: false })]);
  }, 240_000);
});
