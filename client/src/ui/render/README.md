# Render Module - Feature-Composer System

A modern, composable UI rendering system that separates data capabilities (Features) from rendering logic (Composers).

## Overview

The render module is a spiritual successor to the `see` module, designed to provide better modularity, type safety, and composability for UI rendering. It uses a feature-based approach where types declare their capabilities through feature traits, and composers consume these features to produce UI elements.

## Core Concepts

### Features

Features define what data a type can provide. They are simple, focused traits that expose specific capabilities:

- **`FTitle`** - Provides a title/name
- **`FDescription`** - Provides a description
- **`FStats`** - Provides statistics/variables
- **`FTag`** - Provides compact tag representation
- **`FShow`** - Can be displayed (read-only)
- **`FEdit`** - Can be edited (mutable)
- **`FRecursive`** - Can be recursively traversed
- **`FContextMenu`** - Provides context menu actions
- **`FCopy/FPaste`** - Clipboard operations
- **`FInfo`** - Provides detailed information
- **`FRating`** - Has a rating value
- And many more...

### Composers

Composers transform data into UI elements by consuming features:

- **`TitleComposer`** - Renders titles as buttons
- **`TagComposer`** - Renders compact tags
- **`CardComposer`** - Renders full card views
- **`RecursiveComposer`** - Traverses recursive structures
- **`TagCardComposer`** - Expandable tag/card views
- **`FramedComposer`** - Adds frames around content
- **`WithTooltipComposer`** - Adds hover tooltips
- **`ValidatedComposer`** - Shows validation errors
- And more...

### RenderBuilder

The builder provides a fluent API for rendering, with methods automatically available based on implemented features.

## Basic Usage

### Simple Rendering

```rust
// Any type with FTitle can be rendered as a title
let unit = NUnit::default();
unit.render(context).title(ui);

// Types with FShow can be displayed
let value = 42;
value.render(context).show(ui);

// Types with FEdit can be edited
let mut text = String::from("Hello");
if text.render_mut(context).edit(ui) {
    println!("Text changed!");
}
```

### Node Rendering

```rust
// Tag view (compact)
unit.render(context).tag(ui);

// Card view (full details)
unit.render(context).card(ui);

// Expandable tag/card
unit.render(context).tag_card(ui);
```

### Recursive Rendering

```rust
// Display recursive structure
expression.render(context).recursive_show(ui);

// Edit recursive structure
expression.render_mut(context).recursive_edit(ui);

// Custom recursive rendering
expression.render(context).recursive(ui, |ui, context, field| {
    // Custom rendering for each field
    field.name.label(ui);
    call_on_recursive_value!(field, show, context, ui);
});
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

## Advanced Usage

### Custom Composers

```rust
pub struct MyComposer;

impl<T: FTitle + FDescription> Composer<T> for MyComposer {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            data.title(context).label(ui);
            ui.separator();
            data.description(context).label_w(ui);
        }).response
    }
}

// Usage
item.render(context)
    .with_composer(MyComposer)
    .compose(ui);
```

### Composer Wrapping

```rust
// Add frame around any composer
let framed = FramedComposer::new(CardComposer)
    .with_color(GREEN);

// Add tooltip to any composer
let with_info = WithTooltipComposer::new(TagComposer);

// Chain composers
item.render(context)
    .with_composer(framed)
    .with_composer(with_info)
    .compose(ui);
```

### Custom Features

```rust
// Define a custom feature
pub trait FCustom {
    fn custom_data(&self) -> String;
}

// Implement for your types
impl FCustom for MyType {
    fn custom_data(&self) -> String {
        "Custom data".to_string()
    }
}

// Add builder extension
impl<'a, T: FCustom> RenderBuilder<'a, T> {
    pub fn custom(self, ui: &mut Ui) -> Response {
        self.data().custom_data().button(ui)
    }
}
```

## Migration from `see` Module

The render module is designed to coexist with the see module during migration:

| Old API | New API |
|---------|---------|
| `item.see(ctx).button(ui)` | `item.render(ctx).title_button(ui)` |
| `item.see(ctx).tag(ui)` | `item.render(ctx).tag(ui)` |
| `item.see(ctx).card(ui)` | `item.render(ctx).card(ui)` |
| `item.see(ctx).show(ui)` | `item.render(ctx).show(ui)` |
| `item.see_mut(ctx).show_mut(ui)` | `item.render_mut(ctx).edit(ui)` |
| `item.see(ctx).recursive_show(ui)` | `item.render(ctx).recursive_show(ui)` |

## Benefits

### 1. **Modularity**
Features and composers are independent and can be developed separately.

### 2. **Type Safety**
The compiler ensures required features are present before allowing composer usage.

### 3. **Composability**
Composers can be easily combined, wrapped, and chained.

### 4. **Discoverability**
IDE autocomplete shows only available methods based on implemented features.

### 5. **Extensibility**
New features and composers can be added without modifying existing code.

### 6. **Separation of Concerns**
Data capabilities (features) are separate from presentation logic (composers).

## File Structure

```
render/
├── mod.rs           # Module root with main Render trait
├── features.rs      # Feature trait definitions
├── features_impl.rs # Feature implementations for types
├── composers.rs     # Composer implementations
├── builder.rs       # RenderBuilder and extensions
├── example.rs       # Usage examples
└── README.md        # This file
```

## Design Principles

1. **Small, Focused Features** - Each feature trait should have a single responsibility
2. **Composable Composers** - Composers should be able to wrap and combine with others
3. **Progressive Enhancement** - Basic features enable basic rendering, more features enable richer UI
4. **Type-Driven Development** - Let the type system guide what rendering is possible
5. **Backwards Compatible** - Can coexist with existing see module during migration

## Future Enhancements

- [ ] Async composers for loading data
- [ ] Caching layer for expensive computations
- [ ] Theme-aware composers
- [ ] Animation composers
- [ ] Layout composers (grid, flex, etc.)
- [ ] Virtualized rendering for large lists
- [ ] Drag-and-drop support
- [ ] Accessibility features