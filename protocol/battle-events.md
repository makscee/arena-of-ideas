# battle-events/1

A battle is stored as a `BattleRecord` (`GET /api/v1/battles/:battleId`). Its `log` is the
whole fight: a list of `BattleEvent`s in the order they happened. A client replays the log; it
never re-runs the simulation. The types are `BattleEvent` and `EventBody` in `src/types.ts`, and
`BattleEvent` in [`schema.json`](schema.json) (served at `/api/v1/schema.json`).

## The promise

- `battle-events/1` only gains things: new event types, new optional fields on an event, new
  values where a field lists them. Renaming or removing an event type or a field, or changing what
  one means, is `battle-events/2` and comes with `/api/v2`.
- **A client skips an event type it doesn't know**, and ignores fields it doesn't know. Every
  event still has `id`, `turn`, `causedBy` and `source`, so the trace tree stays whole around a
  skipped event.
- A field marked *older logs* may be missing from battles fought before it existed: they are
  stored and served as fought.

## Every event

```ts
{ id: number;              // its index in the log, from 0
  turn: number;            // 0 before the first TurnStart
  causedBy: number | null; // the event it reacted to; null only for the kernel's beats
  source: "kernel" | AbilityRef;
  type: "...";  ...the fields of its type
}
AbilityRef = { unit: string;     // the unit holding the ability
               status?: string;  // set when the ability lives on a status
               ability: number;  // its index in that ability list
               when?: number }   // which of its Whens fired it (older logs: absent)
```

Units are named by instance id (`A1:Fighter`, `B3:Coach`: side, line position at the start, name;
a summon gets a new id). `BattleStart` maps ids to names and stats; `Summon` adds the ids that
enter later. `side` is `"A"` (the record's `player`) or `"B"` (its `opponent`).

## `causedBy`: the trace tree

`causedBy` makes the log a forest. Its roots (`causedBy: null`) are the kernel's beats:
`BattleStart`, each `TurnStart` and `TurnEnd`, and `BattleEnd`. Every other event hangs under the
event that set it off: a `Strike` under its `TurnStart`, the `Hurt` under the `Strike`, a `Death`
under the `Hurt`, a `StatChanged` under the `StatusApplied` that moved the stat. `source` says who
acted: `"kernel"` for the rules themselves, an `AbilityRef` when a unit's ability fired in reaction
to the parent. The why-chain of any event is its parents up to the root; the client's "why" view
(`src/mvp/trace.ts`) is exactly that walk. Chains fire in line order, so siblings keep log order.

## Types

### BattleStart
`teams: { A: RosterEntry[]; B: RosterEntry[] }`, `RosterEntry = { id, name, hp, pwr }`. Always event 0.

### TurnStart / TurnEnd
No fields. Bracket a turn.

### PairFaced
`a`, `b`: the two front units; `first`: the one that strikes first.

### Strike
`striker`, `defender`.

### Hurt
`unit`, `amount`, `hpAfter` (hp after it applied), `absorbed?` (taken by a shield), `pierced?: true`
(sudden death's fatigue: no interceptor stops it).

### Heal
`unit`, `amount`, `hpAfter`.

### Death
`unit`, `pierced?: true` (dealt by a pierced Hurt: no Blessing stops it).

### Summon
`unit` (the new id, or the returning one on a revive), `name`, `side`, `hp`, `pwr`,
`resurrected?` (a revive), `atHp?`, `front?` (entered at the front; absent: the back), `slot?` (a
revive's index in its line; *older logs*: absent, it returned at the back).

### StatusApplied / StatusRemoved
`unit`, `status`, `stacks` (this change), then `total` (Applied) or `remaining` (Removed).

### StatChanged
`unit`, `stat` (`"hp"` or `"pwr"`), `delta`, `now`, `hpAfter?` (hp changes only).

### Silenced
`unit`: its abilities stop firing.

### Fatigue
`amount`, `suddenDeath?: true` (from the sudden-death turn: doubled, piercing).

### ChainBlocked
`ability: AbilityRef`, `at`: a firing that didn't happen because the ability would have reacted to
its own effect; `at` is the event it would have reacted to.

### ChainCapped
`root` (the event the cascade's first firing reacted to), `steps`: the cascade hit the step cap
and its queued firings were dropped.

### NoRoom
`unit` (the holder whose ability tried), `side` (the full line), `name` (who found no room),
`revive?` (the dead unit's id on a revive).

### SummonFailed
`name`, `side`, `unit?`, `revive?: true`, `reason`: `"suddenDeath"` (nobody enters the line) or
`"revivedOnce"` (a unit returns once per battle).

### Intercepted
`by: AbilityRef`, `original` (the cancelled event's type), `unit?` (its subject): an interceptor
ability cancelled an event before it happened; the cancelled event is not in the log.

### BattleEnd
`winner` (`"A"`, `"B"` or `"draw"`), `turns`, `timeUp?: true` (the turn cap ran out with both sides
standing). Always the last event.

## Example

From a real fight (turn 1):

```json
{ "id": 19, "turn": 1, "causedBy": 15, "source": "kernel", "type": "Strike", "striker": "A1:Fighter", "defender": "B1:Nurse" }
{ "id": 20, "turn": 1, "causedBy": 19, "source": "kernel", "type": "Hurt", "unit": "B1:Nurse", "amount": 1, "absorbed": 1, "hpAfter": 4 }
{ "id": 21, "turn": 1, "causedBy": 20, "source": { "unit": "B1:Nurse", "status": "Shield", "ability": 0, "when": 0 }, "type": "StatusRemoved", "unit": "B1:Nurse", "status": "Shield", "stacks": 1, "remaining": 0 }
{ "id": 22, "turn": 1, "causedBy": 19, "source": { "unit": "A1:Fighter", "ability": 0, "when": 0 }, "type": "Hurt", "unit": "B1:Nurse", "amount": 1, "hpAfter": 3 }
```

The Fighter strikes the Nurse (19, under turn 1's `TurnStart` 15). The hit (20) loses 1 to the
Nurse's Shield, whose own ability then removes it (21, under 20). The Fighter's ability reacts to
its strike with one more hit (22, also under 19).
