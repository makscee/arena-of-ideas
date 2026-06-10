# Arena of Ideas — Battle Kernel

Domain glossary for the v5 battle kernel: the DSL, the resolver, and the causal event log. SPEC.md is normative; this file pins the vocabulary so every brief and discussion uses the same words the code uses.

## Language

### Content model

**Part**:
The creator-facing atom: a single trigger, interceptor, condition, selector, or effect. Creation assembles abilities from parts; fusion recombines parts across units.
_Avoid_: component, building block, module

**Unit**:
A named combatant with base stats, a level, abilities, and runtime statuses; identified within a battle by an instance id (e.g. `A1:Dummy`).
_Avoid_: hero (shop-layer word), creature, minion

**Team**:
An ordered list of 1–5 units. Index 0 is the front.
_Avoid_: party, squad

**Side**:
`A` (attacker — the player who initiated this async battle) or `B` (defender — the saved team).
_Avoid_: player 1/2, home/away

**Ability**:
The unit of behavior: ≥1 whens, an optional condition, ≥1 selectors, ≥1 effects. Each matching when fires independently; the effect sequence applies once per selected target, per selector.
_Avoid_: skill, power, spell

**When**:
An ability's binding to an event pattern; its kind is `trigger` or `interceptor`.
_Avoid_: listener, hook

**Trigger**:
A when that fires *after* its event pattern has applied.
_Avoid_: reaction, on-event handler

**Interceptor**:
A when that fires *instead of* a proposed event — it may cancel or transform the event before it applies (MTG triggered-vs-replacement split). Shield, Freeze, and death-prevention are inexpressible without it.
_Avoid_: replacement effect, guard, middleware

**Condition**:
An optional gate on an ability, checked at fire time (e.g. `holderHpAtMost`).
_Avoid_: predicate, requirement

**Selector**:
A target-choosing rule (`holder`, `eventUnit`, `frontEnemy`, `allEnemies`, `allAllies`, `randomEnemy`, `lastDeadAlly`). More selectors = more applications of the effect sequence.
_Avoid_: targeter, target selector (just "selector")

**Effect**:
An atomic state-change instruction; effects run in sequence order. Trigger-context atoms (damage, heal, applyStatus, consumeStacks, summon, silence, resurrect) are inert in interceptor context, and vice versa (cancel, absorbHurt, preventDeathHeal).
_Avoid_: action, operation

**Amount**:
A magnitude expression: `const`, holder's effective `stat`, or `stacks` of the owning status. Stat scaling is opt-in content, priced by the budget, never an engine rule.
_Avoid_: value, magnitude

**Status**:
A named ability-bundle with a stack count, attached to a unit at runtime (`Poison 3`). Content, never engine code — the player-creatable magic vocabulary.
_Avoid_: buff, debuff, keyword, aura

**Stacks**:
The only quantity a status carries. Applying adds stacks; 0 stacks = removed; decay/consumption is each status's own content. The kernel has no duration concept.
_Avoid_: duration, charges, counters

**Status registry**:
The name → StatusDef lookup a battle runs with; applying an unregistered status name throws.
_Avoid_: status table, catalog

**Holder**:
The unit an ability or status lives on. Unit filters (`holder`/`ally`/`enemy`/`any`) are relative to it; "ally" includes the holder.
_Avoid_: owner, bearer

### Stats

**hp / pwr**:
The only two stats. `Strike` proposes a Hurt of the striker's effective pwr; a unit at 0 current hp dies.
_Avoid_: health/attack, atk/def

**Computed stats**:
Effective stat = max(0, base + Σ statMod contributions per stack) — computed on read, never baked into state. Removing a status makes its contribution vanish; there is no "unapply" step and no layering bug class.
_Avoid_: applied buffs, stat snapshot

**statMod**:
A status's per-stack stat contribution while attached (e.g. Strength: +pwr per stack). Changes surface as `StatChanged` events.
_Avoid_: stat buff, modifier stack

### Battle loop

**battle()**:
The pure function `battle(teamA, teamB, seed) → event log`. No I/O, no clock, no global state; same inputs → byte-identical JSONL. Runs identically in browser, server, and sim farm.
_Avoid_: simulation, fight, match (match is a ladder concern)

**Seed**:
The single number seeding the battle's one RNG stream (mulberry32). Only the kernel draws from it: first-striker rolls and `randomEnemy` selectors.
_Avoid_: random state

**Line**:
A team's ordered positions; index 0 is the front. On Death the unit leaves immediately and the line compacts forward; summons and resurrections enter at the back.
_Avoid_: row, formation, board

**Turn**:
One loop iteration: TurnStart → front pair strikes (alternating) → TurnEnd → Fatigue. TURN_CAP (200) is the hard backstop → draw.
_Avoid_: round, tick

**Strike**:
The kernel-proposed basic attack of a front unit; its consequence is a Hurt of the striker's effective pwr. Not an ability — Silence never disables it.
_Avoid_: attack, hit, swing

**Alternating strikes**:
The pinned strike rule: a total order, never simultaneity. First striker strikes, cascade fully settles, then the back-strike — only if both units are still alive.
_Avoid_: simultaneous strikes, exchange

**Pair memory**:
First-striker results are keyed to the (a, b) pair and survive death and resurrection. A fresh pairing emits `PairFaced` recording the roll.
_Avoid_: matchup cache

**First-striker roll**:
The seeded coin deciding who strikes first when two units newly face each other — deliberate glass-cannon design space, observable in the log.
_Avoid_: initiative (Design B word), speed check

**Fatigue**:
Anti-stall damage: from turn FATIGUE_START (10) on, every living unit takes (turn − FATIGUE_START + 1) · FATIGUE_RAMP at end of turn. Grows without bound so battles end.
_Avoid_: sudden death, enrage timer

**Graveyard**:
The per-team list of dead units — units are appended on death and removed when resurrected — the one zone besides the line. Death clears the unit's statuses (the corpse is clean).
_Avoid_: discard pile, dead pool

**Death sweep**:
After any applied event, every living unit at ≤0 current hp dies — its Death caused by that event.
_Avoid_: cleanup step, state check

### Cascade & causality

**Cascade**:
The resolution of all reactions to a state change through one pipeline: propose → intercept → apply → trigger. Trigger firings enqueue FIFO (breadth-first).
_Avoid_: chain, stack (MTG stack is LIFO — ours is not)

**Settle**:
Draining the firing queue until empty before the loop advances. The no-self-retrigger law guarantees the queue always drains.
_Avoid_: resolve loop, flush

**Firing**:
One enqueued (ability, event) reaction. An enqueued firing still resolves if its holder has since died — this is what lets on-death abilities (Summon, Resurrect) fire.
_Avoid_: trigger instance, activation

**Ordering rule**:
Position priority: side A front→back, then side B front→back; within a unit, ability list order, then statuses in attach order. Every collection iteration in resolution has a defined order.
_Avoid_: priority order, speed order

**No-self-retrigger law**:
An ability instance never fires in response to an event whose `causedBy` ancestry contains itself as a source — "an ability never triggers itself, directly or through others". Bounds cascade depth structurally.
_Avoid_: recursion guard, loop limit, cascade energy (dormant [DEFER] fallback)

**ChainBlocked**:
The log event for a firing suppressed by the no-self-retrigger law — the replay explains chain stops with it.
_Avoid_: trigger skipped, loop detected

**Intercepted**:
The log event emitted when an interceptor cancels a proposed event (Freeze cancelling a Strike, Blessing cancelling a Death). Interceptor side-effects (stack consumption, the replacement Heal) are caused by it.
_Avoid_: cancelled, countered

**Kernel**:
The engine loop itself: the small interpreter over the DSL. Events it emits carry `source: "kernel"`; kernel beats are the only events with `causedBy: null`. The kernel grows only when composition provably cannot express something.
_Avoid_: engine core, framework

**Causal log**:
The battle's only output: JSONL, one event per line, each with `id`, `turn`, `causedBy` (parent event id), and `source`. "Why did my unit die" is answered by walking ancestry. Replays, attribution stats, and the sim gate all consume it.
_Avoid_: history, trace, combat log

### Named events worth knowing

**Hurt**:
The hp-loss event (`damage` is the effect atom that proposes it). Like Heal, it carries `hpAfter` — the unit's current hp after application, stamped by the kernel so consumers never re-derive hp bookkeeping. A fully Shield-absorbed Hurt still applies with `amount` 0 and an `absorbed` field — causality stays visible.
_Avoid_: damage event, hit event

**Silence / Silenced**:
The effect removes all statuses from the target and disables its own abilities for the battle (`Silenced` event; `silenced` is kernel state). Point-in-time: statuses applied after the silence still function; the basic strike is unaffected.
_Avoid_: mute, disable

**Summon**:
The event for a unit entering the line mid-battle, at the back; skipped if the line is full (5). Resurrect reuses it with a `resurrected` flag and `atHp`.
_Avoid_: spawn

**Resurrect**:
The effect returning a dead ally (via `lastDeadAlly`) to the back of the line at N hp. The stress test that forced the graveyard into the kernel.
_Avoid_: revive, respawn

### Tooling & acceptance

**Replay**:
The deterministic text rendering of an event log (`renderReplay`): same log in, byte-identical text out, every consequential line carrying its cause chain. The future client is just a fancier replay renderer.
_Avoid_: playback, visualization

**Seed sweep**:
CLI sweep mode: run the same two teams across seeds 0..N−1 and report the win-rate distribution. A matchup is a distribution, not a memorizable result.
_Avoid_: monte carlo, batch sim

**Team file**:
The CLI's input format: JSON `{ "units": UnitDef[] }`, 1–5 units; statuses resolve via the stress registry. Runs through the validator before battle().
_Avoid_: roster file, deck

**Validator**:
The content gate in front of the kernel (`src/validate.ts`): rejects unknown trigger/effect/selector kinds, wrong-context parts (an atom whose ability has no when it could fire in), malformed status bundles, and dangling references — with a path-addressed error, before battle() ever sees them. Without it a typo'd creation is silently inert. The embryo of the sim-gate content linter.
_Avoid_: schema checker, linter (the sim gate is the linter; this is its content-validity layer)

**Stress set**:
The kernel acceptance content — the abilities in `src/content/stress.ts` (Strength, Vitality, Curse, Poison, Shield, Freeze, Blessing, Summon, Silence, Resurrect) shipped as DSL data with behavior tests. The kernel passes when all are expressible; where one isn't, the kernel grows consciously.
_Avoid_: test fixtures, sample content
