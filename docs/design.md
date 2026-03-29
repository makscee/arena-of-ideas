# Arena of Ideas — Game Design Document

## Vision

An auto-battler where the game's content evolves through player creativity and AI.
Players create abilities and units, the community votes, and the best content
enters the game. The game is never stale — it reinvents itself every season.

**One sentence:** Players breed abilities using AI, assemble units from them,
and the community decides what makes it into the game.

---

## Core Concepts

### Two Content Types

**Abilities** — the soul of the game:
- Named, shared mechanics with Rhai scripts
- Bred from two parent abilities + a player prompt
- AI generates the script, community votes
- ~20 active per season, top voted
- Evolution tree traces lineage to primordial set
- Multiple implementations can be proposed per name
- Locked once accepted for a season
- Old abilities can return with new implementations

**Units** — combinations of abilities:
- Trigger + Ability[] + Stats (hp, pwr) + Tier + Name + Painter script
- Assembled by players (pick trigger, pick abilities, pick tier)
- AI generates name + painter script from description
- Community votes, ~100 active
- No behavior script — abilities handle their own targeting and logic

### No Other Entity Types

No houses. No factions. No status effects as separate entities.
Abilities ARE the grouping mechanism — all units sharing "Steal Gold"
form a natural cluster. Statuses are effects that abilities create
(defined within ability scripts).

---

## Ability System

### Structure

```
Ability
├── id: u64
├── name: String              # "Steal Gold"
├── description: String       # "Deals damage and steals power from target"
├── target_type: TargetType   # RandomEnemy, AllEnemies, Owner, AllAllies, etc.
├── effect_script: String     # Rhai script (~3-15 lines)
├── parent_a: Option<u64>     # first parent ability (for evolution tree)
├── parent_b: Option<u64>     # second parent ability
├── rating: i32
├── status: ContentStatus     # Draft/Incubator/Active/Retired
├── created_by: Identity
├── created_at: Timestamp
└── season: u32               # which season this implementation belongs to
```

### Scaling

Abilities scale with two variables:
- **X** = the unit's `pwr` stat (how hard this unit hits with any ability)
- **level** = how many units on the team share this ability

Level thresholds:
- 1-2 units with ability → level 1 (X × 1)
- 3-4 units with ability → level 2 (X × 2)
- 5 units with ability → level 3 (X × 3)

Every ability on a multi-ability unit contributes to scaling independently.
A unit with [Steal Gold, Burning] counts toward both ability pools.

### Targeting

Target logic is baked into the ability, not the unit:

```rhai
// "Steal Gold" — offensive, targets opponent
ability_actions.deal_damage(target, X * level);
ability_actions.steal_stat(target, "pwr", level);
```

```rhai
// "Shield" — defensive, targets owner
ability_actions.add_shield(owner, X * level);
```

```rhai
// "War Cry" — support, targets all allies
for ally in ctx.get_allies(owner.id) {
    ability_actions.buff_stat(ally, "pwr", level);
}
```

The `target_type` field on the ability determines who receives the effect.
Players see this on the ability card: "Targets: Random Enemy" or "Targets: All Allies".

### Ability Creation (Breeding)

```
1. Player picks two parent abilities from the accepted pool
2. Writes a prompt: "combine into a fire theft that steals
   more the longer the target burns"
3. AI generates:
   - New ability name: "Ember Heist"
   - New effect script
   - Description
   - Suggested target type
4. Multiple players can breed the same pair with different prompts
5. All variants go to incubator
6. Community votes → winner enters the pool
```

### Ability Seasons

- Top 20 voted abilities define each season's pool
- For each ability name, the top-voted implementation is selected
- Old abilities can return in future seasons with new implementations
- An ability needs 5+ units using it to qualify for the season
- Between seasons: voting period where implementations compete

### Evolution Tree

All abilities trace lineage to a primordial set (~4-6 seed abilities):
- Strike (basic damage)
- Guard (basic defense)
- Heal (basic restoration)
- Curse (basic debuff)
- Maybe 1-2 more

The tree is visible and explorable. Players can see how "Ember Heist"
evolved from "Steal Gold" + "Burning", which evolved from "Strike" + "Curse", etc.

---

## Unit System

### Structure

```
Unit
├── id: u64
├── name: String              # "Cinderpaw"
├── description: String       # "A sneaky fire thief"
├── hp: i32
├── pwr: i32
├── tier: u8                  # 1-5
├── trigger: Trigger          # when this unit acts
├── abilities: Vec<u64>       # ability IDs (1-3)
├── painter_script: String    # Rhai visual script
├── rating: i32
├── status: ContentStatus
├── created_by: Identity
└── created_at: Timestamp
```

### Tier Rules

- Tier determines stat budget: tier × base_budget
- Tier determines ability count:
  - Tier 1-2: 1 ability
  - Tier 3-4: 2 abilities
  - Tier 5: 3 abilities
- Tier determines shop cost (same as current game)

### Unit Creation

```
1. Player picks trigger [dropdown from Trigger enum]
2. Player picks abilities [from active pool, count based on tier]
3. Player picks tier [determines stat budget]
4. AI generates:
   - Unique name (checks for duplicates)
   - Painter script (from name + abilities description)
5. Player can:
   - Edit name
   - Tweak visuals with sliders or prompt refinement
   - Adjust stats within tier budget
6. Submit to incubator → community votes
```

### Unit Rotation

- ~100 active units
- Top voted enter, lowest rated retire
- Rating decay over time forces turnover
- Tier distribution target: more low-tier, fewer high-tier

---

## Battle System

### Unit Behavior

When a unit's trigger fires:
1. All abilities execute in order
2. Each ability uses its own target_type to select targets
3. Each ability's script runs with X = unit's pwr, level = team ability count
4. BattleActions are queued and animated

### Triggers (from current game)

```
BattleStart, TurnEnd, BeforeDeath, AllyDeath,
BeforeStrike, AfterStrike, DamageTaken, DamageDealt,
StatusApplied, StatusGained, ChangeStat(VarName),
ChangeOutgoingDamage, ChangeIncomingDamage, Any(Vec<Trigger>)
```

### Target Types (available for abilities)

```
RandomEnemy, AllEnemies, RandomAlly, AllAllies,
Owner, All, Caster, Attacker, Target,
AdjacentBack, AdjacentFront,
AllyAtSlot(u8), EnemyAtSlot(u8)
```

### Team Size

5 unit slots per team.

---

## Shop & Match Economy

Carried over from current game:
- Gold-based economy
- Buy units, reroll shop, sell units
- Floors with increasing difficulty
- Lives system

### Stacking (Buying Duplicates)

- Buy same unit again → stat boost (added to existing unit)
- Collect 3 copies of same unit → unit becomes **fuseable**

---

## Fusion System

### Unlock

Collect 3 copies of any unit → it becomes fuseable.
The 3 copies merge into 1 fuseable unit with boosted stats.

### Fuse

Pick the fuseable unit + any other unit from your team:

```
Fuseable Unit A: BeforeStrike, [Steal Gold, Sneak], tier 2
Team Unit B:     TurnEnd, [Burning], tier 1

Fusion choices:
  Trigger: pick BeforeStrike OR TurnEnd
  Abilities: min(2, 1) + 1 = 2, pick from {Steal Gold, Sneak, Burning}
  Tier: max(2, 1) + 1 = 3
  Stats: hp = max(a,b) + min(a,b)/2, pwr = max(a,b) + min(a,b)/2

Result: Tier 3 fused unit with chosen trigger and 2 chosen abilities
```

### Rules

- **Ability count** = min(parent_a.abilities, parent_b.abilities) + 1
- **Tier** = max(parent_a.tier, parent_b.tier) + 1
- **One fusion level deep** — can't fuse two fused units
- Consumes both units, produces 1 fused unit
- Fused unit persists for the rest of the run (not permanent content)

### Feeding (Post-Fusion Upgrade)

After fusion, you can feed other units into the fused unit:
- Donor unit's abilities must ALL be a subset of the fused unit's abilities
- Result: stat boost, donor consumed
- No ability changes, just stronger stats

### Strategic Depth

Fusion creates meaningful decisions:
- Sacrifice 2 team slots for 1 stronger unit
- But lose ability scaling instances (fewer units sharing abilities)
- Higher tier = stronger base stats
- Picking the right trigger + ability combo is skill expression
- Same ability on different triggers = completely different behavior

---

## AI Generation

### Architecture

SpacetimeDB procedure with `ctx.http.fetch()` calls Claude API.
No sidecar service needed.

```
┌─────────┐    reducer      ┌──────────────┐    http.fetch    ┌───────────┐
│  Client  │───────────────▶│ SpacetimeDB  │────────────────▶│ Claude API│
│ (Bevy)   │◀───────────────│              │◀────────────────│           │
│          │   subscribe    │  Reducer:     │   response      └───────────┘
└─────────┘                 │   queue req   │
                            │  Procedure:   │
                            │   fetch AI    │
                            │   write result│
                            └──────────────┘
```

### What AI generates

**For abilities (breeding):**
- Name, description, target_type, effect_script
- Based on two parent abilities + player prompt
- Validated: Rhai compiles, uses only allowed API, under 20 lines

**For units (assembly):**
- Name (unique check), painter_script
- Based on selected trigger + abilities + player description
- Player manually picks trigger, abilities, tier, stats

### Rate Limiting

5 ability breeds + 10 unit generations per player per day.
Ability breeding costs more prompts than unit naming/visuals.

### Generation Tables

```
GenRequest
├── id, player, prompt, target_kind (Ability/Unit)
├── parent_a: Option<u64>     # for ability breeding
├── parent_b: Option<u64>     # for ability breeding
├── context_id: Option<u64>   # for refinement
├── status: Pending/Processing/Done/Failed
└── created_at

GenResult
├── id, request_id
├── data: String              # JSON
├── explanation: String       # AI reasoning
└── created_at
```

---

## Voting & Moderation

### Voting

- One vote per player per entity (can change vote)
- Upvote or downvote
- Rating = sum of votes
- Creator has special rights (can delete own content)

### Ability Voting

- Multiple implementations of same ability compete
- Between seasons: voting period picks winners
- Need 5+ units using the ability for it to qualify
- Top 20 abilities by rating enter the season

### Unit Voting

- Units go to incubator after creation
- Community votes
- Top ~100 by rating are active
- Must use only season-active abilities

### Balance Flags

- Players can flag units as "OP" or "weak"
- Threshold of flags → AI auto-generates rebalanced variant
- Rebalanced version enters incubator as new unit

### Feature Requests

- Players can propose abstract game changes
- "What if units could swap positions?"
- Community votes on proposals
- Passed proposals → developers implement in next game version
- This is how new triggers, target types, and Rhai API methods get added

---

## Seasons

### Season Structure

```
Off-season (creation period):
  - Ability implementations compete via voting
  - New abilities can be bred
  - Units created for new ability pool
  - Feature requests voted on

Season launch:
  - Top 20 abilities locked with winning implementations
  - Only units using those 20 abilities are eligible
  - Top 100 eligible units are active
  - New Rhai API features from passed feature requests ship

During season:
  - Unit rotation continues (new units enter, low-rated exit)
  - Ability implementations locked
  - Ability breeding continues for next season
  - Unit creation continues
```

### Content Lifecycle

```
Ability: Bred → Incubator → Voted → Season Pool → Active → Retired → Can Return
Unit:    Created → Incubator → Voted → Active (if abilities are in season) → Retired
```

---

## Tech Stack

```
Server:   SpacetimeDB 2.1 (Rust) — tables, reducers, procedures, AI calls
Client:   Bevy (Rust) — compiles to native + WASM (web support)
UI:       egui (via bevy_egui) — works in both native and WASM
Visuals:  Painter scripts → enum of painter actions → egui/Bevy renderer
Scripts:  Rhai — ability effect scripts, painter scripts
```

### Crate Structure (3 crates)

```
arena-of-ideas/
├── shared/          # Types shared between client and server
│   └── src/
│       ├── lib.rs
│       ├── ability.rs     # Ability struct
│       ├── unit.rs        # Unit struct
│       ├── battle.rs      # BattleAction enum
│       ├── trigger.rs     # Trigger enum (from current game)
│       ├── target.rs      # TargetType enum
│       ├── tier.rs        # Tier budgets
│       └── gen.rs         # GenRequest, GenResult
│
├── server/          # SpacetimeDB module
│   └── src/
│       ├── lib.rs         # init, tables
│       ├── auth.rs        # register, login
│       ├── content.rs     # ability + unit CRUD
│       ├── generation.rs  # AI procedure + prompts
│       ├── voting.rs      # votes, ratings
│       ├── season.rs      # rotation, ability selection
│       ├── match.rs       # shop, team, battle
│       └── admin.rs       # admin tools
│
├── client/          # Bevy + egui (native + WASM)
│   └── src/
│       ├── lib.rs
│       ├── plugins/
│       │   ├── connect.rs     # SpacetimeDB connection
│       │   ├── auth.rs        # login
│       │   ├── game.rs        # state machine
│       │   ├── shop.rs        # shop phase + fusion UI
│       │   ├── battle.rs      # simulation + rendering
│       │   ├── create.rs      # ability breeding + unit assembly UI
│       │   ├── incubator.rs   # browse, vote
│       │   ├── collection.rs  # browse active content + evolution tree
│       │   ├── rhai.rs        # script engine
│       │   ├── painter.rs     # visual rendering
│       │   ├── audio.rs       # sound
│       │   └── ui.rs          # shared UI utilities
│       ├── resources/
│       │   ├── game_state.rs
│       │   └── assets.rs
│       └── tests/
│           └── battle_test.rs
│
└── assets/
```

---

## Database Tables

```
abilities         — ability definitions with scripts and lineage
units             — unit definitions with trigger, ability refs, painter
votes             — player votes on abilities and units
gen_requests      — AI generation queue
gen_results       — AI generation output
players           — player accounts
matches           — active match state (shop, team, floor)
battle_results    — battle history and logs
feature_requests  — player proposals for game changes
seasons           — season metadata and active ability sets
```

**Total: ~10 tables**

---

## What's Eliminated vs Current Game

| Current (65k LOC, 8 crates) | New (~15-20k LOC, 3 crates) |
|------------------------------|------------------------------|
| 17+ node types | 2 content types (Ability, Unit) |
| PackedNodes graph | Flat structs |
| Build-time codegen (600KB) | None |
| Node links table | Foreign keys |
| Houses / factions | Emergent ability grouping |
| Status effects (separate type) | Part of ability scripts |
| Unit behavior scripts | Removed — abilities handle logic |
| Manual editor panes | Prompt + dropdowns + sliders |
| Client-side AI calls | Server-side procedures |
| RON serialization | JSON / direct serde |
| 30+ database tables | ~10 tables |

---

## Phase Plan

### Phase 0: Foundation
Workspace, auth, connection, empty Bevy app with game states.

### Phase 1: Content Tables + Seeds
Ability and Unit tables, seed primordial abilities, browsing UI.

### Phase 2: Battle Simulation
Rhai engine, ability execution, trigger pipeline, battle rendering.
**Hardest phase — the core game.**

### Phase 3: Shop + Match Loop
Full playable auto-battler with seeded content.
Stacking (duplicates), fusion, feeding.

### Phase 4: AI Generation
Server-side ability breeding via Claude procedure.
Unit name + visual generation. Prompt UI.

### Phase 5: Voting + Incubator
Community voting on abilities and units. Variant competition.

### Phase 6: Seasons + Rotation
Ability season selection, unit rotation, rating decay.

### Phase 7: Polish
Audio, animations, WASM build, migration, onboarding.

Phases 2-3 and Phase 4 can run in parallel.

---

## Decision Log

1. Rhai stays — perfect for AI generation, well-scoped API
2. SpacetimeDB stays — procedures enable server-side AI
3. Node graph system eliminated — flat tables, no codegen
4. Only two content types: Abilities and Units
5. No houses/factions — abilities ARE the grouping
6. Abilities are bred (pick 2 parents + prompt), units are assembled
7. Ability scaling via team count (level 1/2/3 at 1/3/5 units)
8. Target logic lives in ability scripts, not on units
9. Units = Trigger + Abilities + Stats (no behavior script)
10. Fusion: 3 copies → fuseable, combine with another unit
11. Fusion rules: min abilities + 1, max tier + 1, one level deep
12. Feeding: donor abilities must be subset of fused unit
13. ~20 abilities per season, top voted, need 5+ units each
14. ~100 active units, rating-based rotation
15. Seasons lock ability implementations, units rotate within
16. AI generation via SpacetimeDB procedures (ctx.http.fetch)
17. Rust + Bevy + egui, compile to native + WASM
18. Painter scripts use enum of actions (existing pattern)
19. Feature requests for new game mechanics voted by community
20. Limited AI prompts per player
