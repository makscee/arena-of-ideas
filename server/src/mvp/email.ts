// #929: email login. Every account is tied to one email; signing in, on any
// device, is a 6-digit code mailed there. POST /auth/email/start mails the
// code; POST /auth/email/verify takes it back: a known email logs in, a new
// one with a name becomes a new player (the sign-up), a new one without a name
// answers "needsName". A signed-in player without an email (an account from
// before #929) attaches one with a code too (POST /auth/email/attach).
//
// Codes are 6 digits, kept only as their SHA-256 (with the email), live 10
// minutes, take 5 wrong tries, and one email gets at most one a minute. The
// answers to start never say whether an email has a player.
//
// The SMTP password is read from the file ARENA_MAIL_ENV names and lives only
// inside SmtpMailer: no message, error or log line carries it.
import { createHash, randomBytes, randomInt, timingSafeEqual } from "node:crypto";
import { readFileSync } from "node:fs";
import nodemailer from "nodemailer";
import type { EmailStatus, EmailVerify, PlayerRef, PlayerSession } from "../../../src/mvp/contract.js";
import { createInvite, CRUDE_NAME, hashToken, isCrudePlayerName, JoinRefused, NAME_RE, redeemInvite } from "./invites.js";
import type { MvpRuntime } from "./runtime.js";
import { nameKey, type EmailCode, type MvpStore } from "./store.js";
import { readEnvFile } from "./telegram.js";

/** How long a code works. */
export const EMAIL_CODE_MS = 10 * 60_000;
/** Wrong tries a code takes; the last one ends it. */
export const EMAIL_TRIES = 5;
/** One code a minute per email. */
export const EMAIL_RESEND_MS = 60_000;
/** Codes mailed an hour from one address, and from the whole server. */
export const EMAIL_STARTS_PER_ADDRESS_HOUR = 10;
export const EMAIL_STARTS_PER_HOUR = 200;

/** How emails compare and show: trimmed and lower-cased. */
export const emailKey = (raw: string) => raw.trim().toLowerCase();

const EMAIL_RE = /^[^\s@<>()[\]\\,;:"]+@[^\s@<>()[\]\\,;:"]+\.[^\s@<>()[\]\\,;:".]{2,}$/;
/** A plausible address: one @, a dot in the domain, no spaces or brackets. */
export const isEmail = (key: string) => key.length <= 254 && EMAIL_RE.test(key);

/** A refused email request: the status and a message the player reads. */
export class EmailRefused extends Error {
  constructor(readonly status: 400 | 403 | 404 | 409 | 429 | 502, message: string) {
    super(message);
  }
}

/** A code's hash, salted by its email so one hash says nothing of another's. */
export const codeHash = (key: string, code: string) => createHash("sha256").update(`${key}:${code}`).digest("hex");

/** Is the email login required on this server (invite-only, with a mailer)?
 * Then a new player comes only through an email, and a player without one
 * adds it before playing. */
export const emailRequired = (rt: Pick<MvpRuntime, "invites" | "mail">) => rt.invites && !!rt.mail;

export function emailStatus(rt: Pick<MvpRuntime, "invites" | "mail" | "store">, playerId: string): EmailStatus {
  return { enabled: !!rt.mail, required: emailRequired(rt), address: rt.store.emailLink({ playerId })?.emailKey ?? null };
}

// ---------- mail ----------

export interface Mail {
  to: string;
  subject: string;
  text: string;
  html: string;
}

/** Sends mail: SMTP (SmtpMailer) or the fake (FakeMailer). */
export interface Mailer {
  send(m: Mail): Promise<void>;
}

const escapeHtml = (s: string) => s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

/** The code's mail, in the player's language. */
export function loginMail(to: string, code: string, lang: "en" | "ru"): Mail {
  const t =
    lang === "ru"
      ? { subject: `Код для Arena: ${code}`, lead: "Твой код для Арены идей:", ttl: "Он действует 10 минут.", ignore: "Если ты его не запрашивал(а), просто не обращай внимания на это письмо." }
      : { subject: `Arena code: ${code}`, lead: "Your Arena of Ideas code:", ttl: "It works for 10 minutes.", ignore: "If you didn't ask for it, just ignore this mail." };
  const text = `${t.lead} ${code}\n\n${t.ttl} ${t.ignore}\n`;
  const html =
    `<div style="font-family:system-ui,sans-serif;font-size:16px;line-height:1.5">` +
    `<p>${escapeHtml(t.lead)}</p>` +
    `<p style="font-size:32px;font-weight:bold;letter-spacing:6px;margin:16px 0">${code}</p>` +
    `<p style="color:#666">${escapeHtml(t.ttl)} ${escapeHtml(t.ignore)}</p></div>`;
  return { to, subject: t.subject, text, html };
}

/** A send that failed, worded without the account or the password. */
export class MailError extends Error {}

export interface SmtpConfig {
  host: string;
  port: number;
  user: string;
  pass: string;
  from: string;
}

/** Mail over SMTP (port 465: TLS from the start). The password stays in this object. */
export class SmtpMailer implements Mailer {
  readonly #transport: nodemailer.Transporter;
  readonly #from: string;
  constructor(cfg: SmtpConfig) {
    this.#from = cfg.from;
    this.#transport = nodemailer.createTransport({
      host: cfg.host,
      port: cfg.port,
      secure: cfg.port === 465,
      auth: { user: cfg.user, pass: cfg.pass },
      connectionTimeout: 15_000,
      greetingTimeout: 15_000,
      socketTimeout: 30_000,
      logger: false,
      debug: false,
    });
  }
  async send(m: Mail): Promise<void> {
    try {
      await this.#transport.sendMail({ from: this.#from, ...m });
    } catch (err) {
      const e = err as { code?: unknown; responseCode?: unknown };
      throw new MailError(`send failed (${typeof e.code === "string" ? e.code : "error"}${typeof e.responseCode === "number" ? ` ${e.responseCode}` : ""})`);
    }
  }
}

/** The fake mailer (ARENA_MAIL_FAKE=1, dev only; tests): keeps every mail and
 * logs its subject (the code), so e2e reads it from POST /dev/mail/last. */
export class FakeMailer implements Mailer {
  readonly sent: Mail[] = [];
  constructor(private readonly log = true) {}
  /** Fails the next send (tests). */
  failNext = false;
  async send(m: Mail): Promise<void> {
    if (this.failNext) {
      this.failNext = false;
      throw new MailError("send failed (fake)");
    }
    this.sent.push(m);
    if (this.log) console.log(`mail (fake) to ${m.to}: ${m.subject}`);
  }
  /** The last mail to `to` (by its key), or undefined. */
  last(to: string): Mail | undefined {
    const key = emailKey(to);
    return [...this.sent].reverse().find((m) => m.to === key);
  }
  /** The code in the last mail to `to`. */
  lastCode(to: string): string | undefined {
    return this.last(to)?.subject.match(/\d{6}/)?.[0];
  }
}

/** The mailer main.ts runs: the fake (ARENA_MAIL_FAKE=1, dev only), SMTP
 * (ARENA_MAIL_ENV names a file with SMTP_HOST, SMTP_PORT, SMTP_USER,
 * SMTP_PASS, SMTP_FROM), or none (no email login, also when the file is
 * missing or incomplete). */
export function mailFromEnv(env: NodeJS.ProcessEnv, dev: boolean): Mailer | undefined {
  if (env.ARENA_MAIL_FAKE === "1") {
    if (!dev) throw new Error("ARENA_MAIL_FAKE=1 needs MVP_DEV=1");
    return new FakeMailer();
  }
  const file = env.ARENA_MAIL_ENV;
  if (!file) return undefined;
  let text: string;
  try {
    text = readFileSync(file, "utf8");
  } catch {
    console.error(`mail: can't read ${file}: email login is off`);
    return undefined;
  }
  const cfg = smtpConfig(readEnvFile(text));
  if (typeof cfg === "string") {
    console.error(`mail: ${file} ${cfg}: email login is off`);
    return undefined;
  }
  return new SmtpMailer(cfg);
}

/** The SMTP settings from an env file, or what is missing. */
export function smtpConfig(vars: Record<string, string>): SmtpConfig | string {
  const missing = ["SMTP_HOST", "SMTP_USER", "SMTP_PASS", "SMTP_FROM"].filter((k) => !vars[k]);
  if (missing.length) return `has no ${missing.join(", ")}`;
  const port = vars.SMTP_PORT ? Number(vars.SMTP_PORT) : 465;
  if (!Number.isInteger(port) || port <= 0 || port > 65535) return "has a bad SMTP_PORT";
  return { host: vars.SMTP_HOST!, port, user: vars.SMTP_USER!, pass: vars.SMTP_PASS!, from: vars.SMTP_FROM! };
}

// ---------- codes ----------

/** Makes a code for `email` and mails it. `player`: the signed-in caller
 * without an email, who may attach it with this code. Never says whether the
 * email has a player. */
export async function startEmailLogin(store: MvpStore, mailer: Mailer, email: string, player: PlayerRef | undefined, lang: "en" | "ru", now: Date): Promise<{ expiresAt: string }> {
  const key = emailKey(email);
  if (!isEmail(key)) throw new EmailRefused(400, "That doesn't look like an email address.");
  const had = store.emailCode(key);
  if (had && now.getTime() - Date.parse(had.sentAt) < EMAIL_RESEND_MS) throw new EmailRefused(429, "A code was just sent: wait a minute before asking for another.");
  const code = String(randomInt(0, 1_000_000)).padStart(6, "0");
  const expiresAt = new Date(now.getTime() + EMAIL_CODE_MS).toISOString();
  const attach = player && !player.bot && !store.emailLink({ playerId: player.id }) ? player.id : null;
  store.putEmailCode({ emailKey: key, codeHash: codeHash(key, code), playerId: attach, sentAt: now.toISOString(), expiresAt, tries: 0 });
  try {
    await mailer.send(loginMail(key, code, lang));
  } catch (err) {
    store.dropEmailCode(key);
    console.error(`mail: ${err instanceof MailError ? err.message : "send failed"}`);
    throw new EmailRefused(502, "The mail couldn't be sent. Try again in a minute.");
  }
  return { expiresAt };
}

/** The live code of `key` if `code` is it; a wrong one counts a try (the
 * last try ends the code). `playerId`: only a code asked for by this player
 * (an attach). */
function checkCode(store: MvpStore, key: string, code: string, now: Date, playerId?: string): EmailCode {
  const c = store.emailCode(key);
  if (!c || (playerId !== undefined && c.playerId !== playerId)) throw new EmailRefused(404, "No code for this email: ask for a new one.");
  if (Date.parse(c.expiresAt) <= now.getTime() || c.tries >= EMAIL_TRIES) {
    store.dropEmailCode(key);
    throw new EmailRefused(404, "This code has expired: ask for a new one.");
  }
  const given = /^\d{6}$/.test(code.trim()) ? codeHash(key, code.trim()) : "";
  const ok = given.length === c.codeHash.length && timingSafeEqual(Buffer.from(given), Buffer.from(c.codeHash));
  if (ok) return c;
  const tries = c.tries + 1;
  if (tries >= EMAIL_TRIES) {
    store.dropEmailCode(key);
    throw new EmailRefused(400, "Wrong code, and no tries left: ask for a new one.");
  }
  store.putEmailCode({ ...c, tries });
  throw new EmailRefused(400, `Wrong code: ${EMAIL_TRIES - tries} ${EMAIL_TRIES - tries === 1 ? "try" : "tries"} left.`);
}

export interface VerifyOptions {
  /** May a new email make a new player? (an open server, or the join code given) */
  canSignUp: boolean;
  /** Counts a new player against the hour's limit; false refuses (429). */
  admit?: () => boolean;
}

/** Takes a code back: a session for the email's player; a new player named
 * `name` for an email without one; "needsName" without a name (the code
 * stays, for the next try with one). */
export function verifyEmailLogin(store: MvpStore, email: string, code: string, name: string | undefined, now: Date, opts: VerifyOptions): EmailVerify {
  const key = emailKey(email);
  if (!isEmail(key)) throw new EmailRefused(400, "That doesn't look like an email address.");
  checkCode(store, key, code, now);
  const link = store.emailLink({ emailKey: key });
  if (link) {
    // Dropping first makes it single use: of two verifies, one gets the session.
    if (!store.dropEmailCode(key)) throw new EmailRefused(404, "This code was used: ask for a new one.");
    const player = store.player(link.playerId);
    if (!player) throw new EmailRefused(404, "No player for this email.");
    return { status: "done", ...newSession(store, player, now) };
  }
  const n = name?.trim() ?? "";
  if (!n) return { status: "needsName" };
  if (!opts.canSignUp) throw new EmailRefused(403, "Arena is invite-only for now: open a join link to sign up.");
  if (!NAME_RE.test(n)) throw new EmailRefused(400, "name: 1–24 letters, digits, spaces, _ or -");
  if (isCrudePlayerName(n)) throw new EmailRefused(400, CRUDE_NAME);
  if (nameKey(n).startsWith("bot-") || store.playersNamed(n, { bots: true }).some((p) => p.bot)) throw new EmailRefused(409, `${n} is a bot's name: pick another`);
  if (store.playersNamed(n).length || store.invites().some((i) => nameKey(i.name) === nameKey(n))) throw new EmailRefused(409, `${n} is taken: pick another name`);
  if (opts.admit && !opts.admit()) throw new EmailRefused(429, "Too many new players this hour. Try again later.");
  if (!store.dropEmailCode(key)) throw new EmailRefused(404, "This code was used: ask for a new one.");
  // A new player, never admin, with their own invite as a join makes (R4-20).
  let session: PlayerSession | undefined;
  try {
    const invite = createInvite(store, { name: n, admin: false, now });
    store.linkEmail({ emailKey: key, playerId: invite.playerId, linkedAt: now.toISOString() });
    session = redeemInvite(store, invite.code, now);
  } catch (err) {
    if (err instanceof JoinRefused) throw new EmailRefused(err.status, err.message);
    throw err;
  }
  if (!session) throw new Error(`email sign-up: the new invite for ${n} didn't open`);
  return { status: "done", ...session };
}

/** Ties `email` to `player`, who has none yet, with a code they asked for. */
export function attachEmail(rt: Pick<MvpRuntime, "invites" | "mail" | "store">, player: PlayerRef, email: string, code: string, now: Date): EmailStatus {
  const { store } = rt;
  const key = emailKey(email);
  if (!isEmail(key)) throw new EmailRefused(400, "That doesn't look like an email address.");
  if (store.emailLink({ playerId: player.id })) throw new EmailRefused(409, "You have an email already.");
  checkCode(store, key, code, now, player.id);
  if (store.emailLink({ emailKey: key })) {
    store.dropEmailCode(key);
    throw new EmailRefused(409, "This email belongs to another player: sign in with it instead.");
  }
  if (!store.dropEmailCode(key)) throw new EmailRefused(404, "This code was used: ask for a new one.");
  store.linkEmail({ emailKey: key, playerId: player.id, linkedAt: now.toISOString() });
  return emailStatus(rt, player.id);
}

function newSession(store: MvpStore, player: PlayerRef, now: Date): PlayerSession {
  // The same kind of token an invite link gives (./invites.ts).
  const token = randomBytes(24).toString("base64url");
  store.addSession(player.id, hashToken(token), now.toISOString());
  return { player, token };
}
