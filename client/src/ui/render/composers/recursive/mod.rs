use super::*;

mod recursive_types;
pub use recursive_types::*;

mod list_composer;
pub use list_composer::*;

macro_rules! call_on_recursive_field {
    ($field:expr, $method:ident $(, $args:expr)*) => {
        match $field.value {
            RecursiveValue::Expr(v) => v.$method($($args),*),
            RecursiveValue::Action(v) => v.$method($($args),*),
            RecursiveValue::PainterAction(v) => v.$method($($args),*),
            RecursiveValue::Var(v) => v.$method($($args),*),
            RecursiveValue::VarValue(v) => v.$method($($args),*),
            RecursiveValue::HexColor(v) => v.$method($($args),*),
            RecursiveValue::String(v) => v.$method($($args),*),
            RecursiveValue::I32(v) => v.$method($($args),*),
            RecursiveValue::F32(v) => v.$method($($args),*),
            RecursiveValue::Bool(v) => v.$method($($args),*),
            RecursiveValue::Vec2(v) => v.$method($($args),*),
            RecursiveValue::Reaction(v) => v.$method($($args),*),
            RecursiveValue::Material(v) => v.$method($($args),*),
        }
    };
}

macro_rules! call_on_recursive_field_mut {
    ($field:expr, $method:ident $(, $args:expr)*) => {
        match &mut $field.value {
            RecursiveValueMut::Expr(v) => v.$method($($args),*),
            RecursiveValueMut::Action(v) => v.$method($($args),*),
            RecursiveValueMut::PainterAction(v) => v.$method($($args),*),
            RecursiveValueMut::Var(v) => v.$method($($args),*),
            RecursiveValueMut::VarValue(v) => v.$method($($args),*),
            RecursiveValueMut::HexColor(v) => v.$method($($args),*),
            RecursiveValueMut::String(v) => v.$method($($args),*),
            RecursiveValueMut::I32(v) => v.$method($($args),*),
            RecursiveValueMut::F32(v) => v.$method($($args),*),
            RecursiveValueMut::Bool(v) => v.$method($($args),*),
            RecursiveValueMut::Vec2(v) => v.$method($($args),*),
            RecursiveValueMut::Reaction(v) => v.$method($($args),*),
            RecursiveValueMut::Material(v) => v.$method($($args),*),
        }
    };
}

pub struct RecursiveComposer<'a, F> {
    data: RecursiveValue<'a>,
    composer_fn: F,
    show_field_names: bool,
    layout: RecursiveLayout,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecursiveLayout {
    HorizontalVertical,
    Vertical,
    Horizontal,
    Tree { indent: f32 },
}

impl<'a, F> RecursiveComposer<'a, F>
where
    F: FnMut(&Context, &mut Ui, RecursiveValue<'_>) -> Response,
{
    pub fn new(data: RecursiveValue<'a>, composer_fn: F) -> Self {
        Self {
            data,
            composer_fn,
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

    fn render_recursive_field_with_closure(
        &mut self,
        field: &RecursiveField<'_>,
        context: &Context,
        ui: &mut Ui,
        depth: usize,
    ) -> Response {
        let layout = self.layout;
        let show_field_names = self.show_field_names;
        match layout {
            RecursiveLayout::HorizontalVertical => {
                ui.horizontal(|ui| {
                    let mut response =
                        if show_field_names && !field.name.is_empty() && field.name != "root" {
                            ui.label(format!("{}:", field.name))
                        } else {
                            ui.label("")
                        };

                    response = response.union((self.composer_fn)(context, ui, field.value));

                    let inner_fields = call_on_recursive_field!(field, get_inner_fields);

                    if !inner_fields.is_empty() {
                        response = response.union(
                            ui.vertical(|ui| {
                                let mut resp = ui.label("");
                                for inner_field in inner_fields {
                                    resp = resp.union((self.composer_fn)(
                                        context,
                                        ui,
                                        inner_field.value,
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
                    let mut response =
                        if show_field_names && !field.name.is_empty() && field.name != "root" {
                            ui.label(format!("{}:", field.name))
                        } else {
                            ui.label("")
                        };

                    response = response.union((self.composer_fn)(context, ui, field.value));

                    let inner_fields = call_on_recursive_field!(field, get_inner_fields);

                    for inner_field in inner_fields {
                        response =
                            response.union((self.composer_fn)(context, ui, inner_field.value));
                    }
                    response
                })
                .inner
            }
            RecursiveLayout::Horizontal => {
                ui.horizontal(|ui| {
                    let mut response =
                        if show_field_names && !field.name.is_empty() && field.name != "root" {
                            ui.label(format!("{}:", field.name))
                        } else {
                            ui.label("")
                        };

                    response = response.union((self.composer_fn)(context, ui, field.value));

                    let inner_fields = call_on_recursive_field!(field, get_inner_fields);

                    for inner_field in inner_fields {
                        response =
                            response.union((self.composer_fn)(context, ui, inner_field.value));
                    }
                    response
                })
                .inner
            }
            RecursiveLayout::Tree { indent } => {
                let inner_fields = call_on_recursive_field!(field, get_inner_fields);
                let has_children = !inner_fields.is_empty();
                let show_field_names = self.show_field_names;

                ui.horizontal(|ui| {
                    ui.add_space(indent * depth as f32);

                    let mut response = ui.label("");

                    if has_children {
                        let id = ui.id().with(field.name.as_str()).with(depth);
                        let expanded = ui.ctx().data(|r| r.get_temp::<bool>(id)).unwrap_or(true);

                        if ui.button(if expanded { "▼" } else { "▶" }).clicked() {
                            ui.ctx().data_mut(|w| w.insert_temp(id, !expanded));
                        }

                        if show_field_names && !field.name.is_empty() && field.name != "root" {
                            response = response.union(ui.label(format!("{}:", field.name)));
                        }

                        response = response.union((self.composer_fn)(context, ui, field.value));

                        if expanded {
                            response = response.union(
                                ui.vertical(|ui| {
                                    let mut resp = ui.label("");
                                    for inner_field in inner_fields {
                                        resp = resp.union((self.composer_fn)(
                                            context,
                                            ui,
                                            inner_field.value,
                                        ));
                                    }
                                    resp
                                })
                                .inner,
                            );
                        }
                    } else {
                        if show_field_names && !field.name.is_empty() && field.name != "root" {
                            response = response.union(ui.label(format!("{}:", field.name)));
                        }
                        response = response.union((self.composer_fn)(context, ui, field.value));
                    }

                    response
                })
                .inner
            }
        }
    }
}

impl<'a, F> Composer<()> for RecursiveComposer<'a, F>
where
    F: FnMut(&Context, &mut Ui, RecursiveValue<'_>) -> Response,
{
    fn data(&self) -> &() {
        &()
    }

    fn data_mut(&mut self) -> &mut () {
        panic!("RecursiveComposer does not support mutable data access")
    }

    fn is_mutable(&self) -> bool {
        false
    }

    fn compose(mut self, context: &Context, ui: &mut Ui) -> Response {
        let field = RecursiveField::named("root", self.data);
        self.render_recursive_field_with_closure(&field, context, ui, 0)
    }
}

pub struct RecursiveComposerMut<'a, F> {
    data: RecursiveValueMut<'a>,
    composer_fn: F,
    show_field_names: bool,
    layout: RecursiveLayout,
}

impl<'a, F> RecursiveComposerMut<'a, F>
where
    F: FnMut(&Context, &mut Ui, &mut RecursiveValueMut<'_>) -> Response,
{
    pub fn new(data: RecursiveValueMut<'a>, composer_fn: F) -> Self {
        Self {
            data,
            composer_fn,
            show_field_names: true,
            layout: RecursiveLayout::Vertical,
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

    fn render_recursive_field_mut_with_closure(
        &mut self,
        field: &mut RecursiveFieldMut<'_>,
        context: &Context,
        ui: &mut Ui,
        depth: usize,
    ) -> Response {
        match &self.layout {
            RecursiveLayout::Vertical => {
                ui.vertical(|ui| {
                    let mut response = if self.show_field_names
                        && !field.name.is_empty()
                        && field.name != "root"
                    {
                        ui.label(format!("{}:", field.name))
                    } else {
                        ui.label("")
                    };

                    response = response.union((self.composer_fn)(context, ui, &mut field.value));

                    let inner_fields = call_on_recursive_field_mut!(field, get_inner_fields_mut);

                    for mut inner_field in inner_fields {
                        response = response.union(self.render_recursive_field_mut_with_closure(
                            &mut inner_field,
                            context,
                            ui,
                            depth + 1,
                        ));
                    }
                    response
                })
                .inner
            }
            _ => ui.label("Unsupported layout for mutable composer"),
        }
    }
}

impl<'a, F> Composer<()> for RecursiveComposerMut<'a, F>
where
    F: FnMut(&Context, &mut Ui, &mut RecursiveValueMut<'_>) -> Response,
{
    fn data(&self) -> &() {
        &()
    }

    fn data_mut(&mut self) -> &mut () {
        panic!("RecursiveComposerMut does not support mutable data access")
    }

    fn is_mutable(&self) -> bool {
        true
    }

    fn compose(mut self, context: &Context, ui: &mut Ui) -> Response {
        let RecursiveComposerMut {
            data,
            mut composer_fn,
            show_field_names,
            layout,
        } = self;

        let mut field = RecursiveFieldMut::named("root", data);

        // Handle the vertical layout case inline since we can't call the method
        match layout {
            RecursiveLayout::Vertical => {
                ui.vertical(|ui| {
                    let mut response =
                        if show_field_names && !field.name.is_empty() && field.name != "root" {
                            ui.label(format!("{}:", field.name))
                        } else {
                            ui.label("")
                        };

                    response = response.union(composer_fn(context, ui, &mut field.value));

                    let inner_fields = call_on_recursive_field_mut!(field, get_inner_fields_mut);

                    for mut inner_field in inner_fields {
                        response = response.union(composer_fn(context, ui, &mut inner_field.value));
                    }
                    response
                })
                .inner
            }
            _ => ui.label("Unsupported layout for mutable composer"),
        }
    }
}

pub fn recursive_composer_for<'a, T: FRecursive>(
    data: &'a T,
) -> RecursiveComposer<'a, impl FnMut(&Context, &mut Ui, RecursiveValue<'_>) -> Response> {
    RecursiveComposer::new(
        data.to_recursive_value(),
        |_context, ui, value| match value {
            RecursiveValue::Expr(v) => v.display(_context, ui),
            RecursiveValue::Action(v) => v.display(_context, ui),
            RecursiveValue::PainterAction(v) => v.display(_context, ui),
            RecursiveValue::Var(v) => v.display(_context, ui),
            RecursiveValue::VarValue(v) => v.display(_context, ui),
            RecursiveValue::HexColor(v) => v.display(_context, ui),
            RecursiveValue::String(v) => v.display(_context, ui),
            RecursiveValue::I32(v) => v.display(_context, ui),
            RecursiveValue::F32(v) => v.display(_context, ui),
            RecursiveValue::Bool(v) => v.display(_context, ui),
            RecursiveValue::Vec2(v) => v.display(_context, ui),
            RecursiveValue::Reaction(v) => v.display(_context, ui),
            RecursiveValue::Material(v) => v.display(_context, ui),
        },
    )
}

pub fn recursive_composer_mut_for<'a, T: FRecursive>(
    data: &'a mut T,
) -> RecursiveComposerMut<'a, impl FnMut(&Context, &mut Ui, &mut RecursiveValueMut<'_>) -> Response>
{
    RecursiveComposerMut::new(
        data.to_recursive_value_mut(),
        |_context, ui, value| match value {
            RecursiveValueMut::Expr(v) => {
                v.edit(_context, ui);
                ui.label("Expression")
            }
            RecursiveValueMut::Action(v) => {
                v.edit(_context, ui);
                ui.label("Action")
            }
            RecursiveValueMut::PainterAction(v) => {
                v.edit(_context, ui);
                ui.label("PainterAction")
            }
            RecursiveValueMut::Var(v) => {
                v.edit(_context, ui);
                ui.label("Var")
            }
            RecursiveValueMut::VarValue(v) => {
                v.edit(_context, ui);
                ui.label("VarValue")
            }
            RecursiveValueMut::HexColor(v) => {
                v.edit(_context, ui);
                ui.label("HexColor")
            }
            RecursiveValueMut::String(v) => {
                v.edit(_context, ui);
                ui.label("String")
            }
            RecursiveValueMut::I32(v) => {
                v.edit(_context, ui);
                ui.label("I32")
            }
            RecursiveValueMut::F32(v) => {
                v.edit(_context, ui);
                ui.label("F32")
            }
            RecursiveValueMut::Bool(v) => {
                v.edit(_context, ui);
                ui.label("Bool")
            }
            RecursiveValueMut::Vec2(v) => {
                v.edit(_context, ui);
                ui.label("Vec2")
            }
            RecursiveValueMut::Reaction(v) => {
                v.edit(_context, ui);
                ui.label("Reaction")
            }
            RecursiveValueMut::Material(v) => {
                v.edit(_context, ui);
                ui.label("Material")
            }
        },
    )
}
