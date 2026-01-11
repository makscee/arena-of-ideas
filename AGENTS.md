# Arena of Ideas - Agent Guidelines

## Build & Development Commands

### Essential Commands
- `cargo test` - Run all tests
- `cargo test <test_name>` - Run a single test
- `cargo run -- --mode test` - Run test scenarios (battle tests, etc.)
- `cargo build` - Build the project
- `cargo clippy -- -D warnings` - Lint (must pass on PR)
- `cargo fmt --all` - Format code
- `cargo fmt --all -- --check` - Check formatting without modifying
- `cargo build -p server` - Build server module to WASM for SpacetimeDB
- `just gen-binds` - Generate SpacetimeDB Rust bindings

### Run Modes
The client supports multiple run modes via CLI:
- `cargo run` - Regular game mode
- `cargo run -- --mode shop` - Shop mode
- `cargo run -- --mode test` - Test scenarios
- `cargo run -- --mode sync` - Server sync mode
- `cargo run -- --mode world-download` / `--mode world-upload` - World migration

## Project Structure

This is a Rust workspace with these crates:
- **client** - Bevy-based game client
- **server** - SpacetimeDB server logic (compiled to WASM)
- **schema** - Shared data structures and node definitions
- **utils** - Shared utilities (both client/server)
- **utils-client** - Client-specific utilities
- **proc-macros** - Custom derive macros
- **node-build-utils** - Build system helpers

## Code Style Guidelines

### Imports
- Prefer `use super::*;` in child modules
- Export modules with `pub use module::*;` pattern
- All external types come through `client::prelude`
- Keep imports alphabetically organized

### Formatting
- Use `cargo fmt` (standard rustfmt rules)
- 4-space indentation
- Max line width: 100 characters (enforced by CI)

### Naming Conventions
- **Types/Structs/Enums**: PascalCase (e.g., `BattleAction`, `GameState`)
- **Functions/Methods**: snake_case (e.g., `get_value`, `process_actions`)
- **Constants**: UPPER_SNAKE_CASE (e.g., `UNIT_SIZE`, `VERSION`)
- **Private modules**: lowercase, use `r#` for keywords (e.g., `mod r#fn;`)
- **Traits**: PascalCase with `Impl` suffix for implementation traits (e.g., `ExpressionImpl`)

### Type System
- Use `Result<T, NodeError>` for operations that can fail
- Custom types from `schema` crate for all game data
- Bevy `Entity` for ECS entity references, not for game objects
- Use `u64` IDs for persistent game objects (units, teams, etc.)
- Leverage the builder pattern for complex constructors (e.g., `VfxBuilder`)

### Error Handling
- Always propagate errors with `?` operator
- Use `.with_context(msg)` for automatic location tracking with custom message
- Use `.track()` for manual location tracking (legacy, prefer `.with_context()`)
- Use `.log()` for logging errors in a controlled way
- Never ignore errors (no empty catch blocks)
- Custom error types: `NodeError` for game logic, `anyhow::Error` for general cases

#### Creating Errors
- Use specific error variants: `NodeError::not_found(id)`, `NodeError::invalid(field, value)`, `NodeError::not_in_context(msg)`
- For complex cases: `bail!("Error: {}", context)` or `bail_invalid!("field", "value")`
- For custom needs: `NodeError::custom(msg)` (use sparingly, prefer specific variants)

#### Propagating Errors
- With context: `ctx.load::<NUnit>(id)?.with_context("Failed to load unit")?`
- Simple tracking: `ctx.load::<NUnit>(id)?.track()?` (legacy, prefer `.with_context()`)
- Logging: `result.log()` (continues execution)

#### Converting Options
- Generic: `option_value.not_found()?`
- With message: `option_value.ok_or_else(|| NodeError::custom("Entity not found"))?`
- Not in context: `option_value.not_in_context("resource")?`

#### Available Error Variants
- `not_found(id)` - Generic not found by ID
- `entity_not_found(id)` - Entity lookup failed
- `var_not_found(var)` - Variable access failed
- `not_in_context(msg)` - Resource not available in current context
- `invalid_state(context)` - Invalid state for operation
- `OperationNotSupported { ... }` - Type operation not supported (for VarValue operations)
- `custom(msg)` - Fallback for non-specific errors (use sparingly, prefer specific variants)

#### Bail Macros
- `bail!(msg)` - Quick early return with custom error
- `bail_not_found!(id)` - Early return with not found error
- `bail_var!(var)` - Early return with variable not found
- `bail_not_in_context!(msg)` - Early return with not in context error

### Testing
- Unit tests in `#[cfg(test)]` modules within source files
- Integration tests in `client/src/tests/`
- Use the `TestBuilder` pattern for complex battle scenarios
- Test functions use descriptive names: `test_<feature>_<scenario>`
- Assertions should use domain-specific helpers like `.assert_winner(TeamSide::Left)`

### Module Organization
```rust
mod private_module;         // Private modules
pub mod public_module;       // Public API
use super::*;                // Import parent items
pub use private_module::*;   // Re-export for convenience
```

### Node System Pattern
The codebase uses a custom node system:
- Load nodes: `ctx.load::<NodeType>(id)?`
- Access child nodes: `parent.child.load_node(ctx)?`
- Edit nodes: Clone, modify, then `node.save(ctx)?`
- Var fields use `ctx.get_var(var)` and `ctx.set_var(var, value)`
- All node operations go through `ClientContext` for history tracking

### Bevy-Specific Patterns
- Add plugins in `App::add_plugins()`
- Use system scheduling: `Startup`, `Update`, `PreUpdate`, etc.
- State-based systems with `OnEnter`, `OnExit`, `in_state`
- Resources for global state (`pub struct GameStateResource`)
- Components for ECS entities (`#[derive(Component)]`)

### Scripting
- Rhai scripts for game logic (units, abilities, statuses)
- Lua support in some modules
- Scripts access game state via provided context APIs
- Never execute untrusted scripts without sandboxing

## Common Patterns

### Trait Implementation
```rust
pub trait SomeTrait {
    fn method(&self) -> Result<Type, NodeError>;
}

impl SomeTrait for ConcreteType {
    fn method(&self) -> Result<Type, NodeError> {
        // Implementation with ? operator
    }
}
```

### Error Propagation
```rust
fn example(ctx: &Context) -> Result<(), NodeError> {
    let value = ctx.get_var(VarName::example)?;
    ctx.set_var(VarName::output, value)?;
    Ok(())
}
```

### Builder Pattern
```rust
let action = BattleAction::new_vfx("effect_name")
    .with_var(VarName::position, pos)
    .with_var(VarName::color, color)
    .into();
```

## CI/CD Requirements
- All tests must pass: `cargo test`
- Clippy must pass with no warnings: `cargo clippy -- -D warnings`
- Code must be formatted: `cargo fmt --all -- --check`
- Test scenarios must pass: `cargo run -- --mode test`

## Notes
- Use workspace dependencies from `Cargo.toml` [workspace.dependencies]
- No `.rustfmt.toml` - use default settings
- No `.clippy.toml` - use default clippy rules
- The project uses SpacetimeDB for database operations
- Bevy 0.17 for game engine
- The game is an auto-battler with player-generated content
