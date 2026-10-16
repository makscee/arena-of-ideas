// The piece (mission #1286, slice 1, makscee/void-board#1291): one unit drawn
// the same way on every screen, at four sizes. A big emoji on a rim in its
// side's colour, PWR and HP on the rim, the when tab on top, one headline
// badge (with a counter ring when the unit builds one), statuses on a shelf
// hanging below, ★ / ✦ when awoken / fused, the name under it (L, M) and the
// tier with copies (L). The design: r/arena-redesign/final/ (its P()).
//
//   L  · shop 800      everything
//   M  · battle 800    − tier, − copies
//   S  · phone battle  − name, − when word (its icon stays)
//   XS · chips         the emoji on a rim in the when colour, the headline icon
import { faceFallback, fusedFace, type Face } from "../../src/mvp/face";
import type { LineUnit, MvpContent, UnitContent } from "../../src/mvp/contract";
import type { Stats } from "../../src/types";
import { h } from "./dom";
import { roman } from "./roman";

export type PieceSize = "L" | "M" | "S" | "XS";

/** The face box's side, px, per size. */
export const PIECE_PX: Record<PieceSize, number> = { L: 68, M: 64, S: 56, XS: 36 };

/** What a piece needs to draw; a pool unit, a line unit and a battle unit all fit. */
export interface PieceUnit {
  name: string;
  emoji: string;
  stats: Stats;
  face: Face;
  tier?: number;
  awoken?: boolean;
  /** A fusion: its second part's emoji sits small at the bottom right. */
  fused?: { emoji: string };
  copies?: number;
}

export interface PieceOptions {
  size: PieceSize;
  /** Whose rim: yours, theirs, or the shop's neutral one. */
  side?: "you" | "them" | "shop";
  /** Statuses on it, by status name, in order. A status the counter ring shows stays off the shelf. */
  statuses?: { status: string; stacks: number }[];
  /** The counter's stacks now (the ring's fill). */
  counter?: number;
  /** Keeps the shelf's height when there are no statuses (a battle row stays level). */
  reserve?: boolean;
  /** An offer's price, in place of the copies (L). */
  price?: number;
  lock?: boolean;
  /** Overrides the face box's side (the anatomy's hero, the detail sheet). */
  px?: number;
  testid?: string;
}

/** The when tab's colour, by its icon's family; a new icon gets the plain one. */
const WHEN_TONE: Record<string, string> = {
  "🌅": "#ffd36a", "🕯️": "#ffd36a",
  "🌙": "#a3a8f7",
  "🚩": "#f7a184",
  "⚔️": "#f2a65a",
  "💔": "#f5a3b0",
  "💀": "#d3cbe6",
  "♾️": "#7fdcc0", "🐺": "#7fdcc0",
  "🧪": "#c8b4f0", "📜": "#c8b4f0",
  "💚": "#9fe3b4",
  "🛡️": "#9cc7f2",
  "💪": "#f7c08a",
  "🥚": "#9fe3d8",
};
const WHEN_PLAIN = "#d3cbe6";

/** The headline badge's colour comes from its icon's family. */
const FAMILY: Record<string, string> = { shield: "#5aa0e8", hex: "#d65db1", rule: "#3fbfd6", str: "#f0953e", dmg: "#e2546b", summon: "#9a6be0", heal: "#47b96f" };
const HEAD_FAMILY: Record<string, keyof typeof FAMILY> = {
  "🛡️": "shield",
  "💧": "hex", "❄️": "hex", "🔻": "hex", "🤫": "hex",
  "🔁": "rule", "📜": "rule", "🔀": "rule", "🪞": "rule",
  "💪": "str",
  "💥": "dmg",
  "🐲": "summon", "🦋": "summon", "🥚": "summon",
  "💚": "heal", "😇": "heal", "❤️": "heal", "🪦": "heal",
};
const HEAD_PLAIN = "#8b7fb0";

export const whenTone = (icon: string): string => WHEN_TONE[icon] ?? WHEN_PLAIN;
export const headTone = (icon: string): string => {
  const f = HEAD_FAMILY[icon];
  return f ? FAMILY[f]! : HEAD_PLAIN;
};

/** A status chip's emoji and whether it helps its holder. */
const STATUS: Record<string, [string, "good" | "bad"]> = {
  Shield: ["🛡️", "good"],
  Strength: ["💪", "good"],
  Vitality: ["❤️", "good"],
  Blessing: ["😇", "good"],
  Charge: ["⚡", "good"],
  Poison: ["💧", "bad"],
  Curse: ["🔻", "bad"],
  Freeze: ["❄️", "bad"],
  Silence: ["🤫", "bad"],
};
export const statusEmoji = (status: string): [string, "good" | "bad"] => STATUS[status] ?? ["✨", "good"];

/** A unit's face: its own, else its trigger's fallback. */
export function faceOfUnit(u: Pick<UnitContent, "face" | "forms">): Face {
  return u.face ?? faceFallback(u.forms.sleeping);
}

/** A line unit's face: a fusion shows its first part's when and headline. */
export function faceOfLine(u: Pick<LineUnit, "unitId" | "recipe" | "fusion">, content: Pick<MvpContent, "units">): Face {
  const first = content.units.find((x) => x.id === (u.fusion?.first ?? u.unitId));
  if (!first) return faceFallback(u.recipe);
  return u.fusion ? fusedFace(faceOfUnit(first)) : faceOfUnit(first);
}

/** A line unit as a piece unit (the shop, the battle viewer). */
export function lineToPiece(u: LineUnit, content: Pick<MvpContent, "units">): PieceUnit {
  const unit = content.units.find((x) => x.id === u.unitId);
  const second = u.fusion ? content.units.find((x) => x.id === u.fusion!.second) : undefined;
  return {
    name: u.name,
    emoji: u.emoji,
    stats: u.stats,
    face: faceOfLine(u, content),
    ...(unit ? { tier: unit.tier } : {}),
    awoken: u.kind !== "fused" && u.form === "awoken",
    ...(u.kind === "fused" ? { fused: { emoji: second?.emoji ?? "✨" } } : {}),
    copies: u.copies,
  };
}

const px = (n: number) => `${Math.round(n)}px`;
const css = (o: Record<string, string | number>) =>
  Object.entries(o)
    .map(([k, v]) => `${k}:${typeof v === "number" ? px(v) : v}`)
    .join(";");

function emoji(u: PieceUnit, s: number): HTMLElement {
  if (!u.fused) return h("span", { class: "pc-em", style: `font-size:${px(s)}` }, u.emoji);
  return h(
    "span",
    { class: "pc-em2" },
    h("span", { class: "pc-em a", style: `font-size:${px(s * 0.72)}` }, u.emoji),
    h("span", { class: "pc-em b", style: `font-size:${px(s * 0.58)}` }, u.fused.emoji),
  );
}

function chips(list: { status: string; stacks: number }[], cz: number): HTMLElement[] {
  return list.map(({ status, stacks }) => {
    const [ic, tone] = statusEmoji(status);
    const fs = Math.round(cz * 0.6);
    return h("div", { class: `pc-st ${tone}`, "data-status": status, style: css({ height: cz, "font-size": fs }) }, ic, h("i", { style: `font-size:${px(fs)}` }, String(stacks)));
  });
}

/** One piece. Its box is the face's square; the tab, rim numbers, shelf and
 * name hang outside it, so the caller leaves room (pieceBox gives how much). */
export function piece(u: PieceUnit, o: PieceOptions): HTMLElement {
  const s = o.px ?? PIECE_PX[o.size];
  const face = u.face;
  const side = o.side ?? "you";
  const cls = ["pc", `pc-${o.size}`, `side-${side}`, u.awoken ? "aw" : "", u.fused ? "fu" : ""].filter(Boolean).join(" ");
  const attrs: Record<string, string> = { class: cls, style: css({ width: s, height: s }), "data-name": u.name };
  if (o.testid) attrs["data-testid"] = o.testid;
  const root = h("div", attrs);
  const when = whenTone(face.when.icon);

  if (o.size === "XS") {
    const fx = Math.round(s * 0.5);
    root.append(
      h("div", { class: "pc-face", "data-k": "face", style: `border-width:2.5px;box-shadow:inset 0 0 0 4px ${when},0 2px 0 var(--outline)` }, emoji(u, s * 0.6)),
      h(
        "div",
        { class: "pc-hb", "data-k": "head", style: css({ left: s - fx * 0.62, top: s - fx * 0.62, width: fx, height: fx }) },
        h("div", { class: "core", style: `background:${headTone(face.headline.icon)};border-width:2px;font-size:${px(fx * 0.58)}` }, face.headline.icon),
      ),
    );
    return root;
  }

  const showWord = o.size === "L" || o.size === "M" || o.px !== undefined;
  const showName = o.size === "L" || o.size === "M";
  const showMeta = o.size === "L";
  const tabH = Math.round(Math.max(20, s * 0.27));
  const tf = Math.round(Math.max(12, s * 0.17));
  const ti = Math.round(Math.max(13, s * 0.19));
  const d = Math.round(Math.max(22, s * 0.37));
  const sf = Math.round(Math.max(14, d * 0.6));
  const hb = Math.round(Math.max(23, s * 0.42));
  const big = s >= 120;

  // the face, on a rim in its side's colour (awoken lilac, fused gold and pink)
  const inset = Math.max(4, Math.round(s * 0.07));
  const rim = u.awoken ? "var(--awoken)" : side === "them" ? "var(--them)" : side === "shop" ? "#8b7fb0" : "var(--you)";
  const shadow = u.fused ? `inset 0 0 0 ${inset}px #f0c24b,inset 0 0 0 ${inset * 2}px #d65db1` : `inset 0 0 0 ${inset}px ${rim},inset 0 0 0 ${inset + 2}px #120e1c55`;
  root.append(h("div", { class: "pc-face", "data-k": "face", style: `border-width:${big ? 4 : 3}px;box-shadow:${shadow},0 ${big ? 5 : 3}px 0 var(--outline)` }, emoji(u, s * 0.64)));

  // the when tab
  const pad = Math.round(tabH * 0.32);
  root.append(
    h(
      "div",
      { class: "pc-tab", "data-k": "tab", title: face.when.label, style: css({ top: -Math.round(tabH * 0.62), height: tabH, padding: `0 ${showWord ? pad + 3 : pad}px 0 ${pad}px`, background: when, "border-radius": Math.round(tabH * 0.45) }) },
      h("span", { class: "ti", style: `font-size:${px(ti)}` }, face.when.icon),
      showWord ? h("span", { class: "tw", style: `font-size:${px(tf)}` }, face.when.label) : null,
    ),
  );

  // the headline badge, in a counter ring when the unit builds one
  const ring = face.counter;
  const rw = ring ? Math.max(4, Math.round(hb * 0.15)) : 0;
  const tot = hb + rw * 2;
  const hx = s - Math.round(tot * 0.58);
  const hy = Math.round(tabH * 0.42) + 2 - rw;
  const ringStyle = ring ? `--fill:${((Math.min(o.counter ?? 0, ring.max) / ring.max) * 100).toFixed(1)}%;--seg:${360 / ring.max}deg;padding:${rw}px;` : "";
  root.append(
    h(
      "div",
      { class: ring ? "pc-hb ring" : "pc-hb", "data-k": ring ? "ring" : "head", ...(ring ? { title: `${ring.status} ${o.counter ?? 0}/${ring.max}` } : {}), style: `${css({ left: hx, top: hy, width: tot, height: tot })};${ringStyle}` },
      h("div", { class: "core", style: `background:${headTone(face.headline.icon)};font-size:${px(hb * 0.52)};box-shadow:inset 0 -${Math.max(2, Math.round(hb * 0.08))}px 0 #0003` }, face.headline.icon),
    ),
  );
  if (face.headline.amount) {
    const af = Math.round(Math.max(12, hb * 0.42));
    const ah = Math.round(af * 1.3);
    root.append(h("div", { class: "pc-num", "data-k": "num", style: css({ left: hx + tot - Math.round(ah * 0.7), top: hy + tot - Math.round(ah * 0.75), "font-size": af, height: ah, "min-width": ah }) }, face.headline.amount));
  }

  // PWR and HP on the rim
  const sy = s - Math.round(d * 0.7);
  root.append(
    h("div", { class: "pc-b p", "data-k": "pwr", style: css({ left: -Math.round(d * 0.28), top: sy, width: d, height: d, "font-size": sf }) }, String(u.stats.pwr)),
    h("div", { class: "pc-b h", "data-k": "hp", style: css({ left: s - d + Math.round(d * 0.28), top: sy, width: d, height: d, "font-size": sf }) }, String(u.stats.hp)),
  );

  // ★ awoken, ✦ fused: the left edge, above PWR
  if (u.awoken || u.fused) {
    const mz = Math.round(Math.max(16, s * 0.25));
    root.append(h("span", { class: u.fused ? "pc-mark fuse" : "pc-mark star", "data-k": "star", style: css({ left: -Math.round(mz * 0.42), top: Math.round(s * 0.2), "font-size": mz }) }, u.fused ? "✦" : "★"));
  }

  // below: the status shelf, the name, tier and copies
  let y = sy + d;
  const shelf = (o.statuses ?? []).filter((x) => x.stacks > 0 && x.status !== ring?.status);
  if (shelf.length || o.reserve) {
    const cz = big ? 30 : 24;
    if (shelf.length) {
      const top = s - 8;
      const padB = 4;
      root.append(h("div", { class: "pc-shelf", "data-k": "chips", style: css({ top, height: y + 3 - top + cz + padB, padding: `0 6px ${padB}px` }) }, ...chips(shelf, cz)));
    }
    y += 3 + cz + 4;
  }
  if (showName) {
    const nf = Math.round(Math.max(12, s * 0.2));
    root.append(h("div", { class: "pc-nm", "data-k": "name", style: css({ top: y + 3, "font-size": nf }) }, u.name));
    y += 3 + nf;
  }
  if (showMeta) {
    const meta = h("div", { class: "pc-meta", style: css({ top: y + 6 }) });
    if (u.tier) meta.append(h("div", { class: `pc-tier pc-t${u.tier}`, "data-k": "tier" }, roman(u.tier)));
    if (o.price != null) meta.append(h("div", { class: "pc-price" }, h("span", { class: "coin" }), String(o.price)));
    else if (u.awoken) meta.append(h("span", { class: "pc-flag star" }, "★"));
    else if (u.fused) meta.append(h("span", { class: "pc-flag fuse" }, "✦"));
    else {
      const pips = h("div", { class: "pc-pips", "data-k": "copies" });
      const c = u.copies ?? 1;
      for (let i = 0; i < 3; i++) pips.append(h("u", i < c ? {} : { class: "o" }));
      meta.append(pips);
    }
    if (o.lock) meta.append(h("span", { class: "pc-lock" }, "🔒"));
    root.append(meta);
  }
  return root;
}
