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
  const name = (id: string) => id.replace(/^[AB]\+?\d+:/, "");
  const out: string[] = [];
  const firings = new Map<string, { line: number; holder: string; def: UnitDef; targets: string[]; did: string[] }>();
  for (const e of log) {
    if (e.type === "Flipped") {
      out.push(`${name(e.unit)}: 50% → ${e.heads ? "double" : "nothing"}`); // #1063: the chance, never a coin
      continue;
    }
    if (e.type === "Redirected") {
      const what = e.original === "Hurt" ? "the hit" : e.original === "Strike" ? "the strike" : e.original === "StatusApplied" ? "the status" : "the heal";
      out.push(`${name(e.by.unit)} intercepts: ${what} on ${name(e.from)} lands on ${name(e.to)}`);
      continue;
    }
    if (e.type === "Delayed") {
      out.push(`${name(e.by.unit)} sets a delay on ${name(e.unit)}: lands at the end of turn ${e.at}`);
      continue;
    }
    if (e.type === "Transformed") {
      out.push(`${name(e.unit)} becomes ${e.name} (${e.pwr}/${e.hp})`);
      continue;
    }
    if (e.type === "Summon" && e.source !== "kernel" && e.source.status === undefined) {
      const fallen = log.find((x) => x.id === e.causedBy);
      if (fallen?.type === "Death" && defOf(input, e.source.unit)?.abilities?.some((id) => input.abilities?.[id]?.effects.some((x) => x.kind === "summonWithRule"))) {
        out.push(`${name(e.source.unit)} grows ${e.name} with the rule of ${name(fallen.unit)}`);
        continue;
      }
    }
    if (e.type === "RuleMod") {
      const o = name(e.other ?? "");
      const said = { copyRule: `${name(e.unit)} copies the rule of ${o}`, takeRule: `${name(e.unit)} takes the rule of ${o}`, actAgain: `${name(e.unit)}: ${o} acts again`, remember: `${name(e.unit)} remembers the rule of ${o}`, performRemembered: `${name(e.unit)} performs the rule of ${o}`, swapRules: `${name(e.unit)} trades rules with ${o}`, keepRule: `${name(e.unit)} keeps the rule of ${o}` };
      out.push(said[e.op]);
      continue;
    }
    if (e.type === "Strike" && e.again) out.push(`${name(e.striker)} strikes again`);
    if (e.type === "Strike" && e.amount !== undefined) {
      out.push(`${name(e.striker)}'s strike on ${name(e.defender)} hits for ${e.amount}`);
      continue;
    }
    if (e.type === "Intercepted" && e.original === "Death") {
      const paid = log.find((x) => x.causedBy === e.id && x.type === "StatChanged");
      const cost = paid?.type === "StatChanged" ? `, loses ${-paid.delta} ${paid.stat.toUpperCase()}` : "";
      const healed = log.find((x) => x.causedBy === e.id && x.type === "Heal");
      out.push(`${name(e.by.unit)} intercepts: ${name(e.unit ?? e.by.unit)} survives${healed?.type === "Heal" ? ` at ${healed.hpAfter} HP` : ""}${cost}`);
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
