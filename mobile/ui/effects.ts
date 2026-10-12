// The battle's effect language (mission 1007, #1011; the table on
// https://m1.twin-pogona.ts.net/r/arena-battle-research.html, section Redesign).
// Each kind of change looks, moves and sounds its own way, so it reads with
// the sound off, and without colour. Pure: battle.ts draws it, the tests pin
// that two kinds never differ in less than two of colour, motion and sound.
import type { BattleEvent } from "../../src/types";
import type { Change } from "../../src/mvp/trace";
import type { SoundKey } from "./sound-map";

export type EffectKind = "damage" | "blocked" | "heal" | "buff" | "debuff" | "status" | "death" | "summon";

/** What a kind is: its colour token, how it moves, which sound it makes. */
export interface Language {
  /** A CSS colour token (style.css :root), the same in all three palettes. */
  colour: string;
  /** damage: the number drops and the card is knocked back; blocked: a bubble pops;
   * heal: a glow rises through the card; buff / debuff: the stat digit grows / shrinks;
   * status: the icon flies along the beam into the status row; death: freezes, greys, falls;
   * summon: rises from below inside a ring. */
  motion: "knock-drop" | "bubble-pop" | "glow-rise" | "digit-grow" | "digit-shrink" | "fly-in" | "grey-fall" | "ring-rise";
  /** The sound cue a wave of this kind makes (ui/sound-map.ts waveSound; damage is hit, or zap off a Strike). */
  sound: SoundKey;
}

export const LANGUAGE: Record<EffectKind, Language> = {
  damage: { colour: "--k-dmg", motion: "knock-drop", sound: "hit" },
  blocked: { colour: "--k-shield", motion: "bubble-pop", sound: "block" },
  heal: { colour: "--k-heal", motion: "glow-rise", sound: "heal" },
  buff: { colour: "--action", motion: "digit-grow", sound: "buff" },
  debuff: { colour: "--dim", motion: "digit-shrink", sound: "debuff" },
  status: { colour: "--k-status", motion: "fly-in", sound: "status" },
  death: { colour: "--k-silence", motion: "grey-fall", sound: "death" },
  summon: { colour: "--action", motion: "ring-rise", sound: "spawn" },
};

/** One thing to draw on a card for a change. */
export interface Effect {
  kind: EffectKind;
  unit: string;
  eventId: number;
  /** The number the card shows with it: "−4", "+2", "0", a status's "×2", a summon's "new"; none on a digit or a death. */
  text: string | null;
  /** buff / debuff: which digit grows or shrinks. */
  stat?: "pwr" | "hp";
  /** status: the status put on (for its icon and colour). */
  status?: string;
  /** damage: a hit big enough to shake harder. */
  big?: boolean;
}

/** A hit's damage at or past this reads as big (trace.ts BIG_HIT_MIN, kept in step by a test). */
export const BIG_HIT = 4;

/** The effects of a wave's changes, in order. A hit is a damage effect, a
 * blocked bubble, or both (a Shield soaked part of it); a hit that did
 * nothing at all is a blocked "0". A silence and "no room" draw nothing here:
 * their chip and the strip say so. */
export function effectsOf(changes: readonly Change[], log: readonly BattleEvent[]): Effect[] {
  const out: Effect[] = [];
  for (const c of changes) {
    const e = log[c.eventId];
    if (!e) continue;
    switch (c.kind) {
      case "damage": {
        if (e.type !== "Hurt") break;
        if (e.amount > 0) out.push({ kind: "damage", unit: c.unit, eventId: c.eventId, text: `−${e.amount}`, big: e.amount >= BIG_HIT });
        if ((e.absorbed ?? 0) > 0 || e.amount === 0) out.push({ kind: "blocked", unit: c.unit, eventId: c.eventId, text: e.amount > 0 ? null : "0" });
        break;
      }
      case "heal":
        if (e.type === "Heal") out.push({ kind: "heal", unit: c.unit, eventId: c.eventId, text: `+${e.amount}` });
        break;
      case "buff":
      case "debuff":
        if (e.type === "StatChanged" && (e.stat === "pwr" || e.stat === "hp")) out.push({ kind: c.kind, unit: c.unit, eventId: c.eventId, text: null, stat: e.stat });
        break;
      case "status":
        if (e.type === "StatusApplied") out.push({ kind: "status", unit: c.unit, eventId: c.eventId, text: `×${e.stacks}`, status: e.status });
        break;
      case "death":
        out.push({ kind: "death", unit: c.unit, eventId: c.eventId, text: null });
        break;
      case "summon":
        out.push({ kind: "summon", unit: c.unit, eventId: c.eventId, text: c.label });
        break;
      default:
        break;
    }
  }
  return out;
}

/** The net change to each unit's PWR and HP stat over the events given
 * (the turn so far): the digit stays gold or grey until the turn ends. */
export function statNets(ids: Iterable<number>, log: readonly BattleEvent[]): Map<string, { pwr: number; hp: number }> {
  const nets = new Map<string, { pwr: number; hp: number }>();
  for (const id of ids) {
    const e = log[id];
    if (e?.type !== "StatChanged" || (e.stat !== "pwr" && e.stat !== "hp")) continue;
    const n = nets.get(e.unit) ?? { pwr: 0, hp: 0 };
    n[e.stat] += e.delta;
    nets.set(e.unit, n);
  }
  return nets;
}
