# Fix Tasks — Addressing the Roast

## Priority 1: Core Features That Don't Work

### 1.1 Wire up voting UI to server
- [ ] `client/src/plugins/incubator.rs`: Replace all 4 vote button TODOs with actual `vote_cast` reducer calls
- [ ] `client/src/plugins/collection.rs`: Add vote buttons to collection view too
- [ ] Test: integration test that votes from UI flow through to rating changes

### 1.2 Wire up ability breeding to server
- [ ] `client/src/plugins/create.rs`: Replace breeding TODO with actual `gen_breed_ability` reducer call
- [ ] Handle pending state: subscribe to `gen_result` table, show result when ready
- [ ] Replace unit creation TODO with `gen_create_unit` + `unit_create` reducer calls
- [ ] Test: integration test for full breed flow (pick parents → prompt → result → accept)

### 1.3 Wire up content creation to incubator
- [ ] Accept button in create.rs: call `ability_create` with status=Incubator
- [ ] Unit submit: call `unit_create` with status=Incubator
- [ ] Test: create ability via AI → accept → appears in incubator → vote on it

### 1.4 Connect client to SpacetimeDB (replace mock data)
- [ ] Generate client bindings: `spacetime generate --lang rust`
- [ ] `client/src/plugins/connect.rs`: Implement real SpacetimeDB connection
- [ ] Replace `GameContent` mock data with SpacetimeDB table subscriptions
- [ ] Test: client connects, subscribes, sees real seeded data

## Priority 2: Missing Game Logic

### 2.1 Decide and implement fusion stat formula
- [ ] Define formula: e.g., `hp = max(a.hp, b.hp) + min(a.hp, b.hp) / 2`, same for pwr
- [ ] Implement in `server/src/match_reducer.rs` `match_fuse_units`
- [ ] Update `docs/design.md` — remove "TBD"
- [ ] Test: fusion produces expected stats for various input combinations

### 2.2 Implement prompt rate limiting
- [ ] Decide limit: e.g., 5 ability breeds + 10 unit generations per day
- [ ] Add `gen_request_count` tracking per player per day in server
- [ ] Reject requests over limit with clear error
- [ ] Update `docs/design.md` — remove "TBD"
- [ ] Test: player hits limit, subsequent requests rejected

### 2.3 Add combined trigger option to fusion
- [ ] Fusion UI: offer 3 choices for trigger (from A, from B, combine both)
- [ ] "Combine" creates `Trigger::Any(vec![a_trigger, b_trigger])`
- [ ] Server: accept `trigger_choice: String` with values "a", "b", "both"
- [ ] Test: fused unit with combined trigger fires on either condition

### 2.4 Shop randomization
- [ ] Current: deterministic cycling (offers same units every reroll)
- [ ] Add server-side RNG seeded per match for shop offers
- [ ] Weight by tier based on floor (higher floors → more high-tier offers)
- [ ] Test: reroll produces different offers than original

## Priority 3: Error Handling & Robustness

### 3.1 Remove panics from Rhai engine
- [ ] `client/src/plugins/rhai_engine.rs`: Replace all `panic!()` in tests with proper assertions
- [ ] Return `Result` from script execution instead of panicking on bad input
- [ ] Battle simulation: gracefully skip abilities with broken scripts, log warning
- [ ] Test: battle with invalid script completes without crash

### 3.2 Handle AI generation failures gracefully
- [ ] Client: show error message when gen_result has failure status
- [ ] Server: validate Rhai script compiles before accepting gen_result
- [ ] Timeout: if no result after 60s, show "try again" button
- [ ] Test: submit invalid script as gen_result → rejected with error

### 3.3 Graceful handling of missing data
- [ ] Battle: unit references non-existent ability → skip ability, log warning
- [ ] Shop: unit not found → show empty slot instead of crashing
- [ ] Collection: handle empty content gracefully (show "no content yet" message)

## Priority 4: Test Quality

### 4.1 Add gameplay logic tests (not just serde)
- [ ] Test: unit with "Steal Gold" script actually reduces target pwr and adds to owner
- [ ] Test: unit with "Guard" script adds shield that absorbs next damage
- [ ] Test: healing doesn't exceed max hp
- [ ] Test: dead unit's abilities don't fire
- [ ] Test: ability level correctly calculated with mixed ability teams

### 4.2 Add negative path tests
- [ ] Test: buy when broke → fails with "not enough gold"
- [ ] Test: sell from empty team → fails
- [ ] Test: fuse non-fuseable unit → fails
- [ ] Test: feed with non-subset abilities → fails
- [ ] Test: start battle during battle → fails
- [ ] Test: register with empty name → fails
- [ ] Test: create ability with 3000-char script → fails (max 2000)

### 4.3 Add battle edge case tests
- [ ] Test: 0 pwr unit deals 0 damage
- [ ] Test: battle with all healers ends via fatigue
- [ ] Test: 1v5 battle (outnumbered) produces correct result
- [ ] Test: fused unit with combined trigger fires on both trigger types

### 4.4 Replace CLI string matching with structured assertions
- [ ] Create helper: `fn query_json(table: &str) -> Vec<serde_json::Value>`
- [ ] Parse SpacetimeDB SQL output into structured data
- [ ] Replace `contains(&output, "| 2")` with `assert_eq!(match_data.floor, 2)`
- [ ] Makes tests less brittle and more readable

## Priority 5: Polish

### 5.1 Clean up warnings
- [ ] Fix all `dead_code` warnings in server (unused prompt constants)
- [ ] Remove `#[allow(unused_imports)]` — use the imports or remove them
- [ ] Run `cargo clippy` and fix all warnings
- [ ] Add `#![warn(clippy::all)]` to all crate roots

### 5.2 Evolution tree visualization
- [ ] Fetch real ability data with parent_a/parent_b from server
- [ ] Render tree: primordial abilities at root, children below
- [ ] Click ability node → show details
- [ ] Visual: lines connecting parents to children

### 5.3 Battle replay UI
- [ ] Currently: battle simulation runs and produces `Vec<BattleAction>`
- [ ] Add: render actions step by step with timing
- [ ] Show: damage numbers, HP bars, death animations
- [ ] Skip/fast-forward button

### 5.4 Onboarding
- [ ] First-time player sees tutorial overlay
- [ ] Step 1: "Buy a unit from the shop"
- [ ] Step 2: "Start a battle"
- [ ] Step 3: "Try breeding an ability"
- [ ] Show tooltips on hover for triggers, abilities, tier budgets

### 5.5 Update old integration tests
- [ ] `client/tests/harness.rs`: Update to use new match state (MatchState, 7g start)
- [ ] `client/tests/gameplay_flow.rs`: Update for new floor pool / boss mechanics
- [ ] Remove duplicated test coverage between harness, gameplay_flow, and match_flow
- [ ] Consolidate into 2 test files: `unit_tests` and `integration_tests`

## Priority 6: Architecture Improvements

### 6.1 DRY up Rhai engine
- [ ] Extract common script test pattern into helper function
- [ ] Reuse engine instance across tests (currently creates new per test)
- [ ] Consider caching compiled ASTs for known ability scripts

### 6.2 Structured SpacetimeDB test client
- [ ] Create `StdbTestClient` struct that wraps CLI calls
- [ ] Methods: `client.call_reducer(name, args)`, `client.query(table)`
- [ ] Returns parsed data, not raw strings
- [ ] Auto-cleanup on drop (abandon match, etc.)

### 6.3 Separate battle simulation from Bevy
- [ ] Move `simulate_battle` to `shared/` crate (it doesn't need Bevy)
- [ ] Rhai engine setup stays in client, but simulation logic is shared
- [ ] Enables server-side battle validation in the future

### 6.4 Make economy configurable
- [ ] Move all magic numbers (7g start, 1g sell, 3g reward, etc.) to a config struct
- [ ] Store in SpacetimeDB table (like old GlobalSettings)
- [ ] Admin reducer to update economy values without republishing

## Summary

| Priority | Tasks | Impact |
|----------|-------|--------|
| P1: Wire up core features | 4 | Game actually works end-to-end |
| P2: Missing game logic | 4 | Fusion, shop, rate limits work properly |
| P3: Error handling | 3 | Game doesn't crash on bad input |
| P4: Test quality | 4 | Tests catch real bugs, not just serde |
| P5: Polish | 5 | Game looks and feels complete |
| P6: Architecture | 4 | Code is maintainable long-term |
| **Total** | **24** | |
