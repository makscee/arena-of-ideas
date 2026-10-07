// Finds what makes the round-8 phone shop (5 in line, full bench, 6 offers)
// sometimes taller than 640 px: reaches it N times on a fresh local server
// and prints the shop's tallest parts whenever the page scrolls.
//   node e2e/probe-round8-height.mjs [N]   (needs `npm run mvp:build`)
import { spawn } from "node:child_process";
import { createServer } from "node:net";
import { launchChromium } from "./browser.mjs";

const N = Number(process.argv[2] ?? 12);
const port = await new Promise((res) => { const s = createServer().listen(0, () => { const p = s.address().port; s.close(() => res(p)); }); });
const child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: ":memory:" }, stdio: ["ignore", "ignore", "inherit"] });
const url = `http://127.0.0.1:${port}/arena/`;
for (let i = 0; i < 100; i++) { try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {} await new Promise((r) => setTimeout(r, 200)); }
const call = async (method, path, body, pid) => {
  const res = await fetch(new URL(`api/v1${path}`, url), { method, headers: { "content-type": "application/json", ...(pid ? { "X-Arena-Player": pid } : {}) }, ...(body ? { body: JSON.stringify(body) } : {}) });
  const json = await res.json();
  if (!res.ok) throw new Error(`${method} ${path}: ${json.error}`);
  return json;
};
const browser = await launchChromium();
let tall = 0;
try {
  const page = await browser.newPage({ viewport: { width: 360, height: 640 }, deviceScaleFactor: 2, isMobile: true, hasTouch: true });
  await page.goto(url);
  for (let k = 0; k < N; k++) {
    const p = await call("POST", "/players", { name: `Probe${k}x${Date.now().toString(36).slice(-3)}` });
    let r = await call("POST", "/runs", undefined, p.id);
    const full = (r) => r.phase === "shop" && r.round >= 8 && r.line.length === 5 && r.bench.length === 3;
    for (let steps = 0; steps < 4000 && !full(r); steps++) {
      if (r.phase === "over") { r = await call("POST", "/runs", undefined, p.id); continue; }
      const o = r.offers[0];
      const room = r.line.length < 5 || r.bench.length < 3;
      const d = r.gift ? { kind: "gift", pick: null } : r.phase === "shop" && room && o && r.gold >= o.cost ? { kind: "buy", slot: o.slot } : { kind: "fight" };
      r = (await call("POST", `/runs/${r.runId}/decisions`, d, p.id)).run;
    }
    if (!full(r)) { console.log(`#${k}: never full`); continue; }
    await page.evaluate((p) => localStorage.setItem("arena.player", JSON.stringify(p)), p);
    await page.reload();
    await page.getByTestId("play").click();
    await page.getByTestId("fight").waitFor();
    await page.waitForTimeout(300);
    const m = await page.evaluate(() => {
      const H = document.documentElement.scrollHeight;
      const parts = [...document.querySelectorAll("#app *")].filter((e) => e.children.length < 40 && e.getBoundingClientRect().height > 0)
        .map((e) => ({ el: `${e.tagName.toLowerCase()}${e.dataset.testid ? `[${e.dataset.testid}]` : ""}.${[...e.classList].join(".")}`, h: Math.round(e.getBoundingClientRect().height * 10) / 10, text: (e.textContent ?? "").trim().slice(0, 70) }));
      return { H, parts };
    });
    const sig = m.parts.filter((x) => /hint|err|banner|head|top|bar|opp|champ|status|round|info|row/.test(x.el)).map((x) => `${x.el}=${x.h} "${x.text}"`);
    console.log(`#${k}: round ${r.round}, ${m.H}px${m.H > 640.5 ? " SCROLLS" : ""}`);
    if (m.H > 640.5) tall++;
    if (process.env.DUMP) (await import("node:fs")).writeFileSync(`${process.env.DUMP}/r8-${k}-${m.H}.txt`, m.parts.map((x) => `${x.el}=${x.h} "${x.text}"`).join("\n"));
    if (m.H > 640.5) console.log("  " + sig.join("\n  "));
  }
} finally { await browser.close(); child.kill(); }
console.log(`${tall}/${N} scrolled`);
