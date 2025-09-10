# Battle Editor Migration Guide

This document explains how to migrate the battle_editor.rs from the old `see()` API to the new `render()` API.

## Key Changes

### 1. Import the render module
```rust
use crate::ui::render::*;
```

### 2. Basic Display Migration

**Old API:**
```rust
node.see(context).info().label(ui);
node.see(context).button(ui);
node.see(context).tag(ui);
node.see(context).card(ui);
```

**New API:**
```rust
// For nodes that implement FInfo
node.render(context).info().label(ui);

// For nodes that implement FTitle
node.render(context).title_button(ui);

// For nodes that implement FTag
node.render(context).tag(ui);

// For nodes that implement FTitle + FDescription + FStats
node.render(context).card(ui);
```

### 3. Context Menu Migration

**Old API:**
```rust
let btn_response = node.see(context).node_ctxbtn_full().ui(ui);
if btn_response.deleted() {
    // handle deletion
}
if let Some(pasted) = btn_response.pasted() {
    // handle paste
}
```

**New API (for nodes that implement FContextMenu + FTitle + FCopy + FPaste):**
```rust
let response = node.render(context)
    .with_menu()
    .add_copy()
    .add_paste()
    .add_delete()
    .title(ui);

if let Some(deleted) = response.deleted() {
    // handle deletion
}
if let Some(replaced) = response.replaced() {
    // handle paste/replacement
}
```

### 4. Editing Migration

**Old API:**
```rust
changed |= node.see_mut(context).show_mut(ui);
```

**New API (for nodes that implement FEdit):**
```rust
changed |= node.render_mut(context).edit(ui);
```

### 5. Recursive Display Migration

**Old API:**
```rust
node.see(context).recursive_show(ui);
```

**New API (for nodes that implement FRecursive):**
```rust
node.render(context).recursive_show(ui);
```

## Partial Migration Strategy

Since not all nodes implement all features yet, you can use a hybrid approach:

1. Use the new `render()` API where features are implemented
2. Fall back to `see()` API where features are missing
3. Gradually implement missing features

### Example Hybrid Approach:

```rust
// Check if the node implements the required features
if let Some(info_provider) = (node as &dyn std::any::Any).downcast_ref::<&dyn FInfo>() {
    // Use new API
    info_provider.info(context).label(ui);
} else {
    // Fall back to old API
    node.see(context).info().label(ui);
}
```

## Feature Implementation Status

### Fully Implemented Features:
- `FTitle` - All nodes
- `FTag` - Most nodes (NUnit, NHouse, NAbilityMagic, NStatusMagic, etc.)
- `FDescription` - Magic nodes and units
- `FStats` - Units and some other nodes
- `FDisplay` - Basic types and many nodes
- `FEdit` - Basic types and some nodes

### Partially Implemented:
- `FContextMenu` - Implemented but requires FCopy/FPaste
- `FRecursive` - Implemented for Expression, Action, etc.
- `FCopy/FPaste` - Implemented for some nodes

### Not Yet Implemented:
- `FInfo` for all nodes (currently using see().info())
- Complex editing for all nodes
- Validation features

## Recommended Migration Path

1. **Phase 1: Display Only**
   - Replace `.see().button()` with `.render().title_button()`
   - Replace `.see().tag()` with `.render().tag()`
   - Keep `.see().info()` for now

2. **Phase 2: Context Menus**
   - For nodes with FCopy/FPaste, use `.render().with_menu()`
   - For others, keep using `.see().node_ctxbtn_full()`

3. **Phase 3: Editing**
   - Implement FEdit for nodes that need editing
   - Replace `.see_mut().show_mut()` with `.render_mut().edit()`

4. **Phase 4: Complete Migration**
   - Implement remaining features
   - Remove all `.see()` calls

## Example Migration

Here's a simple example of migrating a node display function:

**Before:**
```rust
fn show_node<T: Node + SFnInfo>(node: &T, context: &Context, ui: &mut Ui) {
    ui.group(|ui| {
        node.see(context).info().label(ui);
        node.see(context).button(ui);
    });
}
```

**After:**
```rust
fn show_node<T: Node + FTitle>(node: &T, context: &Context, ui: &mut Ui) {
    ui.group(|ui| {
        // Use hybrid approach for info
        if let Some(info_provider) = (node as &dyn std::any::Any).downcast_ref::<&dyn FInfo>() {
            info_provider.info(context).label(ui);
        } else {
            // Fall back to old API
            node.see(context).info().label(ui);
        }
        
        // Use new API for title
        node.render(context).title_button(ui);
    });
}
```

## Benefits of Migration

1. **Better Type Safety**: The compiler ensures features are implemented
2. **Composability**: Composers can be mixed and matched
3. **Separation of Concerns**: Data capabilities separate from rendering
4. **Extensibility**: Easy to add new features without modifying existing code
5. **Performance**: More efficient rendering with specialized composers

## Current Limitations

1. Not all nodes implement all features yet
2. Some complex interactions still require the old API
3. Context menus require multiple trait implementations
4. FEdit is not implemented for all editable nodes

## Next Steps

1. Continue using the hybrid approach
2. Gradually implement missing features for nodes
3. Replace see() calls as features become available
4. Eventually deprecate the see() API