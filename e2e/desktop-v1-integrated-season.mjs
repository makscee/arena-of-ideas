// AOI-64: one authenticated desktop season through released governance, the
// production run kernel/viewer, supported controls, and the empty-tower submit.
import { createHash } from "node:crypto";
import { execFileSync } from "node:child_process";
import { mkdirSync, readdirSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import Database from "better-sqlite3";
import { BASE, DESKTOP, armGuard, check, finish, launch, loginViaUi } from "./lib.mjs";

const dbPath = process.env.AOI_E2E_DB_PATH;
if (!dbPath) throw new Error("AOI_E2E_DB_PATH is required — the orchestrator must own the temporary governance DB");
const out = process.env.SHOTS_DIR ?? join(process.cwd(), "e2e/.evidence/desktop-v1-integrated-season");
mkdirSync(out, { recursive: true });
const disarm = armGuard(270_000);
const browser = await launch();
const governance = [];
const boundaries = {};

const ideaText = "Frostbite Striker — a durable front-line frost bruiser that weakens the enemy it strikes";
const glassText = "Overpowered glass cannon — huge opening damage with almost no counterplay";
const bounceReason = "sim gauntlet: win rate above the allowed band";
const fusedName = "Frostbiter + Venomancer";

function govern(...args) {
  const stdout = execFileSync("npm", ["run", "govern", "--", ...args], {
    cwd: process.cwd(),
    env: { ...process.env, DB_PATH: dbPath, AOI_E2E: "1" },
    encoding: "utf8",
  });
  governance.push({ args, stdout });
  return stdout;
}
function context(ip) {
  return browser.newContext({ viewport: DESKTOP, extraHTTPHeaders: { "x-forwarded-for": ip } });
}
async function openTitle(ctx) {
  const page = await ctx.newPage();
  page.setDefaultTimeout(25_000);
  await page.goto(BASE, { waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  return page;
}
async function capture(page, file, boundary, fullPage = true) {
  await page.screenshot({ path: join(out, file), fullPage });
  boundaries[file] = boundary;
}
async function openIdeas(page) {
  if (!(await page.locator("#title-view").isVisible())) {
    await page.click("#home-button");
    await page.waitForSelector("#title-view:not([hidden])");
  }
  await page.click("#title-ideas");
  await page.waitForSelector("#ideas-view:not([hidden])");
  await page.waitForFunction(() => document.querySelector("#ideas-list")?.children.length > 0);
}
async function reloadIdeas(page) {
  await page.reload({ waitUntil: "domcontentloaded" });
  await page.waitForSelector("#title-view:not([hidden])");
  await openIdeas(page);
}
const rowByText = (page, text) => page.locator("#ideas-list .ideas-row").filter({ has: page.locator(".ideas-text", { hasText: text }) });
const offers = (page) => page.locator("#run-shop-row [data-offer] .uname").allTextContents();
const readRun = (page) => page.evaluate(() => {
  const raw = localStorage.getItem("remote:aoi.run.v1");
  return raw === null ? null : JSON.parse(raw);
});
const badge = async (page, index) => (await page.locator(`#run-line [data-line="${index}"] .run-progression`).innerText()).replace(/\s+/g, " ").trim();
async function leaveBattleForShop(page) {
  await page.waitForSelector("#run-battle:not([hidden])");
  await page.click("#run-skip");
  await page.waitForSelector("#run-continue:not([hidden])");
  await page.click("#run-continue");
  await page.waitForSelector("#run-shop:not([hidden]), #run-end:not([hidden])");
}
async function fightAndContinue(page) {
  await page.click("#run-fight");
  await leaveBattleForShop(page);
}
async function setPlayhead(page, id) {
  await page.locator("#scrub").fill(String(id));
  await page.waitForFunction((expected) => document.querySelector("#scrub")?.value === String(expected), id);
}
const sha256 = (path) => createHash("sha256").update(readFileSync(path)).digest("hex");

// Fresh v1 authority state, with one persisted primary identity and voters.
govern("fixture-reset", "--name", "aoi62-governance");
const maksCtx = await context("10.64.0.1");
const maks = await openTitle(maksCtx);
await loginViaUi(maks, "maks@aoi62.test", "Maks");
check((await maks.locator("#title-name").textContent()) === "Maks", "primary identity is Maks before governance");
await openIdeas(maks);
await maks.click("#ideas-reveal");
await maks.fill("#ideas-text", ideaText);
await maks.click("#ideas-submit");
await maks.waitForFunction((text) => [...document.querySelectorAll("#ideas-list .ideas-text")].some((node) => node.textContent === text), ideaText);

const glassCtx = await context("10.64.0.2");
const glass = await openTitle(glassCtx);
await loginViaUi(glass, "glass@aoi62.test", "Glass Smith");
await openIdeas(glass);
await glass.click("#ideas-reveal");
await glass.fill("#ideas-text", glassText);
await glass.click("#ideas-submit");
await glass.waitForFunction((text) => [...document.querySelectorAll("#ideas-list .ideas-text")].some((node) => node.textContent === text), glassText);
await glassCtx.close();
await reloadIdeas(maks);
await capture(maks, "still-01-browser-submissions.png", "Maks's browser-authored Frostbite idea and the separate glass-cannon submission coexist in season 1.");

govern("fixture-votes", "--ship", "idea-0", "--bounce", "idea-1");
await reloadIdeas(maks);
const eligibleShip = await rowByText(maks, ideaText).textContent();
const eligibleBounce = await rowByText(maks, glassText).textContent();
check(eligibleShip.includes("eligible") && eligibleBounce.includes("eligible"), "deterministic supporting votes make both ideas eligible");
await capture(maks, "still-02-supporting-votes-eligible.png", "Deterministic persisted supporting votes visibly make both ideas eligible.");

govern("freeze", "--season", "1", "--version", "1");
await reloadIdeas(maks);
check((await rowByText(maks, ideaText).textContent()).includes("selected · building"), "Frostbite is frozen selected/building");
check((await rowByText(maks, glassText).textContent()).includes("selected · building"), "glass cannon is frozen selected/building");
await capture(maks, "still-03-freeze-selected-building.png", "The released governance CLI freeze is visible as selected/building for both rows.");

govern("ship", "--idea", "idea-0", "--candidate", "candidates/frostbite-striker.json", "--season", "1", "--version", "1");
govern("bounce", "--idea", "idea-1", "--reason", bounceReason, "--season", "1", "--version", "1");
govern("roll", "--season", "1", "--version", "1");
await reloadIdeas(maks);
const seasonText = await maks.locator("#ideas-season").textContent();
const shippedText = await rowByText(maks, ideaText).textContent();
const bouncedText = await rowByText(maks, glassText).textContent();
check(seasonText.includes("Season 2 · content v2"), "season/content advance is visible", seasonText);
check(shippedText.includes("shipped"), "Frostbite branch is visibly shipped");
check(bouncedText.includes("bounced") && bouncedText.includes(bounceReason) && bouncedText.includes("4/5 yes"), "glass branch shows exact reason and retained votes", bouncedText);
await capture(maks, "still-04-ship-bounce-season2.png", "Season 2/content v2 shows shipped Frostbite and bounced glass with exact reason and retained 4/5 tally.");

await maks.click("#home-button");
await maks.click("#title-history");
await maks.waitForSelector('#history-view:not([hidden]) .history-row[data-season="1"]');
await maks.click('.history-row[data-season="1"]');
await maks.waitForSelector("#history-detail:not([hidden])");
check((await maks.locator("#history-detail").textContent()).includes("content v1"), "archive visibly retains content v1");
await capture(maks, "still-05-season1-archive.png", "Authenticated server History opens the immutable season-1/content-v1 final tower.");
await maks.click("#home-button");
check((await maks.locator("#title-name").textContent()) === "Maks", "Maks identity survives the season roll");
check((await maks.locator("#hub-season").textContent()).includes("Season 2 · content v2"), "hub advances to season 2/content v2");
await capture(maks, "still-06-season2-hub-same-maks.png", "Desktop hub shows the unchanged Maks identity and season 2/content v2.");

// Exact production-kernel plan, now through only the supported authenticated UI.
await maks.click("#title-play");
await maks.waitForSelector("#run-new:not([hidden])");
await maks.fill("#run-seed", "0");
await maks.click("#run-start");
await maks.waitForSelector("#run-shop:not([hidden])");
check(JSON.stringify(await offers(maks)) === JSON.stringify(["Silencer", "Venomancer", "Silencer"]), "R1 seed-0 canonical offers are exact", JSON.stringify(await offers(maks)));
let run = await readRun(maks);
check(run?.seed === 0 && typeof run?.runId === "string" && run.runId.startsWith("web-") && run.runId.length > 20, "server minted a random real runId", run?.runId ?? "missing");
const runId = run.runId;
await capture(maks, "still-07-r1-real-authenticated-shop.png", "Real authenticated season-2 run begins at seed 0 with the exact R1 canonical shop.");
await maks.click('[data-buy="1"]');
await fightAndContinue(maks);
check(JSON.stringify(await offers(maks)) === JSON.stringify(["Brawler", "Bulwark", "Bulwark"]), "R2 offers are exact", JSON.stringify(await offers(maks)));
await fightAndContinue(maks);
check(JSON.stringify(await offers(maks)) === JSON.stringify(["Venomancer", "Glacier", "Necromancer"]), "R3 offers are exact", JSON.stringify(await offers(maks)));
await maks.click('[data-buy="0"]');
check((await badge(maks, 0)).includes("Base 2/3"), "literal Venomancer duplicate reaches Base 2/3", await badge(maks, 0));
await capture(maks, "still-08-r3-literal-duplicate.png", "Supported buy makes Venomancer Base 2/3 before the deterministic reroll.");
await maks.click("#run-reroll");
check(JSON.stringify(await offers(maks)) === JSON.stringify(["Squire", "Bulwark", "Frostbiter"]), "R3 reroll is exact", JSON.stringify(await offers(maks)));
const frostOffer = maks.locator("#run-shop-row [data-offer]").filter({ hasText: "Frostbiter" }).first();
check((await frostOffer.textContent()).includes("made by Maks"), "real next-season Frostbiter offer is attributed to Maks");
await capture(maks, "still-09-r3-shipped-frostbiter-shop.png", "The real deterministic R3 reroll includes shipped Frostbiter visibly credited made by Maks.");
await capture(maks, "motion-00-shop-shipped-unit.png", "Frame walk: attributed shipped Unit in the real season-2 shop.");
await maks.click('[data-buy="2"]');
await fightAndContinue(maks);
check(JSON.stringify(await offers(maks)) === JSON.stringify(["Frostbiter", "Silencer", "Venomancer", "Frostbiter"]), "R4 offers are exact", JSON.stringify(await offers(maks)));
await capture(maks, "still-10-r4-supported-duplicates.png", "R4 exact shop exposes the supported copies needed for both base Awakenings.");
await maks.click('[data-buy="0"]');
await maks.click('[data-buy="1"]');
await maks.click('[data-buy="1"]');
check((await badge(maks, 0)).includes("Awakened 3/3") && (await badge(maks, 1)).includes("Awakened 3/3"), "both bases visibly reach Awakened 3/3");
run = await readRun(maks);
check(run.gold === 3 && run.lives === 4, "pre-fusion economy/lives are exact", `gold=${run.gold} lives=${run.lives}`);
await capture(maks, "still-11-both-bases-awakened.png", "Venomancer and Frostbiter are both visibly Awakened after literal supported buys; gold 3, lives 4.");
await capture(maks, "motion-01-both-awakened.png", "Frame walk: both legal base parents reach Awakened 3/3.");

await maks.click('[data-fusion-first="1"]');
const preview = await maks.locator('[data-fuse="1:0"]').textContent();
check(preview.includes(fusedName), "ordered preview names Frostbiter first and Venomancer second", preview);
await capture(maks, "still-12-ordered-fusion-preview.png", "Supported ordered preview explicitly reads Frostbiter + Venomancer.");
await capture(maks, "motion-02-ordered-preview.png", "Frame walk: legal ordered fusion preview before commit.");
await maks.click('[data-fuse="1:0"]');
run = await readRun(maks);
const fused = run.team[0];
check(run.team.length === 1 && fused.name === fusedName, "commit creates one equal-identity ordered composite");
check(fused.base.pwr === 7 && fused.base.hp === 25 && run.gold === 3 && run.lives === 4, "fused stats/economy/lives are exact", JSON.stringify({ base: fused.base, gold: run.gold, lives: run.lives }));
check(JSON.stringify(fused.def.abilities) === JSON.stringify(["Frostbite", "Venom"]), "composite has exactly two ordered Abilities", JSON.stringify(fused.def.abilities));
check(fused.fusion.parents[0].name === "Frostbiter" && fused.fusion.parents[1].name === "Venomancer", "ordered parent identity is persisted");
await capture(maks, "still-13-fused-line.png", "Committed Frostbiter + Venomancer is one Fusion Fresh 0/3 Unit at 7 PWR/25 HP.");
await capture(maks, "motion-03-fused-line.png", "Frame walk: committed fused identity on the live line.");

await maks.click('[data-line="0"] .uname');
await maks.waitForSelector("#inspect-overlay:not([hidden])");
const fusionInspectorText = await maks.locator("#inspect-overlay").textContent();
check(fusionInspectorText.includes("Trigger axis · Frostbiter first") && fusionInspectorText.includes("Selector axis · Venomancer second"), "fused inspector names inherited parent axes");
check(fusionInspectorText.includes("Frostbiter · made by Maks") && fusionInspectorText.includes("Venomancer · Arena core"), "fused inspector carries parent attribution");
check((await maks.locator("#inspect-overlay .ins-ab").count()) === 2, "fused inspector has exactly two Ability rows");
check(fusionInspectorText.includes("7 PWR · 25 HP"), "fused inspector carries exact stats");
await capture(maks, "still-14-fused-inspector-anatomy.png", "Fused inspector exposes ordered parent axes, exactly two Abilities, 7/25 stats, and parent attribution.", false);
await maks.keyboard.press("Escape");
await maks.waitForSelector("#inspect-overlay", { state: "hidden" });

// Actual ladder battle: production viewer, real fused Unit, exact event chain.
await maks.click("#run-fight");
await maks.waitForSelector("#run-battle:not([hidden])");
const events = await maks.locator(".bt-row").evaluateAll((rows) => rows.map((row) => ({
  id: Number(row.getAttribute("data-log-event")),
  causedBy: row.getAttribute("data-caused-by") === "" ? null : Number(row.getAttribute("data-caused-by")),
  type: row.querySelector(".bt-type")?.textContent ?? "",
  text: row.textContent ?? "",
})));
const byId = new Map(events.map((event) => [event.id, event]));
const chain = events.find((event) => {
  if (event.type !== "Summon" || event.causedBy === null) return false;
  const death = byId.get(event.causedBy);
  const hurt = death?.causedBy == null ? undefined : byId.get(death.causedBy);
  const strike = hurt?.causedBy == null ? undefined : byId.get(hurt.causedBy);
  return death?.type === "Death" && hurt?.type === "Hurt" && strike?.type === "Strike" && strike.text.includes(fusedName);
});
const deathEvent = chain?.causedBy == null ? undefined : byId.get(chain.causedBy);
const resultEvent = deathEvent?.causedBy == null ? undefined : byId.get(deathEvent.causedBy);
const rootEvent = resultEvent?.causedBy == null ? undefined : byId.get(resultEvent.causedBy);
if (!rootEvent || !resultEvent || !chain) throw new Error("actual fused battle lacks the expected Strike→Hurt→Death→Summon chain");
check([rootEvent.id, resultEvent.id, deathEvent.id, chain.id].join(",") === "12,13,14,15", "actual battle event chain is exact", [rootEvent.id, resultEvent.id, deathEvent.id, chain.id].join(","));

await setPlayhead(maks, rootEvent.id);
await capture(maks, "still-15-battle-root-playhead.png", `Actual fused battle at root Strike #${rootEvent.id}; both board lines remain primary.`, false);
await capture(maks, "motion-04-battle-root.png", `Frame walk: board-first root event #${rootEvent.id}.`, false);
await setPlayhead(maks, resultEvent.id);
await capture(maks, "still-16-battle-result-playhead.png", `Actual fused battle at immediate Hurt result #${resultEvent.id}.`, false);
await capture(maks, "motion-05-battle-result.png", `Frame walk: synchronized result event #${resultEvent.id}.`, false);
await setPlayhead(maks, chain.id);
await capture(maks, "still-17-battle-chain-playhead.png", `Actual fused battle at death-caused Summon chain event #${chain.id}.`, false);
await capture(maks, "motion-06-battle-chain.png", `Frame walk: synchronized multi-hop chain event #${chain.id}.`, false);

await maks.click(".bv-log-toggle");
await maks.locator(`.bt-row[data-log-event="${chain.id}"]`).click();
check(Number(await maks.locator("#scrub").inputValue()) === chain.id, "transcript row synchronizes scrub playhead");
check((await maks.locator(".bt-row.is-current").getAttribute("data-log-event")) === String(chain.id), "transcript current row is synchronized");
check((await maks.locator(".tr-chip.is-cur").getAttribute("data-id")) === String(chain.id), "trace chip is synchronized to chain beat");
check((await maks.locator("#event-cause [data-goto]").count()) >= 3, "cause inspector exposes the multi-hop ancestry");
await capture(maks, "still-18-battle-transcript-trace-sync.png", `Full transcript, trace, scrub and ancestry are synchronized on actual event #${chain.id}.`, false);
await maks.click(".bt-close");
await setPlayhead(maks, rootEvent.id);
await maks.locator(".unit-b[data-unit]").filter({ hasText: fusedName }).first().click();
await maks.waitForSelector("#inspect-overlay:not([hidden])");
check((await maks.locator("#inspect-overlay .ins-ab").count()) === 2, "battle inspector preserves the two Ability rows");
check(Number(await maks.locator("#scrub").inputValue()) === rootEvent.id, "opening battle inspector preserves playhead");
await capture(maks, "still-19-battle-fused-inspector-sync.png", `Shared fused Unit inspector stays open over the actual board without moving root playhead #${rootEvent.id}.`, false);
await maks.keyboard.press("Escape");
await maks.click("#run-skip");
await maks.waitForSelector("#run-continue:not([hidden])");
await capture(maks, "still-20-fused-battle-outcome.png", "Actual fused battle reaches its production result while board, outcome, and controls remain together.", false);
await maks.click("#run-continue");
await maks.waitForSelector("#run-shop:not([hidden])");
run = await readRun(maks);
check(run.round === 5 && run.team[0].name === fusedName, "fused identity survives into the next real shop");
await capture(maks, "still-21-next-shop-empty-tower.png", "Round-5 real shop retains the fused Unit; server tower is still empty before challenge.");
await capture(maks, "motion-07-next-shop.png", "Frame walk: next shop after the actual fused battle.");

await maks.click("#run-challenge");
check((await maks.locator("#run-challenge").textContent()).toLowerCase().includes("tap again"), "first supported challenge tap only arms");
check(await maks.locator("#run-shop").isVisible(), "armed challenge remains on the shop");
await capture(maks, "still-22-empty-tower-challenge-armed.png", "First supported challenge tap visibly arms the still-empty tower confirmation.");
await capture(maks, "motion-08-challenge-armed.png", "Frame walk: two-step empty-tower challenge at its armed boundary.");
await maks.click("#run-challenge");
await maks.waitForSelector("#run-end:not([hidden])");
await maks.waitForFunction(() => {
  const text = document.querySelector("#run-end-status")?.textContent ?? "";
  return text.includes("shared ladder") && !text.includes("submitting");
});
const endHead = await maks.locator("#run-end-head").textContent();
const submitText = await maks.locator("#run-end-status").textContent();
check(endHead.includes("founded the champion at floor 1"), "empty-tower challenge visibly founds floor 1", endHead);
check(submitText.includes("crown is yours") && !submitText.includes("refused"), "real run submission is accepted with crown", submitText);
run = await readRun(maks);
check(run.runId === runId && run.status === "over" && run.endedBy === "crown", "same real runId ends by crown");
await capture(maks, "still-23-founded-submitted-outcome.png", "Two-step challenge founds floor 1 and the server accepts the same run into the shared ladder.");
await capture(maks, "motion-09-founded-outcome.png", "Frame walk: final founded/submitted run outcome.");

await maks.click("#run-new-run");
await maks.waitForSelector("#title-view:not([hidden])");
await maks.click("#title-leaderboard");
await maks.waitForSelector("#leaderboard-view:not([hidden])");
const championText = await maks.locator("#leaderboard-body .tower-floor.is-champ").textContent();
check(championText.includes("Maks"), "final tower visibly names Maks as champion", championText);
await capture(maks, "still-24-final-tower-maks-champion.png", "Shared tower visibly contains the accepted floor-1 Maks champion.");
await maks.click("#home-button");
await maks.click("#title-history");
await maks.waitForSelector('#history-view:not([hidden]) .history-row[data-season="1"]');
await capture(maks, "still-25-final-history.png", "Final authenticated History still exposes the archived season-1/content-v1 tower.");
await maks.click("#home-button");
check((await maks.locator("#title-name").textContent()) === "Maks", "final hub identity remains Maks");
check((await maks.locator("#hub-season").textContent()).includes("Season 2 · content v2"), "final hub content version remains v2");
await capture(maks, "still-26-final-hub-continuity.png", "Final desktop hub preserves Maks and season 2/content v2 after accepted tower submission.");

// Read-only persisted-state observation for the mechanical continuity receipt.
const db = new Database(dbPath, { readonly: true, fileMustExist: true });
const persisted = {
  user: db.prepare("SELECT id, email, display_name FROM users WHERE id=?").get("fixture-maks"),
  pointer: db.prepare("SELECT season, content_version FROM season_state WHERE id=1").get(),
  shippedIdea: db.prepare("SELECT id, author_id, status, bounce_reason FROM ideas WHERE id='idea-0'").get(),
  bouncedIdea: db.prepare("SELECT id, author_id, status, bounce_reason FROM ideas WHERE id='idea-1'").get(),
  shippedBuild: db.prepare("SELECT author_user_id, creator_display_name, shipped_units_json FROM idea_builds WHERE season=1 AND idea_id='idea-0'").get(),
  submission: db.prepare("SELECT run_id, user_id, content_version, seed, ended_by, final_round FROM run_submissions WHERE run_id=?").get(runId),
  champion: db.prepare("SELECT run_id, user_id, round, seq, team FROM ladder_champions WHERE run_id=?").get(runId),
  serves: db.prepare("SELECT round, served_len, champion_run_id FROM run_pool_serves WHERE run_id=? ORDER BY round, id").all(runId),
  archive: db.prepare("SELECT season, content_version FROM season_archives WHERE season=1").get(),
};
db.close();
check(persisted.submission?.user_id === "fixture-maks" && persisted.submission?.content_version === 2 && persisted.submission?.seed === 0, "persisted submission ties Maks/run/content/seed");
check(persisted.champion?.round === 1 && persisted.champion?.run_id === runId, "persisted tower ties run to founded floor 1");
check(persisted.serves.length === 5 && persisted.serves.every((row) => row.served_len === 0 && row.champion_run_id === ""), "all five play reads observed the empty server tower", JSON.stringify(persisted.serves));

const continuity = {
  scenario: "desktop-v1-integrated-season",
  activeUser: { userId: persisted.user.id, email: persisted.user.email, displayName: persisted.user.display_name },
  season: persisted.pointer.season,
  contentVersion: persisted.pointer.content_version,
  run: { runId, seed: run.seed, status: run.status, endedBy: run.endedBy, finalRound: run.round },
  governance: {
    shipped: { ideaId: "idea-0", unit: "Frostbiter", companionUnit: "Glacier", creatorUserId: persisted.shippedBuild.author_user_id, creatorDisplayName: persisted.shippedBuild.creator_display_name },
    bounced: { ideaId: "idea-1", reason: persisted.bouncedIdea.bounce_reason, retainedTally: "4/5 yes" },
    archived: persisted.archive,
  },
  fusion: {
    identity: fused.name,
    orderedParents: fused.fusion.parents.map((parent) => ({ name: parent.name, creator: parent.def._creator ?? "Arena core" })),
    triggerFrom: fused.fusion.parents[0].name,
    selectorFrom: fused.fusion.parents[1].name,
    abilities: fused.def.abilities,
    stats: fused.base,
    goldAtCommit: 3,
    livesAtCommit: 4,
  },
  battle: { rootEventId: rootEvent.id, resultEventId: resultEvent.id, deathEventId: deathEvent.id, chainEventId: chain.id, eventCount: events.length },
  tower: { outcome: "founded floor 1", submission: persisted.submission, champion: { ...persisted.champion, team: JSON.parse(persisted.champion.team) }, emptyServes: persisted.serves },
  browserIntegrity: {
    writes: "supported controls only",
    observation: "read-only product-persisted remote:aoi.run.v1 and read-only SQLite SELECTs",
    forbiddenInjection: ["DOM", "localStorage/page-state", "event/board", "actor/target", "authorization bypass", "internal product mutator"].map((kind) => ({ kind, used: false })),
  },
};
writeFileSync(join(out, "continuity.json"), JSON.stringify(continuity, null, 2));
boundaries["continuity.json"] = "Mechanical identity/season/run/governance/fusion/battle/tower continuity receipt from product-persisted state.";
writeFileSync(join(out, "governance-cli.json"), JSON.stringify(governance, null, 2));
boundaries["governance-cli.json"] = "Exact released filesystem governance CLI command/output receipt.";

const files = readdirSync(out).filter((name) => name !== "manifest.json").sort();
const manifest = {
  scenario: "desktop-v1-integrated-season",
  generatedAt: new Date().toISOString(),
  continuity: {
    activeUser: continuity.activeUser,
    season: continuity.season,
    contentVersion: continuity.contentVersion,
    run: continuity.run,
    shippedUnit: continuity.governance.shipped,
    fusedIdentity: continuity.fusion,
    battleEventIds: continuity.battle,
    towerOutcome: continuity.tower.outcome,
  },
  files: files.map((name) => ({ name, sha256: sha256(join(out, name)), boundary: boundaries[name] })),
  manifestSelfHash: "excluded: a file cannot contain its own stable SHA-256; the implementation artifact records manifest.json's external hash",
};
writeFileSync(join(out, "manifest.json"), JSON.stringify(manifest, null, 2));

await maksCtx.close();
await browser.close();
disarm();
finish("desktop-v1-integrated-season");
