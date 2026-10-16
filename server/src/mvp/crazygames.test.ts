// #1218: guests, CrazyGames logins (a test key signs the tokens), the
// per-address limit and the name and idea filter.
import { generateKeyPairSync, sign } from "node:crypto";
import { describe, expect, it } from "vitest";
import type { HomeView, PlayerSession, RunView } from "../../../src/mvp/contract.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { CrazyGamesRefused, freeName, verifyCgJwt, verifyCgToken, type CgLogin } from "./crazygames.js";
import { isCrudeText } from "./crude.js";
import { createInvite, InviteError, openJoin } from "./invites.js";
import { mvpRuntime, type MvpDeps } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore } from "./store.js";

const pair = () => generateKeyPairSync("rsa", { modulusLength: 2048, publicKeyEncoding: { type: "spki", format: "pem" }, privateKeyEncoding: { type: "pkcs8", format: "pem" } });
const KEY = pair();
const OTHER = pair();
const NOW = new Date("2026-10-16T12:00:00Z");
const sec = NOW.getTime() / 1000;

const b64 = (x: unknown) => Buffer.from(JSON.stringify(x)).toString("base64url");
/** A token as CrazyGames signs it (RS256), with these claims over the usual ones. */
function cgToken(claims: Record<string, unknown>, key = KEY.privateKey, alg = "RS256"): string {
  const head = b64({ alg, typ: "JWT" });
  const body = b64({ userId: "cg-1", gameId: "arena", username: "NightOwl", profilePictureUrl: "", iat: sec - 60, exp: sec + 3600, ...claims });
  return `${head}.${body}.${sign("RSA-SHA256", Buffer.from(`${head}.${body}`), key).toString("base64url")}`;
}

function world(extra: Partial<MvpDeps> = {}) {
  const store = extra.store ?? new MemoryMvpStore();
  let now = NOW;
  const rt = mvpRuntime({ content: mvpContent(), store, invites: true, now: () => now, cgKeys: async () => KEY.publicKey, ...extra });
  const app = createMvpApp(rt);
  const call = async <T>(method: string, path: string, headers: Record<string, string> = {}, body?: unknown) => {
    const res = await app.request(`/api/v1${path}`, { method, headers: { "content-type": "application/json", "x-forwarded-for": "10.0.0.1", ...headers }, ...(body !== undefined ? { body: JSON.stringify(body) } : {}) });
    return { status: res.status, json: (await res.json()) as T & { error?: string } };
  };
  const guest = (ip = "10.0.0.1") => call<PlayerSession>("POST", "/players/guest", { "x-forwarded-for": ip });
  const cg = (token: string, device?: PlayerSession, ip = "10.0.0.1") => call<CgLogin>("POST", "/players/crazygames", { "x-forwarded-for": ip, ...(device ? { "X-Arena-Token": device.token } : {}) }, { token });
  return { rt, store, call, guest, cg, later: (ms: number) => (now = new Date(now.getTime() + ms)) };
}
const tok = (s: PlayerSession) => ({ "X-Arena-Token": s.token });

describe("CrazyGames token check (#1218)", () => {
  it("reads userId and username from a valid token", () => {
    expect(verifyCgJwt(cgToken({}), KEY.publicKey, NOW)).toEqual({ userId: "cg-1", username: "NightOwl" });
  });
  it("refuses an expired token, a wrong signature, no userId, another algorithm and junk", () => {
    const refused = (token: string, why: RegExp) => expect(() => verifyCgJwt(token, KEY.publicKey, NOW)).toThrow(why);
    refused(cgToken({ exp: sec - 1 }), /expired/);
    refused(cgToken({ exp: undefined }), /expired/);
    refused(cgToken({}, OTHER.privateKey), /bad signature/);
    refused(cgToken({ userId: undefined }), /no userId/);
    refused(cgToken({ userId: "  " }), /no userId/);
    refused(cgToken({}, KEY.privateKey, "none"), /RS256/);
    refused(cgToken({}, KEY.privateKey, "HS256"), /RS256/);
    // A signed token whose claims were swapped afterwards
    const [h, , s] = cgToken({}).split(".");
    refused(`${h}.${b64({ userId: "cg-2", exp: sec + 60 })}.${s}`, /bad signature/);
    refused("not.a", /not a token/);
    try {
      verifyCgJwt(cgToken({ exp: sec - 1 }), KEY.publicKey, NOW);
    } catch (err) {
      expect(err).toBeInstanceOf(CrazyGamesRefused);
      expect((err as CrazyGamesRefused).status).toBe(401);
    }
  });
  it("asks for a fresh key once when the cached one doesn't verify (a rotated key)", async () => {
    const asked: boolean[] = [];
    const keys = async (fresh: boolean) => (asked.push(fresh), fresh ? KEY.publicKey : OTHER.publicKey);
    expect((await verifyCgToken(cgToken({}), keys, NOW)).userId).toBe("cg-1");
    expect(asked).toEqual([false, true]);
  });
});

describe("guests and CrazyGames players (#1218)", () => {
  it("a guest plays at once on an invite-only server, with a generated name", async () => {
    const { guest, call } = world();
    const g = await guest();
    expect(g.status).toBe(200);
    expect(g.json.player.name).toMatch(/^\p{L}+ \p{L}+ \d+$/u);
    expect(g.json.player.bot).toBe(false);
    expect((await call<RunView>("POST", "/runs", tok(g.json))).status).toBe(200);
    const home = await call<HomeView>("GET", "/home", tok(g.json));
    expect(home.json.activeRunId).not.toBeNull();
  });

  it("a guest who logs in keeps their run and gets the CrazyGames name", async () => {
    const { guest, cg, call, store } = world();
    const g = (await guest()).json;
    const run = await call<RunView>("POST", "/runs", tok(g));
    const login = await cg(cgToken({}), g);
    expect(login.status).toBe(200);
    expect(login.json.via).toBe("linked");
    expect(login.json.player).toEqual({ id: g.player.id, name: "NightOwl", bot: false });
    expect(store.crazyGamesLink({ cgUserId: "cg-1" })?.playerId).toBe(g.player.id);
    const home = await call<HomeView>("GET", "/home", tok(login.json));
    expect(home.json.activeRunId).toBe(run.json.runId);
    // The guest token keeps working too.
    expect((await call<HomeView>("GET", "/home", tok(g))).json.activeRunId).toBe(run.json.runId);
  });

  it("the same CrazyGames user on a second device is the same player", async () => {
    const { guest, cg, call } = world();
    const first = (await cg(cgToken({}), (await guest()).json)).json;
    const run = await call<RunView>("POST", "/runs", tok(first));
    // Device 2: its own guest first, then the same account.
    const other = (await guest("10.0.0.2")).json;
    const second = await cg(cgToken({ iat: sec }), other, "10.0.0.2");
    expect(second.json.via).toBe("found");
    expect(second.json.player.id).toBe(first.player.id);
    expect((await call<HomeView>("GET", "/home", tok(second.json))).json.activeRunId).toBe(run.json.runId);
    // With no device player at all, too.
    const third = await cg(cgToken({}), undefined, "10.0.0.3");
    expect(third.json.player.id).toBe(first.player.id);
  });

  it("finds by userId, never by username", async () => {
    const { cg } = world();
    const a = (await cg(cgToken({ userId: "u-a", username: "Twin" }))).json;
    const b = (await cg(cgToken({ userId: "u-b", username: "Twin" }))).json;
    expect(b.player.id).not.toBe(a.player.id);
    expect(a.player.name).toBe("Twin");
    expect(b.player.name).toMatch(/^Twin \d+$/);
  });

  it("a guest already linked to one account doesn't move to another", async () => {
    const { guest, cg } = world();
    const g = (await guest()).json;
    await cg(cgToken({ userId: "u-a" }), g);
    const b = await cg(cgToken({ userId: "u-b", username: "Other" }), g);
    expect(b.json.via).toBe("new");
    expect(b.json.player.id).not.toBe(g.player.id);
  });

  it("refuses a bad token with 401 and makes no player", async () => {
    const { cg, store } = world();
    expect((await cg(cgToken({}, OTHER.privateKey))).status).toBe(401);
    expect((await cg(cgToken({ exp: sec - 5 }))).status).toBe(401);
    expect(store.crazyGamesLink({ cgUserId: "cg-1" })).toBeUndefined();
  });

  it("answers 502 when CrazyGames' key can't be fetched", async () => {
    const { cg } = world({ cgKeys: async () => Promise.reject(new Error("offline")) });
    expect((await cg(cgToken({}))).status).toBe(502);
  });

  it("a stale token on this device doesn't stop a guest or a login", async () => {
    const { call, cg } = world();
    expect((await call("POST", "/players/guest", { "X-Arena-Token": "gone" })).status).toBe(200);
    expect((await call("POST", "/players/crazygames", { "X-Arena-Token": "gone" }, { token: cgToken({}) })).status).toBe(200);
    expect((await cg(cgToken({}))).json.via).toBe("found");
  });

  it("keeps the links in SQLite", async () => {
    const store = new SqliteMvpStore(":memory:");
    const { guest, cg } = world({ store });
    const g = (await guest()).json;
    await cg(cgToken({}), g);
    expect(store.crazyGamesLink({ playerId: g.player.id })?.cgUserId).toBe("cg-1");
    expect(() => store.linkCrazyGames({ cgUserId: "cg-1", playerId: "x", linkedAt: NOW.toISOString() })).toThrow();
  });
});

describe("new players per address (#1218)", () => {
  it("the 6th guest from one address in an hour is refused politely; others and later ones aren't", async () => {
    const { guest, later } = world();
    for (let i = 0; i < 5; i++) expect((await guest()).status).toBe(200);
    const sixth = await guest();
    expect(sixth.status).toBe(429);
    expect(sixth.json.error).toMatch(/Too many new players from here this hour/);
    expect((await guest("10.0.0.9")).status).toBe(200);
    later(3_600_001);
    expect((await guest()).status).toBe(200);
  });

  it("a new CrazyGames player counts; an existing one logging in doesn't", async () => {
    const { guest, cg } = world({ guestJoins: { perAddress: 2, perHour: 100 } });
    expect((await cg(cgToken({ userId: "a" }))).status).toBe(200);
    expect((await guest()).status).toBe(200);
    expect((await cg(cgToken({ userId: "b" }))).status).toBe(429);
    expect((await cg(cgToken({ userId: "a" }))).status).toBe(200);
  });

  it("has a server-wide ceiling", async () => {
    const { guest } = world({ guestJoins: { perAddress: 5, perHour: 3 } });
    for (const ip of ["1.1.1.1", "2.2.2.2", "3.3.3.3"]) expect((await guest(ip)).status).toBe(200);
    expect((await guest("4.4.4.4")).status).toBe(429);
  });

  it("leaves the join link's own limit apart", async () => {
    const { guest, call, store } = world({ open: true });
    openJoin(store);
    for (let i = 0; i < 5; i++) await guest();
    expect((await call("POST", "/join", {}, { code: "", name: "Rook" })).status).toBe(200);
  });
});

describe("names and ideas pass the crude filter (#1218)", () => {
  it("a crude CrazyGames username is replaced by a generated name", async () => {
    const { cg } = world();
    const r = await cg(cgToken({ username: "FuckFace" }));
    expect(r.json.player.name).not.toMatch(/fuck/i);
    expect(r.json.player.name).toMatch(/^\p{L}+ \p{L}+ \d+$/u);
  });
  it("a username is cut to fit and stripped of what a name can't hold", () => {
    const store = new MemoryMvpStore();
    expect(freeName(store, "Mr.Owl!!")).toBe("MrOwl");
    expect(freeName(store, "A".repeat(40))).toBe("A".repeat(19));
    expect(freeName(store, "bot-Ash")).toMatch(/^\p{L}+ \p{L}+ \d+$/u);
  });
  it("generated names skip taken ones", () => {
    const store = new MemoryMvpStore();
    store.addPlayer({ id: "p1", name: "Amber Badger 1", bot: false });
    let i = 0;
    // The first guess is taken; the next free one is used.
    expect(freeName(store, undefined, (n) => (i++ < 3 ? 0 : Math.min(1, n - 1)))).not.toBe("Amber Badger 1");
  });
  it("every join path refuses a crude name", async () => {
    const { call, store } = world({ open: true });
    openJoin(store);
    const join = await call("POST", "/join", {}, { code: "", name: "Big Shit" });
    expect(join.status).toBe(400);
    expect(join.json.error).toMatch(/isn't allowed/);
    expect(() => createInvite(store, { name: "Сука", now: NOW })).toThrow(InviteError);
    const openWorld = world({ invites: false });
    expect((await openWorld.call("POST", "/players", {}, { name: "Nazi Lord" })).status).toBe(400);
    expect((await openWorld.call("POST", "/players", {}, { name: "Night Lord" })).status).toBe(200);
  });
  it("idea text with a crude word is refused with 'try other words'", async () => {
    const { guest, call } = world();
    const g = (await guest()).json;
    const crude = await call("POST", "/ideas", tok(g), { text: "A wolf that shits on its enemies every turn" });
    expect(crude.status).toBe(400);
    expect(crude.json.error).toBe("Try other words.");
    expect(isCrudeText("Сука волк")).toBe(true);
    expect(isCrudeText("A classic assassin that strikes twice at night")).toBe(false);
    expect(isCrudeText("Цапля, которая лечит союзников")).toBe(false);
  });
});
