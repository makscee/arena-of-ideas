# AI Game Testing Research - Deep Dive Findings

## 1. Bevy Headless Mode & Testing (Proven, Production-Ready)

### Headless Execution (No GPU Required)
Bevy natively supports headless mode. Disable `default-features` in Cargo.toml:

```toml
[dependencies]
bevy = { version = "0.18", default-features = false }
```

Use `ScheduleRunnerPlugin` to run without a window:
```rust
use bevy::{app::ScheduleRunnerPlugin, prelude::*};
use core::time::Duration;

App::new()
    .add_plugins(DefaultPlugins.set(
        ScheduleRunnerPlugin::run_loop(Duration::from_secs_f64(1.0 / 60.0))
    ))
    .run();
```

Run: `cargo run --example headless --no-default-features --features bevy_log`

Source: https://github.com/bevyengine/bevy/blob/main/examples/app/headless.rs

### Bevy ECS Unit Testing (Official Pattern)
Bevy's official test pattern creates a minimal `App`, inserts resources, spawns entities, calls `app.update()`, then asserts on world state:

```rust
#[test]
fn did_hurt_enemy() {
    let mut app = App::new();
    app.insert_resource(Score(0));
    app.add_message::<EnemyDied>();
    app.add_systems(Update, (hurt_enemies, despawn_dead_enemies).chain());

    let enemy_id = app.world_mut().spawn(Enemy { hit_points: 5, score_value: 3 }).id();
    app.update();

    assert_eq!(app.world().get::<Enemy>(enemy_id).unwrap().hit_points, 4);
}
```

Key: Simulated input works by injecting `ButtonInput<KeyCode>` as a resource:
```rust
let mut input = ButtonInput::<KeyCode>::default();
input.press(KeyCode::Space);
app.insert_resource(input);
app.update();
```

Source: https://github.com/bevyengine/bevy/blob/main/tests/how_to_test_systems.rs

---

## 2. Bevy Testing Crates (Ecosystem)

### bevy-autoplay (Record & Replay Testing)
- **Crate**: `bevy-autoplay` (https://crates.io/crates/bevy-autoplay)
- **Approach**: Record human play sessions, replay them as automated tests
- **How**: `AutoplayPlugin` records input events to `.gsi` files, `AutoplayTestPlugin` replays them
- **Testing**: Write assertions that run during replay (e.g., "player must press F key")
- **Limitation**: Bevy 0.13 era, 30 stars, not heavily maintained
- Source: https://github.com/tobyselway/bevy-autoplay

### bevy_testing
- **Crate**: `bevy_testing` (https://crates.io/crates/bevy_testing)
- **Approach**: Ergonomic test helpers - `app.update_once()`, `app.query::<&C>().matches(vec![...])`
- **Limitation**: Bevy 0.14 only, 0 stars
- Source: https://github.com/bnjmn21/bevy_testing

### rmv-bevy-testing-tools (Most Active)
- **Crate**: `rmv-bevy-testing-tools` v0.10.2 (https://crates.io/crates/rmv-bevy-testing-tools)
- **Approach**: Uses rstest + insta (snapshot testing) + speculoos (assertions)
- **Key**: Supports Bevy 0.18, most up-to-date testing crate
- **Downloads**: 13,225 total
- Source: https://github.com/rmvermeulen/rmv-bevy-testing-tools

---

## 3. AI-Driven Game Testing Approaches

### Architecture for LLM Game Testing
The most practical approach for a card/strategy game like Arena of Ideas:

**Option A: Structured State + LLM Decision Making (Recommended)**
1. Export game state as JSON/structured text (units, HP, abilities, slots)
2. Send state to LLM (Claude) via API/MCP tool
3. LLM returns action (which unit to play, which ability to use)
4. Inject action into Bevy via `ButtonInput` resources or custom events
5. Loop until game ends

**Option B: Screenshot/Vision-Based**
1. Render game to offscreen buffer (headless with `bevy_render` but no window)
2. Send screenshot to vision model
3. Model returns click coordinates or action names
4. More brittle, slower, expensive - NOT recommended for strategy games

**Option C: Deterministic Bot + LLM Evaluation**
1. Write simple rule-based bots that play the game
2. Use LLM to evaluate outcomes, suggest balance changes
3. Best for balance testing at scale

### MCP Server Approach (Best for Claude Integration)
Build an MCP server that exposes:
- `get_game_state()` - returns structured game state
- `perform_action(action)` - executes a game action
- `get_available_actions()` - lists legal moves
- `get_game_result()` - returns win/loss/score

Claude can then play the game through tool use, reasoning about strategy.

### Industry Approaches
- **Unity ML-Agents** (19.3k stars): Deep RL + imitation learning for game testing. Uses Python training + C# environment. Architecture: Agent observes state, chooses action, gets reward.
- **EA SEED**: Automated play-testing with RL agents for balance testing
- **Riot Games**: Uses bots + statistical analysis for balance
- **Key Insight**: For card/strategy games, structured state + tree search outperforms RL

---

## 4. SpacetimeDB Testing (Internal Framework)

SpacetimeDB has an internal `testing` crate at `crates/testing/`:

### Module Testing Pattern
```rust
// Start standalone server in test process
let paths = ensure_standalone_process();  // Spawns SpacetimeDB server in background thread

// Use CLI in-process (no shell-out needed)
invoke_cli(paths, &["publish", "--project-path", module_path]);
invoke_cli(paths, &["call", module_name, "reducer_name", "--"]);
```

### SDK Test Framework
```rust
pub struct Test {
    name: String,
    module_name: String,       // Module in SpacetimeDB/modules directory
    client_project: String,    // Path to client project
    generate_language: String,  // For code generation
}

impl Test {
    pub fn run(self) {
        let paths = ensure_standalone_process();
        // Publishes module, generates client code, builds client, runs client tests
    }
}
```

### For Arena of Ideas
- You can test SpacetimeDB modules by spinning up a standalone server in-process
- Use `spacetimedb-standalone::start_server()` to run server in test thread
- Invoke reducers programmatically via CLI or direct function calls
- Source: https://github.com/clockworklabs/SpacetimeDB/tree/master/crates/testing

---

## 5. Rust Crates for Game Automation

### Input Simulation
| Crate | Downloads | Purpose | Notes |
|-------|-----------|---------|-------|
| **enigo** v0.6.1 | 685K | Cross-platform keyboard/mouse simulation | OS-level input injection (Linux/Win/Mac) |
| **gilrs** v0.11.1 | 5.6M | Gamepad input library | Read-only; no simulation. Bevy uses this internally |

### Headless Rendering
| Crate | Purpose | Notes |
|-------|---------|-------|
| **pixels** | Framebuffer for 2D | Minimal, CPU-side pixel buffer. No GPU needed |
| **softbuffer** | Software rendering | Platform-native software rendering |
| **wgpu** headless | GPU compute without window | `wgpu` supports headless via `InstanceFlags::empty()` |

### Testing Frameworks
| Crate | Purpose | Notes |
|-------|---------|-------|
| **probador/jugar-probar** | WASM game testing (Playwright-like) | GUI coverage tracking, headless WASM testing |
| **bevy-autoplay** | Record/replay Bevy sessions | Input recording to file, replay in tests |
| **rmv-bevy-testing-tools** | Bevy ECS testing with rstest+insta | Most actively maintained, supports Bevy 0.18 |

---

## 6. Recommended Architecture for Arena of Ideas

### Phase 1: Headless Game Logic Testing
```
[Bevy App - No Window] --> [Game Systems] --> [Assert on ECS State]
                      |
                      +--> ScheduleRunnerPlugin (fixed timestep)
                      +--> No DefaultPlugins rendering
                      +--> MinimalPlugins + your game logic plugins
```

**Cargo.toml for test binary:**
```toml
[dev-dependencies]
bevy = { version = "0.18", default-features = false, features = ["bevy_state"] }
```

### Phase 2: AI Play-Testing via MCP
```
[Claude via MCP] <---> [MCP Server]
                           |
                    get_game_state()
                    get_available_actions()
                    perform_action(action)
                           |
                    [Headless Bevy App]
                           |
                    [SpacetimeDB Module]
```

### Phase 3: Automated Balance Testing
```
[CI Pipeline]
    |
    +--> Run N games with bot strategies
    +--> Collect win rates, game length, unit usage stats
    +--> Flag statistical outliers
    +--> (Optional) LLM summarizes findings
```

### Key Implementation Steps
1. **Create a headless test harness**: `MinimalPlugins` + game logic plugins, no rendering
2. **Serialize game state**: Implement `Serialize` on key components for JSON export
3. **Build action interface**: Enum of all possible player actions, injectable via events
4. **MCP server wrapper**: Expose game state + actions as MCP tools
5. **SpacetimeDB test setup**: Use `ensure_standalone_process()` pattern from their test crate
6. **CI integration**: Headless tests in GitHub Actions (no GPU needed)

---

## References
- Bevy headless example: https://github.com/bevyengine/bevy/blob/main/examples/app/headless.rs
- Bevy system testing: https://github.com/bevyengine/bevy/blob/main/tests/how_to_test_systems.rs
- bevy-autoplay: https://github.com/tobyselway/bevy-autoplay
- rmv-bevy-testing-tools: https://crates.io/crates/rmv-bevy-testing-tools
- Probar (WASM game testing): https://github.com/paiml/probar
- SpacetimeDB testing: https://github.com/clockworklabs/SpacetimeDB/tree/master/crates/testing
- Unity ML-Agents (architecture reference): https://github.com/Unity-Technologies/ml-agents
- enigo (input simulation): https://github.com/enigo-rs/enigo
- gilrs (gamepad input): https://gitlab.com/gilrs-project/gilrs
