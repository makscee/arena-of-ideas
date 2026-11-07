// The piece's face (mission #1286, slice 1, makscee/void-board#1291): the
// hints a designer picks so a unit reads at a glance without its rule. A when
// tab (a glyph and a short word), one headline badge (a glyph and maybe a
// number) and, for a unit that builds a counter, a ring. They are free-form
// data, never a trigger/target/effect grammar: any odd future rule picks its
// own words. The rule itself stays in words, in the detail sheet only.
//
// The design: https://m1.twin-pogona.ts.net/r/arena-redesign/final/

import type { When } from "../types.js";
import type { UnitForm } from "./contract.js";
import { GLYPH_NAMES, isGlyphName, knownGlyph, type GlyphName } from "./glyph-names.js";

export interface Face {
  /** The when tab: a glyph name (src/mvp/glyph-names.ts) and a word of at most FACE_LABEL_MAX letters. An older save may hold an emoji: it draws through LEGACY_GLYPH. */
  when: { icon: string; label: string };
  /** One glyph name that sums up the rule, and an optional short amount ("1", "×2", "T"). */
  headline: { icon: string; amount?: string };
  /** A counter the unit builds and spends, drawn as a ring around the headline. */
  counter?: { status: string; max: number };
}

/** The when word's length, spaces not counted: "From turn 3" fits. */
export const FACE_LABEL_MAX = 10;
/** The headline amount's length: it sits in a small pill. */
export const FACE_AMOUNT_MAX = 2;

const charge = { status: "Charge", max: 3 };

/** The rules v2 pool's faces, picked by hand, by unit name. */
export const V2_FACE: Record<string, Face> = {
  Hourglass: { when: { icon: "moon", label: "Turn end" }, headline: { icon: "repeat" }, counter: charge },
  Mirror: { when: { icon: "flag", label: "Battle" }, headline: { icon: "scroll" } },
  Candle: { when: { icon: "skull", label: "Would die" }, headline: { icon: "heart", amount: "1" } },
  Gambler: { when: { icon: "swords", label: "Strikes" }, headline: { icon: "burst", amount: "×2" } },
  Nautilus: { when: { icon: "broken-heart", label: "Is hit" }, headline: { icon: "shield", amount: "1" } },
  Magnet: { when: { icon: "flask", label: "Ally hexed" }, headline: { icon: "swap" } },
  Raven: { when: { icon: "skull", label: "Ally dies" }, headline: { icon: "scroll" } },
  Seed: { when: { icon: "sunrise", label: "From turn 3" }, headline: { icon: "fist", amount: "1" } },
  Sniper: { when: { icon: "sunrise", label: "Turn start" }, headline: { icon: "burst" } },
  Medic: { when: { icon: "broken-heart", label: "Ally hit" }, headline: { icon: "plus" } },
  Leech: { when: { icon: "swords", label: "Strikes" }, headline: { icon: "plus", amount: "½" } },
  "Lone Wolf": { when: { icon: "paw", label: "When alone" }, headline: { icon: "burst", amount: "×2" } },
  Drummer: { when: { icon: "swords", label: "Ally strike" }, headline: { icon: "fist", amount: "1" }, counter: charge },
  Bodyguard: { when: { icon: "broken-heart", label: "Ally hit" }, headline: { icon: "shield" } },
  Chrysalis: { when: { icon: "sunrise", label: "Turn 3" }, headline: { icon: "summon" } },
  Trickster: { when: { icon: "flag", label: "Battle" }, headline: { icon: "swap" } },
  Gravedigger: { when: { icon: "sunrise", label: "Turn start" }, headline: { icon: "burst" } },
  Conductor: { when: { icon: "sunrise", label: "Every 3rd" }, headline: { icon: "repeat" } },
  Thornback: { when: { icon: "broken-heart", label: "Is hit" }, headline: { icon: "burst", amount: "1" } },
  "Coin Thief": { when: { icon: "sunrise", label: "Turn start" }, headline: { icon: "curse", amount: "1" } },
  Lantern: { when: { icon: "sunrise", label: "Turn start" }, headline: { icon: "shield", amount: "1" } },
  Hornet: { when: { icon: "swords", label: "Strikes" }, headline: { icon: "drop", amount: "1" } },
  Prism: { when: { icon: "flask", label: "Hexed" }, headline: { icon: "swap" } },
  "War Horn": { when: { icon: "sunrise", label: "Few allies" }, headline: { icon: "fist", amount: "1" } },
  Vulture: { when: { icon: "skull", label: "Any falls" }, headline: { icon: "fist", amount: "1" } },
  Hexer: { when: { icon: "sunrise", label: "Turn 2" }, headline: { icon: "silence" } },
  "Fortune Teller": { when: { icon: "flag", label: "Battle" }, headline: { icon: "fist", amount: "1" } },
  Aegis: { when: { icon: "broken-heart", label: "First hit" }, headline: { icon: "shield" } },
  Gravecaller: { when: { icon: "skull", label: "Ally falls" }, headline: { icon: "grave", amount: "1" } },
  Hydra: { when: { icon: "skull", label: "Dies" }, headline: { icon: "summon", amount: "×2" } },
  Archmage: { when: { icon: "moon", label: "Turn end" }, headline: { icon: "burst" }, counter: charge },
  Oracle: { when: { icon: "sunrise", label: "From turn 3" }, headline: { icon: "curse", amount: "1" } },
  Berserker: { when: { icon: "swords", label: "Strikes" }, headline: { icon: "burst", amount: "+" } },
  Frostling: { when: { icon: "swords", label: "Strikes" }, headline: { icon: "snowflake", amount: "1" } },
  Priest: { when: { icon: "flag", label: "Battle" }, headline: { icon: "halo" } },
  Battery: { when: { icon: "broken-heart", label: "Ally hit" }, headline: { icon: "shield", amount: "1" }, counter: charge },
  "Frost Witch": { when: { icon: "sunrise", label: "Turn 2" }, headline: { icon: "snowflake", amount: "1" } },
  Bully: { when: { icon: "swords", label: "Strikes" }, headline: { icon: "burst", amount: "×2" } },
  Alchemist: { when: { icon: "moon", label: "Turn end" }, headline: { icon: "drop", amount: "+1" } },
  Executioner: { when: { icon: "skull", label: "Kills" }, headline: { icon: "repeat" } },
  "Plague Rat": { when: { icon: "skull", label: "Dies" }, headline: { icon: "drop", amount: "1" } },
  Martyr: { when: { icon: "skull", label: "Dies" }, headline: { icon: "fist", amount: "½" } },
  Ox: { when: { icon: "swords", label: "Strikes" }, headline: { icon: "burst", amount: "HP" } },
  Daredevil: { when: { icon: "swords", label: "Unhurt" }, headline: { icon: "burst", amount: "×2" } },
  Duelist: { when: { icon: "swords", label: "Strikes" }, headline: { icon: "mirror" } },
  Paladin: { when: { icon: "swords", label: "Strikes" }, headline: { icon: "shield", amount: "+" } },
  Squire: { when: { icon: "shield", label: "Ally shield" }, headline: { icon: "fist", amount: "1" } },
  Dynamo: { when: { icon: "swords", label: "Strikes" }, headline: { icon: "repeat" }, counter: charge },
  Avenger: { when: { icon: "skull", label: "Ally dies" }, headline: { icon: "burst" } },
  Siphon: { when: { icon: "plus", label: "Enemy heal" }, headline: { icon: "swap" } },
  Comet: { when: { icon: "sunrise", label: "Every 4th" }, headline: { icon: "burst", amount: "T" } },
  Reaper: { when: { icon: "sunrise", label: "Few foes" }, headline: { icon: "burst" } },
  Heckler: { when: { icon: "scroll", label: "Foe acts" }, headline: { icon: "burst" } },
  "Witch Doctor": { when: { icon: "flask", label: "Hexed" }, headline: { icon: "plus" }, counter: charge },
  Broodmother: { when: { icon: "sunrise", label: "Every 2nd" }, headline: { icon: "egg" } },
  Dragon: { when: { icon: "swords", label: "Strikes" }, headline: { icon: "burst", amount: "½" } },
  Kraken: { when: { icon: "moon", label: "Turn end" }, headline: { icon: "burst" } },
  Puppeteer: { when: { icon: "sunrise", label: "Even turns" }, headline: { icon: "swap" } },
  Phoenix: { when: { icon: "skull", label: "Dies" }, headline: { icon: "repeat" } },
  Volcano: { when: { icon: "sunrise", label: "Every 2nd" }, headline: { icon: "burst", amount: "1" } },
};

/** A when tab from the trigger alone: what a unit with no picked face shows
 * (a new idea before its designer writes one, a v1 unit). */
export function whenFallback(when: When[]): Face["when"] {
  const w = when[0];
  if (!w || w.kind !== "trigger") return { icon: "infinity", label: "Always" };
  const p = w.on as { on: string; unit?: string; striker?: string; status?: string };
  const ally = p.unit === "otherAlly" || p.unit === "ally" || p.striker === "otherAlly";
  const enemy = p.unit === "enemy";
  switch (p.on) {
    case "BattleStart":
      return { icon: "flag", label: "Battle" };
    case "TurnStart":
      return { icon: "sunrise", label: "Turn start" };
    case "TurnEnd":
      return { icon: "moon", label: "Turn end" };
    case "Strike":
      return { icon: "swords", label: ally ? "Ally strike" : "Strikes" };
    case "Hurt":
      return { icon: "broken-heart", label: ally ? "Ally hit" : enemy ? "Enemy hit" : "Is hit" };
    case "Death":
      return { icon: "skull", label: ally ? "Ally dies" : enemy ? "Enemy dies" : "Dies" };
    case "Heal":
      return { icon: "plus", label: ally ? "Ally healed" : enemy ? "Enemy heal" : "Healed" };
    case "Summon":
      return { icon: "egg", label: "Summon" };
    case "StatusApplied":
      return { icon: "flask", label: enemy ? "Hexed" : p.status ? p.status.slice(0, FACE_LABEL_MAX) : "Status" };
    case "StatChanged":
      return { icon: "fist", label: ally ? "Ally grows" : "Grows" };
    default:
      return { icon: "spark", label: "Acts" };
  }
}

/** A face from the form alone: the trigger's when, a plain sparkle headline. */
export function faceFallback(form: Pick<UnitForm, "when">): Face {
  return { when: whenFallback(form.when), headline: { icon: "spark" } };
}

/** A fusion's face: the first part's when and headline (and its counter). */
export function fusedFace(first: Face): Face {
  return structuredClone(first);
}

/** Why a face doesn't fit the piece, or [] when it does. */
export function faceProblems(face: Face | undefined): string[] {
  if (!face) return ["no face"];
  const out: string[] = [];
  const letters = (s: string) => [...s.replace(/\s/g, "")].length;
  if (!face.when?.icon) out.push("no when icon");
  else if (!isGlyphName(face.when.icon)) out.push(`when icon "${String(face.when.icon).slice(0, 24)}" is not a glyph name`);
  if (!face.when?.label) out.push("no when label");
  else if (letters(face.when.label) > FACE_LABEL_MAX) out.push(`when label "${face.when.label}" is over ${FACE_LABEL_MAX} letters`);
  if (!face.headline?.icon) out.push("no headline icon");
  else if (!isGlyphName(face.headline.icon)) out.push(`headline icon "${String(face.headline.icon).slice(0, 24)}" is not a glyph name`);
  if (face.headline?.amount != null && (face.headline.amount === "" || [...face.headline.amount].length > FACE_AMOUNT_MAX)) out.push(`headline amount "${face.headline.amount}" is over ${FACE_AMOUNT_MAX} characters`);
  if (face.counter && !(face.counter.max >= 1 && face.counter.max <= 8)) out.push("counter max out of 1–8");
  return out;
}

/** A face with its icons as glyph names: a name stays, a known emoji (an older save, a designer that still writes one) maps through the table, anything else stays as it is for faceProblems to refuse. */
export function normalizeFace(face: Face): Face {
  const fix = (icon: string) => knownGlyph(icon) ?? icon;
  return { ...face, when: { ...face.when, icon: fix(face.when?.icon) }, headline: { ...face.headline, icon: fix(face.headline?.icon) } };
}

export { GLYPH_NAMES, type GlyphName };
