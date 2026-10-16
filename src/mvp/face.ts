// The piece's face (mission #1286, slice 1, makscee/void-board#1291): the
// hints a designer picks so a unit reads at a glance without its rule. A when
// tab (an icon and a short word), one headline badge (an icon and maybe a
// number) and, for a unit that builds a counter, a ring. They are free-form
// data, never a trigger/target/effect grammar: any odd future rule picks its
// own words. The rule itself stays in words, in the detail sheet only.
//
// The design: https://m1.twin-pogona.ts.net/r/arena-redesign/final/

import type { When } from "../types.js";
import type { UnitForm } from "./contract.js";

export interface Face {
  /** The when tab: any icon and a word of at most FACE_LABEL_MAX letters. */
  when: { icon: string; label: string };
  /** One icon that sums up the rule, and an optional short amount ("1", "×2", "T"). */
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
  Hourglass: { when: { icon: "🌙", label: "Turn end" }, headline: { icon: "🔁" }, counter: charge },
  Mirror: { when: { icon: "🚩", label: "Battle" }, headline: { icon: "📜" } },
  Candle: { when: { icon: "💀", label: "Would die" }, headline: { icon: "❤️", amount: "1" } },
  Gambler: { when: { icon: "⚔️", label: "Strikes" }, headline: { icon: "💥", amount: "×2" } },
  Nautilus: { when: { icon: "💔", label: "Is hit" }, headline: { icon: "🛡️", amount: "1" } },
  Magnet: { when: { icon: "🧪", label: "Ally hexed" }, headline: { icon: "🔀" } },
  Raven: { when: { icon: "💀", label: "Ally dies" }, headline: { icon: "📜" } },
  Seed: { when: { icon: "🌅", label: "From turn 3" }, headline: { icon: "💪", amount: "1" } },
  Sniper: { when: { icon: "🌅", label: "Turn start" }, headline: { icon: "💥" } },
  Medic: { when: { icon: "💔", label: "Ally hit" }, headline: { icon: "💚" } },
  Leech: { when: { icon: "⚔️", label: "Strikes" }, headline: { icon: "💚", amount: "½" } },
  "Lone Wolf": { when: { icon: "🐺", label: "When alone" }, headline: { icon: "💥", amount: "×2" } },
  Drummer: { when: { icon: "⚔️", label: "Ally strike" }, headline: { icon: "💪", amount: "1" }, counter: charge },
  Bodyguard: { when: { icon: "💔", label: "Ally hit" }, headline: { icon: "🛡️" } },
  Chrysalis: { when: { icon: "🌅", label: "Turn 3" }, headline: { icon: "🦋" } },
  Trickster: { when: { icon: "🚩", label: "Battle" }, headline: { icon: "🔀" } },
  Gravedigger: { when: { icon: "🌅", label: "Turn start" }, headline: { icon: "💥" } },
  Conductor: { when: { icon: "🌅", label: "Every 3rd" }, headline: { icon: "🔁" } },
  Thornback: { when: { icon: "💔", label: "Is hit" }, headline: { icon: "💥", amount: "1" } },
  "Coin Thief": { when: { icon: "🌅", label: "Turn start" }, headline: { icon: "🔻", amount: "1" } },
  Lantern: { when: { icon: "🌅", label: "Turn start" }, headline: { icon: "🛡️", amount: "1" } },
  Hornet: { when: { icon: "⚔️", label: "Strikes" }, headline: { icon: "💧", amount: "1" } },
  Prism: { when: { icon: "🧪", label: "Hexed" }, headline: { icon: "🔀" } },
  "War Horn": { when: { icon: "🌅", label: "Few allies" }, headline: { icon: "💪", amount: "1" } },
  Vulture: { when: { icon: "💀", label: "Any falls" }, headline: { icon: "💪", amount: "1" } },
  Hexer: { when: { icon: "🌅", label: "Turn 2" }, headline: { icon: "🤫" } },
  "Fortune Teller": { when: { icon: "🚩", label: "Battle" }, headline: { icon: "💪", amount: "1" } },
  Aegis: { when: { icon: "💔", label: "First hit" }, headline: { icon: "🛡️" } },
  Gravecaller: { when: { icon: "💀", label: "Ally falls" }, headline: { icon: "🪦", amount: "1" } },
  Hydra: { when: { icon: "💀", label: "Dies" }, headline: { icon: "🐲", amount: "×2" } },
  Archmage: { when: { icon: "🌙", label: "Turn end" }, headline: { icon: "💥" }, counter: charge },
  Oracle: { when: { icon: "🌅", label: "From turn 3" }, headline: { icon: "🔻", amount: "1" } },
  Berserker: { when: { icon: "⚔️", label: "Strikes" }, headline: { icon: "💥", amount: "+" } },
  Frostling: { when: { icon: "⚔️", label: "Strikes" }, headline: { icon: "❄️", amount: "1" } },
  Priest: { when: { icon: "🚩", label: "Battle" }, headline: { icon: "😇" } },
  Battery: { when: { icon: "💔", label: "Ally hit" }, headline: { icon: "🛡️", amount: "1" }, counter: charge },
  "Frost Witch": { when: { icon: "🌅", label: "Turn 2" }, headline: { icon: "❄️", amount: "1" } },
  Bully: { when: { icon: "⚔️", label: "Strikes" }, headline: { icon: "💥", amount: "×2" } },
  Alchemist: { when: { icon: "🌙", label: "Turn end" }, headline: { icon: "💧", amount: "+1" } },
  Executioner: { when: { icon: "💀", label: "Kills" }, headline: { icon: "🔁" } },
  "Plague Rat": { when: { icon: "💀", label: "Dies" }, headline: { icon: "💧", amount: "1" } },
  Martyr: { when: { icon: "💀", label: "Dies" }, headline: { icon: "💪", amount: "½" } },
  Ox: { when: { icon: "⚔️", label: "Strikes" }, headline: { icon: "💥", amount: "HP" } },
  Daredevil: { when: { icon: "⚔️", label: "Unhurt" }, headline: { icon: "💥", amount: "×2" } },
  Duelist: { when: { icon: "⚔️", label: "Strikes" }, headline: { icon: "🪞" } },
  Paladin: { when: { icon: "⚔️", label: "Strikes" }, headline: { icon: "🛡️", amount: "+" } },
  Squire: { when: { icon: "🛡️", label: "Ally shield" }, headline: { icon: "💪", amount: "1" } },
  Dynamo: { when: { icon: "⚔️", label: "Strikes" }, headline: { icon: "🔁" }, counter: charge },
  Avenger: { when: { icon: "💀", label: "Ally dies" }, headline: { icon: "💥" } },
  Siphon: { when: { icon: "💚", label: "Enemy heal" }, headline: { icon: "🔀" } },
  Comet: { when: { icon: "🌅", label: "Every 4th" }, headline: { icon: "💥", amount: "T" } },
  Reaper: { when: { icon: "🌅", label: "Few foes" }, headline: { icon: "💥" } },
  Heckler: { when: { icon: "📜", label: "Foe acts" }, headline: { icon: "💥" } },
  "Witch Doctor": { when: { icon: "🧪", label: "Hexed" }, headline: { icon: "💚" }, counter: charge },
  Broodmother: { when: { icon: "🌅", label: "Every 2nd" }, headline: { icon: "🥚" } },
  Dragon: { when: { icon: "⚔️", label: "Strikes" }, headline: { icon: "💥", amount: "½" } },
  Kraken: { when: { icon: "🌙", label: "Turn end" }, headline: { icon: "💥" } },
  Puppeteer: { when: { icon: "🌅", label: "Even turns" }, headline: { icon: "🔀" } },
  Phoenix: { when: { icon: "💀", label: "Dies" }, headline: { icon: "🔁" } },
  Volcano: { when: { icon: "🌅", label: "Every 2nd" }, headline: { icon: "💥", amount: "1" } },
};

/** A when tab from the trigger alone: what a unit with no picked face shows
 * (a new idea before its designer writes one, a v1 unit). */
export function whenFallback(when: When[]): Face["when"] {
  const w = when[0];
  if (!w || w.kind !== "trigger") return { icon: "♾️", label: "Always" };
  const p = w.on as { on: string; unit?: string; striker?: string; status?: string };
  const ally = p.unit === "otherAlly" || p.unit === "ally" || p.striker === "otherAlly";
  const enemy = p.unit === "enemy";
  switch (p.on) {
    case "BattleStart":
      return { icon: "🚩", label: "Battle" };
    case "TurnStart":
      return { icon: "🌅", label: "Turn start" };
    case "TurnEnd":
      return { icon: "🌙", label: "Turn end" };
    case "Strike":
      return { icon: "⚔️", label: ally ? "Ally strike" : "Strikes" };
    case "Hurt":
      return { icon: "💔", label: ally ? "Ally hit" : enemy ? "Enemy hit" : "Is hit" };
    case "Death":
      return { icon: "💀", label: ally ? "Ally dies" : enemy ? "Enemy dies" : "Dies" };
    case "Heal":
      return { icon: "💚", label: ally ? "Ally healed" : enemy ? "Enemy heal" : "Healed" };
    case "Summon":
      return { icon: "🥚", label: "Summon" };
    case "StatusApplied":
      return { icon: "🧪", label: enemy ? "Hexed" : p.status ? p.status.slice(0, FACE_LABEL_MAX) : "Status" };
    case "StatChanged":
      return { icon: "💪", label: ally ? "Ally grows" : "Grows" };
    default:
      return { icon: "✨", label: "Acts" };
  }
}

/** A face from the form alone: the trigger's when, a plain sparkle headline. */
export function faceFallback(form: Pick<UnitForm, "when">): Face {
  return { when: whenFallback(form.when), headline: { icon: "✨" } };
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
  if (!face.when?.label) out.push("no when label");
  else if (letters(face.when.label) > FACE_LABEL_MAX) out.push(`when label "${face.when.label}" is over ${FACE_LABEL_MAX} letters`);
  if (!face.headline?.icon) out.push("no headline icon");
  if (face.headline?.amount != null && (face.headline.amount === "" || [...face.headline.amount].length > FACE_AMOUNT_MAX)) out.push(`headline amount "${face.headline.amount}" is over ${FACE_AMOUNT_MAX} characters`);
  if (face.counter && !(face.counter.max >= 1 && face.counter.max <= 8)) out.push("counter max out of 1–8");
  return out;
}
