// Russian names and lines for stored units (M4-4, mission #810). Claude
// translates each unit's name and line (its archetype sentence) once; the
// answer is kept beside the unit (StoredUnit.texts.ru), never in its row, so
// the content version and every golden stay the same. `npm run mvp:translate`
// (./translate-cli.ts) runs it and writes a review page for Maks.
import { callClaude, type ClaudeReaderOptions } from "./idea-reader.js";
import { isBlockedName } from "./fusions.js";
import { isCrudeRuLine, isCrudeRuName } from "./crude.js";
import type { RuFusionReport } from "./fusion-names-ru.js";
import { nameKey, type MvpStore, type StoredUnit } from "./store.js";

/** What a translator is given: a unit's English name and line. */
export interface UnitSource {
  unitId: string;
  emoji: string;
  name: string;
  line: string;
}

export interface RuText {
  unitId: string;
  name: string;
  line: string;
}

export interface Translator {
  readonly kind: "claude" | "fake" | string;
  /** Russian names and lines for `units`; `taken` are Russian names already
   * used, `notes` why an earlier answer for a unit was refused. */
  translate(units: UnitSource[], taken: readonly string[], notes?: ReadonlyMap<string, string>): Promise<RuText[]>;
}

/** Why a Russian name or line can't be kept, or null: 1–3 words of Cyrillic
 * letters (a hyphen or apostrophe inside), at most 24 characters, a capital
 * first, nothing crude, not taken; a line of one sentence, at most 140
 * characters, ending in a full stop. */
export function ruTextProblem(t: { name: string; line: string }, taken: ReadonlySet<string>): string | null {
  const name = t.name.trim();
  if (!/^[А-ЯЁ][А-Яа-яЁё'’-]*( [А-Яа-яЁё'’-]+){0,2}$/.test(name)) return "the name must be 1–3 Russian words with a capital first";
  if (name.length > 24) return "the name is longer than 24 characters";
  if (isCrudeRuName(name) || isBlockedName(name)) return "the name reads crude";
  if (taken.has(nameKey(name))) return "another unit already has this Russian name";
  const line = t.line.trim();
  if (!/[а-яё]/i.test(line)) return "the line must be in Russian";
  if (line.length > 140) return "the line is longer than 140 characters";
  if (!/[.!]$/.test(line)) return "the line must end with a full stop";
  if (isCrudeRuLine(line)) return "the line reads crude";
  return null;
}

export interface TranslateReport {
  /** Units given a Russian name and line by this run (written unless dry). */
  done: { unit: StoredUnit; ru: { name: string; line: string } }[];
  /** Units that already had one: kept as they are. */
  kept: StoredUnit[];
  /** Units no answer could be kept for, with the last reason. */
  failed: { unit: StoredUnit; why: string }[];
}

export interface TranslateOpts {
  dryRun?: boolean;
  /** Units per call (default 20). */
  batch?: number;
  /** Calls per unit before it is given up (default 2). */
  tries?: number;
  log?: (line: string) => void;
}

/** The unit's English name and line. */
export const sourceOf = (u: StoredUnit): UnitSource => ({ unitId: u.unitId, emoji: u.row.emoji, name: u.row.name, line: u.row.archetype });

/** Translates every stored unit that isn't rejected and has no Russian text
 * yet; writes each kept answer beside the unit (re-read first, so a status
 * change the server made meanwhile stays) unless `dryRun`. */
export async function translateUnits(store: MvpStore, tr: Translator, opts: TranslateOpts = {}): Promise<TranslateReport> {
  const batch = opts.batch ?? 20;
  const tries = opts.tries ?? 2;
  const all = store.units().filter((u) => u.status !== "rejected");
  const kept = all.filter((u) => u.texts?.ru);
  const taken = new Set(kept.map((u) => nameKey(u.texts!.ru!.name)));
  const report: TranslateReport = { done: [], kept, failed: [] };
  let todo = all.filter((u) => !u.texts?.ru);
  const notes = new Map<string, string>();
  for (let attempt = 1; attempt <= tries && todo.length; attempt++) {
    const next: StoredUnit[] = [];
    for (let i = 0; i < todo.length; i += batch) {
      const group = todo.slice(i, i + batch);
      let answers: RuText[] = [];
      try {
        answers = await tr.translate(group.map(sourceOf), [...taken], notes);
      } catch (e) {
        opts.log?.(`[translate] a call failed: ${(e as Error).message}`);
      }
      const byId = new Map(answers.map((a) => [a.unitId, a]));
      for (const u of group) {
        const a = byId.get(u.unitId);
        const problem = a ? ruTextProblem(a, taken) : "no answer";
        const why = problem && a ? `${problem} (${a.name} | ${a.line})` : problem;
        if (why) {
          notes.set(u.unitId, why);
          next.push(u);
          continue;
        }
        const ru = { name: a!.name.trim(), line: a!.line.trim() };
        taken.add(nameKey(ru.name));
        notes.delete(u.unitId);
        report.done.push({ unit: u, ru });
        if (!opts.dryRun) {
          const fresh = store.unit(u.unitId);
          if (fresh) store.putUnit({ ...fresh, texts: { ...fresh.texts, ru } });
        }
      }
      opts.log?.(`[translate] ${report.done.length} done, ${next.length} to retry (try ${attempt})`);
    }
    todo = next;
  }
  report.failed = todo.map((u) => ({ unit: u, why: notes.get(u.unitId) ?? "no answer" }));
  return report;
}

const cell = (s: string) => s.replace(/\|/g, "\\|").replace(/\n/g, " ");

/** The one page Maks skims before the names go live: emoji, English name and
 * line, Russian name and line, by status then English name. */
export function reviewPage(r: TranslateReport, opts: { dryRun?: boolean; at?: string; fusions?: RuFusionReport } = {}): string {
  const rows = [
    ...r.done.map((d) => ({ u: d.unit, ru: d.ru, note: "" })),
    ...r.kept.map((u) => ({ u, ru: u.texts!.ru!, note: "kept" })),
    ...r.failed.map((f) => ({ u: f.unit, ru: { name: "—", line: "—" }, note: `failed: ${f.why}` })),
  ];
  const order = { live: 0, library: 1, candidate: 2, rejected: 3 } as const;
  rows.sort((x, y) => order[x.u.status] - order[y.u.status] || x.u.row.name.localeCompare(y.u.row.name));
  return [
    "# Russian unit names (M4-4)",
    "",
    `Made by \`npm run mvp:translate\`${opts.dryRun ? " (dry run: nothing written)" : ""}${opts.at ? ` on ${opts.at}` : ""}. ` +
      `${r.done.length} new, ${r.kept.length} kept, ${r.failed.length} failed. Rejected units are left out.`,
    "",
    "| | English | | Русский | | Status |",
    "|---|---|---|---|---|---|",
    ...rows.map(({ u, ru, note }) => `| ${u.row.emoji} | **${cell(u.row.name)}** | ${cell(u.row.archetype)} | **${cell(ru.name)}** | ${cell(ru.line)} | ${u.status}${note ? `, ${cell(note)}` : ""} |`),
    "",
    ...(opts.fusions ? fusionSection(opts.fusions) : []),
  ].join("\n");
}

/** The fusions the namer named in Russian, and those it couldn't (they show the English name). */
function fusionSection(f: RuFusionReport): string[] {
  return [
    "## Fusion names",
    "",
    `The local namer's Russian mode: ${f.named.length} named, ${f.failed.length} left English${f.down ? " (the namer went down; the rest wait for the next pass)" : ""}.`,
    "",
    "| English | Русский | Parts |",
    "|---|---|---|",
    ...f.named.map(({ fusion, name }) => `| ${cell(fusion.name)} | **${cell(name)}** | ${fusion.first} + ${fusion.second} |`),
    ...f.failed.map((fusion) => `| ${cell(fusion.name)} | — | ${fusion.first} + ${fusion.second} |`),
    "",
  ];
}

// ---------- claude -p ----------

export const TRANSLATE_SYSTEM = [
  "You translate unit names and lines of Arena of Ideas, a fantasy auto-battler played on phones, from English into Russian.",
  "A name is what players call the unit: 1 to 3 Russian words, a capital first, no emoji, no quotes, at most 24 characters.",
  "Prefer a natural Russian fantasy name over a transliteration (Squire → Оруженосец, Gnat → Мошка); keep a proper name as a proper name.",
  "A line is one sentence on what the unit is about: say the same thing in plain, short Russian in the game's voice, at most 15 words, ending with a full stop.",
  "Every Russian name must differ from every other one in the answer and from the names listed as taken.",
  "Nothing crude, no slang, no wordplay that only works in English.",
].join("\n");

const TRANSLATE_SCHEMA = {
  type: "object",
  properties: {
    units: {
      type: "array",
      items: { type: "object", properties: { unitId: { type: "string" }, name: { type: "string" }, line: { type: "string" } }, required: ["unitId", "name", "line"], additionalProperties: false },
    },
  },
  required: ["units"],
  additionalProperties: false,
};

export function translatePrompt(units: readonly UnitSource[], taken: readonly string[], notes?: ReadonlyMap<string, string>): string {
  return [
    taken.length ? `Russian names already taken: ${taken.join(", ")}.` : "No Russian names are taken yet.",
    "",
    "Translate each unit (unitId: emoji name | line). Answer with every unitId.",
    ...units.map((u) => `${u.unitId}: ${u.emoji} ${u.name} | ${u.line}${notes?.get(u.unitId) ? ` (an earlier answer was refused: ${notes.get(u.unitId)})` : ""}`),
  ].join("\n");
}

export function claudeTranslator(o: ClaudeReaderOptions): Translator {
  return {
    kind: "claude",
    async translate(units, taken, notes) {
      const out = (await callClaude(o, TRANSLATE_SYSTEM, translatePrompt(units, taken, notes), TRANSLATE_SCHEMA)) as { units?: unknown };
      return Array.isArray(out?.units) ? (out.units as RuText[]).filter((x) => typeof x?.unitId === "string" && typeof x.name === "string" && typeof x.line === "string") : [];
    },
  };
}

// ---------- the fake ----------

const LATIN: Record<string, string> = {
  a: "а", b: "б", c: "к", d: "д", e: "е", f: "ф", g: "г", h: "х", i: "и", j: "дж", k: "к", l: "л", m: "м", n: "н", o: "о", p: "п",
  q: "к", r: "р", s: "с", t: "т", u: "у", v: "в", w: "в", x: "кс", y: "й", z: "з",
};

/** English letters spelled in Cyrillic, case kept ("Gnat" → "Гнат"). */
export function cyrillic(s: string): string {
  return s.replace(/[A-Za-z]/g, (ch) => {
    const ru = LATIN[ch.toLowerCase()]!;
    return ch === ch.toLowerCase() ? ru : ru[0]!.toUpperCase() + ru.slice(1);
  });
}

/** A deterministic translator for tests and dry runs without Claude: the
 * name spelled in Cyrillic (a number added while taken), the line the same
 * way after "Ру:". */
export function fakeTranslator(): Translator {
  return {
    kind: "fake",
    async translate(units, taken) {
      const used = new Set(taken.map(nameKey));
      return units.map((u) => {
        const base = cyrillic(u.name).replace(/[^А-Яа-яЁё' -]/g, "").trim().slice(0, 20) || "Безымянный";
        let name = base;
        for (let n = 2; used.has(nameKey(name)); n++) name = `${base} ${"абвгдежзик"[n % 10]}`;
        used.add(nameKey(name));
        return { unitId: u.unitId, name, line: `Ру: ${cyrillic(u.line)}`.replace(/[^.!]$/, (c) => `${c}.`) };
      });
    },
  };
}
