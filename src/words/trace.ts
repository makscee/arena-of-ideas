// Trace lines for v2 rules (#1043 §3): "Hourglass → the ally behind it
// (Archer)", "Gambler flips: heads". One line per firing of a unit's own
// rule, read off the log and the battle input; pure, no state.

import { describeCondition, describeSelector } from "../describe.js";
import type { BattleEvent, BattleInput, UnitDef } from "../types.js";
import { selectorWord } from "./index.js";

/** The unit a roster id ("A2:Archer") was built from, or undefined for a summon. */
function defOf(input: BattleInput, id: string): UnitDef | undefined {
  const m = /^([AB])(\d+):/.exec(id);
  if (!m) return undefined;
  return (m[1] === "A" ? input.teamA : input.teamB)[Number(m[2]) - 1];
}

export function traceLines(input: BattleInput, log: readonly BattleEvent[]): string[] {
  const name = (id: string) => id.replace(/^[AB]\d+:/, "");
  const out: string[] = [];
  const firings = new Map<string, { line: number; holder: string; def: UnitDef; targets: string[]; did: string[] }>();
  for (const e of log) {
    if (e.type === "Flipped") {
      out.push(`${name(e.unit)} flips: ${e.heads ? "heads" : "tails"}`);
      continue;
    }
    if (e.source === "kernel" || e.source.status !== undefined || !("unit" in e) || typeof e.unit !== "string") continue;
    const def = defOf(input, e.source.unit);
    if (!def?.selectors) continue;
    const key = `${e.source.unit}|${e.causedBy}`;
    let f = firings.get(key);
    if (!f) {
      f = { line: out.length, holder: e.source.unit, def, targets: [], did: [] };
      firings.set(key, f);
      out.push("");
    }
    if (!f.targets.includes(e.unit)) f.targets.push(e.unit);
    const did = e.type === "Hurt" ? `${e.amount} damage` : e.type === "Heal" ? `heal ${e.amount}` : e.type === "StatusApplied" ? `${e.status} ${e.stacks}` : undefined;
    if (did !== undefined && !f.did.includes(did)) f.did.push(did);
  }
  for (const f of firings.values()) {
    const who = f.def.selectors!.map((s) => selectorWord(s.kind)?.en.trace ?? describeSelector(s)).join(" and ");
    const cond = f.def.condition ? ` (${describeCondition(f.def.condition)})` : "";
    out[f.line] = `${name(f.holder)}${cond} → ${who} (${f.targets.map(name).join(", ")})${f.did.length ? ` → ${f.did.join(", ")}` : ""}`;
  }
  return out;
}
