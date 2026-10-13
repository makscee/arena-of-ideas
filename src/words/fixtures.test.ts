// The §4 units and §5 fusions as fixtures (not in the pool): each battle is
// deterministic, matches its golden (goldens/fixture-<name>.jsonl;
// UPDATE_WORD_GOLDENS=1 rewrites them) and shows its trace line.

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { battle, toJSONL } from "../battle.js";
import { FIXTURES } from "./fixtures.js";
import { traceLines } from "./trace.js";

const GOLDENS = join(import.meta.dirname, "goldens");

describe("rules v2 fixtures", () => {
  for (const [name, { input, trace }] of Object.entries(FIXTURES)) {
    it(`${name}: deterministic, golden, traces "${trace}"`, () => {
      const log = battle(input);
      expect(toJSONL(battle(input))).toBe(toJSONL(log));
      const file = join(GOLDENS, `fixture-${name}.jsonl`);
      if (process.env.UPDATE_WORD_GOLDENS || !existsSync(file)) writeFileSync(file, toJSONL(log));
      expect(toJSONL(log)).toBe(readFileSync(file, "utf8"));
      expect(traceLines(input, log)).toContain(trace);
    });
  }
});
