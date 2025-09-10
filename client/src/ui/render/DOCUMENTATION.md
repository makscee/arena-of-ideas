# Feature-Based UI Rendering System

## Overview

The Feature-Based UI Rendering System is a modern, composable approach to rendering game entities and data in the Arena of Ideas UI. It replaces the older `see()` API with a more flexible and type-safe system based on traits (features) and composable renderers.

## Core Concepts

### 1. Features
Features are traits that define specific capabilities for types. Each feature represents a single aspect of how a type can be rendered or interacted with.

### 2. Composers
Composers are objects that transform data into UI elements. They consume data implementing specific features and produce egui responses.

### 3. Render Builder
The render builder provides a fluent API for rendering objects based on their implemented features.

## Architecture

```
┌─────────────┐
│    Type     │ (implements features)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│   Render    │ (trait for all types)
│   Builder   │
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  Composers  │ (transform data to UI)
└──────┬──────┘
       │
       ▼
┌─────────────┐
│  egui UI    │
└─────────────┘
```

## Available Features

### Display Features
- **FTitle**: Provides a title string
- **FColoredTitle**: Provides a colored title
- **FDescription**: Provides a description
- **FIcon**: Provides an icon or short representation
- **FRepresentation**: Provides a visual representation
- **FStats**: Provides stats/variables
- **FTag**: Provides a compact tag view
- **FInfo**: Provides expanded info string

### Interactive Features
- **FDisplay**: Can be displayed in UI
- **FEdit**: Can be edited in UI
- **FContextMenu**: Provides context menu actions
- **FCopy**: Can be copied to clipboard
- **FPaste**: Can be pasted from clipboard

### Structural Features
- **FRecursive**: Can be recursively traversed (read-only)
- **FRecursiveMut**: Can be recursively traversed (mutable)
- **FExpandable**: Can be expanded/collapsed
- **FSelectable**: Can be selected

### Utility Features
- **FValidate**: Provides validation
- **FSearchable**: Can be searched
- **FFilterable**: Can be filtered
- **FSortable**: Can be sorted
- **FColor**: Provides a color
- **FPreview**: Can be previewed
- **FHelp**: Provides help/documentation
- **FDirty**: Tracks changes
- **FRating**: Has a rating

## Basic Usage

### Simple Rendering

```rust
// Any type automatically gets the render() method
let value = 42;
value.render(context).display(ui);

// Nodes can use various rendering methods
unit.render(context).title(ui);
unit.render(context).tag(ui);
unit.render(context).card(ui);
```

### Mutable Rendering

```rust
let mut value = 42;
if value.render_mut(context).edit(ui) {
    println!("Value changed!");
}
```

### Context Menus

```rust
let response = unit.render(context)
    .with_menu()
    .add_copy()
    .add_paste()
    .add_delete()
    .title(ui);

if let Some(deleted) = response.deleted() {
    // Handle deletion
}
```

## Advanced Composers

### List Composers

```rust
// Basic list
let list = ListComposer::new(TitleComposer);
items.render(context).with_composer(list).compose(ui);

// Filtered list
let filtered = FilteredListComposer::new(
    TitleComposer,
    |item: &Item, ctx| item.matches_criteria()
);

// Sorted list
let sorted = SortedListComposer::new(
    TitleComposer,
    |item: &Item, ctx| item.sort_key()
);

// Paginated list
let paginated = PaginatedComposer::new(TitleComposer, 20);
```

### Tree Rendering

```rust
let tree = TreeComposer::new(
    TagComposer,
    |node: &Node, ctx| node.children()
);
root.render(context).with_composer(tree).compose(ui);
```

### Drag and Drop

```rust
// Make draggable
let draggable = DraggableComposer::new(
    TagComposer,
    "drag_id".to_string()
);

// Create drop target
let drop_target = DropTargetComposer::new(
    TagComposer,
    "drag_id".to_string(),
    |dropped_item, ctx| {
        // Handle drop
    }
);
```

## Implementing Custom Features

### Define a Feature

```rust
pub trait FCustom {
    fn custom_data(&self) -> String;
}

impl FCustom for MyType {
    fn custom_data(&self) -> String {
        format!("Custom: {}", self.value)
    }
}
```

### Create a Composer

```rust
pub struct CustomComposer;

impl<T: FCustom> Composer<T> for CustomComposer {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        ui.label(data.custom_data())
    }
}
```

### Extend RenderBuilder

```rust
impl<'a, T: FCustom> RenderBuilder<'a, T> {
    pub fn custom(self, ui: &mut Ui) -> Response {
        CustomComposer.compose(self.data(), self.context(), ui)
    }
}
```

## Migration from see() API

### Old API
```rust
unit.see(context).button(ui);
unit.see(context).tag(ui);
unit.see(context).card(ui);
unit.see(context).recursive_show(ui);
```

### New API
```rust
unit.render(context).title_button(ui);
unit.render(context).tag(ui);
unit.render(context).card(ui);
unit.render(context).recursive_show(ui);
```

## Game Node Implementations

All game nodes have feature implementations:

### Basic Nodes
- NUnit, NHouse, NTeam
- NPlayer, NPlayerData, NPlayerIdentity
- NArena, NFloorPool, NFloorBoss
- NBattle, NMatch

### Magic System Nodes
- NAbilityMagic, NAbilityDescription, NAbilityEffect
- NStatusMagic, NStatusDescription, NStatusBehavior, NStatusRepresentation

### Unit System Nodes
- NUnitDescription, NUnitStats, NUnitState, NUnitBehavior, NUnitRepresentation
- NFusion, NFusionSlot

Each node type implements appropriate features:
- All implement FTitle for basic display
- Complex nodes implement FDescription, FStats, FTag
- Interactive nodes implement FDisplay, FEdit
- Container nodes implement FContextMenu, FCopy, FPaste

## Performance Considerations

1. **Lazy Rendering**: Use LazyComposer for expensive computations
2. **Pagination**: Use PaginatedComposer for large lists
3. **Virtual Scrolling**: Combine with ScrollArea for optimal performance
4. **Caching**: Composers can cache results when appropriate

## Best Practices

1. **Feature Composition**: Implement only the features that make sense for your type
2. **Composer Reuse**: Create reusable composers for common patterns
3. **Type Safety**: The system ensures only available methods are exposed
4. **Separation of Concerns**: Keep data separate from presentation logic
5. **Extensibility**: Add new features and composers without modifying existing code

## Integration Examples

See `integration.rs` for complete examples:
- Team builder UI
- Battle view
- Collection browser with search/filter
- Shop with selectable items
- Node explorer with tree view

## Testing

Run tests with:
```rust
cargo test --package client --lib ui::render::test
```

View demo UI:
```rust
run_render_tests(ui, world);
```
