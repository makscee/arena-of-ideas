// M4-6 (mission #810): log in with Telegram, by a deep link
// (docs/mission4/README.md, Calls). The page asks for a one-time code
// (POST /auth/telegram/start) and opens t.me/<bot>?start=<code>; the bot gets
// `/start <code>` through one long-polling loop (telegramJob, in ./jobs.ts)
// and decides whose code it is; the page's next poll (POST /auth/telegram/poll)
// takes a session token, the same kind an invite link gives (./invites.ts).
// Codes live 10 minutes, work once and are kept only as their SHA-256. One
// Telegram user ↔ one player. A Telegram login never makes or reaches an
// admin: admins stay invite-only.
//
// The bot token is read from the file ARENA_TELEGRAM_ENV names and lives only
// inside TelegramHttpApi: no message, error or log line carries it.
import { randomBytes } from "node:crypto";
import { readFileSync } from "node:fs";
import type { PlayerRef, PlayerSession, TelegramStart } from "../../../src/mvp/contract.js";
import { createInvite, hashToken, isAdmin, NAME_RE } from "./invites.js";
import type { MvpJob, MvpRuntime } from "./runtime.js";
import { nameKey, type MvpStore } from "./store.js";

/** How long a login code works. */
export const TELEGRAM_CODE_MS = 10 * 60_000;
/** Login starts per player (or per address, logged out) an hour. */
export const TELEGRAM_STARTS_PER_HOUR = 20;
export const TELEGRAM_BOT = "arenaofideas_bot";

export const REPLY = {
  done: "Готово — вернитесь в игру / Done, go back to the game",
  expired: "Ссылка устарела или уже использована — начните заново в игре / This link has expired or was used: start again in the game",
  hello: "Откройте игру и нажмите «Войти через Telegram» / Open the game and tap Log in with Telegram",
  bot: "Боты не могут войти / Bots can't log in",
  admin: "Администраторы входят по своей ссылке-приглашению / Admins log in by their invite link",
  taken: "Этот игрок уже связан с другим Telegram / That player is linked to another Telegram account",
  failed: "Не получилось — попробуйте ещё раз / That didn't work: try again",
} as const;

/** A start the server refuses: 403 a bot or an admin, 409 already linked. */
export class TelegramRefused extends Error {
  constructor(readonly status: 403 | 409, message: string) {
    super(message);
  }
}

// The few Bot API shapes the login reads (core.telegram.org/bots/api).
export interface TelegramUser {
  id: number;
  is_bot: boolean;
  first_name: string;
  last_name?: string;
  username?: string;
}
export interface TelegramUpdate {
  update_id: number;
  message?: { chat: { id: number }; from?: TelegramUser; text?: string };
}

/** The bot's side of the Bot API: the real one (TelegramHttpApi) or a fake. */
export interface TelegramApi {
  /** The bot's @name, for the deep link. */
  readonly bot: string;
  /** Long-polls the updates from `offset` on; [] when none came. */
  getUpdates(offset: number, signal: AbortSignal): Promise<TelegramUpdate[]>;
  sendMessage(chatId: number, text: string): Promise<void>;
}

/** Makes a login code. `player` asks to link (settings: Link Telegram); none
 * logs in (the welcome screen). */
export function startTelegramLogin(store: MvpStore, player: PlayerRef | undefined, bot: string, now: Date): TelegramStart {
  if (player?.bot) throw new TelegramRefused(403, "bots can't link Telegram");
  if (player && isAdmin(store, player.id)) throw new TelegramRefused(403, "admins log in by their invite link");
  if (player && store.telegramLink({ playerId: player.id })) throw new TelegramRefused(409, "Telegram is linked already: unlink it first");
  const code = randomBytes(18).toString("base64url");
  const expiresAt = new Date(now.getTime() + TELEGRAM_CODE_MS).toISOString();
  store.putTelegramCode({ codeHash: hashToken(code), playerId: player?.id ?? null, createdAt: now.toISOString(), expiresAt, readyFor: null });
  return { code, url: `https://t.me/${bot}?start=${code}`, expiresAt };
}

/** The page's poll: "waiting" until the bot accepts the code, then a session
 * for the player it names, once; undefined for an unknown, expired or used code. */
export function pollTelegramLogin(store: MvpStore, code: string, now: Date): PlayerSession | "waiting" | undefined {
  if (!code) return undefined;
  const h = hashToken(code);
  const c = store.telegramCode(h);
  if (!c) return undefined;
  if (Date.parse(c.expiresAt) <= now.getTime()) {
    store.dropTelegramCode(h);
    return undefined;
  }
  if (!c.readyFor) return "waiting";
  // Dropping first makes it single use: of two polls, one gets the session.
  if (!store.dropTelegramCode(h)) return undefined;
  const player = store.player(c.readyFor);
  if (!player) return undefined;
  const token = randomBytes(24).toString("base64url");
  store.addSession(player.id, hashToken(token), now.toISOString());
  return { player, token };
}

/** The bot's answer to `/start <code>` from `from`. It links an unlinked
 * user to the player who asked, logs a linked user's device in as their
 * player, and makes a new player for an unlinked user who only logs in. */
export function answerStart(store: MvpStore, from: TelegramUser, code: string, now: Date): string {
  if (from.is_bot) return REPLY.bot;
  const h = hashToken(code);
  const c = store.telegramCode(h);
  if (!c || c.readyFor || Date.parse(c.expiresAt) <= now.getTime()) return REPLY.expired;
  const tgUserId = String(from.id);
  const ready = (playerId: string) => {
    store.putTelegramCode({ ...c, readyFor: playerId });
    return REPLY.done;
  };
  const linked = store.telegramLink({ tgUserId });
  if (linked) return isAdmin(store, linked.playerId) ? REPLY.admin : ready(linked.playerId);
  if (c.playerId) {
    if (isAdmin(store, c.playerId)) return REPLY.admin;
    if (store.telegramLink({ playerId: c.playerId })) return REPLY.taken;
    store.linkTelegram({ tgUserId, playerId: c.playerId, linkedAt: now.toISOString() });
    return ready(c.playerId);
  }
  // A new player, never admin, with their own invite like a join (R4-20).
  const invite = createInvite(store, { name: freeName(store, telegramName(from)), admin: false, now });
  store.linkTelegram({ tgUserId, playerId: invite.playerId, linkedAt: now.toISOString() });
  return ready(invite.playerId);
}

/** A player name from a Telegram name: NAME_RE's letters only, never a bot's. */
export function telegramName(u: TelegramUser): string {
  const raw = [u.first_name, u.last_name].filter(Boolean).join(" ") || u.username || "";
  let name = raw.normalize("NFKC").replace(/[^\p{L}\p{N}_\- ]/gu, "").replace(/\s+/g, " ").trim().slice(0, 20).trim();
  if (!name) name = "Player";
  if (nameKey(name).startsWith("bot-")) name = `Tg ${name}`.slice(0, 20).trim();
  return name;
}

/** `name`, or `name 2`, `name 3`…: the first no player or invite has. */
export function freeName(store: MvpStore, name: string): string {
  const taken = (n: string) =>
    !NAME_RE.test(n) || nameKey(n).startsWith("bot-") || store.playersNamed(n, { bots: true }).length > 0 || store.invites().some((i) => nameKey(i.name) === nameKey(n));
  if (!taken(name)) return name;
  for (let k = 2; k < 1000; k++) if (!taken(`${name} ${k}`)) return `${name} ${k}`;
  return `${name} ${randomBytes(3).toString("hex")}`;
}

const START = /^\/start(?:@\w+)?(?:\s+(\S+))?\s*$/;

/** Answers one update: `/start <code>` and a bare `/start`; the rest is ignored. */
export async function handleUpdate(rt: MvpRuntime, api: TelegramApi, u: TelegramUpdate): Promise<void> {
  const m = u.message;
  const match = m?.text?.match(START);
  if (!m || !match || !m.from) return;
  let reply: string;
  try {
    reply = match[1] ? answerStart(rt.store, m.from, match[1], rt.now()) : REPLY.hello;
  } catch {
    // A link race (one side linked meanwhile) or a name clash: try again.
    reply = REPLY.failed;
  }
  await api.sendMessage(m.chat.id, reply);
}

/** The bot: one long-polling loop, only when rt.telegram is set. */
export const telegramJob: MvpJob = (rt) => {
  const api = rt.telegram;
  if (!api) return () => {};
  const stop = new AbortController();
  let offset = 0;
  void (async () => {
    while (!stop.signal.aborted) {
      try {
        const updates = await api.getUpdates(offset, stop.signal);
        for (const u of updates) {
          offset = Math.max(offset, u.update_id + 1);
          await handleUpdate(rt, api, u);
        }
      } catch (err) {
        if (stop.signal.aborted) break;
        // Only our own messages: they never carry the token or the URL.
        console.error(`telegram: ${err instanceof TelegramApiError ? err.message : "the bot loop failed"}; again in 5 s`);
        await new Promise((r) => setTimeout(r, 5_000));
      }
    }
  })();
  return () => stop.abort();
};

/** A Bot API failure, worded without the token or the URL. */
export class TelegramApiError extends Error {}

/** The real Bot API. The token stays in this object. */
export class TelegramHttpApi implements TelegramApi {
  readonly #token: string;
  constructor(token: string, readonly bot: string = TELEGRAM_BOT, private readonly fetchFn: typeof fetch = fetch) {
    this.#token = token;
  }
  async #call<T>(method: string, body: unknown, signal?: AbortSignal): Promise<T> {
    let res: Response;
    try {
      res = await this.fetchFn(`https://api.telegram.org/bot${this.#token}/${method}`, {
        method: "POST",
        headers: { "content-type": "application/json" },
        body: JSON.stringify(body),
        signal: signal ? AbortSignal.any([signal, AbortSignal.timeout(40_000)]) : AbortSignal.timeout(15_000),
      });
    } catch {
      throw new TelegramApiError(`${method}: no answer`);
    }
    const json = (await res.json().catch(() => null)) as { ok?: boolean; result?: T } | null;
    if (!res.ok || !json?.ok) throw new TelegramApiError(`${method}: HTTP ${res.status}`);
    return json.result as T;
  }
  getUpdates(offset: number, signal: AbortSignal): Promise<TelegramUpdate[]> {
    return this.#call("getUpdates", { offset, timeout: 25, allowed_updates: ["message"] }, signal);
  }
  async sendMessage(chatId: number, text: string): Promise<void> {
    await this.#call("sendMessage", { chat_id: chatId, text });
  }
}

/** The fake Telegram (ARENA_TELEGRAM_FAKE=1, tests): `press` plays a user
 * sending `/start <code>`; the bot loop reads it like a real update. */
export class FakeTelegram implements TelegramApi {
  readonly sent: { chatId: number; text: string }[] = [];
  #queue: TelegramUpdate[] = [];
  #wake: (() => void) | undefined;
  #nextId = 1;
  #replied: ((text: string) => void)[] = [];
  constructor(readonly bot: string = TELEGRAM_BOT) {}
  /** The user `from` sends `/start <code>`; resolves with the bot's reply. */
  press(code: string, from: TelegramUser = FAKE_TELEGRAM_USER): Promise<string> {
    return this.send(`/start ${code}`, from);
  }
  /** The user `from` sends `text`; resolves with the bot's reply. */
  send(text: string, from: TelegramUser = FAKE_TELEGRAM_USER): Promise<string> {
    this.#queue.push({ update_id: this.#nextId++, message: { chat: { id: from.id }, from, text } });
    const reply = new Promise<string>((r) => this.#replied.push(r));
    this.#wake?.();
    return reply;
  }
  async getUpdates(offset: number, signal: AbortSignal): Promise<TelegramUpdate[]> {
    this.#queue = this.#queue.filter((u) => u.update_id >= offset);
    if (!this.#queue.length)
      await new Promise<void>((r) => {
        this.#wake = r;
        signal.addEventListener("abort", () => r(), { once: true });
      });
    this.#wake = undefined;
    return [...this.#queue];
  }
  async sendMessage(chatId: number, text: string): Promise<void> {
    this.sent.push({ chatId, text });
    this.#replied.shift()?.(text);
  }
}

/** The dev button's Telegram user. */
export const FAKE_TELEGRAM_USER: TelegramUser = { id: 1001, is_bot: false, first_name: "Tester" };

/** The Telegram main.ts runs: the fake (ARENA_TELEGRAM_FAKE=1, dev only),
 * the real bot (ARENA_TELEGRAM_ENV names a file with TELEGRAM_BOT_TOKEN=…),
 * or none (no Telegram login). */
export function telegramFromEnv(env: NodeJS.ProcessEnv, dev: boolean): TelegramApi | undefined {
  const bot = env.ARENA_TELEGRAM_BOT || TELEGRAM_BOT;
  if (env.ARENA_TELEGRAM_FAKE === "1") {
    if (!dev) throw new Error("ARENA_TELEGRAM_FAKE=1 needs MVP_DEV=1");
    return new FakeTelegram(bot);
  }
  const file = env.ARENA_TELEGRAM_ENV;
  if (!file) return undefined;
  const token = readEnvFile(readFileSync(file, "utf8")).TELEGRAM_BOT_TOKEN;
  if (!token) throw new Error(`${file} has no TELEGRAM_BOT_TOKEN`);
  return new TelegramHttpApi(token, bot);
}

/** KEY=value lines (quotes and `export ` allowed, # comments). */
export function readEnvFile(text: string): Record<string, string> {
  const out: Record<string, string> = {};
  for (const line of text.split(/\r?\n/)) {
    const m = line.match(/^\s*(?:export\s+)?([A-Za-z_][A-Za-z0-9_]*)\s*=\s*(.*?)\s*$/);
    if (m) out[m[1]!] = m[2]!.replace(/^(['"])(.*)\1$/, "$2");
  }
  return out;
}
