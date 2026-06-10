// The battle resolver — a pure function: battle(teamA, teamB, seed) → causal event log.
// Every state change flows through one pipeline: propose → intercept → apply → trigger (SPEC §5).

import type {
  Ability,
  AbilityRef,
  Amount,
  BattleEvent,
  BattleInput,
  Condition,
  Effect,
  EventBody,
  EventPattern,
  RosterEntry,
  Selector,
  Side,
  SourceRef,
  StatName,
  StatusDef,
  StatusRegistry,
  UnitDef,
  UnitFilter,
} from "./types.js";
import { mulberry32 } from "./rng.js";

export const TEAM_SIZE = 5;
export const FATIGUE_START = 10;
export const FATIGUE_RAMP = 1;
export const TURN_CAP = 200;

interface StatusInstance {
  def: StatusDef;
  stacks: number;
  attachedAt: number;
}

interface UnitState {
  id: string;
  name: string;
  side: Side;
  base: { hp: number; pwr: number };
  level: number;
  abilities: Ability[];
  statuses: StatusInstance[];
  damage: number; // taken; current hp = effective hp − damage
  silenced: boolean;
  alive: boolean;
}

interface ReactorEntry {
  ref: AbilityRef;
  holder: string;
  ability: Ability;
}

interface Firing extends ReactorEntry {
  event: BattleEvent;
}

/** Side-channel for state that an event payload can't carry (defs are not serialized). */
interface Pending {
  summon?: UnitState;
  resurrect?: { unit: string; hp: number };
}

export function battle(input: BattleInput): BattleEvent[] {
  return new Engine(input).run();
}

export function toJSONL(log: BattleEvent[]): string {
  return log.map((e) => JSON.stringify(e)).join("\n") + "\n";
}

export function winnerOf(log: BattleEvent[]): Side | "draw" {
  const end = log[log.length - 1];
  if (!end || end.type !== "BattleEnd") throw new Error("log has no BattleEnd");
  return end.winner;
}

class Engine {
  private log: BattleEvent[] = [];
  private units = new Map<string, UnitState>();
  private lines: Record<Side, string[]> = { A: [], B: [] };
  private graves: Record<Side, string[]> = { A: [], B: [] };
  private queue: Firing[] = [];
  private pairs = new Map<string, string>();
  private rng: () => number;
  private registry: StatusRegistry;
  private turn = 0;
  private attachCounter = 0;
  private summonCounter = 0;
  private input: BattleInput;

  constructor(input: BattleInput) {
    this.input = input;
    this.rng = mulberry32(input.seed);
    this.registry = input.statuses ?? {};
  }

  run(): BattleEvent[] {
    const roster = this.setup();
    const bs = this.propose({ type: "BattleStart", teams: roster }, null, "kernel");
    if (bs) this.applyInitialStatuses(bs.id);
    this.settle();

    let turns = 0; // the turn the battle was decided on; 0 = decided before turn 1
    for (this.turn = 1; this.turn <= TURN_CAP; this.turn++) {
      if (!this.bothAlive()) break;
      turns = this.turn;
      const ts = this.propose({ type: "TurnStart" }, null, "kernel");
      this.settle();
      if (!ts || !this.bothAlive()) break;

      const a = this.front("A");
      const b = this.front("B");
      const key = pairKey(a.id, b.id);
      let first = this.pairs.get(key);
      if (first === undefined) {
        first = this.rng() < 0.5 ? a.id : b.id;
        this.propose({ type: "PairFaced", a: a.id, b: b.id, first }, ts.id, "kernel");
        this.settle();
      }
      const [x, y] = first === a.id ? [a, b] : [b, a];
      this.propose({ type: "Strike", striker: x.id, defender: y.id }, ts.id, "kernel");
      this.settle();
      if (x.alive && y.alive && this.bothAlive()) {
        this.propose({ type: "Strike", striker: y.id, defender: x.id }, ts.id, "kernel");
        this.settle();
      }

      const te = this.propose({ type: "TurnEnd" }, null, "kernel");
      this.settle();
      if (te && this.turn >= FATIGUE_START && this.bothAlive()) {
        const amount = (this.turn - FATIGUE_START + 1) * FATIGUE_RAMP;
        this.propose({ type: "Fatigue", amount }, te.id, "kernel");
        this.settle();
      }
    }

    const aAlive = this.lines.A.length > 0;
    const bAlive = this.lines.B.length > 0;
    const winner: Side | "draw" = aAlive === bAlive ? "draw" : aAlive ? "A" : "B";
    this.turn = turns; // BattleEnd is stamped with the deciding turn, not the loop's overshoot
    this.propose({ type: "BattleEnd", winner, turns }, null, "kernel");
    return this.log;
  }

  // ---------- setup ----------

  private setup(): { A: RosterEntry[]; B: RosterEntry[] } {
    const roster: { A: RosterEntry[]; B: RosterEntry[] } = { A: [], B: [] };
    for (const side of ["A", "B"] as const) {
      const team = side === "A" ? this.input.teamA : this.input.teamB;
      if (team.length === 0 || team.length > TEAM_SIZE) {
        throw new Error(`team ${side} must have 1..${TEAM_SIZE} units`);
      }
      team.forEach((def, i) => {
        const u = this.makeUnit(def, side, `${side}${i + 1}:${def.name}`);
        this.units.set(u.id, u);
        this.lines[side].push(u.id);
        roster[side].push({ id: u.id, name: u.name, hp: u.base.hp, pwr: u.base.pwr });
      });
    }
    return roster;
  }

  private makeUnit(def: UnitDef, side: Side, id: string): UnitState {
    return {
      id,
      name: def.name,
      side,
      base: { ...def.base },
      level: def.level ?? 1,
      abilities: def.abilities ?? [],
      statuses: [],
      damage: 0,
      silenced: false,
      alive: true,
    };
  }

  private applyInitialStatuses(causeId: number): void {
    for (const side of ["A", "B"] as const) {
      const team = side === "A" ? this.input.teamA : this.input.teamB;
      team.forEach((def, i) => {
        const unitId = this.lines[side][i];
        if (unitId === undefined) return;
        for (const s of def.statuses ?? []) {
          this.proposeStatusApplied(unitId, s.status, s.stacks, causeId, "kernel");
        }
      });
    }
  }

  // ---------- the cascade pipeline ----------

  /** propose → intercept → apply → enqueue triggers. Returns the applied event, or null if cancelled. */
  private propose(body: EventBody, causedBy: number | null, source: SourceRef, pending?: Pending): BattleEvent | null {
    const draft: EventBody = { ...body };
    const followUps: Array<(parentId: number) => void> = [];
    const handled = new Set<string>();
    let cancelledBy: AbilityRef | null = null;

    for (const r of this.orderedReactors()) {
      for (const when of r.ability.whens) {
        if (when.kind !== "interceptor" || !this.matches(when.on, draft, r.holder)) continue;
        const k = refKey(r.ref);
        if (handled.has(k)) continue;
        if (this.violatesNoSelf(r.ref, source, causedBy)) {
          this.applyEvent({ type: "ChainBlocked", ability: r.ref, at: causedBy ?? -1 }, causedBy, "kernel");
          continue;
        }
        const holder = this.units.get(r.holder);
        if (!holder || (r.ref.status === undefined && holder.silenced)) continue;
        if (r.ability.condition && !this.checkCondition(r.ability.condition, holder)) continue;
        handled.add(k);
        const res = this.runInterceptor(r, draft, followUps);
        if (res === "cancel") {
          cancelledBy = r.ref;
          break;
        }
      }
      if (cancelledBy) break;
    }

    if (cancelledBy) {
      const subj = subjectOf(draft);
      const note = this.applyEvent(
        { type: "Intercepted", by: cancelledBy, original: draft.type, ...(subj !== undefined ? { unit: subj } : {}) },
        causedBy,
        cancelledBy,
      );
      for (const f of followUps) f(note.id);
      this.deathSweep(note);
      return null;
    }

    const ev = this.applyEvent(draft, causedBy, source, pending);
    for (const f of followUps) f(ev.id);
    this.deathSweep(ev);
    return ev;
  }

  /** Mutate state, append to log, enqueue matching triggers, run kernel consequences. */
  private applyEvent(body: EventBody, causedBy: number | null, source: SourceRef, pending?: Pending): BattleEvent {
    const reactors = this.orderedReactors(); // snapshot before mutation: a dying unit still reacts to its own Death
    const ev: BattleEvent = { id: this.log.length, turn: this.turn, causedBy, source, ...body };
    this.mutate(ev, pending);
    this.log.push(ev);
    for (const r of reactors) {
      for (const when of r.ability.whens) {
        if (when.kind === "trigger" && this.matches(when.on, ev, r.holder)) {
          this.queue.push({ ...r, event: ev });
        }
      }
    }
    this.kernelConsequences(ev);
    return ev;
  }

  private settle(): void {
    while (this.queue.length > 0) {
      const f = this.queue.shift()!;
      this.processFiring(f);
    }
  }

  private processFiring(f: Firing): void {
    if (this.violatesNoSelf(f.ref, f.event.source, f.event.causedBy)) {
      this.applyEvent({ type: "ChainBlocked", ability: f.ref, at: f.event.id }, f.event.id, "kernel");
      return;
    }
    const holder = this.units.get(f.holder);
    if (!holder) return;
    if (f.ref.status === undefined && holder.silenced) return;
    if (f.ability.condition && !this.checkCondition(f.ability.condition, holder)) return;
    for (const sel of f.ability.selectors) {
      for (const target of this.evalSelector(sel, holder, f.event)) {
        for (const effect of f.ability.effects) {
          this.runEffect(effect, target, holder, f);
        }
      }
    }
  }

  // ---------- the no-self-retrigger law ----------

  /** An ability never fires downstream of its own firing: check the proposal's source and full causal ancestry. */
  private violatesNoSelf(ref: AbilityRef, source: SourceRef, causedBy: number | null): boolean {
    if (sourceIs(source, ref)) return true;
    let id = causedBy;
    while (id !== null) {
      const e = this.log[id];
      if (!e) break;
      if (sourceIs(e.source, ref)) return true;
      id = e.causedBy;
    }
    return false;
  }

  // ---------- ordering rule: side A front→back, then B; unit abilities, then statuses in attach order ----------

  private orderedReactors(): ReactorEntry[] {
    const out: ReactorEntry[] = [];
    const addUnit = (u: UnitState) => {
      if (!u.silenced) {
        u.abilities.forEach((ab, i) => out.push({ ref: { unit: u.id, ability: i }, holder: u.id, ability: ab }));
      }
      for (const st of [...u.statuses].sort((p, q) => p.attachedAt - q.attachedAt)) {
        st.def.abilities.forEach((ab, i) =>
          out.push({ ref: { unit: u.id, status: st.def.name, ability: i }, holder: u.id, ability: ab }),
        );
      }
    };
    for (const side of ["A", "B"] as const) {
      for (const uid of this.lines[side]) {
        const u = this.units.get(uid);
        if (u) addUnit(u);
      }
    }
    return out;
  }

  // ---------- matching ----------

  private matches(p: EventPattern, body: EventBody, holderId: string): boolean {
    if (p.on !== body.type) return false;
    const holder = this.units.get(holderId);
    if (!holder) return false;
    if ("striker" in p && body.type === "Strike") return this.filterMatches(p.striker, body.striker, holder);
    if ("status" in p && (body.type === "StatusApplied" || body.type === "StatusRemoved")) {
      if (p.status !== undefined && p.status !== body.status) return false;
    }
    if ("unit" in p) {
      const subj = subjectOf(body);
      if (subj === undefined) return false;
      return this.filterMatches(p.unit, subj, holder);
    }
    return true;
  }

  private filterMatches(f: UnitFilter | undefined, unitId: string, holder: UnitState): boolean {
    if (f === undefined || f === "any") return true;
    if (f === "holder") return unitId === holder.id;
    const u = this.units.get(unitId);
    if (!u) return false;
    return f === "ally" ? u.side === holder.side : u.side !== holder.side;
  }

  private checkCondition(c: Condition, holder: UnitState): boolean {
    switch (c.kind) {
      case "holderHpAtMost":
        return this.curHp(holder) <= c.value;
    }
  }

  // ---------- selectors ----------

  private evalSelector(sel: Selector, holder: UnitState, event: BattleEvent): UnitState[] {
    switch (sel.kind) {
      case "holder":
        return [holder];
      case "eventUnit": {
        const subj = subjectOf(event);
        const u = subj !== undefined ? this.units.get(subj) : undefined;
        return u ? [u] : [];
      }
      case "frontEnemy": {
        const id = this.lines[other(holder.side)][0];
        const u = id !== undefined ? this.units.get(id) : undefined;
        return u ? [u] : [];
      }
      case "allEnemies":
        return this.lines[other(holder.side)].map((id) => this.units.get(id)!).filter(Boolean);
      case "allAllies":
        return this.lines[holder.side].map((id) => this.units.get(id)!).filter(Boolean);
      case "randomEnemy": {
        const pool = this.lines[other(holder.side)].map((id) => this.units.get(id)!).filter(Boolean);
        if (pool.length === 0) return [];
        return [pool[Math.floor(this.rng() * pool.length)]!];
      }
      case "lastDeadAlly": {
        const grave = this.graves[holder.side];
        for (let i = grave.length - 1; i >= 0; i--) {
          const u = this.units.get(grave[i]!);
          if (u && !u.alive) return [u];
        }
        return [];
      }
    }
  }

  // ---------- effects (trigger context) ----------

  private runEffect(e: Effect, target: UnitState, holder: UnitState, f: Firing): void {
    const amountCtx = { holder, ref: f.ref };
    switch (e.kind) {
      case "damage": {
        if (!target.alive) return;
        const amount = this.evalAmount(e.amount, amountCtx);
        this.propose({ type: "Hurt", unit: target.id, amount }, f.event.id, f.ref);
        return;
      }
      case "heal": {
        if (!target.alive) return;
        const amount = Math.min(this.evalAmount(e.amount, amountCtx), target.damage);
        if (amount <= 0) return;
        this.propose({ type: "Heal", unit: target.id, amount }, f.event.id, f.ref);
        return;
      }
      case "applyStatus": {
        if (!target.alive) return;
        this.proposeStatusApplied(target.id, e.status, this.evalAmount(e.stacks, amountCtx), f.event.id, f.ref);
        return;
      }
      case "consumeStacks": {
        const name = e.status ?? f.ref.status;
        if (name === undefined) return;
        const inst = target.statuses.find((s) => s.def.name === name);
        if (!inst) return;
        const n = Math.min(this.evalAmount(e.stacks, amountCtx), inst.stacks);
        if (n <= 0) return;
        this.propose(
          { type: "StatusRemoved", unit: target.id, status: name, stacks: n, remaining: inst.stacks - n },
          f.event.id,
          f.ref,
        );
        return;
      }
      case "summon": {
        const side = target.side;
        if (this.lines[side].length >= TEAM_SIZE) return;
        const id = `${side}+${++this.summonCounter}:${e.unit.name}`;
        const u = this.makeUnit(e.unit, side, id);
        this.propose(
          { type: "Summon", unit: id, name: u.name, side, hp: u.base.hp, pwr: u.base.pwr },
          f.event.id,
          f.ref,
          { summon: u },
        );
        return;
      }
      case "silence": {
        if (!target.alive || target.silenced) return;
        for (const st of [...target.statuses].sort((p, q) => p.attachedAt - q.attachedAt)) {
          this.propose(
            { type: "StatusRemoved", unit: target.id, status: st.def.name, stacks: st.stacks, remaining: 0 },
            f.event.id,
            f.ref,
          );
        }
        this.propose({ type: "Silenced", unit: target.id }, f.event.id, f.ref);
        return;
      }
      case "resurrect": {
        if (target.alive) return;
        if (this.lines[target.side].length >= TEAM_SIZE) return;
        const hp = Math.max(1, this.evalAmount(e.hp, amountCtx));
        this.propose(
          {
            type: "Summon",
            unit: target.id,
            name: target.name,
            side: target.side,
            hp: target.base.hp,
            pwr: target.base.pwr,
            resurrected: true,
            atHp: hp, // hp/pwr above are the unit's base; atHp is the hp it actually returns at
          },
          f.event.id,
          f.ref,
          { resurrect: { unit: target.id, hp } },
        );
        return;
      }
      default:
        return; // interceptor-only atoms are inert in trigger context
    }
  }

  private proposeStatusApplied(unitId: string, status: string, stacks: number, causedBy: number, source: SourceRef): void {
    const def = this.registry[status];
    if (!def) throw new Error(`unknown status "${status}" — not in registry`);
    if (stacks <= 0) return;
    const u = this.units.get(unitId);
    if (!u || !u.alive) return;
    const cur = u.statuses.find((s) => s.def.name === status)?.stacks ?? 0;
    this.propose({ type: "StatusApplied", unit: unitId, status, stacks, total: cur + stacks }, causedBy, source);
  }

  // ---------- interceptor execution ----------

  private runInterceptor(r: ReactorEntry, draft: EventBody, followUps: Array<(parentId: number) => void>): "pass" | "cancel" {
    const holder = this.units.get(r.holder);
    if (!holder) return "pass";
    const own = r.ref.status !== undefined ? holder.statuses.find((s) => s.def.name === r.ref.status) : undefined;
    const consumeOwn = (n: number) => {
      if (!own || r.ref.status === undefined) return;
      const take = Math.min(n, own.stacks);
      if (take <= 0) return;
      followUps.push((parentId) =>
        this.propose(
          { type: "StatusRemoved", unit: holder.id, status: r.ref.status!, stacks: take, remaining: own.stacks - take },
          parentId,
          r.ref,
        ),
      );
    };

    for (const effect of r.ability.effects) {
      switch (effect.kind) {
        case "cancel": {
          if (effect.consumeSelf !== undefined) consumeOwn(effect.consumeSelf);
          return "cancel";
        }
        case "absorbHurt": {
          if (draft.type !== "Hurt" || !own) break;
          const absorbed = Math.min(own.stacks, draft.amount);
          if (absorbed <= 0) break;
          draft.amount -= absorbed;
          draft.absorbed = (draft.absorbed ?? 0) + absorbed;
          consumeOwn(absorbed);
          break;
        }
        case "preventDeathHeal": {
          if (draft.type !== "Death") break;
          const unitId = draft.unit;
          const toHp = this.evalAmount(effect.toHp, { holder, ref: r.ref });
          followUps.push((parentId) => {
            const u = this.units.get(unitId);
            if (!u) return;
            const amount = toHp - this.curHp(u);
            if (amount > 0) this.propose({ type: "Heal", unit: unitId, amount }, parentId, r.ref);
          });
          if (effect.removeSelf && own && r.ref.status !== undefined) consumeOwn(own.stacks);
          return "cancel";
        }
        default:
          break; // trigger-context atoms are inert in interceptor context
      }
    }
    return "pass";
  }

  // ---------- mutation ----------

  private mutate(ev: BattleEvent, pending?: Pending): void {
    switch (ev.type) {
      case "PairFaced":
        this.pairs.set(pairKey(ev.a, ev.b), ev.first);
        return;
      case "Hurt": {
        const u = this.units.get(ev.unit);
        if (u && u.alive) u.damage += ev.amount;
        return;
      }
      case "Heal": {
        const u = this.units.get(ev.unit);
        if (u) u.damage = Math.max(0, u.damage - ev.amount);
        return;
      }
      case "Death": {
        const u = this.units.get(ev.unit);
        if (!u || !u.alive) return;
        u.alive = false;
        u.statuses = []; // the corpse is clean: statuses end at death
        const line = this.lines[u.side];
        const i = line.indexOf(u.id);
        if (i >= 0) line.splice(i, 1);
        this.graves[u.side].push(u.id);
        return;
      }
      case "Summon": {
        if (pending?.resurrect) {
          const u = this.units.get(pending.resurrect.unit);
          if (!u) return;
          u.alive = true;
          u.damage = Math.max(0, this.effStat(u, "hp") - pending.resurrect.hp);
          const gi = this.graves[u.side].indexOf(u.id);
          if (gi >= 0) this.graves[u.side].splice(gi, 1);
          this.lines[u.side].push(u.id);
        } else if (pending?.summon) {
          this.units.set(pending.summon.id, pending.summon);
          this.lines[pending.summon.side].push(pending.summon.id);
        }
        return;
      }
      case "StatusApplied": {
        const u = this.units.get(ev.unit);
        if (!u || !u.alive) return;
        const inst = u.statuses.find((s) => s.def.name === ev.status);
        if (inst) inst.stacks += ev.stacks;
        else {
          const def = this.registry[ev.status];
          if (!def) throw new Error(`unknown status "${ev.status}"`);
          u.statuses.push({ def, stacks: ev.stacks, attachedAt: this.attachCounter++ });
        }
        return;
      }
      case "StatusRemoved": {
        const u = this.units.get(ev.unit);
        if (!u) return;
        const inst = u.statuses.find((s) => s.def.name === ev.status);
        if (!inst) return;
        inst.stacks -= ev.stacks;
        if (inst.stacks <= 0) u.statuses = u.statuses.filter((s) => s !== inst);
        return;
      }
      case "Silenced": {
        const u = this.units.get(ev.unit);
        if (u) u.silenced = true;
        return;
      }
      default:
        return; // BattleStart/TurnStart/TurnEnd/Strike/Fatigue/StatChanged/ChainBlocked/Intercepted/BattleEnd mutate nothing
    }
  }

  // ---------- kernel consequences ----------

  private kernelConsequences(ev: BattleEvent): void {
    switch (ev.type) {
      case "Strike": {
        const striker = this.units.get(ev.striker);
        const defender = this.units.get(ev.defender);
        if (!striker || !defender || !defender.alive) return;
        const amount = this.effStat(striker, "pwr");
        this.propose({ type: "Hurt", unit: ev.defender, amount }, ev.id, "kernel");
        return;
      }
      case "Fatigue": {
        for (const side of ["A", "B"] as const) {
          for (const uid of [...this.lines[side]]) {
            const u = this.units.get(uid);
            if (u && u.alive) this.propose({ type: "Hurt", unit: uid, amount: ev.amount }, ev.id, "kernel");
          }
        }
        return;
      }
      case "StatusApplied":
      case "StatusRemoved": {
        const u = this.units.get(ev.unit);
        const def = this.registry[ev.status];
        if (!u || !def?.statMods) return;
        const sign = ev.type === "StatusApplied" ? 1 : -1;
        for (const stat of ["hp", "pwr"] as StatName[]) {
          const mod = def.statMods[stat];
          if (mod === undefined || mod === 0) continue;
          this.propose(
            { type: "StatChanged", unit: ev.unit, stat, delta: sign * mod * ev.stacks, now: this.effStat(u, stat) },
            ev.id,
            "kernel",
          );
        }
        return;
      }
      default:
        return;
    }
  }

  /** After any applied event, any living unit at ≤0 hp dies — caused by that event. */
  private deathSweep(cause: BattleEvent): void {
    for (const side of ["A", "B"] as const) {
      for (const uid of [...this.lines[side]]) {
        const u = this.units.get(uid);
        if (u && u.alive && this.curHp(u) <= 0) {
          this.propose({ type: "Death", unit: uid }, cause.id, "kernel");
        }
      }
    }
  }

  // ---------- stats ----------

  private effStat(u: UnitState, stat: StatName): number {
    let v = u.base[stat];
    for (const s of u.statuses) v += (s.def.statMods?.[stat] ?? 0) * s.stacks;
    return Math.max(0, v);
  }

  private curHp(u: UnitState): number {
    return this.effStat(u, "hp") - u.damage;
  }

  private evalAmount(a: Amount, ctx: { holder: UnitState; ref: AbilityRef }): number {
    switch (a.kind) {
      case "const":
        return a.value;
      case "stat":
        return this.effStat(ctx.holder, a.stat);
      case "stacks": {
        if (ctx.ref.status === undefined) return 0;
        return ctx.holder.statuses.find((s) => s.def.name === ctx.ref.status)?.stacks ?? 0;
      }
    }
  }

  // ---------- small helpers ----------

  private bothAlive(): boolean {
    return this.lines.A.length > 0 && this.lines.B.length > 0;
  }

  private front(side: Side): UnitState {
    const id = this.lines[side][0];
    const u = id !== undefined ? this.units.get(id) : undefined;
    if (!u) throw new Error(`no living unit on side ${side}`);
    return u;
  }
}

function subjectOf(body: EventBody): string | undefined {
  switch (body.type) {
    case "Hurt":
    case "Heal":
    case "Death":
    case "Summon":
    case "StatusApplied":
    case "StatusRemoved":
    case "StatChanged":
    case "Silenced":
      return body.unit;
    case "Strike":
      return body.striker;
    default:
      return undefined;
  }
}

function sourceIs(source: SourceRef, ref: AbilityRef): boolean {
  return (
    source !== "kernel" && source.unit === ref.unit && source.status === ref.status && source.ability === ref.ability
  );
}

function refKey(r: AbilityRef): string {
  return `${r.unit}|${r.status ?? ""}|${r.ability}`;
}

function pairKey(a: string, b: string): string {
  return a < b ? `${a}|${b}` : `${b}|${a}`;
}

function other(side: Side): Side {
  return side === "A" ? "B" : "A";
}
