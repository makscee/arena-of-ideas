// Slice 13: invite links, session tokens, and the dev tools on an invite-only server.
import { spawn } from "node:child_process";
import { mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, describe, expect, it } from "vitest";
import type { BattleRecord, HomeView, JoinSession, PlayerRef, PlayerSession, RunView } from "../../../src/mvp/contract.js";
import { createMvpApp, JOINS_PER_HOUR } from "./app.js";
import { mvpContent } from "./content.js";
import { createInvite, hashToken, InviteError, isAdmin, openJoin, redeemInvite, revokeInvite } from "./invites.js";
import { mvpRuntime, type MvpDeps } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MAX_SESSIONS, MemoryMvpStore } from "./store.js";

// Temp worlds this file makes, removed when it ends.
const tempDirs: string[] = [];
afterAll(() => {
  for (const d of tempDirs.splice(0)) rmSync(d, { recursive: true, force: true });
});

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
/** Opens an invite link: the code goes in the body, never the URL. */
const open = (call: ReturnType<typeof world>["call"], code: string) => call<PlayerSession>("POST", "/invites/redeem", {}, { code });

describe("invite links (slice 13)", () => {
  it("an invite link gives its player on every device, each with its own token", async () => {
    const { store, call, now } = world();
    const inv = createInvite(store, { name: "Eva", now });
    const a = await open(call, inv.code);
    const b = await open(call, inv.code);
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
    expect((await open(call, "nope")).status).toBe(404);
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
    const maks = await open(call, createInvite(store, { name: "Maks", admin: true, now }).code);
    const eva = await open(call, createInvite(store, { name: "Eva", now }).code);
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
      const a = await open(call, inv.code);
      const b = await open(call, inv.code);
      expect((await call<HomeView>("GET", "/home", tok(a.json))).json.rating).not.toBeNull();
      const r = revokeInvite(store, "eva", now)!;
      expect(r.sessions).toBe(2);
      expect(r.invite.code).not.toBe(inv.code);
      expect(r.invite.playerId).toBe(inv.playerId);
      expect(r.invite.redeemedAt).toBeNull();
      for (const s of [a, b])
        for (const [m, path] of [["GET", "/home"], ["POST", "/runs"], ["GET", "/day"]] as const) {
          const res = await call(m, path, tok(s.json));
          expect([res.status, res.json.error], `${m} ${path}`).toEqual([401, "unknown player: open your invite link"]);
        }
      expect((await open(call, inv.code)).status).toBe(404);
      const c = await open(call, r.invite.code);
      expect(c.json.player.id).toBe(inv.playerId);
      expect(store.invites().map((i) => i.code)).toEqual([r.invite.code]);
      expect(revokeInvite(store, "nobody", now)).toBeUndefined();
    }
  });

  it("promotes and demotes an existing invite's admin, keeping its link", async () => {
    const { store, call, now } = world();
    const inv = createInvite(store, { name: "Eva", now });
    const s = await open(call, inv.code);
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

  it("tells whose a link is without opening it, caps bodies at 16 KB, and asks testers for their link on 401", async () => {
    const { store, call, now } = world();
    const inv = createInvite(store, { name: "Eva", now });
    expect((await call<{ player: PlayerRef }>("POST", "/invites/lookup", {}, { code: inv.code })).json.player.name).toBe("Eva");
    expect(store.invite(inv.code)?.redeemedAt).toBeNull();
    expect((await call("POST", "/invites/lookup", {}, { code: "nope" })).status).toBe(404);
    const s = await open(call, inv.code);
    const run = await call<RunView>("POST", "/runs", tok(s.json));
    expect((await call("POST", `/runs/${run.json.runId}/decisions`, tok(s.json), { kind: "reroll", pad: "x".repeat(20_000) })).status).toBe(413);
    const anon = await call("POST", `/runs/${run.json.runId}/decisions`, {}, { kind: "reroll" });
    expect([anon.status, anon.json.error]).toEqual([401, "unknown player: open your invite link"]);
    expect((await call("POST", "/runs")).json.error).toBe("unknown player: open your invite link");
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
    const dir = mkdtempSync(join(tmpdir(), "arena-588-"));
    tempDirs.push(dir);
    const path = join(dir, "w.db");
    const before = new SqliteMvpStore(path);
    before.addPlayer({ id: "old-maks", name: "makscee", bot: false });
    before.putRating({ player: { id: "old-maks", name: "makscee", bot: false }, rating: 1234, runs: 7, slays: 1, daysAsChampion: 0, playoffWins: 0 });
    before.db.prepare("DELETE FROM mvp_migrations WHERE name = '13-invites.sql'").run();
    before.db.exec("DROP TABLE mvp_invites; DROP TABLE mvp_sessions");
    before.close();
    const store = new SqliteMvpStore(path);
    const { call, now } = world({ store });
    const inv = createInvite(store, { name: "makscee", playerId: "old-maks", admin: true, now });
    const s = await open(call, inv.code);
    const home = await call<HomeView>("GET", "/home", tok(s.json));
    expect(home.json.rating?.rating).toBe(1234);
    expect(home.json.dev).toBe(true);
    store.close();
  });
  it("gives a bare X-Arena-Player nothing on any route of an invite-only server", async () => {
    const { rt, store, call, now } = world();
    const eva = await open(call, createInvite(store, { name: "Eva", admin: true, now }).code);
    const run = await call<RunView>("POST", "/runs", tok(eva.json));
    const id = run.json.runId;
    store.putBattle({ battleId: "b-slay", runId: id, player: eva.json.player } as unknown as BattleRecord);
    store.addSlay({ seq: rt.today().seq, player: eva.json.player, runId: id, battleId: "b-slay", line: [], contentVersion: rt.content.version, at: now.toISOString() });
    const as = { "X-Arena-Player": eva.json.player.id };
    const home = await call<HomeView>("GET", "/home", as);
    expect([home.json.rating, home.json.activeRunId, home.json.dev]).toEqual([null, null, false]);
    const routes: [string, string, unknown?][] = [
      ["POST", "/runs"],
      ["GET", `/runs/${id}`],
      ["POST", `/runs/${id}/decisions`, { kind: "reroll" }],
      ["POST", `/runs/${id}/preview`, { kind: "reroll" }],
      ["POST", `/runs/${id}/abandon`],
      ["GET", "/battles/b-slay"],
      ["POST", "/dev/end-day"],
    ];
    for (const [m, path, body] of routes) expect((await call(m, path, as, body)).status, `${m} ${path}`).toBeOneOf([401, 404]);
    // The same routes with the token: the player's own (the run isn't over).
    expect((await call("GET", `/runs/${id}`, tok(eva.json))).status).toBe(200);
    expect((await call("GET", "/battles/b-slay", tok(eva.json))).status).toBe(200);
  });

  it(`keeps a player's newest ${MAX_SESSIONS} devices: an older token ends when one more opens the link`, async () => {
    for (const store of [new MemoryMvpStore(), new SqliteMvpStore(":memory:")]) {
      const { call, now } = world({ store });
      const inv = createInvite(store, { name: "Eva", now });
      const other = await open(call, createInvite(store, { name: "Ann", now }).code);
      const tokens: string[] = [];
      for (let i = 0; i < MAX_SESSIONS + 2; i++) tokens.push((await open(call, inv.code)).json.token);
      const alive = tokens.map((t) => store.sessionPlayer(hashToken(t)) === inv.playerId);
      expect(alive).toEqual([false, false, ...Array(MAX_SESSIONS).fill(true)]);
      expect(store.sessionPlayer(hashToken(other.json.token))).toBeDefined();
    }
  });

  it("can't be raced by a redeem on another connection: the revoke ends every session the old link gave", async () => {
    const dir = mkdtempSync(join(tmpdir(), "arena-588-race-"));
    try {
      const path = join(dir, "w.db");
      const a = new SqliteMvpStore(path);
      const b = new SqliteMvpStore(path);
      const now = new Date();
      // In turn: a revoke between a stale read and the redeem leaves nothing.
      const inv = createInvite(a, { name: "Eva", now });
      expect(a.invite(inv.code)).toBeDefined();
      const r = revokeInvite(b, "Eva", now)!;
      expect(redeemInvite(a, inv.code, now)).toBeUndefined();
      expect(a.db.prepare("SELECT count(*) AS n FROM mvp_sessions").get()).toEqual({ n: 0 });
      // At once: another process opens the old link in a loop while this one revokes it.
      const code = r.invite.code;
      const child = spawn(process.execPath, ["--import", "tsx/esm", "--input-type=module", "-e", `
        import { SqliteMvpStore } from ${JSON.stringify(join(import.meta.dirname, "sqlite-store.ts"))};
        import { redeemInvite } from ${JSON.stringify(join(import.meta.dirname, "invites.ts"))};
        const s = new SqliteMvpStore(${JSON.stringify(path)});
        const tokens = [];
        let misses = 0;
        console.log("ready");
        while (misses < 50) { const r = redeemInvite(s, ${JSON.stringify(code)}, new Date()); r ? tokens.push(r.token) : misses++; }
        console.log(JSON.stringify(tokens));
        s.close();`], { stdio: ["ignore", "pipe", "inherit"] });
      let out = "";
      const done = new Promise<number | null>((res) => child.on("exit", res));
      await new Promise<void>((res) => child.stdout.on("data", (d: Buffer) => { out += d; if (out.includes("ready")) res(); }));
      // Let it get a few sessions in, then revoke mid-loop.
      while (Number((b.db.prepare("SELECT count(*) AS n FROM mvp_sessions").get() as { n: number }).n) < 3) await new Promise((res) => setTimeout(res, 5));
      revokeInvite(b, "Eva", new Date());
      expect(await done).toBe(0);
      child.stdout.removeAllListeners();
      const tokens = JSON.parse(out.trim().split("\n").at(-1)!) as string[];
      expect(tokens.length).toBeGreaterThanOrEqual(3);
      for (const t of tokens) expect(b.sessionPlayer(hashToken(t))).toBeUndefined();
      expect(b.db.prepare("SELECT count(*) AS n FROM mvp_sessions").get()).toEqual({ n: 0 });
      a.close();
      b.close();
    } finally {
      rmSync(dir, { recursive: true, force: true });
    }
  }, 30_000);
});

describe("the open join link (R4-20)", () => {
  type Joined = JoinSession;
  const join = (call: ReturnType<typeof world>["call"], code: string, name: string, headers: Record<string, string> = {}) =>
    call<Joined>("POST", "/join", headers, { code, name });

  it("a good code makes a new player with a session and their own invite link; never admin", async () => {
    for (const store of [new MemoryMvpStore(), new SqliteMvpStore(":memory:")]) {
      const { call } = world({ store });
      const code = openJoin(store);
      expect(openJoin(store)).toBe(code);
      expect((await call("POST", "/join/check", {}, { code })).json).toEqual({ ok: true });
      const a = await join(call, code, "  Friend ");
      expect(a.status).toBe(200);
      expect(a.json.player).toMatchObject({ name: "Friend", bot: false });
      const inv = store.invite(a.json.invite)!;
      expect(inv).toMatchObject({ name: "Friend", playerId: a.json.player.id, admin: false });
      expect(isAdmin(store, a.json.player.id)).toBe(false);
      const home = await call<HomeView>("GET", "/home", tok(a.json));
      expect(home.json.dev).toBe(false);
      expect(home.json.rating?.player.id).toBe(a.json.player.id);
      expect((await call("POST", "/dev/end-day", tok(a.json))).status).toBe(404);
      expect((await call<RunView>("POST", "/runs", tok(a.json))).status).toBe(200);
      // Their own link opens the same player on another device.
      const b = await open(call, a.json.invite);
      expect(b.json.player.id).toBe(a.json.player.id);
      // A second person, another name.
      const c = await join(call, code, "Other");
      expect(c.status).toBe(200);
      expect(c.json.player.id).not.toBe(a.json.player.id);
    }
  });

  it("refuses a wrong code, no code yet, and a rotated code; joined players keep their sessions", async () => {
    const { store, call } = world();
    expect((await join(call, "", "Friend")).status).toBe(404);
    expect((await join(call, "anything", "Friend")).status).toBe(404);
    expect((await call("POST", "/join/check", {}, { code: "anything" })).status).toBe(404);
    const old = openJoin(store);
    expect((await join(call, old + "x", "Friend")).json.error).toBe("no such join link");
    expect((await join(call, old.slice(0, -1), "Friend")).status).toBe(404);
    const a = await join(call, old, "Friend");
    const fresh = openJoin(store, true);
    expect(fresh).not.toBe(old);
    expect((await join(call, old, "Late")).status).toBe(404);
    expect((await call("POST", "/join/check", {}, { code: old })).status).toBe(404);
    expect((await call<HomeView>("GET", "/home", tok(a.json))).status).toBe(200);
    expect((await open(call, a.json.invite)).status).toBe(200);
    expect((await join(call, fresh, "Late")).status).toBe(200);
    // Nothing was made by the refused joins.
    expect(store.playersNamed("Late").length).toBe(1);
  });

  it("refuses taken, bot and bad names, with the same rules as invites", async () => {
    const { store, call, now } = world();
    const code = openJoin(store);
    store.addPlayer({ id: "b1", name: "Mira", bot: true });
    store.addPlayer({ id: "h1", name: "makscee", bot: false });
    createInvite(store, { name: "Eva", now });
    const cases: [string, number, RegExp][] = [
      ["MaksCee", 409, /taken/],
      ["eva", 409, /taken/],
      ["mira", 409, /bot's name/],
      ["bot-Ash", 409, /bot's name/],
      ["", 400, /name:/],
      ["bad/name", 400, /name:/],
      ["x".repeat(25), 400, /name:/],
    ];
    for (const [name, status, msg] of cases) {
      const r = await join(call, code, name);
      expect([name, r.status], name).toEqual([name, status]);
      expect(r.json.error).toMatch(msg);
    }
    expect((await call("POST", "/join", {}, { code, name: 7 })).status).toBe(400);
    const ok = await join(call, code, "Friend");
    expect((await join(call, code, "FRIEND")).status).toBe(409);
    expect(store.invites().map((i) => i.name).sort()).toEqual(["Eva", "Friend"]);
    // A stale token on the device doesn't block a join.
    expect((await join(call, code, "Second", { "X-Arena-Token": "stale" })).status).toBe(200);
    expect(ok.status).toBe(200);
  });

  it(`makes at most ${JOINS_PER_HOUR} players an hour, then 429 until the hour has passed`, async () => {
    let t = new Date("2026-10-07T18:00:00Z").getTime();
    const { store, call } = world({ now: () => new Date(t) });
    const code = openJoin(store);
    for (let i = 0; i < JOINS_PER_HOUR; i++) {
      expect((await join(call, code, `p${i}`)).status).toBe(200);
      t += 60_000;
    }
    const over = await join(call, code, "late");
    expect(over.status).toBe(429);
    expect(over.json.error).toMatch(/Too many new players/);
    expect(store.playersNamed("late")).toEqual([]);
    // Refused names and wrong codes don't use the budget; the first join's hour ends.
    t = new Date("2026-10-07T19:00:00Z").getTime();
    expect((await join(call, code, "late")).status).toBe(200);
    expect((await join(call, code, "later")).status).toBe(429);
  });
});
