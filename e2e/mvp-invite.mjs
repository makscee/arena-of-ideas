// Slice 13 e2e (mission #574): invite links in Chromium, on a local
// invite-only server (MVP_INVITES=1). Never against the live instance.
//   npm run mvp:invite-e2e -- [--db <a COPY of a world>] [--claim <name>] [--out e2e/.shots/invite]
// It makes a tester's link and an admin link (claiming the human player
// <name> in the copied world, or a new "Maks"), then checks: a tester opens
// the link at 360×640, gets the name, buys and fights; the same link on a
// second device is the same player; the admin sees "End day now" at 1440×900
// and the tester doesn't; another player's link asks before it switches; an
// old device without a token and a bad link both land on a clear screen (a
// bad link on a device with its own session offers Home); a revoked link ends
// every device's session and its new link works. R4-20's open join link
// (#join=): a stranger picks a name and plays, the address becomes their own
// #invite= link and Home offers it for another device; a second stranger
// can't take a used name; a device with a player asks first; a rotated link
// lands on the invite screen. A screenshot of each.
import { spawn, execFileSync } from "node:child_process";
import { copyFileSync, mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const out = opt("out") ?? "e2e/.shots/invite";
mkdirSync(out, { recursive: true });
// A temp world, removed on every exit: with --db it is a copy of a real one.
const tmp = mkdtempSync(join(tmpdir(), "arena-invite-"));
const db = join(tmp, "world.db");
process.on("exit", () => rmSync(tmp, { recursive: true, force: true }));
for (const sig of ["SIGINT", "SIGTERM"]) process.on(sig, () => process.exit(130));
if (opt("db")) copyFileSync(opt("db"), db);
const claim = opt("claim");

const env = { ...process.env, MVP_DB: db, MVP_DEV: "1", MVP_INVITES: "1" };
const cli = (...a) => execFileSync("node", ["--import", "tsx/esm", "server/src/mvp/invite-cli.ts", ...a], { env: { ...env, MVP_PUBLIC_URL: "BASE/" }, encoding: "utf8" }).trim();
const codeOf = (line) => line.slice(line.indexOf("#invite=") + 8);
const TAG = Date.now().toString(36).slice(-4);
const tester = `Tester${TAG}`;
const admin = codeOf(claim ? cli("add", claim, "--claim", "--admin") : cli("add", "Maks", "--admin"));
const guest = codeOf(cli("add", tester));

execFileSync("npm", ["run", "-s", "mvp:build"], { stdio: "inherit" });
const port = await new Promise((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = s.address().port; s.close(() => r(p)); }); });
const child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...env, PORT: String(port) }, stdio: ["ignore", "inherit", "inherit"] });
const url = `http://127.0.0.1:${port}/arena/`;
for (let i = 0; i < 300; i++) { // up to 60 s: m1 and m4 are shared, and under load the server starts slowly
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const browser = await launchChromium();
const errors = [];
let shots = 0;
const PHONE = { viewport: { width: 360, height: 640 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true };
const DESKTOP = { viewport: { width: 1440, height: 900 } };
async function device(opts) {
  const ctx = await browser.newContext(opts);
  const page = await ctx.newPage();
  page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
  return page;
}
const shot = (page, name) => page.screenshot({ path: `${out}/${String(++shots).padStart(2, "0")}-${name}.png` });
const text = (page) => page.evaluate(() => document.body.innerText);
try {
  // A tester's phone: the link gives the name; buy, fight, see the result.
  const a = await device(PHONE);
  await a.goto(`${url}#invite=${guest}`);
  await a.getByTestId("play").waitFor({ timeout: 15_000 });
  await shot(a, "tester-home-phone");
  if (!(await text(a)).includes(tester)) errors.push("tester: home doesn't show the invited name");
  if (a.url().includes("invite=")) errors.push(`tester: the code stays in the address (${a.url()})`);
  if (await a.getByTestId("end-day").count()) errors.push("tester: sees End day now");
  await a.getByTestId("play").click();
  await a.getByTestId("fight").waitFor({ timeout: 10_000 });
  await a.getByTestId("offer-0").click();
  await a.getByTestId("buy").click();
  await a.getByTestId("line").locator(".card.you").first().waitFor();
  await shot(a, "tester-shop-phone");
  await a.getByTestId("fight").click();
  await a.getByTestId("battle-end").waitFor({ timeout: 10_000 });
  await a.getByTestId("battle-end").click();
  await a.getByTestId("end-card").waitFor({ timeout: 10_000 });
  await shot(a, "tester-result-phone");
  // A reload keeps the player (the token is on the device).
  await a.goto(url);
  await a.getByTestId("play").waitFor();
  if (!/Continue/i.test(await a.getByTestId("play").textContent())) errors.push("tester: after a reload Home doesn't offer Continue");

  // The same link on a second device: the same player, the same run.
  const b = await device(PHONE);
  await b.goto(`${url}#invite=${guest}`);
  await b.getByTestId("play").waitFor({ timeout: 15_000 });
  if (!/Continue/i.test(await b.getByTestId("play").textContent())) errors.push("second device: not the same player (no Continue)");
  await shot(b, "tester-second-device");

  // The admin link on a desktop: the claimed player and the dev tools.
  const m = await device(DESKTOP);
  await m.goto(`${url}#invite=${admin}`);
  await m.getByTestId("play").waitFor({ timeout: 15_000 });
  if (!(await text(m)).includes(claim ?? "Maks")) errors.push("admin: home doesn't show the name");
  if (!(await m.getByTestId("end-day").count())) errors.push("admin: no End day now");
  await shot(m, "admin-home-desktop");

  // Another player's link on a device that already has a player asks first.
  await m.goto(`${url}#invite=${guest}`);
  await m.getByTestId("invite-switch").waitFor({ timeout: 10_000 });
  if (!(await m.getByTestId("invite-switch").textContent()).includes(`This link is for ${tester}. Switch from ${claim ?? "Maks"}?`)) errors.push("switch: the question doesn't name both players");
  await shot(m, "switch-ask-desktop");
  await m.getByTestId("invite-stay").click();
  await m.getByTestId("play").waitFor({ timeout: 10_000 });
  if (!(await m.getByTestId("end-day").count())) errors.push("switch: Stay didn't keep the admin");
  if (m.url().includes("invite=")) errors.push("switch: Stay keeps the code in the address");
  await a.goto(`${url}#invite=${admin}`);
  await a.getByTestId("invite-switch").waitFor({ timeout: 10_000 });
  await shot(a, "switch-ask-phone");
  await a.getByTestId("invite-stay").click();
  await a.getByTestId("play").waitFor({ timeout: 10_000 });
  if (!(await text(a)).includes(tester)) errors.push("switch: Stay didn't keep the tester");

  // A bad link on a device with its own session: Home, not a Retry loop.
  await a.goto(`${url}#invite=nope`);
  await a.getByTestId("invite-home").waitFor({ timeout: 10_000 });
  await shot(a, "bad-link-own-session-phone");
  await a.getByTestId("invite-home").click();
  await a.getByTestId("play").waitFor({ timeout: 10_000 });

  // Revoke: both of the tester's devices land on the invite screen, the old
  // link is dead, and the new one gives the same player back.
  const next = codeOf(cli("revoke", tester));
  await b.reload();
  await b.getByTestId("invite-only").waitFor({ timeout: 10_000 });
  await shot(b, "revoked-device-phone");
  await b.goto(`${url}#invite=${guest}`);
  await b.getByTestId("invite-bad").waitFor({ timeout: 10_000 });
  await b.goto(`${url}#invite=${next}`);
  await b.getByTestId("play").waitFor({ timeout: 15_000 });
  if (!/Continue/i.test(await b.getByTestId("play").textContent())) errors.push("revoke: the new link isn't the same player (no Continue)");

  // An old device (a name from before invites, no token) and a bad link.
  const old = await device(DESKTOP);
  await old.addInitScript(() => localStorage.setItem("arena.player", JSON.stringify({ id: "x", name: "old", bot: false })));
  await old.goto(url);
  await old.getByTestId("invite-only").waitFor({ timeout: 10_000 });
  await shot(old, "old-device-desktop");
  const bad = await device(PHONE);
  await bad.goto(`${url}#invite=nope`);
  await bad.getByTestId("invite-bad").waitFor({ timeout: 10_000 });
  if (!(await text(bad)).includes("Ask for a new link")) errors.push("bad link: no session, but it doesn't ask for a new link");
  if (await bad.getByRole("button", { name: "Retry" }).count()) errors.push("bad link: a Retry that reloads the same dead code");
  await shot(bad, "bad-link-phone");
  const fresh = await device(PHONE);
  await fresh.goto(url);
  await fresh.getByTestId("invite-only").waitFor({ timeout: 10_000 });
  await shot(fresh, "no-link-phone");

  // R4-20: the open join link. A stranger picks a name and plays a fight.
  const joinOf = (line) => line.slice(line.indexOf("#join=") + 6);
  const jc = joinOf(cli("open"));
  if (joinOf(cli("open")) !== jc) errors.push("join: open again gave another link");
  const friend = `Friend${TAG}`;
  const j1 = await device(PHONE);
  await j1.goto(`${url}#join=${jc}`);
  await j1.getByTestId("join-name").waitFor({ timeout: 10_000 });
  await shot(j1, "join-name-phone");
  await j1.getByTestId("join-name").fill(friend);
  await j1.getByTestId("join-submit").click();
  await j1.getByTestId("play").waitFor({ timeout: 15_000 });
  if (!(await text(j1)).includes(friend)) errors.push("join: home doesn't show the picked name");
  if (j1.url().includes("join=") || !j1.url().includes("#invite=")) errors.push("join: the address isn't the player's own #invite= link");
  if (await j1.getByTestId("end-day").count()) errors.push("join: a joined player sees End day now");
  const ownUrl = j1.url();
  await j1.getByTestId("own-link-row").locator("summary").click();
  if ((await j1.getByTestId("own-link").inputValue()) !== ownUrl) errors.push("join: Home's own link isn't the address's #invite= link");
  await j1.getByTestId("own-link-copy").scrollIntoViewIfNeeded();
  await shot(j1, "join-home-own-link-phone");
  await j1.getByTestId("play").click();
  await j1.getByTestId("fight").waitFor({ timeout: 10_000 });
  await j1.getByTestId("offer-0").click();
  await j1.getByTestId("buy").click();
  await j1.getByTestId("fight").click();
  await j1.getByTestId("battle-end").waitFor({ timeout: 10_000 });
  await j1.getByTestId("battle-end").click();
  await j1.getByTestId("end-card").waitFor({ timeout: 10_000 });
  await shot(j1, "join-result-phone");
  // The bookmarked address opens the same player on another device.
  const j1b = await device(DESKTOP);
  await j1b.goto(ownUrl);
  await j1b.getByTestId("play").waitFor({ timeout: 15_000 });
  if (!/Continue/i.test(await j1b.getByTestId("play").textContent())) errors.push("join: the own link on another device isn't the same player");

  // A second stranger: a used name is refused, another one plays.
  const j2 = await device(PHONE);
  await j2.goto(`${url}#join=${jc}`);
  await j2.getByTestId("join-name").fill(friend.toLowerCase());
  await j2.getByTestId("join-submit").click();
  await j2.getByTestId("error").filter({ hasText: "taken" }).waitFor({ timeout: 10_000 });
  await shot(j2, "join-taken-phone");
  await j2.getByTestId("join-name").fill(`Other${TAG}`);
  await j2.getByTestId("join-submit").click();
  await j2.getByTestId("play").waitFor({ timeout: 15_000 });
  if (!(await text(j2)).includes(`Other${TAG}`)) errors.push("join: the second stranger isn't their own player");

  // A device that already plays as someone asks first; Stay keeps them.
  await j1.goto(`${url}#join=${jc}`);
  await j1.getByTestId("join-switch").waitFor({ timeout: 10_000 });
  await shot(j1, "join-switch-phone");
  await j1.getByTestId("join-stay").click();
  await j1.getByTestId("play").waitFor({ timeout: 10_000 });
  if (!(await text(j1)).includes(friend)) errors.push("join: Stay didn't keep the player");

  // Rotate: the old link lands on the invite screen, the new one asks for a name.
  const jn = joinOf(cli("open", "--rotate"));
  const j3 = await device(PHONE);
  await j3.goto(`${url}#join=${jc}`);
  await j3.getByTestId("invite-bad").waitFor({ timeout: 10_000 });
  await shot(j3, "join-rotated-phone");
  await j3.goto(`${url}#join=${jn}`);
  await j3.getByTestId("join-name").waitFor({ timeout: 10_000 });
  // The joined players keep playing.
  await j2.reload();
  await j2.getByTestId("play").waitFor({ timeout: 10_000 });
} catch (e) {
  errors.push(`flow: ${e.message.split("\n")[0]}`);
} finally {
  await browser.close();
  child.kill();
}
console.log(`invite e2e: ${shots} screenshots in ${out}`);
if (errors.length) {
  for (const e of errors) console.error(`✗ ${e}`);
  process.exit(1);
}
console.log("✓ invite links work: tester, second device, admin dev tools, switch ask, old device, bad link, revoke, open join link");
