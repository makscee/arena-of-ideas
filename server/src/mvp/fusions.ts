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
//   ("fallback"). A fallback pair is queued too; when the model answers, its
//   stored name is replaced once (and the fused unit on that run's line), and
//   never again.
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
// names), plus words that pretend to be official. Matched per word and on the
// whole name, ignoring case, accents and punctuation.
const BLOCKLIST = [
  // games
  "pikachu", "pokemon", "charizard", "mewtwo", "eevee", "mario", "luigi", "bowser", "yoshi", "wario", "zelda", "ganon", "ganondorf",
  "sonic", "eggman", "kirby", "samus", "metroid", "megaman", "pacman", "minecraft",
  "fortnite", "vbucks", "masterchief", "cortana", "kratos", "croft", "geralt", "witcher", "doomguy",
  "warcraft", "hearthstone", "azeroth", "thrall", "jaina", "arthas", "illidan", "sylvanas", "uther", "garrosh", "gul'dan", "guldan", "anduin",
  "nefarian", "ragnaros", "deathwing", "onyxia", "kelthuzad", "kel'thuzad", "nzoth", "n'zoth", "cthun", "c'thun", "yogg", "yoggsaron", "leeroy",
  "swindle", "landro", "longshot", "elise", "reno", "brann", "medivh", "kael", "kaelthas", "malfurion", "tyrande", "rexxar", "valeera", "gnoll",
  "dota", "pudge", "invoker", "roshan", "teemo", "yasuo", "ahri", "garen", "overwatch", "reinhardt",
  "diablo", "starcraft", "kerrigan", "zerg", "protoss", "terran", "fallout", "skyrim", "dovahkiin", "tamriel", "sephiroth", "chocobo",
  "moogle", "tifa", "aerith", "metalgear", "cuphead", "undertale",
  "papyrus", "terraria", "amogus", "roblox", "genshin", "paimon", "malenia", "dragonborn",
  // film, tv, books, comics
  "gandalf", "frodo", "bilbo", "sauron", "gollum", "smeagol", "aragorn", "legolas", "gimli", "saruman", "mordor", "hobbit", "balrog",
  "darth", "vader", "skywalker", "yoda", "jedi", "sith", "chewbacca", "chewie", "kenobi", "obiwan", "palpatine", "stormtrooper", "grogu", "mandalorian",
  "potter", "hogwarts", "voldemort", "dumbledore", "hermione", "snape", "hagrid", "dobby", "muggle",
  "elsa", "olaf", "simba", "mufasa", "nemo", "dory", "shrek", "fiona", "pinocchio", "dumbo", "bambi", "mickey", "minnie",
  "batman", "superman", "joker", "aquaman", "gotham", "krypton", "spiderman", "ironman", "hulk", "thor",
  "loki", "thanos", "avenger", "avengers", "wolverine", "deadpool", "groot", "marvel", "magneto", "xmen",
  "godzilla", "kingkong", "terminator", "xenomorph", "rambo", "dracula", "frankenstein", "007",
  "naruto", "sasuke", "goku", "vegeta", "pikachu", "luffy", "zoro", "totoro", "gundam", "ultraman", "optimus", "megatron",
  "spongebob", "squidward", "garfield", "snoopy", "scooby", "simpson", "bart", "smurf", "barbie", "lego", "pinkie", "barney",
  "witcher", "atreides", "harkonnen", "khaleesi", "targaryen", "lannister", "westeros", "dothraki", "hodor",
  // pretending to be official, or spam
  "admin", "administrator", "moderator", "developer", "scam", "http", "www",
];
// Ordinary words that are also franchise names: blocked only as the whole name.
const BLOCKLIST_WHOLE = [
  "link", "tails", "knuckles", "donkey", "kong", "cloud", "kingdom", "solid", "snake", "resident", "nemesis", "hollow", "knight", "hornet", "sans",
  "souls", "league", "legends", "peach", "wonder", "flash", "spider", "rocket", "venom", "bond", "solo", "sailor", "homer", "ken", "steve", "stark",
  "harley", "donald", "goofy", "frozen", "halo", "doom", "dune", "lara", "elden", "rocky", "zed", "jinx", "tracer", "creeper", "harry", "tetris",
  "predator", "transformer", "sherlock", "mod", "dev", "free", "com", "system", "staff", "official", "approved", "hollow knight", "solid snake",
  "donkey kong", "wonder woman", "iron man", "spider man", "black widow", "captain america", "dark souls", "free vbucks",
];
const BLOCKED = new Set(BLOCKLIST.map(fold));
const BLOCKED_WHOLE = new Set(BLOCKLIST_WHOLE.map(fold));

function fold(s: string): string {
  return s.normalize("NFKD").replace(/[̀-ͯ]/g, "").toLowerCase().replace(/[^a-z0-9]/g, "");
}

/** A model reply made into a name, or null when it isn't one we'd show:
 * 1–3 words of letters, 3–20 characters, nothing on the blocklist, and not
 * just one of the parts' names. */
export function cleanModelName(raw: string, first?: UnitContent, second?: UnitContent): string | null {
  const line = raw.replace(/<think>[\s\S]*?<\/think>/g, "").trim().split("\n")[0] ?? "";
  const name = line
    .replace(/^(name|fusion|fused name)\s*:\s*/i, "")
    .replace(/["“”«»*_`.!]/g, "")
    .replace(/\s+/g, " ")
    .trim();
  if (!/^[A-Za-z][A-Za-z' -]{2,19}$/.test(name)) return null;
  const words = name.split(/[ -]/).filter(Boolean);
  if (words.length > 3) return null;
  if (BLOCKED.has(fold(name)) || BLOCKED_WHOLE.has(fold(name)) || words.some((w) => BLOCKED.has(fold(w)))) return null;
  if ([first, second].some((u) => u && fold(u.name) === fold(name))) return null;
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
              "You name fused creatures in a fantasy auto-battler. Answer with one new invented name only: one or two words, at most 18 letters, no quotes, no explanation. Never use a name from an existing game, film, book or comic.",
          },
          { role: "user", content: `Fuse ${first.emoji} ${first.name} (first) with ${second.emoji} ${second.name} (second). Name:` },
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
  /** Pairs the model already answered (or refused) once; never asked again. */
  asked: Set<string>;
  /** Fused units named by the fallback, renamed when the model answers. */
  fallbackUnits: Map<string, { runId: string; uid: string }[]>;
  wake?: (() => void) | undefined;
  now: () => Date;
}

const MAX_QUEUE = 200;
const namers = new WeakMap<MvpStore, Namer>();

const keyOf = (first: UnitId, second: UnitId) => JSON.stringify([first, second]);

export function fusionNaming(
  rt: Pick<RunDeps, "store" | "content" | "now">,
  opts: { model?: ModelNamer | null } = {},
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
    now: rt.now,
  };
  namers.set(store, n);
  const enqueue = (first: UnitContent, second: UnitContent) => {
    const key = keyOf(first.id, second.id);
    if (!n.model || n.queued.has(key) || n.asked.has(key) || n.prepared.has(key) || n.queue.length >= MAX_QUEUE) return;
    n.queued.add(key);
    n.queue.push({ key, first, second });
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
        // The first human to fuse a pair only bots had made claims it.
        if (known.discoveredBy === null && ctx.discoveredBy) store.putFusion({ ...known, discoveredBy: ctx.discoveredBy });
        if (known.nameSource === "fallback") n.fallbackUnits.get(key)?.push({ runId: run.runId, uid: fused.uid });
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
      if (!fromModel) {
        n.fallbackUnits.set(key, [{ runId: run.runId, uid: fused.uid }]);
        const first = n.units.get(parts.first);
        const second = n.units.get(parts.second);
        if (first && second) enqueue(first, second);
      }
    },
  };
  return { nameFusion: peek, peek, hooks };
}

/** Asks the model for every queued pair, one at a time, and applies each
 * answer: a pair nobody has fused gets a prepared name, a pair stored with the
 * fallback is renamed once. Resolves when the queue is empty. */
export async function drainFusionNames(store: MvpStore): Promise<void> {
  const n = namers.get(store);
  if (!n?.model) return;
  for (let pair = n.queue.shift(); pair; pair = n.queue.shift()) {
    n.queued.delete(pair.key);
    if (n.asked.has(pair.key)) continue;
    let name: string | null = null;
    try {
      const raw = await n.model(pair.first, pair.second);
      name = raw === null ? null : cleanModelName(raw, pair.first, pair.second);
    } catch {
      continue; // down or slow: the pair keeps the portmanteau, and may be asked again
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
  if (known.nameSource !== "fallback") return;
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

/** The naming queue: drains whenever a pair is queued. */
export const fusionNamingJob: MvpJob = (rt) => {
  const n = namers.get(rt.store);
  if (!n?.model) return () => {};
  let running = false;
  let stopped = false;
  const wake = () => {
    if (running || stopped) return;
    running = true;
    void drainFusionNames(rt.store).finally(() => {
      running = false;
      if (n.queue.length > 0 && !stopped) setTimeout(wake, 1000);
    });
  };
  n.wake = wake;
  wake();
  return () => {
    stopped = true;
    n.wake = undefined;
  };
};
