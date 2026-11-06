// The closed set of glyph names a piece's face and status chips use (mission arena#34,
// slice G, arena#41). A glyph is a small drawn mark (the client draws it from a sigil
// spec, mobile/ui/sigil/glyphs.ts), so the face's `icon` is a name from this list, never
// an emoji. The night designer picks from it; a face an older save or an LLM wrote with an
// emoji maps through LEGACY_GLYPH. Names only: no drawing here, so the server can validate.

export const GLYPH_NAMES = [
  // when: the moment a rule acts
  "sunrise", "moon", "flag", "swords", "broken-heart", "skull", "infinity", "paw", "flask", "scroll", "egg",
  // headline and status marks
  "shield", "fist", "heart", "halo", "bolt", "drop", "snowflake", "curse", "silence",
  "plus", "repeat", "swap", "mirror", "burst", "summon", "grave", "spark",
  // the piece's own chrome (a locked offer); a face may name it too
  "lock",
] as const;
export type GlyphName = (typeof GLYPH_NAMES)[number];

const NAME_SET: ReadonlySet<string> = new Set(GLYPH_NAMES);
export const isGlyphName = (s: unknown): s is GlyphName => typeof s === "string" && NAME_SET.has(s);

/** What each emoji the faces used to carry draws now. */
export const LEGACY_GLYPH: Readonly<Record<string, GlyphName>> = Object.freeze({
  "🌅": "sunrise", "🕯": "sunrise",
  "🌙": "moon",
  "🚩": "flag",
  "⚔": "swords",
  "💔": "broken-heart",
  "💀": "skull",
  "♾": "infinity",
  "🐺": "paw",
  "🧪": "flask",
  "📜": "scroll",
  "💚": "plus",
  "🛡": "shield",
  "💪": "fist",
  "🥚": "egg",
  "💧": "drop",
  "❄": "snowflake",
  "🔻": "curse",
  "🤫": "silence",
  "🔁": "repeat",
  "🔀": "swap",
  "🪞": "mirror",
  "💥": "burst",
  "🐲": "summon", "🦋": "summon",
  "😇": "halo",
  "❤": "heart",
  "🪦": "grave",
  "⚡": "bolt",
  "✨": "spark",
  "🔒": "lock",
});

/** The glyph a face icon draws: a name as it is, an old emoji through the table, else the plain spark. */
export function glyphName(icon: string | undefined): GlyphName {
  if (isGlyphName(icon)) return icon;
  const bare = (icon ?? "").replace(/️/g, "");
  return LEGACY_GLYPH[bare] ?? "spark";
}

/** A face icon as a name when it is one or a known emoji; undefined for anything else (so a face check can refuse it). */
export function knownGlyph(icon: string | undefined): GlyphName | undefined {
  if (isGlyphName(icon)) return icon;
  return LEGACY_GLYPH[(icon ?? "").replace(/️/g, "")];
}

/** A status chip's glyph and whether it helps its holder. */
const STATUS: Record<string, [GlyphName, "good" | "bad"]> = {
  Shield: ["shield", "good"],
  Strength: ["fist", "good"],
  Vitality: ["heart", "good"],
  Blessing: ["halo", "good"],
  Charge: ["bolt", "good"],
  Poison: ["drop", "bad"],
  Curse: ["curse", "bad"],
  Freeze: ["snowflake", "bad"],
  Silence: ["silence", "bad"],
};
export const statusGlyph = (status: string): [GlyphName, "good" | "bad"] => (Object.hasOwn(STATUS, status) ? STATUS[status]! : ["spark", "good"]);
