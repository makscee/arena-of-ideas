// Codex v2 views (rules v2 §6, §7; mission #1043 slice 8, #1051): what the
// Codex UI draws. A pair of units is discovered (a claimed, canonical
// design), ready to claim (three sealed designs nobody has picked) or
// unforged (not designed yet: Wish puts it on tonight's list). The daily
// feed says what tonight's batch brought and who claimed what; the
// Dictionary lists the words found, with credit and lineage, and how many
// are forged but not yet found. The claim moment shows a ready pair's three
// sealed cards: the player picks one, and it is canonical with their name.
//
// The server builds the world's view (GET /codex, server/src/mvp/codex-view.ts)
// with the mappers below; the test scenes draw it from forgeFixture().
import type { FusionDesign } from "./codex.js";
import type { MvpContent, UnitContent } from "./contract.js";
import { formText } from "./form-text.js";

/** One text in both languages (unit and word texts aren't in the client catalog). */
export interface Both {
  en: string;
  ru: string;
}

export interface ForgeUnit {
  id: string;
  emoji: string;
  name: Both;
  hp: number;
  pwr: number;
  /** The signature sentence, sleeping form. */
  rule: Both;
}

export type PairState = "discovered" | "ready" | "unforged";

/** A designed fusion: a candidate before the claim, the canonical one after. */
export interface ForgeDesign {
  emoji: string;
  name: Both;
  rule: Both;
  flavour: Both;
  /** Which way the candidate leans (§5 rules for the designer). */
  lean: "a" | "b" | "twist";
  /** Words this design uses that no claimed design has used yet. */
  newWords?: Both[];
}

export interface ForgePair {
  a: string;
  b: string;
  state: PairState;
  /** discovered: the canonical design and its credit. */
  design?: ForgeDesign;
  claimedBy?: string;
  claimedAt?: string;
  /** unforged: wishes on tonight's list, and whether this player wished it. */
  wishes?: number;
  wished?: boolean;
  /** unforged: sealed until a word is forged ("waiting for a new word: steal"). */
  waitingWord?: Both;
}

export interface FeedItem {
  kind: "arrived" | "claimed" | "word";
  at: string;
  /** arrived: how many pairs tonight's batch made ready. */
  n?: number;
  pair?: [string, string];
  name?: Both;
  by?: string;
  word?: Both;
}

export interface WordEntry {
  word: Both;
  /** What it does, one line. */
  means: Both;
  /** composed: a macro over primitives; forged: a new primitive. */
  kind: "composed" | "forged";
  foundBy: string;
  foundIn: { pair: [string, string]; name: Both };
  foundAt: string;
  /** The words it is made of (composed), and the designs that used it since. */
  of?: string[];
  usedIn: Both[];
}

export interface ForgeView {
  viewer: string;
  units: ForgeUnit[];
  pairs: ForgePair[];
  feed: FeedItem[];
  words: WordEntry[];
  /** Words forged that no claimed design has used yet. */
  forgedNotFound: number;
  /** Their names, when the world shows them (the Dictionary's sealed list). */
  forged?: Both[];
}

/** The claim moment: a ready pair and its three sealed candidates. */
export interface ClaimView {
  pair: [string, string];
  candidates: ForgeDesign[];
}

const b = (en: string, ru: string): Both => ({ en, ru });

type Texts = Pick<MvpContent, "abilities" | "statuses">;
/** A form as one sentence in both languages ("" when a word isn't in the registry). */
function sentence(form: UnitContent["forms"]["sleeping"] | undefined, content: Texts): Both {
  const one = (lang?: "ru"): string => {
    if (!form) return "";
    try {
      return formText(form, content.abilities, lang, content.statuses);
    } catch {
      return form.text ?? "";
    }
  };
  return { en: one(), ru: one("ru") };
}

/** A pool unit as the Codex shows it: its sleeping sentence, its RU name when translated. */
export function forgeUnitOf(u: UnitContent, content: Texts): ForgeUnit {
  return { id: u.id, emoji: u.emoji, name: b(u.name, u.texts?.ru?.name ?? u.name), hp: u.base.hp, pwr: u.base.pwr, rule: sentence(u.forms.sleeping, content) };
}

/** A designed fusion as the Codex shows it: its own RU name and flavour when it has them. */
export function forgeDesignOf(d: FusionDesign, pair: [UnitContent | undefined, UnitContent | undefined], content: Texts, newWords: string[] = []): ForgeDesign {
  const ru = d.texts?.ru;
  return {
    emoji: d.emoji ?? pair.map((u) => u?.emoji ?? "").join(""),
    name: b(d.name, ru?.name ?? d.name),
    // its own coined words read as sentences, never as ids (#1120: "keep inherit")
    rule: sentence(d.recipe, { abilities: { ...content.abilities, ...d.abilities }, statuses: { ...content.statuses, ...d.statuses } }),
    flavour: b(d.flavour ?? "", ru?.flavour ?? d.flavour ?? ""),
    lean: d.lean,
    ...(newWords.length ? { newWords: newWords.map((w) => b(w, w)) } : {}),
  };
}

/** The eight units of the design page (§4) and their partners in every state. */
export function forgeFixture(viewer = "you"): ForgeView {
  const units: ForgeUnit[] = [
    { id: "hourglass", emoji: "⏳", name: b("Hourglass", "Песочные часы"), hp: 8, pwr: 1, rule: b("At turn end, adds a grain; at 3 grains, spends them and the ally behind it acts again.", "В конце хода добавляет песчинку; на 3 песчинках тратит их, и союзник позади действует снова.") },
    { id: "mirror", emoji: "🪞", name: b("Mirror", "Зеркало"), hp: 6, pwr: 1, rule: b("At battle start, takes the rule of the enemy in front of it.", "В начале боя берёт правило врага перед собой.") },
    { id: "candle", emoji: "🕯️", name: b("Candle", "Свеча"), hp: 3, pwr: 3, rule: b("When it would die, it stays at 1 HP and loses 1 power instead; at 0 power it dies.", "Когда должна умереть, остаётся с 1 ОЗ и теряет 1 АТК; при 0 АТК умирает.") },
    { id: "gambler", emoji: "🎲", name: b("Gambler", "Игрок"), hp: 8, pwr: 2, rule: b("When it strikes, it flips: double damage or none.", "Когда бьёт, бросает монету: двойной урон или никакого.") },
    { id: "nautilus", emoji: "🐚", name: b("Nautilus", "Наутилус"), hp: 10, pwr: 1, rule: b("When hurt, gains shield equal to the hits it has taken this battle.", "Получая урон, получает щит, равный числу ударов по нему за бой.") },
    { id: "magnet", emoji: "🧲", name: b("Magnet", "Магнит"), hp: 12, pwr: 1, rule: b("Enemy effects aimed at a random ally hit Magnet instead.", "Вражеские эффекты по случайному союзнику попадают в Магнит.") },
    { id: "raven", emoji: "🐦‍⬛", name: b("Raven", "Ворон"), hp: 8, pwr: 1, rule: b("When an ally dies, remembers its rule; at the next turn start, performs it once.", "Когда союзник умирает, запоминает его правило; в начале следующего хода исполняет его раз.") },
    { id: "seed", emoji: "🌱", name: b("Seed", "Семя"), hp: 4, pwr: 0, rule: b("Does nothing for two turns, then becomes a Tree that gives the ally behind it 1 power each turn.", "Два хода ничего не делает, потом становится Деревом, которое каждый ход даёт союзнику позади 1 АТК.") },
  ];
  const d = (emoji: string, name: Both, rule: Both, flavour: Both, lean: ForgeDesign["lean"] = "twist", newWords?: Both[]): ForgeDesign => ({ emoji, name, rule, flavour, lean, ...(newWords ? { newWords } : {}) });
  const dejaVu = d("⏳🪞", b("Déjà Vu", "Дежавю"), b("Whatever the enemy in front of it does, Déjà Vu does one turn later.", "Что бы ни сделал враг перед ним, Дежавю делает то же ходом позже."), b("It has seen this fight before.", "Этот бой он уже видел."));
  const graveTree = d("🐦‍⬛🌱", b("Grave Tree", "Могильное древо"), b("When an ally dies, a sapling grows in its place with the fallen ally's rule.", "Когда союзник умирает, на его месте вырастает росток с правилом павшего."), b("Nothing here is ever really gone.", "Здесь ничто не уходит насовсем."));
  const wick = d("🕯️🎲", b("Wick", "Фитиль"), b("When it would die, it flips: heads, it stays at 1 HP and strikes twice; tails, it dies.", "Когда должен умереть, бросает монету: орёл — остаётся с 1 ОЗ и бьёт дважды; решка — умирает."), b("Every flame is a bet.", "Каждое пламя — ставка."));
  const pairs: ForgePair[] = [
    { a: "hourglass", b: "mirror", state: "discovered", design: dejaVu, claimedBy: "kim", claimedAt: "2026-10-12" },
    { a: "raven", b: "seed", state: "discovered", design: graveTree, claimedBy: viewer, claimedAt: "2026-10-13" },
    { a: "candle", b: "gambler", state: "discovered", design: wick, claimedBy: "lena", claimedAt: "2026-10-13" },
    { a: "hourglass", b: "gambler", state: "ready" },
    { a: "nautilus", b: "magnet", state: "ready" },
    { a: "hourglass", b: "candle", state: "ready" },
    { a: "hourglass", b: "raven", state: "unforged", wishes: 4, wished: true },
    { a: "hourglass", b: "seed", state: "unforged", wishes: 1 },
    { a: "hourglass", b: "nautilus", state: "unforged", wishes: 0, waitingWord: b("steal", "кража") },
    { a: "hourglass", b: "magnet", state: "unforged", wishes: 0 },
  ];
  const feed: FeedItem[] = [
    { kind: "arrived", at: "2026-10-13T03:10:00Z", n: 72 },
    { kind: "claimed", at: "2026-10-13T09:41:00Z", pair: ["candle", "gambler"], name: wick.name, by: "lena" },
    { kind: "word", at: "2026-10-13T09:41:00Z", word: b("gamble", "риск"), pair: ["candle", "gambler"], by: "lena" },
    { kind: "claimed", at: "2026-10-13T14:02:00Z", pair: ["raven", "seed"], name: graveTree.name, by: viewer },
    { kind: "arrived", at: "2026-10-12T03:05:00Z", n: 64 },
    { kind: "claimed", at: "2026-10-12T19:20:00Z", pair: ["hourglass", "mirror"], name: dejaVu.name, by: "kim" },
    { kind: "word", at: "2026-10-12T19:20:00Z", word: b("echo", "эхо"), pair: ["hourglass", "mirror"], by: "kim" },
  ];
  const words: WordEntry[] = [
    { word: b("echo", "эхо"), kind: "composed", of: ["remember", "delay", "perform"], means: b("Does what someone did, one turn later.", "Повторяет чужое действие ходом позже."), foundBy: "kim", foundIn: { pair: ["hourglass", "mirror"], name: dejaVu.name }, foundAt: "2026-10-12", usedIn: [dejaVu.name] },
    { word: b("gamble", "риск"), kind: "composed", of: ["flip", "twice"], means: b("Flips a coin: heads doubles it, tails loses it.", "Бросает монету: орёл удваивает, решка теряет."), foundBy: "lena", foundIn: { pair: ["candle", "gambler"], name: wick.name }, foundAt: "2026-10-13", usedIn: [wick.name] },
    { word: b("inherit", "наследие"), kind: "forged", means: b("A new unit takes a fallen unit's rule.", "Новый юнит берёт правило павшего."), foundBy: viewer, foundIn: { pair: ["raven", "seed"], name: graveTree.name }, foundAt: "2026-10-13", usedIn: [graveTree.name] },
  ];
  return { viewer, units, pairs, feed, words, forgedNotFound: 3 };
}

/** Hourglass + Gambler's three sealed candidates: lean A, lean B, the twist (with a new word). */
export function claimFixture(): ClaimView {
  return {
    pair: ["hourglass", "gambler"],
    candidates: [
      { emoji: "⏳🎲", lean: "a", name: b("Sandglass Bet", "Ставка на песок"), rule: b("At turn end, adds a grain; at 3 grains, the ally behind it acts again with double power.", "В конце хода добавляет песчинку; на 3 песчинках союзник позади действует снова с двойной АТК."), flavour: b("Patience pays double.", "Терпение окупается вдвойне.") },
      { emoji: "⏳🎲", lean: "b", name: b("Late Coin", "Поздняя монета"), rule: b("When it strikes, it flips: heads, double damage now; tails, the damage lands a turn later.", "Когда бьёт, бросает монету: орёл — двойной урон сейчас; решка — урон приходит ходом позже."), flavour: b("Every debt comes due.", "Любой долг вернётся.") },
      { emoji: "⏳🎲", lean: "twist", name: b("Metronome", "Метроном"), rule: b("Each turn end flips: heads adds two grains, tails removes one; at 3 grains the whole line acts again in order.", "В конце каждого хода бросает монету: орёл — две песчинки, решка — минус одна; на 3 песчинках вся линия действует снова по порядку."), flavour: b("Tick, tock, roll.", "Тик, так, бросок."), newWords: [b("tempo", "темп")] },
    ],
  };
}
