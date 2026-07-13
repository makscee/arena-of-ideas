import { aoi61ReadyRun, aoi61SameAbilityRun, armGuard, check, DESKTOP, finish, launch, openRun } from "./lib.mjs";

const disarm = armGuard();
const browser = await launch();
let opened = await openRun(browser, aoi61ReadyRun(), DESKTOP);
let { ctx, page } = opened;

const stored = () => page.evaluate(() => JSON.parse(localStorage.getItem("aoi.run.v1")));
const badge = async (i = 0, root = "#run-line") =>
  (await page.locator(`${root} [data-line="${i}"] .run-progression`).innerText()).replace(/\s+/g, " ").trim();

check((await badge(0)).includes("Base 2/3"), "base card visibly carries Base 2/3", await badge(0));
check((await badge(1)).includes("Awakened 3/3"), "Awakened card visibly carries Awakened 3/3", await badge(1));
await page.click('[data-line="0"] .uname');
await page.waitForSelector('#inspect-overlay:not([hidden]) .run-progression');
check((await page.locator("#inspect-overlay .run-progression").innerText()).replace(/\s+/g, " ").includes("Base 2/3"), "full inspector card repeats progression without hover");
await page.click("#ins-close");

// Third total copy Awakens the first base.
await page.click('[data-buy="0"]');
await page.waitForFunction(() => document.querySelector("#run-notice")?.textContent.includes("Awakened at 3/3"));
let state = await stored();
check(state.team[0].progression === "Awakened" && state.team[0].copies === 3, "third total copy Awakens once");
check((await badge()).includes("Awakened 3/3"), "third copy visibly changes the badge to Awakened 3/3", await badge());
check(state.team[0].base.pwr === 3 && state.team[0].base.hp === 11, "base duplicate growth is +1 PWR/+2 HP at every copy", JSON.stringify(state.team[0].base));

// Both explicit ordered previews, then commit A+B.
await page.click('[data-fusion-first="0"]');
let preview = await page.locator('[data-fuse="0:1"]').textContent();
check(preview.includes("Necromancer + Silencer"), "first ordered preview is explicit A+B", preview);
await page.click("[data-fusion-cancel]");
await page.click('[data-fusion-first="1"]');
preview = await page.locator('[data-fuse="1:0"]').textContent();
check(preview.includes("Silencer + Necromancer"), "reverse ordered preview is explicit B+A", preview);
await page.click("[data-fusion-cancel]");
await page.click('[data-fusion-first="0"]');
await page.click('[data-fuse="0:1"]');
state = await stored();
check(state.team.length === 1 && state.team[0].name === "Necromancer + Silencer", "ordered fusion creates one equal-identity composite");
check(state.team[0].fusion.meter === 0 && state.team[0].def.abilities.join(",") === "Reanimate,Hush", "fusion starts 0/3 and preserves parent Ability order");
check((await badge()).includes("Fusion · Fresh 0/3"), "fresh fusion visibly carries Fusion · Fresh 0/3", await badge());
const fusedStats = { ...state.team[0].base };

// Either parent routes into the fresh meter. Third forces the blocking choice.
await page.click('[data-buy="0"]');
check((await badge()).includes("Fusion 1/3"), "ordinary fusion meter is persistently visible", await badge());
for (let i = 0; i < 2; i++) await page.click('[data-buy="0"]');
await page.waitForSelector('[data-awaken-path="trigger"]');
state = await stored();
check(state.team[0].fusion.meter === 3, "both parent names route to one 3/3 meter");
check((await badge()).includes("Fusion · Pending 3/3"), "pending choice is visible on-card at 3/3", await badge());
check(state.team[0].base.pwr === fusedStats.pwr + 3 && state.team[0].base.hp === fusedStats.hp + 6, "three post-fusion copies each apply literal +1/+2");
check(await page.locator("#run-reroll").isDisabled(), "forced choice blocks reroll");
check(await page.locator("#run-fight").isDisabled(), "forced choice blocks fight");
check(await page.locator("#run-challenge").isDisabled(), "forced choice blocks challengeBoss");

// Live probe covers Selector branch; screenshot walk covers Trigger branch.
await page.click('[data-awaken-path="selector"]');
state = await stored();
check(state.team[0].fusion.awakening === "selector" && state.team[0].fusion.doubled, "Selector path is permanent");
check((await badge()).includes("Fusion · Selector path 3/3"), "permanent Selector path stays visible on-card", await badge());
check(state.team[0].base.pwr === (fusedStats.pwr + 3) * 2 && state.team[0].base.hp === (fusedStats.hp + 6) * 2, "choice snapshot-doubles current PWR/HP exactly once");
check(state.team[0].def.selectors.length === 2, "Selector path appends first parent's complete Selector set, existing first");
const doubled = { ...state.team[0].base };

// Save/resume, then one later parent copy is literal again.
const resumedRaw = await page.evaluate(() => localStorage.getItem("aoi.run.v1"));
await ctx.close();
opened = await openRun(browser, resumedRaw, DESKTOP);
({ ctx, page } = opened);
state = await stored();
check(state.team[0].fusion.awakening === "selector" && state.runVersion === 2, "versioned fusion state resumes with choice intact");
await page.click('[data-buy="0"]');
state = await stored();
check(state.team[0].base.pwr === doubled.pwr + 1 && state.team[0].base.hp === doubled.hp + 2, "later parent copy returns to literal +1 PWR/+2 HP");

// Complete the real ladder fight path with the ordered composite.
await page.click("#run-fight");
await page.waitForSelector("#run-battle:not([hidden])");
check((await page.locator("#run-battle-head").textContent()).includes("battle seed"), "ordered composite enters a deterministic live battle");

await ctx.close();
opened = await openRun(browser, aoi61SameAbilityRun(), DESKTOP);
({ ctx, page } = opened);
check((await page.locator("[data-fusion-first]").count()) === 0, "same-Ability Awakened pair exposes no fusion selection control");
check((await page.locator("[data-fuse]").count()) === 0, "same-Ability Awakened pair exposes no commit control");

await ctx.close();
await browser.close();
disarm();
finish("probe-awakening-fusion");
