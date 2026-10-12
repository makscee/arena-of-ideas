// The action strip's fit rule (mission 1007): one line, never wider than its
// box. The rule is total: given the widths it measured, it always returns a
// plan that fits, dropping parts in a fixed order (target shrinks, actor
// shrinks, the number's word becomes its icon); the number itself stays.

export interface StripNameWidth {
  /** The whole name's width in px. */
  nat: number;
  /** The least it may shrink to (the actor keeps ~6 characters). */
  min: number;
}

export interface StripMeasure {
  /** The line's inner width. */
  avail: number;
  /** Everything but the names and the number (arrows, icon, "+n", dots at their least), with the gaps between them. */
  others: number;
  actor: StripNameWidth;
  /** Absent when the actor acts on itself. */
  target: StripNameWidth | null;
  /** The number with its word, and its short form (the count, or nothing when only the icon is left), each with its gap; 0 when absent. */
  num: { full: number; short: number };
}

export interface StripPlan {
  actor: number;
  target: number;
  /** The number shows its short form. */
  short: boolean;
}

/** Widths for the names once `room` px is theirs: the target gives way first, then the actor. */
function shrink(actor: StripNameWidth, target: StripNameWidth | null, room: number): { actor: number; target: number; over: number } {
  const t = target ?? { nat: 0, min: 0 };
  let over = actor.nat + t.nat - room;
  const tw = over > 0 ? Math.max(Math.ceil(t.min), Math.floor(t.nat - over)) : t.nat;
  over -= t.nat - tw;
  const aw = over > 0 ? Math.max(Math.ceil(actor.min), Math.floor(actor.nat - over)) : actor.nat;
  over -= actor.nat - aw;
  return { actor: aw, target: tw, over };
}

export function planStrip(m: StripMeasure): StripPlan {
  const full = shrink(m.actor, m.target, m.avail - m.others - m.num.full);
  if (full.over <= 0) return { actor: full.actor, target: full.target, short: false };
  const short = shrink(m.actor, m.target, m.avail - m.others - m.num.short);
  return { actor: short.actor, target: short.target, short: true };
}

/** How the strip's number splits into a word and what stays when the line is full. */
export interface NumForm {
  /** The word and the count of "Curse ×1" (word first) or "1 blocked" (count first). */
  word?: string;
  count?: string;
  wordFirst?: boolean;
  /** What the number shows when the line is full: the count, "0", or "" (the effect icon alone). null: nothing to drop. */
  short: string | null;
  /** The number is only a word ("no damage", "silenced"): it has no count to keep. */
  wordOnly: boolean;
}

export function numFormOf(label: string, kind: string): NumForm {
  const m = /^([^\d×+−-]+?)\s*([×+−-]?\d.*)$/.exec(label);
  if (m) return { word: m[1]!, count: m[2]!, wordFirst: true, short: m[2]!, wordOnly: false };
  const t = /^([×+−-]?\d+)\s+(\p{L}{2,})$/u.exec(label); // "1 blocked", "+2 ATK": the count, then a word
  if (t) return { word: t[2]!, count: t[1]!, wordFirst: false, short: t[1]!, wordOnly: false };
  if (kind === "damage" && !/\d/.test(label)) return { short: "0", wordOnly: true };
  if (/^\p{L}[\p{L} ]{3,}$/u.test(label)) return { short: "", wordOnly: true };
  return { short: null, wordOnly: false };
}
