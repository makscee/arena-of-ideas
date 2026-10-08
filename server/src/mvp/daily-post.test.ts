// M4-8: the daily post, against a fake Telegram Bot API (a local HTTP server):
// off by default, posted once per day (a retried day end or a restart never
// posts twice), retried with backoff, and the day end unaffected by a failure.
import { mkdtempSync, writeFileSync } from "node:fs";
import { createServer, type Server } from "node:http";
import type { AddressInfo } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { afterEach, describe, expect, it } from "vitest";
import type { Champion, PlayerRef } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import { CAPTION_MAX, dailyMessages, dailyPostConfig, dailyPostJob, dailyPostJobWith, postDay, postFacts, postGroups, postText, POST_WINDOW_MS, readBotToken, telegramSender, type TelegramSender } from "./daily-post.js";
import { poolContent, seedUnits } from "./pool.js";
import { mvpRuntime } from "./runtime.js";
import { championSvg, sharePng } from "./share.js";
import { MemoryMvpStore } from "./store.js";

const TOKEN = "123456:TEST-token-never-real";
const ann: PlayerRef = { id: "p-ann", name: "ann", bot: false };
const lev: PlayerRef = { id: "p-lev", name: "lev", bot: false };
const bob: PlayerRef = { id: "p-bob", name: "Bot Bob", bot: true };

/** A fake Bot API: records each call; `fail` answers the next calls with an error. */
interface Call {
  method: string;
  token: string;
  chat: string;
  text: string;
  photo: Uint8Array | null;
}
async function fakeTelegram() {
  const calls: Call[] = [];
  const fail: { status: number; retryAfter?: number }[] = [];
  const server: Server = createServer((req, res) => {
    const chunks: Buffer[] = [];
    req.on("data", (c: Buffer) => chunks.push(c));
    req.on("end", async () => {
      const m = /^\/bot([^/]+)\/(\w+)$/.exec(req.url ?? "");
      const body = Buffer.concat(chunks);
      const type = req.headers["content-type"] ?? "";
      let chat = "", text = "", photo: Uint8Array | null = null;
      if (type.startsWith("multipart/")) {
        const form = await new Response(body, { headers: { "content-type": type } }).formData();
        chat = String(form.get("chat_id"));
        text = String(form.get("caption"));
        photo = new Uint8Array(await (form.get("photo") as Blob).arrayBuffer());
      } else {
        const j = JSON.parse(body.toString()) as { chat_id: string; text: string };
        [chat, text] = [j.chat_id, j.text];
      }
      calls.push({ method: m?.[2] ?? "?", token: m?.[1] ?? "", chat, text, photo });
      const f = fail.shift();
      res.writeHead(f ? f.status : 200, { "content-type": "application/json" });
      res.end(JSON.stringify(f ? { ok: false, description: "fake failure", ...(f.retryAfter ? { parameters: { retry_after: f.retryAfter } } : {}) } : { ok: true, result: {} }));
    });
  });
  await new Promise<void>((r) => server.listen(0, "127.0.0.1", r));
  const api = `http://127.0.0.1:${(server.address() as AddressInfo).port}`;
  open.push(server);
  return { api, calls, fail, send: telegramSender(TOKEN, api) };
}
const open: Server[] = [];
afterEach(async () => {
  for (const s of open.splice(0)) await new Promise((r) => s.close(r));
});

const DAY1 = new Date("2026-10-08T08:00:00.000Z"); // 11:00 Moscow, day 1
const AFTER = new Date("2026-10-09T01:00:30.000Z"); // 04:00:30 Moscow: day 1 ended

/** Day 1 with a champion (ann) and two slayers (lev, a bot); `clock.t` moves the time. */
function world() {
  const store = new MemoryMvpStore();
  const clock = { t: DAY1 };
  seedUnits(store, DAY1);
  const rt = mvpRuntime({ content: poolContent(store), store, now: () => clock.t, seed: () => 7 });
  const units = rt.content.units;
  const line = units.slice(0, 5).map((u, i) => lineUnitOf(u, `u${i}`, 1, rt.rules));
  const d = rt.today();
  for (const p of [ann, lev, bob]) store.addPlayer(p);
  const champ: Champion = { seq: d.seq, day: d.day, player: ann, line, since: DAY1.toISOString(), contentVersion: rt.content.version };
  store.putChampion(champ);
  for (const [i, p] of [lev, bob].entries())
    store.addSlay({ seq: d.seq, player: p, runId: `r${i}`, battleId: `b${i}`, line: structuredClone(line), contentVersion: rt.content.version, at: DAY1.toISOString() });
  return { store, rt, clock, units };
}

/** As a rotation at the end of day 1 would: lev's idea enters, a seed unit leaves. */
function rotateIn(w: ReturnType<typeof world>, seq = 2) {
  const seed = w.store.units()[0]!;
  w.store.putUnit({ unitId: "hedgehog", status: "live", row: { ...seed.row, name: "Hedgehog", emoji: "🦔" }, authorId: lev.id, origin: "idea", parentId: null, createdAt: AFTER.toISOString() });
  w.store.putStint({ unitId: "hedgehog", enteredSeq: seq, leftSeq: null, reason: "idea" });
  w.store.putUnit({ ...seed, status: "library" });
  w.store.putStint({ unitId: seed.unitId, enteredSeq: 1, leftSeq: seq, reason: "rotated" });
  return seed;
}

const until = async (ok: () => boolean, ms = 5000) => {
  const end = Date.now() + ms;
  while (!ok()) {
    if (Date.now() > end) throw new Error("timed out");
    await new Promise((r) => setTimeout(r, 10));
  }
};
const quiet = { log: () => {}, sleep: async () => {} };
const CFG = { channel: "@arenaofideas", groups: [["ru"], ["en"]] as ("ru" | "en")[][], publicUrl: "https://arena.makscee.ru/arena" };

describe("the daily post (M4-8)", () => {
  it("is off by default: no env, no job, nothing sent through a day end", async () => {
    expect(dailyPostConfig({}).on).toBe(false);
    expect(dailyPostConfig({ ARENA_TELEGRAM_POST: "0" }).on).toBe(false);
    expect(dailyPostConfig({ ARENA_TELEGRAM_POST: "1" }).on).toBe(true);
    const tg = await fakeTelegram();
    const w = world();
    const saved = process.env.ARENA_TELEGRAM_POST;
    delete process.env.ARENA_TELEGRAM_POST;
    try {
      const stop = dailyPostJob(w.rt);
      w.clock.t = AFTER;
      expect(w.rt.today().seq).toBe(2);
      await new Promise((r) => setTimeout(r, 50));
      stop();
    } finally {
      if (saved !== undefined) process.env.ARENA_TELEGRAM_POST = saved;
    }
    expect(tg.calls).toEqual([]);
    expect(w.store.dailyPosts()).toEqual([]);
  });

  it("after the day end: one Russian and one English photo post, with the champion, slayers, units in and out and the link", async () => {
    const tg = await fakeTelegram();
    const w = world();
    w.clock.t = AFTER;
    expect(w.rt.today().seq).toBe(2); // the day end crowns lev or the bot
    const left = rotateIn(w);
    const done = await postDay(w.rt, 2, { ...CFG, ...quiet, send: tg.send });
    expect(done.map((p) => [p.key, p.state, p.tries])).toEqual([["ru", "sent", 1], ["en", "sent", 1]]);
    expect(tg.calls.map((c) => [c.method, c.token, c.chat])).toEqual([["sendPhoto", TOKEN, "@arenaofideas"], ["sendPhoto", TOKEN, "@arenaofideas"]]);
    const champ = w.store.champion(2)!;
    expect([lev.id, bob.id]).toContain(champ.player.id);
    const [ru, en] = tg.calls;
    expect(en!.text).toContain(`👑 Champion of Oct 9, 2026: ${champ.player.name}`);
    expect(en!.text).toContain(champ.line.map((u) => `${u.emoji} ${u.name}`).join(" · "));
    expect(en!.text).toContain("⚔️ 2 slayers yesterday. Can you beat the champion?");
    expect(en!.text).toContain("New in the arena:\n🦔 Hedgehog — idea by lev");
    // Game names never carry "@": Telegram would link a stranger's username.
    expect(en!.text).not.toMatch(/(?<![\w/])@(?!arenaofideas)/);
    expect(en!.text).toContain(`Gone to the Library:\n${left.row.emoji} ${left.row.name}`);
    expect(en!.text.endsWith("Play: https://arena.makscee.ru/arena/")).toBe(true);
    expect(ru!.text).toContain(`👑 Чемпион 9 октября 2026: ${champ.player.name}`);
    expect(ru!.text).toContain("⚔️ Вчера 2 убийцы чемпиона. Сможешь победить?");
    expect(ru!.text).toContain("Новые в арене:\n🦔 Hedgehog — идея lev");
    expect(ru!.text).toContain("Играть: https://arena.makscee.ru/arena/");
    for (const c of tg.calls) expect([...c.photo!.slice(0, 4)]).toEqual([0x89, 0x50, 0x4e, 0x47]);
  });

  it("the photo agrees with the text: the card counts the ended day's slayers, the live card today's", async () => {
    const w = world();
    w.clock.t = AFTER;
    expect(w.rt.today().seq).toBe(2);
    // The live /share card of today: no slays yet on day 2.
    expect(championSvg(w.rt, 2, "en")).toContain("0 slayers · Can you beat them?");
    expect(championSvg(w.rt, 2, "en", 2)).toContain("2 slew yesterday · Can you beat them?");
    expect(championSvg(w.rt, 2, "ru", 2)).toContain("Сразили вчера: 2 · Сможешь победить?");
    const [ru, en] = dailyMessages(w.rt, 2, [["ru"], ["en"]], CFG.publicUrl);
    expect(postFacts(w.rt, 2).slayers).toBe(2);
    expect(Buffer.compare(ru!.png!, sharePng(championSvg(w.rt, 2, "ru", 2)!).png)).toBe(0);
    expect(Buffer.compare(en!.png!, sharePng(championSvg(w.rt, 2, "en", 2)!).png)).toBe(0);
  });

  it("counts slayers in Russian with the right word form", () => {
    const w = world();
    w.clock.t = AFTER;
    const f = postFacts(w.rt, 2);
    const line = (n: number) => postText({ ...f, slayers: n }, "ru").split("\n").find((l) => l.startsWith("⚔️"));
    expect([1, 2, 5, 11, 21, 22, 112].map(line)).toEqual([
      "⚔️ Вчера 1 убийца чемпиона. Сможешь победить?",
      "⚔️ Вчера 2 убийцы чемпиона. Сможешь победить?",
      "⚔️ Вчера 5 убийц чемпиона. Сможешь победить?",
      "⚔️ Вчера 11 убийц чемпиона. Сможешь победить?",
      "⚔️ Вчера 21 убийца чемпиона. Сможешь победить?",
      "⚔️ Вчера 22 убийцы чемпиона. Сможешь победить?",
      "⚔️ Вчера 112 убийц чемпиона. Сможешь победить?",
    ]);
    expect(postText({ ...f, slayers: 0 }, "ru")).toContain("⚔️ Вчера убийц чемпиона не было. Сможешь победить?");
    expect(postText({ ...f, slayers: 1 }, "en")).toContain("⚔️ 1 slayer yesterday. Can you beat the champion?");
  });

  it("posts a day once: again, from a second job (a restart) or after a retried day end, nothing more is sent", async () => {
    const tg = await fakeTelegram();
    const w = world();
    const job = dailyPostJobWith(CFG, tg.send, { ...quiet, everyMs: 5 });
    const stop1 = job(w.rt);
    w.clock.t = AFTER; // the job's own tick ends the day and posts
    await until(() => w.store.dailyPosts(2).filter((p) => p.state === "sent").length === 2);
    await new Promise((r) => setTimeout(r, 50));
    stop1();
    const stop2 = job(w.rt); // a restart
    await new Promise((r) => setTimeout(r, 50));
    stop2();
    expect(await postDay(w.rt, 2, { ...CFG, ...quiet, send: tg.send })).toEqual([]);
    expect(tg.calls).toHaveLength(2);
    // The next day end posts the next day, once.
    w.clock.t = new Date(AFTER.getTime() + 24 * 3600_000);
    const stop3 = job(w.rt);
    await until(() => w.store.dailyPosts(3).filter((p) => p.state === "sent").length === 2);
    await new Promise((r) => setTimeout(r, 50));
    stop3();
    expect(tg.calls).toHaveLength(4);
    expect(tg.calls[2]!.text).toContain("сохраняет корону"); // no slayers on day 2: the champion keeps the throne
    expect(tg.calls[3]!.text).toContain("keeps the crown on Oct 10, 2026");
    expect(tg.calls[3]!.text).toContain("⚔️ No slayers yesterday.");
  });

  it("a failed send is retried with backoff (Telegram's retry_after wins), then recorded as sent", async () => {
    const tg = await fakeTelegram();
    const w = world();
    w.clock.t = AFTER;
    w.rt.today();
    tg.fail.push({ status: 500 }, { status: 429, retryAfter: 120 });
    const waits: number[] = [];
    const lines: string[] = [];
    const done = await postDay(w.rt, 2, { ...CFG, groups: [["en"]], send: tg.send, now: w.rt.now, sleep: async (ms) => void waits.push(ms), log: (l) => lines.push(l) });
    expect(waits).toEqual([10_000, 120_000]);
    expect(done).toEqual([{ seq: 2, key: "en", state: "sent", tries: 3, at: AFTER.toISOString() }]);
    expect(tg.calls).toHaveLength(3);
    expect(lines.join("\n")).toContain("try 1 failed (sendPhoto: HTTP 500 fake failure)");
    expect(lines.join("\n")).not.toContain(TOKEN);
  });

  it("gives up after 4 tries: recorded as failed, logged, and never sent again", async () => {
    const tg = await fakeTelegram();
    const w = world();
    w.clock.t = AFTER;
    w.rt.today();
    tg.fail.push(...Array.from({ length: 10 }, () => ({ status: 502 })));
    const lines: string[] = [];
    const done = await postDay(w.rt, 2, { ...CFG, groups: [["ru"]], send: tg.send, sleep: async () => {}, log: (l) => lines.push(l) });
    expect(done.map((p) => [p.state, p.tries, p.error])).toEqual([["failed", 4, "sendPhoto: HTTP 502 fake failure"]]);
    expect(lines.at(-1)).toContain("failed after 4 tries");
    expect(await postDay(w.rt, 2, { ...CFG, groups: [["ru"]], send: tg.send, ...quiet })).toEqual([]);
    expect(tg.calls).toHaveLength(4);
  });

  it("the day end is unaffected by a failing or hanging Telegram", async () => {
    for (const send of [
      { post: () => Promise.reject(new Error("down")) },
      { post: () => new Promise<void>(() => {}) }, // never answers
      { post: () => { throw new Error("sync throw"); } },
    ] as TelegramSender[]) {
      const w = world();
      const stop = dailyPostJobWith(CFG, send, { ...quiet, everyMs: 5 })(w.rt);
      w.clock.t = AFTER;
      const t0 = Date.now();
      const d = w.rt.today();
      expect(Date.now() - t0).toBeLessThan(2000);
      expect(d.seq).toBe(2);
      expect(w.store.champion(2)).toBeDefined();
      expect(w.store.playoff(1)!.winner).not.toBeNull();
      await new Promise((r) => setTimeout(r, 30));
      stop();
      expect(w.rt.today().seq).toBe(2);
    }
  });

  it("never posts a stale day (turned on hours after the day end) or day 1", async () => {
    const tg = await fakeTelegram();
    const w = world();
    let stop = dailyPostJobWith(CFG, tg.send, { ...quiet, everyMs: 5 })(w.rt);
    await new Promise((r) => setTimeout(r, 30));
    stop();
    w.clock.t = AFTER;
    expect(w.rt.today().seq).toBe(2); // ended with the post off
    w.clock.t = new Date(AFTER.getTime() + POST_WINDOW_MS + 60_000);
    stop = dailyPostJobWith(CFG, tg.send, { ...quiet, everyMs: 5 })(w.rt);
    await new Promise((r) => setTimeout(r, 30));
    stop();
    expect(tg.calls).toEqual([]);
  });

  it("ru+en is one message with both; long unit lists are cut to fit a caption", async () => {
    const tg = await fakeTelegram();
    const w = world();
    w.clock.t = AFTER;
    w.rt.today();
    const seed = w.store.units()[0]!;
    for (let i = 0; i < 40; i++) {
      w.store.putUnit({ unitId: `idea-${i}`, status: "live", row: { ...seed.row, name: `A Rather Long Unit Name ${i}`, emoji: "🦔" }, authorId: lev.id, origin: "idea", parentId: null, createdAt: AFTER.toISOString() });
      w.store.putStint({ unitId: `idea-${i}`, enteredSeq: 2, leftSeq: null, reason: "idea" });
    }
    expect(postGroups("ru+en")).toEqual([["ru", "en"]]);
    expect(postGroups("en")).toEqual([["en"]]);
    expect(postGroups("xx")).toEqual([["ru"], ["en"]]);
    const [m] = dailyMessages(w.rt, 2, postGroups("ru+en"));
    expect(m!.key).toBe("ru+en");
    expect(m!.text.length).toBeLessThanOrEqual(CAPTION_MAX);
    expect(m!.text).toMatch(/…и ещё \d+/);
    expect(m!.text).toMatch(/…and \d+ more/);
    expect(m!.text.indexOf("Чемпион")).toBeLessThan(m!.text.indexOf("Champion of"));
    await postDay(w.rt, 2, { ...CFG, groups: [["ru", "en"]], send: tg.send, ...quiet });
    expect(tg.calls.map((c) => c.method)).toEqual(["sendPhoto"]);
    expect(w.store.dailyPosts(2).map((p) => p.key)).toEqual(["ru+en"]);
  });

  it("with no champion the post is a text, and credits name the evolver", async () => {
    const tg = await fakeTelegram();
    const store = new MemoryMvpStore();
    const clock = { t: DAY1 };
    seedUnits(store, DAY1);
    const rt = mvpRuntime({ content: poolContent(store), store, now: () => clock.t });
    rt.today();
    clock.t = AFTER;
    rt.today();
    store.addPlayer(lev);
    const seed = store.units()[0]!;
    store.putUnit({ unitId: "v2", status: "live", row: { ...seed.row }, authorId: lev.id, origin: "evolution", parentId: seed.unitId, rootId: seed.unitId, createdAt: AFTER.toISOString() });
    store.putStint({ unitId: "v2", enteredSeq: 2, leftSeq: null, reason: "evolution" });
    expect(postFacts(rt, 2).entered.map((u) => [u.name, u.by, u.evolvedBy?.name])).toEqual([[seed.row.name, null, "lev"]]);
    await postDay(rt, 2, { ...CFG, groups: [["en"]], send: tg.send, ...quiet });
    expect(tg.calls.map((c) => c.method)).toEqual(["sendMessage"]);
    expect(tg.calls[0]!.text).toContain("👑 Oct 9, 2026: the throne is empty");
    expect(tg.calls[0]!.text).toContain(`${seed.row.emoji} ${seed.row.name} — evolved by lev`);
  });

  it("on with ARENA_TELEGRAM_POST=1: the job reads its channel, languages, API and token file from the env", async () => {
    const tg = await fakeTelegram();
    const dir = mkdtempSync(join(tmpdir(), "tg-"));
    writeFileSync(join(dir, "telegram.env"), `TELEGRAM_BOT_TOKEN=${TOKEN}\n`);
    const env = { ARENA_TELEGRAM_POST: "1", ARENA_TELEGRAM_CHANNEL: "@arenaofideas", ARENA_TELEGRAM_ENV: join(dir, "telegram.env"), ARENA_TELEGRAM_API: tg.api, ARENA_TELEGRAM_POST_LANGS: "en" };
    const saved = Object.fromEntries(Object.keys(env).map((k) => [k, process.env[k]]));
    Object.assign(process.env, env);
    const w = world();
    try {
      w.clock.t = AFTER;
      const stop = dailyPostJob(w.rt);
      await until(() => w.store.dailyPosts(2).some((p) => p.state === "sent"));
      stop();
    } finally {
      for (const [k, v] of Object.entries(saved)) if (v === undefined) delete process.env[k]; else process.env[k] = v;
    }
    expect(tg.calls.map((c) => [c.method, c.token, c.chat])).toEqual([["sendPhoto", TOKEN, "@arenaofideas"]]);
    expect(tg.calls[0]!.text).toMatch(/^👑 Champion of Oct 9, 2026/);
  });

  it("reads the token from the env file, and no error carries it", async () => {
    const dir = mkdtempSync(join(tmpdir(), "tg-"));
    const file = join(dir, "telegram.env");
    writeFileSync(file, `# the bot\nTELEGRAM_BOT_TOKEN=${TOKEN}\n`);
    expect(readBotToken(file)).toBe(TOKEN);
    writeFileSync(file, "OTHER=1\n");
    expect(() => readBotToken(file)).toThrow(/has no TELEGRAM_BOT_TOKEN/);
    // A dead address: the network error names the method, never the token.
    const send = telegramSender(TOKEN, "http://127.0.0.1:9");
    const err = await send.post("@x", "hi", null).catch((e: Error) => e);
    expect(String((err as Error).message)).toMatch(/^sendMessage: /);
    expect(String((err as Error).message)).not.toContain(TOKEN);
  });
});
