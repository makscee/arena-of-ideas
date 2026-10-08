// M4-6 (mission #810): log in with Telegram, by a deep link
// (docs/mission4/README.md, Calls). The page asks for a one-time code
// (POST /auth/telegram/start) and opens t.me/<bot>?start=<code>; the bot gets
// `/start <code>` through one long-polling loop (telegramJob, in ./jobs.ts)
// and asks the user what the code would do, with [Yes] [No]; only Yes acts
// (a link code pressed by a stranger never logs anyone in unasked). The page's
// next poll (POST /auth/telegram/poll) takes a session token, the same kind an invite link gives (./invites.ts).
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
import { nameKey, type MvpStore, type TelegramAsk, type TelegramCode } from "./store.js";

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
  declined: "Отменено — ничего не произошло / Declined: nothing happened",
  changed: "Что-то изменилось — начните заново в игре / Something changed: start again in the game",
  notYours: "Эта кнопка не для вас / This button isn't yours",
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
  /** A tap on an inline button: `data` is the button's, `message` its question. */
  callback_query?: { id: string; from: TelegramUser; data?: string; message?: { message_id: number; chat: { id: number } } };
}

/** The bot's side of the Bot API: the real one (TelegramHttpApi) or a fake. */
export interface TelegramApi {
  /** The bot's @name, for the deep link. */
  readonly bot: string;
  /** Long-polls the updates from `offset` on; [] when none came. */
  getUpdates(offset: number, signal: AbortSignal): Promise<TelegramUpdate[]>;
  /** Sends `text`, with `buttons` in one row under it. */
  sendMessage(chatId: number, text: string, buttons?: InlineButton[]): Promise<void>;
  /** Replaces a message's text and drops its buttons. */
  editMessageText(chatId: number, messageId: number, text: string): Promise<void>;
  /** Ends a tap's spinner, showing `text` briefly. */
  answerCallbackQuery(id: string, text: string): Promise<void>;
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

/** The page's poll: "waiting" until the user taps Yes in the bot, then a
 * session for the player it names, once; "declined" once after No; undefined
 * for an unknown, expired or used code. */
export function pollTelegramLogin(store: MvpStore, code: string, now: Date): PlayerSession | "waiting" | "declined" | undefined {
  if (!code) return undefined;
  const h = hashToken(code);
  const c = store.telegramCode(h);
  if (!c) return undefined;
  if (Date.parse(c.expiresAt) <= now.getTime()) {
    store.dropTelegramCode(h);
    return undefined;
  }
  if (c.declined) {
    store.dropTelegramCode(h);
    return "declined";
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

/** The decision for `from` and code `c`, or the reply that refuses it. */
function decide(store: MvpStore, from: TelegramUser, c: TelegramCode, now: Date): TelegramAsk | string {
  if (from.is_bot) return REPLY.bot;
  if (c.readyFor || c.declined || Date.parse(c.expiresAt) <= now.getTime()) return REPLY.expired;
  const tgUserId = String(from.id);
  // A code is asked about once: someone else pressing it later gets nothing.
  if (c.asked && c.asked.tgUserId !== tgUserId) return REPLY.expired;
  const linked = store.telegramLink({ tgUserId });
  if (linked) return isAdmin(store, linked.playerId) ? REPLY.admin : { kind: "login", playerId: linked.playerId, tgUserId };
  if (c.playerId) {
    if (isAdmin(store, c.playerId)) return REPLY.admin;
    if (store.telegramLink({ playerId: c.playerId })) return REPLY.taken;
    return { kind: "link", playerId: c.playerId, tgUserId };
  }
  return { kind: "new", playerId: null, tgUserId };
}

/** A question with Yes/No buttons, or a plain reply. */
export interface BotReply {
  text: string;
  buttons?: InlineButton[];
}
export interface InlineButton {
  text: string;
  data: string;
}

/** The callback data of a question's buttons: Yes or No, and the code. */
const YES = "y:";
const NO = "n:";

/** The bot's answer to `/start <code>` from `from`: never acts at once, but
 * asks what the code would do, with [Yes] [No] (answerTap does it). */
export function answerStart(store: MvpStore, from: TelegramUser, code: string, now: Date): BotReply {
  const h = hashToken(code);
  const c = store.telegramCode(h);
  if (!c) return { text: REPLY.expired };
  const d = decide(store, from, c, now);
  if (typeof d === "string") return { text: d };
  store.putTelegramCode({ ...c, asked: d });
  const name = (id: string) => store.player(id)?.name ?? "?";
  const no = { text: "Нет / No", data: NO + code };
  if (d.kind === "login")
    return {
      text: ask(
        `Войти в Arena of Ideas на устройстве как @${name(d.playerId)}? Нажимайте «Да», только если вы сами только что нажали «Войти через Telegram».`,
        `Log a device in to Arena of Ideas as @${name(d.playerId)}? Only tap Yes if you just tapped "Log in with Telegram" yourself.`,
      ),
      buttons: [{ text: "Да, войти / Yes, log in", data: YES + code }, no],
    };
  if (d.kind === "link")
    return {
      text: ask(
        `Связать этот Telegram с игроком Arena @${name(d.playerId)}? Теперь вы будете входить как @${name(d.playerId)}.`,
        `Link this Telegram to the Arena player @${name(d.playerId)}? You'll log in as @${name(d.playerId)} from now on.`,
      ),
      buttons: [{ text: "Да, связать / Yes, link", data: YES + code }, no],
    };
  const newName = freeName(store, telegramName(from));
  return {
    text: ask(`Создать игрока Arena @${newName} для этого Telegram?`, `Create an Arena player @${newName} for this Telegram?`),
    buttons: [{ text: "Да / Yes", data: YES + code }, no],
  };
}

const ask = (ru: string, en: string) => `${ru}\n\n${en}`;

/** The bot's answer to a tap on [Yes] or [No]: checked again now (the same
 * Telegram user who pressed Start, the code unused and in time, the links as
 * they were), then Yes does what the question said and No drops the code.
 * `edit`: the question's message becomes the result (false: it stays, as for
 * a stranger's tap). */
export function answerTap(store: MvpStore, from: TelegramUser, data: string, now: Date): { text: string; edit: boolean } {
  const yes = data.startsWith(YES);
  if (!yes && !data.startsWith(NO)) return { text: REPLY.expired, edit: false };
  const h = hashToken(data.slice(YES.length));
  const c = store.telegramCode(h);
  if (!c?.asked) return { text: REPLY.expired, edit: true };
  if (c.asked.tgUserId !== String(from.id)) return { text: REPLY.notYours, edit: false };
  const d = decide(store, from, c, now);
  if (typeof d === "string") return { text: d, edit: true };
  if (d.kind !== c.asked.kind || d.playerId !== c.asked.playerId) {
    store.dropTelegramCode(h);
    return { text: REPLY.changed, edit: true };
  }
  if (!yes) {
    // The page's next poll says "Declined in Telegram" and drops it.
    store.putTelegramCode({ ...c, declined: true });
    return { text: REPLY.declined, edit: true };
  }
  const ready = (playerId: string) => {
    store.putTelegramCode({ ...c, readyFor: playerId });
    return { text: REPLY.done, edit: true };
  };
  if (d.kind === "login") return ready(d.playerId);
  if (d.kind === "link") {
    store.linkTelegram({ tgUserId: d.tgUserId, playerId: d.playerId, linkedAt: now.toISOString() });
    return ready(d.playerId);
  }
  // A new player, never admin, with their own invite like a join (R4-20).
  const invite = createInvite(store, { name: freeName(store, telegramName(from)), admin: false, now });
  store.linkTelegram({ tgUserId: d.tgUserId, playerId: invite.playerId, linkedAt: now.toISOString() });
  return ready(invite.playerId);
}

/** A player name from a Telegram name: NAME_RE's letters only, never a bot's. */
export function telegramName(u: TelegramUser): string {
  const clean = (raw: string) => raw.normalize("NFKC").replace(/[^\p{L}\p{N}_\- ]/gu, "").replace(/\s+/g, " ").trim().slice(0, 20).trim();
  let name = clean([u.first_name, u.last_name].filter(Boolean).join(" ")) || clean(u.username ?? "") || "Player";
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

/** Answers one update: `/start <code>`, a bare `/start` and a tap on a
 * question's button; the rest is ignored. */
export async function handleUpdate(rt: MvpRuntime, api: TelegramApi, u: TelegramUpdate): Promise<void> {
  const q = u.callback_query;
  if (q) {
    let r: { text: string; edit: boolean };
    try {
      r = answerTap(rt.store, q.from, q.data ?? "", rt.now());
    } catch {
      // A link race (one side linked meanwhile) or a name clash: try again.
      r = { text: REPLY.failed, edit: true };
    }
    if (r.edit && q.message) await api.editMessageText(q.message.chat.id, q.message.message_id, r.text);
    await api.answerCallbackQuery(q.id, r.text);
    return;
  }
  const m = u.message;
  const match = m?.text?.match(START);
  if (!m || !match || !m.from) return;
  let reply: BotReply;
  try {
    reply = match[1] ? answerStart(rt.store, m.from, match[1], rt.now()) : { text: REPLY.hello };
  } catch {
    reply = { text: REPLY.failed };
  }
  await api.sendMessage(m.chat.id, reply.text, reply.buttons);
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
    return this.#call("getUpdates", { offset, timeout: 25, allowed_updates: ["message", "callback_query"] }, signal);
  }
  async sendMessage(chatId: number, text: string, buttons?: InlineButton[]): Promise<void> {
    const markup = buttons?.length ? { reply_markup: { inline_keyboard: [buttons.map((b) => ({ text: b.text, callback_data: b.data }))] } } : {};
    await this.#call("sendMessage", { chat_id: chatId, text, ...markup });
  }
  async editMessageText(chatId: number, messageId: number, text: string): Promise<void> {
    await this.#call("editMessageText", { chat_id: chatId, message_id: messageId, text });
  }
  async answerCallbackQuery(id: string, text: string): Promise<void> {
    await this.#call("answerCallbackQuery", { callback_query_id: id, text });
  }
}

/** The fake Telegram (ARENA_TELEGRAM_FAKE=1, tests): `press` plays a user
 * sending `/start <code>`, `tap` a tap on the question's Yes or No; the bot
 * loop reads them like real updates. */
export class FakeTelegram implements TelegramApi {
  /** Every message the bot sent, as it reads now (edits applied). */
  readonly sent: { chatId: number; messageId: number; text: string; buttons?: InlineButton[] }[] = [];
  #queue: { update: TelegramUpdate; reply: (text: string) => void }[] = [];
  #wake: (() => void) | undefined;
  #inFlight: ((text: string) => void) | undefined;
  #nextId = 1;
  #questions = new Map<string, { chatId: number; messageId: number }>();
  constructor(readonly bot: string = TELEGRAM_BOT) {}
  /** The user `from` sends `/start <code>`; resolves with the bot's reply. */
  press(code: string, from: TelegramUser = FAKE_TELEGRAM_USER): Promise<string> {
    return this.send(`/start ${code}`, from);
  }
  /** The user `from` sends `text`; resolves with the bot's reply. */
  send(text: string, from: TelegramUser = FAKE_TELEGRAM_USER): Promise<string> {
    return this.#push({ update_id: this.#nextId++, message: { chat: { id: from.id }, from, text } });
  }
  /** `from` taps Yes (or No) on the newest question about `code`, even one
   * already answered (a double tap); resolves with the bot's answer to the tap. */
  tap(code: string, yes: boolean, from: TelegramUser = FAKE_TELEGRAM_USER): Promise<string> {
    const data = (yes ? YES : NO) + code;
    const msg = this.#questions.get(data);
    if (!msg) return Promise.reject(new Error("no question about this code"));
    const id = this.#nextId++;
    return this.#push({ update_id: id, callback_query: { id: `cb${id}`, from, data, message: { message_id: msg.messageId, chat: { id: msg.chatId } } } });
  }
  /** Both steps: `/start <code>`, then Yes (or No) when the bot asks. */
  async answer(code: string, yes: boolean, from: TelegramUser = FAKE_TELEGRAM_USER): Promise<string> {
    const reply = await this.press(code, from);
    const asked = this.sent.at(-1);
    return asked?.chatId === from.id && asked.buttons?.length ? this.tap(code, yes, from) : reply;
  }
  #push(update: TelegramUpdate): Promise<string> {
    const reply = new Promise<string>((r) => this.#queue.push({ update, reply: r }));
    this.#wake?.();
    return reply;
  }
  // One update at a time, so the bot's next reply answers it.
  async getUpdates(offset: number, signal: AbortSignal): Promise<TelegramUpdate[]> {
    this.#queue = this.#queue.filter((q) => q.update.update_id >= offset);
    if (!this.#queue.length)
      await new Promise<void>((r) => {
        this.#wake = r;
        signal.addEventListener("abort", () => r(), { once: true });
      });
    this.#wake = undefined;
    const next = this.#queue[0];
    if (!next) return [];
    this.#inFlight = next.reply;
    return [next.update];
  }
  #answered(text: string): void {
    this.#inFlight?.(text);
    this.#inFlight = undefined;
  }
  async sendMessage(chatId: number, text: string, buttons?: InlineButton[]): Promise<void> {
    const messageId = this.#nextId++;
    this.sent.push({ chatId, messageId, text, ...(buttons?.length ? { buttons } : {}) });
    for (const b of buttons ?? []) this.#questions.set(b.data, { chatId, messageId });
    this.#answered(text);
  }
  async editMessageText(chatId: number, messageId: number, text: string): Promise<void> {
    const m = this.sent.find((x) => x.chatId === chatId && x.messageId === messageId);
    if (m) {
      m.text = text;
      delete m.buttons;
    }
  }
  async answerCallbackQuery(_id: string, text: string): Promise<void> {
    this.#answered(text);
  }
}

/** The dev button's Telegram user. */
export const FAKE_TELEGRAM_USER: TelegramUser = { id: 1001, is_bot: false, first_name: "Tester" };

/** The Telegram main.ts runs: the fake (ARENA_TELEGRAM_FAKE=1, dev only),
 * the real bot (ARENA_TELEGRAM_ENV names a file with TELEGRAM_BOT_TOKEN=…),
 * or none (no Telegram login, also when the file is missing or has no token). */
export function telegramFromEnv(env: NodeJS.ProcessEnv, dev: boolean): TelegramApi | undefined {
  const bot = env.ARENA_TELEGRAM_BOT || TELEGRAM_BOT;
  if (env.ARENA_TELEGRAM_FAKE === "1") {
    if (!dev) throw new Error("ARENA_TELEGRAM_FAKE=1 needs MVP_DEV=1");
    return new FakeTelegram(bot);
  }
  const file = env.ARENA_TELEGRAM_ENV;
  if (!file) return undefined;
  // A missing or broken file turns Telegram login off; the game still starts.
  let text: string;
  try {
    text = readFileSync(file, "utf8");
  } catch {
    console.error(`telegram: can't read ${file}: Telegram login is off`);
    return undefined;
  }
  const token = readEnvFile(text).TELEGRAM_BOT_TOKEN;
  if (!token) {
    console.error(`telegram: ${file} has no TELEGRAM_BOT_TOKEN: Telegram login is off`);
    return undefined;
  }
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
