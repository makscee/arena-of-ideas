# Arena of Ideas - Battle Rules

## Battle Setup
- Two teams face each other, each with up to 5 fusions
- Fusions are positioned in order by their index (0-4)
- Front fusion (index 0) is the active striker
- Battle starts with BattleStart event triggering all relevant units

## Strike Rules
1. **Simultaneous Strike**: Front fusions strike each other at the same time
2. **Damage Calculation**: Each fusion deals damage equal to its PWR stat
3. **Damage Events**: 
   - OutgoingDamage event fires on attacker (can modify damage)
   - IncomingDamage event fires on defender (can modify damage)
4. **Damage Application**: Final damage value added to target's damage counter

## Death Rules
- Unit dies when: damage >= HP
- Death triggers in order:
  1. Death event sent (triggers BeforeDeath reactions)
  2. Unit removed from battle
  3. AllyDeath triggers on same team units
- Dead fusion is removed, next fusion moves to front

## Turn Structure
1. **Strike Phase**: Front units exchange strikes
2. **Action Resolution**: Process all triggered actions
3. **Death Check**: Remove dead units
4. **Turn End**: TurnEnd event triggers
5. **Next Turn**: If both teams have units, repeat

## Event Processing
- Events cascade: one event can trigger reactions that create more events
- Each unit/status checks if its trigger matches the event
- Matching triggers execute their actions in order
- Actions can modify values, deal damage, apply statuses, etc.

## Status Rules
- Statuses have stax (stack count) representing intensity
- Multiple stacks of same status combine (stax adds up)
- Statuses with 0 or negative stax don't trigger
- Status reactions can modify the event value being processed
- Statuses process after unit reactions for each event

## Fatigue System
- After many turns without resolution, fatigue begins
- Each team takes increasing damage each turn
- Prevents infinite battles

## Victory Conditions
- **Win**: Opponent has no fusions remaining
- **Loss**: You have no fusions remaining  
- **Draw**: Both teams die simultaneously (counts as loss for player)

## Reaction Priority
1. Fusion's unit reactions (in slot order)
2. Status reactions (in order applied)
3. Final value used for action

## Target Selection
- `owner`: The unit performing the action
- `target`: Current target in context
- `all_units`: Every unit in battle
- `all_ally_units`: All units on same team
- `all_enemy_units`: All units on opposite team
- `adjacent_*`: Units in adjacent positions

## Value Flow
- Actions work with a "value" in context
- Each action can read/modify this value
- Common pattern: set_value → add_value → deal_damage
- Values cascade through event reactions

## Fusion Mechanics
- **Trigger Unit**: First unit in fusion determines when it activates
- **Combined Stats**: PWR = sum of all units' PWR, HP = sum of all units' HP
- **Slot Actions**: Each slot can contribute actions when fusion triggers
- **Action Limit**: Maximum actions per trigger (prevents infinite loops)

## Special Rules
- Damage cannot go negative (minimum 0)
- Stats can be modified but don't go below 0
- Empty fusion slots are skipped
- Battle continues until one team is eliminated