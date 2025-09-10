//! Test module for the render system
//! This module contains tests and examples to verify the rendering system works correctly

use super::*;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_trait_available() {
        // Test that the Render trait is available for basic types
        let value = 42;
        let context = Context::empty();
        let _builder = value.render(&context);

        let text = "Hello".to_string();
        let _builder = text.render(&context);
    }

    #[test]
    fn test_feature_implementations() {
        // Test that basic types implement features
        let value = 42;
        let context = Context::empty();

        // FTitle is implemented
        let _title = <i32 as FTitle>::title(&value, &context);

        // FDisplay is implemented
        let _display_fn = <i32 as FDisplay>::display;
    }
}

/// Example function showing the render system in action
pub fn demo_render_system(ui: &mut Ui, world: &World) {
    Context::from_world_ref_r(world, |context| {
        ui.heading("Render System Demo");

        demo_basic_types(context, ui);
        ui.separator();

        demo_node_rendering(context, ui);
        ui.separator();

        demo_advanced_features(context, ui);
        Ok(())
    })
    .log();
}

fn demo_basic_types(context: &Context, ui: &mut Ui) {
    ui.label("Basic Type Rendering:");

    // Render numbers
    let number = 42;
    number.render(context).display(ui);

    // Render strings
    let text = "Hello World".to_string();
    text.render(context).display(ui);

    // Render booleans
    let flag = true;
    flag.render(context).display(ui);

    // Render vectors
    let vec = vec![1, 2, 3, 4, 5];
    vec.render(context).display(ui);

    // Render options
    let some_value: Option<i32> = Some(100);
    some_value.render(context).display(ui);

    let none_value: Option<i32> = None;
    none_value.render(context).display(ui);
}

fn demo_node_rendering(context: &Context, ui: &mut Ui) {
    ui.label("Node Rendering:");

    // Create sample nodes for demonstration
    let unit = NUnit {
        id: 1,
        owner: 1,
        entity: Some(Entity::from_raw(1)),
        unit_name: "Demo Unit".to_string(),
        description: NodePart::default(),
        stats: NodePart::default(),
        state: NodePart::default(),
    };

    // Render as title
    ui.horizontal(|ui| {
        ui.label("Title:");
        unit.render(context).title(ui);
    });

    // Render as tag
    ui.horizontal(|ui| {
        ui.label("Tag:");
        unit.render(context).tag(ui);
    });

    // Render as card
    ui.horizontal(|ui| {
        ui.label("Card:");
        unit.render(context).card(ui);
    });

    // Render with context menu
    ui.horizontal(|ui| {
        ui.label("With Menu:");
        let response = unit
            .render(context)
            .with_menu()
            .add_copy()
            .add_delete()
            .title(ui);

        if response.clicked() {
            println!("Unit clicked!");
        }
    });
}

fn demo_advanced_features(context: &Context, ui: &mut Ui) {
    ui.label("Advanced Features:");

    // List rendering
    let items = vec![
        "Item 1".to_string(),
        "Item 2".to_string(),
        "Item 3".to_string(),
    ];

    ui.label("List Composer:");
    let list_composer = ListComposer::new(TitleComposer).with_max_items(2);
    items
        .render(context)
        .with_composer(list_composer)
        .compose(ui);

    // Filtered list
    ui.label("Filtered List:");
    fn filter_contains_2(item: &String, _: &Context) -> bool {
        item.contains("2")
    }
    let filtered = FilteredListComposer::new(TitleComposer, filter_contains_2)
        .with_empty_message("No matching items".to_string());
    items.render(context).with_composer(filtered).compose(ui);

    // Sorted list
    ui.label("Sorted List (reversed):");
    fn sort_by_string(item: &String, _: &Context) -> String {
        item.clone()
    }
    let sorted = SortedListComposer::new(TitleComposer, sort_by_string).reversed();
    items.render(context).with_composer(sorted).compose(ui);

    // Grouped list
    ui.label("Grouped List:");
    fn group_by_last_char(item: &String, _: &Context) -> char {
        item.chars().last().unwrap_or('?')
    }
    let grouped = GroupedComposer::new(TitleComposer, group_by_last_char);
    items.render(context).with_composer(grouped).compose(ui);
}

/// Example of creating a custom feature
pub trait FCustomExample {
    fn custom_render(&self) -> String;
}

impl FCustomExample for i32 {
    fn custom_render(&self) -> String {
        format!("Custom: {}", self)
    }
}

/// Example of creating a custom composer
pub struct CustomComposer;

impl<T: FCustomExample> Composer<T> for CustomComposer {
    fn compose(&self, data: &T, _context: &Context, ui: &mut Ui) -> Response {
        ui.label(data.custom_render())
    }
}

/// Example showing how to extend the render system
pub fn demo_extensibility(context: &Context, ui: &mut Ui) {
    ui.heading("Extensibility Demo");

    let value = 42;

    // Use custom composer
    value
        .render(context)
        .with_composer(CustomComposer)
        .compose(ui);

    // Chain composers
    let chain = ComposerChain::new().add(TitleComposer).add(CustomComposer);

    value.render(context).with_composer(chain).compose(ui);

    // Wrap with frame
    let framed = FramedComposer::new(TitleComposer).with_color(Color32::from_rgb(100, 100, 255));

    value.render(context).with_composer(framed).compose(ui);
}

/// Performance test showing efficient rendering of many items
pub fn demo_performance(context: &Context, ui: &mut Ui) {
    ui.heading("Performance Demo");

    // Generate many items
    let items: Vec<i32> = (0..1000).collect();

    // Use pagination for efficient rendering
    let paginated = PaginatedComposer::new(TitleComposer, 50);

    ScrollArea::vertical().max_height(200.0).show(ui, |ui| {
        items.render(context).with_composer(paginated).compose(ui);
    });
}

/// Interactive demo with mutable rendering
pub fn demo_interactive(context: &Context, ui: &mut Ui) {
    ui.heading("Interactive Demo");

    // Note: In real usage, this would be stored in state
    let mut value = 42;
    let mut text = "Editable text".to_string();
    let mut flag = false;

    ui.horizontal(|ui| {
        ui.label("Edit number:");
        if value.render_mut(context).edit(ui) {
            println!("Number changed to: {}", value);
        }
    });

    ui.horizontal(|ui| {
        ui.label("Edit text:");
        if text.render_mut(context).edit(ui) {
            println!("Text changed to: {}", text);
        }
    });

    ui.horizontal(|ui| {
        ui.label("Edit flag:");
        if flag.render_mut(context).edit(ui) {
            println!("Flag changed to: {}", flag);
        }
    });
}

/// Main entry point for testing the render system
pub fn run_render_tests(ui: &mut Ui, world: &World) {
    ScrollArea::vertical().show(ui, |ui| {
        demo_render_system(ui, world);
        ui.separator();

        Context::from_world_ref_r(world, |context| {
            demo_extensibility(context, ui);
            ui.separator();

            demo_performance(context, ui);
            ui.separator();

            demo_interactive(context, ui);
            Ok(())
        })
        .log();
    });
}
