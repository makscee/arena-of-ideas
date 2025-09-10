//! Examples demonstrating the new render system
//!
//! This file shows how to use the feature-composer rendering system
//! as a spiritual successor to the see module.

use super::*;
use crate::ui::see::{Cstr, CstrTrait};
use crate::{call_on_recursive_value, call_on_recursive_value_mut};

/// Example of basic rendering with the new system
pub fn example_basic_rendering(ui: &mut Ui, context: &Context) {
    ui.heading("Basic Rendering Examples");

    // Simple types automatically get render methods
    let number = 42;
    number.render(context).title(ui);

    let text = "Hello World".to_string();
    text.render(context).display(ui);

    // Mutable rendering for editing
    let mut value = 3.14;
    if value.render_mut(context).edit(ui) {
        println!("Value changed to: {}", value);
    }
}

/// Example of rendering game nodes
pub fn example_node_rendering(unit: &NUnit, context: &Context, ui: &mut Ui) {
    ui.heading("Node Rendering Examples");

    // Simple title button (replaces see().button())
    unit.render(context).title_button(ui);

    // Tag view (compact representation)
    unit.render(context).tag(ui);

    // Full card view (requires FTitle + FDescription + FStats)
    unit.render(context).card(ui);

    // Expandable tag/card view
    unit.render(context).tag_card(ui);

    // With explicit expanded state
    unit.render(context).tag_card_expanded(true, ui);
}

/// Example of recursive rendering
pub fn example_recursive_rendering(expression: &Expression, context: &Context, ui: &mut Ui) {
    ui.heading("Recursive Rendering");

    // Simple recursive display (read-only)
    expression.render(context).recursive_show(ui);

    // Custom recursive rendering with closure
    expression
        .render(context)
        .recursive(ui, |ui, context, field| {
            ui.horizontal(|ui| {
                if !field.name.is_empty() {
                    // Show field name
                    format!("{}: ", field.name).cstr_c(YELLOW).label(ui);
                }
                // Show field value
                call_on_recursive_value!(field, display, context, ui);
            });
        });
}

/// Example of recursive editing
pub fn example_recursive_editing(expression: &mut Expression, context: &Context, ui: &mut Ui) {
    ui.heading("Recursive Editing");

    // Simple recursive edit (with default editor)
    if expression.render_mut(context).recursive_edit(ui) {
        println!("Expression modified!");
    }

    // Custom recursive editing
    let mut changed = false;
    expression
        .render_mut(context)
        .recursive_mut(ui, |ui, context, field| {
            ui.horizontal(|ui| {
                if !field.name.is_empty() {
                    field.name.label(ui);
                    ui.label(":");
                }
                // Edit field value
                changed |= call_on_recursive_value_mut!(field, edit, context, ui);
            });
        });

    if changed {
        println!("Expression was modified through custom editor");
    }
}

/// Example of context menu integration
pub fn example_context_menu(unit: &mut NUnit, context: &Context, ui: &mut Ui) {
    ui.heading("Context Menu Examples");

    // Basic context menu with title
    let response = unit
        .render(context)
        .with_menu()
        .add_copy()
        .add_paste()
        .add_delete()
        .title(ui);

    if let Some(deleted) = response.deleted() {
        println!("Unit deleted: {:?}", deleted);
    }

    if let Some(replaced) = response.replaced() {
        println!("Unit replaced with: {:?}", replaced);
    }

    // Context menu with card view
    unit.render(context)
        .with_menu()
        .add_copy()
        .add_paste()
        .add_separator()
        .add_action("🔄 Reset".to_string(), |item, _ctx| {
            println!("Resetting unit: {}", item.unit_name);
            Some(ActionResult::Modified(item))
        })
        .add_dangerous_separator()
        .add_delete()
        .card(ui);
}

/// Example of composer chaining and wrapping
pub fn example_composer_patterns(unit: &NUnit, context: &Context, ui: &mut Ui) {
    ui.heading("Composer Patterns");

    // Add frame to any composer
    let framed_title = FramedComposer::new(TitleComposer).with_color(GREEN);
    unit.render(context).with_composer(framed_title).compose(ui);

    // Add tooltip to any composer
    let with_tooltip = WithTooltipComposer::new(CardComposer);
    unit.render(context).with_composer(with_tooltip).compose(ui);

    // Collapsing header wrapper
    let collapsing =
        CollapsingComposer::new(CardComposer, "Unit Details".to_string()).default_open(true);
    unit.render(context).with_composer(collapsing).compose(ui);
}

/// Example showing the migration from old see() API to new render() API
pub fn example_migration_comparison(unit: &NUnit, context: &Context, ui: &mut Ui) {
    ui.heading("Migration from see() to render()");

    ui.label("Old API:");
    ui.code("unit.see(context).button(ui);");
    ui.code("unit.see(context).tag(ui);");
    ui.code("unit.see(context).card(ui);");
    ui.code("unit.see(context).recursive_show(ui);");

    ui.separator();

    ui.label("New API:");
    ui.code("unit.render(context).title_button(ui);");
    ui.code("unit.render(context).tag(ui);");
    ui.code("unit.render(context).card(ui);");
    ui.code("unit.render(context).recursive_show(ui);");

    ui.separator();

    ui.label("Benefits of new system:");
    ui.label("• Better composability - composers can be mixed and matched");
    ui.label("• Type safety - only available methods for implemented features");
    ui.label("• Extensibility - easy to add new features and composers");
    ui.label("• Cleaner separation of concerns - data vs presentation");
}

/// Example of creating custom features and composers
pub fn example_custom_extensions() {
    // Custom feature for types that can be starred/favorited
    pub trait FFavorite {
        fn is_favorite(&self) -> bool;
        fn toggle_favorite(&mut self);
    }

    // Custom composer that adds a star icon
    pub struct FavoriteComposer<C> {
        inner: C,
    }

    impl<T: FFavorite, C: Composer<T>> Composer<T> for FavoriteComposer<C> {
        fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
            ui.horizontal(|ui| {
                if data.is_favorite() {
                    "⭐".cstr_c(YELLOW).label(ui);
                } else {
                    "☆".cstr_c(subtle_borders_and_separators()).label(ui);
                }
                self.inner.compose(data, context, ui)
            })
            .inner
        }
    }

    // Usage would be:
    // item.render(context)
    //     .with_composer(FavoriteComposer::new(TitleComposer))
    //     .compose(ui);
}

/// Example of feature detection and conditional rendering
pub fn example_feature_detection<T>(item: &T, context: &Context, ui: &mut Ui)
where
    T: FTitle,
{
    // This function works with any type that has FTitle
    item.render(context).title(ui);

    // Additional features can be checked at compile time
    // The builder only exposes methods for available features
}

/// Complex example combining multiple concepts
pub fn example_complex_ui(ui: &mut Ui, context: &Context) {
    ui.heading("Complex UI Example");

    // Imagine we have a list of mixed game items
    // Each can be rendered according to its capabilities

    ScrollArea::vertical().show(ui, |ui| {
        // Units with full card display
        if let Ok(units) = context.collect_children_components::<NUnit>(0) {
            for unit in units {
                unit.render(context)
                    .with_menu()
                    .add_copy()
                    .add_paste()
                    .add_delete()
                    .tag_card(ui);
            }
        }

        // Houses with simple tags
        if let Ok(houses) = context.collect_children_components::<NHouse>(0) {
            for house in houses {
                house.render(context).tag(ui);
            }
        }

        // Abilities with tooltips
        if let Ok(abilities) = context.collect_children_components::<NAbilityMagic>(0) {
            for ability in abilities {
                let composer = WithTooltipComposer::new(TagComposer);
                ability.render(context).with_composer(composer).compose(ui);
            }
        }
    });
}

/// Example showing how the system handles different data states
pub fn example_data_states(ui: &mut Ui, context: &Context) {
    ui.heading("Data State Examples");

    // Immutable rendering
    let value = 42;
    value.render(context).display(ui);

    // Mutable rendering
    let mut value = 42;
    value.render_mut(context).edit(ui);

    // Optional values
    let maybe_value: Option<i32> = Some(42);
    maybe_value.render(context).display(ui);

    let none_value: Option<i32> = None;
    none_value.render(context).display(ui);

    // Vectors
    let values = vec![1, 2, 3, 4, 5];
    values.render(context).recursive_show(ui);
}

/// Main example entry point
pub fn run_examples(ui: &mut Ui, world: &World) {
    Context::from_world(world, |context| {
        ScrollArea::vertical().show(ui, |ui| {
            example_basic_rendering(ui, context);
            ui.separator();

            // Get some sample data
            if let Ok(unit) = context.component::<NUnit>(Entity::from_bits(1)) {
                example_node_rendering(&unit, context, ui);
                ui.separator();
            }

            let mut expression = Expression::f32(42.0);
            example_recursive_rendering(&expression, context, ui);
            ui.separator();

            example_recursive_editing(&mut expression, context, ui);
            ui.separator();

            example_migration_comparison(&NUnit::default(), context, ui);
            ui.separator();

            example_complex_ui(ui, context);
            ui.separator();

            example_data_states(ui, context);
        });
    });
}
