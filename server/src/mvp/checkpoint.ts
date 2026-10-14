// Checkpoints (makscee/void-board#1065): a link that opens a world at the
// moment before a discovery. POST /dev/checkpoint { a, b } puts the caller's
// run (their active one, or a new one) in the shop with an Awoken `a` and an
// Awoken `b` on the line, so one Fuse is left. Dev only (an admin invite on
// the workshop); the client opens it from `#checkpoint=a+b` (mobile/main.ts).
// The pair's Codex state is left as it is: a ready pair opens the forge's
// three sealed designs, a discovered one fuses into its canonical design.
import type { PlayerRef, UnitId } from "../../../src/mvp/contract.js";
import { lineUnitOf } from "../../../src/mvp/forms.js";
import type { MvpRunState } from "../../../src/mvp/run.js";
import { startRun } from "./runs.js";
import type { MvpRuntime } from "./runtime.js";

/** The caller's run at the checkpoint of pair a+b; undefined unless a and b are two live units. */
export function checkpointRun(rt: MvpRuntime, player: PlayerRef, a: UnitId, b: UnitId): MvpRunState | undefined {
  const ua = rt.content.units.find((u) => u.id === a);
  const ub = rt.content.units.find((u) => u.id === b);
  if (!ua || !ub || a === b) return undefined;
  const run = startRun(rt, player);
  const n = rt.rules.copiesToAwaken;
  const s: MvpRunState = {
    ...run,
    phase: "shop",
    gold: Math.max(run.gold, 10),
    bench: [],
    line: [lineUnitOf(ua, `${run.runId}-cp-${a}`, n, rt.rules), lineUnitOf(ub, `${run.runId}-cp-${b}`, n, rt.rules)],
  };
  delete s.gift;
  rt.store.putRun(s);
  return s;
}
