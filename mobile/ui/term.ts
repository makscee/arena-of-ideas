// Unit text with its terms highlighted (R2-8, docs/round2/icons.md "How the
// client renders it"). richText() draws describe.ts's tagged pieces: a trigger
// clause is one amber pill with its icon, a status is its icon plus its word in
// its colour, a target is tinted teal (ally) or pink (enemy), and an amount is
// its effect's icon plus a bold number. Every term is a button: hovering it on
// a desktop shows its rule in a tooltip; tapping it (or Enter) opens a small
// sheet with the rule on a phone, and on a desktop (1024px and wider) a
// popover pinned to the term (R2-17), which Esc or a click outside closes.
// Both offer "Open in Codex" once something registers a Codex link (slice
// R2-11, setCodexLink).
import type { DescribeSegment } from "../../src/describe";
import { scopedLabel, scopedTip, termDef, termIcon, type TermDef, type TermId } from "../../src/glossary";
import type { UnitFilter } from "../../src/types";
import { closable, h, isDesktop, onPopoverEsc } from "./dom";
import { changedTokens } from "./diff";
import { icon } from "./icon";
import { withPip } from "./card";
import { scopePip } from "../../src/mvp/card-icons";

/** Opens a term's Codex entry; set by the Codex (R2-11). Unset: no link. */
let codexLink: ((id: TermId, scope?: UnitFilter) => void) | null = null;
export function setCodexLink(open: ((id: TermId, scope?: UnitFilter) => void) | null): void {
  codexLink = open;
}

/** What a run's tooltip and sheet say. An eventUnit target follows the words on
 * screen ("self", "it"), not the selector's generic label. */
export function termInfo(seg: DescribeSegment): (TermDef & { id: TermId; scope?: UnitFilter }) | undefined {
  const id = seg.term;
  if (!id) return undefined;
  const def = termDef(id);
  if (!def) return undefined;
  if (id === "target:eventUnit") {
    const label = seg.text.charAt(0).toUpperCase() + seg.text.slice(1);
    const holder = termDef("target:holder")!;
    return /^self$/i.test(seg.text) ? { ...holder, id, label } : { ...def, id, label };
  }
  // A trigger clause's rule follows its scope ("after an enemy dies"), and so does its Codex line.
  return seg.scope ? { ...def, id, scope: seg.scope, label: scopedLabel(id, seg.scope) ?? def.label, tip: scopedTip(id, seg.scope) ?? def.tip } : { ...def, id };
}

/** The status a run is about: its own status term, or the status a "gets"
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
    // "1 damage" never wraps between the number and its word.
    const kids = segs[i - 1]?.amount && seg.text === " " ? ["\u00a0"] : (o.content?.[i] ?? [seg.text]);
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
      // The pip says whose event it is, as on the card's icon line (R3-4).
      if (ic) pill.append(withPip(icon(ic, size, "tpill-ic"), scopePip(clause.find((s) => s.scope)?.scope)));
      out.push(pill);
    }
    const info = clauseInfo(seg, segs, i);
    const nodes: Node[] = info ? edgeSpaced(kids, (inner) => termButton(seg, info, inner, withAmount, size, !!pill)) : kids.map((k) => (typeof k === "string" ? document.createTextNode(k) : k));
    if (pill) pill.append(...nodes);
    else out.push(...nodes);
  });
  return glueStops(out);
}

/** A term is a button, and a line may break after a button: "self" then
 * a lone "." on the next line. The punctuation that follows a term goes into
 * one unbreakable box with it. */
function glueStops(nodes: Node[]): Node[] {
  const out: Node[] = [];
  for (const n of nodes) {
    const prev = out[out.length - 1];
    const stop = n instanceof Text ? /^[.,;:!?)]+/.exec(n.data)?.[0] : undefined;
    if (stop && prev instanceof HTMLElement && prev.matches("button.t")) {
      out[out.length - 1] = h("span", { class: "t-glue" }, prev, stop);
      const rest = (n as Text).data.slice(stop.length);
      if (rest) out.push(document.createTextNode(rest));
    } else out.push(n);
  }
  return out;
}

/** A button drops the spaces at its edges ("After " + "Shield" read
 * "AfterShield"), so they go outside it as text. */
function edgeSpaced(kids: (Node | string)[], wrap: (inner: (Node | string)[]) => Node): Node[] {
  const inner = [...kids];
  const lead = typeof inner[0] === "string" ? /^\s*/.exec(inner[0])![0] : "";
  if (lead) inner[0] = (inner[0] as string).slice(lead.length);
  const last = inner.length - 1;
  const trail = typeof inner[last] === "string" ? /\s*$/.exec(inner[last] as string)![0] : "";
  if (trail) inner[last] = (inner[last] as string).slice(0, -trail.length);
  const body = inner.filter((k) => k !== "");
  return [...(lead ? [document.createTextNode(lead)] : []), wrap(body), ...(trail ? [document.createTextNode(trail)] : [])];
}

/** termInfo, plus: a "gets" trigger names its status ("Gets Shield", "Ally gets Shield"). */
function clauseInfo(seg: DescribeSegment, segs: DescribeSegment[], i: number): ReturnType<typeof termInfo> {
  const info = termInfo(seg);
  if (!info || (info.id !== "trigger:StatusApplied" && info.id !== "trigger:StatusRemoved")) return info;
  let j = i;
  while (j > 0 && segs[j - 1]!.clause === "when") j--;
  let status: string | undefined;
  for (; j < segs.length && segs[j]!.clause === "when" && !status; j++) status = statusOf(segs[j]!);
  // "Gets status" → "Gets Shield"; scoped: "Ally gets Shield".
  return status ? { ...info, label: info.label.replace(/status$/, status) } : info;
}

function termButton(seg: DescribeSegment, info: TermDef & { id: TermId }, kids: (Node | string)[], withAmount: Set<TermId>, size: number, inPill: boolean): HTMLElement {
  const id = info.id;
  const group = id.slice(0, id.indexOf(":"));
  const b = h("button", { type: "button", class: `t t-${group} ${toneOf(seg, info)}`, "data-term": id, "data-testid": "term", "aria-label": `${seg.text}: ${info.tip}` });
  if (seg.amount) b.classList.add("t-amount");
  // Icons, one per idea: an amount shows its term's (a damage number its
  // effect's, a stack count its status's, "2 HP" HP's), and then its word is
  // only coloured; a status, stat or effect with no number of its own shows
  // it on the word; a target shows its own. A trigger's icon is on its pill.
  let ic = undefined as ReturnType<typeof termIcon>;
  if (!inPill) {
    if (group === "target") ic = info.icon;
    else if (group === "status" || group === "stat" || group === "effect") ic = seg.amount || !withAmount.has(id) ? info.icon : undefined;
  }
  if (ic) b.append(icon(ic, size));
  b.append(...kids);
  b.addEventListener("click", (e) => {
    e.stopPropagation();
    hideTip();
    if (isDesktop()) openTermPopover(b, info);
    else openTermSheet(info);
  });
  // A term that a click just drew under a still cursor (Awoken's text
  // swapped in) waits for the mouse to move before its tip covers anything
  // (R2-17 batch E: it hid "Back to sleeping").
  let waiting = false;
  b.addEventListener("pointerenter", (e) => {
    if (e.pointerType !== "mouse") return;
    if (stillSincePress(e)) waiting = true;
    else queueTip(b, info);
  });
  b.addEventListener("pointermove", (e) => {
    if (!waiting || e.pointerType !== "mouse" || stillSincePress(e)) return;
    waiting = false;
    queueTip(b, info);
  });
  b.addEventListener("pointerleave", () => (waiting = false));
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

const codexButton = (id: TermId, scope?: UnitFilter): HTMLElement | null => {
  if (!codexLink) return null;
  const open = codexLink;
  const b = h("button", { type: "button", class: "small link", "data-testid": "term-codex" }, "Open in Codex ▸");
  b.addEventListener("click", () => open(id, scope));
  return b;
};

/** A phone's tap (and a desktop's click): the rule in a small sheet. */
export function openTermSheet(info: TermDef & { id: TermId; scope?: UnitFilter }): () => void {
  const kids: Node[] = [ruleBlock(info, 28)];
  if (info.more) kids.push(h("div", { class: "dim small" }, info.more));
  const codex = codexButton(info.id, info.scope);
  if (codex) kids.push(codex);
  const close = closable(h("div", { class: "stack term-sheet", "data-testid": "term-sheet" }, ...kids));
  return close;
}

// ---------- the desktop popover ----------

let pop: HTMLElement | null = null;
let popAnchor: HTMLElement | null = null;

/** A desktop's click: the rule in a popover under (or over) the term, with
 * "Open in Codex"; Esc, a click outside, a scroll or the term leaving the
 * page closes it. A second click on the same term closes it too. */
export function openTermPopover(anchor: HTMLElement, info: TermDef & { id: TermId; scope?: UnitFilter }): void {
  const again = popAnchor === anchor;
  closePopover();
  if (again) return;
  const kids: Node[] = [ruleBlock(info, 24)];
  if (info.more) kids.push(h("div", { class: "dim small" }, info.more));
  const codex = codexButton(info.id, info.scope);
  if (codex) {
    codex.addEventListener("click", closePopover);
    kids.push(codex);
  }
  pop = h("div", { class: "stack term-pop", role: "dialog", "aria-label": info.label, "data-testid": "term-popover" }, h("div", { class: "stack term-sheet", "data-testid": "term-sheet" }, ...kids));
  document.body.append(pop);
  popAnchor = anchor;
  place(pop, anchor);
  requestAnimationFrame(watchPopover);
}

export function closePopover(): void {
  pop?.remove();
  pop = null;
  popAnchor = null;
}

function watchPopover(): void {
  if (!pop) return;
  if (!popAnchor?.isConnected) return closePopover();
  requestAnimationFrame(watchPopover);
}

/** Under the anchor when it fits, else over it; never past the window's sides. */
function place(el: HTMLElement, anchor: HTMLElement): void {
  const r = anchor.getBoundingClientRect();
  const w = el.offsetWidth;
  const left = Math.max(8, Math.min(r.left + r.width / 2 - w / 2, innerWidth - w - 8));
  const below = r.bottom + 6 + el.offsetHeight < innerHeight;
  el.style.left = `${left + scrollX}px`;
  el.style.top = `${(below ? r.bottom + 6 : Math.max(8, r.top - 6 - el.offsetHeight)) + scrollY}px`;
}

// Esc closes the popover only: the sheet or the screen under it (the shop's menu) never sees it.
onPopoverEsc(() => (hideTip(), pop ? closePopover : null));
addEventListener("pointerdown", (e) => pop && !pop.contains(e.target as Node) && e.target !== popAnchor && !popAnchor?.contains(e.target as Node) && closePopover(), { capture: true });
addEventListener("scroll", (e) => pop && !pop.contains(e.target as Node) && closePopover(), { capture: true, passive: true });

// ---------- the desktop tooltip ----------

let tip: HTMLElement | null = null;
let tipAnchor: HTMLElement | null = null;
let timer: ReturnType<typeof setTimeout> | undefined;

/** A screen re-render (a key, a battle beat) can drop the anchor without a
 * pointerleave: the tooltip goes with it, checked each frame while it shows. */
function watchAnchor(): void {
  if (!tip) return;
  if (!tipAnchor?.isConnected) return hideTip();
  requestAnimationFrame(watchAnchor);
}

function queueTip(anchor: HTMLElement, info: TermDef & { id: TermId }, delay = 250): void {
  clearTimeout(timer);
  timer = setTimeout(() => showTip(anchor, info), delay);
}

function showTip(anchor: HTMLElement, info: TermDef & { id: TermId }): void {
  // The popover already says it all.
  if (!anchor.isConnected || popAnchor === anchor) return;
  hideTip();
  const codex = codexLink ? h("div", { class: "dim small" }, "Click for more · Open in Codex") : null;
  tip = h("div", { class: "term-tooltip", role: "tooltip", "data-testid": "term-tooltip" }, ruleBlock(info, 20), codex);
  document.body.append(tip);
  tipAnchor = anchor;
  requestAnimationFrame(watchAnchor);
  place(tip, anchor);
}

function hideTip(): void {
  clearTimeout(timer);
  tip?.remove();
  tip = null;
  tipAnchor = null;
}
addEventListener("scroll", hideTip, { passive: true });
addEventListener("keydown", hideTip, { capture: true });

/** Where the mouse last pressed, until it moves away: a term that turns up
 * under it then is the click's doing, not a hover. */
let pressAt: { x: number; y: number } | null = null;
addEventListener("pointerdown", (e) => { if (e.pointerType === "mouse") pressAt = { x: e.clientX, y: e.clientY }; }, { capture: true, passive: true });
const stillSincePress = (e: PointerEvent) => pressAt !== null && Math.abs(e.clientX - pressAt.x) <= 3 && Math.abs(e.clientY - pressAt.y) <= 3;
addEventListener("pointermove", (e) => { if (pressAt && !stillSincePress(e)) pressAt = null; }, { capture: true, passive: true });

// ---------- what changes, over the pieces ----------

/** Per piece of `next`, its text with the words not in `was` underlined
 * (changedTokens). Feed the result to richText(next, { content }). */
export function markChangedPieces(was: DescribeSegment[], next: DescribeSegment[]): (Node | string)[][] {
  const out: (Node | string)[][] = next.map(() => []);
  for (const t of changedTokens(was, next)) out[t.piece]!.push(t.changed ? h("u", { class: "changed" }, t.tok) : t.tok);
  return out;
}
