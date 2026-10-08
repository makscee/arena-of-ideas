// M3-8 credits and history across versions (makscee/void-board#804): a
// local world with a hand-made chain, read in the Codex. Hedgehog was @ana's
// idea (v1, days 2–8), @bo's version is live (v2, from day 9); a seed unit
// has @cy's version live from day 11. Today is day 12. On a 360×640 phone and
// at 1280×800: the Codex card says "v2", its sheet "idea by @ana, evolved by
// @bo · v2 · 11 days live" and lists both versions; v1 opens its Library
// sheet; the seed unit's version says "evolved by @cy" alone. A screenshot
// of each. It builds the client and starts its own server on a free port with
// a temp DB; never point it at the live game.
//   npm run -s mvp:build && node --import tsx/esm e2e/mvp-versions.ts [--out e2e/.shots/mvp-versions]
import { spawn } from "node:child_process";
import { mkdirSync, mkdtempSync, rmSync } from "node:fs";
import { createServer } from "node:net";
import { tmpdir } from "node:os";
import { join } from "node:path";
import { seedUnits, swapUnit } from "../server/src/mvp/pool.js";
import { SqliteMvpStore } from "../server/src/mvp/sqlite-store.js";
import type { MvpStore, StoredUnit } from "../server/src/mvp/store.js";
// @ts-expect-error: a .mjs helper without types
import { launchChromium } from "./browser.mjs";

const args = process.argv.slice(2);
const out = (() => { const i = args.indexOf("--out"); return i >= 0 ? args[i + 1]! : "e2e/.shots/mvp-versions"; })();
mkdirSync(out, { recursive: true });
const dir = mkdtempSync(join(tmpdir(), "arena-804-"));
const dbPath = join(dir, "world.db");

/** The chain, written straight to the store (no proposal flow yet: M3-4..7). */
function buildWorld(store: MvpStore): { seedEvolved: string } {
  const at = new Date("2026-10-08T08:00:00.000Z");
  seedUnits(store, at);
  const [ana, bo, cy] = ["ana", "bo", "cy"].map((name) => ({ id: `p-${name}`, name, bot: false }));
  for (const p of [ana!, bo!, cy!]) store.addPlayer(p);
  // A day that lasts the walk: the server serves it as it is.
  const now = Date.now();
  const toDay = (seq: number) => store.putDay({ seq, day: new Date(now).toISOString().slice(0, 10), startedAt: new Date(now - 3_600_000).toISOString(), endsAt: new Date(now + 6 * 3_600_000).toISOString() });
  const put = (u: Partial<StoredUnit> & Pick<StoredUnit, "unitId" | "row">) =>
    store.putUnit({ status: "candidate", authorId: null, origin: "evolution", parentId: null, createdAt: at.toISOString(), ...u });
  const pool = store.currentPool()!.unitIds;
  const [out1, seed] = [pool[0]!, pool[1]!];
  const rat = store.unit("rat")!.row;
  toDay(2);
  put({ unitId: "hedgehog", row: { ...rat, name: "Hedgehog", emoji: "🦔" }, authorId: ana!.id, origin: "idea", status: "library" });
  swapUnit(store, out1, "hedgehog", at);
  toDay(9);
  put({ unitId: "hedgehog-2", row: { ...rat, name: "Hedgehog", emoji: "🦔" }, authorId: bo!.id, parentId: "hedgehog", rootId: "hedgehog" });
  swapUnit(store, "hedgehog", "hedgehog-2", at);
  toDay(11);
  put({ unitId: `${seed}-2`, row: store.unit(seed)!.row, authorId: cy!.id, parentId: seed, rootId: seed });
  swapUnit(store, seed, `${seed}-2`, at);
  toDay(12);
  return { seedEvolved: store.unit(seed)!.row.name };
}

const store = new SqliteMvpStore(dbPath);
const { seedEvolved } = buildWorld(store);
store.close();

const port = await new Promise<number>((r) => { const s = createServer().listen(0, "127.0.0.1", () => { const p = (s.address() as { port: number }).port; s.close(() => r(p)); }); });
const child = spawn("node", ["--import", "tsx/esm", "server/src/mvp/main.ts"], { env: { ...process.env, PORT: String(port), MVP_DEV: "1", MVP_DB: dbPath, STATIC_DIR: "mobile/dist" }, stdio: ["ignore", "inherit", "inherit"] });
const url = `http://127.0.0.1:${port}/arena/`;
for (let i = 0; i < 300; i++) {
  try { if ((await fetch(url + "api/v1/health")).ok) break; } catch {}
  await new Promise((r) => setTimeout(r, 200));
}

const browser = await launchChromium();
const errors: string[] = [];
let shots = 0;

async function walk(viewport: { width: number; height: number }, name: string): Promise<void> {
  const phone = viewport.width < 700;
  const page = await browser.newPage({ viewport, deviceScaleFactor: 2, isMobile: phone, hasTouch: phone });
  page.on("pageerror", (e: Error) => errors.push(`${name}: pageerror: ${e.message}`));
  page.on("console", (m: { type(): string; text(): string }) => m.type() === "error" && errors.push(`${name}: console: ${m.text()}`));
  const shot = (s: string) => page.screenshot({ path: `${out}/${String(++shots).padStart(2, "0")}-${name}-${s}.png` });
  const expectText = async (testid: string, re: RegExp, where: string) => {
    const t = ((await page.getByTestId(testid).last().textContent()) ?? "").replace(/\s+/g, " ").trim();
    if (!re.test(t)) errors.push(`${name} ${where}: ${testid} says "${t}", not ${re}`);
  };
  await page.goto(url);
  await page.getByTestId("name-input").fill(`${name}${Date.now().toString(36).slice(-4)}`);
  await page.getByTestId("name-submit").click();
  await page.getByTestId("play").waitFor();
  await page.getByTestId("codex").click();
  await page.getByTestId("codex-search").fill("Hedgehog");
  const card = page.getByTestId("codex-unit");
  await card.first().waitFor();
  if ((await card.count()) !== 1) errors.push(`${name}: ${await card.count()} Hedgehog cards`);
  await expectText("card-version", /^v2$/, "card");
  await shot("codex-v2-card");
  await card.first().click({ position: { x: 32, y: 60 } });
  await page.getByTestId("sheet-history").waitFor();
  await expectText("sheet-credit", /^💡 idea by @ana, evolved by @bo · v2 · 11 days live$/, "v2 sheet");
  const rows = (await page.getByTestId("history-version").allTextContents()).map((t) => t.trim());
  if (JSON.stringify(rows) !== JSON.stringify(["v1 by @ana, 7 days", "v2 by @bo, live"])) errors.push(`${name}: history ${JSON.stringify(rows)}`);
  await shot("v2-sheet");
  // v1 opens its own sheet, from the Library.
  await page.locator('[data-testid="history-version"][data-unit="hedgehog"]').last().click();
  await page.getByTestId("library-live").waitFor();
  await expectText("sheet-credit", /^💡 idea by @ana$/, "v1 sheet");
  await expectText("library-live", /^Live 11 days$/, "v1 sheet");
  await shot("v1-sheet");
  for (let i = 0; i < 3 && (await page.getByTestId("sheet-close").count()); i++) await page.getByTestId("sheet-close").last().click();
  // The seed unit @cy evolved: "evolved by" alone.
  await page.getByTestId("codex-search").fill(seedEvolved);
  await page.getByTestId("codex-unit").first().click({ position: { x: 32, y: 60 } });
  await page.getByTestId("sheet-history").waitFor();
  await expectText("sheet-credit", /^NEW ?evolved by @cy · v2 · 12 days live$/, "seed v2 sheet");
  await shot("seed-evolved-sheet");
  for (let i = 0; i < 3 && (await page.getByTestId("sheet-close").count()); i++) await page.getByTestId("sheet-close").last().click();
  // The Library: Hedgehog v1, its card without a version mark.
  await page.getByTestId("codex-tab-library").click();
  const lib = page.locator('[data-testid="library-unit"][data-unit="hedgehog"]');
  await lib.waitFor();
  if (await lib.getByTestId("card-version").count()) errors.push(`${name}: the Library's v1 card says a version`);
  await lib.click({ position: { x: 32, y: 60 } });
  await page.getByTestId("sheet-history").last().waitFor();
  await shot("library-v1");
  await page.close();
}

try {
  await walk({ width: 360, height: 640 }, "phone");
  await walk({ width: 1280, height: 800 }, "desk");
} catch (e) {
  errors.push(`threw: ${(e as Error).message}`);
} finally {
  await browser.close();
  child.kill();
  rmSync(dir, { recursive: true, force: true });
}
console.log(`${shots} screenshots in ${out}`);
if (errors.length) {
  console.error(errors.map((e) => `✗ ${e}`).join("\n"));
  process.exit(1);
}
console.log("✓ versions: credits, days live and history in the Codex");
