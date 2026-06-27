// Inline-SVG glyph icons (#086) — the trigger / action / ability marks the
// ability-chip lines and the acting card draw. The vendored display fonts
// (Chakra Petch / Rajdhani / IBM Plex Mono) DON'T carry the unicode marks the
// design speaks in (⚔ ☠ ⚑ ✸ ◆ ☣ ⛨ ▲ ⇄ ✶ ✦), so a raw glyph fell back to the
// wrong character (⚔→×, ☠→a padlock, ✦→+). The mockup itself draws these as
// inline SVG sigils — so we match it: each mark is a tiny `currentColor` SVG,
// sized to the text (1em) so it inherits the chip's colour and font size. No
// font dependency, no image assets (pillar 3). Static — honours
// prefers-reduced-motion for free.

import type { Family } from "../src/index.js";

/** The trigger marks, by event kind (the moment an ability fires). */
export type TriggerKind =
  | "strike"
  | "battle-start"
  | "turn-start"
  | "turn-end"
  | "damaged"
  | "heal"
  | "death"
  | "summon";

/** The action / effect marks. The seven family marks (the card's colour axis)
 * plus the extra effect marks a RESULT row narrates. */
export type ActionKind =
  | "poison"
  | "damage"
  | "shield"
  | "summon"
  | "arcane"
  | "control"
  | "heal"
  | "death"
  | "status-loss"
  | "stat-up"
  | "stat-down"
  | "silence";

/** Each family's action mark — the effect glyph reads off the colour axis the
 * card already holds (mockup action legend). */
const FAMILY_ACTION: Record<Family, ActionKind> = {
  Poison: "poison",
  Strike: "damage",
  Shield: "shield",
  Summon: "summon",
  Arcane: "arcane",
  Control: "control",
  Heal: "heal",
};

// The mark bodies (viewBox 16×16, drawn in currentColor). Simple, legible marks
// in the mockup's visual language — crossed swords for strike, skull for death,
// banner for battle-start, burst for on-damaged; teardrop for poison, diamond
// for damage, shield for shield, up-chevron for heal, swap arrows for control.
const TRIGGER_BODY: Record<TriggerKind, string> = {
  // crossed swords
  strike:
    '<g fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round"><line x1="3" y1="4" x2="12" y2="13"/><line x1="13" y1="4" x2="4" y2="13"/><line x1="2" y1="2.5" x2="4.2" y2="4.7"/><line x1="14" y1="2.5" x2="11.8" y2="4.7"/></g>',
  // flag / banner on a pole
  "battle-start":
    '<g stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><line x1="4" y1="2" x2="4" y2="14"/><path d="M4 3 L12 5 L4 8 Z" fill="currentColor" stroke="none"/></g>',
  // refresh loop
  "turn-start":
    '<g fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round"><path d="M12.5 6 A5 5 0 1 0 13 9.2"/><path d="M12.6 2.6 L12.6 6 L9.2 6"/></g>',
  "turn-end":
    '<g fill="none" stroke="currentColor" stroke-width="1.4" stroke-linecap="round" stroke-linejoin="round" transform="scale(-1,1) translate(-16,0)"><path d="M12.5 6 A5 5 0 1 0 13 9.2"/><path d="M12.6 2.6 L12.6 6 L9.2 6"/></g>',
  // burst / spark
  damaged:
    '<g stroke="currentColor" stroke-width="1.3" stroke-linecap="round"><line x1="8" y1="1.5" x2="8" y2="14.5"/><line x1="1.5" y1="8" x2="14.5" y2="8"/><line x1="3.7" y1="3.7" x2="12.3" y2="12.3"/><line x1="12.3" y1="3.7" x2="3.7" y2="12.3"/></g>',
  // plus / cross
  heal: '<path d="M8 3 L8 13 M3 8 L13 8" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round"/>',
  // skull
  death:
    '<g fill="none" stroke="currentColor" stroke-width="1.2"><path d="M4 9.5 C4 5.4 5.8 3 8 3 C10.2 3 12 5.4 12 9.5 L12 10.6 L4 10.6 Z"/></g><g fill="currentColor"><circle cx="6.1" cy="7.4" r="1.25"/><circle cx="9.9" cy="7.4" r="1.25"/><path d="M8 8.7 L7.1 10.2 L8.9 10.2 Z"/></g><g stroke="currentColor" stroke-width="1" stroke-linecap="round"><line x1="6" y1="10.6" x2="6" y2="12"/><line x1="8" y1="10.6" x2="8" y2="12.2"/><line x1="10" y1="10.6" x2="10" y2="12"/></g>',
  // 4-point star (the summon / spark trigger)
  summon: '<path d="M8 1.5 L9.4 6.6 L14.5 8 L9.4 9.4 L8 14.5 L6.6 9.4 L1.5 8 L6.6 6.6 Z" fill="currentColor"/>',
};

const ACTION_BODY: Record<ActionKind, string> = {
  // teardrop (poison)
  poison: '<path d="M8 2 C8 2 3.6 7.6 3.6 10.3 A4.4 4.4 0 0 0 12.4 10.3 C12.4 7.6 8 2 8 2 Z" fill="currentColor"/>',
  // diamond / blade (damage)
  damage: '<path d="M8 1.5 L13 8 L8 14.5 L3 8 Z" fill="currentColor"/>',
  // shield
  shield:
    '<path d="M8 1.5 L13 3.4 L13 8 C13 11.2 10.8 13.5 8 14.5 C5.2 13.5 3 11.2 3 8 L3 3.4 Z" fill="currentColor"/>',
  // diamond + plus (summon a token — mockup summon action mark)
  summon:
    '<g fill="none" stroke="currentColor" stroke-width="1.5"><rect x="4" y="4" width="8" height="8" transform="rotate(45 8 8)"/><path d="M8 5.4 V10.6 M5.4 8 H10.6" stroke-width="1.3"/></g>',
  // 6-ray burst (arcane)
  arcane:
    '<g stroke="currentColor" stroke-width="1.5" stroke-linecap="round"><line x1="8" y1="1.8" x2="8" y2="14.2"/><line x1="2.6" y1="5" x2="13.4" y2="11"/><line x1="2.6" y1="11" x2="13.4" y2="5"/></g>',
  // swap arrows (control)
  control:
    '<g fill="none" stroke="currentColor" stroke-width="1.3" stroke-linecap="round" stroke-linejoin="round"><path d="M3 6 H12 L9.8 3.8"/><path d="M13 10 H4 L6.2 12.2"/></g>',
  // up-chevron (heal)
  heal: '<path d="M3 10.5 L8 5 L13 10.5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  death: TRIGGER_BODY.death,
  // hollow ring (a status spent / removed)
  "status-loss": '<circle cx="8" cy="8" r="4.6" fill="none" stroke="currentColor" stroke-width="1.5"/>',
  "stat-up": '<path d="M3 10.5 L8 5 L13 10.5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  "stat-down": '<path d="M3 5.5 L8 11 L13 5.5" fill="none" stroke="currentColor" stroke-width="1.8" stroke-linecap="round" stroke-linejoin="round"/>',
  // no-entry (silenced)
  silence:
    '<g fill="none" stroke="currentColor" stroke-width="1.4"><circle cx="8" cy="8" r="5"/><line x1="4.5" y1="11.5" x2="11.5" y2="4.5"/></g>',
};

/** Wrap a mark body in an inline SVG sized to the text. `kind` rides
 * `data-glyph` so a probe/test can assert the right mark for the right slot. */
function icon(kind: string, body: string, cls: string): string {
  return `<svg class="${cls}" data-glyph="${kind}" viewBox="0 0 16 16" aria-hidden="true">${body}</svg>`;
}

/** The trigger mark for an event kind (⚔ strike, ⚑ battle-start, ☠ death, …). */
export function triggerIcon(kind: TriggerKind, cls = "gly"): string {
  return icon(kind, TRIGGER_BODY[kind], cls);
}

/** The action / effect mark — a Family (the card's colour axis) or an explicit
 * effect kind a RESULT row narrates. */
export function actionIcon(kind: ActionKind | Family, cls = "gly"): string {
  const k = (kind in FAMILY_ACTION ? FAMILY_ACTION[kind as Family] : kind) as ActionKind;
  return icon(k, ACTION_BODY[k], cls);
}

/** The named-ability star (mockup ✦ before the cap-label). */
export function abilityStar(cls = "gly"): string {
  return icon("ability-star", TRIGGER_BODY.summon, cls);
}

// The legacy trigger glyph CHAR → kind bridge: the chip line still carries a
// trigger glyph char (describe.ts's TRIGGER_CHIP, by event kind) through the
// card opts. Map it to the mark here so the call sites stay a one-liner and
// describe.ts (and its tests) are untouched. Unknown chars fall to "strike".
const TRIGGER_GLYPH_KIND: Record<string, TriggerKind> = {
  "⚔": "strike",
  "⚑": "battle-start",
  "⟳": "turn-start",
  "⟲": "turn-end",
  "✸": "damaged",
  "✚": "heal",
  "☠": "death",
  "✦": "summon",
};

/** Resolve a legacy trigger glyph char to its mark (the chip-line bridge). */
export function triggerIconForGlyph(glyph: string, cls = "gly"): string {
  return triggerIcon(TRIGGER_GLYPH_KIND[glyph] ?? "strike", cls);
}
