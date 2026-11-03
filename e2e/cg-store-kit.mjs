// CrazyGames store kit (#1220, mission #1214): covers and videos from the real
// game on a local scenes server, never live.
//   covers  cover-1920x1080.png  cover-800x1200.png  cover-800x800.png
//   videos  video-16x9.mp4 (1920×1080)  video-2x3.mp4 (1080×1620), silent, ~18 s
// A cover is the shop scene's own unit cards (art and icons only: the title is
// its one text) on the Night Plum ground. Each video opens on its cover, which
// fades into the same shop: buy an offer, Fight, the battle plays (speed 2).
// Frames come from Chromium's screencast (headless: no cursor), ffmpeg makes the mp4.
// Emoji render in Noto Color Emoji (SIL OFL 1.1, fetched pinned into e2e/.cache, for
// the render only: the game zip keeps the system emoji). The system's emoji art
// (Apple's on a Mac) is not ours to put in store media: a glyph drawn in any
// other emoji font fails the run.
//   AOI_E2E_BROWSER_PATH=<playwright headless shell> npm run mvp:cg-store-kit -- [--url http://127.0.0.1:PORT/arena/] [--out DIR] [--seconds 18] [--only covers|videos]
import { spawn, execFileSync } from "node:child_process";
import { createHash } from "node:crypto";
import { existsSync, mkdirSync, readFileSync, renameSync, rmSync, writeFileSync } from "node:fs";
import { createServer } from "node:net";
import { join, resolve } from "node:path";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/cg-store-kit";
const seconds = Number(opt("seconds") ?? 18);
const only = opt("only");
const scene = opt("scene") ?? "shop-r3";
mkdirSync(out, { recursive: true });

let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", ARENA_SCENES: "1", MVP_DB: ":memory:", ARENA_TUNER: "instant", ARENA_CODEX: "1" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
}
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const EMOJI_FONT = {
  url: "https://raw.githubusercontent.com/googlefonts/noto-emoji/v2.051/fonts/Noto-COLRv1.ttf",
  sha256: "0ae57fe58645638523ba35f388d93739d292539a9acb84df5700c81b1e1a28d2",
  file: "e2e/.cache/Noto-COLRv1-v2.051.ttf",
};
const sha = (b) => createHash("sha256").update(b).digest("hex");
if (!existsSync(EMOJI_FONT.file) || sha(readFileSync(EMOJI_FONT.file)) !== EMOJI_FONT.sha256) {
  mkdirSync("e2e/.cache", { recursive: true });
  const res = await fetch(EMOJI_FONT.url);
  const buf = Buffer.from(await res.arrayBuffer());
  if (!res.ok || sha(buf) !== EMOJI_FONT.sha256) throw new Error(`Noto Color Emoji: ${res.status} from ${EMOJI_FONT.url}, or its sha256 changed`);
  writeFileSync(EMOJI_FONT.file, buf);
}
const emojiFont = readFileSync(EMOJI_FONT.file);
const EMOJI_PATH = "__store-kit/noto-color-emoji.ttf";
// "Store Emoji" sits in every stack before the generic family, so no emoji falls through to the system's.
const EMOJI_CSS = `@font-face { font-family: "Store Emoji"; src: url("${EMOJI_PATH}") format("truetype"); }
:root:root, :root:root:lang(ru) {
  --font-head: "Lilita One", "Store Emoji", sans-serif;
  --font-body: "Rubik", "Store Emoji", sans-serif;
  --font-num: "Rubik", "Store Emoji", sans-serif;
}
:root:root:lang(ru) { --font-head: "Rubik", "Store Emoji", sans-serif; }
body, .emoji, .emoji * { font-family: var(--font-body); }
.emoji, .emoji * { font-family: "Store Emoji", sans-serif !important; }
.tier, button.chip.tier-chip { font-family: Georgia, "Times New Roman", "Store Emoji", serif; }`;
const useNotoEmoji = async (page) => {
  await page.addStyleTag({ content: EMOJI_CSS });
  await page.evaluate(async () => { await document.fonts.load('40px "Store Emoji"', "🤡"); await document.fonts.ready; });
};
/** Emoji drawn in a system emoji font, not our Noto: [{ text, fonts }] for every element on screen. */
const foreignEmoji = async (cdp) => {
  await cdp.send("DOM.enable");
  await cdp.send("CSS.enable");
  const { root } = await cdp.send("DOM.getDocument", { depth: -1 });
  const bad = [];
  const walk = async (n) => {
    const text = (n.children ?? []).filter((c) => c.nodeType === 3).map((c) => c.nodeValue).join("");
    if (n.nodeType === 1 && /\p{Extended_Pictographic}/u.test(text)) {
      const { fonts } = await cdp.send("CSS.getPlatformFontsForNode", { nodeId: n.nodeId }).catch(() => ({ fonts: [] }));
      const other = fonts.filter((f) => !f.isCustomFont && /emoji/i.test(f.familyName));
      if (other.length) bad.push({ text: text.trim().slice(0, 20), fonts: other.map((f) => f.familyName).join(", ") });
    }
    for (const c of n.children ?? []) await walk(c);
  };
  await walk(root);
  return bad;
};
const checkEmoji = async (cdp, where) => {
  const bad = await foreignEmoji(cdp);
  if (bad.length) errors.push(`${where}: emoji drawn in ${[...new Set(bad.map((b) => b.fonts))].join("; ")}, not Noto: ${bad.slice(0, 5).map((b) => b.text).join(" ")}`);
  else console.log(`${where}: every emoji in Noto Color Emoji`);
};

/** The cover: the game's own unit cards (icons and art, no names or numbers) on
 * the Night Plum ground, your line facing theirs, and the title above: the
 * only text a cover may carry. Runs on a shop scene's page. */
const COVER = () => {
  const W = innerWidth, H = innerHeight, wide = W / H > 1.2, tall = H / W > 1.2;
  const cards = [...document.querySelectorAll(".card")].filter((c) => c.querySelector(".emoji")).map((c) => c.cloneNode(true));
  // One card per art, none whose art carries lettering (the coin is stamped with words).
  const seen = new Set(["🪙"]);
  cards.splice(0, cards.length, ...cards.filter((c) => { const e = c.querySelector(".emoji").textContent.trim(); return !seen.has(e) && seen.add(e); }));
  for (const c of cards) {
    c.querySelectorAll(".name,.stats,.foot,.cost,.tier,.roman,.dots,.awoken,.badge,b,small").forEach((x) => x.remove());
    for (const n of [...c.querySelectorAll("*")]) if (n.children.length === 0 && !n.closest(".emoji") && !n.closest("svg") && n.textContent.trim()) n.remove();
    c.removeAttribute("data-testid");
  }
  const stage = document.createElement("div");
  stage.id = "__cover";
  stage.style.cssText = "position:fixed;inset:0;z-index:99999;overflow:hidden;background:var(--ground);background-image:radial-gradient(var(--dots) 1.6px,transparent 1.7px);background-size:22px 22px;";
  const per = wide ? 5 : tall ? 3 : 4;
  const cw = Math.min(W / (per + 0.9), (H * (tall ? 0.23 : wide ? 0.25 : 0.27)) / 1.3);
  const row = (list, y, side) => {
    const r = document.createElement("div");
    r.style.cssText = `position:absolute;left:0;right:0;top:${y}px;display:flex;justify-content:center;gap:${cw * 0.12}px;`;
    list.forEach((c, i) => {
      const k = cw / 96;
      const wrap = document.createElement("div");
      const tilt = (i - (list.length - 1) / 2) * (side === "you" ? 4 : -4);
      wrap.style.cssText = `width:${cw}px;height:${cw * 1.3}px;transform:rotate(${tilt}deg) translateY(${Math.abs(i - (list.length - 1) / 2) * cw * 0.06}px);`;
      c.style.cssText = `position:relative;width:96px;height:125px;transform:scale(${k});transform-origin:0 0;margin:0;box-sizing:border-box;display:flex;flex-direction:column;align-items:center;justify-content:center;padding:14px 4px 10px;` +
        (side === "them" ? "border-color:var(--them);box-shadow:0 5px 0 var(--them-deep);" : "border-color:var(--you);");
      c.classList.remove("ghost", "acting");
      c.classList.add(side === "you" ? "you" : "them");
      const e = c.querySelector(".emoji");
      if (e) e.style.cssText = "font-size:54px;line-height:1;margin:auto 0;position:static;transform:none;";
      const ic = c.querySelector(".icons");
      if (ic) ic.style.cssText = "position:absolute;top:6px;left:6px;";
      wrap.append(c);
      r.append(wrap);
    });
    stage.append(r);
  };
  const titleH = H * (tall ? 0.24 : wide ? 0.36 : 0.3);
  const gap = (H - titleH - 2 * cw * 1.3) / 3;
  row(cards.slice(0, per), titleH + gap * 0.8, "them");
  row(cards.slice(per, per * 2), titleH + gap * 1.6 + cw * 1.3, "you");
  const t = document.createElement("div");
  const size = Math.min(W * (wide ? 0.105 : 0.14), titleH * 0.5);
  t.textContent = "Arena of Ideas";
  t.style.cssText = `position:absolute;left:0;right:0;top:${(titleH - size) / 2 + size * 0.05}px;text-align:center;font:${size}px/1 'Lilita One',sans-serif;color:var(--action);letter-spacing:.01em;white-space:nowrap;` +
    `-webkit-text-stroke:${size * 0.12}px var(--outline);paint-order:stroke fill;text-shadow:0 ${size * 0.09}px 0 var(--outline)`;
  stage.append(t);
  document.body.append(stage);
};

const SHAPES = [
  { name: "16x9", viewport: { width: 960, height: 540 }, cover: "cover-1920x1080.png", coverScale: 2, video: "video-16x9.mp4", videoScale: 2 },
  { name: "2x3", viewport: { width: 540, height: 810 }, cover: "cover-800x1200.png", coverScale: 800 / 540, video: "video-2x3.mp4", videoScale: 2 },
  { name: "1x1", viewport: { width: 540, height: 540 }, cover: "cover-800x800.png", coverScale: 800 / 540 },
];

// The screencast needs Playwright's headless shell: a system Chrome's window is
// shorter than the viewport and crops every frame (set AOI_E2E_BROWSER_PATH).
const browser = await launchChromium();
const errors = [];
try {
  for (const s of SHAPES) {
    // One page per shape: the cover is shot on it, and its video starts from that same frame.
    const video = s.video && only !== "covers";
    if (!video && only === "videos") continue;
    const ctx = await browser.newContext({ viewport: s.viewport, deviceScaleFactor: video ? s.videoScale : s.coverScale });
    await ctx.route(`**/${EMOJI_PATH}`, (r) => r.fulfill({ body: emojiFont, contentType: "font/ttf" }));
    const page = await ctx.newPage();
    const cdp = await ctx.newCDPSession(page);
    await page.goto(`${url}#scene=${scene}&lang=en&theme=plum&speed=2`);
    await page.getByTestId("fight").waitFor({ timeout: 30_000 });
    await useNotoEmoji(page);
    await page.waitForTimeout(800);
    await page.evaluate(COVER);
    await page.waitForTimeout(300);
    await checkEmoji(cdp, `${s.name} cover`);
    if (only !== "videos") {
      const shot = join(out, s.cover);
      await page.screenshot({ path: shot });
      if (video) execFileSync("ffmpeg", ["-y", "-loglevel", "error", "-i", shot, "-q:v", "1", join(out, `first-${s.name}.jpg`)]);
      // The video's size differs from the cover's (2:3 is 1080×1620 vs 800×1200): scale the same frame down.
      if (video && s.videoScale !== s.coverScale) {
        const [cw, ch] = s.cover.match(/(\d+)x(\d+)/).slice(1);
        execFileSync("ffmpeg", ["-y", "-loglevel", "error", "-i", shot, "-vf", `scale=${cw}:${ch}:flags=lanczos`, shot + ".tmp.png"]);
        renameSync(shot + ".tmp.png", shot);
      }
    }
    if (!video) { await ctx.close(); continue; }

    const frames = join(out, `frames-${s.name}`);
    rmSync(frames, { recursive: true, force: true });
    mkdirSync(frames, { recursive: true });
    const got = [];
    cdp.on("Page.screencastFrame", (f) => {
      const file = join(frames, `${String(got.length).padStart(5, "0")}.jpg`);
      writeFileSync(file, Buffer.from(f.data, "base64"));
      got.push({ file, t: f.metadata.timestamp });
      if (got.length === 1 && Math.round(f.metadata.deviceHeight) !== s.viewport.height)
        errors.push(`${s.name}: screencast frames are ${f.metadata.deviceWidth}×${f.metadata.deviceHeight}, not the viewport: run with Playwright's headless shell (AOI_E2E_BROWSER_PATH)`);
      cdp.send("Page.screencastFrameAck", { sessionId: f.sessionId }).catch(() => {});
    });
    const W = s.viewport.width * s.videoScale, H = s.viewport.height * s.videoScale;
    await cdp.send("Page.startScreencast", { format: "jpeg", quality: 92, maxWidth: W, maxHeight: H, everyNthFrame: 1 });
    const t0 = Date.now();
    const at = (ms) => page.waitForTimeout(Math.max(0, ms - (Date.now() - t0)));
    // A heartbeat so a still screen still yields frames at its true length.
    const beat = setInterval(() => page.evaluate(() => document.body.style.setProperty("--beat", String(Math.random()))).catch(() => {}), 100);
    await at(1500);
    const fadeAt = Date.now() / 1000;
    await page.evaluate(() => { const c = document.getElementById("__cover"); if (c) { c.style.transition = "opacity .6s"; c.style.opacity = "0"; } });
    await at(2200);
    await page.evaluate(() => document.getElementById("__cover")?.remove());
    try {
      await page.getByTestId("offer-0").click(); await page.getByTestId("offer-0").click();
      await page.getByTestId("buy").waitFor({ timeout: 5_000 });
      await at(3300);
      await page.getByTestId("buy").click();
      await at(4600);
      // The scene's offers are random: a buy that awakens a unit brings a gift, and Fight waits for it. Skip it.
      if (await page.getByTestId("fight").isDisabled()) {
        if (await page.getByTestId("gift-open").isVisible()) await page.getByTestId("gift-open").click();
        await page.getByTestId("gift-skip").click({ timeout: 5_000 });
        await page.getByTestId("gift-title").waitFor({ state: "hidden", timeout: 5_000 }).catch(() => {});
      }
      await page.getByTestId("fight").click();
      await page.getByTestId("battle-you").waitFor({ timeout: 10_000 });
      await page.waitForTimeout(1500);
      await checkEmoji(cdp, `${s.name} battle`);
    } catch (e) {
      errors.push(`${s.name}: the scripted run broke: ${String(e.message).split("\n").slice(0, 12).join("\n")}`);
    }
    await at(seconds * 1000 + 300);
    clearInterval(beat);
    await cdp.send("Page.stopScreencast");
    await ctx.close();
    // A still page sends no screencast frame until something paints (the cover fades at 1.5 s):
    // the video starts when recording did, on the cover, whenever the first frame came.
    const start = t0 / 1000;
    if (!got.length || got[0].t > start) got.unshift({ file: got[0]?.file ?? "", t: start });
    // Concat list: each frame lasts until the next one; the last fills to `seconds`.

    let list = "";
    for (let i = 0; i < got.length; i++) {
      const end = i + 1 < got.length ? got[i + 1].t : start + seconds;
      if (got[i].t >= start + seconds) break;
      const d = Math.min(end, start + seconds) - got[i].t;
      if (d <= 0) continue; // two frames with one timestamp: the first never shows
      // Until the cover fades, the frame is the cover itself, at full resolution (the screencast's are 1×).
      const file = got[i].t < fadeAt && existsSync(join(out, `first-${s.name}.jpg`)) ? resolve(out, `first-${s.name}.jpg`) : got[i].file.split("/").pop();
      list += `file '${file}'\nduration ${d.toFixed(4)}\n`;
    }
    list += `file '${got[Math.max(0, got.length - 1)].file.split("/").pop()}'\n`;
    writeFileSync(join(frames, "list.txt"), list);
    // The cover jpg is full size, the screencast frames 1×: without -reinit_filter 0 the size change
    // rebuilds the filter graph and drops the cover's hold (the video came out 1.5 s short).
    execFileSync("ffmpeg", ["-y", "-loglevel", "error", "-reinit_filter", "0", "-f", "concat", "-safe", "0", "-i", join(frames, "list.txt"),
      "-vf", `fps=30,scale=${W}:${H}:flags=lanczos,format=yuv420p`, "-t", String(seconds), "-an", "-c:v", "libx264", "-preset", "slow", "-crf", "20",
      "-movflags", "+faststart", join(out, s.video)]);
    if (!process.env.KEEP_FRAMES) rmSync(frames, { recursive: true, force: true });
    rmSync(join(out, `first-${s.name}.jpg`), { force: true });
    console.log(`${s.video}: ${got.length} frames`);
  }
} finally {
  await browser.close();
  child?.kill();
}
console.log(errors.length ? `cg-store-kit FAILED\n${errors.join("\n")}` : `cg-store-kit ok (${out})`);
process.exit(errors.length ? 1 : 0);
