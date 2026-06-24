// Dev cheats (#066 slice 4) — the in-run mutations the DEV panel applies, as a
// pure module over RunState. DOM-free on purpose, like sweep.ts is UI-free: the
// panel wires clicks to these, and the vitest suite drives them directly.
//
// They follow run.ts's transition convention exactly — return a fresh RunState,
// never mutate the input (a transition that mutated its argument would desync
// every state derived from the old one). The shapes they produce are VALID
// RunStates: spawnUnit grows the line the way buy() does (the drafted def with a
// grown base and level on it), respects TEAM_SIZE, and drops into the shop as a
// plain offer.
//
// These are NOT a security boundary. The arena is client-authoritative: a run
// touched here no longer re-derives from its seed + decision log, so the server
// REJECTS it on submission (proven in server/src/ladder-api.test.ts). The DEV
// panel marks such a run local-only only to skip a submission it knows is
// doomed — the guarantee is the re-derivation, not the flag.

import { TEAM_SIZE, type RunState, type RunUnit, type UnitDef } from "../src/index.js";

/** Clone the layers a dev mutation may touch, mirroring run.ts's clone() so a
 * cheat never mutates its input (and the result stays an independent value). */
function clone(s: RunState): RunState {
  return {
    ...s,
    team: s.team.map((u) => ({ ...u, base: { ...u.base } })),
    offers: [...s.offers],
    log: [...s.log],
  };
}

/** Add `n` gold (negative subtracts). Gold floors at 0 — a cheat never leaves
 * the run in a negative-gold shape no honest transition could produce. */
export function addGold(state: RunState, n: number): RunState {
  return setGold(state, state.gold + n);
}

/** Set gold to exactly `n`, floored at 0 (and rounded down — gold is whole). */
export function setGold(state: RunState, n: number): RunState {
  const s = clone(state);
  s.gold = Math.max(0, Math.floor(n));
  return s;
}

export type SpawnDest = "shop" | "team";

/** Drop a unit into the run: as a shop offer (buyable for free-spent gold), or
 * straight onto the line. A team spawn grows the line the way buy() does — the
 * drafted def with a fresh grown base and level — so the result is a unit no
 * different in shape from a bought one. A full line (TEAM_SIZE) is left
 * untouched for a team spawn: the cheat is a no-op rather than an over-full,
 * invalid line. The def is taken as given (the palette already deep-clones its
 * picks), but team insertion copies its base so a later override never aliases
 * the source def. */
export function spawnUnit(state: RunState, unit: UnitDef, dest: SpawnDest): RunState {
  const s = clone(state);
  if (dest === "shop") {
    s.offers = [...s.offers, unit];
    return s;
  }
  if (s.team.length >= TEAM_SIZE) return s; // line full — no-op, never an invalid over-full line
  const grown: RunUnit = { name: unit.name, base: { ...unit.base }, level: unit.level ?? 1, stacks: 1, def: unit };
  s.team = [...s.team, grown];
  return s;
}
