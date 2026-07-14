// AOI-63 reproducible still and motion frame walk for both named fixtures.
import { mkdirSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { BASE, armGuard, check, finish, launch } from "./lib.mjs";

const disarm = armGuard(180_000);
const browser = await launch();
const out = process.env.SHOTS_DIR;
if (!out) throw new Error("SHOTS_DIR required");
mkdirSync(out, { recursive: true });
const manifest = { scenarios: {} };

async function open(name, viewport, reducedMotion = "no-preference") {
  const ctx = await browser.newContext({ viewport, reducedMotion, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(30_000);
  await page.goto(`${BASE}/?battleFixture=${name}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector(`#result[data-fixture="${name}"]:not([hidden])`);
  return { ctx, page };
}
const setStep = (page, n) => page.evaluate((step) => { const scrub = document.querySelector("#scrub"); scrub.value = String(step); scrub.dispatchEvent(new Event("input", { bubbles:true })); }, n);
const events = (page) => page.evaluate(() => [...document.querySelectorAll(".bt-row")].map((r) => ({ id:Number(r.getAttribute("data-log-event")), causedBy:r.getAttribute("data-caused-by")===""?null:Number(r.getAttribute("data-caused-by")), type:r.querySelector(".bt-type")?.textContent??"", text:r.textContent??"" })));
async function shot(page, name, fullPage = true) { await page.screenshot({ path:join(out, `${name}.png`), fullPage }); }

{
  const name = "board-first-strike-status-chain";
  const { ctx, page } = await open(name, { width:1280, height:900 });
  const es = await events(page);
  const roots = new Set(["BattleStart","TurnStart","TurnEnd","PairFaced","Strike","Fatigue","BattleEnd"]);
  const strikeRoot = { value: null };
  let currentRoot = null;
  for (const event of es) {
    if (roots.has(event.type)) currentRoot = event;
    if (event.type === "StatusApplied" && /Poison/.test(event.text) && currentRoot?.type === "Strike") { strikeRoot.value = currentRoot; break; }
  }
  const root = strikeRoot.value?.id;
  const nextRoot = es.find((event) => event.id > root && roots.has(event.type));
  const complete = (nextRoot?.id ?? es.length) - 1;
  const status = es.find((event) => event.id > root && event.id <= complete && event.type === "StatusApplied");
  const partial = root + 1;
  if (root == null || !status || complete <= partial) throw new Error("missing multi-event Strike/Poison beat");
  await setStep(page, root);
  manifest.scenarios[name] = { root, partial, complete, status:status.id };
  await shot(page, "desktop-chain-00-root");
  await shot(page, "motion-chain-00-root", false);
  await setStep(page, partial); await shot(page, "desktop-chain-01-partial"); await shot(page, "motion-chain-01-partial", false);
  await setStep(page, complete); await shot(page, "desktop-chain-02-complete"); await shot(page, "motion-chain-02-complete", false);
  await page.click(".bv-log-toggle"); await shot(page, "desktop-chain-03-full-log", false); await page.click(".bt-close");
  await page.locator(".unit-b[data-unit]").first().click(); await shot(page, "desktop-chain-04-inspector", false); await page.keyboard.press("Escape");
  await ctx.close();
}
{
  const name = "board-first-summon-death-advance";
  const { ctx, page } = await open(name, { width:1280, height:900 });
  const es = await events(page);
  const death = es.find((e) => e.type === "Death");
  const summon = death && es.find((e) => e.type === "Summon" && e.causedBy === death.id);
  if (!death || !summon) throw new Error("missing death/summon boundary");
  manifest.scenarios[name] = { before:death.id-1, death:death.id, summon:summon.id, after:summon.id+1 };
  for (const [label,id] of [["00-before",death.id-1],["01-death",death.id],["02-summon",summon.id],["03-advance",summon.id+1]]) {
    await setStep(page,id); await shot(page,`desktop-death-${label}`); await shot(page,`motion-death-${label}`,false);
  }
  await ctx.close();
}
{
  const { ctx, page } = await open("board-first-strike-status-chain", { width:1280, height:900 }, "reduce");
  const strike = (await events(page)).find((e) => e.type === "Strike");
  await setStep(page,strike.id); await shot(page,"desktop-reduced-motion-static",false);
  const anim = await page.evaluate(() => getComputedStyle(document.querySelector(".unit-b.is-acting .shape-body")).animationName);
  check(anim === "none", "reduced-motion evidence frame is statically rendered");
  await ctx.close();
}
for (const scenario of ["board-first-strike-status-chain","board-first-summon-death-advance"]) {
  const { ctx, page } = await open(scenario, { width:375, height:667 });
  const es = await events(page);
  const boundary = scenario.includes("strike") ? es.find((e) => e.type === "StatusApplied")?.id : es.find((e) => e.type === "Summon")?.id;
  await setStep(page,boundary ?? 0);
  await shot(page,scenario.includes("strike")?"phone-chain-00-board":"phone-death-00-board");
  await page.click(".bv-log-toggle"); await shot(page,scenario.includes("strike")?"phone-chain-01-log":"phone-death-01-log",false); await page.click(".bt-close");
  await page.locator(".unit-b[data-unit]").first().click(); await shot(page,scenario.includes("strike")?"phone-chain-02-inspector":"phone-death-02-inspector",false);
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1);
  check(!overflow, `${scenario} 375px evidence has no horizontal overflow`);
  await ctx.close();
}
writeFileSync(join(out,"manifest.json"), JSON.stringify(manifest,null,2)+"\n");
await browser.close();
disarm();
finish("shots-board-first");
