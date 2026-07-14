// AOI-63 named board-first production-kernel scenarios.
import { BASE, DESKTOP, PHONE, armGuard, check, finish, launch } from "./lib.mjs";

const disarm = armGuard(180_000);
const browser = await launch();

async function openScenario(name, viewport, reducedMotion = "no-preference") {
  const ctx = await browser.newContext({ viewport, reducedMotion, hasTouch: viewport.width < 700 });
  const page = await ctx.newPage();
  page.setDefaultTimeout(30_000);
  await page.goto(`${BASE}/?battleFixture=${name}`, { waitUntil: "domcontentloaded" });
  await page.waitForSelector(`#result[data-fixture="${name}"]:not([hidden])`);
  await page.waitForSelector("#board .bv-side[data-side=A] .unit-b[data-rank=front]");
  return { ctx, page };
}

const setStep = (page, id) => page.evaluate((step) => {
  const scrub = document.querySelector("#scrub");
  scrub.value = String(step);
  scrub.dispatchEvent(new Event("input", { bubbles: true }));
}, id);

const eventTable = (page) => page.evaluate(() => [...document.querySelectorAll(".bt-row")].map((row) => ({
  id: Number(row.getAttribute("data-log-event")),
  causedBy: row.getAttribute("data-caused-by") === "" ? null : Number(row.getAttribute("data-caused-by")),
  type: row.querySelector(".bt-type")?.textContent ?? "",
  text: row.textContent ?? "",
})));

async function strikeScenario() {
  const { ctx, page } = await openScenario("board-first-strike-status-chain", DESKTOP);
  const events = await eventTable(page);
  for (const family of ["Strike", "Hurt", "StatusApplied", "StatusRemoved", "Death", "Summon"])
    check(events.some((e) => e.type === family), `strike scenario actually produced ${family}`);
  check(events.some((e) => e.type === "Hurt" && /Poison/.test(e.text)), "strike scenario produced a Poison status tick");
  check(events.some((e) => {
    let parent = e.causedBy, hops = 0;
    while (parent !== null) { hops++; parent = events[parent]?.causedBy ?? null; }
    return hops >= 3;
  }), "strike scenario contains a multi-hop caused chain");

  const initial = await page.evaluate(() => ({
    a: document.querySelectorAll('.bv-side[data-side="A"] [data-zone="line"] > .unit-b').length,
    b: document.querySelectorAll('.bv-side[data-side="B"] [data-zone="line"] > .unit-b').length,
    fronts: [...document.querySelectorAll('.unit-b[data-rank="front"]')].map((el) => el.getBoundingClientRect().height),
    rears: [...document.querySelectorAll('.unit-b[data-rank="rear"]')].map((el) => ({ h: el.getBoundingClientRect().height, opacity: getComputedStyle(el).opacity })),
  }));
  check(initial.a === 5 && initial.b === 5, "both complete five-Unit lines are visible at event 0", JSON.stringify(initial));
  check(initial.fronts.length === 2 && initial.fronts.every((h) => h >= 118), "the two fronts carry larger portrait/card attention", JSON.stringify(initial.fronts));
  check(initial.rears.length === 8 && initial.rears.every((r) => Number(r.opacity) < 0.9), "all eight rear Units deliberately recede without disappearing");

  const roots = new Set(["BattleStart","TurnStart","TurnEnd","PairFaced","Strike","Fatigue","BattleEnd"]);
  const strikeRoot = { value: null };
  let currentRoot = null;
  for (const event of events) {
    if (roots.has(event.type)) currentRoot = event;
    if (event.type === "StatusApplied" && /Poison/.test(event.text) && currentRoot?.type === "Strike") { strikeRoot.value = currentRoot; break; }
  }
  const root = strikeRoot.value?.id;
  const nextRoot = events.find((event) => event.id > root && roots.has(event.type));
  const complete = (nextRoot?.id ?? events.length) - 1;
  const status = events.find((event) => event.id > root && event.id <= complete && event.type === "StatusApplied");
  const partial = root + 1;
  if (root == null || !status || complete <= partial) throw new Error("missing multi-event Strike/Poison beat");
  await setStep(page, root);
  const stable = await page.evaluate(({ partial, complete }) => {
    const cards = [...document.querySelectorAll(".unit-b[data-unit]")];
    const actor = document.querySelector(".unit-b.is-acting")?.getAttribute("data-unit");
    const targets = new Set([...document.querySelectorAll(".unit-b.is-target")].map((el) => el.getAttribute("data-unit")));
    const unrelated = cards.filter((el) => el.getAttribute("data-unit") !== actor && !targets.has(el.getAttribute("data-unit")));
    const before = new Map(unrelated.map((el) => [el.getAttribute("data-unit"), { el, rect: el.getBoundingClientRect().toJSON() }]));
    const order0 = cards.map((el) => el.getAttribute("data-unit"));
    const scrub = document.querySelector("#scrub");
    scrub.value = String(partial); scrub.dispatchEvent(new Event("input", { bubbles: true }));
    const partialRows = document.querySelectorAll(".battle-event .be-row").length;
    scrub.value = String(complete); scrub.dispatchEvent(new Event("input", { bubbles: true }));
    const completeRows = document.querySelectorAll(".battle-event .be-row").length;
    const now = [...document.querySelectorAll(".unit-b[data-unit]")];
    const order1 = now.map((el) => el.getAttribute("data-unit"));
    let identity = true, geometry = true;
    for (const [id, prior] of before) {
      const el = now.find((node) => node.getAttribute("data-unit") === id);
      if (el !== prior.el) identity = false;
      if (!el) { geometry = false; continue; }
      const r = el.getBoundingClientRect();
      if (["x","y","width","height"].some((k) => Math.abs(r[k] - prior.rect[k]) > .6)) geometry = false;
    }
    return { identity, geometry, sameOrder: JSON.stringify(order0) === JSON.stringify(order1), partialRows, completeRows };
  }, { partial, complete });
  check(stable.identity, "root→partial→complete retains unrelated Unit DOM identity");
  check(stable.geometry, "root→partial→complete retains unrelated Unit geometry");
  check(stable.sameOrder, "root→partial→complete retains board Unit order");
  check(stable.partialRows > 0 && stable.completeRows >= stable.partialRows, "causal beat streams partial then complete effects", JSON.stringify(stable));

  await setStep(page, root);
  // Explicitly compare ids without relying on narrated text.
  const markTruth = await page.evaluate(() => ({ actor: document.querySelector(".unit-b.is-acting")?.getAttribute("data-unit"), panel: document.querySelector(".battle-event")?.getAttribute("data-acting"), target: document.querySelector(".unit-b.is-target")?.getAttribute("data-unit") }));
  check(markTruth.actor === markTruth.panel && markTruth.actor !== markTruth.target, "structured actor and real target highlight in existing board positions", JSON.stringify(markTruth));

  const turnEnd = events.find((e) => e.type === "TurnEnd" && events.some((x) => x.type === "Hurt" && x.causedBy === e.id));
  if (turnEnd) {
    await setStep(page, turnEnd.id);
    check(await page.locator(".acting-phase").count() === 1 && await page.locator(".is-acting,.is-target").count() === 0, "non-Strike phase root clears stale highlights");
    await setStep(page, turnEnd.id + 1);
    check((await page.locator(".battle-event").getAttribute("data-root")) === "End of turn" && await page.locator(".battle-event .be-row").count() > 0, "non-Strike root streams its caused effects in the adjacent beat");
  }

  const boardBefore = await page.locator("#board").boundingBox();
  await page.click(".bv-log-toggle");
  check(await page.locator(".battle-transcript").isVisible(), "full event transcript opens on demand");
  const boardAfter = await page.locator("#board").boundingBox();
  check(Math.abs(boardBefore.y - boardAfter.y) < .5 && Math.abs(boardBefore.height - boardAfter.height) < .5, "fixed transcript does not move or replace the board");
  await page.locator(`.bt-row[data-log-event="${status.id}"]`).click();
  check(Number(await page.locator("#scrub").inputValue()) === status.id && await page.locator(".bt-row.is-current").getAttribute("data-log-event") === String(status.id), "transcript, scrub and current event id stay synchronized");
  const ancestryHtml = await page.locator("#event-cause").innerHTML();
  check(!ancestryHtml.includes("cause-empty") && ancestryHtml.includes("data-goto"), "transcript event selection exposes linked ancestry", ancestryHtml.slice(0, 140));
  await page.click(".bt-close");

  await page.locator('.unit-b[data-unit]').first().click();
  check(await page.locator("#inspect-overlay:not([hidden]) [data-inspector-card]").count() === 1, "shared Unit inspector remains reachable without replacing board");
  await page.keyboard.press("Escape");
  await ctx.close();
}

async function deathScenario(viewport, tag) {
  const { ctx, page } = await openScenario("board-first-summon-death-advance", viewport);
  const events = await eventTable(page);
  const death = events.find((e) => e.type === "Death");
  const summon = death && events.find((e) => e.type === "Summon" && e.causedBy === death.id);
  check(Boolean(death && summon), `${tag} fixture produced death-caused summon`);
  if (!death || !summon) throw new Error("missing named boundaries");
  await setStep(page, death.id - 1);
  const transition = await page.evaluate(({ deathId, summonId }) => {
    const scrub = document.querySelector("#scrub");
    const beforeCards = [...document.querySelectorAll(".unit-b[data-unit]")];
    const beforeIds = beforeCards.map((el) => el.getAttribute("data-unit"));
    const fronts0 = [...document.querySelectorAll('[data-zone="line"] > .unit-b:first-child')].map((el) => el.getAttribute("data-unit"));
    const refs = new Map(beforeCards.map((el) => [el.getAttribute("data-unit"), el]));
    scrub.value = String(deathId); scrub.dispatchEvent(new Event("input", { bubbles: true }));
    const fallen = document.querySelector(".unit-b.is-dying-event");
    const fronts1 = [...document.querySelectorAll('[data-zone="line"] > .unit-b:first-child')].map((el) => el.getAttribute("data-unit"));
    const identityAtDeath = [...refs].filter(([id]) => id !== fallen?.getAttribute("data-unit")).every(([id, el]) => document.querySelector(`.unit-b[data-unit="${CSS.escape(id)}"]`) === el);
    scrub.value = String(summonId); scrub.dispatchEvent(new Event("input", { bubbles: true }));
    const entering = document.querySelector(".unit-b.is-summoning");
    const enteringId = entering?.getAttribute("data-unit");
    return { beforeIds, fallenRank: fallen?.getAttribute("data-rank"), identityAtDeath, fronts0, fronts1, summonWasAbsent: enteringId != null && !beforeIds.includes(enteringId), entering: enteringId, enteringRank: entering?.getAttribute("data-rank") };
  }, { deathId: death.id, summonId: summon.id });
  check(transition.fallenRank === "grave", `${tag} death moves the same local identity to grave at its logged event`, JSON.stringify(transition));
  check(transition.identityAtDeath, `${tag} unrelated Unit DOM nodes survive the death boundary`);
  check(JSON.stringify(transition.fronts0) !== JSON.stringify(transition.fronts1), `${tag} next front advances exactly at death boundary`, JSON.stringify(transition));
  check(transition.summonWasAbsent && transition.entering && transition.enteringRank !== "grave", `${tag} death-caused summon enters the real line only at its logged event`, JSON.stringify(transition));
  const overflow = await page.evaluate(() => document.documentElement.scrollWidth > document.documentElement.clientWidth + 1);
  check(!overflow, `${tag} board/log/controls have no horizontal page overflow`);
  await page.click(".bv-log-toggle");
  check(await page.locator(".battle-transcript").isVisible(), `${tag} full log remains reachable`);
  await page.click(".bt-close");
  await page.locator('.unit-b[data-unit]').first().click();
  check(await page.locator("#inspect-overlay:not([hidden])").isVisible(), `${tag} inspector remains reachable`);
  await ctx.close();
}

async function reducedMotion() {
  const { ctx, page } = await openScenario("board-first-strike-status-chain", DESKTOP, "reduce");
  const events = await eventTable(page);
  const strike = events.find((e) => e.type === "Strike");
  await setStep(page, strike.id);
  const state = await page.evaluate(() => ({ actor: getComputedStyle(document.querySelector(".unit-b.is-acting .shape-body")).animationName, target: getComputedStyle(document.querySelector(".unit-b.is-target .shape-aura")).animationName, acting: document.querySelectorAll(".unit-b.is-acting").length, targets: document.querySelectorAll(".unit-b.is-target").length }));
  check(state.actor === "none" && state.target === "none", "reduced motion removes nonessential portrait motion", JSON.stringify(state));
  check(state.acting === 1 && state.targets >= 1, "reduced motion preserves static actor/target meaning");
  await ctx.close();
}

await strikeScenario();
await deathScenario(DESKTOP, "desktop");
await deathScenario(PHONE, "375px");
await reducedMotion();
await browser.close();
disarm();
finish("probe-board-first");
