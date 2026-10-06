// Slice 13: invite links, session tokens, and the dev tools on an invite-only server.
import { mkdtempSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import type { HomeView, PlayerRef, PlayerSession, RunView } from "../../../src/mvp/contract.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { createInvite, hashToken, InviteError, revokeInvite } from "./invites.js";
import { mvpRuntime, type MvpDeps } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore } from "./store.js";

function world(extra: Partial<MvpDeps> = {}) {
  const store = extra.store ?? new MemoryMvpStore();
  const rt = mvpRuntime({ content: mvpContent(), store, dev: true, invites: true, ...extra });
  const app = createMvpApp(rt);
  const call = async <T>(method: string, path: string, headers: Record<string, string> = {}, body?: unknown) => {
    const res = await app.request(`/api/v1${path}`, { method, headers: { "content-type": "application/json", ...headers }, ...(body !== undefined ? { body: JSON.stringify(body) } : {}) });
    return { status: res.status, json: (await res.json()) as T & { error?: string } };
  };
  return { rt, store, call, now: new Date("2026-10-06T18:00:00Z") };
}
const tok = (s: PlayerSession) => ({ "X-Arena-Token": s.token });

describe("invite links (slice 13)", () => {
  it("an invite link gives its player on every device, each with its own token", async () => {
    const { store, call, now } = world();
    const inv = createInvite(store, { name: "Eva", now });
    const a = await call<PlayerSession>("POST", `/invites/${inv.code}`);
    const b = await call<PlayerSession>("POST", `/invites/${inv.code}`);
    expect(a.status).toBe(200);
    expect(a.json.player).toEqual({ id: inv.playerId, name: "Eva", bot: false });
    expect(b.json.player.id).toBe(a.json.player.id);
    expect(b.json.token).not.toBe(a.json.token);
    expect(store.invite(inv.code)?.redeemedAt).not.toBeNull();
    // The token, not its hash, is what the store must never hold.
    expect(store.sessionPlayer(a.json.token)).toBeUndefined();
    expect(store.sessionPlayer(hashToken(a.json.token))).toBe(inv.playerId);
    const run = await call<RunView>("POST", "/runs", tok(a.json));
    expect(run.status).toBe(200);
    // The second device continues the same run.
    const home = await call<HomeView>("GET", "/home", tok(b.json));
    expect(home.json.activeRunId).toBe(run.json.runId);
  });

  it("refuses an unknown code, a bare player id, a made-up token and a new name", async () => {
    const { store, call, now } = world();
    const inv = createInvite(store, { name: "Eva", now });
    expect((await call("POST", "/invites/nope")).status).toBe(404);
    // Player ids are public (the day, fusions, battles): alone they are no login.
    expect((await call("POST", "/runs", { "X-Arena-Player": inv.playerId })).status).toBe(401);
    const bad = await call("POST", "/runs", { "X-Arena-Token": "made-up" });
    expect(bad.status).toBe(401);
    expect(bad.json.error).toMatch(/^unknown player/);
    const reg = await call<PlayerRef>("POST", "/players", {}, { name: "Mallory" });
    expect(reg.status).toBe(403);
    const health = await call<{ invites: boolean }>("GET", "/health");
    expect(health.json.invites).toBe(true);
  });

  it("shows the dev tools only to an admin invite", async () => {
    const { store, call, now } = world();
    const maks = await call<PlayerSession>("POST", `/invites/${createInvite(store, { name: "Maks", admin: true, now }).code}`);
    const eva = await call<PlayerSession>("POST", `/invites/${createInvite(store, { name: "Eva", now }).code}`);
    expect((await call<HomeView>("GET", "/home", tok(maks.json))).json.dev).toBe(true);
    expect((await call<HomeView>("GET", "/home", tok(eva.json))).json.dev).toBe(false);
    expect((await call<HomeView>("GET", "/home")).json.dev).toBe(false);
    expect((await call("POST", "/dev/end-day", tok(eva.json))).status).toBe(404);
    expect((await call("POST", "/dev/end-day")).status).toBe(404);
    expect((await call("POST", "/dev/end-day", tok(maks.json))).status).toBe(200);
  });

  it("keeps an open server as it was: names, X-Arena-Player, dev for all", async () => {
    const { call } = world({ invites: false });
    const p = await call<PlayerRef>("POST", "/players", {}, { name: "Maks" });
    expect(p.status).toBe(200);
    expect((await call("POST", "/runs", { "X-Arena-Player": p.json.id })).status).toBe(200);
    expect((await call<HomeView>("GET", "/home")).json.dev).toBe(true);
    expect((await call<{ invites: boolean }>("GET", "/health")).json.invites).toBe(false);
  });

  it("claims an existing player by id, keeps invited names unique, and refuses taken names", () => {
    const store = new MemoryMvpStore();
    const now = new Date();
    const old: PlayerRef = { id: "old-maks", name: "makscee", bot: false };
    store.addPlayer(old);
    store.addPlayer({ id: "b1", name: "bot-Eva", bot: true });
    expect(() => createInvite(store, { name: "MaksCee", now })).toThrow(/taken by old-maks/);
    const mine = createInvite(store, { name: "makscee", playerId: "old-maks", admin: true, now });
    expect(mine.playerId).toBe("old-maks");
    expect(store.player("old-maks")).toEqual(old);
    expect(createInvite(store, { name: "makscee", playerId: "old-maks", now }).code).toBe(mine.code);
    expect(() => createInvite(store, { name: "someone", playerId: "b1", now })).toThrow(InviteError);
    expect(() => createInvite(store, { name: "other", playerId: "old-maks-2", now })).toThrow(/no human player/);
    const eva = createInvite(store, { name: "Eva", now });
    expect(() => createInvite(store, { name: "eva", now })).toThrow(/taken/);
    expect(eva.code).toMatch(/^[\w-]{16}$/);
    expect(() => createInvite(store, { name: "bad/name", now })).toThrow(/name/);
  });

  it("revoke ends every device's session and rotates the code; the player and rating stay", async () => {
    for (const store of [new MemoryMvpStore(), new SqliteMvpStore(":memory:")]) {
      const { call, now } = world({ store });
      const inv = createInvite(store, { name: "Eva", now });
      const a = await call<PlayerSession>("POST", `/invites/${inv.code}`);
      const b = await call<PlayerSession>("POST", `/invites/${inv.code}`);
      expect((await call<HomeView>("GET", "/home", tok(a.json))).json.rating).not.toBeNull();
      const r = revokeInvite(store, "eva", now)!;
      expect(r.sessions).toBe(2);
      expect(r.invite.code).not.toBe(inv.code);
      expect(r.invite.playerId).toBe(inv.playerId);
      expect(r.invite.redeemedAt).toBeNull();
      for (const s of [a, b]) {
        expect((await call<HomeView>("GET", "/home", tok(s.json))).json.rating).toBeNull();
        expect((await call("POST", "/runs", tok(s.json))).status).toBe(401);
      }
      expect((await call("POST", `/invites/${inv.code}`)).status).toBe(404);
      const c = await call<PlayerSession>("POST", `/invites/${r.invite.code}`);
      expect(c.json.player.id).toBe(inv.playerId);
      expect(store.invites().map((i) => i.code)).toEqual([r.invite.code]);
      expect(revokeInvite(store, "nobody", now)).toBeUndefined();
    }
  });

  it("promotes and demotes an existing invite's admin, keeping its link", async () => {
    const { store, call, now } = world();
    const inv = createInvite(store, { name: "Eva", now });
    const s = await call<PlayerSession>("POST", `/invites/${inv.code}`);
    const up = createInvite(store, { name: "Eva", playerId: inv.playerId, admin: true, now });
    expect(up).toEqual({ ...inv, admin: true, redeemedAt: up.redeemedAt });
    expect((await call<HomeView>("GET", "/home", tok(s.json))).json.dev).toBe(true);
    expect(createInvite(store, { name: "Eva", playerId: inv.playerId, now }).admin).toBe(true);
    const down = createInvite(store, { name: "Eva", playerId: inv.playerId, admin: false, now });
    expect(down.code).toBe(inv.code);
    expect(down.admin).toBe(false);
    expect((await call<HomeView>("GET", "/home", tok(s.json))).json.dev).toBe(false);
    expect((await call("POST", "/dev/end-day", tok(s.json))).status).toBe(404);
  });

  it("never gives an invite a bot's name, and folds names with Unicode case (Cyrillic too)", () => {
    const store = new SqliteMvpStore(":memory:");
    const now = new Date();
    store.addPlayer({ id: "b1", name: "bot-Esk", bot: true });
    expect(() => createInvite(store, { name: "BOT-esk", now })).toThrow(/bot's name/);
    expect(createInvite(store, { name: "Esk", now }).name).toBe("Esk");
    expect(() => createInvite(store, { name: "bot-Someone", now })).toThrow(/bot's name/);
    store.addPlayer({ id: "p1", name: "Макс", bot: false });
    expect(store.playersNamed("МАКС").map((p) => p.id)).toEqual(["p1"]);
    expect(() => createInvite(store, { name: "МАКС", now })).toThrow(/taken by p1/);
    createInvite(store, { name: "Ёжик", now });
    expect(() => createInvite(store, { name: "ёЖИК", now })).toThrow(/taken/);
    store.close();
  });

  it("keeps players, runs and ratings when the migration runs on an existing SQLite world", async () => {
    const path = join(mkdtempSync(join(tmpdir(), "arena-588-")), "w.db");
    const before = new SqliteMvpStore(path);
    before.addPlayer({ id: "old-maks", name: "makscee", bot: false });
    before.putRating({ player: { id: "old-maks", name: "makscee", bot: false }, rating: 1234, runs: 7, slays: 1, daysAsChampion: 0, playoffWins: 0 });
    before.db.prepare("DELETE FROM mvp_migrations WHERE name = '13-invites.sql'").run();
    before.db.exec("DROP TABLE mvp_invites; DROP TABLE mvp_sessions");
    before.close();
    const store = new SqliteMvpStore(path);
    const { call, now } = world({ store });
    const inv = createInvite(store, { name: "makscee", playerId: "old-maks", admin: true, now });
    const s = await call<PlayerSession>("POST", `/invites/${inv.code}`);
    const home = await call<HomeView>("GET", "/home", tok(s.json));
    expect(home.json.rating?.rating).toBe(1234);
    expect(home.json.dev).toBe(true);
    store.close();
  });
});
