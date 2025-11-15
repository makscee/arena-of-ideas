# Arena of Ideas - Game Mechanics Guide

## Core Concepts

Arena of Ideas is a PvP auto-battler where players build teams by combining units into fusions that battle automatically.

## Battle System

### Team Composition
- **Teams** consist of up to 5 **Fusions**
- **Fusions** are combat units created by combining multiple **Units** in slots
- Each fusion has a trigger unit that determines when its actions fire
- Fusions inherit combined stats: PWR (power/damage), HP (health)

### Combat Flow
1. **Strike Phase**: Front fusions from each team strike each other simultaneously
   - Each striker deals PWR damage to the opposing striker
   - Damage can be modified by events (OutgoingDamage, IncomingDamage)
2. **Death Check**: Units with damage >= HP are removed
3. **Events**: Various triggers fire based on what happened
4. **Turn End**: Status effects and other turn-based mechanics process
5. Battle continues until one team has no fusions remaining

### Damage System
- Base damage = attacker's PWR stat
- Damage triggers OutgoingDamage event on attacker (can modify value)
- Damage triggers IncomingDamage event on defender (can modify value)
- Final damage is applied to target's damage counter
- When damage >= HP, unit dies

## Content Architecture

### Units
Each unit has:
- **Stats**: PWR and HP base values
- **Trigger**: Condition that activates the unit (BattleStart, TurnEnd, AllyDeath, etc.)
- **Behavior**: Reaction containing actions that execute when triggered
- **Magic Type**: Ability or Status
- Belongs to a **House** (faction)

### Houses
Thematic factions containing:
- **Units**: Collection of themed units
- **Ability Magic**: Active effect that can be triggered
- **Status Magic**: Passive effect that can be applied to units
- **Color**: Visual theme
- **Stax**: Stacking counter for multiple copies

### Abilities
Active effects with:
- **Name** and **Description**
- **Effect**: List of actions to execute
- Can target self, allies, enemies, or all units
- Consumes stax when used

### Statuses  
Passive effects with:
- **Name** and **Description**
- **Behavior**: Reactions that trigger on specific events
- **Stax**: Stack count (intensity/duration)
- **Representation**: Visual appearance
- Can modify damage, stats, or trigger special effects

## Action System

### Available Actions
- `deal_damage`: Deal damage equal to value to targets
- `heal_damage`: Heal damage equal to value from targets
- `set_value/add_value`: Modify the context value for chaining
- `add_target`: Add units to target list
- `use_ability`: Trigger house ability with stax power
- `apply_status`: Apply house status to targets
- `change_status_stax`: Modify status stack count
- `repeat`: Loop actions multiple times

### Expressions
Dynamic values used in actions:
- Unit references: `owner`, `target`, `all_ally_units`, `all_enemy_units`
- Variables: `var(pwr)`, `var(hp)`, `var(dmg)`, `var(stax)`
- Math: `sum`, `mul`, `div`, `max`, `min`
- Conditionals: `if`, `equals`, `greater_then`
- Random: `rand`, `random_unit`

## Triggers & Events

### Trigger Types
- **BattleStart**: When battle begins
- **TurnEnd**: End of each combat round
- **BeforeDeath**: Just before unit dies
- **AllyDeath**: When an ally dies
- **ChangeStat**: When a stat changes
- **ChangeOutgoingDamage**: Modify outgoing damage
- **ChangeIncomingDamage**: Modify incoming damage

### Event Flow
1. Event occurs (damage dealt, unit dies, turn ends, etc.)
2. All units/statuses with matching triggers activate
3. Actions execute in order, potentially creating new events
4. Process continues until no more triggers fire

## Match Progression

### Shop Phase
- Players have gold (g) to spend
- Shop offers units and houses for purchase
- Can reroll shop for new options
- Bought units go to team, houses provide abilities/statuses
- Houses stack when buying duplicates (increases power)

### Fusion Building
- Drag units into fusion slots
- First unit becomes the trigger unit
- Additional slots can be purchased
- Each slot can have action range configuration
- Combined stats = sum of all units in fusion

### Battle Progression
1. **Shop**: Buy units and organize team
2. **Battle**: Face random opponent from floor pool
3. **Win**: Gain gold, progress to next floor
4. **Loss**: Lose a life, stay on same floor
5. **Boss Battle**: Every few floors, face the floor boss
6. **Champion Battle**: At last floor, face previous champion

### Floor System
- Each floor has a pool of player teams
- Boss floors have special boss teams
- Defeating floor boss can make you the new boss
- Last floor champion is the ultimate challenge
- Players gain extra life every 5 floors

## Content Creation Guidelines

### Creating Units
Consider:
- **Role**: Damage dealer, support, tank, effect trigger
- **Trigger timing**: When should this unit activate?
- **Synergies**: What other units/houses work well with it?
- **Counter-play**: What can opponents do against it?

### Creating Abilities
- **Target selection**: Who does this affect?
- **Power scaling**: How does stax affect strength?
- **Timing**: Immediate effect or setup for later?
- **Combo potential**: What triggers can this enable?

### Creating Statuses
- **Trigger conditions**: What events should this react to?
- **Stack behavior**: How do multiple stacks interact?
- **Duration**: Permanent or decay over time?
- **Removal**: Can it be cleansed or does it expire?

### Balance Considerations
- **Power budget**: PWR * HP should be balanced
- **Action economy**: More complex effects need drawbacks
- **Rarity/cost**: Stronger effects should cost more
- **Counterplay**: Everything needs potential counters
- **Scaling**: Early game vs late game power

## Example Content Patterns

### Damage Amplifier Unit
```
Trigger: ChangeOutgoingDamage
Actions: add_value(mul(var(value), i32(2)))
Result: Doubles all outgoing damage
```

### Shield Status
```
Trigger: ChangeIncomingDamage  
Actions: set_value(sub(var(value), var(stax)))
Result: Reduces incoming damage by stax amount
```

### Chain Lightning Ability
```
Actions:
1. add_target(random_unit(all_enemy_units))
2. deal_damage
3. repeat(i32(2), [
   add_target(random_unit(all_enemy_units)),
   set_value(div(var(value), i32(2))),
   deal_damage
])
Result: Hits random enemy, then chains to 2 more at half damage
```
