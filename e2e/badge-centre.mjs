// The PWR / HP number sits in the middle of its badge (#1124, mission #1043).
// `badgeGaps(page)` measures every visible stat badge on the page: a card's
// `.stats` spans (line, bench, shop, forge preview, claim sheet, Codex) and the
// unit sheet's `.sheet-stat`. Vertically it takes the digits' own ink, not the
// line box, so a font's ascent and descent can't hide an off-centre number;
// sideways it takes the text's width (a 1's ink isn't symmetric in its advance).
// One row per badge: the gap from the badge's edge to the number at the top,
// bottom, left and right. The battle's big numbers have no badge and aren't read.
export async function badgeGaps(page) {
  return page.evaluate(() => {
    const ctx = document.createElement("canvas").getContext("2d");
    const rows = [];
    // A card's badge is the span itself; the unit sheet's is `.sheet-stat`, whose number is its <b>.
    for (const badge of document.querySelectorAll(".card .stats:not(.big) > span, .sheet-stat")) {
      const el = badge.matches(".sheet-stat") ? badge.querySelector("b") : badge;
      if (!el) continue;
      const box = badge.getBoundingClientRect();
      if (box.width < 4 || box.height < 4) continue;
      const cs = getComputedStyle(badge);
      if (cs.visibility === "hidden" || cs.display === "none" || +cs.opacity === 0) continue;
      const nodes = [];
      const walk = document.createTreeWalker(el, NodeFilter.SHOW_TEXT);
      for (let n = walk.nextNode(); n; n = walk.nextNode()) if (n.textContent.trim()) nodes.push(n);
      if (nodes.length !== 1) continue;
      const node = nodes[0];
      const range = document.createRange();
      range.selectNodeContents(node);
      const text = range.getBoundingClientRect();
      const tcs = getComputedStyle(node.parentElement);
      ctx.font = `${tcs.fontStyle} ${tcs.fontWeight} ${tcs.fontSize} ${tcs.fontFamily}`;
      const text0 = node.textContent.trim();
      const m = ctx.measureText(text0);
      const baseline = text.top + m.fontBoundingBoxAscent;
      const inkTop = baseline - m.actualBoundingBoxAscent;
      const inkBottom = baseline + m.actualBoundingBoxDescent;
      const icons = el.querySelectorAll("svg, img").length;
      rows.push({
        card: badge.closest(".card")?.getAttribute("data-testid") ?? (badge.matches(".sheet-stat") ? "sheet" : badge.closest(".card")?.className ?? ""),
        kind: badge.className, text: text0, font: tcs.fontFamily.split(",")[0],
        w: Math.round(box.width * 10) / 10, h: Math.round(box.height * 10) / 10,
        top: inkTop - box.top, bottom: box.bottom - inkBottom,
        left: icons || badge.matches(".sheet-stat") ? null : text.left - box.left, right: icons || badge.matches(".sheet-stat") ? null : box.right - text.right,
      });
    }
    return rows;
  });
}

// Rows whose number is off-centre by more than `tol` px, as readable strings.
export function badgeFailures(rows, tol = 1) {
  const bad = [];
  for (const r of rows) {
    const dv = r.top - r.bottom;
    const dh = r.left == null ? 0 : r.left - r.right;
    if (Math.abs(dv) > tol) bad.push(`${r.card} ${r.kind} "${r.text}" (${r.w}x${r.h}): ${dv > 0 ? "low" : "high"}, top gap ${r.top.toFixed(1)} vs bottom ${r.bottom.toFixed(1)}`);
    if (Math.abs(dh) > tol) bad.push(`${r.card} ${r.kind} "${r.text}" (${r.w}x${r.h}): left gap ${r.left.toFixed(1)} vs right ${r.right.toFixed(1)}`);
  }
  return bad;
}
