// Fusion names (mission #574). Slice 10 owns this file: the local model on m1
// behind a blocklist, the stored discoveries (MvpStore.fusion, putFusion) and
// GET /fusions. mvpRuntime (./runtime.ts) wires fusionNaming() once, so slice
// 10 fills it in here and never edits decide() or the routes. The rules are on
// FusionDiscovery in the contract: the first fuse of a pair fixes its name, a
// model name is only ever prepared for pairs nobody has fused yet, and bots
// are never credited.
//
// How a pair gets its name, never waiting on the model:
// - onDecision queues every fusable ordered pair on a human's line that has no
//   stored name yet; the naming job asks the model and keeps the answer in
//   this module's cache, so peek and the fuse use it.
// - The first fuse stores the cached model name, else the portmanteau
//   ("fallback"). A human's fuse fixes the pair's name for good.
// - A pair only bots have fused (discoveredBy null) with the fallback is
//   queued; when the model answers, its stored name (and the bots' fused
//   units) is replaced once. Queued pairs survive a restart (re-queued from
//   the store) and a model that is down, slow or times out is asked again
//   with a backoff, until a human fuses the pair.
import { fuseCheck } from "../../../src/mvp/forms.js";
import type { FuseContext, FusionDiscovery, LineUnit, PlayerRef, UnitContent, UnitId } from "../../../src/mvp/contract.js";
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
// Stems: distinctive enough to match anywhere inside the folded name, so glued
// and inflected forms are caught too ("Darthvader", "Gandalfs", "Hogwart").
const BLOCKED_STEMS = [
  // games
  "pikachu", "pokemon", "charizard", "mewtwo", "eevee", "luigi", "bowser", "yoshi", "wario", "zelda", "ganondorf",
  "eggman", "kirby", "samus", "metroid", "megaman", "pacman", "minecraft",
  "fortnite", "vbucks", "masterchief", "cortana", "kratos", "geralt", "witcher", "doomguy",
  "warcraft", "hearthstone", "azeroth", "jaina", "arthas", "illidan", "sylvanas", "garrosh", "guldan", "anduin",
  "nefarian", "ragnaros", "deathwing", "onyxia", "kelthuzad", "nzoth", "cthun", "yogg", "leeroy",
  "landro", "medivh", "kaelthas", "malfurion", "tyrande", "rexxar", "valeera",
  "roshan", "teemo", "yasuo", "overwatch", "reinhardt",
  "diablo", "starcraft", "kerrigan", "protoss", "skyrim", "dovahkiin", "tamriel", "sephiroth", "chocobo",
  "moogle", "aerith", "metalgear", "cuphead", "undertale",
  "terraria", "amogus", "roblox", "genshin", "paimon", "malenia", "dragonborn",
  // film, tv, books, comics
  "gandalf", "frodo", "bilbo", "sauron", "gollum", "smeagol", "aragorn", "legolas", "gimli", "saruman", "mordor", "hobbit", "balrog",
  "darth", "skywalker", "chewbacca", "chewie", "kenobi", "obiwan", "palpatine", "stormtrooper", "grogu", "mandalorian",
  "hogwart", "voldemort", "dumbledore", "hermione", "hagrid",
  "simba", "mufasa", "shrek", "pinocchio", "dumbo", "bambi", "mickey",
  "batman", "superman", "aquaman", "gotham", "krypton", "spiderman", "ironman",
  "thanos", "wolverine", "deadpool", "groot", "magneto", "xmen",
  "godzilla", "kingkong", "terminator", "xenomorph", "rambo", "dracula", "frankenstein", "007",
  "naruto", "sasuke", "vegeta", "luffy", "totoro", "gundam", "ultraman", "megatron",
  "spongebob", "squidward", "garfield", "snoopy", "scooby", "simpson", "smurf", "barbie", "pinkie",
  "atreides", "harkonnen", "khaleesi", "targaryen", "lannister", "westeros", "dothraki", "hodor",
  // pretending to be official, or spam
  "admin", "moderator", "http", "www",
];
// Words: short or ordinary enough that a stem would hit real words ("Thorn",
// "Invader", "Marionette", "Smuggler", "Scamper"), so they match one word of
// the name, also with a plural or possessive ending ("Marios", "Thor's").
const BLOCKED_WORDS = [
  "mario", "ganon", "sonic", "croft", "thrall", "uther", "gul", "swindle", "longshot", "elise", "reno", "brann", "kael", "gnoll",
  "dota", "pudge", "invoker", "ahri", "garen", "zerg", "zergling", "terran", "fallout", "tifa", "papyrus",
  "vader", "yoda", "jedi", "sith", "potter", "snape", "dobby", "muggle",
  "elsa", "olaf", "nemo", "dory", "fiona", "minnie", "joker", "hulk", "thor", "loki", "avenger", "marvel",
  "goku", "zoro", "optimus", "bart", "lego", "barney", "scam", "developer", "administrator",
];
// Ordinary words that are also franchise names: blocked only as the whole name.
const BLOCKED_WHOLE = [
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
  const whole = fold(name);
  if (BLOCKED_WHOLE.includes(whole) || BLOCKED_STEMS.some((stem) => whole.includes(stem))) return true;
  const words = name.split(/[\s'’-]+/).map(fold).filter(Boolean);
  return words.some((w) => BLOCKED_WORDS.some((b) => WORD_ENDINGS.some((end) => w === b + end)));
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
    if (!res.ok) return null;
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
  /** RunDeps.peekFusionName: the stored name, else a prefetched model name,
   * else the portmanteau. Never records, never waits on the model. */
  peek: NameFusion;
  /** onFuse records the FusionDiscovery (name and credit rules on the
   * contract); onDecision queues model names for every fusable ordered pair
   * on a human's line that nobody has fused yet. */
  hooks: RunHooks;
}

interface Pair {
  key: string;
  first: UnitContent;
  second: UnitContent;
  /** Times the model was down, slow or timed out on this pair. */
  failures: number;
  /** Not asked again before this time (ms since epoch). */
  due: number;
}

/** The naming state of one store: the model, its queue and the names it gave
 * pairs nobody has fused yet. Never in MvpStore. */
interface Namer {
  model: ModelNamer | null;
  units: Map<UnitId, UnitContent>;
  /** Model names for pairs nobody has fused yet. */
  prepared: Map<string, string>;
  queue: Pair[];
  queued: Set<string>;
  /** Pairs the model answered once (with a name or only refused ones); never asked again. */
  asked: Set<string>;
  /** Bots' fused units named by the fallback, renamed when the model answers. */
  fallbackUnits: Map<string, { runId: string; uid: string }[]>;
  backoffMs: (failures: number) => number;
  wake?: (() => void) | undefined;
  now: () => Date;
}

const MAX_QUEUE = 200;
const MODEL_TRIES = 3;
const namers = new WeakMap<MvpStore, Namer>();

/** The wait before asking the model again about a pair it failed on: 5 s,
 * doubling, at most 10 minutes. */
export const defaultBackoffMs = (failures: number) => Math.min(5_000 * 2 ** (failures - 1), 600_000);

const keyOf = (first: UnitId, second: UnitId) => JSON.stringify([first, second]);

/** Whether the model may still name a pair: nobody has fused it, or only bots
 * have and it holds the fallback. Once a human fuses a pair, its name is fixed. */
function renamable(store: MvpStore, first: UnitId, second: UnitId): boolean {
  const known = store.fusion(first, second);
  return !known || (known.discoveredBy === null && known.nameSource === "fallback");
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
    queue: [],
    queued: new Set(),
    asked: new Set(),
    fallbackUnits: new Map(),
    backoffMs: opts.backoffMs ?? defaultBackoffMs,
    now: rt.now,
  };
  namers.set(store, n);
  const enqueue = (first: UnitContent, second: UnitContent) => {
    const key = keyOf(first.id, second.id);
    if (!n.model || n.queued.has(key) || n.asked.has(key) || n.prepared.has(key) || n.queue.length >= MAX_QUEUE) return;
    n.queued.add(key);
    n.queue.push({ key, first, second, failures: 0, due: 0 });
    n.wake?.();
  };
  // A restart loses the queue: ask again for every pair only bots have fused
  // with the fallback.
  for (const f of store.fusions()) {
    const first = n.units.get(f.first);
    const second = n.units.get(f.second);
    if (first && second && renamable(store, f.first, f.second)) enqueue(first, second);
  }
  const peek: NameFusion = (first, second, by) => {
    const known = store.fusion(first.id, second.id);
    return {
      name: known?.name ?? n.prepared.get(keyOf(first.id, second.id)) ?? portmanteau(first.name, second.name),
      discoveredBy: known?.discoveredBy ?? (by.bot ? null : by),
    };
  };
  const hooks: RunHooks = {
    onDecision(_before, _d, after) {
      if (after.player.bot || after.phase === "over") return;
      const line = after.line;
      for (let i = 0; i < line.length; i++)
        for (let j = 0; j < line.length; j++) {
          if (i === j || fuseCheck(line[i]!, line[j]!) !== null) continue;
          const first = n.units.get(line[i]!.unitId);
          const second = n.units.get(line[j]!.unitId);
          if (first && second && !store.fusion(first.id, second.id)) enqueue(first, second);
        }
    },
    onFuse(run, fused, ctx) {
      const parts = fused.fusion;
      if (!parts) return;
      const key = keyOf(parts.first, parts.second);
      const known = store.fusion(parts.first, parts.second);
      if (known) {
        if (known.discoveredBy === null && ctx.discoveredBy) {
          // The first human to fuse a pair only bots had made claims it, and
          // fixes its name.
          store.putFusion({ ...known, discoveredBy: ctx.discoveredBy });
          n.fallbackUnits.delete(key);
        } else if (known.discoveredBy === null && known.nameSource === "fallback") {
          n.fallbackUnits.get(key)?.push({ runId: run.runId, uid: fused.uid });
        }
        return;
      }
      const fromModel = n.prepared.get(key) === ctx.name;
      n.prepared.delete(key);
      store.putFusion({
        first: parts.first,
        second: parts.second,
        name: ctx.name,
        discoveredBy: ctx.discoveredBy,
        discoveredAt: n.now().toISOString(),
        nameSource: fromModel ? "model" : "fallback",
      });
      if (!fromModel && ctx.discoveredBy === null) {
        n.fallbackUnits.set(key, [{ runId: run.runId, uid: fused.uid }]);
        const first = n.units.get(parts.first);
        const second = n.units.get(parts.second);
        if (first && second) enqueue(first, second);
      }
    },
  };
  return { nameFusion: peek, peek, hooks };
}

/** Asks the model once for every queued pair that is due, one at a time, and
 * applies each answer: a pair nobody has fused gets a prepared name, a pair
 * only bots fused with the fallback is renamed once. A pair the model fails on
 * goes back in the queue with a backoff while the model may still name it.
 * Resolves when every due pair was asked. */
export async function drainFusionNames(store: MvpStore): Promise<void> {
  const n = namers.get(store);
  if (!n?.model) return;
  const now = Date.now();
  const due = n.queue.filter((p) => p.due <= now);
  n.queue = n.queue.filter((p) => p.due > now);
  for (const pair of due) {
    n.queued.delete(pair.key);
    if (n.asked.has(pair.key) || !renamable(store, pair.first.id, pair.second.id)) continue;
    let name: string | null = null;
    try {
      // A small model often wraps a good name in emoji or markdown: ask again.
      for (let tries = 0; tries < MODEL_TRIES && !name; tries++) {
        const raw = await n.model(pair.first, pair.second);
        name = raw === null ? null : cleanModelName(raw, pair.first, pair.second);
      }
    } catch {
      // Down, slow or timed out: ask again later, while a name can still change.
      if (renamable(store, pair.first.id, pair.second.id) && !n.queued.has(pair.key)) {
        const failures = pair.failures + 1;
        n.queued.add(pair.key);
        n.queue.push({ ...pair, failures, due: Date.now() + n.backoffMs(failures) });
      }
      continue;
    }
    n.asked.add(pair.key);
    if (name) applyModelName(store, n, pair, name);
  }
}

function applyModelName(store: MvpStore, n: Namer, pair: Pair, name: string): void {
  const known = store.fusion(pair.first.id, pair.second.id);
  if (!known) {
    n.prepared.set(pair.key, name);
    return;
  }
  // Checked again: a human may have fused the pair while the model thought.
  if (!renamable(store, pair.first.id, pair.second.id)) return;
  const renamed: FusionDiscovery = { ...known, name, nameSource: "model" };
  store.putFusion(renamed);
  for (const at of n.fallbackUnits.get(pair.key) ?? []) {
    const run = store.run(at.runId);
    const unit: LineUnit | undefined = run?.line.find((u) => u.uid === at.uid);
    if (!run || !unit?.fusion || unit.fusion.name !== known.name) continue;
    unit.name = name;
    unit.fusion = { ...unit.fusion, name };
    store.putRun(run);
  }
  n.fallbackUnits.delete(pair.key);
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
      if (n.queue.length === 0 || stopped) return;
      const next = Math.min(...n.queue.map((p) => p.due));
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
