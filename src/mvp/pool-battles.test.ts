// Goldens for the rules v2 pool (#1046): every unit's pool battle (the one
// the report page and the lab's `#scene=v2-unit-<slug>[-awoken]` show),
// sleeping and awoken, is deterministic and matches its golden
// (goldens/pool-<slug>[-awoken].jsonl; UPDATE_WORD_GOLDENS=1 rewrites them),
// and a trace line names the unit. The tuned numbers (units-v2-numbers.ts)
// move these battles: rewrite the goldens after `npm run mvp:tune-v2`.

import { existsSync, readFileSync, writeFileSync } from "node:fs";
import { join } from "node:path";
import { describe, expect, it } from "vitest";
import { contentOf } from "../../server/src/mvp/content.js";
import { toJSONL } from "../battle.js";
import { poolBattle, poolBattleId } from "./pool-battles.js";
import { ROWS_V2 } from "./units-v2.js";

const GOLDENS = join(import.meta.dirname, "..", "words", "goldens");
const content = contentOf(ROWS_V2);
const player = { id: "golden", name: "golden", bot: true };
const AT = "2026-10-13T00:00:00.000Z";

describe("rules v2 pool battles", () => {
  for (const row of ROWS_V2)
    for (const awoken of [false, true]) {
      const id = poolBattleId(row.name, awoken).replace(/^v2-unit-/, "pool-");
      it(`${row.name}${awoken ? " awoken" : ""}: deterministic, golden, names the unit`, () => {
        const a = poolBattle(content, row.name, awoken, player, AT);
        expect(toJSONL(poolBattle(content, row.name, awoken, player, AT).record.log)).toBe(toJSONL(a.record.log));
        const file = join(GOLDENS, `${id}.jsonl`);
        if (process.env.UPDATE_WORD_GOLDENS || !existsSync(file)) writeFileSync(file, toJSONL(a.record.log));
        expect(toJSONL(a.record.log)).toBe(readFileSync(file, "utf8"));
        expect(a.own.length).toBeGreaterThan(0);
      });
    }
});
