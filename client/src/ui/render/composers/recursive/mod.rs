use super::*;

mod recursive_types;
pub use recursive_types::*;

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum RecursiveLayout {
    HorizontalVertical,
    Vertical,
    Horizontal,
    Tree { indent: f32 },
}

pub struct RecursiveComposer<'a, T, F>
where
    T: FRecursive,
{
    data: DataRef<'a, T>,
    composer_fn: F,
    show_field_names: bool,
    layout: RecursiveLayout,
}

impl<'a, T, F> RecursiveComposer<'a, T, F>
where
    T: FRecursive,
    F: FnMut(&ClientContext, &mut Ui, RecursiveValue<'_>) -> Response,
{
    pub fn new(data: &'a T, composer_fn: F) -> Self {
        Self {
            data: DataRef::Immutable(data),
            composer_fn,
            show_field_names: true,
            layout: RecursiveLayout::HorizontalVertical,
        }
    }

    pub fn new_mut(data: &'a mut T, composer_fn: F) -> Self {
        Self {
            data: DataRef::Mutable(data),
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
}

impl<'a, T, F> Composer<T> for RecursiveComposer<'a, T, F>
where
    T: FRecursive,
    F: FnMut(&ClientContext, &mut Ui, RecursiveValue<'_>) -> Response,
{
    fn data(&self) -> &T {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut T {
        self.data.as_mut()
    }

    fn is_mutable(&self) -> bool {
        self.data.is_mutable()
    }

    fn compose(self, context: &ClientContext, ui: &mut Ui) -> Response {
        let RecursiveComposer {
            data,
            mut composer_fn,
            show_field_names,
            layout,
        } = self;

        let recursive_value = data.as_ref().to_recursive_value();
        let field = RecursiveField::named("root", recursive_value);

        fn render_field<F>(
            field: &RecursiveField<'_>,
            context: &ClientContext,
            ui: &mut Ui,
            composer_fn: &mut F,
            layout: RecursiveLayout,
            show_field_names: bool,
            depth: usize,
        ) -> Response
        where
            F: FnMut(&ClientContext, &mut Ui, RecursiveValue<'_>) -> Response,
        {
            let inner_fields = call_on_recursive_value!(field.value, get_inner_fields);

            match layout {
                RecursiveLayout::HorizontalVertical => {
                    ui.horizontal(|ui| {
                        let mut response = render_self(
                            &field.name,
                            show_field_names,
                            context,
                            ui,
                            |context, ui| composer_fn(context, ui, field.value),
                        );

                        ui.vertical(|ui| {
                            for inner_field in inner_fields {
                                response |= render_field(
                                    &inner_field,
                                    context,
                                    ui,
                                    composer_fn,
                                    layout,
                                    show_field_names,
                                    depth + 1,
                                );
                            }
                        });
                        response
                    })
                    .inner
                }
                RecursiveLayout::Vertical => {
                    ui.vertical(|ui| {
                        let mut response = render_self(
                            &field.name,
                            show_field_names,
                            context,
                            ui,
                            |context, ui| composer_fn(context, ui, field.value),
                        );

                        for inner_field in inner_fields {
                            response |= render_field(
                                &inner_field,
                                context,
                                ui,
                                composer_fn,
                                layout,
                                show_field_names,
                                depth + 1,
                            );
                        }
                        response
                    })
                    .inner
                }
                RecursiveLayout::Horizontal => {
                    ui.horizontal(|ui| {
                        let mut response = render_self(
                            &field.name,
                            show_field_names,
                            context,
                            ui,
                            |context, ui| composer_fn(context, ui, field.value),
                        );

                        for inner_field in inner_fields {
                            response |= render_field(
                                &inner_field,
                                context,
                                ui,
                                composer_fn,
                                layout,
                                show_field_names,
                                depth + 1,
                            );
                        }
                        response
                    })
                    .inner
                }
                RecursiveLayout::Tree { indent } => {
                    let has_children = !inner_fields.is_empty();

                    ui.horizontal(|ui| {
                        ui.add_space(indent * depth as f32);

                        if has_children {
                            let id = ui.id().with(field.name.as_str()).with(depth);
                            let expanded =
                                ui.ctx().data(|r| r.get_temp::<bool>(id)).unwrap_or(true);

                            let mut response = render_self(
                                &field.name,
                                show_field_names,
                                context,
                                ui,
                                |context, ui| {
                                    ui.horizontal(|ui| {
                                        if ui.button(if expanded { "▼" } else { "▶" }).clicked()
                                        {
                                            ui.ctx().data_mut(|w| w.insert_temp(id, !expanded));
                                        }
                                        composer_fn(context, ui, field.value)
                                    })
                                    .inner
                                },
                            );

                            if expanded {
                                ui.vertical(|ui| {
                                    for inner_field in inner_fields {
                                        response |= render_field(
                                            &inner_field,
                                            context,
                                            ui,
                                            composer_fn,
                                            layout,
                                            show_field_names,
                                            depth + 1,
                                        );
                                    }
                                });
                            }
                            response
                        } else {
                            render_self(
                                &field.name,
                                show_field_names,
                                context,
                                ui,
                                |context, ui| composer_fn(context, ui, field.value),
                            )
                        }
                    })
                    .inner
                }
            }
        }

        render_field(
            &field,
            context,
            ui,
            &mut composer_fn,
            layout,
            show_field_names,
            0,
        )
    }
}

pub struct RecursiveComposerMut<'a, T, F>
where
    T: FRecursive,
{
    data: DataRef<'a, T>,
    composer_fn: F,
    show_field_names: bool,
    layout: RecursiveLayout,
}

impl<'a, T, F> RecursiveComposerMut<'a, T, F>
where
    T: FRecursive,
    F: FnMut(&ClientContext, &mut Ui, &mut RecursiveValueMut<'_>) -> Response,
{
    pub fn new_mut(data: &'a mut T, composer_fn: F) -> Self {
        Self {
            data: DataRef::Mutable(data),
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
}

impl<'a, T, F> Composer<T> for RecursiveComposerMut<'a, T, F>
where
    T: FRecursive,
    F: FnMut(&ClientContext, &mut Ui, &mut RecursiveValueMut<'_>) -> Response,
{
    fn data(&self) -> &T {
        self.data.as_ref()
    }

    fn data_mut(&mut self) -> &mut T {
        self.data.as_mut()
    }

    fn is_mutable(&self) -> bool {
        self.data.is_mutable()
    }

    fn compose(self, context: &ClientContext, ui: &mut Ui) -> Response {
        let RecursiveComposerMut {
            mut data,
            mut composer_fn,
            show_field_names,
            layout,
        } = self;

        let recursive_value_mut = data.as_mut().to_recursive_value_mut();
        let mut field = RecursiveFieldMut::named("root", recursive_value_mut);

        fn render_field_mut<F>(
            field: &mut RecursiveFieldMut<'_>,
            context: &ClientContext,
            ui: &mut Ui,
            composer_fn: &mut F,
            layout: RecursiveLayout,
            show_field_names: bool,
            depth: usize,
        ) -> Response
        where
            F: FnMut(&ClientContext, &mut Ui, &mut RecursiveValueMut<'_>) -> Response,
        {
            let field_name = field.name.clone();

            match layout {
                RecursiveLayout::HorizontalVertical => {
                    ui.horizontal(|ui| {
                        let mut response = render_self(
                            &field_name,
                            show_field_names,
                            context,
                            ui,
                            |context, ui| composer_fn(context, ui, &mut field.value),
                        );

                        let inner_fields =
                            call_on_recursive_value_mut!(&mut field.value, get_inner_fields_mut);
                        ui.vertical(|ui| {
                            for mut inner_field in inner_fields {
                                response |= render_field_mut(
                                    &mut inner_field,
                                    context,
                                    ui,
                                    composer_fn,
                                    layout,
                                    show_field_names,
                                    depth + 1,
                                );
                            }
                        });
                        response
                    })
                    .inner
                }
                RecursiveLayout::Vertical => {
                    ui.vertical(|ui| {
                        let mut response = render_self(
                            &field_name,
                            show_field_names,
                            context,
                            ui,
                            |context, ui| composer_fn(context, ui, &mut field.value),
                        );

                        let inner_fields =
                            call_on_recursive_value_mut!(&mut field.value, get_inner_fields_mut);
                        for mut inner_field in inner_fields {
                            response |= render_field_mut(
                                &mut inner_field,
                                context,
                                ui,
                                composer_fn,
                                layout,
                                show_field_names,
                                depth + 1,
                            );
                        }
                        response
                    })
                    .inner
                }
                RecursiveLayout::Horizontal => {
                    ui.horizontal(|ui| {
                        let mut response = render_self(
                            &field_name,
                            show_field_names,
                            context,
                            ui,
                            |context, ui| composer_fn(context, ui, &mut field.value),
                        );

                        let inner_fields =
                            call_on_recursive_value_mut!(&mut field.value, get_inner_fields_mut);
                        for mut inner_field in inner_fields {
                            response |= render_field_mut(
                                &mut inner_field,
                                context,
                                ui,
                                composer_fn,
                                layout,
                                show_field_names,
                                depth + 1,
                            );
                        }
                        response
                    })
                    .inner
                }
                RecursiveLayout::Tree { indent } => {
                    ui.horizontal(|ui| {
                        ui.add_space(indent * depth as f32);

                        let id = ui.id().with(&field_name).with(depth);
                        let expanded = ui.ctx().data(|r| r.get_temp::<bool>(id)).unwrap_or(true);

                        let mut response = render_self(
                            &field_name,
                            show_field_names,
                            context,
                            ui,
                            |context, ui| {
                                ui.horizontal(|ui| {
                                    if ui.button(if expanded { "🔽" } else { "▶️" }).clicked()
                                    {
                                        ui.ctx().data_mut(|w| w.insert_temp(id, !expanded));
                                    }
                                    composer_fn(context, ui, &mut field.value)
                                })
                                .inner
                            },
                        );

                        if expanded {
                            let inner_fields = call_on_recursive_value_mut!(
                                &mut field.value,
                                get_inner_fields_mut
                            );

                            ui.vertical(|ui| {
                                for mut inner_field in inner_fields {
                                    response |= render_field_mut(
                                        &mut inner_field,
                                        context,
                                        ui,
                                        composer_fn,
                                        layout,
                                        show_field_names,
                                        depth + 1,
                                    );
                                }
                            });
                        }

                        response
                    })
                    .inner
                }
            }
        }

        render_field_mut(
            &mut field,
            context,
            ui,
            &mut composer_fn,
            layout,
            show_field_names,
            0,
        )
    }
}

fn render_self(
    field_name: &String,
    show_field_names: bool,
    context: &ClientContext,
    ui: &mut Ui,
    content: impl FnOnce(&ClientContext, &mut Ui) -> Response,
) -> Response {
    ui.group(|ui| {
        ui.vertical(|ui| {
            if show_field_names && !field_name.is_empty() && *field_name != "root" {
                format!("[s [tw {}:]]", field_name).label(ui);
            }
            content(context, ui)
        })
        .inner
    })
    .inner
}
