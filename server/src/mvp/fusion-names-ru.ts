// Russian fusion names (M4-4, mission #810). The local namer (ARENA_NAMER_URL,
// the model on m1's :8792) is asked in Russian for every stored discovery that
// has no Russian name yet, and the answer is kept beside the English one
// (FusionDiscovery.texts.ru). The English name and its rules (./fusions.ts)
// stay as they are; a pair without a Russian name shows the English one.
import type { FusionDiscovery } from "../../../src/mvp/contract.js";
import { isCrudeRuName } from "./crude.js";
import { isBlockedName } from "./fusions.js";
import type { MvpJob } from "./runtime.js";
import { nameKey, type MvpStore } from "./store.js";

/** Asks the model for a raw Russian name for two fighters; null or a throw means none. */
export type RuNamer = (first: string, second: string) => Promise<string | null>;

export const NAMER_SYSTEM_RU =
  "Ты придумываешь имена существам в фэнтези-автобаттлере. Два бойца сливаются в одно новое существо. Придумай ему свежее, звучное русское имя: одно слово из одного или двух русских корней, легко произнести, не больше 14 букв. Не склеивай и не повторяй имена бойцов, без эмодзи, без кавычек, без пояснений, и никогда не бери имя из существующей игры, фильма, книги или комикса.";

/** Few-shot turns, all one word; no unit is named like these fighters. */
export const NAMER_EXAMPLES_RU: readonly { first: string; second: string; name: string }[] = [
  { first: "Рыцарь", second: "Волк", name: "Клыкостраж" },
  { first: "Искра", second: "Целитель", name: "Грозолекарь" },
  { first: "Голем", second: "Ворон", name: "Могилокрыл" },
  { first: "Вор", second: "Монах", name: "Милостырез" },
];

// No case after a preposition to get wrong: "Сливаются: Боец и Мошка."
const ask = (first: string, second: string) => `Сливаются: ${first} и ${second}. Имя:`;

export function namerMessagesRu(first: string, second: string): { role: "system" | "user" | "assistant"; content: string }[] {
  return [
    { role: "system", content: NAMER_SYSTEM_RU },
    ...NAMER_EXAMPLES_RU.flatMap((e) => [
      { role: "user" as const, content: ask(e.first, e.second) },
      { role: "assistant" as const, content: e.name },
    ]),
    { role: "user", content: ask(first, second) },
  ];
}

/** The namer's Russian mode, through the same OpenAI-compatible endpoint. */
export function httpRuNamer(url: string, timeoutMs = 15_000): RuNamer {
  return async (first, second) => {
    const res = await fetch(url, {
      method: "POST",
      headers: { "content-type": "application/json" },
      signal: AbortSignal.timeout(timeoutMs),
      body: JSON.stringify({ messages: namerMessagesRu(first, second), max_tokens: 16, temperature: 0.8 }),
    });
    if (!res.ok) throw new Error(`namer answered ${res.status}`);
    const body = (await res.json()) as { choices?: { message?: { content?: string } }[] };
    return body.choices?.[0]?.message?.content ?? null;
  };
}

const foldRu = (s: string) => s.toLowerCase().replace(/ё/g, "е").replace(/[^а-я]/g, "");

/** A model reply made into a Russian name, or null: 1–2 Cyrillic words, 3–20
 * characters, nothing crude, not just the parts' names glued (what is left
 * after taking out their words is under three letters). */
export function cleanRuName(raw: string, first: string, second: string): string | null {
  const line = raw.replace(/<think>[\s\S]*?<\/think>/g, "").trim().split("\n")[0] ?? "";
  const name = line
    .replace(/^(имя|name)\s*:\s*/i, "")
    .replace(/["“”«»*_`.!]/g, "")
    .replace(/\s+/g, " ")
    .replace(/^[^А-Яа-яЁё]+|[^А-Яа-яЁё]+$/g, "");
  if (!/^[А-Яа-яЁё][а-яё-]{2,19}( [А-Яа-яЁё][а-яё-]{1,19})?$/.test(name) || name.length > 20) return null;
  if (isCrudeRuName(name) || isBlockedName(name)) return null;
  let rest = foldRu(name);
  for (const part of [first, second].flatMap((n) => n.split(/\s+/).map(foldRu)).filter((w) => w.length >= 3).sort((x, y) => y.length - x.length)) rest = rest.split(part).join("");
  if (rest.length < 3) return null;
  return name.split(" ").map((w) => w[0]!.toUpperCase() + w.slice(1).toLowerCase()).join(" ");
}

const TRIES = 3;

export interface RuFusionReport {
  named: { fusion: FusionDiscovery; name: string }[];
  /** Pairs the model gave no usable name for (they show the English one). */
  failed: FusionDiscovery[];
  /** The model was down: the pass stopped there. */
  down: boolean;
}

/** Asks for a Russian name for up to `limit` stored discoveries without one,
 * oldest first: each pair's parts by their Russian names where they have
 * them, a name unique among Russian fusion and unit names. Writes each (the
 * discovery re-read first) unless `dryRun`; `skip` leaves pairs out; `ruNames`
 * are units' Russian names not stored yet (a dry run's). A throw
 * (the model down) ends the pass; the next one picks up. */
export async function nameFusionsRu(store: MvpStore, namer: RuNamer, opts: { limit?: number; dryRun?: boolean; skip?: (f: FusionDiscovery) => boolean; ruNames?: ReadonlyMap<string, string> } = {}): Promise<RuFusionReport> {
  const report: RuFusionReport = { named: [], failed: [], down: false };
  const units = new Map(store.units().map((u) => [u.unitId, u]));
  const fusions = store.fusions();
  const taken = new Set([
    ...fusions.flatMap((f) => (f.texts?.ru ? [nameKey(f.texts.ru.name)] : [])),
    ...[...units.values()].flatMap((u) => (u.texts?.ru ? [nameKey(u.texts.ru.name)] : [])),
    ...[...(opts.ruNames?.values() ?? [])].map(nameKey),
  ]);
  const partName = (id: string) => {
    const u = units.get(id);
    return opts.ruNames?.get(id) ?? u?.texts?.ru?.name ?? u?.row.name ?? id;
  };
  const todo = fusions.filter((f) => !f.texts?.ru && !opts.skip?.(f)).sort((x, y) => (x.discoveredAt < y.discoveredAt ? -1 : 1)).slice(0, opts.limit ?? Infinity);
  for (const f of todo) {
    const [a, b] = [partName(f.first), partName(f.second)];
    let name: string | null = null;
    try {
      for (let i = 0; i < TRIES && !name; i++) {
        const raw = await namer(a, b);
        name = raw === null ? null : cleanRuName(raw, a, b);
        if (name && taken.has(nameKey(name))) name = null;
      }
    } catch {
      report.down = true;
      break;
    }
    if (!name) {
      report.failed.push(f);
      continue;
    }
    taken.add(nameKey(name));
    report.named.push({ fusion: f, name });
    if (!opts.dryRun) {
      const fresh = store.fusion(f.first, f.second);
      if (fresh && !fresh.texts?.ru) store.putFusion({ ...fresh, texts: { ...fresh.texts, ru: { name } } });
    }
  }
  return report;
}

/** How often the runtime names discoveries in Russian, and how many at most:
 * few, so the shared model stays free for the English names players wait on. */
export const RU_NAMING_MS = 60_000;
const RU_NAMING_BATCH = 3;

/** The Russian naming pass, every RU_NAMING_MS while ARENA_NAMER_URL is set
 * and ARENA_NAMER_RU=1: off until Maks has seen the namer's Russian sample
 * (docs/mission4/names-ru.md), since a stored name stays for good.
 * A pair the model can't name isn't asked again until the server restarts. */
export const fusionRuNamingJob: MvpJob = (rt) => {
  const url = process.env.ARENA_NAMER_URL;
  if (!url || process.env.ARENA_NAMER_RU !== "1") return () => {};
  const namer = httpRuNamer(url);
  let running = false;
  const gaveUp = new Set<string>();
  const key = (f: FusionDiscovery) => `${f.first} ${f.second}`;
  const tick = () => {
    if (running) return;
    running = true;
    void nameFusionsRu(rt.store, namer, { limit: RU_NAMING_BATCH, skip: (f) => gaveUp.has(key(f)) })
      .then((r) => r.failed.forEach((f) => gaveUp.add(key(f))))
      .catch((e: unknown) => console.warn(`[namer-ru] ${(e as Error).message}`))
      .finally(() => (running = false));
  };
  const timer = setInterval(tick, RU_NAMING_MS);
  return () => clearInterval(timer);
};
