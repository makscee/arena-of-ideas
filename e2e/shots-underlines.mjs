// #921: before/after of the keyword underlines, as one sheet: rows EN/RU at
// 360 and 1280, columns before | after, each cell the Codex text of Coach,
// Distractor, Bat and Divinity. "Before" puts back what the terms inherited from the
// Toybox button (its hard drop shadow and radius), which painted a dark bar
// from the icon on under each term, and lets the underline skip descenders
// (so "Blessing" stopped short of its "g"). Needs a running server.
//   node e2e/shots-underlines.mjs --url http://127.0.0.1:<port>/arena/ [--out file.png]
import { launchChromium } from "./browser.mjs";
import { localeOf } from "./lang.mjs";

const args = process.argv.slice(2);
const opt = (n) => { const i = args.indexOf(`--${n}`); return i >= 0 ? args[i + 1] : undefined; };
const url = opt("url");
const out = opt("out") ?? "docs/m5-r2/underlines.png";
const BEFORE = "button.t { box-shadow: var(--drop) !important; border-radius: var(--radius) !important; } button.t, button.t * { text-decoration-skip-ink: auto !important; }";
const UNITS = ["Coach", "Distractor", "Bat", "Divinity"];

const browser = await launchChromium();
const rows = [];
for (const [lang, width] of [["en", 360], ["ru", 360], ["en", 1280], ["ru", 1280]]) {
  const phone = width < 700;
  const page = await browser.newPage({ viewport: { width, height: phone ? 640 : 800 }, deviceScaleFactor: 2, locale: localeOf(lang), isMobile: phone, hasTouch: phone });
  await page.goto(url);
  await page.getByTestId("name-input").fill(`Shot${lang}${width}${Date.now() % 10000}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("codex").click();
  await page.getByTestId("codex-unit").first().waitFor();
  const row = { label: `${lang.toUpperCase()} ${width}`, before: [], after: [] };
  for (const name of UNITS) {
    // The search knows every unit's English name, in RU too.
    await page.getByTestId("codex-search").fill(name);
    const target = page.locator("[data-testid=codex-unit]:visible").first();
    await target.scrollIntoViewIfNeeded();
    await target.click({ position: { x: 20, y: 60 } });
    const form = page.getByTestId("sheet-form").last();
    await form.waitFor();
    for (const when of ["before", "after"]) {
      const style = when === "before" ? await page.addStyleTag({ content: BEFORE }) : null;
      await page.mouse.move(0, 0);
      row[when].push((await form.screenshot()).toString("base64"));
      await style?.evaluate((s) => s.remove());
    }
    if (await page.getByTestId("overlay").count()) {
      await page.keyboard.press("Escape");
      await page.getByTestId("overlay").waitFor({ state: "detached", timeout: 3000 }).catch(() => {});
    }
  }
  rows.push(row);
  await page.close();
}
const sheet = await browser.newPage({ viewport: { width: 1400, height: 400 }, deviceScaleFactor: 1 });
const img = (b) => `<img src="data:image/png;base64,${b}">`;
await sheet.setContent(`<style>
  body { margin: 0; padding: 16px; background: #1b1525; color: #eee; font: 15px system-ui; }
  table { border-collapse: collapse; } td, th { padding: 8px; vertical-align: top; text-align: left; }
  th { font-size: 18px; } td.l { font-weight: 700; white-space: nowrap; }
  .c { display: flex; flex-direction: column; gap: 6px; } img { width: 300px; display: block; }
</style><table><tr><th></th><th>Before</th><th>After</th></tr>${rows.map((r) => `<tr><td class="l">${r.label}</td><td><div class="c">${r.before.map(img).join("")}</div></td><td><div class="c">${r.after.map(img).join("")}</div></td></tr>`).join("")}</table>`);
await sheet.locator("table").screenshot({ path: out });
await browser.close();
console.log(`wrote ${out}`);
