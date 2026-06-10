# Battle Kernel Spec — DSL v1 (draft)

Normative spec for the v5 battle kernel. The resolver is built against this document; where code and spec disagree, one of them is a bug to fix consciously. Markers: **[PINNED]** decided, **[DEFER]** deliberately undecided, resolve during the stress test.

## 0. Doctrine

1. **Rules are data.** The engine is a small interpreter over this DSL. Heroes, abilities, and statuses are content, never engine code. New design ideas should land as data/schema changes; the engine only changes when the kernel itself grows.
2. **The battle is a pure function.** `battle(teamA, teamB, seed) → EventLog`. No I/O, no clock, no global state. Same inputs → byte-identical log. This single function runs in the browser, on the server, and in the simulation farm.
3. **The log is causal.** Every event records what caused it. "Why did my unit die" must be answerable by walking ancestry — observability is structural, not a UI afterthought (Artifact died of unreadable causality).
4. **The kernel stays small.** Magic (Poison, Freeze, Shield…) is composed content. The kernel grows only when composition provably cannot express something (see §7 stress set).

## 1. Data model

### Stats
Units have two base stats: `hp` and `pwr` (non-negative integers).

**[PINNED] Effective stats are computed, never baked:** `effective = base + Σ(status statMod contributions)`. Removing a status (e.g. Silence) makes its contribution vanish with it — there is no "unapply" step and no layering bug class.

### Unit
```
Unit {
  id: string            // instance id, unique within a battle
  name: string
  base: { hp, pwr }
  level: number         // shop concern; kernel reads it only for content that references it
  abilities: Ability[]  // usually 1
  statuses: StatusInstance[]   // runtime
}
```
Damage reduces a unit's *current* hp (tracked separately from base.hp = max). A unit whose hp reaches 0 dies (→ Death event, §4).

### Team
Ordered list of up to **5** units **[PINNED]**. Index 0 is the front. Sides are `A` (attacker — the player who initiated this async battle) and `B` (defender — the saved team).

### Ability
```
Ability {
  whens: When[]          // ≥1; Trigger or Interceptor
  condition?: Condition  // optional gate, checked at fire time
  selectors: Selector[]  // ≥1
  effects: Effect[]      // ≥1; a sequence
}
When = { kind: "trigger" | "interceptor", on: EventPattern }
```
**[PINNED] Firing semantics (v3 fusion semantics):** each matching `when` fires independently (more whens = more firings); on a firing, the effect sequence is applied **once per selected target, per selector** (more selectors = more applications); effects run in sequence order. This multiplicative structure is the depth engine; pricing it is the budget's job (out of scope v1, §8).

- **Trigger** — fires *after* its event pattern has applied.
- **Interceptor** — fires *instead of* a proposed event: it may cancel or transform the event before it applies (MTG triggered-vs-replacement split). Shield, Freeze, and death-prevention are inexpressible without interceptors.

### Status
A status is a named ability-bundle with a stack count — content, not an engine concept:
```
StatusDef {
  name: string
  statMods?: { hp?, pwr? }   // contribution per stack, applied while attached
  abilities: Ability[]        // whens may reference "holder"
}
StatusInstance { def: StatusDef, stacks: number }
```
**[PINNED] Stacks only — no durations.** Applying a status adds stacks (`StatusApplied`); a status at 0 stacks is removed (`StatusRemoved`). Decay/consumption is each status's own content (e.g. Poison consumes 1 stack per tick). The kernel knows nothing about decay.

### Part
The creator-facing atom: a single trigger, interceptor, condition, selector, or effect. Creation assembles abilities from parts; fusion recombines parts across units. (Empirical anchor: all of Super Auto Pets factors into ~25 such atoms.) Pricing/budget: §8.

## 2. Event model

```
Event {
  id: number          // ordinal, assigned at apply time
  turn: number
  type: EventType
  payload: {...}
  causedBy: number | null      // parent event id; null only for kernel beats
  source: AbilityRef | "kernel" // what emitted it: ability instance (unit/status + index) or the kernel loop
}
```

EventTypes v1: `BattleStart, TurnStart, PairFaced, Strike, Hurt, Heal, Death, Summon, StatusApplied, StatusRemoved, StatChanged, Fatigue, ChainBlocked, TurnEnd, Silenced, Intercepted, BattleEnd`.

- `PairFaced {first}` — emitted when two units face each other for the first time; records the seeded first-striker roll **[PINNED]** (glass-cannon design space: a roll, observable in the log).
- `Hurt {unit, amount, hpAfter, absorbed?}` / `Heal {unit, amount, hpAfter}` — hp deltas. `Strike` proposes a `Hurt` of the striker's effective pwr. `hpAfter` is the unit's current hp after the event applied, stamped by the kernel at apply time — consumers (replay, client) read it instead of re-deriving hp bookkeeping. `absorbed?` records how many hp a Shield interceptor consumed; a fully-absorbed Hurt still applies with `amount` 0.
- `ChainBlocked {ability, at}` — a would-be firing suppressed by the no-self-retrigger law (§5). The replay explains chain stops with this event.
- `Intercepted {by, original, unit?}` — emitted when an interceptor cancels a proposed event (e.g. Freeze cancelling a Strike, Blessing cancelling a Death); cancellations must be visible or the replay can't explain them. Interceptor side-effects (stack consumption, the replacement Heal) are caused by this event.
- `Silenced {unit}` — the unit's own abilities are disabled for the battle (ability disabling is kernel state, not removable content).
- The log is JSONL, one event per line. It is the *only* output of the battle; replays, attribution stats, and the sim gate all consume it.

## 3. Determinism

- One RNG stream (small PRNG, e.g. mulberry32), seeded by the battle seed. Only the kernel draws from it, at spec-defined points: the first-striker roll, and `random` selectors. Draw order is fully determined by the loop.
- Every collection iteration in this spec has a defined order (§5 ordering rule). There is no hash-map iteration anywhere in resolution.
- **Acceptance test:** running the same `(teamA, teamB, seed)` twice produces byte-identical JSONL.
- Ladder practice (from Mechabellum): store decisions + seeds, recompute outcomes; re-roll the seed per match, so a matchup is a *distribution*, not a memorizable result.

## 4. Battle loop (normative)

```
emit BattleStart; settle cascade
turn = 1
while both teams have living units and turn ≤ TURN_CAP:
  emit TurnStart; settle
  a = front(A), b = front(B)
  if (a,b) not faced before:
      first = seeded coin            // PairFaced {first}
  else first = remembered for (a,b)
  // alternating strikes [PINNED]:
  strike(first → other); settle      // Strike proposes Hurt = striker's effective pwr
  if other.alive and battle not decided:
      strike(other → first); settle   // back-strike only if BOTH units still alive
  emit TurnEnd; settle; then Fatigue (caused by TurnEnd, from FATIGUE_START on); settle
  turn += 1

fatigue [PINNED, v3]: from turn FATIGUE_START onward, every living unit
takes Fatigue damage = (turn − FATIGUE_START + 1) · FATIGUE_RAMP at end of turn.
Grows without bound → battles end. TURN_CAP is the hard backstop → draw.

end: a team has no living units → BattleEnd {winner};
both empty simultaneously, or TURN_CAP reached → BattleEnd {draw}.
BattleEnd {turns} = the turn the battle was decided on (the last turn that
started); 0 if it was decided in the BattleStart cascade, before turn 1.
```

- **Death and line collapse:** when a `Death` event applies, the unit leaves the line immediately and the team compacts forward (positions shift). Pairings are re-evaluated next strike; a fresh pairing rolls `PairFaced`.
- Dead units go to the **graveyard** — an append-only per-team list, the one zone besides the line **[PINNED]**. Death clears the unit's statuses (the corpse is clean). A resurrected unit returns at the back of the line and keeps its pair memory (first-striker rolls).

## 5. Cascade algorithm (normative)

Every state change flows through one pipeline: **propose → intercept → apply → trigger.**

1. **Propose.** An effect (or the kernel) proposes an event E with `causedBy` = the event that fired its ability (or the kernel beat).
2. **Intercept.** Eligible interceptors are offered E in **ordering rule** order; each may cancel or transform E. An interceptor instance handles a given proposed event at most once, and is subject to the no-self law below. What survives is the actual E.
3. **Apply.** E mutates state, gets its `id`, and is appended to the log.
4. **Trigger.** All trigger-whens matching E are collected in **ordering rule** order and their firings are enqueued FIFO (breadth-first). Processing continues until the queue is empty ("settle"), then the loop advances.

**[PINNED] Ordering rule (position priority):** whenever multiple abilities react to the same event, order is: side A front→back, then side B front→back; within a unit, ability list order; unit statuses after unit abilities, in attach order.

**[PINNED] The no-self-retrigger law:** an ability instance X may not fire in response to event E if any event in E's `causedBy` ancestry has `source` = X. One sentence for players: *an ability never triggers itself, directly or through others.* A suppressed firing emits `ChainBlocked`.
*Termination:* every causal path can contain each ability instance at most once as a source, so cascade depth is bounded by the number of ability instances on the board; the queue always drains. **Validated at stress test (2026-06-10):** ChainBlocked observed, queue always drained; the cascade-energy fallback was not needed and stays dormant **[DEFER]**.
Enqueued trigger firings resolve even if their holder has since died (MTG rule: triggers on the stack survive their source) — this is what lets on-death abilities like Summon fire.

## 6. Constants (tunable; sim-tunable, not design pins)

| constant | v1 value |
|---|---|
| TEAM_SIZE | 5 |
| FATIGUE_START | 10 |
| FATIGUE_RAMP | 1 |
| TURN_CAP | 200 |

## 7. Stress set — the kernel acceptance content

Nine abilities, shipped **as DSL data with behavior tests**. The kernel passes when all nine are expressible; where one isn't, the kernel grows consciously (a written verdict, then the schema change).

| name | composition | what it stress-tests |
|---|---|---|
| Strength | status, statMod +pwr/stack | computed stat layer |
| Vitality | status, statMod +hp/stack | computed stat layer |
| Curse | status, statMod −pwr/stack (floor 0) | negative mods |
| Poison | status; trigger: turn end → Hurt holder = stacks; consume 1 | triggers, self-referential decay |
| Shield | status; interceptor: Hurt on holder → absorb up to stacks, consume = absorbed | interceptors, event transformation |
| Freeze | status; interceptor: holder's Strike → cancel; consume 1 | interceptors, action denial |
| Blessing | status; interceptor: holder's Death → Heal to `stacks` hp instead; remove status | death interception |
| Summon | effect: spawn a defined unit at the back of caster's team | mid-cascade board mutation, fresh pairings |
| Silence | effect: remove all statuses from target; disable its abilities for the battle | layering (statMods must vanish), ability disabling |
| Resurrect | effect: return most recently dead ally at N hp | **likely kernel-breaker**: needs the graveyard zone |

(Silence and Resurrect are the designated kernel-breakers; their verdicts get written into the project notes before any schema change.)

## 8. Out of scope for v1 (pointers, not promises)

- **Run/shop layer:** gold carryover + round income, buy/reroll, 3–6 offers, duplicate-stacking → level-up → fusion eligibility.
- **Fusion & the budget:** fused units compose any subset of parents' parts *within a level-grown budget*; one pricing formula serves the creation gate and fusion alike; multi-when/multi-selector priced superlinearly (their value is multiplicative).
- **Creation pipeline:** LLM text→parts interface; sim gate (candidate parts vs live meta, win-rate band); vote gate (fun/flavor only).
- **Client:** a replay renderer over the event log; chosen last.

## 9. Stress-test resolutions (2026-06-10)

In-code decisions made during the stress build of `src/battle.ts`; recorded here per spec doctrine (conscious disagreement → fix one side).

- **Event types added:** `TurnEnd`, `Intercepted`, `Silenced` are now first-class event types (§2).
- **Graveyard adopted:** graveyard zone is real — per-team append-only list (§4). Resurrect's stress test demanded it; verdict: kernel, not content.
- **Death clears statuses:** the corpse is clean; statMod contributions vanish with the statuses (consistent with the computed-stat law, §1).
- **Fully-absorbed Hurt applies at amount 0:** the event still lands, causality is visible; Shield's interceptor records `absorbed` on it (§2).
- **Back-strike requires both units alive:** if the first striker kills the second, the return strike does not occur (§4).
- **Enqueued firings outlive their holder:** a trigger on the queue fires even if its source unit has since died — MTG rule; enables on-death abilities (§5).
- **Silence scope:** disables a unit's own abilities (including statuses) but not its basic kernel strike; statuses applied *after* the silence event still function (Silence is a point-in-time effect, not an ongoing shield against future application).
- **Pair memory survives death and resurrection:** first-striker rolls are keyed to the (a, b) pair; a resurrected unit rejoins with its prior rolls intact (§4).
- **Kernel passed:** all 9 stress abilities expressible as DSL data; no unresolved kernel-breakers remain.
