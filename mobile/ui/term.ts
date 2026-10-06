// Unit text with its terms highlighted (R2-8, docs/round2/icons.md "How the
// client renders it"). richText() draws describe.ts's tagged pieces: a trigger
// clause is one amber pill with its icon, a status is its icon plus its word in
// its colour, a target is tinted teal (ally) or pink (enemy), and an amount is
// its effect's icon plus a bold number. Every term is a button: hovering it on
// a desktop shows its rule in a tooltip, tapping it (or Enter) opens a small
// sheet with the rule. Both offer "Open in Codex" once something registers a
// Codex link (slice R2-11, setCodexLink).
import type { DescribeSegment } from "../../src/describe";
import { termDef, termIcon, type TermDef, type TermId } from "../../src/glossary";
import { closable, h } from "./dom";
import { changedTokens } from "./diff";
import { icon } from "./icon";

/** Opens a term's Codex entry; set by the Codex (R2-11). Unset: no link. */
let codexLink: ((id: TermId) => void) | null = null;
export function setCodexLink(open: ((id: TermId) => void) | null): void {
  codexLink = open;
}

/** What a run's tooltip and sheet say. An eventUnit target follows the words on
 * screen ("this unit", "that ally"), not the selector's generic label. */
export function termInfo(seg: DescribeSegment): (TermDef & { id: TermId }) | undefined {
  const id = seg.term;
  if (!id) return undefined;
  const def = termDef(id);
  if (!def) return undefined;
  if (id === "target:eventUnit") {
    const label = seg.text.charAt(0).toUpperCase() + seg.text.slice(1);
    const holder = termDef("target:holder")!;
    return /^this unit$/i.test(seg.text) ? { ...holder, id, label } : { ...def, id, label };
  }
  return { ...def, id };
}

/** The status a run is about: its own status term, or the status a "lands on"
 * clause names (so the clause's pill shows that status's icon). */
const statusOf = (seg: DescribeSegment): string | undefined => (seg.term?.startsWith("status:") ? seg.term.slice("status:".length) : undefined);

/** The tone class of a run: its term's tone, a target's side, a status's own. */
function toneOf(seg: DescribeSegment, def: TermDef): string {
  if (seg.term?.startsWith("target:")) return seg.side ? `tone-${seg.side}` : `tone-${def.tone}`;
  return `tone-${def.tone}`;
}

export interface RichOptions {
  /** Per piece, what to draw in place of its text (markChangedPieces' underlines). */
  content?: (Node | string)[][];
  /** Icon size in px (20 reads well in a sheet). */
  size?: number;
}

/** A sentence's pieces as inline nodes, its terms highlighted and tappable. */
export function richText(segs: DescribeSegment[], o: RichOptions = {}): Node[] {
  const size = o.size ?? 16;
  // The effects whose number is its own run: those numbers carry the icon,
  // their verbs ("deal … damage", "heal") are only coloured.
  const withAmount = new Set(segs.filter((s) => s.amount && s.term).map((s) => s.term!));
  const out: Node[] = [];
  let pill: HTMLElement | null = null;
  segs.forEach((seg, i) => {
    const kids = o.content?.[i] ?? [seg.text];
    if (seg.clause !== "when") pill = null;
    else if (!pill) {
      // One pill per trigger clause; its icon is the status the clause names,
      // else the trigger's own.
      let j = i;
      const clause: DescribeSegment[] = [];
      while (j < segs.length && segs[j]!.clause === "when") clause.push(segs[j++]!);
      const trig = clause.find((s) => s.term?.startsWith("trigger:"))?.term;
      const status = clause.map(statusOf).find(Boolean);
      const ic = trig ? termIcon(trig, status) : undefined;
      pill = h("span", { class: "tpill tone-when", "data-testid": "term-when" });
      if (ic) pill.append(icon(ic, size, "tpill-ic"));
      out.push(pill);
    }
    const info = termInfo(seg);
    const nodes: Node[] = info ? [termButton(seg, info, kids, withAmount, size, !!pill)] : kids.map((k) => (typeof k === "string" ? document.createTextNode(k) : k));
    if (pill) pill.append(...nodes);
    else out.push(...nodes);
  });
  return out;
}

function termButton(seg: DescribeSegment, info: TermDef & { id: TermId }, kids: (Node | string)[], withAmount: Set<TermId>, size: number, inPill: boolean): HTMLElement {
  const id = info.id;
  const group = id.slice(0, id.indexOf(":"));
  const b = h("button", { type: "button", class: `t t-${group} ${toneOf(seg, info)}`, "data-term": id, "data-testid": "term", "aria-label": `${seg.text}: ${info.tip}` });
  if (seg.amount) b.classList.add("t-amount");
  // Icons: a status shows its own; an amount shows its effect's; an effect
  // with no number of its own (summon, revive, silence) shows it on the verb;
  // a target shows its own. A trigger's icon is on its pill instead.
  let ic = undefined as ReturnType<typeof termIcon>;
  if (!inPill) {
    if (group === "status" || group === "target") ic = info.icon;
    else if (group === "effect") ic = seg.amount || !withAmount.has(id) ? info.icon : undefined;
  }
  if (ic) b.append(icon(ic, size));
  b.append(...kids);
  b.addEventListener("click", (e) => {
    e.stopPropagation();
    hideTip();
    openTermSheet(info);
  });
  b.addEventListener("pointerenter", (e) => e.pointerType === "mouse" && queueTip(b, info));
  b.addEventListener("pointerleave", hideTip);
  b.addEventListener("focus", () => matchMedia("(hover: hover)").matches && queueTip(b, info, 0));
  b.addEventListener("blur", hideTip);
  return b;
}

/** The rule as a block: icon, name, the one-line tip. */
function ruleBlock(info: TermDef & { id: TermId }, size: number): HTMLElement {
  return h(
    "div",
    { class: "term-rule" },
    h("div", { class: `term-head tone-${info.tone}` }, info.icon ? icon(info.icon, size) : null, h("b", {}, info.label)),
    h("div", { class: "term-tip", "data-testid": "term-tip" }, info.tip),
  );
}

const codexButton = (id: TermId): HTMLElement | null => {
  if (!codexLink) return null;
  const open = codexLink;
  const b = h("button", { type: "button", class: "small link", "data-testid": "term-codex" }, "Open in Codex ▸");
  b.addEventListener("click", () => open(id));
  return b;
};

/** A phone's tap (and a desktop's click): the rule in a small sheet. */
export function openTermSheet(info: TermDef & { id: TermId }): () => void {
  const kids: Node[] = [ruleBlock(info, 28)];
  if (info.more) kids.push(h("div", { class: "dim small" }, info.more));
  const codex = codexButton(info.id);
  if (codex) kids.push(codex);
  const close = closable(h("div", { class: "stack term-sheet", "data-testid": "term-sheet" }, ...kids));
  return close;
}

// ---------- the desktop tooltip ----------

let tip: HTMLElement | null = null;
let timer: ReturnType<typeof setTimeout> | undefined;

function queueTip(anchor: HTMLElement, info: TermDef & { id: TermId }, delay = 250): void {
  clearTimeout(timer);
  timer = setTimeout(() => showTip(anchor, info), delay);
}

function showTip(anchor: HTMLElement, info: TermDef & { id: TermId }): void {
  if (!anchor.isConnected) return;
  hideTip();
  const codex = codexLink ? h("div", { class: "dim small" }, "Click for more · Open in Codex") : null;
  tip = h("div", { class: "term-tooltip", role: "tooltip", "data-testid": "term-tooltip" }, ruleBlock(info, 20), codex);
  document.body.append(tip);
  const r = anchor.getBoundingClientRect();
  const w = tip.offsetWidth;
  const left = Math.max(8, Math.min(r.left + r.width / 2 - w / 2, innerWidth - w - 8));
  const below = r.bottom + 6 + tip.offsetHeight < innerHeight;
  tip.style.left = `${left + scrollX}px`;
  tip.style.top = `${(below ? r.bottom + 6 : r.top - 6 - tip.offsetHeight) + scrollY}px`;
}

function hideTip(): void {
  clearTimeout(timer);
  tip?.remove();
  tip = null;
}
addEventListener("scroll", hideTip, { passive: true });

// ---------- what changes, over the pieces ----------

/** Per piece of `next`, its text with the words not in `was` underlined
 * (changedTokens). Feed the result to richText(next, { content }). */
export function markChangedPieces(was: DescribeSegment[], next: DescribeSegment[]): (Node | string)[][] {
  const out: (Node | string)[][] = next.map(() => []);
  for (const t of changedTokens(was, next)) out[t.piece]!.push(t.changed ? h("u", { class: "changed" }, t.tok) : t.tok);
  return out;
}
