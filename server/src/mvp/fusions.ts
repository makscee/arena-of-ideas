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
// - A bot fuses a pair only once fusionNameReady: its name stored or
//   prepared, or the model gave up on it. Bots run in the background, so a
//   bot that wants a pair whose name isn't ready waits for it
//   (awaitFusionName), and the pair is asked next, ahead of every queue. A model that is down, slow
//   or times out (a 5xx included) is asked again with a backoff,
//   MODEL_FAILURES times; after MODEL_FAILURES failed asks in a row, on any
//   pairs, the model counts as down and no bot waits until it answers again.
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
 * second's ("Brawler" + "Medic" → "Brawdic"). The only portmanteau. */
export function portmanteau(first: string, second: string): string {
  const a = first.replace(/\s+/g, "");
  const b = second.replace(/\s+/g, "").toLowerCase();
  const head = a.slice(0, Math.ceil(a.length / 2));
  const tail = b.slice(Math.floor(b.length / 2));
  return head.slice(-1).toLowerCase() === tail[0] ? head + tail.slice(1) : head + tail;
}

/** The stored name, else the portmanteau, with the credit by FusionDiscovery's
 * rule (the stored discoverer, else `by` unless it is a bot). It only reads:
 * it never records a discovery or calls a model. */
export function storedOrPortmanteau(store: MvpStore): NameFusion {
  return (first, second, by) => {
    const known = store.fusion(first.id, second.id);
    return {
      name: known?.name ?? portmanteau(first.name, second.name),
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
  // film, tv, books, comics
  "gandalf", "frodo", "bilbo", "sauron", "gollum", "smeagol", "aragorn", "legolas", "gimli", "saruman", "mordor", "hobbit", "balrog",
  "darth", "skywalker", "chewbacca", "chewie", "kenobi", "obiwan", "palpatine", "stormtrooper", "grogu", "mandalorian",
  "hogwart", "voldemort", "dumbledore", "hermione",
  "mufasa", "shrek", "pinocchio", "dumbo", "mickey",
  "batman", "superman", "aquaman", "gotham", "krypton", "spiderman",
  "thanos", "wolverine", "deadpool",
  "godzilla", "kingkong", "xenomorph", "dracula", "frankenstein", "007",
  "naruto", "sasuke", "totoro", "gundam", "ultraman", "megatron",
  "spongebob", "squidward", "garfield", "snoopy", "scooby", "simpson", "smurf", "barbie", "pinkie",
  "atreides", "harkonnen", "khaleesi", "targaryen", "lannister", "westeros", "dothraki",
  // pretending to be official, or spam
  "vader", "yoda", "jedi", "snape", "dobby", "pudge", "zerg", "goku", "optimus", "hulk",
  "mario", "thorhammer", "thorshammer",
  "moderator", "http", "www",
];
// Ordinary words that contain a stem: taken out of a word before stems match.
const ORDINARY = ["invader", "evader", "pervader", "hulking", "hulky", "marionette", "mariology", "mariolat", "marion", "snaper"];
// Words: short or ordinary enough that a stem would hit real words ("Thorn",
// "Invader", "Marionette", "Smuggler", "Scamper"), so they match one word of
// the name, also with a plural or possessive ending ("Marios", "Thor's").
const BLOCKED_WORDS = [
  "ganon", "croft", "thrall", "uther", "gul", "longshot", "elise", "reno", "brann", "kael", "gnoll",
  "dota", "invoker", "ahri", "garen", "terran", "fallout", "tifa", "papyrus",
  "sith", "potter", "muggle",
  "elsa", "olaf", "nemo", "dory", "fiona", "minnie", "thor", "loki", "avenger", "marvel",
  "zoro", "bart", "lego", "barney", "scam", "developer", "administrator",
  // the prompt forbids them: the model's way of not naming
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
  "donkey kong", "wonder woman", "iron man", "spider man", "black widow", "captain america", "dark souls", "free vbucks",
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
  return words.map((w) => w[0]!.toUpperCase() + w.slice(1)).join(name.includes("-") && words.length > 1 ? "-" : " ");
}

/** The m1 model through an OpenAI-compatible chat endpoint (mlx_lm.server,
 * llama.cpp): ARENA_NAMER_URL, e.g. http://127.0.0.1:8792/v1/chat/completions. */
export function httpModelNamer(url: string, timeoutMs = 15_000): ModelNamer {
  return async (first, second) => {
    const res = await fetch(url, {
      method: "POST",
      headers: { "content-type": "application/json" },
      signal: AbortSignal.timeout(timeoutMs),
      body: JSON.stringify({
        messages: [
          {
            role: "system",
            content:
              "You invent names for creatures in a fantasy auto-battler. Two fighters merge into one new creature. Answer with its name only: one or two words, at most 18 letters, no quotes, no explanation. Never use a name from an existing game, film, book or comic, and never the words fusion or fuse.",
          },
          { role: "user", content: `${first.emoji} ${first.name} (first) merges with ${second.emoji} ${second.name} (second). Name:` },
        ],
        max_tokens: 12,
        temperature: 0.8,
      }),
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
  /** Not asked again before this time (ms since epoch). */
  due: number;
}

/** The naming state of one store: the model, its queues and the names it gave
 * pairs nobody has fused yet. Never in MvpStore: a restart only loses names
 * nobody has used, and lines queue them again. */
interface Namer {
  model: ModelNamer | null;
  units: Map<UnitId, UnitContent>;
  /** Model names for pairs nobody has fused yet. */
  prepared: Map<string, string>;
  /** Pairs waiting for the model: humans' first, then bots'. */
  human: Pair[];
  bot: Pair[];
  /** Every pair in either queue or being asked, by key. */
  queued: Map<string, Pair>;
  /** Pairs the model can't name, until the time stored: it failed
   * MODEL_FAILURES times in a row (asked again after GIVE_UP_MS), or answered
   * only refused names (never asked again). Bots fuse them with the portmanteau. */
  gaveUp: Map<string, number>;
  /** Failed asks in a row, on any pairs; MODEL_FAILURES or more is down. */
  failStreak: number;
  /** Bots waiting for a pair's name (awaitFusionName), by key. */
  waiters: Map<string, (() => void)[]>;
  backoffMs: (failures: number) => number;
  wake?: (() => void) | undefined;
  now: () => Date;
}

const MODEL_TRIES = 3;
/** Failures in a row (down, slow, timed out) before the model gives up on a
 * pair: with the default backoff about 35 s, plus the timeouts. */
export const MODEL_FAILURES = 4;
const GIVE_UP_MS = 600_000;
const namers = new WeakMap<MvpStore, Namer>();

/** The wait before asking the model again about a pair it failed on: 5 s,
 * doubling, at most 10 minutes. */
export const defaultBackoffMs = (failures: number) => Math.min(5_000 * 2 ** (failures - 1), 600_000);

const keyOf = (first: UnitId, second: UnitId) => JSON.stringify([first, second]);

/** Whether a bot may fuse `first` then `second` now: the pair's name is
 * stored, a model name is prepared, or the model gave up on it (or there is
 * none), so a bot's fuse never fixes the portmanteau while a model name may
 * still come. A store with no namer (a scratch store) is always ready. Humans
 * never wait: they fuse whenever they like. */
export function fusionNameReady(store: MvpStore, first: UnitId, second: UnitId): boolean {
  const n = namers.get(store);
  if (!n?.model || n.failStreak >= MODEL_FAILURES || store.fusion(first, second)) return true;
  const key = keyOf(first, second);
  return n.prepared.has(key) || n.gaveUp.has(key);
}

/** Resolves once fusionNameReady(first, second): for a bot that wants to fuse
 * the pair. The pair is asked next, ahead of every queued pair: the top-up
 * waits on one pair at a time and the champion seed on a few, so humans'
 * pairs wait at most a few asks. Bounded by the model's final failure: MODEL_FAILURES failed asks on
 * the pair, or the model counting as down. Needs the naming job running. */
export function awaitFusionName(store: MvpStore, first: UnitId, second: UnitId): Promise<void> {
  const n = namers.get(store);
  const key = keyOf(first, second);
  if (!n?.model || store.fusion(first, second) || n.prepared.has(key) || n.gaveUp.has(key)) return Promise.resolve();
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
    const pair: Pair = { key, first: a, second: b, human: true, failures: 0, due: 0 };
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
  return found;
}

export function fusionNaming(
  rt: Pick<RunDeps, "store" | "content" | "now">,
  opts: { model?: ModelNamer | null; backoffMs?: (failures: number) => number } = {},
): FusionNaming {
  const { store } = rt;
  const n: Namer = {
    model: opts.model === undefined ? envModelNamer() : opts.model,
    units: new Map(rt.content.units.map((u) => [u.id, u])),
    prepared: new Map(),
    human: [],
    bot: [],
    queued: new Map(),
    gaveUp: new Map(),
    failStreak: 0,
    waiters: new Map(),
    backoffMs: opts.backoffMs ?? defaultBackoffMs,
    now: rt.now,
  };
  namers.set(store, n);
  const enqueue = (first: UnitContent, second: UnitContent, human: boolean) => {
    const key = keyOf(first.id, second.id);
    if (!n.model || n.prepared.has(key) || (n.gaveUp.get(key) ?? 0) > Date.now() || store.fusion(first.id, second.id)) return;
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
    const pair: Pair = { key, first, second, human, failures: 0, due: 0 };
    n.queued.set(key, pair);
    (human ? n.human : n.bot).push(pair);
    n.wake?.();
  };
  const peek: NameFusion = (first, second, by) => {
    const known = store.fusion(first.id, second.id);
    return {
      name: known?.name ?? n.prepared.get(keyOf(first.id, second.id)) ?? portmanteau(first.name, second.name),
      discoveredBy: known?.discoveredBy ?? (by.bot ? null : by),
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

/** The next pair due, humans' first; null when none is. Skips `tried`. */
function takeDue(n: Namer, tried: Set<string>): Pair | null {
  const now = Date.now();
  for (const q of [n.human, n.bot]) {
    const i = q.findIndex((p) => p.due <= now && !tried.has(p.key));
    if (i >= 0) return q.splice(i, 1)[0]!;
  }
  return null;
}

/** Asks the model, one pair at a time, humans' pairs first (also pairs queued
 * while it runs), until no pair is due; each pair at most once per drain. A
 * pair nobody has fused gets a prepared name. A pair the model fails on goes
 * back in its queue with a backoff, and after MODEL_FAILURES in a row it gives
 * up on it for a while (bots then fuse it with the portmanteau). */
export async function drainFusionNames(store: MvpStore): Promise<void> {
  const n = namers.get(store);
  if (!n?.model) return;
  const tried = new Set<string>();
  for (let pair = takeDue(n, tried); pair; pair = takeDue(n, tried)) {
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
      }
    } catch {
      // Down, slow or timed out: ask again later, while nobody has fused it.
      pair.failures++;
      // Down: no bot waits for the model until it answers again.
      if (++n.failStreak >= MODEL_FAILURES) settle(n);
      if (store.fusion(pair.first.id, pair.second.id)) {
        n.queued.delete(pair.key);
        settle(n, pair.key);
      } else if (pair.failures >= MODEL_FAILURES) {
        n.queued.delete(pair.key);
        n.gaveUp.set(pair.key, Date.now() + GIVE_UP_MS);
        settle(n, pair.key);
      } else {
        pair.due = Date.now() + n.backoffMs(pair.failures);
        (pair.human ? n.human : n.bot).push(pair);
      }
      continue;
    }
    n.queued.delete(pair.key);
    n.failStreak = 0;
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
      const next = Math.min(...waiting.map((p) => p.due));
      timer = setTimeout(wake, Math.max(1000, next - Date.now()));
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
