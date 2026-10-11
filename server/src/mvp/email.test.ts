// #929: email login, against the fake mailer (no network).
import { mkdtempSync, rmSync, writeFileSync } from "node:fs";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterAll, afterEach, describe, expect, it, vi } from "vitest";
import type { EmailStatus, EmailVerify, HomeView, PlayerSession } from "../../../src/mvp/contract.js";
import { createMvpApp } from "./app.js";
import { mvpContent } from "./content.js";
import {
  codeHash,
  EMAIL_CODE_MS,
  EMAIL_RESEND_MS,
  EMAIL_STARTS_PER_ADDRESS_HOUR,
  EMAIL_TRIES,
  emailKey,
  FakeMailer,
  isEmail,
  loginMail,
  mailFromEnv,
  SmtpMailer,
  smtpConfig,
} from "./email.js";
import { createInvite, openJoin, redeemInvite } from "./invites.js";
import { mvpRuntime, type MvpDeps } from "./runtime.js";
import { SqliteMvpStore } from "./sqlite-store.js";
import { MemoryMvpStore, type MvpStore } from "./store.js";
import { answerStart, FakeTelegram, REPLY, startTelegramLogin } from "./telegram.js";

const tempDirs: string[] = [];
afterAll(() => {
  for (const d of tempDirs.splice(0)) rmSync(d, { recursive: true, force: true });
});
afterEach(() => vi.restoreAllMocks());

function world(extra: Partial<MvpDeps> = {}) {
  const store = extra.store ?? new MemoryMvpStore();
  const mail = new FakeMailer(false);
  let clock = new Date("2026-10-11T12:00:00Z");
  const rt = mvpRuntime({ content: mvpContent(), store, dev: true, invites: true, open: true, mail, now: () => clock, ...extra });
  const app = createMvpApp(rt);
  let ip = 1;
  const call = async <T>(method: string, path: string, headers: Record<string, string> = {}, body?: unknown) => {
    const res = await app.request(`/api/v1${path}`, { method, headers: { "content-type": "application/json", "x-forwarded-for": `10.0.0.${ip}`, ...headers }, ...(body !== undefined ? { body: JSON.stringify(body) } : {}) });
    return { status: res.status, json: (await res.json()) as T & { error?: string } };
  };
  const start = (email: string, headers: Record<string, string> = {}, extraBody: object = {}) => call<{ ok: true; expiresAt: string }>("POST", "/auth/email/start", headers, { email, lang: "en", ...extraBody });
  const verify = (body: { email: string; code: string; name?: string; join?: string }, headers: Record<string, string> = {}) => call<EmailVerify>("POST", "/auth/email/verify", headers, body);
  const code = (email: string) => mail.lastCode(email)!;
  const tick = (ms: number) => (clock = new Date(clock.getTime() + ms));
  /** Requests from another address from now on. */
  const moveIp = () => ip++;
  return { rt, store, mail, call, start, verify, code, tick, moveIp, now: () => clock };
}
const tok = (s: { token: string }) => ({ "X-Arena-Token": s.token });
function invited(store: MvpStore, name: string, now: Date, admin = false): PlayerSession {
  const inv = createInvite(store, { name, admin, now });
  return redeemInvite(store, inv.code, now)!;
}

describe("email addresses and mail (#929)", () => {
  it("compares emails trimmed and lower-cased, and checks their form", () => {
    expect(emailKey("  Maks@Example.COM ")).toBe("maks@example.com");
    for (const ok of ["a@b.co", "maks.chugunov+arena@gmail.com", "имя@почта.рф"]) expect(isEmail(ok), ok).toBe(true);
    for (const bad of ["", "maks", "maks@", "@x.com", "a@b", "a b@c.com", "a@b.c", "<a@b.com>", "a@@b.com", `${"x".repeat(250)}@b.com`]) expect(isEmail(bad), bad).toBe(false);
  });

  it("writes the mail in the player's language, the code in the subject", () => {
    const en = loginMail("a@b.co", "012345", "en");
    expect(en.subject).toBe("Arena code: 012345");
    expect(en.text).toContain("012345");
    expect(en.html).toContain("012345");
    const ru = loginMail("a@b.co", "012345", "ru");
    expect(ru.subject).toBe("Код для Arena: 012345");
    expect(ru.text).toMatch(/10 минут/);
  });

  it("reads SMTP settings from ARENA_MAIL_ENV, fake only on dev, off when missing", () => {
    expect(mailFromEnv({}, false)).toBeUndefined();
    expect(mailFromEnv({ ARENA_MAIL_FAKE: "1" }, true)).toBeInstanceOf(FakeMailer);
    expect(() => mailFromEnv({ ARENA_MAIL_FAKE: "1" }, false)).toThrow(/MVP_DEV/);
    const dir = mkdtempSync(join(tmpdir(), "arena-mail-"));
    tempDirs.push(dir);
    const file = join(dir, "mail.env");
    writeFileSync(file, "SMTP_HOST=smtp.example.com\nSMTP_PORT=465\nSMTP_USER=arena@example.com\nSMTP_PASS='s3cret pass'\nSMTP_FROM=Arena <arena@example.com>\n");
    const errors = vi.spyOn(console, "error").mockImplementation(() => {});
    expect(mailFromEnv({ ARENA_MAIL_ENV: file }, false)).toBeInstanceOf(SmtpMailer);
    expect(mailFromEnv({ ARENA_MAIL_ENV: join(dir, "nope.env") }, false)).toBeUndefined();
    writeFileSync(file, "SMTP_HOST=smtp.example.com\nSMTP_PASS=s3cret\n");
    expect(mailFromEnv({ ARENA_MAIL_ENV: file }, false)).toBeUndefined();
    expect(errors.mock.calls.flat().join(" ")).not.toContain("s3cret");
    expect(errors.mock.calls.flat().join(" ")).toContain("SMTP_USER, SMTP_FROM");
    expect(smtpConfig({ SMTP_HOST: "h", SMTP_USER: "u", SMTP_PASS: "p", SMTP_FROM: "f" })).toMatchObject({ port: 465 });
    expect(smtpConfig({ SMTP_HOST: "h", SMTP_USER: "u", SMTP_PASS: "p", SMTP_FROM: "f", SMTP_PORT: "x" })).toBe("has a bad SMTP_PORT");
  });

  it("an SMTP failure says neither the account nor the password", async () => {
    const m = new SmtpMailer({ host: "127.0.0.1", port: 1, user: "arena@example.com", pass: "s3cret", from: "Arena <arena@example.com>" });
    const err = await m.send(loginMail("a@b.co", "123456", "en")).catch((e: Error) => e);
    expect(err).toBeInstanceOf(Error);
    expect(String((err as Error).message)).not.toMatch(/s3cret|arena@example/);
  });
});

describe("email login routes (#929)", () => {
  it("signs up with name and email, then signs in on another device by email alone", async () => {
    const w = world();
    expect((await w.start("New@Example.com", {}, { name: "Nova" })).status).toBe(200);
    expect(w.mail.sent).toHaveLength(1);
    expect(w.mail.sent[0]!.to).toBe("new@example.com");
    // The code is kept only as its hash.
    const stored = w.store.emailCode("new@example.com")!;
    expect(stored.codeHash).toBe(codeHash("new@example.com", w.code("new@example.com")));
    expect(JSON.stringify(stored)).not.toContain(`"${w.code("new@example.com")}"`);
    const up = await w.verify({ email: " new@EXAMPLE.com", code: w.code("new@example.com"), name: "Nova" });
    expect(up.status).toBe(200);
    expect(up.json).toMatchObject({ status: "done", player: { name: "Nova", bot: false } });
    const nova = up.json as PlayerSession & { status: "done" };
    const home = await w.call<HomeView>("GET", "/home", tok(nova));
    expect(home.json.email).toEqual({ enabled: true, required: true, address: "new@example.com" });
    expect((await w.call("POST", "/runs", tok(nova))).status).toBe(200);
    // The code is single use.
    expect((await w.verify({ email: "new@example.com", code: w.code("new@example.com") })).status).toBe(404);

    // Another device: email alone, any case.
    w.tick(EMAIL_RESEND_MS);
    expect((await w.start("NEW@example.com")).status).toBe(200);
    const inn = await w.verify({ email: "new@example.com", code: w.code("new@example.com") });
    expect(inn.json).toMatchObject({ status: "done", player: nova.player });
    expect((inn.json as PlayerSession).token).not.toBe(nova.token);
    // One email, one player: signing up again with it logs in instead.
    w.tick(EMAIL_RESEND_MS);
    await w.start("new@example.com", {}, { name: "Other" });
    expect((await w.verify({ email: "new@example.com", code: w.code("new@example.com"), name: "Other" })).json).toMatchObject({ player: nova.player });
  });

  it("an unknown email without a name asks for one, and the code still works with it", async () => {
    const w = world();
    await w.start("fresh@example.com");
    const code = w.code("fresh@example.com");
    expect((await w.verify({ email: "fresh@example.com", code })).json).toEqual({ status: "needsName" });
    // A taken name keeps the code too.
    invited(w.store, "Eva", w.now());
    const taken = await w.verify({ email: "fresh@example.com", code, name: "eva" });
    expect(taken.status).toBe(409);
    expect(taken.json.error).toMatch(/taken/);
    expect((await w.verify({ email: "fresh@example.com", code, name: "bot-x" })).status).toBe(409);
    expect((await w.verify({ email: "fresh@example.com", code, name: "<b>" })).status).toBe(400);
    expect((await w.verify({ email: "fresh@example.com", code, name: "Fresh" })).json).toMatchObject({ status: "done", player: { name: "Fresh" } });
  });

  it("start never says whether an email has a player", async () => {
    const w = world();
    const eva = invited(w.store, "Eva", w.now());
    w.store.linkEmail({ emailKey: "eva@example.com", playerId: eva.player.id, linkedAt: "t" });
    const known = await w.start("eva@example.com");
    const unknown = await w.start("nobody@example.com");
    expect(known.status).toBe(200);
    expect(unknown.status).toBe(200);
    expect(Object.keys(known.json).sort()).toEqual(Object.keys(unknown.json).sort());
    expect((await w.start("not an email")).status).toBe(400);
    expect((await w.start("a@b.co", {}, { name: "<script>" })).status).toBe(400);
  });

  it("a code takes 5 tries, lives 10 minutes, and one comes a minute per email", async () => {
    const w = world();
    await w.start("t@example.com");
    const code = w.code("t@example.com");
    const wrong = code === "000000" ? "111111" : "000000";
    for (let i = 1; i < EMAIL_TRIES; i++) {
      const r = await w.verify({ email: "t@example.com", code: wrong, name: "Tess" });
      expect(r.status).toBe(400);
      expect(r.json.error).toContain(`${EMAIL_TRIES - i} tr`);
    }
    expect((await w.verify({ email: "t@example.com", code: "12ab56", name: "Tess" })).json.error).toMatch(/no tries left/);
    // The right code is gone with the last try.
    expect((await w.verify({ email: "t@example.com", code, name: "Tess" })).status).toBe(404);

    // Resend: once a minute per email.
    expect((await w.start("t@example.com")).status).toBe(200);
    expect((await w.start("T@example.com")).status).toBe(429);
    w.tick(EMAIL_RESEND_MS - 1);
    expect((await w.start("t@example.com")).status).toBe(429);
    w.tick(1);
    expect((await w.start("t@example.com")).status).toBe(200);
    // A new code replaces the old one, and expires.
    const latest = w.code("t@example.com");
    w.tick(EMAIL_CODE_MS);
    expect((await w.verify({ email: "t@example.com", code: latest, name: "Tess" })).status).toBe(404);
    expect(w.store.emailCode("t@example.com")).toBeUndefined();
  });

  it("limits codes per address and for the whole server, an hour", async () => {
    const w = world();
    for (let i = 0; i < EMAIL_STARTS_PER_ADDRESS_HOUR; i++) expect((await w.start(`p${i}@example.com`)).status).toBe(200);
    expect((await w.start("one-more@example.com")).status).toBe(429);
    w.moveIp();
    expect((await w.start("one-more@example.com")).status).toBe(200);
    w.tick(3_600_000);
    w.moveIp();
    for (let i = 0; i < 200; i++) {
      if (i % EMAIL_STARTS_PER_ADDRESS_HOUR === 0) w.moveIp();
      expect((await w.start(`s${i}@example.com`)).status, `start ${i}`).toBe(200);
    }
    w.moveIp();
    expect((await w.start("late@example.com")).status).toBe(429);
  });

  it("a mail that fails to send answers 502 and lets the player ask again at once", async () => {
    const w = world();
    vi.spyOn(console, "error").mockImplementation(() => {});
    w.mail.failNext = true;
    expect((await w.start("x@example.com")).status).toBe(502);
    expect((await w.start("x@example.com")).status).toBe(200);
  });

  it("an invite-only server signs up only with the join code; open to all needs none", async () => {
    const w = world({ open: false });
    await w.start("j@example.com");
    const code = w.code("j@example.com");
    const r = await w.verify({ email: "j@example.com", code, name: "Jo" });
    expect(r.status).toBe(403);
    expect((await w.verify({ email: "j@example.com", code, name: "Jo", join: "wrong" })).status).toBe(403);
    const join = openJoin(w.store);
    expect((await w.verify({ email: "j@example.com", code, name: "Jo", join })).json).toMatchObject({ status: "done", player: { name: "Jo" } });
    // The name-only join is refused while email login is required.
    expect((await w.call("POST", "/join", {}, { code: join, name: "Nameonly" })).status).toBe(403);
  });

  it("a player without an email adds one with a code before playing", async () => {
    const w = world();
    const old = invited(w.store, "Oldie", w.now());
    const home = await w.call<HomeView>("GET", "/home", tok(old));
    expect(home.json.email).toEqual({ enabled: true, required: true, address: null });
    expect((await w.call("POST", "/runs", tok(old))).status).toBe(403);
    // A code asked for without the session can't attach.
    await w.start("old@example.com");
    expect((await w.call("POST", "/auth/email/attach", tok(old), { email: "old@example.com", code: w.code("old@example.com") })).status).toBe(404);
    w.tick(EMAIL_RESEND_MS);
    await w.start("old@example.com", tok(old));
    const wrong = await w.call("POST", "/auth/email/attach", tok(old), { email: "old@example.com", code: "999999" === w.code("old@example.com") ? "000000" : "999999" });
    expect(wrong.status).toBe(400);
    const ok = await w.call<EmailStatus>("POST", "/auth/email/attach", tok(old), { email: "Old@Example.com", code: w.code("old@example.com") });
    expect(ok.json).toEqual({ enabled: true, required: true, address: "old@example.com" });
    expect((await w.call("POST", "/runs", tok(old))).status).toBe(200);
    // Signing in with it now gives the same player.
    w.tick(EMAIL_RESEND_MS);
    await w.start("old@example.com");
    expect((await w.verify({ email: "old@example.com", code: w.code("old@example.com") })).json).toMatchObject({ player: old.player });
    // A second email for the same player is refused.
    await w.start("second@example.com", tok(old));
    expect((await w.call("POST", "/auth/email/attach", tok(old), { email: "second@example.com", code: w.code("second@example.com") })).status).toBe(409);
  });

  it("an email that belongs to another player can't be attached", async () => {
    const w = world();
    const a = invited(w.store, "Ann", w.now());
    const b = invited(w.store, "Ben", w.now());
    w.store.linkEmail({ emailKey: "ann@example.com", playerId: a.player.id, linkedAt: "t" });
    await w.start("ann@example.com", tok(b));
    const r = await w.call("POST", "/auth/email/attach", tok(b), { email: "ann@example.com", code: w.code("ann@example.com") });
    expect(r.status).toBe(409);
    expect(w.store.emailLink({ playerId: b.player.id })).toBeUndefined();
  });

  it("admins still open their invite link, and add an email like anyone", async () => {
    const w = world();
    const admin = invited(w.store, "Maks", w.now(), true);
    expect((await w.call<HomeView>("GET", "/home", tok(admin))).json.dev).toBe(true);
    await w.start("maks@example.com", tok(admin));
    expect((await w.call("POST", "/auth/email/attach", tok(admin), { email: "maks@example.com", code: w.code("maks@example.com") })).status).toBe(200);
    w.tick(EMAIL_RESEND_MS);
    await w.start("maks@example.com");
    const s = (await w.verify({ email: "maks@example.com", code: w.code("maks@example.com") })).json as PlayerSession;
    expect((await w.call<HomeView>("GET", "/home", tok(s))).json.dev).toBe(true);
  });

  it("without a mailer email login is off and nothing is required", async () => {
    const w = world();
    const rt = mvpRuntime({ content: mvpContent(), dev: true, invites: true, open: true });
    const app = createMvpApp(rt);
    const res = await app.request("/api/v1/auth/email/start", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ email: "a@b.co" }) });
    expect(res.status).toBe(404);
    const eva = invited(rt.store, "Eva", new Date());
    const home = (await (await app.request("/api/v1/home", { headers: tok(eva) })).json()) as HomeView;
    expect(home.email).toEqual({ enabled: false, required: false, address: null });
    expect((await app.request("/api/v1/runs", { method: "POST", headers: tok(eva) })).status).toBe(200);
    const health = (await (await app.request("/api/v1/health")).json()) as { mail: unknown };
    expect(health.mail).toBe(false);
    expect(((await w.call<{ mail: unknown }>("GET", "/health")).json).mail).toBe("fake");
  });

  it("an open dev server (no invites) keeps POST /players and offers email too", async () => {
    const w = world({ invites: false, open: false });
    const p = await w.call<{ id: string }>("POST", "/players", {}, { name: "Dev" });
    expect(p.status).toBe(200);
    expect((await w.call("POST", "/runs", { "X-Arena-Player": p.json.id })).status).toBe(200);
    await w.start("dev@example.com");
    expect((await w.verify({ email: "dev@example.com", code: w.code("dev@example.com"), name: "Devmail" })).json).toMatchObject({ status: "done" });
  });

  it("the fake mailer's inbox reads the last code, on a dev server only", async () => {
    const w = world();
    await w.start("in@example.com");
    const r = await w.call<{ code: string }>("POST", "/dev/mail/last", {}, { email: "IN@example.com" });
    expect(r.json.code).toBe(w.code("in@example.com"));
    expect((await w.call("POST", "/dev/mail/last", {}, { email: "none@example.com" })).status).toBe(404);
  });

  it("works on the SQLite store too", async () => {
    const w = world({ store: new SqliteMvpStore(":memory:") });
    await w.start("sql@example.com", {}, { name: "Sql" });
    expect((await w.verify({ email: "sql@example.com", code: w.code("sql@example.com"), name: "Sql" })).json).toMatchObject({ status: "done" });
    expect(w.store.emailLink({ emailKey: "sql@example.com" })).toBeDefined();
  });
});

describe("Telegram with email login required (#929)", () => {
  it("never makes a player; it still logs in a linked one", () => {
    const w = world({ telegram: new FakeTelegram() });
    const now = w.now();
    const s = startTelegramLogin(w.store, undefined, "arenaofideas_bot", now);
    const ANNA = { id: 501, is_bot: false, first_name: "Anna" };
    expect(answerStart(w.store, ANNA, s.code, now, { newPlayers: false }).text).toBe(REPLY.needsEmail);
    const eva = invited(w.store, "Eva", now);
    w.store.linkTelegram({ tgUserId: "501", playerId: eva.player.id, linkedAt: "t" });
    expect(answerStart(w.store, ANNA, s.code, now, { newPlayers: false }).buttons).toHaveLength(2);
  });
});
