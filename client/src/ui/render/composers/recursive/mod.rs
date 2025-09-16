use super::super::*;
use crate::{call_on_recursive_value, call_on_recursive_value_mut};
use std::marker::PhantomData;

mod list_composer;
mod recursive_types;
mod tree_composer;
pub use list_composer::*;
pub use recursive_types::*;
pub use tree_composer::*;

/// Layout mode for recursive rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecursiveLayout {
    /// Horizontal layout with vertical nesting
    HorizontalVertical,
    /// Pure vertical layout
    Vertical,
    /// Pure horizontal layout
    Horizontal,
    /// Tree-like indented layout
    Tree { indent: f32 },
    /// Grid layout
    Grid { columns: usize },
}

/// A unified recursive composer that traverses nested structures
pub struct RecursiveComposer<F> {
    layout: RecursiveLayout,
    field_renderer: std::cell::RefCell<F>,
    show_field_names: bool,
    collapsible: bool,
}

impl<F> RecursiveComposer<F> {
    pub fn new(field_renderer: F) -> Self {
        Self {
            layout: RecursiveLayout::HorizontalVertical,
            field_renderer: std::cell::RefCell::new(field_renderer),
            show_field_names: true,
            collapsible: false,
        }
    }

    pub fn with_layout(mut self, layout: RecursiveLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn show_field_names(mut self, show: bool) -> Self {
        self.show_field_names = show;
        self
    }

    pub fn collapsible(mut self, collapsible: bool) -> Self {
        self.collapsible = collapsible;
        self
    }
}

impl<T, F> Composer<T> for RecursiveComposer<F>
where
    T: FRecursive,
    F: FnMut(&mut Ui, &Context, &RecursiveField<'_>) -> Response,
{
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        let root = RecursiveField::named("root", data.to_recursive_value());
        render_recursive_field(
            &root,
            context,
            ui,
            &self.layout,
            &self.field_renderer,
            self.show_field_names,
            self.collapsible,
            0,
        )
    }
}

impl<T, F> ComposerMut<T> for RecursiveComposer<F>
where
    T: FRecursive,
    F: FnMut(&mut Ui, &Context, &mut RecursiveFieldMut<'_>) -> bool,
{
    fn compose_mut(&self, data: &mut T, context: &Context, ui: &mut Ui) -> bool {
        let mut root = RecursiveFieldMut::named("root", data.to_recursive_value_mut());
        render_recursive_field_mut(
            &mut root,
            context,
            ui,
            &self.layout,
            &self.field_renderer,
            self.show_field_names,
            self.collapsible,
            0,
        )
    }
}

/// Render a recursive field based on layout
fn render_recursive_field<F>(
    field: &RecursiveField<'_>,
    context: &Context,
    ui: &mut Ui,
    layout: &RecursiveLayout,
    renderer: &std::cell::RefCell<F>,
    show_names: bool,
    collapsible: bool,
    depth: usize,
) -> Response
where
    F: FnMut(&mut Ui, &Context, &RecursiveField<'_>) -> Response,
{
    match layout {
        RecursiveLayout::HorizontalVertical => {
            ui.horizontal(|ui| {
                let field_response = show_grouped_field(field, context, renderer, show_names, ui);

                // Get inner fields
                let inner_fields = call_on_recursive_value!(field, get_inner_fields);
                if !inner_fields.is_empty() {
                    ui.vertical(|ui| {
                        for inner_field in inner_fields {
                            render_recursive_field(
                                &inner_field,
                                context,
                                ui,
                                layout,
                                renderer,
                                show_names,
                                collapsible,
                                depth + 1,
                            );
                        }
                    });
                }

                field_response
            })
            .inner
        }
        RecursiveLayout::Vertical => {
            ui.vertical(|ui| {
                let mut response = show_grouped_field(field, context, renderer, show_names, ui);

                let inner_fields = call_on_recursive_value!(field, get_inner_fields);
                for inner_field in inner_fields {
                    response |= response.union(render_recursive_field(
                        &inner_field,
                        context,
                        ui,
                        layout,
                        renderer,
                        show_names,
                        collapsible,
                        depth + 1,
                    ));
                }
                response
            })
            .inner
        }
        RecursiveLayout::Horizontal => {
            ui.horizontal(|ui| {
                let mut response = show_grouped_field(field, context, renderer, show_names, ui);

                let inner_fields = call_on_recursive_value!(field, get_inner_fields);
                for inner_field in inner_fields {
                    response |= response.union(render_recursive_field(
                        &inner_field,
                        context,
                        ui,
                        layout,
                        renderer,
                        show_names,
                        collapsible,
                        depth + 1,
                    ));
                }
                response
            })
            .inner
        }
        RecursiveLayout::Tree { indent } => {
            ui.horizontal(|ui| {
                ui.add_space(indent * depth as f32);

                let inner_fields = call_on_recursive_value!(field, get_inner_fields);
                let has_children = !inner_fields.is_empty();

                if has_children && collapsible {
                    let id = ui.id().with(field.name.as_str()).with(depth);
                    let expanded = ui.ctx().data(|r| r.get_temp::<bool>(id)).unwrap_or(true);

                    if ui.button(if expanded { "▼" } else { "▶" }).clicked() {
                        ui.ctx().data_mut(|w| w.insert_temp(id, !expanded));
                    }

                    if show_names && !field.name.is_empty() && field.name != "root" {
                        ui.label(format!("{}:", field.name));
                    }

                    let mut response = renderer.borrow_mut()(ui, context, field);

                    if expanded {
                        ui.vertical(|ui| {
                            for inner_field in inner_fields {
                                response |= response.union(render_recursive_field(
                                    &inner_field,
                                    context,
                                    ui,
                                    layout,
                                    renderer,
                                    show_names,
                                    collapsible,
                                    depth + 1,
                                ));
                            }
                        });
                    }
                    response
                } else {
                    let mut response = show_grouped_field(field, context, renderer, show_names, ui);

                    if has_children {
                        ui.vertical(|ui| {
                            for inner_field in inner_fields {
                                response |= response.union(render_recursive_field(
                                    &inner_field,
                                    context,
                                    ui,
                                    layout,
                                    renderer,
                                    show_names,
                                    collapsible,
                                    depth + 1,
                                ));
                            }
                        });
                    }
                    response
                }
            })
            .inner
        }
        RecursiveLayout::Grid { columns } => {
            egui::Grid::new(ui.id().with("recursive_grid").with(depth))
                .num_columns(*columns)
                .show(ui, |ui| {
                    let mut response = show_grouped_field(field, context, renderer, show_names, ui);
                    ui.end_row();

                    let inner_fields = call_on_recursive_value!(field, get_inner_fields);
                    for (i, inner_field) in inner_fields.iter().enumerate() {
                        response |= response.union(render_recursive_field(
                            inner_field,
                            context,
                            ui,
                            layout,
                            renderer,
                            show_names,
                            collapsible,
                            depth + 1,
                        ));
                        if (i + 1) % columns == 0 {
                            ui.end_row();
                        }
                    }
                    response
                })
                .inner
        }
    }
}

/// Render a mutable recursive field based on layout
fn render_recursive_field_mut<F>(
    field: &mut RecursiveFieldMut<'_>,
    context: &Context,
    ui: &mut Ui,
    layout: &RecursiveLayout,
    renderer: &std::cell::RefCell<F>,
    show_names: bool,
    collapsible: bool,
    depth: usize,
) -> bool
where
    F: FnMut(&mut Ui, &Context, &mut RecursiveFieldMut<'_>) -> bool,
{
    let mut changed = false;

    match layout {
        RecursiveLayout::HorizontalVertical => {
            ui.horizontal(|ui| {
                show_grouped_field_mut(field, context, renderer, show_names, &mut changed, ui);
                let inner_fields = call_on_recursive_value_mut!(field, get_inner_fields_mut);
                if !inner_fields.is_empty() {
                    ui.vertical(|ui| {
                        for mut inner_field in inner_fields {
                            changed |= render_recursive_field_mut(
                                &mut inner_field,
                                context,
                                ui,
                                layout,
                                renderer,
                                show_names,
                                collapsible,
                                depth + 1,
                            );
                        }
                    });
                }
            });
        }
        RecursiveLayout::Vertical => {
            ui.vertical(|ui| {
                if show_names && !field.name.is_empty() && field.name != "root" {
                    ui.label(format!("{}:", field.name));
                }

                changed |= renderer.borrow_mut()(ui, context, field);

                let inner_fields = call_on_recursive_value_mut!(field, get_inner_fields_mut);
                for mut inner_field in inner_fields {
                    changed |= render_recursive_field_mut(
                        &mut inner_field,
                        context,
                        ui,
                        layout,
                        renderer,
                        show_names,
                        collapsible,
                        depth + 1,
                    );
                }
            });
        }
        RecursiveLayout::Horizontal => {
            ui.horizontal(|ui| {
                if show_names && !field.name.is_empty() && field.name != "root" {
                    ui.label(format!("{}:", field.name));
                }

                changed |= renderer.borrow_mut()(ui, context, field);

                let inner_fields = call_on_recursive_value_mut!(field, get_inner_fields_mut);
                for mut inner_field in inner_fields {
                    changed |= render_recursive_field_mut(
                        &mut inner_field,
                        context,
                        ui,
                        layout,
                        renderer,
                        show_names,
                        collapsible,
                        depth + 1,
                    );
                }
            });
        }
        RecursiveLayout::Tree { indent } => {
            ui.horizontal(|ui| {
                ui.add_space(indent * depth as f32);

                let inner_fields_check = call_on_recursive_value_mut!(field, get_inner_fields_mut);
                let has_children = !inner_fields_check.is_empty();

                if has_children && collapsible {
                    let id = ui.id().with(field.name.as_str()).with(depth);
                    let expanded = ui.ctx().data(|r| r.get_temp::<bool>(id)).unwrap_or(true);

                    if ui.button(if expanded { "▼" } else { "▶" }).clicked() {
                        ui.ctx().data_mut(|w| w.insert_temp(id, !expanded));
                    }

                    show_grouped_field_mut(field, context, renderer, show_names, &mut changed, ui);

                    if expanded {
                        ui.vertical(|ui| {
                            let inner_fields =
                                call_on_recursive_value_mut!(field, get_inner_fields_mut);
                            for mut inner_field in inner_fields {
                                changed |= render_recursive_field_mut(
                                    &mut inner_field,
                                    context,
                                    ui,
                                    layout,
                                    renderer,
                                    show_names,
                                    collapsible,
                                    depth + 1,
                                );
                            }
                        });
                    }
                } else {
                    show_grouped_field_mut(field, context, renderer, show_names, &mut changed, ui);

                    if has_children {
                        ui.vertical(|ui| {
                            let inner_fields =
                                call_on_recursive_value_mut!(field, get_inner_fields_mut);
                            for mut inner_field in inner_fields {
                                changed |= render_recursive_field_mut(
                                    &mut inner_field,
                                    context,
                                    ui,
                                    layout,
                                    renderer,
                                    show_names,
                                    collapsible,
                                    depth + 1,
                                );
                            }
                        });
                    }
                }
            });
        }
        RecursiveLayout::Grid { columns } => {
            egui::Grid::new(ui.id().with("recursive_grid_mut").with(depth))
                .num_columns(*columns)
                .show(ui, |ui| {
                    show_grouped_field_mut(field, context, renderer, show_names, &mut changed, ui);
                    ui.end_row();

                    let inner_fields = call_on_recursive_value_mut!(field, get_inner_fields_mut);
                    for (i, mut inner_field) in inner_fields.into_iter().enumerate() {
                        changed |= render_recursive_field_mut(
                            &mut inner_field,
                            context,
                            ui,
                            layout,
                            renderer,
                            show_names,
                            collapsible,
                            depth + 1,
                        );
                        if (i + 1) % columns == 0 {
                            ui.end_row();
                        }
                    }
                });
        }
    }

    changed
}

fn show_grouped_field_mut<F>(
    field: &mut RecursiveFieldMut<'_>,
    context: &Context<'_>,
    renderer: &RefCell<F>,
    show_names: bool,
    changed: &mut bool,
    ui: &mut Ui,
) where
    F: FnMut(&mut Ui, &Context, &mut RecursiveFieldMut<'_>) -> bool,
{
    ui.group(|ui| {
        ui.vertical(|ui| {
            if show_names && !field.name.is_empty() && field.name != "root" {
                format!("[s {}:]", field.name).label(ui);
            }
            *changed |= renderer.borrow_mut()(ui, context, field);
        });
    });
}

fn show_grouped_field<F>(
    field: &RecursiveField<'_>,
    context: &Context<'_>,
    renderer: &RefCell<F>,
    show_names: bool,
    ui: &mut Ui,
) -> Response
where
    F: FnMut(&mut Ui, &Context, &RecursiveField<'_>) -> Response,
{
    ui.group(|ui| {
        ui.vertical(|ui| {
            if show_names && !field.name.is_empty() && field.name != "root" {
                format!("[s {}:]", field.name).label(ui);
            }
            renderer.borrow_mut()(ui, context, field)
        })
    })
    .inner
    .inner
}

/// Default field renderer for display
pub fn default_field_renderer(
    ui: &mut Ui,
    context: &Context,
    field: &RecursiveField<'_>,
) -> Response {
    call_on_recursive_value!(field, display, context, ui);
    ui.label("")
}

/// Default field renderer for editing
pub fn default_field_renderer_mut(
    ui: &mut Ui,
    context: &Context,
    field: &mut RecursiveFieldMut<'_>,
) -> bool {
    call_on_recursive_value_mut!(field, edit, context, ui)
}

/// Helper to create a display composer with custom layout
pub fn recursive_display_composer(
    layout: RecursiveLayout,
) -> RecursiveComposer<impl FnMut(&mut Ui, &Context, &RecursiveField<'_>) -> Response> {
    RecursiveComposer::new(default_field_renderer).with_layout(layout)
}

/// Helper to create an edit composer with custom layout
pub fn recursive_edit_composer(
    layout: RecursiveLayout,
) -> RecursiveComposer<impl FnMut(&mut Ui, &Context, &mut RecursiveFieldMut<'_>) -> bool> {
    RecursiveComposer::new(default_field_renderer_mut).with_layout(layout)
}
