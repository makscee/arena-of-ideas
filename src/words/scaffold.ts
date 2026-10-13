// The word card template (#1043 §7, #1052): what a forged word is made of.
// A word is one file in this folder (how it applies, its English and Russian
// templates, its trace line, an example battle) plus its entry in index.ts and
// its kind in the type union of ../types.ts. The registry test then runs the
// example against a golden, so a word cannot land without a test and a golden.
// `npm run word:new` writes all of that from here; pure functions, no I/O.

export type WordClass = "selector" | "condition" | "amount" | "effect";

export const WORD_CLASSES: readonly WordClass[] = ["selector", "condition", "amount", "effect"];

/** Per class: the registry record in index.ts, the type in types.ts and the word interface. */
const CLASS = {
  selector: { record: "SELECTOR_WORDS", union: "PositionSelector", iface: "SelectorWord", variant: "PositionSelector", meta: "position" },
  condition: { record: "CONDITION_WORDS", union: "ConditionV2", iface: "ConditionWord", variant: "ConditionV2", meta: "condition" },
  amount: { record: "AMOUNT_WORDS", union: "AmountV2", iface: "AmountWord", variant: "AmountV2", meta: "counter" },
  effect: { record: "EFFECT_WORDS", union: "EffectV2", iface: "EffectWord", variant: "EffectV2", meta: "ruleMod" },
} as const;

export const isWordKind = (kind: string): boolean => /^[a-z][A-Za-z0-9]*$/.test(kind);

const body: Record<WordClass, (kind: string) => string> = {
  selector: (kind) => `  side: "enemy",
  select(ctx, holder) {
    // TODO: who this picks, by place in the line (ctx.line(side) is front to back)
    return [];
  },
  en: { phrase: "TODO the ${kind}", chip: "TODO", trace: "TODO the ${kind}" },
  ru: { dat: "TODO", acc: "TODO", chip: "TODO", trace: "TODO" },
  example: {
    input: example([rule("Holder", 4, 1, { on: "TurnStart" }, { kind: "${kind}" }, "Zap")], [plain("Brute", 6, 2)]),
    trace: "TODO a line traceLines(log) produces",
  },`,
  condition: (kind) => `  check(ctx, c, at) {
    // TODO: true when the rule may fire (at.holder, at.event; ctx for the battle)
    return false;
  },
  en: (c, say) => "TODO clause",
  ru: (c, say) => "TODO clause",
  example: {
    input: example([rule("Holder", 6, 1, { on: "TurnStart" }, { kind: "frontEnemy" }, "Zap", { kind: "${kind}" })], [plain("Brute", 9, 1)]),
    trace: "TODO a line traceLines(log) produces",
  },`,
  amount: (kind) => `  value(ctx, a, at) {
    // TODO: the number (at.holder, at.target, at.eventUnit; at.eval(other amount))
    return 0;
  },
  en: (a, say) => "TODO noun after 'equal to'",
  ru: (a, say) => "TODO noun, dative",
  example: {
    input: example([rule("Holder", 9, 2, { on: "TurnStart" }, { kind: "frontEnemy" }, "Hit")], [plain("Brute", 12, 1)]),
    trace: "TODO a line traceLines(log) produces",
  },`,
  effect: (kind) => `  context: "trigger",
  // an interceptor effect sets context: "interceptor" and adds intercept(ctx, e, at); a trigger
  // effect also needs a case "${kind}" in the kernel's effect switch (src/battle.ts)
  en: (e, say, target) => \`TODO clause \${target}\`,
  ru: (e, say, target) => \`TODO clause \${target}\`,
  example: {
    input: { ...example([rule("Holder", 6, 1, { on: "BattleStart" }, { kind: "frontEnemy" }, "Do")], [plain("Brute", 9, 1)]), abilities: { ...EXAMPLE_ABILITIES, Do: { name: "Do", family: "Arcane", effects: [{ kind: "${kind}" }] } } },
    trace: "TODO a line traceLines(log) produces",
  },`,
};

/** The new word's file. `doc` is one line saying what the word means. */
export function wordSource(cls: WordClass, kind: string, doc: string): string {
  const c = CLASS[cls];
  const imports = cls === "effect" ? "EXAMPLE_ABILITIES, example, plain, rule" : "example, plain, rule";
  const typeImport = cls === "selector" ? "" : `import type { ${c.variant} } from "../types.js";\n`;
  const alias = cls === "selector" ? "" : `\ntype T = Extract<${c.variant}, { kind: "${kind}" }>;\n`;
  const generic = cls === "selector" ? "" : "<T>";
  return `import { ${imports} } from "./examples.js";
import type { ${c.iface} } from "./types.js";
${typeImport}${alias}
/** ${doc} */
export const ${kind}: ${c.iface}${generic} = {
  kind: "${kind}",
  class: "${c.meta}",
${body[cls](kind)}
};
`;
}

/** index.ts with the word imported and entered in its record. */
export function registerInIndex(index: string, cls: WordClass, kind: string): string {
  const c = CLASS[cls];
  if (new RegExp(`import \\{ ${kind} \\} from`).test(index)) throw new Error(`word "${kind}" is already registered`);
  const lastImport = [...index.matchAll(/^import .* from "\.\/[A-Za-z0-9]+\.js";$/gm)].pop();
  if (!lastImport) throw new Error("index.ts: no word imports found");
  const at = lastImport.index! + lastImport[0].length;
  let out = `${index.slice(0, at)}\nimport { ${kind} } from "./${kind}.js";${index.slice(at)}`;
  const rec = new RegExp(`(export const ${c.record}[^=]*= \\{[\\s\\S]*?)(\\n\\};)`);
  if (!rec.test(out)) throw new Error(`index.ts: record ${c.record} not found`);
  out = out.replace(rec, (_m, head: string, tail: string) => `${head.replace(/,?\s*$/, ",")} ${kind},${tail}`);
  return out;
}

/** types.ts with `{ kind: "<kind>" }` added to the class's union. */
export function addToUnion(types: string, cls: WordClass, kind: string): string {
  const c = CLASS[cls];
  const decl = types.indexOf(`export type ${c.union} =`);
  if (decl < 0) throw new Error(`types.ts: ${c.union} not found`);
  if (types.includes(`{ kind: "${kind}"`)) throw new Error(`types.ts: kind "${kind}" is already used`);
  // the first member line that ends the union with ";"
  const lines = types.slice(decl).split("\n");
  const i = lines.findIndex((l) => /^\s*\|.*;( *\/\/.*)?$/.test(l));
  if (i < 0) throw new Error(`types.ts: end of ${c.union} not found`);
  const m = /^(\s*\|.*?)(;)( *\/\/.*)?$/.exec(lines[i]!)!;
  lines[i] = `${m[1]}${m[3] ?? ""}`;
  lines.splice(i + 1, 0, `  | { kind: "${kind}" };`);
  return types.slice(0, decl) + lines.join("\n");
}
