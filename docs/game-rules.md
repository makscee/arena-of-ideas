# Arena of Ideas - Game Rules and Mechanics

## Game Overview

Arena of Ideas is a strategic auto-battler where players build teams of fusions (combined units) to compete in automated battles. Players progress through floors, managing resources and strategically combining units from different houses to create powerful synergies.

## Core Concepts

### 1. Units
- **Basic building blocks** of your team
- Each unit belongs to a **House**
- Has base stats:
  - **HP** (Hit Points): Health/durability
  - **PWR** (Power): Attack damage
  - **Stacks**: Additional unit instances/power multiplier
- Has a **Behavior**: Set of reactions (trigger + actions)
- Can be combined into **Fusions**
- Represented by a **Material** (visual appearance)

### 2. Houses
- **Thematic factions** that units belong to
- Each house has:
  - **House Name**: Unique identifier
  - **Color**: Visual theme (HexColor)
  - **Ability Magic**: Active spell that can be used
  - **Status Magic**: Passive effect that can be applied to units
  - **Units**: Collection of units belonging to this house

### 3. Fusions
- **Combat units** created by combining up to multiple units
- Properties:
  - **Slots**: Contains units (NFusionSlot with index, actions range, and unit reference)
  - **Trigger**: Selected behavior trigger from contained units (UnitTriggerRef)
  - **Index**: Position in team lineup (0-4)
  - **Stats**: Combined stats from units:
    - **PWR**: Combined power
    - **HP**: Combined hit points
    - **DMG**: Current damage taken
  - **Actions Limit**: Maximum number of actions per trigger

### 4. Teams
- **Battle formations** consisting of:
  - **Up to 5 Fusions** arranged in a row
  - **Houses**: Collection of houses used by the team
  - Front fusion engages in combat first

### 5. Matches
- **Game session** with:
  - **G (Gold)**: Currency for buying units/upgrades
  - **Floor**: Current progression level
  - **Lives**: Remaining health before game over
  - **Active**: Whether match is ongoing
  - **Shop Offers**: Available units/items to purchase
  - **Team**: Player's current team
  - **Battles**: History of battles fought

## Battle System

### Battle Flow
1. **Battle Start**
   - Teams positioned opposite each other
   - BattleStart triggers fire
   - Initial spawn animations play

2. **Combat Phase**
   - **Front fusions strike each other simultaneously**
   - Strike mechanic:
     - Each fusion deals its PWR as damage to opponent's HP
     - Damage can be modified by ChangeOutgoingDamage/ChangeIncomingDamage triggers
     - Visual and audio feedback (strike animation, sound effect)

3. **Death Check**
   - After each exchange, check for defeated units (HP <= 0)
   - BeforeDeath triggers fire for dying units
   - Death animations play
   - Dead fusions are removed from battle

4. **Slot Synchronization**
   - Remaining fusions move forward to fill gaps
   - Next fusions in line become the new front fighters

5. **Turn End**
   - TurnEnd triggers fire
   - Status effects tick down

6. **Loop or Victory**
   - If both teams have fusions: Return to Combat Phase
   - If one team has no fusions: Battle ends with victory/defeat

### Battle Actions
- **strike(attacker, defender)**: Basic attack exchange
- **damage(source, target, amount)**: Deal damage
- **heal(source, target, amount)**: Restore HP
- **death(entity)**: Remove unit from battle
- **spawn(entity)**: Add unit to battle
- **apply_status(target, status, charges, color)**: Add status effect
- **var_set(entity, variable, value)**: Modify unit statistics
- **send_event(event)**: Trigger game events
- **vfx(parameters, effect_name)**: Play visual effect

## Behavior System

### Triggers
Events that activate unit behaviors:
- **BattleStart**: When battle begins
- **TurnEnd**: After each combat round
- **BeforeDeath**: When unit is about to die
- **ChangeStat(VarName)**: Modify specific stat changes
- **ChangeOutgoingDamage**: Modify damage dealt
- **ChangeIncomingDamage**: Modify damage received

### Actions
Effects that can be executed:
- **noop**: No operation
- **debug(expression)**: Debug output
- **set_value(expression)**: Set value variable
- **add_value(expression)**: Increase value variable
- **subtract_value(expression)**: Decrease value variable
- **add_target(expression)**: Add target to context
- **deal_damage**: Inflict damage on targets
- **heal_damage**: Restore HP to targets
- **use_ability**: Activate house ability
- **repeat(count, actions)**: Loop actions

### Reactions
- Combination of **Trigger + Actions**
- Fusion inherits subset of unit reactions based on:
  - Selected trigger unit (UnitTriggerRef)
  - Action ranges (UnitActionRange: trigger index, start, length)

## Magic System

### Abilities (Active Magic)
- **Activated spells** from houses
- Components:
  - **Ability Name**: Identifier
  - **Description**: Explanation text
  - **Effect**: List of actions to execute

### Statuses (Passive Magic)
- **Temporary effects** applied to units
- Components:
  - **Status Name**: Identifier
  - **Description**: Effect explanation
  - **Behavior**: Additional reactions while active
  - **Representation**: Visual indicator (Material)
  - **Charges**: Duration/stack count

## Shop System

### Shop Mechanics
- **Shop Offers**: Available after each battle
- **Card Types**:
  - **Unit Cards**: Add new units to roster
  - **House Cards**: Unlock new houses
- **Shop Slots**:
  - **Price**: G cost to purchase
  - **Sold**: Whether already purchased
  - **Buy Limit**: Maximum purchases allowed
  - **Buy Text**: Custom purchase description

### Economy
- **Gold (G)**: Primary currency
- Income sources:
  - Battle victories
  - Selling units (unit_sell value)
- Expenses:
  - Buying units/houses
  - Rerolling shop offers

## Progression

### Floor System
- Players advance through numbered floors
- Each floor has:
  - **Floor Pools**: Regular enemy teams
  - **Floor Boss**: Powerful enemy team
- Difficulty increases with floor number

### Lives System
- Players start with limited lives
- Lose lives on battle defeat
- Match ends when lives reach 0
- Match is marked inactive on game over

## Variables (VarName)

### Combat Stats
- **pwr**: Power/attack damage
- **hp**: Hit points/health
- **dmg**: Current damage taken

### Positional
- **index**: Position in lineup
- **max_index**: Highest position
- **position**: Spatial coordinates
- **extra_position**: Additional positioning
- **offset**: Position modifier

### Identifiers
- **player_name**: Player identifier
- **unit_name**: Unit identifier
- **house_name**: House identifier
- **ability_name**: Ability identifier
- **status_name**: Status identifier

### Game State
- **g**: Gold currency
- **lives**: Remaining lives
- **floor**: Current floor
- **round**: Battle round
- **action_limit**: Max actions per turn

### Unit Properties
- **lvl**: Level
- **xp**: Experience
- **rarity**: Rarity tier
- **tier**: Power tier
- **stacks**: Unit count/multiplier
- **charges**: Ability/status uses
- **unit_size**: Visual scale
- **visible**: Display state

### Other
- **description**: Text description
- **color**: Color value
- **data**: Generic data field
- **value**: Generic value
- **t**: Time
- **text**: Display text
- **price**: Cost in gold
- **unit**: Unit reference
- **slot**: Slot position
- **side**: Team side

## Expressions

### Value Types
- **Constants**: one, zero, pi, pi2
- **Numbers**: i32, f32, bool
- **Strings**: string
- **Colors**: HexColor, oklch
- **Vectors**: vec2

### Target Selectors
- **owner**: Current unit
- **target**: Selected target
- **all_units**: Every unit in battle
- **all_enemy_units**: All opponents
- **all_ally_units**: All teammates
- **all_other_ally_units**: Teammates except self
- **adjacent_ally_units**: Neighboring allies
- **adjacent_front**: Unit in front
- **adjacent_back**: Unit behind

### Operations
- **Mathematical**: sum, sub, mul, div, mod, sqr, abs, floor, ceil
- **Trigonometric**: sin, cos
- **Logical**: and, or, equals, greater_then, less_then
- **Conditional**: if, fallback
- **Random**: rand, random_unit
- **Conversion**: to_f32, neg, even
- **Other**: min, max, fract, unit_vec

## Visual System

### Materials
- Units and effects use **PainterAction** sequences
- Define appearance and animations
- Can be modified by statuses

### Animations
- **strike**: Attack animation
- **death_vfx**: Death effect
- **spawn**: Entrance animation
- Status add/remove effects

## Team Building Strategy

### Fusion Composition
- Balance between high PWR (offense) and high HP (defense)
- Consider unit synergies within fusions
- Trigger selection determines active behaviors
- Action limits affect ability usage

### House Synergies
- Multiple units from same house strengthen abilities
- Mix houses for diverse magic options
- Consider ability and status combinations

### Positioning
- Front fusions engage first (tank role)
- Back fusions protected initially (damage dealers)
- Adjacent unit bonuses matter
- Death order affects battle flow

## Win Conditions

### Battle Victory
- Eliminate all enemy fusions
- Preserve your fusions
- Efficient trades (favorable PWR to HP ratios)

### Match Success
- Maintain lives through floors
- Build gold economy
- Upgrade team progressively
- Defeat floor bosses

## Advanced Mechanics

### Trigger Priority
- Earlier triggers in reaction list have priority
- Fusion uses selected unit's trigger
- Action ranges determine which actions execute

### Damage Calculation
1. Base damage = attacker's PWR
2. Apply ChangeOutgoingDamage modifiers
3. Apply ChangeIncomingDamage modifiers
4. Final damage applied to defender's HP

### Status Stacking
- Statuses can have multiple charges
- Each charge represents duration or intensity
- Some statuses stack, others refresh

### Event System
Events propagate through the game:
- **OutgoingDamage(source, target)**: Damage dealt
- **IncomingDamage(source, target)**: Damage received
- **StatChanged(entity, stat)**: Stat modification
- Custom events for special mechanics

## Configuration Notes

### Global Settings
- **team_slots**: Maximum fusions per team (typically 5)
- **match_g.unit_sell**: Gold gained from selling units
- Various balance parameters for economy and progression

### Technical Limits
- Maximum units per fusion determined by slot system
- Action limit prevents infinite loops
- Turn limits prevent stalled battles

---

*This document describes the core game rules and mechanics. Individual units, houses, and abilities may introduce unique mechanics that interact with these systems.*
