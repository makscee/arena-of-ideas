// The one card (PRD #015 slice 1, made uniform in #078) — every place a Unit
// OR a Status renders draws THIS markup at ONE fixed size: shop offers, the
// team line, the battle board, ladder pools, and the codex (units AND statuses).
// A Unit and a Status are the same shape (PRD #074 ontology: a bundle of Parts),
// so they share the card — a Status frames its per-stack statMods where a Unit
// frames hp/pwr. The fixed size IS the complexity budget (#078): geometry lives
// in CSS on `.unit` itself (one `--card-w`), never set per-surface. Pure
// presentation over data the kernel already produced: generative code-drawn
// shape art (hash → layered SVG, no image assets — pillar 3), framed stats, a
// level badge with fusion pips, status chips. The card owns zero rules; every
// description behind it stays kernel-derived (chipsHtml / the describe helpers).

import { FAMILY_HEX } from "../src/index.js";
import type { Family, StatusRegistry } from "../src/index.js";
import { chipsHtml } from "./inspect.js";

const esc = (s: string): string =>
  s.replace(/[&<>"]/g, (c) => ({ "&": "&amp;", "<": "&lt;", ">": "&gt;", '"': "&quot;" })[c]!);

/** Small string hash — picks a stable shape + hue per unit name. */
function hashName(name: string): number {
  let h = 2166136261;
  for (let i = 0; i < name.length; i++) {
    h ^= name.charCodeAt(i);
    h = Math.imul(h, 16777619);
  }
  return h >>> 0;
}

/** Hash-stable hue for a name — the colour identity the shape art wears,
 * exported so a non-unit card (the codex's status cards) can carry the same
 * scheme without re-deriving it. */
export function nameHue(name: string): number {
  return (hashName(name) * 137.508) % 360;
}

const SHAPES = [
  '<circle cx="16" cy="16" r="12"/>',
  '<rect x="5" y="5" width="22" height="22" rx="3"/>',
  '<polygon points="16,3 29,16 16,29 3,16"/>', // diamond
  '<polygon points="16,4 29,27 3,27"/>', // triangle
  '<polygon points="9,4 23,4 30,16 23,28 9,28 2,16"/>', // hexagon
  '<polygon points="16,3 29,12 24,28 8,28 3,12"/>', // pentagon
];

/** Generative shape art, layered from the name's hash alone: a soft aura, the
 * main shape, and a rotated inner accent — same name, same art, everywhere a
 * unit shows. Dead units go monochrome (the grave keeps the silhouette). */
export function shapeSvg(unitName: string, dead: boolean): string {
  const h = hashName(unitName);
  const shape = SHAPES[h % SHAPES.length]!;
  const inner = SHAPES[(h >>> 3) % SHAPES.length]!;
  const spin = ((h >>> 7) % 8) * 45; // the accent's rotation — hash-stable
  const hue = nameHue(unitName);
  const fill = dead ? "hsl(0 0% 35%)" : `hsl(${hue.toFixed(0)} 45% 58%)`;
  const stroke = dead ? "hsl(0 0% 25%)" : `hsl(${hue.toFixed(0)} 50% 38%)`;
  const aura = dead ? "hsl(0 0% 30% / 0.15)" : `hsl(${hue.toFixed(0)} 60% 60% / 0.16)`;
  const accent = dead ? "hsl(0 0% 48%)" : `hsl(${((hue + 40) % 360).toFixed(0)} 55% 74%)`;
  return (
    `<svg class="shape" viewBox="0 0 32 32" aria-hidden="true">` +
    `<circle cx="16" cy="16" r="15" fill="${aura}"/>` +
    `<g fill="${fill}" stroke="${stroke}" stroke-width="2">${shape}</g>` +
    `<g transform="rotate(${spin} 16 16) translate(16 16) scale(0.42) translate(-16 -16)" fill="${accent}" opacity="0.9">${inner}</g>` +
    `</svg>`
  );
}

// ---------- B·Arena card: family colour axis + animated sigils (PRD #080) ----
// The card's colour is the unit's ABILITY FAMILY, taken as an INPUT (never
// re-derived from hashName). When a caller can't supply the family (pre-081, or
// a status/part card), `nameFamily` degrades a name to a stable family so the
// card still renders coloured — the card never *requires* the 081 model.

const FAMILIES: readonly Family[] = ["Poison", "Strike", "Shield", "Summon", "Arcane", "Control", "Heal"];

const FAMILY_CLASS: Record<Family, string> = {
  Poison: "fam-poison",
  Strike: "fam-strike",
  Shield: "fam-shield",
  Summon: "fam-summon",
  Arcane: "fam-arcane",
  Control: "fam-control",
  Heal: "fam-heal",
};

// The action chip's glyph, by family (mockup action legend): the effect glyph
// reads off the colour axis the card already holds — ☣ poison, ◆ strike/damage,
// ⛨ shield, ☠ summon, ✶ arcane, ⇄ control, ▲ heal. The trigger chip's glyph
// rides in on `triggerGlyph` (it depends on the event kind, not the family).
const FAMILY_GLYPH: Record<Family, string> = {
  Poison: "☣",
  Strike: "◆",
  Shield: "⛨",
  Summon: "☠",
  Arcane: "✶",
  Control: "⇄",
  Heal: "▲",
};

// Presentation-only name→family heuristics (the degrade path). Keyword first,
// then a hash so EVERY name still lands on a stable family.
const FAMILY_KEYWORDS: readonly [RegExp, Family][] = [
  [/venom|poison|toxic|plague|blight|rot/i, "Poison"],
  [/summon|necro|conjur|raise|spawn|warlock/i, "Summon"],
  [/shield|bulwark|warden|guard|aegis|bastion|squire|wall/i, "Shield"],
  [/heal|cleric|medic|mend|priest|sooth|vital/i, "Heal"],
  [/arcane|mage|wizard|sorcer|mystic|rune|seer/i, "Arcane"],
  [/silenc|control|warlord|hex|chain|bind|command|tyrant/i, "Control"],
  [/strike|brawl|warrior|blade|fighter|knight|berserk/i, "Strike"],
];

/** Degrade a unit name to a stable family — the presentation fallback used when
 * no `family`/`color` is supplied (so a card renders coloured pre-081). */
export function nameFamily(name: string): Family {
  for (const [re, fam] of FAMILY_KEYWORDS) if (re.test(name)) return fam;
  return FAMILIES[hashName(name) % FAMILIES.length]!;
}

/** Mix a hex toward white by `t` (0..1) — the light accent each sigil draws its
 * highlights in, derived from the one family hex (no second palette). */
function lighten(hex: string, t: number): string {
  const m = hex.replace("#", "");
  const full = m.length === 3 ? m.split("").map((c) => c + c).join("") : m;
  const n = parseInt(full, 16);
  const ch = (c: number) => Math.round(c + (255 - c) * t).toString(16).padStart(2, "0");
  return `#${ch((n >> 16) & 255)}${ch((n >> 8) & 255)}${ch(n & 255)}`;
}

/** The animated glyph per family (mockup 421–427). One sigil, drawn at the art
 * size (full) and mini (compact). Motion rides CSS classes (`ub-spin`/`ub-pulse`
 * /`ub-glow`), NOT SMIL, so the `prefers-reduced-motion` media query in CSS can
 * drop it to the static end-state — the same way the death anim is guarded. */
function familySigilInner(family: Family, hex: string): string {
  const c = hex;
  const l = lighten(hex, 0.5);
  switch (family) {
    case "Poison":
      return (
        `<circle cx="50" cy="50" r="30" fill="${c}" opacity=".10"/>` +
        `<g class="ub-spin" style="animation-duration:16s"><circle cx="50" cy="18" r="2.6" fill="${c}"/><circle cx="82" cy="50" r="2" fill="${c}" opacity=".7"/><circle cx="50" cy="82" r="2.6" fill="${c}" opacity=".5"/><circle cx="18" cy="50" r="2" fill="${c}" opacity=".7"/></g>` +
        `<path d="M50 30 L65 58 L35 58 Z" fill="${c}" fill-opacity=".15" stroke="${l}" stroke-width="1.4"/>` +
        `<circle class="ub-pulse" style="animation-duration:3s" cx="50" cy="51" r="4" fill="${l}"/>`
      );
    case "Strike":
      return (
        `<circle cx="50" cy="50" r="30" fill="${c}" opacity=".10"/>` +
        `<path d="M36 30 L58 50 L36 70" fill="none" stroke="${c}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round"/>` +
        `<path d="M52 30 L74 50 L52 70" fill="none" stroke="${l}" stroke-width="3" stroke-linecap="round" stroke-linejoin="round" opacity=".75"/>` +
        `<circle class="ub-pulse" style="animation-duration:1.6s" cx="50" cy="50" r="3.4" fill="${c}"/>`
      );
    case "Shield":
      return (
        `<circle cx="50" cy="50" r="30" fill="${c}" opacity=".10"/>` +
        `<polygon points="50,22 74,36 74,64 50,78 26,64 26,36" fill="${c}" fill-opacity=".10" stroke="${c}" stroke-width="1.4"/>` +
        `<polygon class="ub-glow" style="animation-duration:3.2s" points="50,32 65,41 65,59 50,68 35,59 35,41" fill="none" stroke="${l}" stroke-width="1.6"/>`
      );
    case "Summon":
      return (
        `<circle cx="50" cy="50" r="30" fill="${c}" opacity=".10"/>` +
        `<rect x="42" y="42" width="16" height="16" transform="rotate(45 50 50)" fill="${c}" fill-opacity=".18" stroke="${l}" stroke-width="1.4"/>` +
        `<g class="ub-spin" style="animation-duration:10s"><rect x="46.5" y="16.5" width="7" height="7" fill="${l}"/><rect x="70.5" y="60.5" width="7" height="7" fill="${l}" opacity=".7"/><rect x="22.5" y="60.5" width="7" height="7" fill="${l}" opacity=".7"/></g>`
      );
    case "Heal":
      return (
        `<circle class="ub-pulse" style="animation-duration:3.5s" cx="50" cy="50" r="30" fill="${c}" opacity=".10"/>` +
        `<path d="M44 30 H56 V44 H70 V56 H56 V70 H44 V56 H30 V44 H44 Z" fill="${c}" fill-opacity=".16" stroke="${l}" stroke-width="1.4"/>`
      );
    case "Arcane":
      return (
        `<circle cx="50" cy="50" r="30" fill="${c}" opacity=".10"/>` +
        `<g class="ub-spin" style="animation-duration:18s"><line x1="50" y1="22" x2="50" y2="78" stroke="${c}" stroke-width="1.6"/><line x1="50" y1="22" x2="50" y2="78" stroke="${l}" stroke-width="1.6" transform="rotate(60 50 50)"/><line x1="50" y1="22" x2="50" y2="78" stroke="${c}" stroke-width="1.6" transform="rotate(120 50 50)"/></g>` +
        `<circle class="ub-pulse" style="animation-duration:2.4s" cx="50" cy="50" r="5" fill="${l}"/>`
      );
    case "Control":
      return (
        `<circle cx="50" cy="50" r="30" fill="${c}" opacity=".10"/>` +
        `<g class="ub-spin" style="animation-duration:11s"><path d="M30 40 A22 22 0 0 1 70 40" fill="none" stroke="${l}" stroke-width="2.6"/><path d="M70 40 l0 -8 l7 5 z" fill="${l}"/><path d="M70 60 A22 22 0 0 1 30 60" fill="none" stroke="${c}" stroke-width="2.6"/><path d="M30 60 l0 8 l-7 -5 z" fill="${c}"/></g>` +
        `<circle class="ub-pulse" style="animation-duration:2.6s" cx="50" cy="50" r="3.5" fill="${l}"/>`
      );
  }
}

/** A family sigil SVG (viewBox 100×100). `cls` lets the compact card draw a
 * mini variant of the same glyph. Exported so a probe/test can assert the
 * family colour rode into the sigil. */
export function familySigil(family: Family, hex: string = FAMILY_HEX[family], cls = "ub-sigil"): string {
  return `<svg class="${cls}" viewBox="0 0 100 100" aria-hidden="true">${familySigilInner(family, hex)}</svg>`;
}

export interface UnitCardOpts {
  /** What entity this card renders. The skeleton (art, name, framed stats,
   * chips) and the fixed size are IDENTICAL for both — a Status is the same
   * shape as a Unit (a bundle of Parts, PRD #074 ontology), so it wears the
   * same card at the same size (the card size IS the complexity budget, #078).
   * A `status` frames its per-stack `statMods` in the stat cells where a unit
   * frames base hp/pwr. Defaults to "unit", so every existing caller is
   * unchanged. */
  kind?: "unit" | "status" | "part";
  /** For a `part` card (#078): the atom's family label ("Effect", "Selector",
   * "Trigger", …), shown in the stat band where a unit frames hp/pwr — a Part
   * is the SAME card at the SAME fixed size, framing its family where a unit
   * frames stats and a status frames statMods. Ignored for unit/status. */
  tag?: string;
  /** Drives the generative art — the def name, stable across levels/instances. */
  artName: string;
  /** The name shown on the card (board instances may carry a display name). */
  label: string;
  /** The unit's team (#065 item 2): tints the name by side so the player reads it
   * as "their side". Only the battle board passes it; the shop/team/ladder omit
   * it (those screens aren't two-sided) — the card contract there is unchanged. */
  side?: "A" | "B";
  /** Pre-formatted stat text: "7", or "3/9" for a board card's current/max. */
  hp: string | number;
  pwr: string | number;
  registry: StatusRegistry;
  statuses?: readonly { status: string; stacks: number }[] | undefined;
  /** Renders the level badge when given; `pips` (●●○) ride inside it. */
  level?: number;
  pips?: string;
  front?: boolean;
  dead?: boolean;
  hit?: boolean;
  sel?: boolean;
  silenced?: boolean;
  fused?: boolean;
  /** Dying-in-place (#065 slice 2): a unit whose Death landed this beat shows
   * greyed with a ✕ in its line slot until the next beat collapses it to the
   * grave — distinct from `dead` (already in the grave). */
  dying?: boolean;
  /** Death-reveal moment (#065 item 4): the dying unit plays a distinct death
   * animation (red flash + shake, settling to the grey+✕ end-state) the single
   * step its Death is revealed. Set only at that step; on later steps it stays
   * `dying` (static grey+✕). Honors prefers-reduced-motion (the anim is skipped,
   * the end-state stays) via CSS. */
  dyingNew?: boolean;
  /** Beat-overlay badge layer (#065 slice 2): pre-built typed-badge HTML drawn
   * ON the card. Empty for every non-replay surface, so the card contract the
   * shop/team/ladder rely on is unchanged. */
  overlay?: string;
  /** Persistent coin marker (#065 slice 3): pre-built HTML for the coin chip the
   * holder wears (the most recent pairing's first striker). A PERSISTENT state
   * marker, separate from the per-beat `overlay` deltas. Empty everywhere but the
   * replay board, so the shop/team/ladder card contract is unchanged. */
  marker?: string;
  /** Context hooks (run-card, lv-unit, codex-unit) — for framing/state only
   * (selection ring, opacity, layout slot). The card's WIDTH is NOT theirs to
   * set: size is fixed on `.unit` itself (#078). A surface that needs different
   * framing wraps the card; it never resizes it. */
  classes?: string;
  /** The caller's wiring, pre-escaped: `data-offer="0"`, `data-unit="A1:X"`… */
  attrs: string;
  title: string;
  /** Controls under the chips (buy button, move arrows) — caller-built HTML. */
  footer?: string;

  // ---- B·Arena card inputs (PRD #080) ----
  /** The unit's ability family — the card's colour, taken as an INPUT. Drives
   * the border, glow, sigil and accents. When absent, `nameFamily(artName)`
   * degrades a name to a stable family (renders coloured pre-081). */
  family?: Family | undefined;
  /** An explicit colour override (any CSS hex) — used when a caller wants a
   * colour the 7 families don't name. Defaults to the family's FAMILY_HEX. */
  color?: string | undefined;
  /** Which markup the B·Arena card draws. Passing `variant` (or `family`/
   * `color`) opts a call site into the new chamfered, family-coloured card;
   * callers that pass none keep the legacy #078 card unchanged. Default `full`
   * once opted in. */
  variant?: "full" | "compact" | undefined;
  /** The ABILITY cap-label in the header (e.g. "TOXIC STRIKE"). Falls back to
   * the family name uppercased. New card only. */
  abilityLabel?: string | undefined;
  /** Ability line segments (mockup `⚔ trigger ▸ target ▸ ☣ action`). Any subset
   * renders; absent segments (and their separators) are dropped. New card only. */
  trigger?: string | undefined;
  target?: string | undefined;
  action?: string | undefined;
  /** The trigger chip's glyph, by event kind (⚔ strike, ⚑ battle-start, ☠ death,
   * …). Defaults to ⚔. The action chip's glyph is the family glyph (derived from
   * the card's colour axis). New card only. */
  triggerGlyph?: string | undefined;
  /** A top state bar drawn as the FIRST child inside the card (#082 slice D):
   * the battle board's `ACTING` / `TARGET` ribbon. Pre-built HTML; empty
   * everywhere but the replay board, so every other caller is unchanged. */
  topTag?: string | undefined;
  /** This unit already acted this turn (#082 slice D): dims the card and strikes
   * through its ability line (a `✓ USED` chip is drawn by CSS). New card only. */
  used?: boolean | undefined;
}

/** The shared card markup. Class names and child order are the app's card
 * contract: probes, hit-target CSS, and the inspector's anchors all key off
 * `.unit`, `.uname`, `.unums`, `.chips` and the data-* attrs the caller adds. */
export function unitCardHtml(o: UnitCardOpts): string {
  // Opt-in (#080): a call that names a `variant`/`family`/`color` gets the new
  // B·Arena chamfered card; every legacy caller is byte-unchanged below.
  if (o.variant !== undefined || o.family !== undefined || o.color !== undefined) {
    return variantCardHtml(o);
  }
  const cls = [
    "unit",
    o.kind === "status" && "is-status",
    o.kind === "part" && "is-part",
    o.classes,
    o.front === true && "front",
    o.dead === true && "dead",
    o.dying === true && "dying",
    o.dyingNew === true && "dying-new",
    o.hit === true && "hit",
    o.sel === true && "sel",
    o.fused === true && "fused",
  ]
    .filter(Boolean)
    .join(" ");
  // Team tint on the name (#065 item 2): side A / side B get distinct hues so a
  // name reads as its side. Reuses the battle log's .u/.ua/.ub side palette.
  const unameCls = ["uname", o.side === "A" && "u ua", o.side === "B" && "u ub"].filter(Boolean).join(" ");
  const badge =
    o.level !== undefined
      ? `<span class="run-lvl">L${o.level}${o.pips !== undefined ? ` <span class="run-pips">${o.pips}</span>` : ""}</span>`
      : "";
  // Like every other chip, the title explains the state, not just names it.
  const silenced =
    o.silenced === true
      ? '<span class="chip mute" title="Silenced — its statuses are stripped and its own abilities are disabled for the battle">mut</span>'
      : "";
  return `
    <div class="${cls}" ${o.attrs} title="${esc(o.title)}">
      ${o.front === true ? '<span class="front-tag">front</span>' : ""}
      ${shapeSvg(o.artName, o.dead === true)}
      <span class="${unameCls}">${esc(o.label)}</span>
      ${badge}
      <span class="unums">${
        o.kind === "part"
          ? `<span class="ptag">${esc(o.tag ?? "")}</span>`
          : `<span class="hp">${o.hp}</span><span class="pwr">${o.pwr}</span>`
      }</span>
      <span class="chips">${chipsHtml(o.statuses, o.registry)}${silenced}</span>
      ${o.footer ?? ""}
      ${o.dying === true ? '<span class="dying-x" aria-hidden="true">✕</span>' : ""}
      ${o.overlay ?? ""}
      ${o.marker ?? ""}
    </div>`;
}

/** The B·Arena card (PRD #080) — full + compact, coloured by ability family.
 * Keeps the card contract anchors (`.unit`, `.uname`, `.unums` with `.hp`/`.pwr`,
 * `.chips`) so probes and the inspector still key off them; the chamfer, glow,
 * sigil and family border are layered in CSS off the inline `--fam` colour. */
function variantCardHtml(o: UnitCardOpts): string {
  const variant = o.variant ?? "full";
  const family: Family = o.family ?? nameFamily(o.artName);
  const hex = o.color ?? FAMILY_HEX[family];
  const cls = [
    "unit",
    "unit-b",
    `is-${variant}`,
    FAMILY_CLASS[family],
    o.classes,
    o.front === true && "is-front",
    o.sel === true && "sel",
    o.fused === true && "fused",
    o.dead === true && "dead",
    o.used === true && "is-used",
  ]
    .filter(Boolean)
    .join(" ");

  const label = `<span class="uname">${esc(o.label)}</span>`;
  const cap = `<span class="ub-cap"><svg class="ub-spark" viewBox="0 0 16 16" width="10" height="10" aria-hidden="true"><path d="M8 1 L9.4 6.6 L15 8 L9.4 9.4 L8 15 L6.6 9.4 L1 8 L6.6 6.6 Z" fill="currentColor"/></svg><span class="ub-cap-t">${esc((o.abilityLabel ?? family).toUpperCase())}</span></span>`;
  const nums = `<span class="unums"><span class="hp">${o.hp}</span><span class="ub-sep">·</span><span class="pwr">${o.pwr}</span></span>`;

  // Ability line: <glyph> trigger ▸ target ▸ <glyph> action — any subset,
  // separators only between present segments. The trigger glyph travels with the
  // chips (it's event-kind-specific); the action glyph is the family glyph.
  const trigGlyph = o.triggerGlyph ?? "⚔";
  const actGlyph = FAMILY_GLYPH[family];
  const segs: string[] = [];
  if (o.trigger !== undefined && o.trigger !== "")
    segs.push(`<span class="ub-seg ub-trig"><b>${esc(trigGlyph)}</b> ${esc(o.trigger)}</span>`);
  if (o.target !== undefined && o.target !== "") segs.push(`<span class="ub-seg ub-tgt">${esc(o.target)}</span>`);
  if (o.action !== undefined && o.action !== "")
    segs.push(`<span class="ub-seg ub-act"><b>${esc(actGlyph)}</b> ${esc(o.action)}</span>`);
  const ability = segs.length > 0 ? `<div class="ub-ability">${segs.join('<span class="ub-arrow">▸</span>')}</div>` : "";

  const badge =
    o.level !== undefined
      ? `<span class="run-lvl">L${o.level}${o.pips !== undefined ? ` <span class="run-pips">${o.pips}</span>` : ""}</span>`
      : "";
  const silenced =
    o.silenced === true
      ? '<span class="chip mute" title="Silenced — its statuses are stripped and its own abilities are disabled for the battle">mut</span>'
      : "";
  // Keep the status badges in a DIRECT `.chips` strip (like the legacy card) so
  // the run rows' gapless 44px touch band + min-height reserve apply unchanged;
  // the level badge + footer (buy / move) ride a separate `.ub-foot` row below.
  const chips = `<span class="chips">${chipsHtml(o.statuses, o.registry)}${silenced}</span>`;
  const foot = badge !== "" || (o.footer ?? "") !== "" ? `<div class="ub-foot">${badge}${o.footer ?? ""}</div>` : "";

  const sigil = familySigil(family, hex);
  const sigilMini = familySigil(family, hex, "ub-sigil ub-sigil-mini");

  const head =
    variant === "compact"
      ? `<div class="ub-head"><div class="ub-mini">${sigilMini}</div><div class="ub-id">${label}${cap}</div>${nums}</div>`
      : `<div class="ub-head"><div class="ub-id">${label}${cap}</div>${nums}</div>`;
  const art = variant === "compact" ? "" : `<div class="ub-art">${sigil}</div>`;

  return `<div class="${cls}" style="--fam:${hex}" ${o.attrs} title="${esc(o.title)}">${o.topTag ?? ""}${head}${art}${ability}${chips}${foot}</div>`;
}
