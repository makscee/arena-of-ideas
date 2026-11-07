// Home as a table state (arena#31, mission #26 move 1; the duel's mockup, home-1280x800.png):
// a signed-in player with no run in progress sees one stage like the shop: the HUD plates (the
// title top left; RATING n and the ≡ Esc menu plate top right), the banner "Today's champion
// · @name" with the slayers and the new day, the champion's five-unit line as TO BEAT (real
// pieces in the shop's line frames), YOUR LINE as five empty frames in the shop's bench row and, in the tray, PLAY
// where Fight sits with "12 rounds, then this team" as its second line, and the tray: the Workshop card, the vote (when one is open) and the rows
// Codex, Forge and Champions. At 1920×1080, 1440×900, 1280×720, 1024×768, 800×450 and 390×844:
// nothing clipped or overlapping, no text under 11 px as drawn, no scroll, PLAY on screen, the
// tray's rows 44 px on a phone. Measures boxes. Starts its own server (rules v2's pool).
//   npm run mvp:home -- [--url http://127.0.0.1:8841/arena/] [--out e2e/.shots/home]
import { spawn, execFileSync } from "node:child_process";
import { mkdirSync } from "node:fs";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/home";
mkdirSync(out, { recursive: true });
let url = opt("url");
let child = null;
if (!url) {
  execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
  const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
  child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:", ARENA_CODEX: "1", MVP_POOL: "v2", ARENA_TUNER: "instant" }, stdio: ["ignore", "inherit", "inherit"] });
  url = `http://127.0.0.1:${port}/arena/`;
  for (let i = 0; i < 300; i++) {
    try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
    await new Promise((r) => setTimeout(r, 200));
  }
}

const SIZES = [["1920x1080", 1920, 1080], ["1440x900", 1440, 900], ["1280x720", 1280, 720], ["1024x768", 1024, 768], ["800x450", 800, 450], ["390x844", 390, 844]];
const errors = [];
const fail_ = (where, m) => errors.push(`${where}: ${m}`);
const browser = await launchChromium();
const api = (path, init) => fetch(url + "api/v1" + path, init).then((r) => r.json());
const day = await api("/day");
const champ = (day.champion?.line ?? []).map((u) => u.name);
const rounds = (await api("/home", { headers: {} }).catch(() => ({})))?.rules?.rounds ?? 12;

/** What the page shows now: boxes in window pixels, the stage's scale, and what reads under 11 px as drawn. */
const measure = () => {
  const $ = (id) => document.querySelector(`[data-testid="${id}"]`);
  const stage = $("stage");
  const k = parseFloat(stage?.style.getPropertyValue("--k") || "1");
  const box = (e) => { const r = e?.getBoundingClientRect(); return r && r.width ? { l: r.left, t: r.top, r: r.right, b: r.bottom, w: r.width, h: r.height } : null; };
  /** A piece reaches past its box: the tab above, the name below, the badges at its sides. */
  const reach = (e) => {
    const rs = [e, ...e.querySelectorAll("*")].map((x) => x.getBoundingClientRect()).filter((r) => r.width > 0);
    return { l: Math.min(...rs.map((r) => r.left)), t: Math.min(...rs.map((r) => r.top)), r: Math.max(...rs.map((r) => r.right)), b: Math.max(...rs.map((r) => r.bottom)) };
  };
  const drawn = (e) => {
    const f = parseFloat(getComputedStyle(e).fontSize);
    let b = e;
    while (b && b !== document.body && (getComputedStyle(b).display === "inline" || b.offsetWidth < 24 || b.offsetHeight < 24)) b = b.parentElement;
    const r = b?.getBoundingClientRect();
    const s = b && b.offsetWidth > 0 && r.width > 0 ? r.width / b.offsetWidth : 1;
    return f * (s > 0 ? s : 1);
  };
  const visible = (e) => e.getClientRects().length && getComputedStyle(e).visibility !== "hidden";
  const small = [...document.querySelectorAll("#app *")].filter((e) => visible(e) && /^(SPAN|DIV|BUTTON|P|LABEL|H1|H2|H3|A|SMALL|B|I|INPUT|KBD)$/.test(e.tagName) && !e.closest("[aria-hidden='true']") && (e.tagName === "INPUT" || [...e.childNodes].some((n) => n.nodeType === 3 && n.textContent.trim()))).map((e) => ({ t: (e.textContent || e.placeholder || "").trim().slice(0, 24), px: drawn(e) })).filter((x) => x.px < 10.9);
  const d = document.scrollingElement ?? document.documentElement;
  const root = $("home");
  // Text that is cut: a box that scrolls inside itself, or a child that leaves its card.
  const clipped = root ? [...root.querySelectorAll("*")].filter((e) => visible(e) && !e.closest("svg") && e.scrollWidth > e.clientWidth + 1 && getComputedStyle(e).display !== "inline" && e.clientWidth > 0 && /^(hidden|clip|auto|scroll)$/.test(getComputedStyle(e).overflowX)).map((e) => `${e.dataset.testid || e.className || e.tagName} (${e.scrollWidth}>${e.clientWidth})`) : [];
  const outside = (card, within) => {
    const c = box($(card));
    if (!c) return [];
    return [...$(card).querySelectorAll("*")].filter((e) => visible(e) && !e.closest("svg")).map((e) => ({ e, r: e.getBoundingClientRect() })).filter(({ r }) => r.width > 0 && (r.left < c.l - 1 || r.right > c.r + 1 || r.top < c.t - 1 || r.bottom > c.b + 1)).map(({ e }) => `${e.dataset.testid || e.className || e.tagName} leaves ${card}`);
  };
  const boxes = {};
  for (const id of ["home-title", "home-rating", "settings", "home-banner", "home-side-them", "home-side-you", "play", "home-note", "home-tray", "ideas", "vote-card", "codex", "forge-open", "stats", "new-run"]) boxes[id] = box($(id));
  const champs = [0, 1, 2, 3, 4].map((i) => $(`home-champ-${i}`)).filter(Boolean).map((e) => ({ name: e.getAttribute("data-name"), px: box(e)?.w, reach: reach(e), ...box(e) }));
  const picks = [...document.querySelectorAll('[data-testid="vote-pick"]')].map((e) => reach(e.querySelector(".pc") ?? e));
  const slots = [0, 1, 2, 3, 4].map((i) => box($(`home-slot-${i}`)));
  const controls = [...root?.querySelectorAll("button, input, [role=button]") ?? []].filter(visible).map((e) => ({ id: e.dataset.testid || e.textContent.trim().slice(0, 16), h: e.getBoundingClientRect().height, w: e.getBoundingClientRect().width }));
  return {
    k, stage: box(stage), orient: stage?.getAttribute("data-orient"), W: innerWidth, H: innerHeight,
    scroll: d.scrollHeight - d.clientHeight, hscroll: d.scrollWidth - d.clientWidth,
    home: !!root, inStage: !!(root && stage?.contains(root)),
    boxes, champs, slots, picks, small, clipped, controls,
    leaks: ["ideas", "home-banner", "vote-card", "home-tray"].flatMap((id) => outside(id)),
    titleText: $("home-title")?.textContent?.trim(), ratingText: $("home-rating")?.textContent?.replace(/\s+/g, " ").trim(),
    bannerText: $("home-banner")?.textContent?.replace(/\s+/g, " ").trim(), noteText: $("home-note")?.textContent?.trim(),
    playText: $("play")?.textContent?.replace(/\s+/g, " ").trim(),
    labels: { them: $("home-side-them")?.querySelector(".label")?.textContent?.trim(), you: $("home-side-you")?.querySelector(".label")?.textContent?.trim() },
    rows: { codex: $("codex")?.textContent?.replace(/\s+/g, " ").trim(), forge: $("forge-open")?.textContent?.replace(/\s+/g, " ").trim(), stats: $("stats")?.textContent?.replace(/\s+/g, " ").trim() },
    workshop: $("ideas")?.textContent?.replace(/\s+/g, " ").trim(), vote: !!$("vote-card"),
    old: ["home-actions", "last-run", "idea-meter"].filter((id) => $(id)),
  };
};

const hit = (a, b) => a && b && a.l < b.r - 0.5 && b.l < a.r - 0.5 && a.t < b.b - 0.5 && b.t < a.b - 0.5;

async function signIn(ctx, label, W, H, fail) {
  const page = await ctx.newPage();
  page.on("pageerror", (e) => fail(`pageerror: ${e.message}`));
  page.on("console", (m) => m.type() === "error" && fail(`console: ${m.text()}`));
  await page.goto(url, { timeout: 20_000 });
  await page.getByTestId("name-input").waitFor({ timeout: 15_000 });
  await page.getByTestId("name-input").fill(`HomeTester${label}${Date.now() % 100000}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").waitFor({ timeout: 15_000 });
  await page.waitForTimeout(700);
  return page;
}

function check(label, m, fail, { vote }) {
  const L = m.orient === "L";
  const phone = m.W < 600;
  // One stage.
  if (!m.home || !m.inStage) return fail("Home is not on the table's stage ([data-testid=home] inside [data-testid=stage])");
  if (m.old.length) fail(`the page's old Home is still there: ${m.old}`);
  if (m.scroll > 0 || m.hscroll > 0) fail(`Home scrolls (${m.scroll}px down, ${m.hscroll}px across)`);
  const S = m.stage;
  const rel = (b) => (b ? ((b.l + b.r) / 2 - S.l) / S.w : NaN);
  // 1. The HUD plates.
  if (m.titleText !== "ARENA OF IDEAS") fail(`the title reads ${JSON.stringify(m.titleText)}`);
  const { "home-title": title, "home-rating": rating, settings: menu } = m.boxes;
  if (!title || title.l > S.l + S.w * 0.1 || title.t > S.t + S.h * 0.15) fail(`the title is not at the top left: ${JSON.stringify(title)}`);
  if (!/^RATING\s+\d+/i.test(m.ratingText ?? "")) fail(`the rating plate reads ${JSON.stringify(m.ratingText)}, not "RATING n"`);
  if (!rating || !menu || rating.r > menu.l + 1 || menu.r < S.r - S.w * 0.04 || menu.t > S.t + S.h * 0.15 || rating.t > S.t + S.h * 0.15) fail(`the rating and menu plates are not at the top right: ${JSON.stringify({ rating, menu })}`);
  if (menu && (menu.h < (phone ? 44 : 32) - 0.5 || menu.w < (phone ? 44 : 32) - 0.5)) fail(`the ≡ menu plate is ${menu.w.toFixed(0)}×${menu.h.toFixed(0)}`);
  // 2. The banner.
  const ban = m.bannerText ?? "";
  if (!/Today's champion/.test(ban) || !/new day in \d+ ?h/.test(ban) || !/slayers?/.test(ban)) fail(`the banner reads ${JSON.stringify(ban)}`);
  if (champ.length && day.champion && !ban.includes(`@${day.champion.player.name}`)) fail(`the banner does not name @${day.champion.player.name}: ${JSON.stringify(ban)}`);
  // 3. The champion's line, centred, the table's piece size; your line; PLAY over its middle.
  if (champ.length >= 3) {
    if (m.champs.map((c) => c.name).join() !== champ.join()) fail(`TO BEAT is ${m.champs.map((c) => c.name)}, not today's champion's ${champ}`);
    const k = m.k;
    const px = m.champs[0].w / k;
    if (px < (L ? 70 : 34) - 0.5) fail(`the champion's pieces are ${px.toFixed(0)} px on the stage, under ${L ? "the shop's 70" : 34}`);
    // The shop's places (arena#51): each piece stands in a frame of the line row, over the same column as your slot below it.
    m.champs.forEach((c, i) => {
      const s = m.slots[i];
      if (s && Math.abs((c.reach.l + c.reach.r) / 2 - (s.l + s.r) / 2) > m.k * 14) fail(`champion piece ${i} is not over your slot ${i + 1}`);
    });
    for (let i = 1; i < m.champs.length; i++) if (hit(m.champs[i - 1].reach, m.champs[i].reach)) fail(`champion pieces ${i - 1} and ${i} overlap (${JSON.stringify(m.champs[i - 1].reach)} / ${JSON.stringify(m.champs[i].reach)})`);
  } else fail(`the server has no champion line to show (${champ.length} units)`);
  if (m.labels.them !== "TO BEAT" && m.labels.them?.toUpperCase() !== "TO BEAT") fail(`the champion's line is labelled ${JSON.stringify(m.labels.them)}`);
  if (m.labels.you?.toUpperCase() !== "YOUR LINE") fail(`your line is labelled ${JSON.stringify(m.labels.you)}`);
  if (m.slots.some((s) => !s)) fail("YOUR LINE does not have five slots");
  else {
    const sw = m.slots[0].w / m.k;
    if (sw < (L ? 70 : 48) - 0.5) fail(`the slots are ${sw.toFixed(0)} px, under the piece's`);
    // PLAY sits where Fight sits: in the tray's right column at the foot, never over the slots.
    const play = m.boxes.play;
    const tr = m.boxes["home-tray"];
    if (!play || !tr || play.r > tr.r + 1 || play.b > tr.b + 1 || play.l < tr.l + (tr.r - tr.l) * (L ? 0.7 : 0.2) || play.t < tr.t + (tr.b - tr.t) * 0.2) fail(`PLAY is not at Fight's place in the tray: ${JSON.stringify({ play, tr })}`);
    if (play && hit(play, m.slots[2])) fail("PLAY is over your line's slots");
    for (let i = 1; i < 5; i++) if (hit(m.slots[i - 1], m.slots[i])) fail(`slots ${i - 1} and ${i} overlap`);
    if (Math.min(...m.champs.map((c) => c.reach.b)) > m.slots[0].t + 1 && m.champs.length) fail("the champion's line is not above your line");
  }
  const play = m.boxes.play;
  if (!play || play.t < 0 || play.b > m.H + 0.5 || play.l < 0 || play.r > m.W + 0.5) fail(`PLAY is not on screen: ${JSON.stringify(play)}`);
  else if (play.h < 48 - 0.5) fail(`PLAY is only ${play.h.toFixed(0)} px tall`);
  if (!/^PLAY|Play/i.test(m.playText ?? "")) fail(`PLAY reads ${JSON.stringify(m.playText)}`);
  if (m.noteText !== `${rounds} rounds, then this team`) fail(`the note under PLAY reads ${JSON.stringify(m.noteText)}, not "${rounds} rounds, then this team"`);
  // The note is PLAY's second line (the phone's PLAY has no room for it).
  if (L && m.boxes["home-note"] && play && (m.boxes["home-note"].t < play.t - 1 || m.boxes["home-note"].b > play.b + 1)) fail("the note is not inside PLAY");
  // 4. The tray.
  const tray = m.boxes["home-tray"];
  if (!tray || tray.b > S.b + 1 || tray.l < S.l - 1 || tray.r > S.r + 1) fail(`the tray is not inside the stage: ${JSON.stringify(tray)}`);
  if (!m.workshop || !/Workshop/.test(m.workshop)) fail(`the Workshop card reads ${JSON.stringify(m.workshop)}`);
  else if (!/idea/i.test(m.workshop)) fail(`the Workshop card says nothing of the player's ideas: ${JSON.stringify(m.workshop)}`);
  if (vote !== m.vote) fail(vote ? "a vote is open but the tray shows none" : "the tray shows a vote when none is open");
  if (vote && !/Which would you rather see\?/.test(m.vote ? "Which would you rather see?" : "")) fail("no vote question");
  if (!/Codex/.test(m.rows.codex ?? "") || (!phone && !/\bC\b/.test(m.rows.codex ?? "")) || !/\d+ units/.test(m.rows.codex ?? "")) fail(`the Codex row reads ${JSON.stringify(m.rows.codex)}`);
  if (!/Forge/.test(m.rows.forge ?? "") || (!phone && !/\bF\b/.test(m.rows.forge ?? ""))) fail(`the Forge row reads ${JSON.stringify(m.rows.forge)}`);
  if (!/Champions/.test(m.rows.stats ?? "") || (!phone && !/\bS\b/.test(m.rows.stats ?? ""))) fail(`the Champions row reads ${JSON.stringify(m.rows.stats)}`);
  for (const id of ["codex", "forge-open", "stats"]) {
    const b = m.boxes[id];
    if (!b) fail(`the ${id} row is missing`);
    else if (b.h < (phone ? 44 : 32) - 0.5) fail(`the ${id} row is ${b.h.toFixed(0)} px tall (${phone ? 44 : 32} wanted)`);
  }
  if (phone) {
    // The tray stacks: the Workshop card, the vote, the three rows side by side, then PLAY.
    const col = ["ideas", ...(vote ? ["vote-card"] : []), "codex", "play"].map((id) => m.boxes[id]);
    if (col.some((b) => !b)) fail("the phone's tray lacks a part");
    else for (let i = 1; i < col.length; i++) if (col[i].t < col[i - 1].b - 1) fail(`the phone's tray does not stack (part ${i} starts at ${col[i].t.toFixed(0)}, the one above ends ${col[i - 1].b.toFixed(0)})`);
    const rows = ["codex", "forge-open", "stats"].map((id) => m.boxes[id]);
    if (rows.every(Boolean) && (rows[1].l < rows[0].r - 1 || rows[2].l < rows[1].r - 1)) fail("the phone's three rows are not side by side");
  }
  // Nothing clipped or overlapping.
  const named = { title: "home-title", rating: "home-rating", menu: "settings", banner: "home-banner", tray: "home-tray" };
  const parts = Object.entries(named).map(([n, id]) => [n, m.boxes[id]]).filter(([, b]) => b);
  for (let i = 0; i < parts.length; i++) for (let j = i + 1; j < parts.length; j++) if (hit(parts[i][1], parts[j][1])) fail(`${parts[i][0]} and ${parts[j][0]} overlap`);
  for (const c of m.champs) {
    for (const [n, b] of parts) if (hit(c.reach, b)) fail(`champion piece ${c.name} overlaps the ${n}`);
  }
  if (vote && (m.picks.length !== 2 || hit(m.picks[0], m.picks[1]))) fail(`the vote's two candidates ${m.picks.length === 2 ? "overlap" : "are not two"}`);
  // New run is in the tray, where Reroll is (above PLAY, or beside it on a phone).
  const nr = m.boxes["new-run"], tr2 = m.boxes["home-tray"];
  if (nr && tr2 && (nr.l < tr2.l - 1 || nr.r > tr2.r + 1 || nr.t < tr2.t - 1 || nr.b > tr2.b + 1)) fail("New run leaves the tray");
  for (const [n, b] of [["title", title], ["banner", m.boxes["home-banner"]], ["tray", tray], ["menu", menu]]) if (b && (b.l < S.l - 1 || b.r > S.r + 1 || b.t < S.t - 1 || b.b > S.b + 1)) fail(`${n} leaves the stage`);
  if (m.clipped.length) fail(`clipped text: ${m.clipped.join(", ")}`);
  if (m.leaks.length) fail(`a card's contents leave it: ${m.leaks.join(", ")}`);
  if (m.small.length) fail(`text under 11 px as drawn: ${m.small.map((s) => `${s.t} (${s.px.toFixed(1)})`).join(", ")}`);
  for (const c of m.controls) if (c.h < (phone ? 44 : 32) - 0.5 || c.w < (phone ? 44 : 32) - 0.5) fail(`control ${c.id} is ${c.w.toFixed(0)}×${c.h.toFixed(0)} (${phone ? 44 : 32} wanted)`);
}

try {
  // A. No vote open: the table without it. B. A vote open (another player's candidate): the full tray.
  for (const phase of ["plain", "vote"]) {
    if (phase === "vote") {
      const other = await api("/players", { method: "POST", headers: { "content-type": "application/json" }, body: JSON.stringify({ name: `Writer${Date.now() % 100000}` }) });
      const as = { "X-Arena-Player": other.id, "content-type": "application/json" };
      await api("/dev/seed-candidate", { method: "POST", headers: as });
      await api("/dev/overnight-check", { method: "POST", headers: as });
      for (let i = 0; i < 40; i++) {
        if ((await api("/ideas", { headers: as })).sent?.some((x) => x.state === "voting")) break;
        await new Promise((r) => setTimeout(r, 500));
      }
    }
    for (const [name, W, H] of SIZES) {
      const label = `${phase} ${name}`;
      const ctx = await browser.newContext({ viewport: { width: W, height: H }, hasTouch: W < 600, isMobile: W < 600 });
      const fail = (m) => fail_(label, m);
      const page = await signIn(ctx, name.replace("x", ""), W, H, fail);
      if (phase === "vote") await page.getByTestId("vote-card").waitFor({ timeout: 8_000 }).catch(() => {});
      await page.waitForTimeout(300);
      await page.screenshot({ path: `${out}/home-${phase === "vote" ? "" : "novote-"}${name}.png` });
      check(label, await page.evaluate(measure), fail, { vote: phase === "vote" });
      await ctx.close();
    }
  }

  // The menu plate opens the ≡ menu window (Esc too), with everything Home used to carry.
  {
    const ctx = await browser.newContext({ viewport: { width: 1280, height: 720 } });
    const page = await signIn(ctx, "menu", 1280, 720, (m) => fail_("menu", m));
    await page.keyboard.press("Escape");
    await page.getByTestId("settings-sheet").waitFor({ timeout: 4_000 }).catch(() => fail_("menu", "Esc does not open the ≡ menu"));
    for (const id of ["sound-toggle", "lang-row", "dictionary-open", "rules-open", "end-day"]) if (!(await page.getByTestId("settings-sheet").getByTestId(id).count())) fail_("menu", `the ≡ menu lacks ${id}`);
    await page.screenshot({ path: `${out}/menu-1280x720.png` });
    await page.keyboard.press("Escape");
    await page.getByTestId("settings-sheet").waitFor({ state: "detached", timeout: 4_000 }).catch(() => fail_("menu", "Esc does not close the ≡ menu"));
    // The plate opens it too.
    await page.getByTestId("settings").click();
    await page.getByTestId("settings-sheet").waitFor({ timeout: 4_000 }).catch(() => fail_("menu", "the ≡ plate does not open the menu"));
    await page.getByTestId("sheet-close").click();
    // Keys: C, F, S open the Codex, the Forge and Champions; Enter plays.
    await page.keyboard.press("c");
    await page.getByTestId("codex-units").waitFor({ timeout: 8_000 }).catch(() => fail_("keys", "C does not open the Codex"));
    await page.screenshot({ path: `${out}/codex-1280x720.png` });
    await page.getByTestId("codex-back").click();
    await page.getByTestId("home").waitFor({ timeout: 4_000 }).catch(() => fail_("keys", "Back from the Codex does not return Home"));
    // The Workshop card opens the player's ideas.
    await page.getByTestId("ideas").click();
    await page.getByTestId("ideas-held").waitFor({ timeout: 8_000 }).catch(() => fail_("workshop", "the Workshop card does not open the ideas"));
    await page.goBack().catch(() => {});
    await page.getByTestId("home").waitFor({ timeout: 8_000 }).catch(async () => { await page.getByTestId("ideas-back").click().catch(() => {}); });
    // Enter plays: round 1 of the shop; the run then waits behind Home's RESUME.
    await page.getByTestId("home").waitFor({ timeout: 8_000 });
    await page.keyboard.press("Enter");
    await page.getByTestId("hud").waitFor({ timeout: 8_000 }).catch(() => fail_("keys", "Enter on Home does not open round 1"));
    await ctx.close();
  }

  // A run in progress: the same button reads RESUME, New run is where Reroll is.
  for (const [name, W, H] of [SIZES[2], SIZES[4], SIZES[5]]) {
    const ctx = await browser.newContext({ viewport: { width: W, height: H }, hasTouch: W < 600, isMobile: W < 600 });
    const label = `resume ${name}`;
    const fail = (m) => fail_(label, m);
    const page = await signIn(ctx, `r${name.replace("x", "")}`, W, H, fail);
    await page.getByTestId("play").click();
    await page.getByTestId("hud").waitFor({ timeout: 10_000 });
    await page.getByTestId("menu-open").click();
    await page.getByTestId("menu-title").click();
    await page.getByTestId("home").waitFor({ timeout: 10_000 });
    await page.waitForTimeout(500);
    const m = await page.evaluate(measure);
    if (!/resume/i.test(m.playText ?? "")) fail(`the button reads ${JSON.stringify(m.playText)}, not RESUME`);
    if (!m.boxes["new-run"]) fail("New run is gone while a run waits");
    else if (m.boxes.play && !(m.boxes["new-run"].b <= m.boxes.play.t + 1 || m.boxes["new-run"].r <= m.boxes.play.l + 1)) fail("New run is not above or beside RESUME, in Reroll's place");
    check(label, m, (x) => (/the note under PLAY|the note is not under PLAY|PLAY reads/.test(x) ? undefined : fail(x)), { vote: m.vote });
    await page.screenshot({ path: `${out}/resume-${name}.png` });
    await ctx.close();
  }
} catch (e) {
  errors.push(`${e instanceof Error ? e.stack : e}`);
} finally {
  await browser.close();
  child?.kill();
}
if (errors.length) {
  console.error(errors.map((e) => `FAIL ${e}`).join("\n"));
  process.exit(1);
}
console.log("mvp home: ok");
