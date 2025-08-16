# Match System Requirements Document

## Overview
The Match System manages individual game sessions, player progression through arena floors, shop interactions, and the overall match lifecycle. It coordinates between battles, handles shop mechanics, tracks player lives and progression, and manages the transition between different match phases.

## Core Responsibilities
- Manage match lifecycle from start to completion
- Handle shop phase with unit offerings and purchases
- Track player progression through arena floors and rounds
- Manage player lives and elimination conditions
- Coordinate battle execution and result processing
- Handle match completion and reward distribution
- Provide shop interface for team building and unit management
- Manage bench slots and team composition

## Dependencies
### Input Dependencies
- Player nodes (NPlayer) with active match references
- Team nodes (NTeam) for player compositions
- Unit and house data for shop offerings
- Battle results from battle system
- Shop interaction events from UI system
- Arena configuration for floor and round progression

### Output Dependencies
- Battle system receives team configurations for combat
- Node system stores match progress and team changes
- UI system displays match status and shop interface
- Game state management handles match completion transitions
- Player progression system receives match results

## Data Model
### Primary Node Types
- NMatch: Core match data with floor, round, lives, shop offers, team, bench, battles
- NTeam: Player's current team composition
- NBenchSlot: Individual bench positions with unit assignments
- NBattle: Combat instances within the match
- NUnit: Units available for purchase and team building
- NPlayer: Player data with active match reference
- NFloorPool: Available opponents for current floor
- NFloorBoss: Boss encounters at floor completion

### State Management
- Match progression state (floor, round, lives remaining)
- Shop phase with available unit offerings
- Team composition and bench configuration
- Battle queue and completion tracking
- Match completion status and results
- Shop transaction history and currency

## User Interactions
### UI Components
- Shop interface with unit offerings and purchase options
- Team composition viewer with drag-and-drop functionality
- Bench management with unit positioning
- Match status display with floor, round, and lives
- Battle initiation controls
- Match completion summary and rewards

### Input Handling
- Unit purchase clicks in shop interface
- Drag-and-drop for team and bench management
- Battle start confirmation
- Shop refresh and reroll actions
- Match completion acknowledgment

## Game Flow Integration
### Game States
- Shop: Primary match management and team building state
- Battle: Combat execution triggered from match system
- MatchOver: End-of-match summary and progression
- Title: Return after match completion

### Event Handling
- Match initialization when entering shop state
- Shop offer generation and refresh
- Unit purchase and team modification events
- Battle completion processing
- Match progression and floor advancement
- Match completion and cleanup

## Technical Architecture
### Plugin Structure
- MatchPlugin in src/plugins/match.rs
- Systems scheduled in Update for shop state
- OnEnter system for match initialization
- Battle integration systems for result processing

### Key Components
- Match state tracking and validation
- Shop offer generation and management
- Team composition modification tools
- Battle queue and execution coordination
- Match completion detection and handling
- Bench slot management utilities

## Business Logic
### Rules and Constraints
- Players start with limited lives (typically 3)
- Shop offers refresh between rounds
- Unit purchases consume currency/resources
- Team composition has size and formation limits
- Bench provides backup units and strategic options
- Floor progression requires battle victories
- Match ends when lives reach zero or floor goals achieved
- Boss battles at specific floor intervals

### Algorithms
- Shop offer generation with rarity and balance considerations
- Team composition validation and optimization
- Match progression calculation based on performance
- Currency and resource management
- Battle opponent selection from floor pools
- Match completion criteria evaluation

## Network Integration
### SpacetimeDB Tables
- Match table for persistent match state
- Team tables for composition tracking
- Battle tables for combat history
- Shop transaction logs
- Player progression data

### Client-Server Synchronization
- Real-time match state synchronization
- Shop offer consistency across sessions
- Team composition updates
- Battle result integration
- Match completion verification

## Configuration
### Settings
- Match duration and progression parameters
- Shop offer pool and refresh rates
- Starting lives and difficulty scaling
- Currency and resource allocation
- Floor progression requirements

### Content Data
- Unit availability and shop pools
- Floor configuration and opponent pools
- Boss encounter definitions
- Reward structures and progression bonuses
- Balance parameters for match difficulty

## Testing Considerations
### Unit Test Areas
- Shop offer generation and balancing
- Team composition validation
- Match progression calculation
- Battle result integration
- Currency and resource tracking

### Integration Points
- Battle system coordination
- Node system data consistency
- UI state synchronization
- Player progression integration
- Network state management

## Future Considerations
### Planned Features
- Tournament and league match formats
- Custom match rules and modifiers
- Spectator mode for ongoing matches
- Match replay and analysis tools
- Social features for match sharing

### Extensibility Points
- Plugin-based match rule modifications
- Custom shop mechanics and offerings
- Alternative progression systems
- External tournament integration
- User-created match formats

## Implementation Notes
### Performance Requirements
- Responsive shop interface with instant unit previews
- Efficient team composition updates
- Quick battle initiation without loading delays
- Smooth transitions between match phases
- Minimal network latency for shop interactions

### Error Handling
- Graceful handling of network disconnections during matches
- Recovery from corrupted match state
- Validation of shop purchases and team modifications
- Error reporting for invalid match configurations
- Fallback options for failed battle initiation

## Related Documentation
- battle-system.md for combat integration
- node-system.md for match data structure
- game-state-management.md for match flow
- ui-system.md for shop and match interfaces
- network-system.md for match synchronization