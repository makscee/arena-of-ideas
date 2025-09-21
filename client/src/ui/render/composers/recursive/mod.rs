use super::*;
use crate::{call_on_recursive_value, call_on_recursive_value_mut};

mod recursive_types;
pub use recursive_types::*;

/// Recursive composer that wraps data implementing FRecursive
pub struct RecursiveComposer<'a, T: FRecursive> {
    data: DataRef<'a, T>,
    show_field_names: bool,
    layout: RecursiveLayout,
}

/// Layout options for recursive rendering
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecursiveLayout {
    HorizontalVertical,
    Vertical,
    Horizontal,
    Tree { indent: f32 },
}

impl<'a, T: FRecursive> RecursiveComposer<'a, T> {
    pub fn new(data: &'a T) -> Self {
        Self {
            data: DataRef::Immutable(data),
            show_field_names: true,
            layout: RecursiveLayout::HorizontalVertical,
        }
    }

    pub fn new_mut(data: &'a mut T) -> Self {
        Self {
            data: DataRef::Mutable(data),
            show_field_names: true,
            layout: RecursiveLayout::HorizontalVertical,
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
}

impl<'a, T: FRecursive> Composer<T> for RecursiveComposer<'a, T> {
    fn data(&self) -> &T {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut T {
        self.data.as_mut()
    }

    fn is_mutable(&self) -> bool {
        self.data.is_mutable()
    }

    fn compose(&self, context: &Context, ui: &mut Ui) -> Response {
        let root = RecursiveField::named("root", self.data.as_ref().to_recursive_value());
        render_recursive_field(&root, context, ui, &self.layout, self.show_field_names, 0)
    }
}

fn render_recursive_field(
    field: &RecursiveField<'_>,
    context: &Context,
    ui: &mut Ui,
    layout: &RecursiveLayout,
    show_names: bool,
    depth: usize,
) -> Response {
    match layout {
        RecursiveLayout::HorizontalVertical => {
            ui.horizontal(|ui| {
                let mut response = if show_names && !field.name.is_empty() && field.name != "root" {
                    ui.label(format!("{}:", field.name))
                } else {
                    ui.label("")
                };

                response = response.union(call_on_recursive_value!(field, display, context, ui));

                let inner_fields = call_on_recursive_value!(field, get_inner_fields);
                if !inner_fields.is_empty() {
                    response = response.union(
                        ui.vertical(|ui| {
                            let mut resp = ui.label("");
                            for inner_field in inner_fields {
                                resp = resp.union(render_recursive_field(
                                    &inner_field,
                                    context,
                                    ui,
                                    layout,
                                    show_names,
                                    depth + 1,
                                ));
                            }
                            resp
                        })
                        .inner,
                    );
                }
                response
            })
            .inner
        }
        RecursiveLayout::Vertical => {
            ui.vertical(|ui| {
                let mut response = if show_names && !field.name.is_empty() && field.name != "root" {
                    ui.label(format!("{}:", field.name))
                } else {
                    ui.label("")
                };

                response = response.union(call_on_recursive_value!(field, display, context, ui));

                let inner_fields = call_on_recursive_value!(field, get_inner_fields);
                for inner_field in inner_fields {
                    response = response.union(render_recursive_field(
                        &inner_field,
                        context,
                        ui,
                        layout,
                        show_names,
                        depth + 1,
                    ));
                }
                response
            })
            .inner
        }
        RecursiveLayout::Horizontal => {
            ui.horizontal(|ui| {
                let mut response = if show_names && !field.name.is_empty() && field.name != "root" {
                    ui.label(format!("{}:", field.name))
                } else {
                    ui.label("")
                };

                response = response.union(call_on_recursive_value!(field, display, context, ui));

                let inner_fields = call_on_recursive_value!(field, get_inner_fields);
                for inner_field in inner_fields {
                    response = response.union(render_recursive_field(
                        &inner_field,
                        context,
                        ui,
                        layout,
                        show_names,
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

                let mut response = ui.label("");

                if has_children {
                    let id = ui.id().with(field.name.as_str()).with(depth);
                    let expanded = ui.ctx().data(|r| r.get_temp::<bool>(id)).unwrap_or(true);

                    if ui.button(if expanded { "▼" } else { "▶" }).clicked() {
                        ui.ctx().data_mut(|w| w.insert_temp(id, !expanded));
                    }

                    if show_names && !field.name.is_empty() && field.name != "root" {
                        response = response.union(ui.label(format!("{}:", field.name)));
                    }

                    response =
                        response.union(call_on_recursive_value!(field, display, context, ui));

                    if expanded {
                        response = response.union(
                            ui.vertical(|ui| {
                                let mut resp = ui.label("");
                                for inner_field in inner_fields {
                                    resp = resp.union(render_recursive_field(
                                        &inner_field,
                                        context,
                                        ui,
                                        layout,
                                        show_names,
                                        depth + 1,
                                    ));
                                }
                                resp
                            })
                            .inner,
                        );
                    }
                } else {
                    if show_names && !field.name.is_empty() && field.name != "root" {
                        response = response.union(ui.label(format!("{}:", field.name)));
                    }
                    response =
                        response.union(call_on_recursive_value!(field, display, context, ui));
                }

                response
            })
            .inner
        }
    }
}

/// List composer for Vec<T> where T: FRecursive
pub struct RecursiveListComposer<'a, T: FRecursive> {
    data: DataRef<'a, Vec<T>>,
    show_index: bool,
}

impl<'a, T: FRecursive + Clone> RecursiveListComposer<'a, T> {
    pub fn new(data: &'a Vec<T>) -> Self {
        Self {
            data: DataRef::Immutable(data),
            show_index: true,
        }
    }

    pub fn new_mut(data: &'a mut Vec<T>) -> Self {
        Self {
            data: DataRef::Mutable(data),
            show_index: true,
        }
    }

    pub fn show_index(mut self, show: bool) -> Self {
        self.show_index = show;
        self
    }
}

impl<'a, T: FRecursive + Clone> Composer<Vec<T>> for RecursiveListComposer<'a, T> {
    fn data(&self) -> &Vec<T> {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut Vec<T> {
        self.data.as_mut()
    }

    fn is_mutable(&self) -> bool {
        self.data.is_mutable()
    }

    fn compose(&self, context: &Context, ui: &mut Ui) -> Response {
        ui.vertical(|ui| {
            let mut response = ui.label("");
            for (i, item) in self.data.as_ref().iter().enumerate() {
                if self.show_index {
                    response = response.union(ui.label(format!("[{}]", i)));
                }
                let item_composer = RecursiveComposer::new(item);
                response = response.union(item_composer.compose(context, ui));
                ui.separator();
            }
            response
        })
        .inner
    }
}

/// Extension trait for FRecursive types
pub trait RecursiveExt: FRecursive + Sized {
    fn as_recursive(&self) -> RecursiveComposer<'_, Self> {
        RecursiveComposer::new(self)
    }

    fn as_recursive_mut(&mut self) -> RecursiveComposer<'_, Self> {
        RecursiveComposer::new_mut(self)
    }
}

impl<T: FRecursive> RecursiveExt for T {}

/// Extension trait for Vec<T> where T: FRecursive
pub trait RecursiveListExt<T: FRecursive> {
    fn as_recursive_list(&self) -> RecursiveListComposer<'_, T>;
    fn as_recursive_list_mut(&mut self) -> RecursiveListComposer<'_, T>;
}

impl<T: FRecursive + Clone> RecursiveListExt<T> for Vec<T> {
    fn as_recursive_list(&self) -> RecursiveListComposer<'_, T> {
        RecursiveListComposer::new(self)
    }

    fn as_recursive_list_mut(&mut self) -> RecursiveListComposer<'_, T> {
        RecursiveListComposer::new_mut(self)
    }
}

/// Edit composer for recursive data
pub struct RecursiveEditComposer<'a, T: FRecursive> {
    data: DataRef<'a, T>,
    layout: RecursiveLayout,
}

impl<'a, T: FRecursive> RecursiveEditComposer<'a, T> {
    pub fn new_mut(data: &'a mut T) -> Self {
        Self {
            data: DataRef::Mutable(data),
            layout: RecursiveLayout::Vertical,
        }
    }

    pub fn with_layout(mut self, layout: RecursiveLayout) -> Self {
        self.layout = layout;
        self
    }

    pub fn edit(&mut self, context: &Context, ui: &mut Ui) -> bool {
        match &mut self.data {
            DataRef::Mutable(data) => {
                let mut root = RecursiveFieldMut::named("root", data.to_recursive_value_mut());
                render_recursive_field_mut(&mut root, context, ui, &self.layout, true, 0)
            }
            DataRef::Immutable(_) => {
                panic!("Cannot edit immutable recursive data");
            }
        }
    }
}

fn render_recursive_field_mut(
    field: &mut RecursiveFieldMut<'_>,
    context: &Context,
    ui: &mut Ui,
    layout: &RecursiveLayout,
    show_names: bool,
    depth: usize,
) -> bool {
    let mut changed = false;

    match layout {
        RecursiveLayout::Vertical => {
            ui.vertical(|ui| {
                if show_names && !field.name.is_empty() && field.name != "root" {
                    ui.label(format!("{}:", field.name));
                }

                changed |= call_on_recursive_value_mut!(field, edit, context, ui);

                let inner_fields = call_on_recursive_value_mut!(field, get_inner_fields_mut);
                for mut inner_field in inner_fields {
                    changed |= render_recursive_field_mut(
                        &mut inner_field,
                        context,
                        ui,
                        layout,
                        show_names,
                        depth + 1,
                    );
                }
            });
        }
        _ => {
            // For now, only vertical layout is supported for editing
            changed = render_recursive_field_mut(
                field,
                context,
                ui,
                &RecursiveLayout::Vertical,
                show_names,
                depth,
            );
        }
    }

    changed
}
