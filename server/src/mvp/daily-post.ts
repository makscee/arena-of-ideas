// The daily post to Telegram (M4-8, mission #810). After the day end (04:00
// Moscow, ./day.ts endDay, rotation included) the bot posts to the channel:
// the new champion and their team with M4-7's share card as the photo, how
// many slew the old one, the units that entered and left the pool ("idea by
// @…", "evolved by @…") and a link to the game. One message per language.
//
// Off unless ARENA_TELEGRAM_POST=1. It runs as a job beside the rollover,
// never inside endDay: a slow or failing Telegram never holds up or fails the
// day end. Each message is claimed in the store before it is sent
// (mvp_daily_posts), so a retried day end or a restart never posts a day
// twice; a failed send is retried with backoff, then recorded as failed and
// logged. The bot's token is read from the file ARENA_TELEGRAM_ENV names and
// never logged.
//
// Env (main.ts):
//   ARENA_TELEGRAM_POST        1 posts; anything else, nothing is sent
//   ARENA_TELEGRAM_CHANNEL     the channel, e.g. @arenaofideas
//   ARENA_TELEGRAM_ENV         a file with TELEGRAM_BOT_TOKEN=… (M4-6's)
//   ARENA_TELEGRAM_POST_LANGS  messages and their languages: "ru,en" (default)
//                              is one Russian and one English message;
//                              "ru+en" is one message with both
//   ARENA_TELEGRAM_API         the Bot API's address (tests: a fake)
//   ARENA_PUBLIC_URL           the game's address in the post
// Dry run: npm run mvp:post -- --db <copy> [--day <seq>] --dry-run
import { randomBytes } from "node:crypto";
import { readFileSync } from "node:fs";
import type { Champion, PlayerRef } from "../../../src/mvp/contract.js";
import { versionCredits } from "./credits.js";
import { championOf } from "./day.js";
import type { MvpJob, MvpRuntime } from "./runtime.js";
import { championSvg, dateText, PUBLIC_URL, sharePng, type ShareLang } from "./share.js";
import type { DailyPost } from "./store.js";

type PostDeps = Pick<MvpRuntime, "store" | "content" | "today">;

// ---------- what the day brought ----------

export interface PostUnit {
  emoji: string;
  name: string;
  by: PlayerRef | null;
  evolvedBy: PlayerRef | null;
  /** Back from the Library unchanged (M3-7). */
  returned: boolean;
}

/** What day `seq`'s post tells: the day that ended is seq - 1. */
export interface PostFacts {
  seq: number;
  /** Day seq's date label (YYYY-MM-DD). */
  day: string;
  champion: Champion | null;
  /** True: the playoff crowned them; false: the champion kept the throne. */
  crowned: boolean;
  /** The players who slew the champion on the day that ended. */
  slayers: number;
  entered: PostUnit[];
  left: PostUnit[];
}

export function postFacts(rt: PostDeps, seq: number): PostFacts {
  const { store } = rt;
  const champion = championOf(store, seq) ?? null;
  const credit = versionCredits(rt);
  const unit = (unitId: string, returned: boolean): PostUnit | null => {
    const u = store.unit(unitId);
    if (!u) return null;
    const c = credit(u);
    return { emoji: u.row.emoji, name: u.row.name, by: c.by, evolvedBy: c.evolvedBy, returned };
  };
  const stints = store.stints();
  const entered = stints.filter((s) => s.enteredSeq === seq && s.leftSeq !== seq && s.reason !== "seed").map((s) => unit(s.unitId, s.reason === "return"));
  const left = stints.filter((s) => s.leftSeq === seq).map((s) => unit(s.unitId, false));
  const t = rt.today();
  return {
    seq,
    day: champion?.seq === seq ? champion.day : seq === t.seq ? t.day : (store.champion(seq)?.day ?? t.day),
    champion,
    crowned: !!store.playoff(seq - 1)?.winner,
    slayers: new Set(store.slays(seq - 1).map((s) => s.player.id)).size,
    entered: entered.filter((u): u is PostUnit => !!u),
    left: left.filter((u): u is PostUnit => !!u),
  };
}

// ---------- the words ----------

const WORDS = {
  en: {
    champion: (date: string, who: string) => `👑 Champion of ${date}: ${who}`,
    keeps: (date: string, who: string) => `👑 ${who} keeps the crown on ${date}`,
    none: (date: string) => `👑 ${date}: the throne is empty`,
    slayers: (n: number) => (n === 0 ? "⚔️ No one slew the champion yesterday." : `⚔️ ${n} ${n === 1 ? "player" : "players"} slew the champion yesterday.`),
    canYou: "Can you beat them?",
    entered: "New in the arena:",
    left: "Gone to the Library:",
    ideaBy: (p: string) => `idea by @${p}`,
    evolvedBy: (p: string) => `evolved by @${p}`,
    returned: "back from the Library",
    more: (n: number) => `…and ${n} more`,
    play: (url: string) => `Play: ${url}`,
  },
  ru: {
    champion: (date: string, who: string) => `👑 Чемпион ${date}: ${who}`,
    keeps: (date: string, who: string) => `👑 ${who} сохраняет корону ${date}`,
    none: (date: string) => `👑 ${date}: трон пуст`,
    slayers: (n: number) => (n === 0 ? "⚔️ Вчера чемпиона не сразил никто." : `⚔️ Вчера чемпиона сразили: ${n}.`),
    canYou: "Сможешь победить?",
    entered: "Новые в арене:",
    left: "Ушли в Библиотеку:",
    ideaBy: (p: string) => `идея @${p}`,
    evolvedBy: (p: string) => `развил @${p}`,
    returned: "вернулся из Библиотеки",
    more: (n: number) => `…и ещё ${n}`,
    play: (url: string) => `Играть: ${url}`,
  },
} satisfies Record<ShareLang, unknown>;

/** A photo's caption holds at most 1024 characters (Telegram's limit). */
export const CAPTION_MAX = 1024;

function unitLine(u: PostUnit, lang: ShareLang): string {
  const w = WORDS[lang];
  const credit = u.returned ? [w.returned] : [u.by ? w.ideaBy(u.by.name) : null, u.evolvedBy ? w.evolvedBy(u.evolvedBy.name) : null].filter(Boolean);
  return `${u.emoji} ${u.name}${credit.length ? ` — ${credit.join(", ")}` : ""}`;
}

/** One language's text; `max` caps each unit list (the rest: "…and N more"). */
export function postText(f: PostFacts, lang: ShareLang, publicUrl = PUBLIC_URL, max = Infinity): string {
  const w = WORDS[lang];
  const date = dateText(f.day, lang);
  const out: string[] = [];
  if (!f.champion) out.push(w.none(date));
  else {
    const who = `@${f.champion.player.name}`;
    out.push(f.crowned ? w.champion(date, who) : w.keeps(date, who));
    out.push(f.champion.line.map((u) => `${u.emoji} ${u.name}`).join(" · "));
  }
  out.push(`${w.slayers(f.slayers)}${f.champion ? ` ${w.canYou}` : ""}`);
  for (const [title, units] of [[w.entered, f.entered], [w.left, f.left]] as const) {
    if (!units.length) continue;
    const shown = units.slice(0, max);
    out.push("", title, ...shown.map((u) => unitLine(u, lang)));
    if (units.length > shown.length) out.push(w.more(units.length - shown.length));
  }
  out.push("", w.play(`${publicUrl.replace(/\/$/, "")}/`));
  return out.join("\n");
}

export interface DailyMessage {
  /** The store's key: the languages joined by "+". */
  key: string;
  langs: ShareLang[];
  text: string;
  /** The champion's share card in the first language; null with no champion. */
  png: Buffer | null;
}

/** One message per group of languages; texts of a group follow each other,
 * unit lists cut so the whole fits a photo's caption. */
export function dailyMessages(rt: PostDeps, seq: number, groups: ShareLang[][], publicUrl = PUBLIC_URL): DailyMessage[] {
  const f = postFacts(rt, seq);
  return groups.map((langs) => {
    const text = (max: number) => langs.map((l) => postText(f, l, publicUrl, max)).join("\n\n");
    let max = Math.max(f.entered.length, f.left.length);
    while (max > 0 && text(max).length > CAPTION_MAX) max--;
    const svg = championSvg(rt, seq, langs[0]!);
    return { key: langs.join("+"), langs, text: text(max), png: svg ? sharePng(svg).png : null };
  });
}

// ---------- Telegram ----------

export interface TelegramSender {
  /** Posts a photo with a caption, or a text when `png` is null. */
  post(chat: string, text: string, png: Buffer | null): Promise<void>;
}

export class TelegramError extends Error {
  constructor(message: string, readonly retryAfterMs?: number) {
    super(message);
  }
}

/** The token from an env file's TELEGRAM_BOT_TOKEN=… line; throws naming the
 * file, never the value. */
export function readBotToken(file: string): string {
  const text = readFileSync(file, "utf8");
  const m = /^\s*(?:export\s+)?TELEGRAM_BOT_TOKEN\s*=\s*["']?([^"'\s#]+)/m.exec(text);
  if (!m) throw new Error(`${file} has no TELEGRAM_BOT_TOKEN=`);
  return m[1]!;
}

/** The Bot API (sendPhoto, sendMessage) at `api`. Errors never carry the token. */
export function telegramSender(token: string, api = "https://api.telegram.org", fetchImpl: typeof fetch = fetch): TelegramSender {
  const hide = (s: string) => s.split(token).join("<token>");
  return {
    async post(chat, text, png) {
      const method = png ? "sendPhoto" : "sendMessage";
      let body: Blob | string;
      const headers: Record<string, string> = {};
      if (png) {
        // Multipart by hand: FormData would turn the caption's \n into \r\n.
        const boundary = `arena${randomBytes(12).toString("hex")}`;
        const field = (name: string, value: string) => `--${boundary}\r\nContent-Disposition: form-data; name="${name}"\r\n\r\n${value}\r\n`;
        body = new Blob([
          field("chat_id", chat) + field("caption", text) + `--${boundary}\r\nContent-Disposition: form-data; name="photo"; filename="champion.png"\r\nContent-Type: image/png\r\n\r\n`,
          new Uint8Array(png),
          `\r\n--${boundary}--\r\n`,
        ]);
        headers["content-type"] = `multipart/form-data; boundary=${boundary}`;
      } else {
        body = JSON.stringify({ chat_id: chat, text, link_preview_options: { is_disabled: true } });
        headers["content-type"] = "application/json";
      }
      let res: Response;
      try {
        res = await fetchImpl(`${api.replace(/\/$/, "")}/bot${token}/${method}`, { method: "POST", body, headers, signal: AbortSignal.timeout(30_000) });
      } catch (err) {
        throw new TelegramError(hide(`${method}: ${(err as Error).message}`));
      }
      const j = (await res.json().catch(() => null)) as { ok?: boolean; description?: string; parameters?: { retry_after?: number } } | null;
      if (!res.ok || !j?.ok) {
        const after = j?.parameters?.retry_after;
        throw new TelegramError(hide(`${method}: HTTP ${res.status}${j?.description ? ` ${j.description}` : ""}`), after ? after * 1000 : undefined);
      }
    },
  };
}

// ---------- posting a day ----------

export interface PostOptions {
  send: TelegramSender;
  channel: string;
  groups: ShareLang[][];
  publicUrl?: string;
  /** Tries per message (default 4). */
  tries?: number;
  /** The waits between tries (default 10 s, 1 min, 5 min); Telegram's
   * retry_after wins when longer, up to 10 min. */
  backoffMs?: number[];
  sleep?: (ms: number) => Promise<void>;
  now?: () => Date;
  log?: (line: string) => void;
}

export const BACKOFF_MS = [10_000, 60_000, 300_000];
const sleepMs = (ms: number) => new Promise<void>((r) => setTimeout(r, ms).unref?.());

/** Posts day `seq` once: each message is claimed first, and a claimed one is
 * never sent again. Never throws; returns what it recorded. */
export async function postDay(rt: PostDeps, seq: number, o: PostOptions): Promise<DailyPost[]> {
  const log = o.log ?? ((l: string) => console.log(l));
  const now = () => (o.now ? o.now() : new Date()).toISOString();
  const tries = o.tries ?? 4;
  const backoff = o.backoffMs ?? BACKOFF_MS;
  const sleep = o.sleep ?? sleepMs;
  const done: DailyPost[] = [];
  let messages: DailyMessage[];
  try {
    messages = dailyMessages(rt, seq, o.groups, o.publicUrl);
  } catch (err) {
    log(`[post] day ${seq}: the post couldn't be made: ${(err as Error).message}`);
    return done;
  }
  for (const m of messages) {
    if (!rt.store.claimDailyPost(seq, m.key, now())) continue;
    let rec: DailyPost = { seq, key: m.key, state: "failed", tries: 0, at: now() };
    for (let i = 1; i <= tries; i++) {
      try {
        await o.send.post(o.channel, m.text, m.png);
        rec = { seq, key: m.key, state: "sent", tries: i, at: now() };
        log(`[post] day ${seq} ${m.key}: sent to ${o.channel}${i > 1 ? ` on try ${i}` : ""}`);
        break;
      } catch (err) {
        const e = err as TelegramError;
        rec = { seq, key: m.key, state: "failed", tries: i, at: now(), error: e.message };
        if (i === tries) {
          log(`[post] day ${seq} ${m.key}: failed after ${i} tries: ${e.message}`);
          break;
        }
        const wait = Math.min(Math.max(backoff[Math.min(i - 1, backoff.length - 1)] ?? 0, e.retryAfterMs ?? 0), 600_000);
        log(`[post] day ${seq} ${m.key}: try ${i} failed (${e.message}); again in ${Math.round(wait / 1000)}s`);
        await sleep(wait);
      }
    }
    rt.store.putDailyPost(rec);
    done.push(rec);
  }
  return done;
}

// ---------- the job ----------

export interface DailyPostConfig {
  on: boolean;
  channel: string;
  groups: ShareLang[][];
  tokenFile: string;
  api: string;
  publicUrl: string;
}

/** "ru,en" → [["ru"], ["en"]]; "ru+en" → [["ru", "en"]]. Unknown languages are dropped. */
export function postGroups(spec: string | undefined): ShareLang[][] {
  const groups = (spec ?? "ru,en").split(",").map((g) => g.split("+").map((l) => l.trim()).filter((l): l is ShareLang => l === "ru" || l === "en"));
  const out = groups.filter((g) => g.length);
  return out.length ? out : [["ru"], ["en"]];
}

export function dailyPostConfig(env: NodeJS.ProcessEnv = process.env): DailyPostConfig {
  return {
    on: env.ARENA_TELEGRAM_POST === "1",
    channel: env.ARENA_TELEGRAM_CHANNEL ?? "",
    groups: postGroups(env.ARENA_TELEGRAM_POST_LANGS),
    tokenFile: env.ARENA_TELEGRAM_ENV ?? "",
    api: env.ARENA_TELEGRAM_API ?? "https://api.telegram.org",
    publicUrl: (env.ARENA_PUBLIC_URL ?? PUBLIC_URL).replace(/\/$/, ""),
  };
}

/** Posts only for a day that started at most this long ago: turning the post
 * on (or a server down past the day end) never posts a stale day. */
export const POST_WINDOW_MS = 6 * 3600_000;
export const POST_TICK_MS = 30_000;

/** The job with a sender: each tick, today's post when the day started
 * within POST_WINDOW_MS and it isn't claimed yet. One post at a time. */
export function dailyPostJobWith(cfg: Pick<DailyPostConfig, "channel" | "groups" | "publicUrl">, send: TelegramSender, opts: Partial<PostOptions> & { everyMs?: number } = {}): MvpJob {
  return (rt) => {
    let busy = false;
    const tick = () => {
      if (busy) return;
      try {
        const d = rt.today();
        if (d.seq < 2 || rt.now().getTime() - Date.parse(d.startedAt) > POST_WINDOW_MS) return;
        const keys = new Set(rt.store.dailyPosts(d.seq).map((p) => p.key));
        if (cfg.groups.every((g) => keys.has(g.join("+")))) return;
        busy = true;
        void postDay(rt, d.seq, { now: rt.now, ...opts, send, channel: cfg.channel, groups: cfg.groups, publicUrl: cfg.publicUrl }).finally(() => (busy = false));
      } catch (err) {
        console.error("[post] the daily post job failed", err);
      }
    };
    tick();
    const timer = setInterval(tick, opts.everyMs ?? POST_TICK_MS);
    timer.unref?.();
    return () => clearInterval(timer);
  };
}

/** The daily post job (./jobs.ts): nothing unless ARENA_TELEGRAM_POST=1. */
export const dailyPostJob: MvpJob = (rt) => {
  const cfg = dailyPostConfig();
  if (!cfg.on) return () => {};
  if (!cfg.channel || !cfg.tokenFile) {
    console.error("[post] ARENA_TELEGRAM_POST=1 needs ARENA_TELEGRAM_CHANNEL and ARENA_TELEGRAM_ENV; not posting");
    return () => {};
  }
  let token: string;
  try {
    token = readBotToken(cfg.tokenFile);
  } catch (err) {
    console.error(`[post] not posting: ${(err as Error).message}`);
    return () => {};
  }
  console.log(`[post] the daily post is on: ${cfg.channel}, ${cfg.groups.map((g) => g.join("+")).join(", ")}`);
  return dailyPostJobWith(cfg, telegramSender(token, cfg.api))(rt);
};
