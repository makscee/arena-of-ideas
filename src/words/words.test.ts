// Every word in the registry (#1043 §3, §7) brings an example battle, its
// golden log (goldens/<kind>.jsonl) and a trace line the battle must show.
// Regenerate a golden on purpose with UPDATE_WORD_GOLDENS=1.

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { battle, toJSONL } from "../battle.js";
import { describeAbilitySegments, describeCondition } from "../describe.js";
import { describeAbilitySegmentsRu } from "../describe-ru.js";
import { unitActionsOf } from "../types.js";
import { assertValidContent } from "../validate.js";
import { ALL_WORDS } from "./index.js";
import { traceLines } from "./trace.js";

const GOLDENS = join(import.meta.dirname, "goldens");

describe("word registry", () => {
  for (const word of ALL_WORDS) {
    describe(word.kind, () => {
      const { input, trace } = word.example;
      it("its example is legal content", () => {
        assertValidContent(input.teamA, input.statuses!, input.abilities!, "teamA");
        assertValidContent(input.teamB, input.statuses!, input.abilities!, "teamB");
      });
      it("says its example in English and Russian", () => {
        const unit = [...input.teamA, ...input.teamB].find((u) => JSON.stringify(u).includes(`"kind":"${word.kind}"`))!;
        const action = unitActionsOf(unit, input.abilities!)[0]!;
        const en = describeAbilitySegments(action).map((s) => s.text).join("");
        const ru = describeAbilitySegmentsRu(action, {}).map((s) => s.text).join("");
        expect(en).not.toMatch(/undefined/);
        expect(ru).not.toMatch(/undefined/);
        expect(ru).toMatch(/[а-яё]/i);
      });
      it("is deterministic and matches its golden", () => {
        const log = battle(input);
        expect(toJSONL(battle(input))).toBe(toJSONL(log));
        const file = join(GOLDENS, `${word.kind}.jsonl`);
        if (process.env.UPDATE_WORD_GOLDENS || !existsSync(file)) writeFileSync(file, toJSONL(log));
        expect(toJSONL(log)).toBe(readFileSync(file, "utf8"));
      });
      it(`traces: ${trace}`, () => {
        expect(traceLines(input, battle(input))).toContain(trace);
      });
    });
  }
});

describe("conditions", () => {
  it("nest with and / not", () => {
    expect(describeCondition({ kind: "and", all: [{ kind: "turnAtLeast", n: 2 }, { kind: "not", cond: { kind: "onceHolder" } }] })).toBe("from turn 2 and unless once per battle");
  });
});
