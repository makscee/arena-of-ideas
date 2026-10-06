// R3-18: a battle card's click opens its Now sheet, the unit as it is on the
// beat on screen. Shared by mvp-phone.mjs and mvp-desktop.mjs.
//
// From the battle's start it steps beat by beat to the first card holding
// Shield (or, with no Shield in the fight, any status), clicks the card and
// checks the sheet lists every status the card shows with the same stacks,
// and the card's live HP; a status chip opens the same sheet; Esc (or Close
// on the phone) closes it and playback stays paused. A summoned card, when the
// fight has one, opens the sheet too, not a trace.

/** The cards on screen, read from the page: id, HP and status stacks. */
const readCards = (page) =>
  page.evaluate(() =>
    [...document.querySelectorAll(".bv-card:not(.dead)")].map((c) => ({
      id: c.dataset.unit,
      hp: c.querySelector(".stats .h, .stats [data-testid='hp']")?.textContent ?? null,
      statuses: [...c.querySelectorAll('[data-testid="card-status"]')].map((s) => `${s.dataset.status}×${s.querySelector("b")?.textContent}`),
      count: Number(c.querySelector('[data-testid="card-statuses"]')?.dataset.count ?? 0),
    })),
  );

export async function nowSheetChecks(page, errors, { shot, phone }) {
  const paused = async () => (await page.getByTestId("battle-play").textContent()) === "▶";
  const toStart = async () => {
    await page.getByTestId("battle-replay").click();
    if (!(await paused())) await page.getByTestId("battle-play").click();
    for (let i = 0; i < 400 && !(await page.getByTestId("battle-back").isDisabled()); i++) await page.getByTestId("battle-back").click();
  };
  const close = async () => {
    if (phone) await page.locator('[data-testid="now-sheet"] ~ .sheet-actions [data-testid="sheet-close"], .overlay:has([data-testid="now-sheet"]) [data-testid="sheet-close"]').first().click();
    else await page.keyboard.press("Escape");
    await page.getByTestId("now-sheet").waitFor({ state: "detached", timeout: 2_000 }).catch(() => errors.push(`now sheet: ${phone ? "Close" : "Esc"} didn't close it`));
  };

  // Step to the first beat a card holds Shield, else the first with any status.
  await toStart();
  let pick = null;
  let firstAny = -1;
  let summonAt = -1;
  for (let step = 0; step < 400; step++) {
    const cards = await readCards(page);
    if (summonAt < 0 && cards.some((c) => c.id?.includes("+"))) summonAt = step;
    const shield = cards.find((c) => c.statuses.some((s) => s.startsWith("Shield×")));
    if (shield) { pick = { step, id: shield.id }; break; }
    if (firstAny < 0 && cards.some((c) => c.statuses.length)) firstAny = step;
    if (await page.getByTestId("battle-step").isDisabled()) break;
    await page.getByTestId("battle-step").click();
  }
  if (!pick && firstAny >= 0) {
    await toStart();
    for (let i = 0; i < firstAny; i++) await page.getByTestId("battle-step").click();
    pick = { step: firstAny, id: (await readCards(page)).find((c) => c.statuses.length).id };
  }
  if (!pick) {
    console.log("now sheet: no unit holds a status in this fight; checked the sheet on the front card only");
    await toStart();
    pick = { step: 0, id: (await readCards(page))[0]?.id };
  }
  if (!pick.id) return void errors.push("now sheet: no battle card to click");

  const card = page.locator(`.bv-card:not(.dead)[data-unit="${pick.id}"]`);
  const want = (await readCards(page)).find((c) => c.id === pick.id);
  const box = await card.boundingBox();
  // Click the card low, clear of the change chip on its top (which traces).
  await page.mouse.click(box.x + box.width / 2, box.y + box.height - 8);
  await page.getByTestId("now-sheet").waitFor({ timeout: 2_000 }).catch(() => errors.push("now sheet: a card's click didn't open it"));
  if (!(await page.getByTestId("now-sheet").count())) return;
  if (!(await paused())) errors.push("now sheet: opened with the battle still playing");
  const got = await page.getByTestId("now-sheet").evaluate((el) => ({
    unit: el.dataset.unit,
    stats: el.querySelector('[data-testid="now-stats"]')?.textContent ?? "",
    statuses: [...el.querySelectorAll('[data-testid="live-status"]')].map((s) => `${s.dataset.status}×${s.dataset.stacks}`),
    tips: [...el.querySelectorAll('[data-testid="live-status"] .dim')].length,
    ability: el.querySelector('[data-testid="now-ability"]')?.textContent ?? "",
    none: !!el.querySelector('[data-testid="now-no-statuses"]'),
  }));
  if (got.unit !== pick.id) errors.push(`now sheet: opened for ${got.unit}, not the clicked ${pick.id}`);
  if (!/^PWR \d+.* · HP \d+ \/ \d+/.test(got.stats)) errors.push(`now sheet: stats read "${got.stats}"`);
  if (want.hp && !got.stats.includes(`HP ${want.hp} /`)) errors.push(`now sheet: HP "${got.stats}" isn't the card's ${want.hp}`);
  if (got.statuses.length !== want.count) errors.push(`now sheet: ${got.statuses.length} statuses, the card holds ${want.count}`);
  for (const s of want.statuses) if (!got.statuses.includes(s)) errors.push(`now sheet: the card shows ${s}, the sheet lists ${got.statuses.join(", ") || "none"}`);
  if (got.statuses.length && got.tips < got.statuses.length) errors.push(`now sheet: ${got.statuses.length - got.tips} statuses without a tip`);
  if (!got.statuses.length && !got.none) errors.push('now sheet: no statuses and no "No statuses." line');
  if (!got.ability.trim()) errors.push("now sheet: no ability text");
  console.log(`now sheet: ${pick.id} at step ${pick.step}: ${got.stats}; ${got.statuses.join(", ") || "no statuses"}`);
  await shot("battle-now-sheet");
  await close();
  if (!(await paused())) errors.push("now sheet: closing it resumed playback");

  // A status chip opens the same sheet.
  const chip = card.getByTestId("card-status").first();
  if (await chip.count()) {
    await chip.click();
    await page.getByTestId("now-sheet").waitFor({ timeout: 2_000 }).catch(() => errors.push("now sheet: a status chip's click didn't open it"));
    if (await page.getByTestId("now-sheet").count()) await close();
  }

  // A summoned unit opens the sheet too, not a trace.
  if (summonAt >= 0) {
    await toStart();
    for (let i = 0; i < summonAt; i++) await page.getByTestId("battle-step").click();
    const sid = (await readCards(page)).find((c) => c.id?.includes("+"))?.id;
    const sb = sid ? await page.locator(`.bv-card:not(.dead)[data-unit="${sid}"]`).boundingBox() : null;
    if (sb) {
      await page.mouse.click(sb.x + sb.width / 2, sb.y + sb.height - 8);
      await page.getByTestId("now-sheet").waitFor({ timeout: 2_000 }).catch(() => errors.push(`now sheet: the summoned ${sid} didn't open it`));
      if (await page.getByTestId("now-sheet").count()) {
        const text = await page.getByTestId("now-ability").textContent();
        console.log(`now sheet: summoned ${sid}: ${text}`);
        // R3-5: the summon has its own emoji and card.
        const head = (await page.locator('[data-testid="now-sheet"] h2').textContent()) ?? "";
        if (head.includes("✨")) errors.push(`now sheet: the summoned ${sid} has no emoji ("${head}")`);
        if (!(await page.getByTestId("now-full-card").count())) errors.push(`now sheet: the summoned ${sid} has no Full card`);
        await shot("battle-now-summon");
        await close();
      }
    }
  }
}
