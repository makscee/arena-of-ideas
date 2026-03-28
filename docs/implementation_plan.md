# Arena of Ideas — Implementation Plan

## Prerequisites

### Branch Setup
```
git checkout main
git checkout -b v2
# Delete all source code, keep git history
rm -rf schema/ server/ client/ utils/ utils-client/ proc-macros/ node-build-utils/
rm -rf src/
# Keep:
#   assets/          — SVGs, fonts, sounds (reuse)
#   docs/            — design docs
#   Cargo.toml       — will be rewritten
#   .github/         — CI (will be updated)
#   .claude/         — agent config
```

### New Workspace Cargo.toml
```toml
[workspace]
members = ["shared", "server", "client"]
resolver = "2"
```

---

## Phase 0: Skeleton + CI (Days 1-3)

**Goal:** Empty workspace compiles, tests run, CI passes on every push.

### 0.1 Create crate scaffolding
- [ ] `shared/` crate with `lib.rs`
- [ ] `server/` crate with `lib.rs`, SpacetimeDB dependency
- [ ] `client/` crate with `lib.rs`, Bevy + bevy_egui dependencies
- [ ] Verify `cargo build` succeeds for all crates
- [ ] Verify `cargo build --target wasm32-unknown-unknown -p client` succeeds (WASM)

### 0.2 CI pipeline
- [ ] GitHub Actions workflow: build + test on every push to v2 branch
- [ ] Steps: `cargo fmt --check`, `cargo clippy`, `cargo test --workspace`
- [ ] WASM build check: `cargo build --target wasm32-unknown-unknown -p client`

### 0.3 Test infrastructure
- [ ] `shared/src/lib.rs` with `#[cfg(test)] mod tests`
- [ ] `server/src/lib.rs` with test module
- [ ] `client/src/lib.rs` with test module
- [ ] First test: `#[test] fn it_compiles() {}` in each crate
- [ ] Verify `cargo test --workspace` passes

### Tests
- [ ] All three crates compile
- [ ] WASM target compiles
- [ ] CI runs green

---

## Phase 1: Shared Types (Days 3-6)

**Goal:** All game data types defined, serializable, tested.

### 1.1 Core enums
- [ ] `shared/src/trigger.rs` — Trigger enum (migrate from current schema/src/trigger.rs)
  - BattleStart, TurnEnd, BeforeDeath, AllyDeath, BeforeStrike, AfterStrike,
    DamageTaken, DamageDealt, StatusApplied, StatusGained, ChangeStat,
    ChangeOutgoingDamage, ChangeIncomingDamage, Any
- [ ] `shared/src/target.rs` — TargetType enum
  - RandomEnemy, AllEnemies, RandomAlly, AllAllies, Owner, All,
    Caster, Attacker, Target, AdjacentBack, AdjacentFront,
    AllyAtSlot(u8), EnemyAtSlot(u8)
- [ ] `shared/src/content_status.rs` — ContentStatus enum (Draft, Incubator, Active, Retired)

### 1.2 Content types
- [ ] `shared/src/ability.rs` — Ability struct
  - id, name, description, target_type, effect_script, parent_a, parent_b,
    rating, status, season
- [ ] `shared/src/unit.rs` — Unit struct
  - id, name, description, hp, pwr, tier, trigger, abilities (Vec<u64>),
    painter_script, rating, status
- [ ] `shared/src/tier.rs` — tier budget calculation, ability count per tier

### 1.3 Battle types
- [ ] `shared/src/battle.rs` — BattleAction enum (migrate from current)
  - damage, heal, death, spawn, apply_status, use_ability, var_set,
    vfx, wait, fatigue
- [ ] `shared/src/battle.rs` — BattleResult, BattleSide

### 1.4 Generation types
- [ ] `shared/src/gen.rs` — GenRequest, GenResult structs
  - request: player, prompt, target_kind, parent_a, parent_b, context_id, status
  - result: request_id, data (JSON), explanation

### Tests
- [ ] Trigger: serialize/deserialize roundtrip for every variant
- [ ] TargetType: serialize/deserialize roundtrip
- [ ] Ability: create, validate fields, serialize roundtrip
- [ ] Unit: create, validate tier budget (hp + pwr <= tier × base), ability count vs tier
- [ ] Unit: reject invalid tier/ability count combos
- [ ] BattleAction: serialize/deserialize roundtrip
- [ ] Tier: budget calculation returns correct values for each tier
- [ ] Tier: ability count limits correct for each tier

---

## Phase 2: SpacetimeDB Server — Tables + Auth (Days 6-10)

**Goal:** Server runs, players can register and login, content tables exist.

### 2.1 Database tables
- [ ] `server/src/lib.rs` — SpacetimeDB module init
- [ ] `server/src/tables.rs` — all table definitions:
  - players, abilities, units, votes, gen_requests, gen_results,
    matches, battle_results, seasons, feature_requests
- [ ] `server/src/auth.rs` — register, login_by_identity, logout reducers

### 2.2 Content reducers
- [ ] `server/src/content.rs` — CRUD:
  - ability_create, ability_update, ability_delete
  - unit_create, unit_update, unit_delete
  - Validation: tier budget, ability count, ability exists, name unique

### 2.3 Seed data
- [ ] `server/src/seed.rs` — init reducer seeds:
  - Primordial abilities: Strike, Guard, Heal, Curse (hand-written Rhai scripts)
  - 6-8 sample units using primordial abilities across tiers 1-3
  - Season 0 with all primordial abilities active

### Tests
- [ ] Auth: register creates player, login_by_identity finds player
- [ ] Content: create ability with valid data succeeds
- [ ] Content: create ability with empty script fails
- [ ] Content: create unit with valid data succeeds
- [ ] Content: create unit with too many abilities for tier fails
- [ ] Content: create unit with over-budget stats fails
- [ ] Content: create unit referencing non-existent ability fails
- [ ] Content: duplicate ability name fails
- [ ] Content: duplicate unit name fails
- [ ] Seed: init creates primordial abilities and sample units

---

## Phase 3: Bevy Client — Connect + Auth + Browse (Days 10-16)

**Goal:** Player can login, see seeded abilities and units in a browsing UI.

### 3.1 App skeleton
- [ ] `client/src/lib.rs` — Bevy app setup, plugin registration
- [ ] `client/src/plugins/game.rs` — GameState enum: Title, Login, Home, Shop, Battle, Create, Incubator
- [ ] State transitions with Bevy States

### 3.2 Connection
- [ ] `client/src/plugins/connect.rs` — SpacetimeDB connection plugin
  - Connect to server, handle identity, subscribe to tables

### 3.3 Auth
- [ ] `client/src/plugins/auth.rs` — Login UI
  - Register / login flow
  - Display player name on success
  - Transition to Home state

### 3.4 UI foundation
- [ ] `client/src/plugins/ui.rs` — shared UI utilities
  - Text styling, colors, layout helpers
  - Standard button/panel styles

### 3.5 Collection browser
- [ ] `client/src/plugins/collection.rs` — browse content
  - Ability list: name, description, target type, rating
  - Unit list: name, trigger, abilities, tier, stats, rating
  - Ability detail card with script (collapsible)
  - Unit detail card with visual preview
  - Filter by: tier, ability, status (active/incubator)

### Tests
- [ ] UI test: GameState transitions (Title → Login → Home)
- [ ] UI test: Collection browser renders with mock data
- [ ] Integration: client connects to local SpacetimeDB server
- [ ] Integration: login flow creates player and transitions to Home
- [ ] Integration: collection shows seeded abilities and units

---

## Phase 4: Rhai Engine + Battle Simulation (Days 16-28)

**Goal:** Two teams of units fight. Scripts execute. Battle plays out visually.

This is the hardest and most critical phase.

### 4.1 Rhai engine setup
- [ ] `client/src/plugins/rhai.rs` — Rhai script engine plugin
  - Initialize engine with sandbox (no file I/O, no network)
  - Register types: unit properties (.id, .hp, .pwr, .dmg)
  - Register context: ctx.get_enemies(), ctx.get_allies(), ctx.get_all_units()
  - Register ability_actions: deal_damage, heal_damage, steal_stat, add_shield,
    buff_stat, change_status (expand as needed)
  - Expose X (pwr) and level (team ability count) as script variables

### 4.2 Battle simulation core
- [ ] `client/src/resources/battle.rs` — BattleSimulation
  - Initialize from two teams of units (Vec<Unit> + resolved Ability data)
  - Calculate ability levels per team (count units sharing each ability)
  - Turn loop:
    1. Check each unit's trigger condition
    2. For triggered units: execute each ability in order
    3. Ability selects targets via its target_type
    4. Ability script runs with (X, level, owner, target, ctx)
    5. Script produces ability_actions → converted to BattleActions
    6. Apply BattleActions (damage, heal, death, etc.)
    7. Check deaths, remove dead units
    8. Check win condition (one team eliminated)
  - Produce Vec<BattleAction> as battle log
  - Fatigue timer: if battle exceeds N turns, both teams take increasing damage

### 4.3 Battle renderer
- [ ] `client/src/plugins/battle.rs` — battle visualization
  - Display two teams facing each other
  - Animate BattleActions sequentially (damage numbers, HP bars, death)
  - Unit painter scripts render each unit's visual
  - Skip/fast-forward button

### 4.4 Painter system
- [ ] `client/src/plugins/painter.rs` — execute painter scripts
  - Painter action enum (circle, rect, line, text, color, offset, rotate, scale)
  - Rhai binds painter actions
  - Render painter actions to egui or Bevy 2D

### 4.5 Battle test mode
- [ ] `client/src/plugins/battle.rs` — test fight UI
  - Pick any two units → run battle → watch result
  - Accessible from collection browser

### Tests (unit tests)
- [ ] Rhai: ability script with deal_damage produces correct BattleAction
- [ ] Rhai: ability script with heal_damage produces correct BattleAction
- [ ] Rhai: ability script can read owner.hp, owner.pwr
- [ ] Rhai: ability script can read target.hp, target.pwr
- [ ] Rhai: ctx.get_enemies() returns correct units
- [ ] Rhai: ctx.get_allies() returns correct units
- [ ] Rhai: X equals unit's pwr
- [ ] Rhai: level equals team ability count at correct thresholds (1/3/5)
- [ ] Rhai: invalid script returns compile error, doesn't crash
- [ ] Rhai: infinite loop in script times out gracefully

### Tests (battle simulation)
- [ ] Battle: unit with BeforeStrike trigger fires ability before attacking
- [ ] Battle: unit with TurnEnd trigger fires at end of turn
- [ ] Battle: unit with DamageTaken trigger fires when hit
- [ ] Battle: ability with RandomEnemy target picks a living enemy
- [ ] Battle: ability with AllEnemies target hits all living enemies
- [ ] Battle: ability with Owner target affects the casting unit
- [ ] Battle: ability with AllAllies target affects all allies
- [ ] Battle: unit dies when hp <= dmg
- [ ] Battle: dead units don't trigger
- [ ] Battle: battle ends when one team is eliminated
- [ ] Battle: fatigue kicks in after N turns
- [ ] Battle: ability level 1 at 1-2 units, level 2 at 3-4, level 3 at 5
- [ ] Battle: multi-ability unit fires all abilities on trigger
- [ ] Battle: deterministic — same input produces same output

### Tests (painter)
- [ ] Painter: script producing circle action renders without crash
- [ ] Painter: empty script renders default placeholder
- [ ] Painter: invalid script shows error, doesn't crash

---

## Phase 5: Shop + Match Loop (Days 28-38)

**Goal:** Full playable auto-battler: shop → team → battle → repeat.

### 5.1 Match server reducers
- [ ] `server/src/match.rs`:
  - match_start — create match, generate shop offers from active units
  - match_shop_buy — spend gold, add unit to team (max 5 slots)
  - match_shop_reroll — pay gold, new random offers
  - match_sell_unit — remove from team, refund gold
  - match_move_unit — reorder team slots
  - match_start_battle — generate opponent team, record battle
  - match_submit_result — apply outcome (gold, lives, floor advance)
  - match_abandon — quit run

### 5.2 Stacking
- [ ] `server/src/match.rs` — match_stack_unit
  - Buy duplicate → stat boost to existing unit
  - Track copy count per unit
  - At 3 copies → unit becomes fuseable (flag on team slot)

### 5.3 Fusion
- [ ] `server/src/match.rs` — match_fuse_units
  - Validate: unit A is fuseable (3 copies stacked)
  - Validate: unit B is on team
  - Player submits: chosen trigger, chosen abilities
  - Validate: trigger is from A or B
  - Validate: abilities count = min(A.count, B.count) + 1
  - Validate: all chosen abilities are from A or B's ability pools
  - Compute: tier = max(A.tier, B.tier) + 1
  - Compute: stats from tier budget (or combined with penalty TBD)
  - Create fused unit on team, remove A and B

### 5.4 Feeding
- [ ] `server/src/match.rs` — match_feed_unit
  - Validate: target is a fused unit
  - Validate: donor's abilities are ALL a subset of target's abilities
  - Apply: stat boost to target, remove donor

### 5.5 Shop client UI
- [ ] `client/src/plugins/shop.rs`:
  - Display shop offers (unit cards)
  - Buy button (deducts gold)
  - Team display (5 slots, drag to reorder)
  - Sell button on team units
  - Reroll button with cost
  - Gold counter
  - Fusion UI: select fuseable unit + another → show choices → confirm
  - Feed UI: drag donor onto fused unit → confirm
  - "Start Battle" button
  - Floor / lives display

### 5.6 Match flow
- [ ] `client/src/plugins/game.rs` — match state machine
  - Home → match_start → Shop → battle → result → Shop → ... → game over
  - Victory/defeat screen with stats
  - Return to Home

### Tests (server)
- [ ] Match: start creates match with correct initial gold and empty team
- [ ] Match: buy adds unit to team, deducts gold
- [ ] Match: buy fails when not enough gold
- [ ] Match: buy fails when team is full (5 slots)
- [ ] Match: sell removes unit, refunds gold
- [ ] Match: reroll replaces shop offers, deducts gold
- [ ] Match: move_unit swaps slots correctly
- [ ] Match: submit_result advances floor and awards gold on win
- [ ] Match: submit_result deducts life on loss
- [ ] Match: game over when lives reach 0
- [ ] Stack: buying duplicate boosts stats
- [ ] Stack: 3rd copy makes unit fuseable
- [ ] Fusion: valid fusion produces correct tier and ability count
- [ ] Fusion: fusion with wrong trigger source fails
- [ ] Fusion: fusion with too many abilities fails
- [ ] Fusion: fusion with ability not from either parent fails
- [ ] Fusion: can't fuse non-fuseable unit
- [ ] Fusion: fused unit can't be fused again
- [ ] Feed: valid feed boosts stats and removes donor
- [ ] Feed: feed fails when donor has ability not in target
- [ ] Feed: feed fails when target is not fused

### Tests (client UI)
- [ ] Shop: renders offers and team slots
- [ ] Shop: buy updates team and gold display
- [ ] Shop: fusion UI shows correct trigger and ability choices
- [ ] Match: state transitions Shop → Battle → Result → Shop

---

## Phase 6: AI Generation (Days 38-50)

**Goal:** Players breed abilities and create units via AI prompts.

### 6.1 AI procedure
- [ ] `server/src/generation.rs`:
  - System prompt constant: Rhai API reference, trigger/target docs,
    tier budgets, output format spec, few-shot examples
  - process_ability_breeding procedure:
    1. Load parent_a and parent_b ability data
    2. Build prompt: parents + player prompt + system prompt
    3. Call Claude via ctx.http.fetch()
    4. Parse JSON response: name, description, target_type, effect_script
    5. Validate: Rhai compiles, target_type is valid, name is unique
    6. Write GenResult
  - process_unit_generation procedure:
    1. Load selected abilities
    2. Build prompt: abilities + trigger + tier + player description
    3. Call Claude for name + painter_script
    4. Validate: name unique, painter script compiles
    5. Write GenResult

### 6.2 Generation reducers
- [ ] `server/src/generation.rs`:
  - gen_breed_ability — validates parents exist, creates GenRequest
  - gen_create_unit — validates abilities + trigger, creates GenRequest
  - gen_accept_result — player accepts, creates ability/unit in incubator
  - gen_reject_result — player rejects, can retry with new prompt

### 6.3 Creation UI
- [ ] `client/src/plugins/create.rs`:
  - **Ability breeding tab:**
    - Browse ability pool, pick two parents
    - Show parent ability cards side by side
    - Prompt input: "how should these combine?"
    - Loading spinner while AI generates
    - Result: new ability card with name, description, script, explanation
    - "Accept" → submits to incubator
    - "Refine" → new prompt, same parents
    - "Reject" → discard
  - **Unit assembly tab:**
    - Trigger dropdown
    - Ability picker (from active pool, count limited by tier)
    - Tier selector
    - Stat sliders (within budget)
    - "Generate name & visual" button → AI call
    - Preview: unit card with painter script visual
    - Manual painter script editor (advanced, collapsible)
    - Visual sliders (color, size — modify painter script params)
    - "Submit" → to incubator

### Tests (server)
- [ ] Generation: breed request with valid parents creates GenRequest
- [ ] Generation: breed request with non-existent parent fails
- [ ] Generation: breed request with same parent for both fails
- [ ] Generation: unit request with invalid ability count fails
- [ ] Generation: accept result creates ability in incubator
- [ ] Generation: accept result creates unit in incubator
- [ ] Generation: reject result marks request as rejected
- [ ] Validation: generated Rhai script that compiles passes validation
- [ ] Validation: generated Rhai script that doesn't compile fails
- [ ] Validation: duplicate ability name is rejected

### Tests (AI integration — can mock HTTP)
- [ ] Procedure: builds correct prompt with parent ability data
- [ ] Procedure: parses valid Claude response into GenResult
- [ ] Procedure: handles Claude API error gracefully
- [ ] Procedure: handles malformed Claude response gracefully
- [ ] Procedure: handles timeout gracefully

### Tests (client UI)
- [ ] Create: ability breeding shows two parent picker panels
- [ ] Create: unit assembly shows trigger dropdown and ability picker
- [ ] Create: prompt input is disabled while generation is processing
- [ ] Create: result display shows ability/unit card after generation

---

## Phase 7: Voting + Incubator (Days 50-58)

**Goal:** Community votes on content, best rises to the top.

### 7.1 Voting reducers
- [ ] `server/src/voting.rs`:
  - vote_cast — one vote per player per entity, can change
  - vote_retract — remove vote
  - Rating update: recalculate on every vote
  - Validation: can't vote on own content? (design choice)
  - Validation: can't vote on Draft content (only Incubator+)

### 7.2 Incubator UI
- [ ] `client/src/plugins/incubator.rs`:
  - Browse incubator content sorted by rating
  - Filter: abilities / units, tier, ability used
  - Ability cards: name, description, script (collapsible), parent lineage, rating
  - Unit cards: name, visual preview, trigger, abilities, stats, rating
  - Vote buttons (up/down) with current rating
  - "Breed from this" → jumps to create tab with this ability as parent
  - Battle preview: test incubator unit vs active unit

### 7.3 Evolution tree view
- [ ] `client/src/plugins/collection.rs` — add tree visualization
  - Show ability lineage as interactive tree
  - Click node → see ability detail
  - Highlight active abilities vs incubator vs retired

### Tests (server)
- [ ] Vote: cast upvote increases rating
- [ ] Vote: cast downvote decreases rating
- [ ] Vote: change vote from up to down adjusts rating by 2
- [ ] Vote: retract removes vote effect
- [ ] Vote: can't vote on Draft content
- [ ] Vote: one vote per player per entity enforced
- [ ] Vote: voting on non-existent entity fails

### Tests (client UI)
- [ ] Incubator: renders list sorted by rating
- [ ] Incubator: vote buttons update rating display
- [ ] Incubator: filter by abilities works
- [ ] Evolution tree: renders primordial → children → grandchildren

---

## Phase 8: Seasons + Rotation (Days 58-68)

**Goal:** Abilities rotate by season, units rotate continuously.

### 8.1 Season logic
- [ ] `server/src/season.rs`:
  - Season table: id, start_date, end_date, active_ability_ids
  - season_start procedure:
    1. Count units per ability (only abilities with 5+ units qualify)
    2. Rank qualifying abilities by rating
    3. Top 20 → Active, rest → Retired
    4. Lock ability implementations for the season
    5. Mark units using non-active abilities as ineligible
    6. From eligible units, top 100 by rating → Active
  - Rating decay: scheduled procedure reduces active unit ratings over time
  - season_end: unlock abilities, open voting for next season

### 8.2 Unit rotation (within season)
- [ ] `server/src/season.rs`:
  - Periodic check (daily or on-demand):
    - Incubator units above threshold + using active abilities → promote
    - Active units below threshold → demote
    - Maintain ~100 active units

### 8.3 Balance flags
- [ ] `server/src/voting.rs`:
  - flag_overpowered / flag_underpowered reducers
  - When flags exceed threshold → auto-create GenRequest for AI rebalance
  - AI suggests stat adjustment → enters incubator as variant

### 8.4 Season UI
- [ ] `client/src/plugins/collection.rs` updates:
  - "Season X" header with active abilities listed
  - "New this week" section for recently promoted units
  - "Retiring soon" for low-rated active units
  - "Vote to keep" button
  - Season countdown timer

### Tests (server)
- [ ] Season: abilities with 5+ units qualify
- [ ] Season: abilities with <5 units don't qualify
- [ ] Season: top 20 abilities selected by rating
- [ ] Season: units using non-active abilities become ineligible
- [ ] Season: top 100 eligible units become active
- [ ] Rotation: unit above threshold in incubator gets promoted
- [ ] Rotation: unit below threshold in active gets demoted
- [ ] Rotation: rating decay reduces ratings each period
- [ ] Balance: flag_overpowered increments flag count
- [ ] Balance: threshold reached creates rebalance GenRequest

---

## Phase 9: Feature Requests (Days 68-73)

**Goal:** Players propose game changes, community votes, devs implement between seasons.

### 9.1 Feature request system
- [ ] `server/src/feature_requests.rs`:
  - feature_request_create — player submits text proposal
  - Reuses voting system (vote_cast with entity_kind = FeatureRequest)
  - feature_request_accept — admin marks as accepted (queued for dev)
  - feature_request_reject — admin marks as rejected with reason

### 9.2 Feature request UI
- [ ] `client/src/plugins/incubator.rs` — add feature request tab:
  - Submit proposal text
  - Browse proposals sorted by rating
  - Vote on proposals
  - Status indicator: proposed / accepted / in-progress / shipped / rejected

### Tests
- [ ] Feature request: create with text succeeds
- [ ] Feature request: voting works same as content voting
- [ ] Feature request: admin accept/reject changes status
- [ ] Feature request: non-admin can't accept/reject

---

## Phase 10: Polish + Launch Prep (Days 73-85)

**Goal:** Production quality, web build, onboarding.

### 10.1 WASM web build
- [ ] Configure Bevy for WASM target
- [ ] Set up wasm-bindgen / trunk / wasm-pack build pipeline
- [ ] Test in browser: Chrome, Firefox, Safari
- [ ] Handle SpacetimeDB WebSocket connection in WASM
- [ ] Fix any WASM-specific issues (audio, rendering, input)

### 10.2 Audio
- [ ] Migrate audio assets from current game
- [ ] `client/src/plugins/audio.rs` — sound effects for:
  - Buy, sell, reroll, fusion, battle hits, death, victory, defeat

### 10.3 Visual polish
- [ ] Battle animations: smooth transitions, damage numbers
- [ ] Screen transitions between game states
- [ ] Shop UI polish: drag/drop feedback, gold animations
- [ ] Loading states and error messages

### 10.4 Onboarding
- [ ] First-time player tutorial:
  1. Show shop, explain buying
  2. Run a battle with pre-built team
  3. Guide to breed first ability
  4. Explain voting
- [ ] Tooltip system for triggers, abilities, keywords

### 10.5 Admin tools
- [ ] `client/src/plugins/admin.rs`:
  - Manual promote/demote content
  - Manual season start/end
  - Player moderation
  - Content deletion

### 10.6 Error handling
- [ ] Network disconnection recovery
- [ ] AI generation timeout / failure UI
- [ ] Invalid state recovery in match flow

### Tests
- [ ] WASM: game loads in browser
- [ ] WASM: can login and browse content
- [ ] WASM: battle renders and plays
- [ ] Audio: sounds play on correct events
- [ ] Onboarding: tutorial flow completes without errors
- [ ] Error: network disconnect shows reconnection UI
- [ ] Error: AI timeout shows retry option

---

## Phase 11: Migration + Data Seeding (Days 85-90)

**Goal:** Migrate best content from old game, seed a healthy starting state.

### 11.1 Content migration
- [ ] Script to read old PackedNodes format and extract:
  - Best ability scripts → convert to new Ability format
  - Best unit definitions → convert to new Unit format
- [ ] Review and hand-curate migrated content
- [ ] Seed primordial evolution tree with migrated abilities as descendants

### 11.2 Initial game state
- [ ] Create Season 1 with curated 20 abilities
- [ ] Ensure each ability has 5+ units
- [ ] Verify all units battle correctly via headless tests
- [ ] Balance pass: run automated tournaments, flag outliers

### Tests
- [ ] Migration: old ability scripts compile in new Rhai engine
- [ ] Migration: old units load correctly with new flat format
- [ ] Headless: all 100 active units can complete a battle without errors
- [ ] Headless: no unit has >70% or <30% win rate (reasonable balance)

---

## Testing Strategy Summary

### Test Types

| Type | Where | What | When |
|------|-------|------|------|
| **Unit tests** | `shared/`, `server/`, `client/` | Individual functions, validation, serialization | Every phase |
| **Integration tests** | `server/` | Reducer chains, multi-table operations | Phase 2+ |
| **Battle tests** | `client/` | Simulation correctness, script execution | Phase 4+ |
| **Headless tests** | `client/` | Full battles with real content, catch crashes | Phase 4+ |
| **UI tests** | `client/` | State transitions, render with mock data | Phase 3+ |
| **CI** | GitHub Actions | All of the above on every push | Phase 0+ |

### Test Count Targets

| Phase | New Tests | Cumulative |
|-------|-----------|------------|
| 0 | 3 | 3 |
| 1 | ~15 | ~18 |
| 2 | ~12 | ~30 |
| 3 | ~8 | ~38 |
| 4 | ~25 | ~63 |
| 5 | ~22 | ~85 |
| 6 | ~18 | ~103 |
| 7 | ~10 | ~113 |
| 8 | ~12 | ~125 |
| 9 | ~5 | ~130 |
| 10 | ~8 | ~138 |
| 11 | ~5 | ~143 |

### Headless Battle Testing

From Phase 4 onward, every CI run should:
1. Seed test abilities and units
2. Run N headless battles
3. Assert: no crashes, no infinite loops, all battles complete
4. Assert: battle results are deterministic (same seed = same result)

---

## Timeline Summary

| Phase | Days | What | Milestone |
|-------|------|------|-----------|
| 0 | 1-3 | Skeleton + CI | Workspace compiles, tests run |
| 1 | 3-6 | Shared types | All data types defined and tested |
| 2 | 6-10 | Server tables + auth | Player can register, content seeded |
| 3 | 10-16 | Client connect + browse | Player can login and browse content |
| 4 | 16-28 | Battle simulation | Two teams can fight (**hardest**) |
| 5 | 28-38 | Shop + match + fusion | Full playable auto-battler |
| 6 | 38-50 | AI generation | Ability breeding + unit creation via AI |
| 7 | 50-58 | Voting + incubator | Community curation works |
| 8 | 58-68 | Seasons + rotation | Content lifecycle complete |
| 9 | 68-73 | Feature requests | Player proposals for game evolution |
| 10 | 73-85 | Polish + WASM | Web build, audio, onboarding |
| 11 | 85-90 | Migration + seeding | Launch-ready with real content |

**Total: ~90 working days**

Phases 4-5 (battle + shop) and Phase 6 (AI generation) can overlap
since they're independent systems after Phase 2 is done.

---

## Branch Strategy

```
main          — current live game (untouched)
v2            — full rewrite (this plan)

Phase 0: create v2 branch, delete old source, scaffold new workspace
Phases 1-11: all work on v2 branch
Launch: merge v2 → main when ready
```

All commits to v2 go through CI. No merging broken code.
Old game stays playable on main throughout the rewrite.
