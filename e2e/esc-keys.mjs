// Esc everywhere (round 3, R3-6, docs/round3/words.md (9)): a keyboard pass
// with a fresh player, at the width it's given. Esc closes the top-most thing
// (a term's popover or sheet, then the sheet under it), else goes back, else
// opens the run menu; Esc on the menu resumes, on Abandon cancels.
// e2e/mvp-phone.mjs runs it on a 390px phone; e2e/mvp-desktop.mjs at its
// desktop width and in a 390px window (a keyboard under 1024px).

/** Runs the pass in its own context; each failure goes into `errors`, tagged. */
export async function escPass(browser, url, { label, viewport, touch = false, errors }) {
  const ctx = await browser.newContext({ viewport, deviceScaleFactor: 1, ...(touch ? { isMobile: true, hasTouch: true } : {}) });
  // Under 1024px (ui/dom.ts desktopQuery) sheets are overlays; wider, the inspector.
  const mobile = viewport.width < 1024;
  const page = await ctx.newPage();
  const fail = (m) => errors.push(`esc ${label}: ${m}`);
  page.on("pageerror", (e) => fail(`pageerror: ${e.message}`));
  const esc = () => page.keyboard.press("Escape");
  const tid = (id) => page.getByTestId(id);
  /** Waits up to 3 s for the element to be (or stop being) on screen. */
  const expectShown = async (what, locator, shown = true) => {
    const ok = await locator.first().waitFor({ state: shown ? "visible" : "hidden", timeout: 3000 }).then(() => true, () => false);
    if (!ok) fail(`${what}: ${shown ? "not shown" : "still shown"}`);
    return ok;
  };
  const home = () => expectShown("back on Home", tid("play"));
  try {
    await page.goto(url, { timeout: 20_000 });
    await tid("name-input").fill(`Esc${label}${Date.now().toString(36).slice(-4)}`);
    await tid("name-submit").click();
    await tid("play").waitFor();

    // Home: Esc closes the settings; Rules from them, then Esc closes the rules; with nothing open it stays on Home.
    await tid("settings").click();
    await tid("settings-sheet").waitFor();
    await esc();
    await expectShown("Home → Settings → Esc", tid("settings-sheet"), false);
    await tid("settings").click();
    await tid("rules-open").click();
    await tid("rules").waitFor();
    await esc();
    await expectShown("Home → Rules → Esc", tid("rules"), false);
    await esc();
    await home();

    // Stats → Esc → Home.
    await tid("stats").click();
    await tid("stats-back").waitFor();
    await esc();
    await home();

    // Codex: a unit, then a term in it; Esc closes the term only, then the
    // unit (phone: its sheet; desktop: the inspector), then goes Back.
    await tid("codex").click();
    await tid("codex-units").waitFor();
    await tid("codex-unit").first().click();
    const unit = mobile ? page.locator('.overlay [data-testid="unit-sheet"]') : page.locator('[data-testid="inspector"] [data-testid="unit-sheet"]');
    await unit.first().waitFor();
    await unit.first().getByTestId("term").first().click();
    const term = mobile ? page.locator('.overlay [data-testid="term-sheet"]') : tid("term-popover");
    await term.first().waitFor();
    await esc();
    await expectShown("Codex → unit → term → Esc closes the term", term, false);
    await expectShown("Codex → unit → term → Esc keeps the unit", unit);
    await esc();
    await expectShown("Codex → unit → Esc closes the unit", unit, false);
    await expectShown("Codex → unit → Esc stays in the Codex", tid("codex-units"));
    // The search: the browser clears it, then Esc blurs it, then Back.
    await tid("codex-search").fill("a");
    await esc();
    if ((await tid("codex-search").inputValue()) !== "") fail("Codex search → Esc doesn't clear it");
    await esc();
    if (await tid("codex-search").evaluate((el) => el === document.activeElement)) fail("Codex search → Esc twice: still focused");
    await expectShown("Codex search → Esc twice stays in the Codex", tid("codex-units"));
    await esc();
    await home();

    // Shop: a sheet closes first (phone: an offer stays unbought), then the
    // run menu opens, and Esc on it resumes.
    await tid("play").click();
    await tid("gold").waitFor();
    if (mobile) {
      const gold = await tid("gold").textContent();
      await tid("offer-0").click();
      await tid("offer-close").waitFor();
      await esc();
      await expectShown("Shop → offer → Esc", tid("offer-close"), false);
      if ((await tid("gold").textContent()) !== gold) fail(`Shop → offer → Esc bought it (${gold} → ${await tid("gold").textContent()})`);
    }
    await tid("shop-rules").click();
    await tid("rules").waitFor();
    await esc();
    await expectShown("Shop → Rules → Esc", tid("rules"), false);
    await expectShown("Shop → Rules → Esc opens no menu", tid("run-menu"), false);
    await esc();
    await expectShown("Shop → Esc opens the menu", tid("run-menu"));
    await esc();
    await expectShown("Shop → menu → Esc resumes", tid("run-menu"), false);

    // A fight: Esc opens the menu (the battle pauses), Esc resumes play.
    if (mobile) {
      await tid("offer-0").click();
      await tid("buy").click();
    } else await page.keyboard.press("1");
    await tid("line").locator(".card").first().waitFor().catch(() => {});
    await tid("fight").click();
    await tid("battle-play").waitFor();
    await page.waitForTimeout(300);
    await esc();
    await expectShown("Battle → Esc opens the menu", tid("run-menu"));
    if ((await tid("battle-play").textContent()) === "❚❚") fail("Battle → Esc: still playing under the menu");
    await esc();
    await expectShown("Battle → menu → Esc resumes", tid("run-menu"), false);
    const finished = await tid("end-card").isVisible();
    if (!finished && (await tid("battle-play").textContent()) !== "❚❚") fail("Battle → menu → Esc: not playing again");

    // Abandon's confirm: Esc is Cancel; then the run over screen's Esc goes Home.
    await esc();
    await tid("menu-abandon").click();
    await tid("abandon-text").waitFor();
    await esc();
    await expectShown("Abandon → Esc cancels", tid("abandon-text"), false);
    await expectShown("Abandon → Esc stays in the battle", tid("battle-play"));
    await esc();
    await tid("menu-abandon").click();
    await tid("abandon-confirm").click();
    await tid("run-over").waitFor();
    await esc();
    await home();

    // A playoff game watched from Home: Esc goes back Home. It needs a day
    // with 2 slayers: bots slay on their own, topping up once a minute
    // (server/src/mvp/bots.ts), so wait up to 2 minutes for them.
    const slayers = async () => (await (await page.request.get(new URL("api/v1/day", url).href, { timeout: 10_000 })).json()).slayers ?? 0;
    for (let t = Date.now(); (await slayers()) < 2 && Date.now() - t < 130_000; ) await page.waitForTimeout(5000);
    await page.locator("details.dev summary").click();
    // Home redraws with the day just ended: wait for a new Play button.
    await tid("play").evaluate((el) => (el.dataset.stale = "1"));
    await tid("end-day").click();
    await page.locator('[data-testid="play"]:not([data-stale])').waitFor({ timeout: 10_000 });
    const open = (await tid("playoff-games-open").count()) ? tid("playoff-games-open") : null;
    if (open) {
      await open.click();
      await tid("playoff-game").first().click();
      await tid("battle-play").waitFor();
      await esc();
      await home();
    } else console.log(`esc ${label}: the day ended without a playoff game; a battle from Home not checked`);
  } catch (e) {
    fail(`stopped: ${e instanceof Error ? e.message.split("\n").slice(0, 3).join(" / ") : e}`);
  } finally {
    await ctx.close();
  }
}
