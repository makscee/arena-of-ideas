// Units' names and lines in the reader's language (M4-4). The content pack
// carries each translated unit's Russian name and line (`texts.ru`, beside
// the unit, never in its version); a Russian page reads them, and a unit
// without one keeps its English name and line.
import type { FusionDiscovery, MvpContent, UnitContent } from "../src/mvp/contract";
import { rulesLang } from "./lang";

let ru = new Map<string, { en: string; name: string; line: string }>();
let fusionRu = new Map<string, { en: string; name: string }>();
const pairKey = (first: string, second: string) => `${first} ${second}`;

/** The unit in `lang`: its Russian name and line (`archetype`) when it has
 * them and `lang` is Russian, else as it came. */
export function localUnit(u: UnitContent, lang = rulesLang()): UnitContent {
  const t = lang === "ru" ? u.texts?.ru : undefined;
  return t ? { ...u, name: t.name, archetype: t.line } : u;
}

/** The pack in the page's language (localUnit on every unit, live and left),
 * remembering each unit's English name for unitName. */
export function localContent(c: MvpContent, lang = rulesLang()): MvpContent {
  ru = new Map(
    [...c.units, ...(c.left ?? [])].flatMap((u) => (u.texts?.ru ? [[u.id, { en: u.name, ...u.texts.ru }] as const] : [])),
  );
  if (lang !== "ru" || ru.size === 0) return c;
  return { ...c, units: c.units.map((u) => localUnit(u, lang)), ...(c.left ? { left: c.left.map((u) => localUnit(u, lang)) } : {}) };
}

/** A name the server sent (a line, shop or battle unit), in the page's
 * language: the unit's Russian name when `name` is its English one. A fused
 * unit's name, or a summon's, is left as it is. */
export function unitName(unitId: string | undefined, name: string, lang = rulesLang()): string {
  if (lang !== "ru" || !unitId) return name;
  const t = ru.get(unitId);
  return t && t.en === name ? t.name : name;
}

/** Remembers the discoveries' Russian names (GET /fusions) for fusionName. */
export function setFusionTexts(list: readonly FusionDiscovery[]): void {
  fusionRu = new Map(list.flatMap((f) => (f.texts?.ru ? [[pairKey(f.first, f.second), { en: f.name, name: f.texts.ru.name }] as const] : [])));
}

/** A discovery's name in the page's language: its Russian name when it has one. */
export const discoveryName = (f: Pick<FusionDiscovery, "name" | "texts">, lang = rulesLang()): string => (lang === "ru" && f.texts?.ru?.name) || f.name;

/** A fused unit's name the server sent, in the page's language: the pair's
 * Russian name when `name` is its English one (an unnamed preview's "" stays). */
export function fusionName(fusion: { first: string; second: string } | undefined, name: string, lang = rulesLang()): string {
  if (lang !== "ru" || !fusion || !name) return name;
  const t = fusionRu.get(pairKey(fusion.first, fusion.second));
  return t && t.en === name ? t.name : name;
}
