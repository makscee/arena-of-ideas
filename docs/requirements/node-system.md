# Node System Requirements Document

## Overview
The Node System is the foundational data architecture that represents all game entities, relationships, and hierarchical structures. It provides a unified model for game content including players, houses, units, teams, battles, and matches through a parent-child relationship graph stored in SpacetimeDB.

## Core Responsibilities
- Define and maintain the hierarchical structure of all game data
- Provide type-safe access to node relationships (parents, children, links)
- Handle node creation, deletion, and relationship management
- Ensure data consistency across the distributed game state
- Support recursive loading and querying of node trees
- Manage node lifecycle and garbage collection

## Dependencies
### Input Dependencies
- SpacetimeDB tables for persistent storage
- Schema definitions from schema crate
- Raw node definitions from raw-nodes crate
- Network synchronization from STDB plugin

### Output Dependencies
- All game systems depend on node data structure
- UI systems query nodes for display information
- Battle system operates on team and unit nodes
- Match system manages player and team relationships
- Content system populates initial node structures

## Data Model
### Primary Node Types
Based on raw_nodes.rs structure:

**Core Nodes:**
- NCore: Root container for houses
- NPlayers: Container for all player data
- NArena: Container for floor pools and bosses

**Player Nodes:**
- NPlayer: Individual player with name, data, identity, active match
- NPlayerData: Authentication and session information
- NPlayerIdentity: Player identity data

**Game Content Nodes:**
- NHouse: Faction with units, abilities, and visual theme
- NHouseColor: Visual representation of house
- NActionAbility/NStatusAbility: House-specific abilities
- NUnit: Individual game piece with stats and behavior
- NUnitStats/NUnitState/NUnitBehavior: Unit properties

**Match Nodes:**
- NMatch: Individual game session with round, floor, lives
- NTeam: Collection of houses and fusions for battle
- NBattle: Individual combat instance with results
- NBenchSlot: Unit positioning in team formation
- NFusion: Combined unit with enhanced abilities

**Arena Nodes:**
- NFloorPool: Available teams for specific floor
- NFloorBoss: Boss encounters per floor

### State Management
- Node ownership relationships (OwnedParent/OwnedChildren)
- Linked relationships for references (LinkedParent/LinkedChildren)
- Recursive loading capabilities for complete subtrees
- Lazy loading for performance optimization

## User Interactions
### UI Components
- Node Explorer: Tree view for debugging node relationships
- Entity viewers that display node properties
- Relationship editors for modifying connections

### Input Handling
- Node selection and navigation
- Property editing interfaces
- Relationship creation/deletion tools

## Game Flow Integration
### Game States
- Node loading during Loading state
- Node synchronization in ServerSync state
- Node editing in Editor state
- Node querying in all active game states

### Event Handling
- Node creation/deletion events
- Relationship change notifications
- Data synchronization events from server
- Cache invalidation on node updates

## Technical Architecture
### Plugin Structure
- NodeStatePlugin in src/plugins/node_state.rs
- STDB integration in src/plugins/stdb.rs
- Node definitions in src/nodes/ directory

### Key Components
- Node trait defining common node behavior
- Relationship types (OwnedParent, OwnedChildren, LinkedParent, LinkedChildren)
- Context system for node access and queries
- Recursive loading utilities
- Node caching and lazy loading

## Business Logic
### Rules and Constraints
- Each node has exactly one owner (except root nodes)
- Cyclic relationships are prevented
- Node deletion cascades to owned children
- Linked relationships don't affect ownership
- Data integrity maintained across client-server synchronization

### Algorithms
- Recursive tree traversal for loading
- Reference counting for garbage collection
- Relationship validation during modifications
- Efficient querying with caching strategies
- Conflict resolution for concurrent modifications

## Network Integration
### SpacetimeDB Tables
- Relationship tables for parent-child mappings
- Efficient bulk loading queries
- Subscription management for real-time updates

### Client-Server Synchronization
- Incremental updates for modified nodes
- Conflict resolution for simultaneous edits
- Optimistic updates with rollback capability
- Batch operations for performance

## Configuration
### Settings
- Debug visualization options

### Content Data
- Initial node structures defined in content files
- Node templates for common patterns
- Validation rules for node relationships

## Testing Considerations
### Unit Test Areas
- Node relationship validation
- Recursive loading correctness
- Cache consistency
- Ownership transfer operations
- Garbage collection behavior

### Integration Points
- Database consistency with SpacetimeDB
- UI synchronization with node changes
- Performance under high node counts
- Concurrent access patterns

## Future Considerations
### Planned Features
- Node versioning for rollback capabilities
- Query optimization with indexing
- Schema migration support for node structure changes
- Plugin-defined custom node types

### Extensibility Points
- Custom relationship types
- Node behavior hooks for game logic
- Validation plugin system
- External node sources

## Implementation Notes
### Performance Requirements
- Sub-millisecond node access for cached data
- Efficient bulk operations for large trees
- Memory usage optimization for mobile platforms
- Lazy loading to reduce network traffic

### Error Handling
- Graceful degradation for missing nodes
- Validation error reporting with context
- Recovery from corrupted relationships
- Detailed logging for debugging node issues

## Related Documentation
- game-state-management.md for lifecycle coordination
- battle-system.md for combat node usage
- match-system.md for game session nodes
- ui-system.md for node visualization
- network-system.md for synchronization details
