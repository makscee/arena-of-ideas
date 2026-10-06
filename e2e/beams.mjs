// R3-21: target beams. Shared by mvp-phone.mjs and mvp-desktop.mjs.
//
// From the battle's start it steps beat by beat (paused: beams stay, still)
// and checks every bright beam: it starts in its source card (or at the
// clash) and ends in its target card, a self-target is a ring around its
// card, and no beam covers the caption or a card's PWR / HP numbers: with the
// layer made hit-testable, the point at each number's centre and every point
// of a beam inside the caption still hits the number or the caption.
// Then, playing, the newest beam draws (an animation runs); under
// prefers-reduced-motion none runs and the beams still show.

/** Every beam on screen, with its ends and the boxes it should start and end in. */
const readBeams = (page) =>
  page.evaluate(() => {
    const box = (id) => {
      const c = document.querySelector(`.bv-line .bv-card[data-unit="${CSS.escape(id)}"]`);
      const r = (c?.parentElement ?? c)?.getBoundingClientRect();
      return r ? { l: r.left, t: r.top, r: r.right, b: r.bottom } : null;
    };
    return [...document.querySelectorAll(".bv-fx .bv-beam")].map((g) => {
      const d = g.querySelector("path")?.getAttribute("d") ?? "";
      const n = (d.match(/-?\d+(\.\d+)?/g) ?? []).map(Number);
      const rect = g.querySelector("rect");
      return {
        now: g.classList.contains("now"),
        ring: g.classList.contains("ring"),
        from: g.dataset.from,
        to: g.dataset.to,
        p0: n.length === 6 ? { x: n[0], y: n[1] } : null,
        c: n.length === 6 ? { x: n[2], y: n[3] } : null,
        p1: n.length === 6 ? { x: n[4], y: n[5] } : null,
        ringBox: rect ? { l: +rect.getAttribute("x"), t: +rect.getAttribute("y"), r: +rect.getAttribute("x") + +rect.getAttribute("width"), b: +rect.getAttribute("y") + +rect.getAttribute("height") } : null,
        fromBox: g.dataset.from === "clash" ? null : box(g.dataset.from),
        toBox: box(g.dataset.to),
      };
    });
  });

/** What hides what, measured: the layer is made hit-testable, then each
 * PWR/HP number's centre and each beam point inside the caption must still
 * hit the number or the caption, not the layer. */
const covered = (page) =>
  page.evaluate(() => {
    const fx = document.querySelector(".bv-fx");
    if (!fx) return [];
    const style = document.createElement("style");
    // Only the beams themselves: the layer's own box covers the whole screen.
    style.textContent = ".bv-fx * { pointer-events: visiblePainted !important; }";
    document.head.append(style);
    const bad = [];
    try {
      for (const n of document.querySelectorAll(".bv-line .bv-card:not(.dead) .stats > *")) {
        const r = n.getBoundingClientRect();
        if (!r.width) continue;
        const hit = document.elementFromPoint(r.left + r.width / 2, r.top + r.height / 2);
        if (hit && hit !== fx && fx.contains(hit)) bad.push(`a beam covers ${n.closest(".bv-card").dataset.unit}'s number ${n.textContent}`);
      }
      const cap = document.querySelector('[data-testid="caption"]').getBoundingClientRect();
      for (const p of fx.querySelectorAll("path")) {
        const len = p.getTotalLength();
        for (let i = 0; i <= 40; i++) {
          const pt = p.getPointAtLength((len * i) / 40);
          if (pt.x <= cap.left + 1 || pt.x >= cap.right - 1 || pt.y <= cap.top + 1 || pt.y >= cap.bottom - 1) continue;
          const hit = document.elementFromPoint(pt.x, pt.y);
          if (hit && hit !== fx && fx.contains(hit)) { bad.push(`a beam covers the caption at ${Math.round(pt.x)},${Math.round(pt.y)}`); break; }
        }
      }
    } finally {
      style.remove();
    }
    return bad;
  });

const fxAnimations = (page) => page.evaluate(() => document.getAnimations().filter((a) => a.effect?.target?.closest?.(".bv-fx")).length);

export async function beamChecks(page, errors, { shot, phone }) {
  const where = phone ? "phone" : "desktop";
  const paused = async () => (await page.getByTestId("battle-play").textContent()) === "▶";
  const toStart = async () => {
    await page.getByTestId("battle-replay").click();
    if (!(await paused())) await page.getByTestId("battle-play").click();
    for (let i = 0; i < 400 && !(await page.getByTestId("battle-back").isDisabled()); i++) await page.getByTestId("battle-back").click();
  };
  const inside = (p, b, pad = 2) => p && b && p.x >= b.l - pad && p.x <= b.r + pad && p.y >= b.t - pad && p.y <= b.b + pad;

  await toStart();
  const seen = { line: 0, ring: 0, clash: 0, fan: 0 };
  let shotTaken = false;
  for (let step = 0; step < 400; step++) {
    if (await page.getByTestId("battle-step").isDisabled()) break;
    await page.getByTestId("battle-step").click();
    const beams = await readBeams(page);
    const now = beams.filter((b) => b.now);
    for (const b of now) {
      const tag = `beams (${where}, step ${step + 1}): ${b.from} → ${b.to}`;
      if (b.ring) {
        seen.ring++;
        if (!b.toBox || !inside({ x: b.ringBox.l + 4, y: b.ringBox.t + 4 }, b.toBox, 4) || !inside({ x: b.ringBox.r - 4, y: b.ringBox.b - 4 }, b.toBox, 4)) errors.push(`${tag}: the ring isn't around its card`);
        continue;
      }
      if (!b.p0) { errors.push(`${tag}: no path`); continue; }
      seen.line++;
      if (b.from === "clash") seen.clash++;
      else if (!inside(b.p0, b.fromBox)) errors.push(`${tag}: starts at ${Math.round(b.p0.x)},${Math.round(b.p0.y)}, outside its source card`);
      if (!inside(b.p1, b.toBox)) errors.push(`${tag}: ends at ${Math.round(b.p1.x)},${Math.round(b.p1.y)}, outside its target card`);
    }
    if (new Set(now.map((b) => b.from)).size < now.length) seen.fan++;
    for (const e of await covered(page)) errors.push(`beams (${where}, step ${step + 1}): ${e}`);
    if (!shotTaken && now.length > 1 && beams.some((b) => !b.now)) { await shot("battle-beams"); shotTaken = true; }
    if (errors.length > 20) break;
  }
  if (!seen.line) errors.push(`beams (${where}): no beam drawn in the whole fight`);
  if (!shotTaken) await shot("battle-beams");
  console.log(`beams (${where}): ${seen.line} lines (${seen.clash} from the clash), ${seen.ring} rings, ${seen.fan} fans checked`);

  // Playing, the newest beam draws.
  await toStart();
  await page.getByTestId("battle-play").click();
  let ran = 0;
  for (let i = 0; i < 40 && !ran; i++) { ran = await fxAnimations(page); if (!ran) await page.waitForTimeout(50); }
  if (!ran) errors.push(`beams (${where}): no beam drew while playing`);
  await page.getByTestId("battle-play").click();

  // Under reduced motion nothing in the layer moves, and the beams still show.
  await page.emulateMedia({ reducedMotion: "reduce" });
  await toStart();
  await page.getByTestId("battle-play").click();
  let shown = 0, moving = 0;
  for (let i = 0; i < 30; i++) {
    await page.waitForTimeout(100);
    shown = Math.max(shown, (await readBeams(page)).length);
    moving = Math.max(moving, await fxAnimations(page));
  }
  if (!(await paused())) await page.getByTestId("battle-play").click();
  await page.emulateMedia({ reducedMotion: null });
  if (moving) errors.push(`beams (${where}): ${moving} animations ran under reduced motion`);
  if (!shown) errors.push(`beams (${where}): no beam showed under reduced motion`);
}
