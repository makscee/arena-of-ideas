# Fully Automatic Node System

A completely automatic node generation system that reads node definitions from a single source file and generates client-specific and server-specific implementations with zero manual maintenance.

## Key Feature: Zero Manual Maintenance

**Just add a struct to `nodes/src/raw_nodes.rs` and everything else happens automatically:**
- ✅ Client structs generated with client-specific derives and traits
- ✅ Server structs generated with server-specific derives and traits  
- ✅ NodeKind enum updated with new variants
- ✅ All implementations automatically available
- ✅ No need to update any lists or maintain any other files

## Architecture

The system consists of four main crates:

### 1. `nodes` (Base Crate)
- Contains ONLY the raw node definitions in `src/raw_nodes.rs`
- Provides common types like `ChildComponent`, `ParentComponent`, `ParentLinks`, etc.
- No build script needed - just provides shared type definitions

### 2. `nodes-client` 
- **Build script automatically reads `raw_nodes.rs`**
- **Generates client-specific structs with derives:** `Debug, Clone, Serialize, Deserialize, Default, Hash, PartialEq, Eq`
- **Generates client-specific trait implementations:** `ClientNode` trait
- **Generates NodeKind enum automatically**
- Zero manual maintenance required

### 3. `nodes-server`
- **Build script automatically reads `raw_nodes.rs`**
- **Generates server-specific structs with same derives as client**
- **Generates server-specific trait implementations:** `ServerNode` and `DatabaseNode` traits
- **Generates NodeKind enum automatically**
- Zero manual maintenance required

### 4. `nodes-example`
- Demonstrates the fully automatic system
- Shows how new nodes are instantly available in both client and server
- Proves that no manual updates are needed anywhere

## Usage

### 1. Define Nodes (The ONLY Manual Step)

Add node structures to `nodes/src/raw_nodes.rs`:

```rust
struct NPlayer {
    pub player_name: String,
    pub player_data: ParentComponent<NPlayerData>,
    pub identity: ParentComponent<NPlayerIdentity>,
    pub active_match: ParentComponent<NMatch>,
}

struct NHouse {
    pub house_name: String,
    pub color: ParentComponent<NHouseColor>,
    pub ability_magic: ParentComponent<NAbilityMagic>,
    pub status_magic: ParentComponent<NStatusMagic>,
    pub units: ChildComponents<NUnit>,
}

// Add any new node - it automatically works everywhere!
struct NNewFeature {
    pub new_field: String,
    pub new_data: i32,
}
```

### 2. Use Client Nodes (Automatically Generated)

```rust
// Import automatically generated client implementations
use nodes_client::*;

// All structs are automatically available with client traits
let player = NPlayer { 
    player_name: "test".to_string(),
    ..Default::default() 
};

// Client-specific methods automatically implemented
let render_data = player.prepare_for_render();
player.sync_from_server(&data)?;

// NodeKind enum automatically includes all variants
let kind = NodeKind::NPlayer;
```

### 3. Use Server Nodes (Automatically Generated)

```rust
// Import automatically generated server implementations  
use nodes_server::*;

// Same structs, different traits and implementations
let player = NPlayer { 
    player_name: "test".to_string(),
    ..Default::default() 
};

// Server-specific methods automatically implemented
player.save()?;
let table = NPlayer::table_name(); // "NPlayer"
player.validate()?;
let authorized = player.authorize_access(user_id);

// NodeKind enum automatically includes all variants
let kind = NodeKind::NPlayer;
```

## Key Features

### 🤖 Fully Automatic Generation
- **Build scripts automatically parse `raw_nodes.rs` at compile time**
- **Zero maintenance:** Add struct → everything works everywhere
- **Automatic recompilation when `raw_nodes.rs` changes**
- **No manual lists to maintain anywhere in the codebase**

### 🔄 Dual Implementation Strategy
- **Same definitions → Different implementations**
- **Client nodes:** Optimized for rendering, UI, synchronization
- **Server nodes:** Optimized for persistence, validation, business logic
- **Type-safe separation:** Cannot accidentally mix client/server code

### 📦 Rich Type System (Automatically Available)
- `ChildComponent<T>` - Optional child component
- `ChildComponents<T>` - Vector of child components  
- `ParentComponent<T>` - Optional parent reference
- `ParentComponents<T>` - Vector of parent references
- `ParentLinks<T>` - ID-based references to parent nodes

### 🎯 Automatic Trait Implementation

#### Client Traits (Auto-Generated)
- `ClientNode` - Sync from server, prepare for rendering
- Methods: `new()`, `id()`, `sync_from_server()`, `prepare_for_render()`
- Derives: `Debug, Clone, Serialize, Deserialize, Default, Hash, PartialEq, Eq`

#### Server Traits (Auto-Generated)
- `ServerNode` - Server operations, authorization, client sync
- `DatabaseNode` - Persistence operations, table mapping
- Methods: `new()`, `id()`, `validate()`, `save()`, `load()`, `table_name()`
- Same derives as client for compatibility

## Node Types

The system includes the following node types:

### Core Nodes
- `NCore` - Root container for houses
- `NPlayers` - Container for all players
- `NArena` - Game arena with floors and bosses

### Game Entities
- `NPlayer` - Player with identity and match data
- `NHouse` - House with color, magic, and units
- `NTeam` - Team composed of houses and fusions
- `NMatch` - Active game match with battles
- `NBattle` - Individual battle record

### Components
- `NUnit` - Game unit with stats and behavior
- `NFusion` - Combined units with special behavior
- `NAbilityMagic` - Magical abilities with effects
- `NStatusMagic` - Status effects with behavior

### Data Containers
- `NPlayerData` - Player authentication and state
- `NUnitStats` - Unit power and health
- `NBehavior` - Reaction-based behavior system
- `NRepresentation` - Visual representation data

## Automatic Build Process

### Client Build Process (`nodes-client/build.rs`)
1. **Automatically reads** `../nodes/src/raw_nodes.rs`
2. **Parses all struct definitions** using `syn`
3. **Adds client-specific derives** to each struct
4. **Generates client trait implementations** for each struct
5. **Creates NodeKind enum** with all variants
6. **Writes everything** to `$OUT_DIR/client_impls.rs`
7. **Automatically included** in the client library

### Server Build Process (`nodes-server/build.rs`)
1. **Automatically reads** `../nodes/src/raw_nodes.rs` 
2. **Parses all struct definitions** using `syn`
3. **Adds server-specific derives** to each struct
4. **Generates server trait implementations** for each struct
5. **Creates NodeKind enum** with all variants
6. **Writes everything** to `$OUT_DIR/server_impls.rs`
7. **Automatically included** in the server library

## Example Generated Code

**Input** (in `raw_nodes.rs`):
```rust
struct NPlayer {
    pub player_name: String,
    pub active_match: ParentComponent<NMatch>,
}
```

**Auto-Generated Client Code**:
```rust
#[derive(Debug, Clone, Serialize, Deserialize, Default, Hash, PartialEq, Eq)]
pub struct NPlayer {
    pub player_name: String,
    pub active_match: ParentComponent<NMatch>,
}

impl NPlayer {
    pub fn new() -> Self { Self::default() }
    pub fn id(&self) -> u64 { /* client-specific ID */ }
}

impl ClientNode for NPlayer {
    fn sync_from_server(&mut self, data: &[u8]) -> Result<(), ClientSyncError> { /* */ }
    fn prepare_for_render(&self) -> RenderData { /* */ }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum NodeKind {
    NPlayer,
    // ... all other nodes automatically included
}
```

**Auto-Generated Server Code**: Same struct + `ServerNode` + `DatabaseNode` traits

## Integration

### For Client Code:
```toml
[dependencies]
nodes-client = { path = "path/to/nodes-client" }
```

```rust
use nodes_client::*;
// All nodes automatically available with client traits
```

### For Server Code:
```toml
[dependencies] 
nodes-server = { path = "path/to/nodes-server" }
```

```rust
use nodes_server::*;
// All nodes automatically available with server traits
```

### Adding New Nodes:
1. **Add struct to `nodes/src/raw_nodes.rs`**
2. **That's it!** - Everything else is automatic

## Benefits

✅ **Zero Maintenance**: Never update lists or maintain separate files  
✅ **Single Source of Truth**: All definitions in one place  
✅ **Type Safety**: Compile-time separation of client/server concerns  
✅ **Automatic Consistency**: Same structs, different optimized implementations  
✅ **Instant Availability**: New nodes work immediately in both client and server  
✅ **Build-Time Generation**: No runtime overhead  

The system provides the ultimate in developer productivity - just define your data structures once and get fully-featured, type-safe implementations automatically generated for all your use cases.