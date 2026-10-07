// R4-16: units dance to the beat. Shared by mvp-phone.mjs and mvp-desktop.mjs.
//
// Every visible card emoji on the shop or battle screen runs the `bob`
// animation (a two-beat cycle), keyed on the beat clock (window.__beat):
// its progress through the cycle matches the clock's, odd slots a beat
// (half a cycle) behind their neighbours. Under prefers-reduced-motion,
// or on a hidden tab, no emoji moves.

/** What each visible card emoji runs, and how far off the beat it is (in
 * cycles: 0 is in step, 0.5 is a whole beat out). */
export const readDance = (page) =>
  page.evaluate(() => {
    const b = window.__beat?.();
    const out = [];
    for (const el of document.querySelectorAll("#app .card .emoji")) {
      if (!el.getClientRects().length) continue;
      const a = el.getAnimations().find((x) => x.animationName === "bob");
      const card = el.closest(".card");
      const row = [...card.parentElement.children].filter((c) => c.classList.contains("card"));
      const odd = row.indexOf(card) % 2 === 1;
      let off = null;
      if (a && b) {
        const want = ((b.index % 2) + b.phase) / 2 + (odd ? 0.5 : 0);
        const d = (((a.effect.getComputedTiming().progress - want) % 1) + 1) % 1;
        off = Math.min(d, 1 - d);
      }
      out.push({ unit: card.dataset.unit ?? card.dataset.testid ?? "", dead: card.classList.contains("dead"), anim: getComputedStyle(el).animationName, off });
    }
    return { beat: b, cards: out };
  });

/** In step: every live card emoji bobs, within 6% of a cycle (~70 ms at 105 BPM). */
export async function danceChecks(page, where, errors) {
  const r = await readDance(page);
  if (!r.beat) return void errors.push(`dance (${where}): no beat clock (window.__beat)`);
  const live = r.cards.filter((c) => !c.dead);
  if (!live.length) return void errors.push(`dance (${where}): no card emoji on screen`);
  for (const c of live) {
    if (c.anim !== "bob") errors.push(`dance (${where}): ${c.unit} doesn't bob (animation ${c.anim})`);
    else if (c.off === null || c.off > 0.06) errors.push(`dance (${where}): ${c.unit} is off the beat by ${c.off?.toFixed(3)} of a cycle`);
  }
  for (const c of r.cards.filter((c) => c.dead)) if (c.anim !== "none") errors.push(`dance (${where}): fallen ${c.unit} still bobs`);
}

/** Still: under prefers-reduced-motion no emoji moves. */
export async function stillChecks(page, where, errors) {
  const r = await readDance(page);
  for (const c of r.cards) if (c.anim !== "none") errors.push(`dance (${where}, reduced motion): ${c.unit} still bobs`);
}
