// The crude check of fusion names: profanity, sexual words, slurs and hate
// words, from a public list (LDNOOBW, ldnoobw-en.ts) plus the slurs it lacks,
// matched as substrings of the name's words joined, so glued forms are caught
// too ("Paedoking", "Gookrat", "Noct Scum"). The costs are lopsided: a name
// wrongly refused costs one more ask, a crude name stays in the world for good.
// Collateral is measured, not guessed: namer-corpus.ts holds innocent fantasy
// names that must all pass (fusions.test.ts), and MASKS grow only from it.
import { LDNOOBW_EN } from "./ldnoobw-en.js";

// Slurs, hate words and crude words LDNOOBW lacks: every crude entry of the
// hand lists before #594 (namer-corpus.ts CRUDE_BEFORE_594 holds them all),
// every name the #594 merge checks found, and toilet words (#609).
const EXTRA = [
  "fuk", "fuq", "fvck", "phuck", "phuk", "fck", "sht", "shite", "scum", "biatch", "beyotch", "yotch", "slut", "skank", "jism", "dildo",
  "pussies", "asshat", "asswipe", "assface", "asshead", "assclown", "asskick", "asslick", "badass", "dumbass", "jackass", "fatass",
  "smartass", "kickass", "arse", "prick", "bollock", "douche",
  "buttplug", "scrotum", "molest", "ballsack", "nutsack", "smegma", "sodom", "knobhead", "bellend", "sexslave", "orgasm", "masturbat",
  "necrophil", "lolicon", "pedo", "paedo", "pedophil", "paedophil", "pube", "turd", "twat", "dyke", "homo", "sperm", "testic", "incest",
  "boner", "erection", "felch", "queef", "minge", "choad", "mofo", "tosser", "wanker", "spunk", "semen", "milf", "hentai", "hooker",
  "lesbo", "queer", "gay", "fag", "fap", "thot", "cuck", "dong", "butt", "ass", "piss", "smut", "wank",
  // toilet and crude words
  "fart", "poop", "crap", "bum", "pimp", "perv", "gimp", "knob", "erect", "hymen",
  // slurs and hate
  "nig", "nigg", "nog", "negro", "kike", "kyke", "jew", "yid", "heeb", "hymie", "gook", "chink", "chingchong", "slanteye", "zipperhead",
  "jap", "paki", "raghead", "towelhead", "sandmonkey", "cameljockey", "jihadi", "kafir", "kaffir", "wetback", "beaner", "spic", "spick",
  "greaser", "gringo", "wop", "dago", "kraut", "honky", "honkey", "whitey", "wigger", "coon", "darkie", "darky", "jigaboo", "golliwog",
  "sambo", "porchmonkey", "junglebunny", "redskin", "injun", "squaw", "boong", "wog", "gyp", "gypo", "gyppo", "gypsy", "pikey",
  "chinaman", "coolie", "abo",
  "tranny", "trannie", "shemale", "ladyboy", "poof", "pouf", "poofter", "retard", "tard", "spastic", "spaz", "mongoloid", "mong",
  "cripple", "midget", "halfbreed", "dothead", "nazi", "hitler", "swastika", "kkk", "klan", "lynch", "heil", "siegheil", "fuhrer",
  "gestapo", "holocaust", "auschwitz", "gaschamber", "whitepower", "whitepride", "killgays", "nonce", "rapist", "rape", "orgy", "schlong",
];

// LDNOOBW entries left out, each only because a name of namer-corpus.ts needs
// it: "suck" (Bloodsucker, Honeysuckle), "snatch" (Soulsnatcher, Snatchwisp),
// "scat" (Scatterclaw), "poon" (Spoonbill).
const SKIP = new Set(["suck", "sucks", "snatch", "scat", "poon"]);

// The truly ambiguous short stems, refused only at a word's start
// ("Japslayer", "Cuckfang", "Assrat") or as its glued tail ("Kingdong",
// "Kingbutt"): anywhere they would refuse too much (Glass, Butterfly, Album).
// Every other stem matches anywhere, and MASKS excuse the innocent words.
const EDGE = new Set(["ass", "butt", "dong", "jap", "cuck", "thot", "fap", "horny", "bum", "knob"]);
// Refused only as a whole word or a glued tail ("Abo", "Kingabo"): at a
// word's start it is About, Above, Abomination.
const TAIL = new Set(["abo"]);

// Innocent words that hold a stem. A hit is excused only when one of these
// covers all of it ("Spicefang" keeps "spic" inside "spice"), so a mask never
// hides a stem beside it. Each entry names the stem it covers and the names
// of namer-corpus.ts that need it. A mask holds the innocent word's own
// letters around the stem, never a bare head's ending, so a stem glued to a
// fantasy head or tail stays seen ("rshti" hid Embershtide). "^" anchors a mask
// at a word's start and "$" at its end (a plural ending off): "^ashthorn"
// passes Ashthorn, not Aurashthorn; "monger$" passes Warmonger, not Mongeroid.
const MASKS = [
  "night", "knight", // nig: Nightshade, Knightfall
  "enigma", // nig: Enigma
  "benign", "nigni", // nig: Benign, Moonignite
  "jewel", // jew: Jewelwing
  "cockatrice", "cockatoo", // cock: beasts and birds
  "cocoon", "raccoon", "^racoon", "tycoon", // coon (the misspelt Racoon only at a word's start: never Auracoonrat)
  "cumul", "cucumber", "circum", "succumb", "talcum", "docum", "cumberso", // cum: Cumulus, Document, Cumbersome
  "scumul", // scum: Mosscumulus
  "spice", "spicy", "auspic", "conspic", // spic: Spicefang, Spicy
  "osemend", // semen: Rosemender
  "manali", "manale", "analog", "analy", // anal: Manalith, Manaleech, Analyst
  "bonereap", // boner: Bonereaper
  "mustard", "custard", "dastard", "stard", "tardigrad", // tard: Mustardseed, Stardust, Stardrake, Tardigrade
  "cuirass", "rrass", "glass", "brass", "grass", "class", "mass", "bass", "pass", "lass", "crass", "sass", "compass", "morass",
  "cutlass", "carcass", "assassin", "assail", "assault", "assay", "assemb", "assent", "assert", "assess", "asset", "assign", "assist",
  "associat", "assort", "assuag", "assum", "assur", // ass: Cuirass, Embarrass, Glass, Assassin
  "glassface", "glasshead", "grasshead", "brasshead", // assface, asshead: Glassface, Glasshead, Grasshead, Brasshead
  "butter", "button", "buttress", // butt: Butterfly
  "titan", "^tithe$", "title", "stitch", "titmouse", // tit: Titanfang, Stitchpunch, Titmouse, Tithe, Title (not Tithead, Titlord)
  "cuckoo", // cuck: Cuckoo-clock
  "sauerkraut", // kraut
  "pakistan", // paki
  "yiddish", // yid
  "japan", // jap
  "homogen", // homo
  "mongoose", "mongrel", "^among", // mong: beasts, Among (not Auramongrat)
  "basement", // semen
  "specialis", // cialis: Specialist
  "therapeu", "grapevine", "trapez", // rape: Therapeutic, Grapevine, Trapeze
  "starsee", "starser", "starsen", "warser", "warsen", "briarsee", // arse: Starseeker, Starserpent, Starsentinel, Warserpent, Warsentinel, Starseer, Briarseer
  "twinkle", "twinkling", "twinkly", // twink: Twinkle (not Twinklord)
  "boobytrap", // boob: Boobytrap
  "gypsum", // gyp: Gypsum
  "bumbl", "album", // bum: Bumblebee, Album
  "knobbl", // knob: Knobble
  "sexton", "sextant", // sex
  "scrap", // crap: Scrapper
  // Fantasy roots the R2-4b check found refused (51ad08c6), measured on namer-corpus.ts:
  "monger$", "mongering$", // mong: Warmonger, Ironmonger, Fearmonger (not Mongeroid)
  "rklance", "sklance", "ackland", "darkland", // klan: Darklance, Dusklance, Blackland, Darkland (never Klan+d… or Lunarklandrake)
  // sht: Ashthorn, Ashtalon, "Ash Titan", Ashtooth, Marshtide, Fleshtearer, Brushtail, Wishtide, Fishtail: the whole
  // root before it, Ash only at a word's start (never Ashthead, Aurashthorn, Boneshter, Embershtide or Kingshtrat)
  "^ashthorn", "^ashtalon", "^ashtitan", "^ashtooth", "marsht", "flesht", "brusht", "wisht", "fisht",
  // tit: Fortitude, Altitude, Gratitude, Entity, Identity, Sanctity, Petition and the -tition, -titious words (not Bigtitude, Moontity)
  "fortitud", "certitud", "altitud", "multitud", "gratitud", "latitud", "beatitud", "attitud", "ptitud", "ctitud", "entity", "quantity",
  "sanctity", "antithes", "antithet", "titio",
  "nogre$", // nog: Moonogre, "Dragon Ogre", Thornogre: ogre as the tail (not Nogreaper, Lunarnogreaper)
  "egypt", // gyp: Egyptian
  "pervious", // perv: Impervious
  "pimpernel", // pimp: Pimpernel
  "farthing", "^farther", "^farthest", // fart: Farthing, Farther (not Farthead, Kingfarther)
  "bump", // bum: Bump, Bumper
  "harass", "kvass", // ass: Harass, Kvass
];

// Whole real words that may be a name on their own though they hold a stem
// ("Grape", "Therapist"); only the whole word, never a compound ("Vinegrape").
const REAL_WORDS = new Set([
  "grape", "drape", "scrape", "serape", "therapist", "canal", "banal", "analyst", "analysis", "cockle", "cockerel", "cockroach",
  "woodcock", "gamecock", "shuttlecock", "weathercock", "cocktail", "cockney", "dickens", "parapet", "torpedo", "pedometer",
  "encumber", "document", "modicum", "capsicum", "incumbent", "hearse", "coarse", "hoarse", "parse", "sparse", "arsenal", "arsenic",
  "scunthorpe", "peacock", "cockpit", "cumin", "farseer", "starseed", "marseille", "grapeshot",
]);
const ENDINGS = ["", "s", "es"];

function fold(s: string): string {
  return s.normalize("NFKD").replace(/[̀-ͯ]/g, "").toLowerCase().replace(/[^a-z]/g, "");
}

/** Stems from LDNOOBW and EXTRA, folded, SKIP left out. A stem that holds
 * another is kept: a mask that excuses the short one ("stard" for "tard")
 * must not excuse the long one ("bastard"). */
export const CRUDE_STEMS: { anywhere: readonly string[]; edge: readonly string[]; tail: readonly string[] } = (() => {
  const all = [...new Set([...LDNOOBW_EN, ...EXTRA].map(fold))].filter((s) => s.length >= 3 && !SKIP.has(s));
  const anywhere = all.filter((s) => !EDGE.has(s) && !TAIL.has(s));
  return { anywhere, edge: all.filter((s) => EDGE.has(s)), tail: all.filter((s) => TAIL.has(s)) };
})();

const MASK_RULES = MASKS.map((mask) => ({ text: mask.replace(/^\^|\$$/g, ""), start: mask.startsWith("^"), end: mask.endsWith("$") }));

/** Where the words start and end in their join; an end also sits before a plural ending. */
interface Bounds {
  starts: ReadonlySet<number>;
  ends: ReadonlySet<number>;
}

function bounds(words: readonly string[]): Bounds {
  const starts = new Set<number>();
  const ends = new Set<number>();
  let at = 0;
  for (const word of words) {
    starts.add(at);
    at += word.length;
    for (const end of ENDINGS) if (word.endsWith(end)) ends.add(at - end.length);
  }
  return { starts, ends };
}

function covered(text: string, at: number, length: number, edges: Bounds): boolean {
  for (const mask of MASK_RULES)
    for (let m = text.indexOf(mask.text, Math.max(0, at + length - mask.text.length)); m >= 0 && m <= at; m = text.indexOf(mask.text, m + 1))
      if (m + mask.text.length >= at + length && (!mask.start || edges.starts.has(m)) && (!mask.end || edges.ends.has(m + mask.text.length))) return true;
  return false;
}

function hits(text: string, edges: Bounds, stem: string, at: (i: number) => boolean): boolean {
  for (let i = text.indexOf(stem); i >= 0; i = text.indexOf(stem, i + 1)) if (at(i) && !covered(text, i, stem.length, edges)) return true;
  return false;
}

/** True when the name reads crude: an anywhere-stem in its words joined, or an
 * edge stem at a word's start or end, or a tail stem at its end (a plural
 * ending off), unless MASKS cover the hit (at a word's edge where anchored);
 * a name that is one REAL_WORDS word passes whole. */
export function isCrudeName(name: string): boolean {
  const words = name.replace(/([a-z])([A-Z])/g, "$1 $2").split(/[\s'’-]+/).map(fold).filter(Boolean);
  if (words.length === 0) return false;
  if (words.length === 1 && ENDINGS.some((end) => words[0]!.endsWith(end) && REAL_WORDS.has(words[0]!.slice(0, words[0]!.length - end.length)))) return false;
  const joined = words.join("");
  const edges = bounds(words);
  if (CRUDE_STEMS.anywhere.some((stem) => hits(joined, edges, stem, () => true))) return true;
  for (const word of words) {
    const own = bounds([word]);
    const tails = (stem: string) => ENDINGS.filter((end) => word.endsWith(end)).map((end) => word.length - end.length - stem.length);
    if (CRUDE_STEMS.edge.some((stem) => hits(word, own, stem, (i) => i === 0 || tails(stem).includes(i)))) return true;
    if (CRUDE_STEMS.tail.some((stem) => hits(word, own, stem, (i) => tails(stem).includes(i)))) return true;
  }
  return false;
}

/** True when any anywhere-stem is in the folded name, MASKS ignored: the
 * stricter check of machine-made stand-ins ("Sexmancer", "Hencummer"). */
export function hasCrudeStem(name: string): boolean {
  const folded = fold(name);
  return CRUDE_STEMS.anywhere.some((stem) => folded.includes(stem));
}
