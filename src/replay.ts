// Text replay renderer — a pure function over the causal event log.
// The bar: a reader who has never seen the code can tell WHY the battle went
// the way it did. Every consequential line carries its cause chain, walked
// straight off the `causedBy` links the log records (SPEC §0.3).
//
// Deterministic: same log in, byte-identical text out. No clock, no randomness.

import type { AbilityRef, BattleEvent } from "./types.js";

export function renderReplay(log: BattleEvent[]): string {
  return new Renderer(log).render();
}

/** Per-unit view state the renderer tracks to show readable hp counters.
 * Current hp comes off the events themselves (`hpAfter`); only the max needs tracking. */
interface UnitView {
  baseHp: number;
  maxHp: number; // effective max hp (follows StatChanged)
}

class Renderer {
  private readonly log: BattleEvent[];
  private readonly lines: string[] = [];
  private readonly units = new Map<string, UnitView>();
  private readonly display = new Map<string, string>(); // unit id → display name
  private readonly depth: number[] = []; // event id → causal depth
  private readonly mergedHurts = new Set<number>(); // Hurts folded into their Strike line

  constructor(log: BattleEvent[]) {
    this.log = log;
    this.computeDisplayNames();
    for (const e of log) {
      this.depth[e.id] = e.causedBy === null ? 0 : (this.depth[e.causedBy] ?? 0) + 1;
    }
  }

  render(): string {
    for (const e of this.log) this.renderEvent(e);
    return this.lines.join("\n") + "\n";
  }

  // ---------- naming ----------

  /** Unit names when unique, instance ids (already readable, e.g. "A1:Dummy") when not. */
  private computeDisplayNames(): void {
    const owners = new Map<string, Set<string>>(); // name → unit ids
    const claim = (id: string, name: string) => {
      if (!owners.has(name)) owners.set(name, new Set());
      owners.get(name)!.add(id);
    };
    for (const e of this.log) {
      if (e.type === "BattleStart") {
        for (const side of ["A", "B"] as const) for (const r of e.teams[side]) claim(r.id, r.name);
      } else if (e.type === "Summon") {
        claim(e.unit, e.name);
      }
    }
    for (const [name, ids] of owners) {
      for (const id of ids) this.display.set(id, ids.size === 1 ? name : id);
    }
  }

  private name(id: string): string {
    return this.display.get(id) ?? id;
  }

  /** "Poison on Orc" for status abilities, "Witch's ability" for unit abilities. */
  private refDesc(ref: AbilityRef): string {
    return ref.status !== undefined ? `${ref.status} on ${this.name(ref.unit)}` : `${this.name(ref.unit)}'s ability`;
  }

  // ---------- hp bookkeeping ----------

  private hpText(id: string, cur: number | undefined): string {
    const u = this.units.get(id);
    if (!u || cur === undefined) return "";
    return `${Math.max(0, cur)}/${u.maxHp} hp`;
  }

  // ---------- emission ----------

  private push(ev: BattleEvent, text: string): void {
    const d = this.depth[ev.id] ?? 0;
    const indent = 2 * Math.max(0, Math.min(d, 5) - 1);
    this.lines.push(" ".repeat(indent) + text);
  }

  private renderEvent(e: BattleEvent): void {
    switch (e.type) {
      case "BattleStart": {
        this.lines.push("=== BATTLE ===");
        for (const side of ["A", "B"] as const) {
          const roster = e.teams[side].map((r) => {
            this.units.set(r.id, { baseHp: r.hp, maxHp: r.hp });
            return `${this.name(r.id)} (${r.hp} hp, ${r.pwr} pwr)`;
          });
          this.lines.push(`Side ${side}: ${roster.join(", ")}`);
        }
        return;
      }

      case "TurnStart":
        this.lines.push("");
        this.lines.push(`--- Turn ${e.turn} ---`);
        return;

      case "TurnEnd":
        return; // structure only; its consequences (poison ticks, fatigue) speak for themselves

      case "PairFaced":
        this.push(e, `${this.name(e.a)} faces ${this.name(e.b)} — the coin says ${this.name(e.first)} strikes first.`);
        return;

      case "Strike": {
        // Fold the kernel Hurt this strike produces into one readable line.
        const hurt = this.log.find(
          (h) => h.id > e.id && h.type === "Hurt" && h.causedBy === e.id && h.source === "kernel",
        ) as Extract<BattleEvent, { type: "Hurt" }> | undefined;
        const striker = this.name(e.striker);
        const defender = this.name(e.defender);
        if (!hurt) {
          this.push(e, `${striker} strikes ${defender}.`);
          return;
        }
        this.mergedHurts.add(hurt.id);
        const after = this.hpText(e.defender, hurt.hpAfter);
        if (hurt.amount === 0 && hurt.absorbed !== undefined) {
          this.push(e, `${striker} strikes ${defender} — Shield absorbs all ${hurt.absorbed}, no harm done (${after}).`);
        } else if (hurt.amount === 0) {
          this.push(e, `${striker} strikes ${defender} for 0 — too weak to harm (${after}).`);
        } else if (hurt.absorbed !== undefined) {
          this.push(e, `${striker} strikes ${defender} for ${hurt.amount} (${hurt.absorbed} absorbed by Shield) — ${defender} at ${after}.`);
        } else {
          this.push(e, `${striker} strikes ${defender} for ${hurt.amount} — ${defender} at ${after}.`);
        }
        return;
      }

      case "Hurt": {
        if (this.mergedHurts.has(e.id)) return; // already told as part of its Strike line
        const who = this.name(e.unit);
        const at = this.hpText(e.unit, e.hpAfter);
        const parent = e.causedBy !== null ? this.log[e.causedBy] : undefined;
        const absorbNote = e.absorbed !== undefined ? ` (${e.absorbed} absorbed by Shield)` : "";
        if (parent?.type === "Fatigue") {
          this.push(e, `${who} takes ${e.amount} fatigue damage${absorbNote} — at ${at}.`);
        } else if (e.source !== "kernel" && e.source.status !== undefined) {
          this.push(e, `${e.source.status} ticks on ${who} for ${e.amount}${absorbNote} — at ${at}.`);
        } else if (e.source !== "kernel") {
          this.push(e, `${this.name(e.source.unit)}'s ability hits ${who} for ${e.amount}${absorbNote} — at ${at}.`);
        } else {
          this.push(e, `${who} takes ${e.amount} damage${absorbNote} — at ${at}.`);
        }
        return;
      }

      case "Heal": {
        const by = e.source !== "kernel" ? ` (${this.refDesc(e.source)})` : "";
        this.push(e, `${this.name(e.unit)} heals ${e.amount}${by} — back to ${this.hpText(e.unit, e.hpAfter)}.`);
        return;
      }

      case "Death": {
        const chain = e.causedBy !== null ? this.causeChain(e.causedBy) : [];
        const tail = chain.length > 0 ? ` ← ${chain.join(" ← ")}` : "";
        this.push(e, `${this.name(e.unit)} dies${tail}`);
        return;
      }

      case "Summon": {
        const by = e.source !== "kernel" ? this.name(e.source.unit) : "the kernel";
        if (e.resurrected) {
          const u = this.units.get(e.unit);
          if (u) u.maxHp = u.baseHp; // the corpse was clean: stat-modding statuses ended at death
          const at = e.atHp !== undefined ? ` at ${e.atHp} hp` : "";
          this.push(e, `${this.name(e.unit)} rises from the grave${at}, back of side ${e.side} — ${by}'s doing.`);
        } else {
          this.units.set(e.unit, { baseHp: e.hp, maxHp: e.hp });
          this.push(e, `${by} summons ${this.name(e.unit)} (${e.hp} hp, ${e.pwr} pwr) to the back of side ${e.side}.`);
        }
        return;
      }

      case "StatusApplied": {
        const by =
          e.source !== "kernel" ? ` — applied by ${this.name(e.source.unit)}` : e.turn === 0 ? " — starting status" : "";
        this.push(e, `${this.name(e.unit)} gains ${e.status} x${e.stacks} (total ${e.total})${by}.`);
        return;
      }

      case "StatusRemoved": {
        const who = this.name(e.unit);
        const stripped = e.source !== "kernel" && e.source.unit !== e.unit ? ` — stripped by ${this.name(e.source.unit)}` : "";
        if (e.remaining > 0) this.push(e, `${who}'s ${e.status} drops by ${e.stacks} to ${e.remaining}${stripped}.`);
        else this.push(e, `${who}'s ${e.status} is spent${stripped}.`);
        return;
      }

      case "StatChanged": {
        const u = this.units.get(e.unit);
        if (u && e.stat === "hp") u.maxHp = e.now;
        const dir = e.delta >= 0 ? `+${e.delta}` : `${e.delta}`;
        this.push(e, `${this.name(e.unit)}'s ${e.stat} ${dir} → now ${e.now}.`);
        return;
      }

      case "Silenced": {
        const by = e.source !== "kernel" ? ` by ${this.name(e.source.unit)}` : "";
        this.push(e, `${this.name(e.unit)} is silenced${by} — its abilities are dead for the rest of the battle.`);
        return;
      }

      case "Fatigue":
        this.push(e, `Fatigue ${e.amount}: the drawn-out battle wears everyone down.`);
        return;

      case "ChainBlocked": {
        const what = this.refDesc(e.ability);
        const after = this.shortDesc(e.at >= 0 ? this.log[e.at] : undefined);
        this.push(
          e,
          `(chain stopped: ${what} stayed quiet after ${after} — it already acted in this chain, and an ability never triggers itself)`,
        );
        return;
      }

      case "Intercepted": {
        const by = this.refDesc(e.by);
        const who = e.unit !== undefined ? this.name(e.unit) : undefined;
        if (e.original === "Strike" && who !== undefined) {
          this.push(e, `${who} tries to strike, but ${by} cancels it — the blow never lands.`);
        } else if (e.original === "Death" && who !== undefined) {
          this.push(e, `${who} should die, but ${by} refuses the death.`);
        } else if (e.original === "Hurt" && who !== undefined) {
          this.push(e, `${by} cancels the hit on ${who}.`);
        } else {
          this.push(e, `${by} cancels a ${e.original}${who !== undefined ? ` on ${who}` : ""}.`);
        }
        return;
      }

      case "BattleEnd": {
        this.lines.push("");
        const verdict = e.winner === "draw" ? "Draw" : `Side ${e.winner} wins`;
        this.lines.push(`=== ${verdict} after ${e.turns} ${e.turns === 1 ? "turn" : "turns"} ===`);
        return;
      }
    }
  }

  // ---------- cause chains ----------

  /**
   * Walk `causedBy` ancestry from a death's proximate cause and narrate it
   * compactly: "Poison tick (3 dmg) ← Poison applied turn 4 by Witch".
   */
  private causeChain(startId: number): string[] {
    const parts: string[] = [];
    let id: number | null = startId;
    for (let hops = 0; id !== null && hops < 6; hops++) {
      const e: BattleEvent | undefined = this.log[id];
      if (!e) break;
      switch (e.type) {
        case "Hurt": {
          if (e.source !== "kernel" && e.source.status !== undefined) {
            parts.push(`${e.source.status} tick (${e.amount} dmg)`);
            const origin = this.statusOrigin(e.unit, e.source.status, e.id);
            if (origin) parts.push(origin);
            return parts;
          }
          if (e.source !== "kernel") {
            parts.push(`hit by ${this.name(e.source.unit)}'s ability (${e.amount} dmg)`);
            return parts;
          }
          const parent = e.causedBy !== null ? this.log[e.causedBy] : undefined;
          if (parent?.type === "Strike") {
            parts.push(`struck by ${this.name(parent.striker)} for ${e.amount}`);
            return parts;
          }
          if (parent?.type === "Fatigue") {
            parts.push(`fatigue (${e.amount} dmg)`);
            return parts;
          }
          parts.push(`${e.amount} damage`);
          id = e.causedBy;
          continue;
        }
        case "StatChanged":
          parts.push(`max ${e.stat} ${e.delta >= 0 ? "rose" : "fell"} to ${e.now}`);
          id = e.causedBy;
          continue;
        case "StatusRemoved": {
          const by = e.source !== "kernel" && e.source.unit !== e.unit ? ` by ${this.name(e.source.unit)}` : "";
          parts.push(`${e.status} stripped${by}`);
          id = e.causedBy;
          continue;
        }
        case "Heal":
          parts.push(`after healing ${e.amount}`);
          id = e.causedBy;
          continue;
        default:
          return parts; // turn structure and the rest add no story to a death
      }
    }
    return parts;
  }

  /** Where did this status come from? The most recent application before the given event. */
  private statusOrigin(unitId: string, status: string, beforeId: number): string | undefined {
    for (let i = beforeId - 1; i >= 0; i--) {
      const e = this.log[i];
      if (e && e.type === "StatusApplied" && e.unit === unitId && e.status === status) {
        if (e.turn === 0) return `${status} carried from the start`;
        const by = e.source !== "kernel" ? ` by ${this.name(e.source.unit)}` : "";
        return `${status} applied turn ${e.turn}${by}`;
      }
    }
    return undefined;
  }

  /** One-clause description of an event, for ChainBlocked explanations. */
  private shortDesc(e: BattleEvent | undefined): string {
    if (!e) return "an earlier event";
    switch (e.type) {
      case "Hurt":
        return `${this.name(e.unit)} was hurt`;
      case "Heal":
        return `${this.name(e.unit)} healed`;
      case "Death":
        return `${this.name(e.unit)} died`;
      case "Strike":
        return `${this.name(e.striker)}'s strike`;
      case "Summon":
        return `${this.name(e.unit)} appeared`;
      case "StatusApplied":
        return `${e.status} landed on ${this.name(e.unit)}`;
      case "StatusRemoved":
        return `${e.status} left ${this.name(e.unit)}`;
      case "TurnStart":
        return "the turn began";
      case "TurnEnd":
        return "the turn ended";
      case "Fatigue":
        return "fatigue struck";
      default:
        return `a ${e.type} event`;
    }
  }
}
