# Old Gameplay Flow Analysis & Porting Plan

## Old System Summary

### Economy (GlobalSettings)
- **Starting gold**: 7
- **Unit buy cost**: 3 gold (flat, all units same price)
- **Unit sell value**: 1 gold
- **House buy cost**: 3 gold
- **Reroll cost**: 1 gold
- **Team slots**: 5
- **Fatigue start turn**: 10

### Match Flow

1. **Start match** → creates 5 empty team slots, fills shop
2. **Shop phase** → player buys units/houses, stacks, fuses, rerolls
3. **Regular battle** → fight against a team from the floor pool (other players' teams)
4. **Boss battle** → fight the floor boss (strongest team from previous floor)
5. **Champion battle** → after beating the boss, fight for champion spot
6. Win → advance floor, your team joins the floor pool
7. Lose regular → lose a life (3 lives total)
8. Lose boss → match ends (active = false)
9. Win champion → your team becomes the new floor boss

### Opponent Generation (KEY MECHANIC)

**Floor pools**: Each floor has a pool of teams submitted by players who passed through.

When starting a regular battle:
1. Player's current team is **copied** and added to the floor pool
2. A **random team from the pool** is selected as the opponent
3. If the pool is empty (first player on this floor), opponent is empty/default

This means **you fight other players' team compositions** asynchronously.
Players don't fight each other live — their team snapshots persist as opponents.

### Boss System

- Each floor has a **floor boss** — the champion team from when that floor was first conquered
- When a player wins a champion battle, their team becomes the new boss for that floor
- Boss battles are required to advance past the last floor
- The "last floor" advances when someone beats the champion battle

### Stacking

Two units of the **same type** can be stacked:
- Source unit is deleted
- Target unit gains +stax (stack count) to both HP and PWR
- `stax` tracks how many times the unit was stacked
- Stacking adds the source's `stax` value to target's `stax`, HP, and PWR

### Fusion (Old System)

The old fusion was the **beloved mechanic** with trigger/target/effect swapping:

1. **Start fusion**: Player drags one unit onto another
   - Both must be "fusible" (had specific conditions)
2. **Choose fusion**: Player picks 3 characters: `<`, `>`, `=`
   - Each character maps to: trigger, target, effect
   - `<` = take from source unit
   - `>` = take from target unit
   - `=` = combine both
3. **Result**:
   - Trigger: source's, target's, or `Any(both)`
   - Target: source's, target's, or `List(both)`
   - Effect: source's, target's, or concatenated scripts
   - Stats: kept from target unit
   - Name: concatenated (targetName + sourceName)
   - Both original units deleted, merged unit placed in target's slot

### Shop Structure

- Shop offers are `Vec<ShopSlot>` with:
  - `card_kind`: Unit or House
  - `node_id`: the actual content ID
  - `sold`: whether already bought
  - `price`: individual slot price
- Shop could offer both units AND houses
- `fill_shop_case` regenerated offers from the active content pool

### Battle States

```
Shop → RegularBattle → Shop (if won) → ... → BossBattle →
  if won → ChampionShop → ChampionBattle →
    if won → match ends (victory, team becomes new boss)
    if lost → match ends
  if lost → match ends
```

### Match Completion

- Match has `active` flag
- When `active = false`, player sees "Done" button → `match_complete` deletes the match
- Player can also `match_abandon` at any time

---

## Differences: Old vs New

| Feature | Old | New (Current) | Need to Port |
|---------|-----|---------------|--------------|
| Starting gold | 7 | 10 | Change to 7 |
| Unit buy cost | 3 (flat) | tier-based (1-5) | Decide: flat or tier-based |
| Unit sell value | 1 (flat) | tier-based (1-5) | Decide: flat or tier-based |
| Reroll cost | 1 | 1 | Same ✓ |
| Team slots | 5 | 5 | Same ✓ |
| Lives | 3 | 3 | Same ✓ |
| Opponents | Floor pool (other players' teams) | Random from active units | **PORT**: floor pool system |
| Boss battles | Floor boss (champion's team) | Not implemented | **PORT**: boss system |
| Champion battle | Win to become floor boss | Not implemented | **PORT**: champion mechanic |
| Stacking | Same unit type → +hp/+pwr per stax | Duplicate buy → +1/+1 per copy | Similar, adjust values |
| Fusion trigger | `<`/`>`/`=` for source/target/combine | Pick from either parent | **PORT**: `=` (combine) option |
| Fusion target | `<`/`>`/`=` for source/target/combine | N/A (abilities handle targeting) | Simplified in new design |
| Fusion effect | `<`/`>`/`=` for source/target/combine | Pick abilities from pool | Different but equivalent |
| Floor advancement | Win regular → stay on floor, win boss → next floor | Win → next floor | **PORT**: split regular/boss |
| Fatigue | Turn 10 | Turn 30 | Change to 10 |

---

## Porting Plan

### Phase A: Economy Adjustments

1. Change starting gold: 10 → 7
2. Decide on pricing:
   - Option 1: Keep tier-based (new design, more strategic)
   - Option 2: Flat 3g buy / 1g sell (old design, simpler)
   - **Recommend**: Keep tier-based but lower values. Tier 1=1g, Tier 2=2g, etc.
3. Change fatigue start: 30 → 10

### Phase B: Floor Pool System (Opponent Generation)

This is the most important port. Currently opponents are random — old system had asynchronous PvP.

**New tables needed:**
```
FloorPool
├── id: u64
├── floor: u8
└── created_at: Timestamp

FloorTeam
├── id: u64
├── pool_id: u64          # which floor pool
├── player: Identity      # who submitted
├── team_snapshot: String  # JSON of team composition at time of battle
└── created_at: Timestamp
```

**New reducers:**
- `match_start_battle`:
  1. Build player's team snapshot
  2. Add team to floor pool
  3. Pick random team from pool as opponent (exclude self)
  4. Create battle record
  5. Return opponent team for client simulation

**Flow:**
```
Start match (floor 1) → shop → buy units →
  start_battle → team copied to floor 1 pool →
    random opponent from floor 1 pool →
    client simulates battle →
  submit_result:
    win → stay on same floor, back to shop
    lose → lose life, back to shop (same floor)

  N wins on floor → boss battle available
  boss_battle → fight floor boss team →
    win → advance to next floor
    lose → lose life
```

### Phase C: Boss System

**New tables:**
```
FloorBoss
├── id: u64
├── floor: u8
├── team_snapshot: String  # the boss team
├── player: Identity       # who set this boss
└── created_at: Timestamp
```

**New reducers:**
- `match_boss_battle`:
  1. Get boss team for current floor
  2. Create battle with player team vs boss team
- On boss battle win:
  1. Player's team becomes new floor boss (or champion battle first)
  2. Advance to next floor

### Phase D: Fusion Enhancement

Current new system: pick trigger from either parent, pick abilities from pool.

Port the `=` (combine) option:
- **Trigger**: Add `Trigger::Any(vec![a, b])` — fires on either trigger (already in Trigger enum!)
- **Abilities**: Already picks from combined pool — this is equivalent to the old effect combining

The old `<`/`>`/`=` system maps to:
- `<` (take from source) = pick source's trigger
- `>` (take from target) = pick target's trigger
- `=` (combine both) = `Trigger::Any(source_trigger, target_trigger)`

**Changes needed:**
- Add fusion option for combined trigger (already supported by Trigger::Any)
- UI: show three options instead of two for trigger selection

### Phase E: Gold per Floor

Old system didn't have explicit gold-per-floor rewards — gold came from:
- Starting gold each match: 7
- Selling units: 1g each

New system gives gold on win. Adjust to match old feel:
- Win regular battle: +2g (enough to buy roughly one unit)
- Win boss battle: +3g
- Consider: gold = floor + 1 (scaling rewards)

### Phase F: Match State Machine

Port the richer state machine:
```rust
enum MatchState {
    Shop,
    RegularBattle,
    BossBattle,
    ChampionBattle,
    MatchOver { won: bool },
}
```

Add floor completion tracking:
- Track wins on current floor
- After N wins (e.g., 3), boss battle becomes available
- Boss battle required to advance floor

---

## Implementation Priority

1. **Phase B (Floor Pool)** — most impactful, gives the game its asynchronous PvP identity
2. **Phase C (Boss System)** — adds progression depth
3. **Phase D (Fusion `=` option)** — restores the beloved mechanic
4. **Phase A (Economy)** — tuning values
5. **Phase E (Gold rewards)** — tuning values
6. **Phase F (State machine)** — structural improvement
