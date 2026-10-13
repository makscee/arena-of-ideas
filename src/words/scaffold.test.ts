import { readFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { ALL_WORDS } from "./index.js";
import { WORD_CLASSES, addToUnion, isWordKind, registerInIndex, wordSource } from "./scaffold.js";

const src = (f: string) => readFileSync(join(import.meta.dirname, f), "utf8");

describe("the word card scaffold (#1052)", () => {
  it("writes a file with every part a word needs, for each class", () => {
    for (const cls of WORD_CLASSES) {
      const s = wordSource(cls, "probe", "Does a probe.");
      expect(s).toContain('kind: "probe"');
      expect(s).toContain("/** Does a probe. */");
      expect(s).toMatch(/example: \{/);
      expect(s).toMatch(/trace: "TODO/);
      expect(s).toMatch(/\ben[:(]/);
      expect(s).toMatch(/\bru[:(]/);
    }
  });

  it("registers a word in the index and the type union, once", () => {
    for (const [cls, record, union] of [["selector", "SELECTOR_WORDS", "PositionSelector"], ["condition", "CONDITION_WORDS", "ConditionV2"], ["amount", "AMOUNT_WORDS", "AmountV2"], ["effect", "EFFECT_WORDS", "EffectV2"]] as const) {
      const index = registerInIndex(src("index.ts"), cls, "probe");
      expect(index).toContain('import { probe } from "./probe.js";');
      const rec = new RegExp(`export const ${record}[^=]*= \\{[^}]*\\bprobe,\\n\\};`).test(index);
      expect(rec, record).toBe(true);
      expect(() => registerInIndex(index, cls, "probe")).toThrow(/already registered/);
      const types = addToUnion(src("../types.ts"), cls, "probe");
      const decl = types.slice(types.indexOf(`export type ${union} =`));
      expect(decl.slice(0, decl.indexOf("\n\n"))).toMatch(/\| \{ kind: "probe" \};$/);
      expect(() => addToUnion(types, cls, "probe")).toThrow(/already used/);
    }
  });

  it("lets no registered word keep a TODO from the scaffold", () => {
    // the scaffold's placeholders say TODO; the registry test alone would pass
    // its sentences and write a missing golden, so a word with one never lands
    for (const w of ALL_WORDS) expect(src(`${w.kind}.ts`), `src/words/${w.kind}.ts`).not.toMatch(/TODO/);
  });

  it("takes a kind that is a camelCase name", () => {
    expect(isWordKind("steal")).toBe(true);
    expect(isWordKind("hitsTaken")).toBe(true);
    expect(isWordKind("Steal gold")).toBe(false);
  });
});
