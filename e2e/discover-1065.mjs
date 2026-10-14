// The discovery checkpoints of makscee/void-board#1065 on keys only: each
// `#invite=…&checkpoint=a+b` link opens the shop with Awoken a and b on the
// line (both marked for a fuse); on one pair the walk fuses by keys (Tab to
// the first, Enter, F, Tab to the second, Enter), picks a sealed card (2),
// claims (Enter), and the Codex then shows the pair discovered by this player.
// A pointer click anywhere fails the run, as in mvp-keyboard.mjs.
//   node e2e/discover-1065.mjs --url http://127.0.0.1:PORT/arena/ --invite CODE --pairs a+b,c+d --discover a+b [--out dir]
import { mkdirSync } from "node:fs";
import { launchChromium } from "./browser.mjs";

const arg = (n, d) => {
  const i = process.argv.indexOf(`--${n}`);
  return i > 0 ? process.argv[i + 1] : d;
};
const url = arg("url");
const invite = arg("invite");
const pairs = arg("pairs", "").split(",").filter(Boolean);
const discover = arg("discover");
const out = arg("out", "e2e/.shots/discover-1065");
mkdirSync(out, { recursive: true });
const errors = [];
const browser = await launchChromium();
const ctx = await browser.newContext({ viewport: { width: 1280, height: 800 } });
await ctx.addInitScript(() => {
  window.__mouse = 0;
  for (const type of ["click", "pointerdown", "mousedown"]) addEventListener(type, (e) => { if (e.isTrusted && (type !== "click" || e.detail > 0)) window.__mouse++; }, true);
});
const page = await ctx.newPage();
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
const tid = (id) => page.getByTestId(id);
const press = (k) => page.keyboard.press(k);
const active = () => page.evaluate(() => document.activeElement?.dataset?.testid ?? document.activeElement?.tagName ?? "");
const settle = async () => {
  for (let i = 0; i < 2; i++) {
    await page.evaluate(() => new Promise((r) => requestAnimationFrame(() => requestAnimationFrame(r))));
    await page.waitForFunction(() => !document.getElementById("app").classList.contains("busy"), null, { timeout: 30_000 });
  }
};
const tabTo = async (id) => {
  for (let i = 0; i < 70; i++) {
    if ((await active()) === id) return true;
    await press("Tab");
  }
  errors.push(`Tab never reached ${id}`);
  return false;
};
const open = async (pair) => {
  // a fresh document each time: a hash change on the open page reloads it (the app's hashchange) under the keys
  await page.goto("about:blank");
  await page.goto(`${url}#invite=${invite}&checkpoint=${pair}`);
  await tid("line").waitFor({ timeout: 20_000 });
  await page.waitForFunction(() => document.querySelectorAll('[data-testid^="fuse-mark-"]').length >= 2, null, { timeout: 15_000 }).catch(() => {});
  await settle();
};
for (const pair of pairs) {
  await open(pair);
  const n = await tid("line").locator(".card.you").count();
  const awoken = await page.locator('[data-testid^="line-"].awoken').count();
  const marks = await page.locator('[data-testid^="fuse-mark-"]').count();
  const ok = n === 2 && awoken === 2 && marks >= 2;
  console.log(`${ok ? "ok  " : "FAIL"} checkpoint ${pair}: ${n} on the line, ${awoken} Awoken, ${marks} fuse marks`);
  if (!ok) errors.push(`checkpoint ${pair}`);
  await page.screenshot({ path: `${out}/checkpoint-${pair}.png` });
}
if (discover) {
  await open(discover);
  await tabTo("line-0");
  await press("Enter");
  await press("f");
  await settle();
  await tabTo("line-1");
  await press("Enter");
  await tid("forge-claim").waitFor({ timeout: 10_000 }).catch(() => errors.push("the forge's sealed cards didn't open"));
  await page.screenshot({ path: `${out}/forge-${discover}.png` });
  await press("2");
  await press("Enter");
  await tid("forge-claimed").waitFor({ timeout: 10_000 }).catch(() => errors.push("Enter on Claim didn't claim"));
  await page.screenshot({ path: `${out}/claimed-${discover}.png` });
  const [a, b] = discover.split("+");
  const view = await page.evaluate(async ([a, b]) => (await fetch(`api/v1/codex/${a}/${b}`)).json(), [a, b]);
  const me = await page.evaluate(() => JSON.parse(localStorage.getItem("arena.player") ?? "null")?.name ?? "");
  console.log(`codex ${discover}: ${view.state}, by ${view.claim?.by?.name ?? view.discoveredBy?.name ?? JSON.stringify(view).slice(0, 200)} (this player: ${me})`);
  if (view.state !== "discovered") errors.push(`codex: ${discover} is ${view.state}`);
  await press("Enter");
  await settle();
}
const clicks = await page.evaluate(() => window.__mouse);
if (clicks) errors.push(`${clicks} mouse clicks`);
await browser.close();
if (errors.length) {
  console.log(`FAIL\n${errors.join("\n")}`);
  process.exit(1);
}
console.log("ok: keys only");
