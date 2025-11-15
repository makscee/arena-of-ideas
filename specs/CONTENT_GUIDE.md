# Arena of Ideas - Content Creation Guide for AI

## Quick Reference

This guide helps AI assistants create balanced, interesting content for Arena of Ideas.

## Unit Creation Template

### Basic Structure
```
Unit Name: [Descriptive name]
House: [Which faction it belongs to]
Stats: PWR [0-10], HP [0-10]
Trigger: [When this unit activates]
Actions: [What it does when triggered]
Theme: [Flavor/concept]
```

### Power Levels
- **Common**: 3-5 total stats, simple effect
- **Rare**: 6-8 total stats, moderate effect
- **Epic**: 9-12 total stats, complex effect
- **Legendary**: 13+ total stats, unique mechanic

### Unit Archetypes

**Glass Cannon**
- High PWR (4-5), Low HP (1-2)
- Trigger: BattleStart or ChangeOutgoingDamage
- Example: "Berserker" - PWR 5, HP 1, doubles damage when below half HP

**Tank**
- Low PWR (1-2), High HP (5-8)
- Trigger: ChangeIncomingDamage or AllyDeath
- Example: "Guardian" - PWR 1, HP 7, reduces damage to allies

**Support**
- Balanced stats (PWR 2-3, HP 3-4)
- Trigger: TurnEnd or BattleStart
- Example: "Healer" - PWR 2, HP 3, heals lowest HP ally each turn

**Engine**
- Low stats but powerful effect
- Trigger: Specific condition
- Example: "Channeler" - PWR 1, HP 2, gains +1 PWR when ability used

## Ability Creation

### Templates

**Direct Damage**
```
set_value(i32([damage_amount]))
add_target(all_enemy_units)
deal_damage
```

**Targeted Buff**
```
add_target(all_ally_units)
repeat(var(stax), [
  add_value(i32(1))
])
apply_status
```

**Conditional Effect**
```
if(greater_then(var(hp), i32(5)),
  [use_ability],
  [deal_damage]
)
```

## Status Creation

### Status Types

**Buff** - Enhances unit
- Trigger: ChangeOutgoingDamage/ChangeStat
- Increases values
- Example: "Rage" - +2 damage per stack

**Debuff** - Weakens unit
- Trigger: ChangeIncomingDamage/TurnEnd
- Decreases values or deals damage
- Example: "Poison" - 1 damage per stack each turn

**Shield** - Protective
- Trigger: ChangeIncomingDamage
- Reduces/negates damage
- Example: "Barrier" - blocks next X damage

**Transform** - Changes behavior
- Trigger: Multiple events
- Modifies how unit works
- Example: "Frozen" - can't trigger abilities

## Synergy Patterns

### House Themes

**Fire House**
- Units: High damage, self-damage
- Ability: Area damage
- Status: Burn (damage over time)
- Synergy: Triggers on damage dealt

**Nature House**
- Units: Healing, growth
- Ability: Mass heal/buff
- Status: Regeneration
- Synergy: Scales with HP

**Shadow House**
- Units: Debuffs, life steal
- Ability: Steal stats
- Status: Curse (reverse healing)
- Synergy: Death triggers

**Lightning House**
- Units: Chain effects
- Ability: Random multi-hit
- Status: Static (spreads on hit)
- Synergy: Multiple triggers

## Balance Guidelines

### Stat Budgets
- Common: 4-5 total points
- Rare: 6-8 total points  
- Epic: 9-11 total points
- Legendary: 12+ total points

### Effect Costs
- Deal X damage: X points
- Heal X damage: X * 0.75 points
- Apply status: 2-3 points
- Add stats: 1 point per stat
- Complex trigger: -1 point cost

### Counter Relationships
- High PWR counters low HP
- High HP counters burst damage
- Multiple units counter single target
- Area effects counter multiple units
- Shields counter direct damage
- Status removal counters debuffs

## Common Pitfalls to Avoid

### Infinite Loops
- Always limit repeat counts
- Add action limits to fusions
- Don't create triggers that trigger themselves

### Overwhelming Complexity
- Max 3-4 actions per trigger
- Clear, understandable effects
- One main mechanic per unit

### No Counterplay
- Everything needs a counter
- Avoid permanent, unstoppable effects
- Leave room for opponent interaction

## Testing Checklist

Before finalizing content:
- [ ] Stats within budget?
- [ ] Effect has clear purpose?
- [ ] Synergizes with house theme?
- [ ] Has meaningful counterplay?
- [ ] Text is concise and clear?
- [ ] No infinite loops possible?
- [ ] Fun to play with AND against?

## Example Creation Process

**Goal**: Create a "Vampire" unit

1. **Theme**: Life-stealing attacker
2. **Stats**: PWR 3, HP 4 (7 total - rare)
3. **Trigger**: ChangeOutgoingDamage
4. **Effect**: Heal self for half damage dealt
5. **House**: Shadow (fits theme)
6. **Actions**:
   ```
   set_value(div(var(value), i32(2)))
   heal_damage
   ```
7. **Balance Check**: Strong vs low HP, weak vs high HP tanks ✓

## Quick Formulas

### Damage Over Time
```
repeat(var(stax), [
  set_value(i32(1))
  deal_damage
  wait(1.0)
])
```

### Percentage Boost
```
set_value(mul(var(value), div(sum(i32(100), var(stax)), i32(100))))
```

### Random Target
```
add_target(random_unit(all_enemy_units))
```

### Conditional Trigger
```
if(less_then(var(hp), div(var(max_hp), i32(2))), 
  [triggered_actions],
  [noop]
)
```
