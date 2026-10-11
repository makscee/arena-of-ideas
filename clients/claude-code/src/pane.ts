// The Arena pane in Claude Code (#960), as data: the lines it shows and what
// each key does. No engine calls here (hooks/register.tsx makes them), so
// pane.spec.ts plays it without Claude Code. Board numbers are 1-based on
// screen (the line 1..lineSize, then the bench), 0-based in a Decision.
import type { Decision, LineUnit, MvpContent, MvpRules, RunView } from "../../../src/mvp/contract.js";
import { lineUnitText, unitName, type TextOpts } from "../../core/text.js";

/** A key waiting for its number: `s` then a board slot, `m` then two, … */
export type Pending =
  | { kind: "sell" | "lock" | "move" | "fuse" }
  | { kind: "move2" | "fuse2"; first: number };

/** Claude's pick, shown until the player takes it with `y` or it goes stale. */
export interface Suggestion {
  runId: string;
  decision: Decision;
  why: string;
}

export interface Key {
  key: string;
  label: string;
}

/** What a key press asks for. */
export type Press =
  | { decision: Decision }
  | { pending: Pending | null }
  | { do: "battle" | "accept" | "lang" | "ranked" | "practice" | "back" | "older" | "newer" }
  | { none: true };

const benchOf = (r: MvpRules) => r.benchSize ?? 0;

const KEY_WORDS = {
  en: { older: "older", newer: "newer", back: "back", lang: "RU/EN", ranked: "ranked run", practice: "practice run", battle: "battle", cancel: "cancel", gift: "gift", skip: "skip gift", buy: "buy", sell: "sell", reroll: "reroll", lock: "lock", move: "move", fuse: "fuse", fight: "fight", accept: "accept", which: { sell: "sell which", lock: "lock which", move: "move which", move2: "move to", fuse: "fuse which", fuse2: "fuse with" } },
  ru: { older: "раньше", newer: "дальше", back: "назад", lang: "RU/EN", ranked: "рейтинговый забег", practice: "тренировка", battle: "бой", cancel: "отмена", gift: "дар", skip: "без дара", buy: "купить", sell: "продать", reroll: "обновить", lock: "закрепить", move: "переставить", fuse: "слить", fight: "в бой", accept: "принять", which: { sell: "продать какой", lock: "закрепить какой", move: "переставить какой", move2: "переставить на", fuse: "слить какой", fuse2: "слить с" } },
} as const;

const range = (n: number) => (n > 1 ? `1-${n}` : "1");

/** The keys the pane holds now, in the order they're drawn. */
export function keysFor(run: RunView | null, rules: MvpRules | null, pending: Pending | null, suggestion: Suggestion | null, view: "run" | "battle", o: TextOpts = {}): Key[] {
  const w = KEY_WORDS[o.lang === "ru" ? "ru" : "en"];
  if (view === "battle") return [{ key: "k", label: w.older }, { key: "j", label: w.newer }, { key: "b", label: w.back }, { key: "t", label: w.lang }];
  if (!run || !rules || run.phase === "over") return [{ key: "n", label: w.ranked }, { key: "p", label: w.practice }, ...(run?.fights.length ? [{ key: "b", label: w.battle }] : []), { key: "t", label: w.lang }];
  if (pending) return [{ key: "1", label: `${w.which[pending.kind]} (${range(pending.kind === "lock" ? run.offers.length : rules.lineSize + benchOf(rules))})` }, { key: "c", label: w.cancel }];
  const keys: Key[] = [];
  if (run.gift?.length) keys.push({ key: "1", label: `${w.gift} ${range(run.gift.length)}` }, { key: "x", label: w.skip });
  else if (run.offers.length) keys.push({ key: "1", label: `${w.buy} ${range(run.offers.length)}` });
  keys.push({ key: "s", label: w.sell });
  if (!run.gift?.length) keys.push({ key: "r", label: w.reroll }, { key: "l", label: w.lock });
  keys.push({ key: "m", label: w.move });
  if (!run.gift?.length) keys.push({ key: "f", label: w.fuse }, { key: "g", label: w.fight });
  if (run.fights.length) keys.push({ key: "b", label: w.battle });
  if (suggestion && suggestion.runId === run.runId) keys.push({ key: "y", label: w.accept });
  keys.push({ key: "t", label: w.lang });
  return keys;
}

/** Every key a screen holds: the drawn ones and each digit (a pane can't
 * hold a range on one Button). */
export const heldKeys = (keys: Key[]): string[] => [...new Set([...keys.map((k) => k.key), "1", "2", "3", "4", "5", "6", "7", "8", "9"])];

/** What `key` does on the run shown. */
export function press(key: string, run: RunView | null, rules: MvpRules | null, pending: Pending | null, suggestion: Suggestion | null, view: "run" | "battle"): Press {
  if (key === "t") return { do: "lang" };
  if (view === "battle") return key === "b" ? { do: "back" } : key === "k" ? { do: "older" } : key === "j" ? { do: "newer" } : { none: true };
  if (key === "b") return { do: "battle" };
  if (!run || !rules || run.phase === "over") return key === "n" ? { do: "ranked" } : key === "p" ? { do: "practice" } : { none: true };
  const n = /^[1-9]$/.test(key) ? Number(key) - 1 : -1;
  const board = rules.lineSize + benchOf(rules);
  if (pending) {
    if (key === "c") return { pending: null };
    if (n < 0) return { none: true };
    switch (pending.kind) {
      case "sell":
        return n < board ? { decision: { kind: "sell", index: n } } : { none: true };
      case "lock":
        return n < run.offers.length ? { decision: { kind: "lock", slot: n } } : { none: true };
      case "move":
        return n < board ? { pending: { kind: "move2", first: n } } : { none: true };
      case "fuse":
        return n < board ? { pending: { kind: "fuse2", first: n } } : { none: true };
      case "move2":
        return n < board ? { decision: { kind: "reorder", from: pending.first, to: n } } : { none: true };
      case "fuse2":
        return n < board ? { decision: { kind: "fuse", first: pending.first, second: n } } : { none: true };
    }
  }
  if (n >= 0) {
    if (run.gift?.length) return n < run.gift.length ? { decision: { kind: "gift", pick: n } } : { none: true };
    return n < run.offers.length ? { decision: { kind: "buy", slot: n } } : { none: true };
  }
  switch (key) {
    case "x":
      return run.gift?.length ? { decision: { kind: "gift", pick: null } } : { none: true };
    case "r":
      return { decision: { kind: "reroll" } };
    case "g":
      return { decision: { kind: "fight" } };
    case "s":
    case "l":
    case "m":
    case "f":
      return { pending: { kind: ({ s: "sell", l: "lock", m: "move", f: "fuse" } as const)[key] } };
    case "y":
      return suggestion && suggestion.runId === run.runId ? { do: "accept" } : { none: true };
  }
  return { none: true };
}

const offerName = (run: RunView, slot: number, content: MvpContent, o: TextOpts) => {
  const of = run.offers.find((x) => x.slot === slot);
  const u = of && content.units.find((x) => x.id === of.unitId);
  return u ? unitName(u.id, u.name, content, o) : "?";
};
const boardUnit = (run: RunView, rules: MvpRules, i: number): LineUnit | undefined => (i < rules.lineSize ? run.line[i] : run.bench[i - rules.lineSize]);
const boardName = (run: RunView, rules: MvpRules, i: number, content: MvpContent, o: TextOpts) => {
  const u = boardUnit(run, rules, i);
  return u ? (u.kind === "fused" ? u.name : unitName(u.unitId, u.name, content, o)) : "—";
};

/** A move in the pane's numbers: "buy 2 Priest", "move 1 Keeper → 3". */
export function decisionText(d: Decision, run: RunView, rules: MvpRules, content: MvpContent, o: TextOpts = {}): string {
  switch (d.kind) {
    case "buy":
      return `buy ${d.slot + 1} ${offerName(run, d.slot, content, o)}`;
    case "lock":
      return `lock ${d.slot + 1} ${offerName(run, d.slot, content, o)}`;
    case "sell":
      return `sell ${d.index + 1} ${boardName(run, rules, d.index, content, o)}`;
    case "reorder":
      return `move ${d.from + 1} ${boardName(run, rules, d.from, content, o)} → ${d.to + 1}`;
    case "fuse":
      return `fuse ${d.first + 1} ${boardName(run, rules, d.first, content, o)} + ${d.second + 1} ${boardName(run, rules, d.second, content, o)}`;
    case "gift": {
      const id = d.pick === null ? undefined : run.gift?.[d.pick];
      return d.pick === null ? "skip the gift" : `gift ${d.pick + 1} ${id ? unitName(id, content.units.find((u) => u.id === id)?.name ?? id, content, o) : "?"}`;
    }
    case "reroll":
      return "reroll";
    case "fight":
      return "fight";
  }
  return JSON.stringify(d);
}

const short = (u: LineUnit, content: MvpContent, o: TextOpts) =>
  `${u.emoji} ${u.kind === "fused" ? u.name : unitName(u.unitId, u.name, content, o)} ${u.stats.pwr}/${u.stats.hp}${u.kind === "fused" ? " ✦" : u.form === "awoken" ? " ★" : ` ${u.copies}/3`}`;

/** The run as the pane's lines: header, SHOP, LINE, BENCH (numbers 1-based,
 * as the keys take them), the gift, the last fight. `wide` adds what each
 * shop unit and board unit does. */
export function paneLines(run: RunView, content: MvpContent, rules: MvpRules, o: TextOpts = {}, wide = false): string[] {
  const ru = o.lang === "ru";
  const tag = (en: string, r: string) => (ru ? r : en).padEnd(7) + " ";
  const blank = "".padEnd(8);
  const outcome = (x: string) => (ru ? ({ win: "победа", loss: "поражение", draw: "ничья" } as Record<string, string>)[x] ?? x : x);
  const round = run.phase === "crown" || run.round > rules.rounds ? (ru ? "Корона" : "Crown") : `${ru ? "Раунд" : "Round"} ${run.round}/${rules.rounds}`;
  const vs = run.nextOpponent ? ` · vs @${run.nextOpponent.player.name}` : "";
  const mode = run.mode === "practice" ? (ru ? "тренировка" : "practice") : ru ? "рейтинг" : "ranked";
  const out = [`Arena · ${round} · ♥${run.hearts} · ${run.gold}g · ${run.wins}W ${run.losses}L${vs}   [${mode}]`];
  if (run.phase === "over") out.push(`${ru ? "Забег окончен" : "Run over"}${run.endedBy ? ` (${run.endedBy})` : ""}${run.rating ? ` · ${ru ? "рейтинг" : "rating"} ${run.rating.before} → ${run.rating.after}` : ""}`);
  if (run.gift?.length) out.push(`${tag("GIFT", "ДАР")}${run.gift.map((id, i) => `${i + 1} ${unitName(id, content.units.find((u) => u.id === id)?.name ?? id, content, o)}`).join("   ")}`);
  if (run.offers.length && run.phase !== "over") {
    if (wide)
      for (const of of run.offers) {
        const u = content.units.find((x) => x.id === of.unitId);
        out.push(`${of.slot === 0 ? tag("SHOP", "МАГАЗИН") : blank}${of.slot + 1} ${u ? `${u.emoji} ${unitName(u.id, u.name, content, o)} ${u.base.pwr}/${u.base.hp}` : of.unitId} T${of.tier} ${of.cost}g${of.locked ? " 🔒" : ""}`);
      }
    else out.push(`${tag("SHOP", "МАГАЗИН")}${run.offers.map((of) => `${of.slot + 1} ${offerName(run, of.slot, content, o)} T${of.tier} ${of.cost}g${of.locked ? " 🔒" : ""}`).join("   ")}`);
  }
  const slots = (from: number, n: number, units: LineUnit[]) => Array.from({ length: n }, (_, i) => `${from + i} ${units[i] ? short(units[i]!, content, o) : "—"}`);
  if (wide) {
    for (let i = 0; i < rules.lineSize; i++) out.push(`${i === 0 ? tag("LINE", "СТРОЙ") : blank}${i + 1} ${run.line[i] ? lineUnitText(run.line[i]!, content, o) : "—"}`);
    for (let i = 0; i < benchOf(rules); i++) out.push(`${i === 0 ? tag("BENCH", "СКАМЬЯ") : blank}${rules.lineSize + i + 1} ${run.bench[i] ? lineUnitText(run.bench[i]!, content, o) : "—"}`);
  } else {
    out.push(`${tag("LINE", "СТРОЙ")}${slots(1, rules.lineSize, run.line).join(" · ")}`);
    if (benchOf(rules)) out.push(`${tag("BENCH", "СКАМЬЯ")}${slots(rules.lineSize + 1, benchOf(rules), run.bench).join(" · ")}`);
  }
  const last = run.fights.at(-1);
  if (last) out.push(`${tag("LAST", "БОЙ")}${last.kind === "crown" ? (ru ? "Корона" : "Crown") : `${ru ? "раунд" : "round"} ${last.round}`} vs @${last.opponent.player.name}: ${outcome(last.outcome)}${last.heartsLost ? ` (−${last.heartsLost}♥)` : ""}`);
  return out;
}

/** A page of battle text: `page` 0 is the top, each `rows` tall. */
export function battlePage(text: string, page: number, rows: number): { lines: string[]; page: number; pages: number } {
  const all = text.split("\n");
  const pages = Math.max(1, Math.ceil(all.length / rows));
  const p = Math.min(Math.max(0, page), pages - 1);
  return { lines: all.slice(p * rows, (p + 1) * rows), page: p, pages };
}

/** `$.http.fetch` as the `fetch` clients/core calls: what it reads of a Response. */
export function hostFetch(f: (url: string, init?: { method?: string; headers?: Record<string, string>; body?: string }) => Promise<{ status: number; ok: boolean; text: string }>): typeof fetch {
  return (async (url: string, init?: { method?: string; headers?: Record<string, string>; body?: string }) => {
    const r = await f(String(url), { ...(init?.method ? { method: init.method } : {}), ...(init?.headers ? { headers: init.headers } : {}), ...(init?.body !== undefined ? { body: init.body } : {}) });
    return { ok: r.ok, status: r.status, statusText: `HTTP ${r.status}`, json: async () => JSON.parse(r.text) };
  }) as unknown as typeof fetch;
}
