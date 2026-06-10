// Board projection — the visible board state after the first N events of a log.
// A pure fold over the events the kernel already stamped: hp comes off
// `hpAfter`, stack counts off `total`/`remaining`, stat maxima off
// `StatChanged.now`. No rules are re-derived here — a viewer that steps a
// battle reads this projection instead of re-implementing bookkeeping.

import type { BattleEvent, Side } from "./types.js";

export interface BoardUnit {
  id: string;
  name: string;
  /** Current hp, clamped at 0 for display — `hpAfter` may be negative on overkill. */
  hp: number;
  /** Effective max hp (follows StatChanged). */
  maxHp: number;
  /** Effective pwr (follows StatChanged). */
  pwr: number;
  /** Attached statuses in attach order. */
  statuses: { status: string; stacks: number }[];
  silenced: boolean;
}

export interface BoardState {
  /** Turn of the last applied event (0 during the BattleStart cascade). */
  turn: number;
  /** Living units per side, front (index 0) first. */
  lines: { A: BoardUnit[]; B: BoardUnit[] };
  /** Dead units per side, in death order. */
  graves: { A: BoardUnit[]; B: BoardUnit[] };
  /** Present once BattleEnd has applied. */
  ended?: { winner: Side | "draw"; turns: number };
}

interface UnitProj extends BoardUnit {
  side: Side;
  baseHp: number; // roster stats; the corpse is clean, so death resets to these
  basePwr: number;
}

/**
 * Project the board after events `0..upto` (inclusive) have applied.
 * `upto` past the end of the log projects the final board.
 */
export function boardAt(log: BattleEvent[], upto: number): BoardState {
  const units = new Map<string, UnitProj>();
  const state: BoardState = { turn: 0, lines: { A: [], B: [] }, graves: { A: [], B: [] } };

  const add = (id: string, name: string, side: Side, hp: number, pwr: number): UnitProj => {
    const u: UnitProj = { id, name, side, hp, maxHp: hp, pwr, baseHp: hp, basePwr: pwr, statuses: [], silenced: false };
    units.set(id, u);
    return u;
  };
  const removeFrom = (list: BoardUnit[], id: string) => {
    const i = list.findIndex((u) => u.id === id);
    if (i >= 0) list.splice(i, 1);
  };

  const last = Math.min(upto, log.length - 1);
  for (let i = 0; i <= last; i++) {
    const e = log[i];
    if (!e) continue;
    state.turn = e.turn;
    switch (e.type) {
      case "BattleStart":
        for (const side of ["A", "B"] as const) {
          for (const r of e.teams[side]) state.lines[side].push(add(r.id, r.name, side, r.hp, r.pwr));
        }
        break;

      case "Hurt":
      case "Heal": {
        const u = units.get(e.unit);
        if (u && e.hpAfter !== undefined) u.hp = Math.max(0, e.hpAfter);
        break;
      }

      case "Death": {
        const u = units.get(e.unit);
        if (!u) break;
        removeFrom(state.lines[u.side], u.id);
        // The corpse is clean (SPEC §4): statuses end at death, contributions vanish.
        u.statuses = [];
        u.silenced = false;
        u.hp = 0;
        u.maxHp = u.baseHp;
        u.pwr = u.basePwr;
        state.graves[u.side].push(u);
        break;
      }

      case "Summon": {
        if (e.resurrected) {
          const u = units.get(e.unit);
          if (!u) break;
          removeFrom(state.graves[u.side], u.id);
          u.hp = Math.min(e.atHp ?? 1, u.maxHp); // kernel caps revival hp at effective max
          state.lines[u.side].push(u);
        } else {
          state.lines[e.side].push(add(e.unit, e.name, e.side, e.hp, e.pwr));
        }
        break;
      }

      case "StatusApplied": {
        const u = units.get(e.unit);
        if (!u) break;
        const inst = u.statuses.find((s) => s.status === e.status);
        if (inst) inst.stacks = e.total;
        else u.statuses.push({ status: e.status, stacks: e.total });
        break;
      }

      case "StatusRemoved": {
        const u = units.get(e.unit);
        if (!u) break;
        if (e.remaining > 0) {
          const inst = u.statuses.find((s) => s.status === e.status);
          if (inst) inst.stacks = e.remaining;
        } else {
          u.statuses = u.statuses.filter((s) => s.status !== e.status);
        }
        break;
      }

      case "StatChanged": {
        const u = units.get(e.unit);
        if (!u) break;
        if (e.stat === "hp") {
          if (e.hpAfter !== undefined) u.hp = Math.max(0, e.hpAfter);
          u.maxHp = e.now;
        } else {
          u.pwr = e.now;
        }
        break;
      }

      case "Silenced": {
        const u = units.get(e.unit);
        if (u) u.silenced = true;
        break;
      }

      case "BattleEnd":
        state.ended = { winner: e.winner, turns: e.turns };
        break;

      default:
        break; // turn structure, strikes, traces — no board mutation of their own
    }
  }
  return state;
}
