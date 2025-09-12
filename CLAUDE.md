# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Development Commands

### Building and Running
```bash
# Build the entire project
cargo build

# Build with optimizations
cargo build --release

# Run the client
cargo run

# Run the client with dynamic linking for faster development builds
cargo run --features dynamic_linking
```

### Database Operations (SpacetimeDB)
```bash
# Publish the server module to SpacetimeDB
spacetime publish server

# Clear the database
spacetime delete <database-name>

# View database logs
spacetime logs <database-name>
```

### Testing and Quality
```bash
# Run tests
cargo test

# Check for compilation issues
cargo check

# Format code
cargo fmt

# Run linter
cargo clippy
```

## Architecture Overview

### Technology Stack
Arena of Ideas is a **Rust-based PvP auto-battler** built with:
- **Bevy** - Game engine and ECS framework
- **SpacetimeDB** - Real-time multiplayer database backend
- **egui** - Immediate mode GUI framework
- **Custom Node System** - Dynamic game content system

### Workspace Structure

**Client-Server Split:**
- `client/` - Bevy-based game client with UI and rendering
- `server/` - SpacetimeDB module handling game logic and state
- `schema/` - Shared data structures and types between client/server

**Core Systems:**
- `raw-nodes/` - Raw node definitions for the dynamic content system
- `node-loaders/` - Code generation for efficient node loading
- `utils/` - Shared server utilities  
- `utils-client/` - Client-specific utilities
- `settings-derive/` - Procedural macros for settings management

### Node System Architecture

The project uses a **sophisticated node system** for dynamic game content:

1. **Raw Nodes** (`raw-nodes/`): Define the structure of game entities (units, houses, abilities, etc.)

2. **Code Generation**: Build scripts automatically generate:
   - Node loaders with efficient loading patterns
   - Server implementations with CRUD operations
   - Client implementations with ECS integration
   - NodeKind enums for type safety

3. **Data Flow**:
   ```
   Raw Node Definitions → Code Generation → Server/Client Implementations
                                         ↓
   SpacetimeDB ←→ Server Module ←→ Client ECS Components
   ```

### Game Architecture

**Core Game Concepts:**
- **Units** - Basic building blocks with stats (HP, PWR) and behaviors
- **Houses** - Thematic factions containing units and magic abilities
- **Fusions** - Combat units created by combining multiple units
- **Teams** - Battle formations of up to 5 fusions
- **Matches** - Game sessions with progression through floors

**Battle System:**
- Auto-battler with trigger-based behaviors
- Front fusions engage first, with complex action/reaction systems
- Status effects, damage calculation, and turn-based progression

### SpacetimeDB Integration

The server runs as a **SpacetimeDB module** (`server/src/lib.rs`):
- Handles multiplayer state synchronization
- Manages game logic and battle resolution  
- Provides reducers for client actions
- Stores persistent game data

Client connects via **SpacetimeDB SDK** for real-time updates.

### Build System Notes

- **Workspace Cargo.toml**: Defines shared dependencies and workspace members
- **Code Generation**: Multiple build scripts generate implementations from node definitions
- **Dynamic Linking**: Available for faster development builds via feature flag
- **Profile Optimization**: Development builds optimize dependencies for better performance

### UI System

Uses **egui** with Bevy integration:
- `client/src/ui/` - UI system implementation
- `client/src/plugins/` - Various game systems as Bevy plugins
- Immediate mode GUI with tiles for complex layouts

### Asset Management

- `assets/` - Game assets (textures, sounds, etc.)
- RON format for configuration files
- Bevy asset loader integration with hot reloading support

## Development Notes

- **Node System**: When adding new game content, define it in `raw-nodes/src/raw_nodes.rs` and rebuild
- **Client-Server Sync**: Changes to shared types in `schema/` require rebuilding both client and server
- **Database Schema**: Server changes may require database resets during development
- **Hot Reloading**: Client supports asset hot reloading with file watcher enabled

## Common Development Patterns

- Most game logic lives in the server module for authoritative multiplayer
- Client focuses on rendering, UI, and user input
- Node system enables data-driven game design without code changes
- Use workspace dependencies for consistent versions across crates