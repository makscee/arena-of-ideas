// Fusion names (mission #574). Slice 10 owns this file: the local model on m1
// behind a blocklist, the stored discoveries (MvpStore.fusion, putFusion) and
// GET /fusions. mvpRuntime (./runtime.ts) wires fusionNaming() once, so slice
// 10 fills it in here and never edits decide() or the routes. The rules are on
// FusionDiscovery in the contract: a pair's name is fixed the first time
// anyone, bot or human, fuses it, and bots are never credited.
//
// How a pair gets its name, a human never waiting on the model:
// - onDecision queues both orders of every fusable pair on any line that has
//   no stored name yet, humans' pairs ahead of bots'; the naming job asks the
//   model and keeps the answer here ("prepared"), so peek and the fuse use it.
// - The first fuse stores the prepared model name, else the portmanteau
//   ("fallback"), for good; onFuse records it (recordFusion).
// - Pairs never share a name: a new pair's name is never a stored fusion's or
//   a base unit's (compared folded). A model answer that clashes is refused
//   like a blocked one, and the portmanteau moves its split (portmanteau).
//   Names stored before this rule stay as they are.
// - A bot fuses a pair only once fusionNameReady: its name stored or
//   prepared, or the model gave up on it. Bots run in the background, so a
//   bot that wants a pair whose name isn't ready waits for it
//   (awaitFusionName), and the pair is asked next, ahead of every queue. A model that is down, slow
//   or times out (a 5xx included) is asked again with a backoff. A final
//   failure is MODEL_FAILURES failed asks in a row over at least MODEL_DOWN_MS:
//   on one pair, the model gives up on it for a while; on any pairs, the
//   model counts as down. So a model still loading (503s for a few seconds) is
//   waited for. While it is down no bot waits, new pairs aren't asked, and one
//   pair is asked every MODEL_PROBE_MS; the first answer brings it back.
import { fuseCheck } from "../../../src/mvp/forms.js";
import type { FuseContext, FusionDiscovery, FusionParts, PlayerRef, UnitContent, UnitId } from "../../../src/mvp/contract.js";
import type { RunDeps, RunHooks } from "./runs.js";
import type { MvpJob } from "./runtime.js";
import type { MvpStore } from "./store.js";

/** Names an ordered pair for a fuse by `by`, synchronously. */
export type NameFusion = (first: UnitContent, second: UnitContent, by: PlayerRef) => FuseContext;

/** Asks the model for a raw name; null or a throw means no name. */
export type ModelNamer = (first: UnitContent, second: UnitContent) => Promise<string | null>;

/** The deterministic fallback name: the front of first's name and the back of
 * second's ("Brawler" + "Medic" → "Brawdic"). The only portmanteau. It is
 * never a part's name or one of `taken` (the base units' names and the stored
 * fusions' names, compared folded): the split moves until it isn't ("Rose" +
 * "Rot" → "Rorot", not "Rot"; a second "Sileat" → "Silenat"), then any split,
 * then both names joined whole, and as a last resort a numeral ("Sileat II"). */
export function portmanteau(first: string, second: string, taken: Iterable<string> = []): string {
  const avoid = new Set([...taken].map(fold));
  return portmanteauAvoiding(first, second, (folded) => avoid.has(folded));
}

/** portmanteau(), `isTaken` answering for folded names (peek passes its set
 * of taken names, so no copy of it is made per fuse). */
function portmanteauAvoiding(first: string, second: string, isTaken: (folded: string) => boolean): string {
  const a = first.replace(/\s+/g, "");
  const b = second.replace(/\s+/g, "").toLowerCase();
  const parts = new Set([first, second].map(fold));
  const free = (name: string) => !parts.has(fold(name)) && !isTaken(fold(name)) && !isBlockedName(name) && !STANDIN_CRUDE.some((c) => fold(name).includes(c));
  const join = (head: string, tail: string) => (head.slice(-1).toLowerCase() === tail[0] ? head + tail.slice(1) : head + tail);
  const h0 = Math.ceil(a.length / 2);
  const t0 = Math.floor(b.length / 2);
  const heads: number[] = [];
  for (let dh = 0; dh < a.length; dh++) for (const h of dh === 0 ? [h0] : [h0 + dh, h0 - dh]) if (h >= 1 && h <= a.length) heads.push(h);
  for (const h of heads)
    for (const t of [t0, t0 - 1, t0 + 1]) {
      if (t < 0 || t >= b.length) continue;
      const name = join(a.slice(0, h), b.slice(t));
      if (free(name)) return name;
    }
  // Every split: the tail moves further from the middle too.
  for (const h of heads)
    for (let t = 0; t < b.length; t++) {
      const name = join(a.slice(0, h), b.slice(t));
      if (free(name)) return name;
    }
  if (free(a + b)) return a + b;
  // Last resort: a numeral after the middle split, or after the first part's
  // name when that split is itself blocked (a numeral can't unblock it).
  const mid = join(a.slice(0, h0), b.slice(Math.min(t0, b.length - 1)));
  const base = isBlockedName(mid) ? first : mid;
  for (let k = 2; k < 1000; k++) {
    const name = `${base} ${ROMAN[k] ?? k}`;
    if (free(name)) return name;
  }
  return `${first} ${second}`;
}

/** A stand-in is made by machine and can take another split, so it is held
 * stricter than a model's answer: no crude fragment anywhere ("Sexmancer",
 * "Hencummer" from Sexton and War Drummer). */
const STANDIN_CRUDE = ["sex", "cum", "anal", "anus", "arse", "boner", "nig", "fag", "spic", "kike", "kyke", "rape", "porn", "piss", "dick",
  "cock", "cunt", "fuck", "shit", "slut", "smut", "jizz", "homo", "dyke", "coon", "jew", "nazi", "turd", "twat", "wank", "pube", "tard", "poof",
  "pedo", "clit", "semen", "gay"];

const ROMAN = ["", "I", "II", "III", "IV", "V", "VI", "VII", "VIII", "IX", "X", "XI", "XII"];

/** The stored name, else the portmanteau, with the credit by FusionDiscovery's
 * rule (the stored discoverer, else `by` unless it is a bot). A new pair's
 * portmanteau avoids the base units' names and every stored name. It only
 * reads: it never records a discovery or calls a model. */
export function storedOrPortmanteau(store: MvpStore, unitNames: readonly string[] = []): NameFusion {
  return (first, second, by) => {
    const known = store.fusion(first.id, second.id);
    return {
      name: known?.name ?? portmanteau(first.name, second.name, [...unitNames, ...store.fusions().map((f) => f.name)]),
      discoveredBy: known?.discoveredBy ?? (by.bot ? null : by),
    };
  };
}

// Franchise and character names that small models leak into titles (every
// model in the 2026-10 benchmark kept all six probes and 7–8 of 8 Hearthstone
// names), plus words that pretend to be official. Every list is matched after
// folding: accents stripped, lowercase, only letters and digits kept.
//
// Stems: distinctive enough to match anywhere inside one word of the name, so
// glued and inflected forms are caught too ("Darthvader", "Gandalfs",
// "Hogwart", "Lordvader"), once the ORDINARY words that contain them are taken
// out ("Invader", "Marionette"). A stem ending in "man" matches only at the end
// of a word ("Lordbatman", not "Batmancer"). Across words a stem matches only
// whole words joined ("Spider Man", not "Spider Mantis" or "Dart Hawk").
const BLOCKED_STEMS = [
  // games
  "pikachu", "pokemon", "charizard", "mewtwo", "luigi", "bowser", "yoshi", "wario", "zelda", "ganondorf",
  "eggman", "kirby", "samus", "megaman", "pacman", "minecraft",
  "fortnite", "vbucks", "masterchief", "cortana", "kratos", "geralt", "doomguy",
  "warcraft", "hearthstone", "azeroth", "jaina", "arthas", "illidan", "sylvanas", "garrosh", "guldan", "anduin",
  "nefarian", "ragnaros", "deathwing", "onyxia", "kelthuzad", "cthun", "yogg", "leeroy",
  "medivh", "kaelthas", "malfurion", "tyrande", "rexxar", "valeera",
  "roshan", "teemo", "yasuo", "overwatch", "reinhardt",
  "diablo", "starcraft", "kerrigan", "protoss", "skyrim", "dovahkiin", "tamriel", "sephiroth", "chocobo",
  "moogle", "aerith", "metalgear", "cuphead", "undertale",
  "terraria", "amogus", "roblox", "genshin", "paimon", "malenia", "dragonborn",
  "mindflayer", "illithid", "drizzt", "wrathion",
  // film, tv, books, comics
  "gandalf", "frodo", "bilbo", "sauron", "gollum", "smeagol", "aragorn", "legolas", "gimli", "saruman", "mordor", "hobbit", "balrog",
  "darth", "skywalker", "chewbacca", "chewie", "kenobi", "obiwan", "palpatine", "stormtrooper", "grogu", "mandalorian",
  "hogwart", "voldemort", "dumbledore", "hermione",
  "mufasa", "shrek", "pinocchio", "dumbo", "mickey",
  "batman", "superman", "aquaman", "gotham", "krypton", "spiderman",
  "thanos", "wolverine", "deadpool",
  "godzilla", "kingkong", "xenomorph", "dracula", "frankenstein", "007",
  "naruto", "sasuke", "totoro", "gundam", "ultraman", "megatron",
  "spongebob", "squidward", "garfield", "snoopy", "scooby", "simpson", "smurf", "barbie", "pinkie", "ratbert", "dilbert",
  "atreides", "harkonnen", "khaleesi", "targaryen", "lannister", "westeros", "dothraki",
  // pretending to be official, or spam
  "vader", "yoda", "jedi", "snape", "dobby", "pudge", "zerg", "goku", "optimus", "hulk",
  "mario", "thorhammer", "thorshammer",
  "moderator", "http", "www",
  // crude: profanity, slurs and hate words, as roots that also catch joined
  // forms ("Shitlord"); ORDINARY keeps the ordinary words that contain them.
  // Shorter ones that hit ordinary words match only as whole words (CRUDE_WORDS).
  "fuck", "fvck", "phuck", "shit", "cunt", "nigg", "negro", "faggot", "bitch", "biatch", "whore", "slut", "smut", "porn",
  "rape", "rapist", "pedophil", "paedophil", "dildo", "blowjob", "handjob", "cumshot", "cumslut", "cumdump", "cumlord", "cumbucket",
  "cumrag", "jizz", "jism", "vagina", "penis", "pussy", "pussies", "asshole", "arsehole", "asshat", "asswipe", "dumbass", "jackass",
  "fatass", "smartass", "badass", "kickass", "bastard", "dick", "cock", "anal", "boob", "prick", "wank", "twat", "piss", "nazi",
  "hitler", "swastika", "kkk", "klansman", "kike", "kyke", "retard", "chink", "wetback", "raghead", "towelhead", "tranny", "trannie",
  "shemale", "ladyboy", "bollock", "douche", "butthole", "buttplug", "scrotum", "gangbang", "molest", "skank", "yotch", "ballsack",
  "nutsack", "bukkake", "jewkill", "killjew", "whitepower", "siegheil", "heilhitler", "gaschamber", "holocaust", "auschwitz",
  "gestapo", "fuhrer", "redskin", "beaner", "golliwog", "jigaboo", "porchmonkey", "zipperhead", "slanteye", "mongoloid", "spastic",
  "lolicon", "necrophil", "bestiality", "masturbat", "orgasm", "knobhead", "bellend", "sexslave", "rimjob", "smegma", "sodom",
  // roots a fusion of this content makes likely (Pediatrician, Sexton) and
  // hate phrases: also caught joined ("Pedomancer", "Sexwraith", "Jew Slayer")
  "pedo", "sex", "cum", "jew", "clit", "tits", "arse", "fag", "assface", "asslick", "asskick", "assclown", "asshead",
  "whitepride", "chingchong", "junglebunny", "sandmonkey", "cameljockey", "killgays",
];
// Ordinary words that contain a stem: taken out of a word before stems match.
const ORDINARY = ["invader", "evader", "pervader", "hulking", "hulky", "marionette", "mariology", "mariolat", "marion", "snaper",
  // crude stems inside ordinary words
  "grape", "drape", "scrape", "crape", "trapez", "parapet", "therapist", "cockatoo", "cockatiel", "cockle", "cockscomb", "gamecock",
  "weathercock", "hitchcock", "cockatrice", "peacock", "cockpit", "cockerel", "woodcock", "cockroach", "shuttlecock", "cocktail",
  "hancock", "cockade", "cocky", "babcock", "dickens", "dickcissel", "analy", "analog", "analg", "analem", "banal", "canal",
  "bacchanal", "manal", "tanal", "hanal", "penistone", "scunthorpe", "swank", "niggl", "snigg", "niggard", "ashkenazi", "nazirite",
  "parse", "sparse", "coarse", "hoarse", "arsen", "booby", "prickl", "retardant", "chinkapin", "raccoon", "racoon", "cocoon", "tycoon",
  "puccoon", "pakistan", "negroni", "montenegro", "pussywillow", "fagot", "fagus", "titsch", "pissarro", "coonhound", "twattle", "shitake",
  "cocker", "cockney", "dicky", "dickory", "dickon", "pussycat", "rapeseed", "serape", "chinking", "shittim", "shitzu", "marseill", "ranald",
  "torpedo", "pedomet", "pedolog", "sexton", "sextant", "sextet", "sextup", "essex", "sussex", "wessex", "sexagen", "scum", "cumul",
  "cucum", "cumber", "circum", "docum", "cumin", "incumb", "succumb", "talcum", "modicum", "capsicum", "jewel", "clitheroe", "clitter",
  "petits", "tomtits", "farse", "hearse", "starse", "fagin", "fagot", "fagus"];
// Words: short or ordinary enough that a stem would hit real words ("Thorn",
// "Invader", "Marionette", "Smuggler", "Scamper"), so they match one word of
// the name, also with a plural or possessive ending ("Marios", "Thor's").
const BLOCKED_WORDS = [
  "ganon", "croft", "thrall", "uther", "gul", "longshot", "elise", "reno", "brann", "kael", "gnoll",
  "dota", "invoker", "ahri", "garen", "terran", "fallout", "tifa", "papyrus",
  "sith", "potter", "muggle",
  "elsa", "olaf", "nemo", "dory", "fiona", "minnie", "thor", "loki", "avenger", "marvel",
  "zoro", "bart", "lego", "barney", "scam", "developer", "administrator",
  // crude words that hit ordinary words as roots: whole words only
  "nig", "nog", "anus", "wop", "dago", "mofo", "choad", "fap", "gay", "gays",
  "abo", "arse", "arsehole", "ass", "asses", "asshat", "asshole", "asswipe", "auschwitz", "badass", "ballsack", "bastard", "bastards",
  "beaner", "beaners", "bellend", "bestiality", "beyotch", "biatch", "bitch", "bitches", "bollock", "bollocks", "boner", "boobies",
  "boobs", "boong", "bukkake", "bullshit", "buttplug", "chinaman", "chink", "cock", "cocks", "cocksucker", "coolie", "coon", "cripple",
  "cuck", "cum", "cumbucket", "cumming", "cumrag", "cumshot", "cumslut", "cunt", "cunts", "darkie", "darky", "dick", "dickhead", "dickwad",
  "dildo", "dothead", "dumbass", "dyke", "erection", "fag", "faggot", "fags", "fatass", "fck", "felch", "fuck", "fucker", "fuckface",
  "fucking", "fuhrer", "fuk", "fuq", "gestapo", "golliwog", "gook", "greaser", "gringo", "gyp", "gypo", "gyppo", "gypsy", "halfbreed",
  "heeb", "heil", "hencummer", "hentai", "hitler", "holocaust", "homo", "honkey", "honky", "horny", "hymie", "incest", "injun", "jackass",
  "jap", "jerkoff", "jewkiller", "jigaboo", "jihadi", "jizz", "kaffir", "kafir", "kickass", "kike", "kkk", "klan", "klansman", "knobhead",
  "kraut", "kyke", "ladyboy", "lesbo", "libtard", "lolicon", "lynch", "lyncher", "lynching", "masturbate", "masturbator", "midget", "milf",
  "minge", "molester", "mong", "mongoloid", "motherfucker", "nazi", "nazis", "necrophile", "negro", "niga", "nigga", "niggas", "nigger",
  "nigguh", "nonce", "nutsack", "orgasm", "orgy", "paedo", "paki", "pedo", "pedophile", "penis", "phuk", "pikey", "piss", "pissed",
  "pisser", "poof", "poofter", "poontang", "porchmonkey", "porn", "porno", "pouf", "prick", "pricks", "pussies", "pussy", "queef",
  "raghead", "rape", "raper", "rapist", "redskin", "retard", "retarded", "rimjob", "saboner", "sambo", "sandnigger", "schlong", "scrotum",
  "semen", "sex", "sexmancer", "sexslave", "sextim", "sexy", "shemale", "shit", "shite", "shithead", "shitskin", "sht", "siegheil",
  "skank", "slanteye", "slut", "sluts", "smartass", "smegma", "sodomite", "sodomy", "spastic", "spaz", "spick", "spics", "spunk", "squaw",
  "swastika", "tard", "testicle", "thot", "tits", "titties", "tosser", "towelhead", "trannie", "trannies", "tranny", "turd", "twat",
  "twats", "vagina", "wank", "wanker", "wetback", "whitepower", "whitey", "whore", "whores", "wigger", "wog", "wogs", "yid", "yids",
  "zipperhead",
  // the model's way of not naming
  "fusion", "fuse", "fused",
  // stems that hit ordinary words: Carambola, Adminicle, Hexmender, Bambino,
  // Benzothiazole, Twitcher, Pigroot, Fluffy, Revegetate, Magnetodynamo,
  // Hagride, Rhodora, Malandrous, Simball, Sleeveen, Exterminator,
  // Geometroid, Iron Mantle
  "rambo", "admin", "xmen", "bambi", "nzoth", "witcher", "groot", "luffy", "vegeta", "magneto", "hagrid", "hodor",
  "landro", "simba", "eevee", "terminator", "metroid", "ironman",
];
// Ordinary words that are also franchise names: blocked only as the whole name.
const BLOCKED_WHOLE = [
  "sonic", "joker", "swindle",
  "link", "tails", "knuckles", "donkey", "kong", "cloud", "kingdom", "solid", "snake", "resident", "nemesis", "hollow", "knight", "hornet", "sans",
  "souls", "league", "legends", "peach", "wonder", "flash", "spider", "rocket", "venom", "bond", "solo", "sailor", "homer", "ken", "steve", "stark",
  "harley", "donald", "goofy", "frozen", "halo", "doom", "dune", "lara", "elden", "rocky", "zed", "jinx", "tracer", "creeper", "harry", "tetris",
  "predator", "transformer", "sherlock", "mod", "dev", "free", "com", "system", "staff", "official", "approved", "hollow knight", "solid snake",
  "spic", "donkey kong", "wonder woman", "iron man", "spider man", "black widow", "captain america", "dark souls", "free vbucks",
].map(fold);
const WORD_ENDINGS = ["", "s", "es", "z"];

function fold(s: string): string {
  return stripAccents(s).toLowerCase().replace(/[^a-z0-9]/g, "");
}

function stripAccents(s: string): string {
  return s.normalize("NFKD").replace(/[\u0300-\u036f]/g, "");
}

/** True when a name (or one of its words) is on the blocklist. */
export function isBlockedName(name: string): boolean {
  if (BLOCKED_WHOLE.includes(fold(name))) return true;
  // CamelCase is split first, so glued words match too ("LordVader", "SuperMario").
  const words = name.replace(/([a-z])([A-Z])/g, "$1 $2").split(/[\s'’-]+/).map(fold).filter(Boolean);
  const named = (w: string, b: string) => WORD_ENDINGS.some((end) => w === b + end);
  for (let i = 0; i < words.length; i++) {
    const w = words[i]!;
    if (BLOCKED_WORDS.some((b) => named(w, b))) return true;
    const inner = ORDINARY.reduce((rest, o) => rest.split(o).join("."), w);
    if (BLOCKED_STEMS.some((stem) => (stem.endsWith("man") ? WORD_ENDINGS.some((end) => inner.endsWith(stem + end)) : inner.includes(stem)))) return true;
    // Whole words joined: "Spider Man", "Kel'Thuzad", "Iron Man".
    for (let j = i + 2; j <= words.length; j++) {
      const run = words.slice(i, j).join("");
      if (BLOCKED_STEMS.some((b) => named(run, b)) || BLOCKED_WORDS.some((b) => named(run, b))) return true;
    }
  }
  return false;
}

/** True when a name is nothing but the parts' names joined ("SquireGnat",
 * "Medic"): what is left after taking out every word of both names is under
 * three letters. */
function isGlued(name: string, first?: UnitContent, second?: UnitContent): boolean {
  const parts = [first, second].flatMap((u) => (u ? u.name.split(/\s+/).map(fold) : [])).filter((w) => w.length >= 3);
  if (parts.length === 0) return false;
  let rest = fold(name);
  for (const part of parts.sort((x, y) => y.length - x.length)) rest = rest.split(part).join("");
  return rest.length < 3;
}

const SMALL_WORDS = ["of", "the", "and"];

/** A model reply made into a name, or null when it isn't one we'd show:
 * 1–3 words of letters, 3–20 characters, nothing on the blocklist, and not
 * just the parts' names. */
export function cleanModelName(raw: string, first?: UnitContent, second?: UnitContent): string | null {
  const line = stripAccents(raw.replace(/<think>[\s\S]*?<\/think>/g, "")).trim().split("\n")[0] ?? "";
  const name = line
    .replace(/^(name|fusion|fused name)\s*:\s*/i, "")
    .replace(/["“”«»*_`.!]/g, "")
    .replace(/\s+/g, " ")
    .replace(/^[^A-Za-z0-9]+|[^A-Za-z0-9]+$/g, "");
  if (!/^[A-Za-z][A-Za-z' -]{2,19}$/.test(name)) return null;
  const words = name.split(/[ -]/).filter(Boolean);
  if (words.length > 3 || new Set(words.map(fold)).size < words.length) return null;
  if (isBlockedName(name) || isGlued(name, first, second)) return null;
  // Title case, but a joining word inside stays small: "Mender of Storms".
  const small = (w: string, i: number) => i > 0 && i < words.length - 1 && SMALL_WORDS.includes(w.toLowerCase());
  return words.map((w, i) => (small(w, i) ? w.toLowerCase() : w[0]!.toUpperCase() + w.slice(1))).join(name.includes("-") && words.length > 1 ? "-" : " ");
}

/** The namer's system prompt. Measured on m1's Qwen2.5-1.5B (#587): the
 * earlier prompt, with the parts' emoji in the ask, got mostly emoji or the
 * parts glued ("🌹 Rat", "RatRose"), which cleanModelName refuses. */
export const NAMER_SYSTEM =
  "You name creatures in a fantasy auto-battler. Two fighters merge into one new creature. Invent a fresh, evocative English name for it: one or two words, at most 18 letters, letters only. Do not just join or repeat the two fighters' names, use no emoji, no quotes, no explanation, and never a name from an existing game, film, book or comic.";

/** Few-shot turns before the real ask. The fighters here are no unit's name
 * in the content (the model echoes example words; a test checks it). */
export const NAMER_EXAMPLES: readonly { first: string; second: string; name: string }[] = [
  { first: "Knight", second: "Wolf", name: "Fang Paladin" },
  { first: "Spark", second: "Healer", name: "Mender of Storms" },
  { first: "Golem", second: "Raven", name: "Gravewing" },
  { first: "Thief", second: "Monk", name: "Alms Cutter" },
];

const namerAsk = (first: string, second: string) => `${first} merges with ${second}. Name:`;

/** The chat messages for one ask: system, the few-shot turns, then the pair
 * by name only (no emoji: the model copies it into the answer). */
export function namerMessages(first: UnitContent, second: UnitContent): { role: "system" | "user" | "assistant"; content: string }[] {
  return [
    { role: "system", content: NAMER_SYSTEM },
    ...NAMER_EXAMPLES.flatMap((e) => [
      { role: "user" as const, content: namerAsk(e.first, e.second) },
      { role: "assistant" as const, content: e.name },
    ]),
    { role: "user", content: namerAsk(first.name, second.name) },
  ];
}

/** The m1 model through an OpenAI-compatible chat endpoint (mlx_lm.server,
 * llama.cpp): ARENA_NAMER_URL, e.g. http://127.0.0.1:8792/v1/chat/completions. */
export function httpModelNamer(url: string, timeoutMs = 15_000): ModelNamer {
  return async (first, second) => {
    const res = await fetch(url, {
      method: "POST",
      headers: { "content-type": "application/json" },
      signal: AbortSignal.timeout(timeoutMs),
      body: JSON.stringify({ messages: namerMessages(first, second), max_tokens: 12, temperature: 0.8 }),
    });
    // A 5xx (mlx still loading) or any other refusal is the model being down,
    // not an answer: thrown, so the pair is asked again after a backoff.
    if (!res.ok) throw new Error(`namer answered ${res.status}`);
    const body = (await res.json()) as { choices?: { message?: { content?: string } }[] };
    return body.choices?.[0]?.message?.content ?? null;
  };
}

/** The model the runtime uses: ARENA_NAMER_URL, or none (always the portmanteau). */
export function envModelNamer(): ModelNamer | null {
  const url = process.env.ARENA_NAMER_URL;
  return url ? httpModelNamer(url) : null;
}

/** Everything the runtime takes from slice 10, built once per runtime. */
export interface FusionNaming {
  /** decide()'s namer. Read-only like peek: the discovery is recorded by
   * hooks.onFuse, after the fuse went through. */
  nameFusion: NameFusion;
  /** RunDeps.peekFusionName: the stored name, else a prepared model name,
   * else the portmanteau. Never records, never waits on the model. */
  peek: NameFusion;
  /** onFuse records the FusionDiscovery (name and credit rules on the
   * contract); onDecision queues model names for every fusable ordered pair
   * on any line that nobody has fused yet, humans' lines first. */
  hooks: RunHooks;
}

interface Pair {
  key: string;
  first: UnitContent;
  second: UnitContent;
  /** Seen on a human's line: asked before every bots' pair. */
  human: boolean;
  /** Times in a row the model was down, slow or timed out on this pair. */
  failures: number;
  /** When the first of those failures was (clock ms); null with none. */
  failingSince: number | null;
  /** Not asked again before this time (clock ms). */
  due: number;
}

/** The naming state of one store: the model, its queues and the names it gave
 * pairs nobody has fused yet. Never in MvpStore: a restart only loses names
 * nobody has used, and lines queue them again. */
interface Namer {
  model: ModelNamer | null;
  units: Map<UnitId, UnitContent>;
  /** Folded names a new pair can't take: the base units' and every stored
   * fusion's (built from the store on first use; recordFusion adds to it). */
  taken: Set<string> | null;
  /** Model names for pairs nobody has fused yet. */
  prepared: Map<string, string>;
  /** Pairs waiting for the model: humans' first, then bots'. */
  human: Pair[];
  bot: Pair[];
  /** Every pair in either queue or being asked, by key. */
  queued: Map<string, Pair>;
  /** Pairs the model can't name, until the time stored: its final failure on
   * the pair (asked again after GIVE_UP_MS, or once the model is back from
   * down), or only refused names (never asked again). Bots fuse them with the
   * portmanteau. */
  gaveUp: Map<string, number>;
  /** Failed asks in a row, on any pairs, and when the first was (clock ms). */
  failStreak: number;
  failingSince: number | null;
  /** When the model went down (its final failure); null while it is up. */
  downSince: number | null;
  /** While down: the next ask (a probe) is not before this (clock ms). */
  nextProbe: number;
  /** Milliseconds now; Date.now unless a test drives the clock. */
  clock: () => number;
  /** Bots waiting for a pair's name (awaitFusionName), by key. */
  waiters: Map<string, (() => void)[]>;
  backoffMs: (failures: number) => number;
  wake?: (() => void) | undefined;
  now: () => Date;
}

const MODEL_TRIES = 3;
/** Failed asks in a row (down, slow, timed out) before a final failure, on
 * one pair or on the model... */
export const MODEL_FAILURES = 4;
/** ...over at least this long: with the default backoff a pair's 5th ask, at
 * about 75 s plus the timeouts. A model still loading is waited for. */
export const MODEL_DOWN_MS = 60_000;
/** While the model is down, one pair is asked this often. */
export const MODEL_PROBE_MS = 60_000;
const GIVE_UP_MS = 600_000;
const namers = new WeakMap<MvpStore, Namer>();

/** The wait before asking the model again about a pair it failed on: 5 s,
 * doubling, at most 10 minutes. */
export const defaultBackoffMs = (failures: number) => Math.min(5_000 * 2 ** (failures - 1), 600_000);

const keyOf = (first: UnitId, second: UnitId) => JSON.stringify([first, second]);

/** The folded names a new pair's name must not be: every base unit's and
 * every stored fusion's. Pairs never share a name. */
function takenNames(n: Namer, store: MvpStore): Set<string> {
  n.taken ??= new Set([...[...n.units.values()].map((u) => u.name), ...store.fusions().map((f) => f.name)].map(fold));
  return n.taken;
}

/** True when a model's name for `key` would clash: it is taken (takenNames) or
 * another pair's prepared name. Such an answer counts as refused. */
function clashes(n: Namer, store: MvpStore, key: string, name: string): boolean {
  const folded = fold(name);
  if (takenNames(n, store).has(folded)) return true;
  for (const [k, other] of n.prepared) if (k !== key && fold(other) === folded) return true;
  return false;
}

/** Whether a bot may fuse `first` then `second` now: the pair's name is
 * stored, a model name is prepared, or the model gave up on it (or there is
 * none), so a bot's fuse never fixes the portmanteau while a model name may
 * still come. A store with no namer (a scratch store) is always ready. Humans
 * never wait: they fuse whenever they like. */
export function fusionNameReady(store: MvpStore, first: UnitId, second: UnitId): boolean {
  const n = namers.get(store);
  if (!n?.model || n.downSince !== null || store.fusion(first, second)) return true;
  const key = keyOf(first, second);
  return n.prepared.has(key) || gaveUpOn(n, key);
}

/** True while the model has given up on the pair (Namer.gaveUp). */
function gaveUpOn(n: Namer, key: string): boolean {
  return (n.gaveUp.get(key) ?? 0) > n.clock();
}

/** Resolves once fusionNameReady(first, second): for a bot that wants to fuse
 * the pair. The pair is asked next, ahead of every queued pair: the top-up
 * waits on one pair at a time and the champion seed on a few, so humans'
 * pairs wait at most a few asks. Bounded by the model's final failure on the
 * pair, or the model counting as down (MODEL_FAILURES failed asks over
 * MODEL_DOWN_MS). Needs the naming job running. */
export function awaitFusionName(store: MvpStore, first: UnitId, second: UnitId): Promise<void> {
  const n = namers.get(store);
  const key = keyOf(first, second);
  if (!n?.model || store.fusion(first, second) || n.prepared.has(key) || gaveUpOn(n, key)) return Promise.resolve();
  const waiting = n.queued.get(key);
  if (waiting) {
    // Moved to the front; one being asked right now stays put.
    const q = waiting.human ? n.human : n.bot;
    const i = q.indexOf(waiting);
    if (i >= 0) {
      q.splice(i, 1);
      waiting.human = true;
      n.human.unshift(waiting);
    }
  } else {
    const a = n.units.get(first);
    const b = n.units.get(second);
    if (!a || !b) return Promise.resolve();
    const pair: Pair = { key, first: a, second: b, human: true, failures: 0, failingSince: null, due: 0 };
    n.queued.set(key, pair);
    n.human.unshift(pair);
  }
  // Down: the pair is still asked, but nobody waits for it.
  if (fusionNameReady(store, first, second)) {
    n.wake?.();
    return Promise.resolve();
  }
  return new Promise((resolve) => {
    n.waiters.set(key, [...(n.waiters.get(key) ?? []), resolve]);
    n.wake?.();
  });
}

/** Wakes the bots waiting for `key`, or every waiting bot (the model is down). */
function settle(n: Namer, key?: string): void {
  for (const [k, waiters] of [...n.waiters]) {
    if (key !== undefined && k !== key) continue;
    n.waiters.delete(k);
    for (const resolve of waiters) resolve();
  }
}

/** Records the first fuse of a pair: the FusionDiscovery with the unit's
 * name and credit, its source the model when that name was prepared by it.
 * A pair already stored keeps its name; the first human claims a bots' pair.
 * Returns the stored discovery. Used by onFuse and by slice 6's champion seed. */
export function recordFusion(store: MvpStore, parts: FusionParts, at: string): FusionDiscovery {
  const known = store.fusion(parts.first, parts.second);
  if (known) {
    if (known.discoveredBy !== null || !parts.discoveredBy) return known;
    const claimed = { ...known, discoveredBy: parts.discoveredBy };
    store.putFusion(claimed);
    return claimed;
  }
  const n = namers.get(store);
  const key = keyOf(parts.first, parts.second);
  const fromModel = n?.prepared.get(key) === parts.name;
  n?.prepared.delete(key);
  if (n) settle(n, key);
  const found: FusionDiscovery = {
    first: parts.first,
    second: parts.second,
    name: parts.name,
    discoveredBy: parts.discoveredBy,
    discoveredAt: at,
    nameSource: fromModel ? "model" : "fallback",
  };
  store.putFusion(found);
  n?.taken?.add(fold(found.name));
  return found;
}

export function fusionNaming(
  rt: Pick<RunDeps, "store" | "content" | "now">,
  opts: { model?: ModelNamer | null; backoffMs?: (failures: number) => number; clock?: () => number } = {},
): FusionNaming {
  const { store } = rt;
  const n: Namer = {
    model: opts.model === undefined ? envModelNamer() : opts.model,
    units: new Map(rt.content.units.map((u) => [u.id, u])),
    taken: null,
    prepared: new Map(),
    human: [],
    bot: [],
    queued: new Map(),
    gaveUp: new Map(),
    failStreak: 0,
    failingSince: null,
    downSince: null,
    nextProbe: 0,
    clock: opts.clock ?? Date.now,
    waiters: new Map(),
    backoffMs: opts.backoffMs ?? defaultBackoffMs,
    now: rt.now,
  };
  namers.set(store, n);
  const enqueue = (first: UnitContent, second: UnitContent, human: boolean) => {
    const key = keyOf(first.id, second.id);
    if (!n.model || n.prepared.has(key) || gaveUpOn(n, key) || store.fusion(first.id, second.id)) return;
    const waiting = n.queued.get(key);
    if (waiting) {
      // A bots' pair now on a human's line moves to the humans' queue.
      if (human && !waiting.human && n.bot.includes(waiting)) {
        n.bot.splice(n.bot.indexOf(waiting), 1);
        waiting.human = true;
        n.human.push(waiting);
      }
      return;
    }
    const pair: Pair = { key, first, second, human, failures: 0, failingSince: null, due: 0 };
    n.queued.set(key, pair);
    (human ? n.human : n.bot).push(pair);
    n.wake?.();
  };
  const peek: NameFusion = (first, second, by) => {
    const known = store.fusion(first.id, second.id);
    if (known) return { name: known.name, discoveredBy: known.discoveredBy ?? (by.bot ? null : by) };
    // A new pair's name is unique: a prepared name stored for another pair
    // since is dropped for the portmanteau, which avoids every taken name.
    const taken = takenNames(n, store);
    const prepared = n.prepared.get(keyOf(first.id, second.id));
    return {
      name: prepared !== undefined && !taken.has(fold(prepared)) ? prepared : portmanteauAvoiding(first.name, second.name, (f) => taken.has(f)),
      discoveredBy: by.bot ? null : by,
    };
  };
  const hooks: RunHooks = {
    onDecision(_before, _d, after) {
      if (after.phase === "over" || !n.model) return;
      const line = after.line;
      for (let i = 0; i < line.length; i++)
        for (let j = 0; j < line.length; j++) {
          if (i === j || fuseCheck(line[i]!, line[j]!) !== null) continue;
          const first = n.units.get(line[i]!.unitId);
          const second = n.units.get(line[j]!.unitId);
          if (first && second) enqueue(first, second, !after.player.bot);
        }
    },
    onFuse(_run, fused, _ctx) {
      if (fused.fusion) recordFusion(store, fused.fusion, n.now().toISOString());
    },
  };
  return { nameFusion: peek, peek, hooks };
}

/** The next pair to ask, humans' first, skipping `tried`; null when none is
 * due. While the model is down only a probe is: the first queued pair, once
 * per MODEL_PROBE_MS. */
function takeNext(n: Namer, tried: Set<string>): Pair | null {
  const now = n.clock();
  if (n.downSince !== null && now < n.nextProbe) return null;
  for (const q of [n.human, n.bot]) {
    const i = q.findIndex((p) => (n.downSince !== null || p.due <= now) && !tried.has(p.key));
    if (i >= 0) return q.splice(i, 1)[0]!;
  }
  return null;
}

/** The model answered after being down: every queued pair is asked again
 * now, with its failures forgotten, and pairs it gave up on while failing are
 * asked again (bots wait for them again). */
function backUp(n: Namer): void {
  n.downSince = null;
  for (const p of n.queued.values()) {
    p.failures = 0;
    p.failingSince = null;
    p.due = 0;
  }
  for (const [key, until] of n.gaveUp) if (until !== Infinity) n.gaveUp.delete(key);
}

/** Asks the model, one pair at a time, humans' pairs first (also pairs queued
 * while it runs), until no pair is due; each pair at most once per drain. A
 * pair nobody has fused gets a prepared name. A pair the model fails on goes
 * back in its queue with a backoff; its final failure (MODEL_FAILURES over
 * MODEL_DOWN_MS) gives up on it for a while (bots then fuse it with the
 * portmanteau). Failures in a row on any pairs, as many over as long, put
 * the model down: then only one probe per MODEL_PROBE_MS is asked, and its
 * answer brings the model back. */
export async function drainFusionNames(store: MvpStore): Promise<void> {
  const n = namers.get(store);
  if (!n?.model) return;
  const tried = new Set<string>();
  for (let pair = takeNext(n, tried); pair; pair = takeNext(n, tried)) {
    tried.add(pair.key);
    if (store.fusion(pair.first.id, pair.second.id)) {
      n.queued.delete(pair.key);
      settle(n, pair.key);
      continue;
    }
    let name: string | null = null;
    try {
      // A small model often wraps a good name in emoji or markdown: ask again.
      for (let tries = 0; tries < MODEL_TRIES && !name; tries++) {
        const raw = await n.model(pair.first, pair.second);
        name = raw === null ? null : cleanModelName(raw, pair.first, pair.second);
        // Another pair's name (or a base unit's) is refused too: ask again.
        if (name && clashes(n, store, pair.key, name)) name = null;
      }
    } catch {
      // Down, slow or timed out: ask again later, while nobody has fused it.
      const now = n.clock();
      pair.failures++;
      pair.failingSince ??= now;
      n.failStreak++;
      n.failingSince ??= now;
      if (n.downSince !== null) n.nextProbe = now + MODEL_PROBE_MS;
      else if (n.failStreak >= MODEL_FAILURES && now - n.failingSince >= MODEL_DOWN_MS) {
        // Down: no bot waits for the model, and only probes ask it.
        n.downSince = now;
        n.nextProbe = now + MODEL_PROBE_MS;
        settle(n);
      }
      if (store.fusion(pair.first.id, pair.second.id)) {
        n.queued.delete(pair.key);
        settle(n, pair.key);
      } else if (pair.failures >= MODEL_FAILURES && now - pair.failingSince >= MODEL_DOWN_MS) {
        n.queued.delete(pair.key);
        n.gaveUp.set(pair.key, now + GIVE_UP_MS);
        settle(n, pair.key);
      } else {
        pair.due = now + n.backoffMs(pair.failures);
        (pair.human ? n.human : n.bot).push(pair);
      }
      continue;
    }
    n.queued.delete(pair.key);
    n.failStreak = 0;
    n.failingSince = null;
    if (n.downSince !== null) backUp(n);
    // Checked again: someone may have fused the pair while the model thought;
    // its name is then fixed.
    if (!store.fusion(pair.first.id, pair.second.id)) {
      if (name) n.prepared.set(pair.key, name);
      else n.gaveUp.set(pair.key, Infinity);
    }
    settle(n, pair.key);
  }
}

/** The naming queue: drains whenever a pair is queued, and again when the
 * earliest pair waiting on a backoff is due. */
export const fusionNamingJob: MvpJob = (rt) => {
  const n = namers.get(rt.store);
  if (!n?.model) return () => {};
  let running = false;
  let stopped = false;
  let timer: ReturnType<typeof setTimeout> | undefined;
  const wake = () => {
    if (running || stopped) return;
    running = true;
    clearTimeout(timer);
    void drainFusionNames(rt.store).finally(() => {
      running = false;
      const waiting = [...n.human, ...n.bot];
      if (waiting.length === 0 || stopped) return;
      // Down: the next probe; up: the earliest pair's backoff.
      const next = n.downSince !== null ? n.nextProbe : Math.min(...waiting.map((p) => p.due));
      timer = setTimeout(wake, Math.max(1000, next - n.clock()));
    });
  };
  n.wake = wake;
  wake();
  return () => {
    stopped = true;
    clearTimeout(timer);
    n.wake = undefined;
  };
};
