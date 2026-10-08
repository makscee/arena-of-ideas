// M4-6 (mission #810): log in with Telegram, against a fake Telegram (no network).
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, afterEach, describe, expect, it, vi } from "vitest";
import type { HomeView, PlayerSession, TelegramPoll, TelegramStart, TelegramStatus } from "../../../src/mvp/contract.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import { createInvite, isAdmin, redeemInvite } from "./invites.js";
import { mvpRuntime, type MvpDeps } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";
import {
  FakeTelegram,
  freeName,
  readEnvFile,
  REPLY,
  TELEGRAM_CODE_MS,
  TELEGRAM_STARTS_PER_HOUR,
  telegramFromEnv,
  TelegramHttpApi,
  telegramJob,
  telegramName,
  type TelegramUser,
} from "./telegram.js";

const tempDirs: string[] = [];
afterAll(() => {
  for (const d of tempDirs.splice(0)) rmSync(d, { recursive: true, force: true });
});
const stops: (() => void)[] = [];
afterEach(() => {
  for (const s of stops.splice(0)) s();
  vi.restoreAllMocks();
});

const ANNA: TelegramUser = { id: 501, is_bot: false, first_name: "Anna", last_name: "K." };
const BORIS: TelegramUser = { id: 502, is_bot: false, first_name: "Борис" };

function world(extra: Partial<MvpDeps> = {}) {
  const store = extra.store ?? new MemoryMvpStore();
  const tg = new FakeTelegram();
  let clock = new Date("2026-10-08T12:00:00Z");
  const rt = mvpRuntime({ content: mvpContent(), store, dev: true, invites: true, telegram: tg, now: () => clock, ...extra });
  stops.push(telegramJob(rt));
  const app = createMvpApp(rt);
  const call = async <T>(method: string, path: string, headers: Record<string, string> = {}, body?: unknown) => {
    const res = await app.request(`/api/v1${path}`, { method, headers: { "content-type": "application/json", ...headers }, ...(body !== undefined ? { body: JSON.stringify(body) } : {}) });
    return { status: res.status, json: (await res.json()) as T & { error?: string } };
  };
  const start = (headers: Record<string, string> = {}) => call<TelegramStart>("POST", "/auth/telegram/start", headers);
  const poll = (code: string) => call<TelegramPoll>("POST", "/auth/telegram/poll", {}, { code });
  const tick = (ms: number) => (clock = new Date(clock.getTime() + ms));
  return { rt, store, tg, call, start, poll, tick, now: () => clock };
}
const tok = (s: { token: string }) => ({ "X-Arena-Token": s.token });
/** A player with an opened invite on this device. */
function invited(store: MvpStore, name: string, now: Date, admin = false): PlayerSession {
  const inv = createInvite(store, { name, admin, now });
  return redeemInvite(store, inv.code, now)!;
}

describe("Telegram login (M4-6)", () => {
  it("links a logged-in player, then logs a second device in as them", async () => {
    const w = world();
    const eva = invited(w.store, "Eva", w.now());
    // Settings → Link Telegram.
    const s = await w.start(tok(eva));
    expect(s.status).toBe(200);
    expect(s.json.url).toBe(`https://t.me/arenaofideas_bot?start=${s.json.code}`);
    expect((await w.poll(s.json.code)).json).toEqual({ status: "waiting" });
    expect(await w.tg.press(s.json.code, ANNA)).toBe(REPLY.done);
    const linked = await w.poll(s.json.code);
    expect(linked.json).toMatchObject({ status: "done", player: eva.player });
    expect((await w.call<TelegramStatus>("GET", "/auth/telegram", tok(eva))).json).toEqual({ enabled: true, linked: true });

    // A second device, logged out: Log in with Telegram.
    const s2 = await w.start();
    expect(await w.tg.press(s2.json.code, ANNA)).toBe(REPLY.done);
    const second = await w.poll(s2.json.code);
    expect(second.json).toMatchObject({ status: "done", player: eva.player });
    const t2 = (second.json as PlayerSession).token;
    expect(t2).not.toBe(eva.token);
    // Its token works like an invite's.
    const home = await w.call<HomeView>("GET", "/home", tok({ token: t2 }));
    expect(home.status).toBe(200);
    expect(home.json.rating?.player.id).toBe(eva.player.id);
  });

  it("makes a new player, named from Telegram, for an unlinked user who logs in", async () => {
    const w = world();
    invited(w.store, "Борис", w.now()); // the name is taken: the new one gets "Борис 2"
    const s = await w.start();
    expect(await w.tg.press(s.json.code, BORIS)).toBe(REPLY.done);
    const done = (await w.poll(s.json.code)).json as PlayerSession;
    expect(done.player).toMatchObject({ name: "Борис 2", bot: false });
    expect(w.store.telegramLink({ tgUserId: "502" })?.playerId).toBe(done.player.id);
    expect(isAdmin(w.store, done.player.id)).toBe(false);
    // The same Telegram user again: the same player, not another new one.
    const s2 = await w.start();
    await w.tg.press(s2.json.code, BORIS);
    expect(((await w.poll(s2.json.code)).json as PlayerSession).player.id).toBe(done.player.id);
  });

  it("a code expires after 10 minutes, for the bot and for the page", async () => {
    const w = world();
    const a = await w.start();
    w.tick(TELEGRAM_CODE_MS);
    expect(await w.tg.press(a.json.code, ANNA)).toBe(REPLY.expired);
    expect((await w.poll(a.json.code)).status).toBe(404);
    // Accepted in time, but polled too late: no session either.
    const b = await w.start();
    expect(await w.tg.press(b.json.code, ANNA)).toBe(REPLY.done);
    w.tick(TELEGRAM_CODE_MS);
    expect((await w.poll(b.json.code)).status).toBe(404);
  });

  it("a code works once", async () => {
    const w = world();
    const s = await w.start();
    await w.tg.press(s.json.code, ANNA);
    expect((await w.poll(s.json.code)).json.status).toBe("done");
    expect((await w.poll(s.json.code)).status).toBe(404);
    // The bot refuses it again (another user can't take it over), and an unknown code.
    expect(await w.tg.press(s.json.code, BORIS)).toBe(REPLY.expired);
    expect(await w.tg.press("not-a-code-at-all", BORIS)).toBe(REPLY.expired);
    expect(w.store.telegramLink({ tgUserId: "502" })).toBeUndefined();
  });

  it("one Telegram user ↔ one player; unlink in settings", async () => {
    const w = world();
    const eva = invited(w.store, "Eva", w.now());
    const max = invited(w.store, "Max", w.now());
    const a = await w.start(tok(eva));
    await w.tg.press(a.json.code, ANNA);
    await w.poll(a.json.code);
    // Eva can't start a second link while linked.
    expect((await w.start(tok(eva))).status).toBe(409);
    // Anna's Telegram, from Max's settings: logs in as Eva (her linked player), never links Max.
    const b = await w.start(tok(max));
    await w.tg.press(b.json.code, ANNA);
    expect(((await w.poll(b.json.code)).json as PlayerSession).player.id).toBe(eva.player.id);
    expect(w.store.telegramLink({ playerId: max.player.id })).toBeUndefined();
    // Unlink: Anna's next login is a new player; Eva can link again.
    expect((await w.call<TelegramStatus>("POST", "/auth/telegram/unlink", tok(eva))).json).toEqual({ enabled: true, linked: false });
    expect(w.store.telegramLink({ tgUserId: "501" })).toBeUndefined();
    const c = await w.start();
    await w.tg.press(c.json.code, ANNA);
    const anna = (await w.poll(c.json.code)).json as PlayerSession;
    expect(anna.player.id).not.toBe(eva.player.id);
    expect(anna.player.name).toBe("Anna K");
    expect((await w.start(tok(eva))).status).toBe(200);
    // Unlink needs a player.
    expect((await w.call("POST", "/auth/telegram/unlink")).status).toBe(401);
  });

  it("admins stay invite-only: no Telegram link or login reaches one", async () => {
    const w = world();
    const boss = invited(w.store, "Boss", w.now(), true);
    expect((await w.start(tok(boss))).status).toBe(403);
    // A linked player made admin later: the bot refuses to log a device in as them.
    const eva = invited(w.store, "Eva", w.now());
    const a = await w.start(tok(eva));
    await w.tg.press(a.json.code, ANNA);
    await w.poll(a.json.code);
    createInvite(w.store, { name: "Eva", playerId: eva.player.id, admin: true, now: w.now() });
    const b = await w.start();
    expect(await w.tg.press(b.json.code, ANNA)).toBe(REPLY.admin);
    expect((await w.poll(b.json.code)).json).toEqual({ status: "waiting" });
    // A new Telegram player is never an admin, and admins' invites are untouched.
    const c = await w.start();
    await w.tg.press(c.json.code, BORIS);
    const boris = (await w.poll(c.json.code)).json as PlayerSession;
    expect(isAdmin(w.store, boris.player.id)).toBe(false);
    expect(w.store.invites().filter((i) => i.admin).map((i) => i.name).sort()).toEqual(["Boss", "Eva"]);
    expect((await w.call("POST", "/dev/end-day", tok(boris))).status).toBe(404);
  });

  it("bots can't link: a Telegram bot account, nor a game bot", async () => {
    const w = world({ invites: false });
    const s = await w.start();
    expect(await w.tg.press(s.json.code, { id: 900, is_bot: true, first_name: "Spam" })).toBe(REPLY.bot);
    expect(w.store.telegramLink({ tgUserId: "900" })).toBeUndefined();
    w.store.addPlayer({ id: "bot-1", name: "bot-mira", bot: true });
    expect((await w.start({ "X-Arena-Player": "bot-1" })).status).toBe(403);
  });

  it("rate-limits starts per player and per address", async () => {
    const w = world();
    const eva = invited(w.store, "Eva", w.now());
    const ip = { "x-forwarded-for": "203.0.113.9" };
    for (let i = 0; i < TELEGRAM_STARTS_PER_HOUR; i++) expect((await w.start(ip)).status).toBe(200);
    expect((await w.start(ip)).status).toBe(429);
    expect((await w.start({ "x-forwarded-for": "203.0.113.10" })).status).toBe(200);
    for (let i = 0; i < TELEGRAM_STARTS_PER_HOUR; i++) expect((await w.start(tok(eva))).status).toBe(200);
    expect((await w.start(tok(eva))).status).toBe(429);
    w.tick(3_600_000);
    expect((await w.start(ip)).status).toBe(200);
  });

  it("is off without a Telegram, and the fake's dev button plays the bot", async () => {
    const off = mvpRuntime({ content: mvpContent(), invites: true });
    const app = createMvpApp(off);
    expect((await app.request("/api/v1/auth/telegram/start", { method: "POST" })).status).toBe(404);
    const w = world();
    const s = await w.start();
    // Logged out, on an invite-only dev server: the dev button still answers.
    const r = await w.call<{ reply: string }>("POST", "/dev/telegram/accept", {}, { code: s.json.code });
    expect(r.json.reply).toBe(REPLY.done);
    expect((await w.poll(s.json.code)).json.status).toBe("done");
    expect(w.tg.sent.at(-1)).toEqual({ chatId: 1001, text: REPLY.done });
    // A bare /start gets the how-to.
    expect(await w.tg.send("/start", ANNA)).toBe(REPLY.hello);
  });

  it("keeps links, codes and sessions in SQLite", async () => {
    const dir = mkdtempSync(join(tmpdir(), "arena-tg-"));
    tempDirs.push(dir);
    const store = new SqliteMvpStore(join(dir, "t.db"));
    const w = world({ store });
    const eva = invited(store, "Eva", w.now());
    const s = await w.start(tok(eva));
    await w.tg.press(s.json.code, ANNA);
    const done = (await w.poll(s.json.code)).json as PlayerSession;
    const again = new SqliteMvpStore(join(dir, "t.db"));
    expect(again.telegramLink({ tgUserId: "501" })?.playerId).toBe(eva.player.id);
    expect(() => again.linkTelegram({ tgUserId: "777", playerId: eva.player.id, linkedAt: "x" })).toThrow();
    expect(again.telegramCode("nope")).toBeUndefined();
    expect((await w.call("GET", "/home", tok(done))).status).toBe(200);
    // The code is kept only as its hash.
    const rows = store.db.prepare("SELECT code_hash, json FROM mvp_telegram_codes").all() as { code_hash: string; json: string }[];
    expect(JSON.stringify(rows)).not.toContain(s.json.code);
  });

  it("names: Telegram names fit NAME_RE, never a bot's, made unique", () => {
    expect(telegramName({ id: 1, is_bot: false, first_name: "🔥 Ivan", last_name: "Petrov!!" })).toBe("Ivan Petrov");
    expect(telegramName({ id: 1, is_bot: false, first_name: "🔥", username: "ivan_p" })).toBe("ivan_p");
    expect(telegramName({ id: 1, is_bot: false, first_name: "🔥" })).toBe("Player");
    expect(telegramName({ id: 1, is_bot: false, first_name: "bot-mira" })).toBe("Tg bot-mira");
    expect(telegramName({ id: 1, is_bot: false, first_name: "A very long name that goes on" })).toHaveLength(20);
    const store = new MemoryMvpStore();
    store.addPlayer({ id: "b", name: "Mira", bot: true });
    expect(freeName(store, "Mira")).toBe("Mira 2");
  });
});

describe("the bot token", () => {
  const TOKEN = "123456:SECRET-token-ABCdef_never-logged";

  it("is read from ARENA_TELEGRAM_ENV's file and never appears in a log line or error", async () => {
    const dir = mkdtempSync(join(tmpdir(), "arena-tg-env-"));
    tempDirs.push(dir);
    const file = join(dir, "telegram.env");
    writeFileSync(file, `# the bot\nexport TELEGRAM_BOT_TOKEN="${TOKEN}"\n`);
    expect(readEnvFile(`TELEGRAM_BOT_TOKEN='${TOKEN}'`).TELEGRAM_BOT_TOKEN).toBe(TOKEN);
    const api = telegramFromEnv({ ARENA_TELEGRAM_ENV: file }, false);
    expect(api).toBeInstanceOf(TelegramHttpApi);
    expect(api!.bot).toBe("arenaofideas_bot");
    expect(JSON.stringify(api)).not.toContain(TOKEN);
    expect(() => telegramFromEnv({ ARENA_TELEGRAM_FAKE: "1" }, false)).toThrow(/MVP_DEV/);
    writeFileSync(file, "OTHER=1\n");
    expect(() => telegramFromEnv({ ARENA_TELEGRAM_ENV: file }, false)).toThrow(/no TELEGRAM_BOT_TOKEN/);

    // The real API's failures (network down, 401, 409) through the bot loop: logged, without the token.
    const lines: string[] = [];
    for (const m of ["log", "error", "warn", "info", "debug"] as const)
      vi.spyOn(console, m).mockImplementation((...a: unknown[]) => void lines.push(a.map(String).join(" ")));
    const urls: string[] = [];
    let n = 0;
    const fakeFetch = (async (url: string | URL | Request) => {
      urls.push(String(url));
      n++;
      if (n === 1) throw new TypeError(`fetch failed: ${String(url)}`);
      return new Response(JSON.stringify({ ok: false, description: `Unauthorized ${TOKEN}` }), { status: 401 });
    }) as typeof fetch;
    vi.useFakeTimers({ toFake: ["setTimeout"] });
    try {
      const rt = mvpRuntime({ content: mvpContent(), telegram: new TelegramHttpApi(TOKEN, "arenaofideas_bot", fakeFetch) });
      const stop = telegramJob(rt);
      await vi.advanceTimersByTimeAsync(0);
      await vi.advanceTimersByTimeAsync(5_000);
      await vi.advanceTimersByTimeAsync(5_000);
      stop();
      await vi.advanceTimersByTimeAsync(5_000);
    } finally {
      vi.useRealTimers();
    }
    expect(urls.length).toBeGreaterThanOrEqual(2);
    expect(urls[0]).toContain(TOKEN); // it did use the token…
    expect(lines.length).toBeGreaterThanOrEqual(2);
    expect(lines.join("\n")).toContain("getUpdates: no answer");
    expect(lines.join("\n")).toContain("getUpdates: HTTP 401");
    for (const l of lines) expect(l).not.toContain(TOKEN.split(":")[1]); // …but never says it
  });
});
