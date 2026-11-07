// The chain on the field (arena#30, mission #26 move 2): one step's wire at a time, its number on it, and the
// tray's ladder numbering the steps so far with CHAIN ×N above them (arena#52). Shared by mvp-start, mvp-phone and mvp-desktop (via beams.mjs).

/** What the screen shows of the chain now: the wires, their numbers, the tray's count and rows. */
export const readChain = (page) =>
  page.evaluate(() => ({
    wires: document.querySelectorAll(".bv-fx .bv-beam").length,
    old: document.querySelectorAll(".bv-fx .bv-beam.old").length,
    dots: [...document.querySelectorAll(".bv-fx .bv-beam .bv-bn")].map((d) => Number(d.dataset.n)),
    count: Number(document.querySelector('[data-testid="chain-count"]')?.dataset.n) || 0,
    text: document.querySelector('[data-testid="chain-count"]')?.textContent ?? "",
    // The ladder's rows (arena#52): the steps of the chain on screen wear their number, the rest a dot; the foot row is the lit one.
    rows: [...document.querySelectorAll(".bv-ladder .bv-lrow .bv-lno")].map((r) => Number(r.textContent)).filter((n) => n > 0),
    now: Number(document.querySelector(".bv-ladder .bv-lrow.current .bv-lno")?.textContent) || 0,
  }));

/** What is wrong with a reading, as sentences: earlier wires left on the field, a number outside a chain,
 * a list out of step with the count, a wire numbered past the steps landed. */
export function chainProblems(c) {
  const bad = [];
  if (c.old) bad.push(`${c.old} earlier wire(s) are still on the field`);
  if (new Set(c.dots).size > 1) bad.push(`wires of different steps are drawn together (${c.dots.join(",")})`);
  if (!c.count) {
    if (c.dots.length) bad.push(`a wire wears a number (${c.dots.join(",")}) with no chain in the tray`);
    if (c.rows.length) bad.push("the tray lists steps with no CHAIN count");
    return bad;
  }
  if (!new RegExp(`×${c.count}$`).test(c.text.trim())) bad.push(`the tray says "${c.text}", the chain is at ${c.count}`);
  if (c.now !== c.count) bad.push(`the lit step is ${c.now}, the chain is at ${c.count}`);
  if (!c.rows.length || c.rows.at(-1) !== c.count) bad.push(`the list ends at ${c.rows.at(-1)}, the chain is at ${c.count}`);
  if (c.rows.some((n, i) => i > 0 && n !== c.rows[i - 1] + 1)) bad.push(`the list skips a step (${c.rows.join(",")})`);
  if (c.dots.some((n) => n > c.count)) bad.push(`a wire is numbered ${Math.max(...c.dots)}, past the steps landed (${c.count})`);
  return bad;
}
