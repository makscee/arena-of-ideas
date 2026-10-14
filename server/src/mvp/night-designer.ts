// The night batch's designer (rules v2 §5, makscee/void-board#1050): Opus
// designs three candidates per pair, Sonnet names them in EN/RU. Both run as
// `claude -p` like the idea reader (no tools, a JSON schema, a timeout), on
// the workers' token. ARENA_NIGHT_DESIGNER=claude|fake (fake by default),
// ARENA_NIGHT_MODEL (opus), ARENA_NIGHT_NAMER_MODEL (sonnet),
// ARENA_CLAUDE_BIN, ARENA_NIGHT_TIMEOUT_MS.

import { readFileSync } from "node:fs";
import { join } from "node:path";
import { DejaVu, FIXTURE_ABILITIES, FIXTURE_STATUSES, GraveTree } from "../../../src/words/fixtures.js";
import { MACROS } from "../../../src/words/macros.js";
import { callClaude } from "./idea-reader.js";
import { type DesignDraft, type Designer, LEANS, type Naming, type Parent } from "./night-batch.js";

const ROOT = join(import.meta.dirname, "../../..");

/** The rules for the designer, rules v2 §5 (the four rules) and §7 (words). */
export const DESIGNER_RULES = [
  "Use the shape of both parents: a counter from one, a flip from the other. Two different pairs must not come out looking alike.",
  "Add one element neither parent has (Déjà Vu's 'one turn later', Grave Tree's inherited rule).",
  "Three candidates lean differently: lean A (toward the first parent), lean B (toward the second), and twist (reads as a third creature).",
  "Output is the v2 schema only. No stats: the fusion has the parents' summed HP and power; numbers inside the rule are a first guess the quick tune rescales.",
];

/** The engine's rule language, read from the source so it never drifts. */
function language(): string {
  const types = readFileSync(join(ROOT, "src/types.ts"), "utf8");
  const end = types.indexOf("// ---------- Events");
  const words = readFileSync(join(ROOT, "src/words/types.ts"), "utf8");
  return `${end > 0 ? types.slice(0, end) : types}\n\n// ---- v2 words (src/words/types.ts) ----\n${words}`;
}

export function designerSystemPrompt(): string {
  const macros = Object.values(MACROS).map((m) => `- ${m.name}: ${m.en}`).join("\n");
  return [
    "You design fusions for Arena of Ideas, an auto-battler. A fusion of two units is a third creature with ONE designed rule (When → Who → Does), not a splice of its parents.",
    "",
    "Rules for the designer:",
    ...DESIGNER_RULES.map((r, i) => `${i + 1}. ${r}`),
    "",
    "The rule is engine JSON in the types below: `rule` holds the unit's triggers, optional condition, selectors, abilities (ids) and optional also-clauses and starting statuses; `abilities` and `statuses` hold the registry entries your rule refers to (ids you make up; you may also reuse the parents' ids).",
    "A unit has one When. A second rule lives on a counter status (StatusDef with counter: true and its own triggers), as Raven's Memories do below. The shared counter is Charge (status "Charge", shown as ⚡): a unit gains 1 Charge in its Does and spends 3 the moment it holds 3, as Hourglass, Drummer, Battery and Archmage do; make up no counter of your own.",
    "A status you make up carries its names, as Charge and Memories do: `names` {en, ru: {name, acc, count: [one, few, many]}}, the Russian written natively with its accusative and its three counted forms (1 Заряд, 2 Заряда, 5 Зарядов). A status without them is refused.",
    "Rule-modifying effects (copyRule, swapRules, remember, performRemembered, actAgain) may not listen to a chain link: use them on root events (BattleStart, TurnStart, TurnEnd, Death, Acted).",
    `Composed words you may use as {"kind":"macro","name":…}:\n${macros}`,
    "If the design truly needs a word the language lacks, do not bend it: set `wordRequest` {word, meaning} and still give your best rule without it.",
    "",
    "Examples (the §4 units' registry and the two written fusions):",
    `abilities: ${JSON.stringify(FIXTURE_ABILITIES)}`,
    `statuses: ${JSON.stringify(FIXTURE_STATUSES)}`,
    `Déjà Vu (Hourglass + Mirror): ${JSON.stringify(DejaVu)}`,
    `Grave Tree (Raven + Seed): ${JSON.stringify(GraveTree)}`,
    "",
    "The language (TypeScript types):",
    language(),
  ].join("\n");
}

const parentBlock = (tag: string, p: Parent): string => `<${tag} id="${p.id}">${JSON.stringify({ unit: p.def, abilities: p.abilities, statuses: p.statuses })}</${tag}>`;

export function designPrompt(a: Parent, b: Parent): string {
  return `${parentBlock("parentA", a)}\n${parentBlock("parentB", b)}\n\nDesign three fusions of A and B: lean "A", lean "B" and "twist", in that order. For each, \`intent\` is the rule in one plain sentence.`;
}

const OBJ = { type: "object" };
export const DESIGN_SCHEMA = {
  type: "object",
  properties: {
    candidates: {
      type: "array",
      items: {
        type: "object",
        properties: {
          lean: { type: "string", enum: LEANS },
          intent: { type: "string" },
          rule: OBJ,
          abilities: OBJ,
          statuses: OBJ,
          wordRequest: { type: "object", properties: { word: { type: "string" }, meaning: { type: "string" } }, required: ["word", "meaning"] },
        },
        required: ["lean", "intent", "rule", "abilities"],
      },
    },
  },
  required: ["candidates"],
};

export const NAMER_SYSTEM =
  "You name fusions in Arena of Ideas, an auto-battler whose units are players' ideas. Give the new creature a short evocative name (one or two words, not the parents' names glued) and one line of flavour (under 90 characters), in English and in Russian. The Russian is written natively, not translated word for word.";

export function namePrompt(a: Parent, b: Parent, d: DesignDraft, taken: string[] = []): string {
  const not = taken.length ? `\nThis pair's other candidates are already named ${taken.map((n) => `"${n}"`).join(", ")}: give this one a different name.` : "";
  return `Parents: ${a.emoji ?? ""} ${a.id} and ${b.emoji ?? ""} ${b.id}.\nThe fusion (${d.lean === "twist" ? "a third creature" : `leaning to ${d.lean === "A" ? a.id : b.id}`}): ${d.intent}${not}`;
}

const PAIR = { type: "object", properties: { en: { type: "string" }, ru: { type: "string" } }, required: ["en", "ru"] };
export const NAME_SCHEMA = { type: "object", properties: { name: PAIR, flavour: PAIR }, required: ["name", "flavour"] };

export interface ClaudeDesignerOptions {
  model: string;
  namerModel: string;
  bin: string;
  timeoutMs: number;
}

export function claudeDesigner(o: ClaudeDesignerOptions): Designer {
  const system = designerSystemPrompt();
  return {
    kind: "claude",
    async design(a, b) {
      const out = (await callClaude({ model: o.model, bin: o.bin, timeoutMs: o.timeoutMs }, system, designPrompt(a, b), DESIGN_SCHEMA)) as { candidates?: unknown };
      if (!Array.isArray(out?.candidates)) throw new Error("claude -p gave no candidates");
      return (out.candidates as DesignDraft[]).filter((c) => c && LEANS.includes(c.lean) && c.rule && typeof c.rule === "object");
    },
    async name(a, b, d, taken) {
      const out = (await callClaude({ model: o.namerModel, bin: o.bin, timeoutMs: o.timeoutMs }, NAMER_SYSTEM, namePrompt(a, b, d, taken), NAME_SCHEMA)) as Naming;
      if (!out?.name?.en || !out?.name?.ru) throw new Error("claude -p gave no name");
      return out;
    },
  };
}

/** A designer that never calls a model: each candidate strikes the front enemy
 * on a different trigger. For tests and dev worlds. */
export function fakeDesigner(): Designer {
  const on = { A: { on: "TurnStart" }, B: { on: "TurnEnd" }, twist: { on: "BattleStart" } } as const;
  return {
    kind: "fake",
    async design(a, b) {
      return LEANS.map((lean) => ({
        lean,
        intent: `${a.id} and ${b.id}, lean ${lean}: hits the front enemy.`,
        rule: { triggers: [{ kind: "trigger", on: on[lean] }], selectors: [{ kind: "frontEnemy" }], abilities: ["Hit"] },
        abilities: { Hit: { name: "Hit", family: "Strike", effects: [{ kind: "damage", amount: { kind: "const", value: 2 } }] } },
      })) as DesignDraft[];
    },
    async name(a, b, d) {
      return { name: { en: `${a.id}${b.id} ${d.lean}`, ru: `${a.id}${b.id} ${d.lean}` }, flavour: { en: "A test fusion.", ru: "Пробное слияние." } };
    },
  };
}

export function designerFromEnv(env: NodeJS.ProcessEnv = process.env): Designer {
  if (env.ARENA_NIGHT_DESIGNER === "claude") {
    return claudeDesigner({
      model: env.ARENA_NIGHT_MODEL || "opus",
      namerModel: env.ARENA_NIGHT_NAMER_MODEL || "sonnet",
      bin: env.ARENA_CLAUDE_BIN || "claude",
      timeoutMs: Number(env.ARENA_NIGHT_TIMEOUT_MS) || 600_000,
    });
  }
  if (env.ARENA_NIGHT_DESIGNER && env.ARENA_NIGHT_DESIGNER !== "fake") throw new Error(`ARENA_NIGHT_DESIGNER is "claude" or "fake", not "${env.ARENA_NIGHT_DESIGNER}"`);
  return fakeDesigner();
}
