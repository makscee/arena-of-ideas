// Probe (arena#13, the CrazyGames audit): at the CG frame's size (800×450) a tap
// near an offer card reacts, a tap on bare ground says what to do, the menu closes,
// the fight's end card shows by itself and every Next round gets to the shop.
//   node e2e/probe-cg-flow.mjs <url, e.g. http://127.0.0.1:8841/arena/>
import { launchChromium } from "./browser.mjs";
const url = process.argv[2];
const b = await launchChromium();
const page = await (await b.newContext({ viewport: { width: 800, height: 450 } })).newPage();
const errors = [];
page.on("pageerror", (e) => errors.push(`pageerror: ${e.message}`));
const check = (ok, what) => ok || errors.push(what);
await page.goto(url);
await page.getByTestId("name-input").fill("probe" + Date.now().toString(36).slice(-4));
await page.getByTestId("name-submit").click();
await page.getByTestId("play").click();
await page.getByTestId("offer-0").waitFor();
const box = await page.getByTestId("offer-0").boundingBox();
await page.mouse.click(300, 400); // bare ground of the shop panel
await page.waitForTimeout(100);
check((await page.locator(".nudge").count()) > 0, "a tap on bare ground gave no feedback");
await page.mouse.click(box.x + box.width / 2, box.y + box.height + 4); // just under the card
check((await page.getByTestId("offer-0").getAttribute("class")).includes("selected"), "a tap just past the card's edge didn't select it");
await page.mouse.click(box.x + box.width / 2, box.y + 8); // the card's top, far from the price
check((await page.getByTestId("offer-0").getAttribute("class")).includes("tapped") || (await page.getByTestId("overlay").count()) > 0, "a tap on the card's top gave no feedback");
await page.keyboard.press("Escape");
await page.reload();
await page.getByTestId("play").click();
await page.getByTestId("offer-0").click();
await page.locator('[data-testid^="line-0"]').first().click();
await page.getByTestId("menu-open").click();
await page.getByTestId("menu-resume").click();
check((await page.getByTestId("run-menu").count()) === 0, "the menu's Resume didn't close it");
await page.getByTestId("fight").click();
await page.getByTestId("end-card").waitFor({ state: "visible", timeout: 60000 }).catch(() => errors.push("the end card never showed by itself"));
await page.getByTestId("end-why").click();
await page.getByTestId("why-lost").locator("button").first().click();
await page.getByTestId("trace-done").click();
await page.getByTestId("fight").waitFor({ timeout: 8000 }).catch(() => errors.push("Next round from the chain view didn't reach the shop"));
await b.close();
if (errors.length) { console.error(errors.join("\n")); process.exit(1); }
console.log("cg flow: ok");
