// Share cards (M4-7, mission #810): PNGs to share, in the game's look, made
// on the server from an SVG template rendered by resvg (no browser). 1200×630,
// the size link previews show. GET /share/champion/<day>.png is a day's
// champion and their team; GET /share/unit/<id>.png one unit's card big.
// The fonts are the client's (Chakra Petch, Rajdhani, IBM Plex Mono) plus
// Play for Cyrillic, which the first two lack, from server/assets/fonts; the
// emoji are Twemoji's SVGs (@twemoji/svg), so a card looks the same on any
// host. Reads only.
import { createHash } from "node:crypto";
import { existsSync, readFileSync } from "node:fs";
import { createRequire } from "node:module";
import { dirname, resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { Resvg } from "@resvg/resvg-js";
import type { Champion, LineUnit, PlayerRef, UnitContent } from "../../../src/mvp/contract.js";
import { formSegments } from "../../../src/mvp/form-text.js";
import type { AbilityRegistry } from "../../../src/types.js";
import { creditsView, libraryView } from "./credits.js";
import { championOf, dayView } from "./day.js";
import type { MvpRuntime } from "./runtime.js";

export const SHARE_W = 1200;
export const SHARE_H = 630;
/** Where a card sends people (and the page Open Graph tags point at). */
export const PUBLIC_URL = "https://arena.makscee.ru/arena";

export type ShareLang = "en" | "ru";
type ShareDeps = Pick<MvpRuntime, "store" | "content" | "today">;

/** `?lang=ru` wins; else an Accept-Language starting "ru"; else English. */
export function shareLang(lang: string | undefined, acceptLanguage?: string): ShareLang {
  if (lang === "ru" || lang === "en") return lang;
  return /^\s*ru\b/i.test(acceptLanguage ?? "") ? "ru" : "en";
}

// ---------- words (Russian names come with M4-4; until then the English) ----------

const MONTHS: Record<ShareLang, string[]> = {
  en: ["Jan", "Feb", "Mar", "Apr", "May", "Jun", "Jul", "Aug", "Sep", "Oct", "Nov", "Dec"],
  ru: ["января", "февраля", "марта", "апреля", "мая", "июня", "июля", "августа", "сентября", "октября", "ноября", "декабря"],
};
/** "Oct 8, 2026" / "8 октября 2026" from a YYYY-MM-DD label. */
export function dateText(day: string, lang: ShareLang): string {
  const m = /^(\d{4})-(\d{2})-(\d{2})$/.exec(day);
  if (!m) return day;
  const [y, mo, d] = [m[1]!, MONTHS[lang][Number(m[2]) - 1] ?? m[2]!, String(Number(m[3]))];
  return lang === "ru" ? `${d} ${mo} ${y}` : `${mo} ${d}, ${y}`;
}
const WORDS = {
  en: {
    game: "ARENA OF IDEAS",
    championOf: (day: string) => `Champion of ${day}`,
    slayers: (n: number) => `${n} ${n === 1 ? "slayer" : "slayers"}`,
    slewYesterday: (n: number) => `${n} slew yesterday`,
    canYou: "Can you beat them?",
    awoken: "AWOKEN",
    fused: "FUSED",
    isNew: "NEW",
    sleeping: "Sleeping",
    awokenForm: "Awoken",
    ideaBy: (p: string) => `idea by @${p}`,
    evolvedBy: (p: string) => `evolved by @${p}`,
    seed: "one of the first units",
    tier: "Tier",
    left: "in the Library",
    unitTitle: (name: string) => `${name}: a unit of Arena of Ideas`,
  },
  ru: {
    game: "ARENA OF IDEAS",
    championOf: (day: string) => `Чемпион ${day}`,
    slayers: (n: number) => `Сразили: ${n}`,
    slewYesterday: (n: number) => `Сразили вчера: ${n}`,
    canYou: "Сможешь победить?",
    awoken: "ПРОБУЖДЁН",
    fused: "СЛИЯНИЕ",
    isNew: "НОВЫЙ",
    sleeping: "Спит",
    awokenForm: "Пробуждён",
    ideaBy: (p: string) => `идея @${p}`,
    evolvedBy: (p: string) => `развил @${p}`,
    seed: "из первых юнитов",
    tier: "Уровень",
    left: "в Библиотеке",
    unitTitle: (name: string) => `${name}: юнит Arena of Ideas`,
  },
} satisfies Record<ShareLang, unknown>;

// ---------- the look (mobile/style.css's "B · Arena") ----------

const C = { bg: "#07080c", panel: "#10131b", line: "#1f2533", text: "#e8ecf4", dim: "#8a93a6", you: "#25e6d4", ghost: "#ff3d7f", gold: "#ffcc4d" };
const TIER_COLORS = ["#a8b3c7", "#3fdc8f", "#5aa7ff", "#c08cff"];
const ROMAN = ["I", "II", "III", "IV"];
type Font = { family: string; weight: number };
const DISPLAY: Font = { family: "Chakra Petch, Play", weight: 700 };
const BODY: Font = { family: "Rajdhani, Play", weight: 600 };
const MONO: Font = { family: "IBM Plex Mono, Play", weight: 500 };

const here = dirname(fileURLToPath(import.meta.url));
const FONT_DIR = resolve(here, "../../assets/fonts");
const FONT_FILES = ["ChakraPetch-Bold.ttf", "Rajdhani-SemiBold.ttf", "IBMPlexMono-Medium.ttf", "Play-Bold.ttf", "Play-Regular.ttf"].map((f) => resolve(FONT_DIR, f));
const RESVG_FONT = { fontFiles: FONT_FILES, loadSystemFonts: false, defaultFontFamily: "Play" };
const TWEMOJI_DIR = dirname(createRequire(import.meta.url).resolve("@twemoji/svg/package.json"));

const esc = (s: string) => s.replace(/&/g, "&amp;").replace(/</g, "&lt;").replace(/>/g, "&gt;").replace(/"/g, "&quot;");

/** Chakra Petch and Rajdhani have no Cyrillic, and resvg's own fallback picks
 * the mono face: a text with Cyrillic is set in Play, the whole line alike. */
const CYRILLIC = /[\u0400-\u04ff]/;
function fontFor(s: string, f: Font): Font {
  return CYRILLIC.test(s) && f !== MONO ? { family: "Play", weight: f.weight >= 600 ? 700 : 400 } : f;
}

function textEl(s: string, x: number, y: number, size: number, f: Font, fill: string, extra = ""): string {
  const g = fontFor(s, f);
  return `<text x="${x}" y="${y}" font-family="${g.family}" font-weight="${g.weight}" font-size="${size}" fill="${fill}" ${extra}>${esc(s)}</text>`;
}

const widths = new Map<string, number>();
/** A text's drawn width, measured by resvg itself with the same fonts. */
export function textWidth(s: string, size: number, f: Font): number {
  const key = `${f.family}|${f.weight}|${size}|${s}`;
  let w = widths.get(key);
  if (w === undefined) {
    const svg = `<svg xmlns="http://www.w3.org/2000/svg" width="8000" height="${size * 2}">${textEl(s, 0, size * 1.5, size, f, "#000")}</svg>`;
    const box = new Resvg(svg, { font: RESVG_FONT }).getBBox();
    w = box ? box.x + box.width : 0;
    if (widths.size > 5000) widths.clear();
    widths.set(key, w);
  }
  return w;
}

/** A one-line text that fits `max`: smaller down to `min`, then cut with "…". */
export function fit(s: string, f: Font, max: number, size: number, min = Math.round(size * 0.6)): { text: string; size: number } {
  for (let z = size; z >= min; z -= 2) if (textWidth(s, z, f) <= max) return { text: s, size: z };
  const chars = [...s];
  for (let n = chars.length - 1; n > 0; n--) {
    const t = chars.slice(0, n).join("").trimEnd() + "…";
    if (textWidth(t, min, f) <= max) return { text: t, size: min };
  }
  return { text: "…", size: min };
}

/** Words greedily into lines no wider than `max`, at most `maxLines` (the last cut with "…"). */
export function wrap(s: string, f: Font, size: number, max: number, maxLines: number): string[] {
  const lines: string[] = [];
  let cur = "";
  for (const word of s.split(/\s+/).filter(Boolean)) {
    const next = cur ? `${cur} ${word}` : word;
    if (textWidth(next, size, f) <= max || !cur) cur = next;
    else {
      lines.push(cur);
      cur = word;
    }
  }
  if (cur) lines.push(cur);
  if (lines.length <= maxLines) return lines.map((l) => (textWidth(l, size, f) <= max ? l : fit(l, f, max, size, size).text));
  const kept = lines.slice(0, maxLines);
  kept[maxLines - 1] = fit(`${kept[maxLines - 1]} ${lines.slice(maxLines).join(" ")}`, f, max, size, size).text;
  return kept;
}

// ---------- emoji ----------

const emojiCache = new Map<string, string | null>();
/** One emoji as a data URI of its Twemoji SVG, or null when there is none. */
function twemoji(g: string): string | null {
  if (emojiCache.has(g)) return emojiCache.get(g)!;
  const cps = [...g].map((c) => c.codePointAt(0)!.toString(16));
  let found: string | null = null;
  for (const name of [cps.join("-"), cps.filter((c) => c !== "fe0f").join("-")]) {
    const file = resolve(TWEMOJI_DIR, `${name}.svg`);
    if (name && existsSync(file)) {
      found = `data:image/svg+xml;base64,${readFileSync(file).toString("base64")}`;
      break;
    }
  }
  emojiCache.set(g, found);
  return found;
}
const segmenter = new Intl.Segmenter("en", { granularity: "grapheme" });
/** The emoji in `s` (a fused unit's two), side by side, centred on cx, `size` tall. */
function emojiRow(s: string, cx: number, top: number, size: number): string {
  const gs = [...segmenter.segment(s)].map((x) => x.segment).filter((g) => g.trim());
  const each = gs.length > 1 ? size * 0.72 : size;
  const gap = each * 0.06;
  let x = cx - (gs.length * each + (gs.length - 1) * gap) / 2;
  const y = top + (size - each) / 2;
  return gs
    .map((g) => {
      const uri = twemoji(g);
      const el = uri
        ? `<image href="${uri}" x="${x}" y="${y}" width="${each}" height="${each}"/>`
        : textEl(g, x, y + each * 0.85, each * 0.8, BODY, C.text);
      x += each + gap;
      return el;
    })
    .join("");
}

// ---------- shapes ----------

/** The client's chamfered card: corners cut top-left and bottom-right. */
function chamfer(x: number, y: number, w: number, h: number, k: number, fill: string, stroke: string, sw = 2): string {
  const p = [[x + k, y], [x + w, y], [x + w, y + h - k], [x + w - k, y + h], [x, y + h], [x, y + k]].map(([a, b]) => `${a},${b}`).join(" ");
  return `<polygon points="${p}" fill="${fill}" stroke="${stroke}" stroke-width="${sw}"/>`;
}

function frame(body: string, lang: ShareLang): string {
  return `<svg xmlns="http://www.w3.org/2000/svg" width="${SHARE_W}" height="${SHARE_H}" viewBox="0 0 ${SHARE_W} ${SHARE_H}">
<defs><radialGradient id="glow" cx="0.5" cy="0" r="0.9"><stop offset="0" stop-color="${C.you}" stop-opacity="0.13"/><stop offset="1" stop-color="${C.bg}" stop-opacity="0"/></radialGradient></defs>
<rect width="${SHARE_W}" height="${SHARE_H}" fill="${C.bg}"/><rect width="${SHARE_W}" height="${SHARE_H}" fill="url(#glow)"/>
${chamfer(16, 16, SHARE_W - 32, SHARE_H - 32, 22, "none", C.line, 2)}
${textEl(WORDS[lang].game, 56, 74, 22, DISPLAY, C.you, 'letter-spacing="3"')}
${body}
${textEl(PUBLIC_URL.replace(/^https?:\/\//, "").replace(/\/.*$/, ""), SHARE_W - 56, SHARE_H - 46, 26, MONO, C.you, 'text-anchor="end"')}
</svg>`;
}

/** One unit card: chamfered panel, emoji, name, PWR/HP; gold for Awoken. */
function unitCard(u: { emoji: string; name: string; pwr: number; hp: number; tier?: number; awoken?: boolean; fused?: boolean; isNew?: boolean }, x: number, y: number, w: number, h: number, lang: ShareLang): string {
  const k = Math.round(w * 0.08);
  const s = w / 200; // the champion card's 200 px is the unit
  const border = u.awoken ? C.gold : u.fused ? C.you : C.line;
  const out = [chamfer(x, y, w, h, k, C.panel, border, u.awoken || u.fused ? 3 : 2)];
  if (u.awoken) out.push(`<polygon points="${x + k},${y} ${x + w},${y} ${x + w},${y + h - k} ${x + w - k},${y + h} ${x},${y + h} ${x},${y + k}" fill="${C.gold}" fill-opacity="0.07"/>`);
  if (u.tier) out.push(textEl(ROMAN[u.tier - 1] ?? "", x + w - 14 * s, y + 30 * s, 20 * s, { family: "Play", weight: 700 }, TIER_COLORS[u.tier - 1] ?? C.dim, 'text-anchor="end"'));
  if (u.isNew) {
    const t = WORDS[lang].isNew;
    const tw = textWidth(t, 15 * s, DISPLAY) + 12 * s;
    out.push(`<rect x="${x + 12 * s}" y="${y + 12 * s}" width="${tw}" height="${22 * s}" rx="${3 * s}" fill="${C.gold}"/>`, textEl(t, x + 18 * s, y + 29 * s, 15 * s, DISPLAY, C.bg));
  }
  out.push(emojiRow(u.emoji, x + w / 2, y + 40 * s, 92 * s));
  // One line, a little smaller if need be; a long name takes two lines.
  const name = fit(u.name, BODY, w - 24 * s, 30 * s, 24 * s);
  if (name.text === u.name) out.push(textEl(name.text, x + w / 2, y + 172 * s, name.size, BODY, C.text, 'text-anchor="middle"'));
  else wrap(u.name, BODY, 22 * s, w - 24 * s, 2).forEach((l, i) => out.push(textEl(l, x + w / 2, y + (151 + i * 23) * s, 22 * s, BODY, C.text, 'text-anchor="middle"')));
  const stats = `<text x="${x + w / 2}" y="${y + 210 * s}" font-family="${MONO.family}" font-weight="${MONO.weight}" font-size="${26 * s}" text-anchor="middle"><tspan fill="${C.gold}">${u.pwr}</tspan><tspan fill="${C.dim}">/</tspan><tspan fill="${C.ghost}">${u.hp}</tspan></text>`;
  out.push(stats);
  const mark = u.fused ? WORDS[lang].fused : u.awoken ? WORDS[lang].awoken : null;
  if (mark) out.push(textEl(mark, x + w / 2, y + h - 14 * s, 15 * s, DISPLAY, u.fused ? C.you : C.gold, `text-anchor="middle" letter-spacing="${2 * s}"`));
  return out.join("");
}

// ---------- the two cards ----------

/** Day `seq`'s champion: null when the day has none yet, or hasn't come.
 * `slewYesterday` shows the day before's slayers instead of day `seq`'s (the
 * daily post, M4-8: right after the day end today's count is still 0). */
export function championSvg(rt: ShareDeps, seq: number, lang: ShareLang, slewYesterday?: number): string | null {
  if (seq > rt.today().seq) return null;
  const champ = championOf(rt.store, seq);
  if (!champ) return null;
  const w = WORDS[lang];
  const slayers = seq === rt.today().seq ? dayView(rt).slayers : new Set(rt.store.slays(seq).map((s) => s.player.id)).size;
  const title = fit(w.championOf(dateText(champOfDay(rt, champ, seq), lang)), DISPLAY, SHARE_W - 112, 52, 34);
  const who = fit(`@${champ.player.name}`, BODY, SHARE_W - 112, 60, 36);
  const body = [
    textEl(title.text, 56, 136, title.size, DISPLAY, C.text),
    textEl(who.text, 56, 200, who.size, BODY, C.gold),
    lineCards(champ.line, lang),
    emojiRow("⚔️", 72, SHARE_H - 74, 32),
    textEl(`${slewYesterday === undefined ? w.slayers(slayers) : w.slewYesterday(slewYesterday)} · ${w.canYou}`, 100, SHARE_H - 46, 28, BODY, C.dim),
  ].join("\n");
  return frame(body, lang);
}
/** The date of day `seq` (the champion row may be an earlier day's, kept). */
function champOfDay(rt: ShareDeps, champ: Champion, seq: number): string {
  const t = rt.today();
  if (seq === t.seq) return t.day;
  const exact = rt.store.champion(seq);
  if (exact) return exact.day;
  // A day with no new champion kept the last: its date counts back from today.
  const d = new Date(`${t.day}T12:00:00Z`);
  if (Number.isNaN(d.getTime())) return champ.day;
  d.setUTCDate(d.getUTCDate() - (t.seq - seq));
  return d.toISOString().slice(0, 10);
}

function lineCards(line: LineUnit[], lang: ShareLang): string {
  const units = line.slice(0, 5);
  const cw = 200, ch = 250, gap = 24;
  const total = units.length * cw + Math.max(0, units.length - 1) * gap;
  let x = (SHARE_W - total) / 2;
  return units
    .map((u) => {
      const el = unitCard({ emoji: u.emoji, name: u.name, pwr: u.stats.pwr, hp: u.stats.hp, awoken: u.kind === "unit" && u.form === "awoken", fused: u.kind === "fused" }, x, 250, cw, ch, lang);
      x += cw + gap;
      return el;
    })
    .join("");
}

interface UnitShare {
  unit: UnitContent;
  abilities: AbilityRegistry;
  by: PlayerRef | null;
  evolvedBy: PlayerRef | null;
  isNew: boolean;
  live: boolean;
}

/** A live unit, else one in the Library; with its credits. */
function findUnit(rt: ShareDeps, unitId: string): UnitShare | null {
  const live = rt.content.units.find((u) => u.id === unitId);
  if (live) {
    const c = creditsView(rt).units.find((x) => x.unitId === unitId);
    return { unit: live, abilities: rt.content.abilities, by: c?.by ?? null, evolvedBy: c?.evolvedBy ?? null, isNew: !!c?.isNew, live: true };
  }
  const lib = libraryView(rt);
  const left = lib.units.find((x) => x.unit.id === unitId);
  return left ? { unit: left.unit, abilities: lib.abilities, by: left.by, evolvedBy: left.evolvedBy, isNew: false, live: false } : null;
}

/** The plain sentence of a form (When → Who → Does), as cards show it. */
function ruleText(u: UnitContent, form: "sleeping" | "awoken", abilities: AbilityRegistry): string {
  return formSegments(u.forms[form], abilities)
    .map((s) => s.text)
    .join("");
}

/** One unit's card big, its rule in words and who it is by; null: no such unit. */
export function unitSvg(rt: ShareDeps, unitId: string, lang: ShareLang): string | null {
  const s = findUnit(rt, unitId);
  if (!s) return null;
  const { unit } = s;
  const w = WORDS[lang];
  const cardX = 56, cardY = 110, cardW = 340, cardH = 425;
  const tx = cardX + cardW + 48, tw = SHARE_W - tx - 56;
  const out = [unitCard({ emoji: unit.emoji, name: unit.name, pwr: unit.base.pwr, hp: unit.base.hp, tier: unit.tier, isNew: s.isNew }, cardX, cardY, cardW, cardH, lang)];
  const name = fit(unit.name, DISPLAY, tw, 56, 34);
  out.push(textEl(name.text, tx, 150, name.size, DISPLAY, C.text));
  const arch = fit(`${w.tier} ${ROMAN[unit.tier - 1]} · ${unit.archetype}${s.live ? "" : ` · ${w.left}`}`, BODY, tw, 26, 20);
  out.push(textEl(arch.text, tx, 190, arch.size, BODY, C.dim));
  let y = 240;
  for (const [label, form, color] of [[w.sleeping, "sleeping", C.you], [w.awokenForm, "awoken", C.gold]] as const) {
    out.push(textEl(label.toUpperCase(), tx, y, 18, DISPLAY, color, 'letter-spacing="2"'));
    y += 34;
    for (const l of wrap(ruleText(unit, form, s.abilities), BODY, 28, tw, 3)) {
      out.push(textEl(l, tx, y, 28, BODY, C.text));
      y += 32;
    }
    y += 22;
  }
  const credit = [s.by ? w.ideaBy(s.by.name) : null, s.evolvedBy ? w.evolvedBy(s.evolvedBy.name) : null].filter(Boolean).join(" · ") || w.seed;
  const cr = fit(credit, BODY, tw, 30, 20);
  out.push(textEl(cr.text, tx, Math.max(y + 6, 500), cr.size, BODY, s.by || s.evolvedBy ? C.gold : C.dim));
  return frame(out.join("\n"), lang);
}

// ---------- PNG, cached per content ----------

const pngs = new Map<string, Buffer>();
const PNG_CACHE = 64;
/** The SVG as a PNG, and its hash (the ETag): a card whose content didn't
 * change is rendered once. */
export function sharePng(svg: string): { png: Buffer; hash: string } {
  const hash = createHash("sha256").update(svg).digest("hex").slice(0, 32);
  let png = pngs.get(hash);
  if (!png) {
    png = new Resvg(svg, { font: RESVG_FONT, fitTo: { mode: "width", value: SHARE_W } }).render().asPng();
    if (pngs.size >= PNG_CACHE) pngs.delete(pngs.keys().next().value!);
    pngs.set(hash, png);
  }
  return { png, hash };
}

// ---------- Open Graph tags for a shared link ----------

/** The <meta> tags for the game's page opened from a shared link
 * (`?share=champion[&day=N]`, `?share=unit&unit=<id>`), or null for any
 * other page. `publicUrl` is the game's public address. */
export function shareMeta(rt: ShareDeps, url: URL, publicUrl = PUBLIC_URL, acceptLanguage?: string): string | null {
  const kind = url.searchParams.get("share");
  const lang = shareLang(url.searchParams.get("lang") ?? undefined, acceptLanguage);
  const w = WORDS[lang];
  const q = lang === "ru" ? "?lang=ru" : "";
  let title: string, description: string, image: string;
  if (kind === "champion") {
    const day = Number(url.searchParams.get("day") ?? rt.today().seq);
    if (!Number.isInteger(day) || !championSvg(rt, day, lang)) return null;
    const champ = championOf(rt.store, day)!;
    title = `${w.championOf(dateText(champOfDay(rt, champ, day), lang))}: @${champ.player.name}`;
    description = `${champ.line.map((u) => `${u.emoji} ${u.name}`).join(", ")}. ${w.canYou}`;
    image = `${publicUrl}/api/v1/share/champion/${day}.png${q}`;
  } else if (kind === "unit") {
    const s = findUnit(rt, url.searchParams.get("unit") ?? "");
    if (!s) return null;
    title = w.unitTitle(`${s.unit.emoji} ${s.unit.name}`);
    description = ruleText(s.unit, "sleeping", s.abilities);
    image = `${publicUrl}/api/v1/share/unit/${encodeURIComponent(s.unit.id)}.png${q}`;
  } else return null;
  const page = `${publicUrl}/${url.search}`;
  return [
    ["og:type", "website"],
    ["og:site_name", "Arena of Ideas"],
    ["og:title", title],
    ["og:description", description],
    ["og:url", page],
    ["og:image", image],
    ["og:image:width", String(SHARE_W)],
    ["og:image:height", String(SHARE_H)],
  ]
    .map(([p, v]) => `<meta property="${p}" content="${esc(v!)}" />`)
    .concat([`<meta name="twitter:card" content="summary_large_image" />`, `<meta name="twitter:image" content="${esc(image)}" />`, `<meta name="description" content="${esc(description)}" />`])
    .join("\n    ");
}
