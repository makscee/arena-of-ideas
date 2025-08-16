# Battle System Requirements Document

## Overview
The Battle System manages combat simulation, visualization, and interaction within the Arena of Ideas game. It handles the execution of battles between teams, processes unit abilities and reactions, manages battle state, and provides real-time visualization of combat events.

## Core Responsibilities
- Execute combat logic between two teams following game rules
- Process unit abilities, status effects, and reactions in proper order
- Manage battle state transitions and turn progression
- Provide real-time battle visualization and animation
- Handle user input during battle observation and interaction
- Generate and store battle results for match progression
- Support battle replay and analysis functionality

## Dependencies
### Input Dependencies
- Team nodes (NTeam) with configured units and fusions
- Unit definitions with stats, abilities, and behaviors
- House abilities and status effects from node system
- Battle configuration from match system
- User input for battle controls and interactions

### Output Dependencies
- Match system receives battle results and progression
- Node system stores battle records (NBattle)
- UI system displays battle visualization
- Audio system triggers combat sound effects
- Statistics system tracks battle performance data

## Data Model
### Primary Node Types
- NBattle: Individual combat instance with teams, timestamp, hash, result
- NTeam: Team composition with houses and fusions
- NUnit: Individual combat units with stats and abilities
- NFusion: Combined units with enhanced capabilities
- NHouse: Faction providing abilities and status effects
- NActionAbility/NStatusAbility: Combat abilities and effects

### State Management
- BattleState resource tracking current combat phase
- Unit positioning and formation data
- Active status effects and their durations
- Turn order and initiative tracking
- Combat event history for replay functionality
- Animation state for visual effects

## User Interactions
### UI Components
- Battle viewport showing unit positions and animations
- Unit selection and inspection panels
- Ability activation controls and targeting
- Battle speed and pause controls
- Combat log and event history viewer
- Battle statistics and analysis displays

### Input Handling
- Unit selection via mouse clicks
- Camera controls for battle viewing
- Battle speed adjustment (pause, normal, fast forward)
- Ability targeting and confirmation
- Battle restart and exit controls

## Game Flow Integration
### Game States
- Battle: Primary combat execution and visualization state
- Editor: Battle scenario setup and testing
- TestScenariosRun: Automated battle testing mode

### Event Handling
- Battle start initialization from match system
- Combat event processing and resolution
- Battle completion and result generation
- Animation completion events for visual synchronization
- User input events for battle interaction

## Technical Architecture
### Plugin Structure
- BattlePlugin in src/plugins/battle.rs
- BattleEditorPlugin in src/plugins/battle_editor.rs
- Systems scheduled in FixedUpdate for deterministic combat
- Reload system for editor mode battle testing

### Key Components
- BattleState resource for combat tracking
- ReloadData resource for editor integration
- Combat event queue for turn-based processing
- Animation system for visual effects
- Battle result generation and storage

## Business Logic
### Rules and Constraints
- Turn-based combat with deterministic order
- Unit abilities have specific activation conditions
- Status effects follow stack-based duration rules
- Combat follows rock-paper-scissors style interactions
- Battle results must be reproducible with same inputs
- Unit positioning affects ability targeting and effects

### Algorithms
- Initiative calculation for turn order
- Damage calculation with modifiers and resistances
- Status effect application and resolution
- Ability target selection and validation
- Combat result hash generation for verification
- Animation timing and interpolation

## Network Integration
### SpacetimeDB Tables
- Battle table for storing combat results
- Team tables for battle participants
- Unit and fusion data for combat execution
- Battle history for replay and analysis

### Client-Server Synchronization
- Battle verification through deterministic hashing
- Replay data synchronization for analysis
- Real-time battle observation for multiplayer
- Conflict resolution for simultaneous battles

## Configuration
### Settings
- Battle animation speed settings
- Visual effect quality options
- Combat calculation precision
- Replay storage duration
- Debug visualization toggles

### Content Data
- Unit ability definitions and effects
- Status effect behaviors and interactions
- Combat formulas and balance parameters
- Animation timing and visual assets

## Testing Considerations
### Unit Test Areas
- Combat calculation accuracy
- Status effect application logic
- Turn order determination
- Ability targeting validation
- Battle result reproducibility

### Integration Points
- Node system data consistency
- UI animation synchronization
- Match system result integration
- Network battle verification
- Editor mode battle testing

## Future Considerations
### Planned Features
- Advanced battle AI for automated testing
- Battle replay sharing and analysis tools
- Real-time multiplayer battle observation
- Custom battle scenario creation
- Performance profiling and optimization tools

### Extensibility Points
- Plugin-based ability system for mods
- Custom status effect creation tools
- Battle rule modifications for game modes
- External battle analysis integration
- User-created battle scenarios

## Implementation Notes
### Performance Requirements
- 60 FPS smooth animation during battles
- Sub-second battle calculation for fast-forward mode
- Efficient memory usage for large team battles
- Responsive UI during combat execution
- Scalable replay data storage

### Error Handling
- Graceful handling of invalid battle configurations
- Recovery from corrupted battle state
- Validation of user input during battle
- Error reporting for battle calculation failures
- Fallback for missing unit or ability data

## Related Documentation
- node-system.md for battle data structure
- match-system.md for battle integration
- game-state-management.md for battle flow
- ui-system.md for battle visualization
- network-system.md for battle synchronization