use super::*;
use crate::ui::see::CstrTrait;

/// TreeComposer renders recursive tree structures with horizontal-first then vertical layout
pub struct TreeComposer<T, C> {
    item_composer: C,
    indent: f32,
    show_expand_button: bool,
    expanded_by_default: bool,
    _phantom: PhantomData<T>,
}

impl<T, C> TreeComposer<T, C> {
    pub fn new(item_composer: C) -> Self {
        Self {
            item_composer,
            indent: 16.0,
            show_expand_button: true,
            expanded_by_default: true,
            _phantom: PhantomData,
        }
    }

    pub fn with_indent(mut self, indent: f32) -> Self {
        self.indent = indent;
        self
    }

    pub fn show_expand_button(mut self, show: bool) -> Self {
        self.show_expand_button = show;
        self
    }

    pub fn expanded_by_default(mut self, expanded: bool) -> Self {
        self.expanded_by_default = expanded;
        self
    }
}

/// Composer implementation for types that can be rendered in a tree
impl<T: FRecursive + Clone, C: Composer<T>> Composer<T> for TreeComposer<T, C> {
    fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
        self.render_tree_node(data, context, ui, 0)
    }
}

impl<T: FRecursive + Clone, C: Composer<T>> TreeComposer<T, C> {
    fn render_tree_node(&self, data: &T, context: &Context, ui: &mut Ui, depth: usize) -> Response {
        let mut response = ui.label("");

        // Horizontal layout for self, then vertical for inner fields
        ui.horizontal(|ui| {
            // Indentation
            ui.add_space(self.indent * depth as f32);

            // Get inner fields to check if expandable
            let inner_fields = data.get_inner_fields();
            let has_children = !inner_fields.is_empty();

            if has_children && self.show_expand_button {
                let id = ui.id().with(depth).with(data as *const _ as usize);
                let expanded = ui
                    .ctx()
                    .data(|r| r.get_temp::<bool>(id))
                    .unwrap_or(self.expanded_by_default);

                if ui.button(if expanded { "▼" } else { "▶" }).clicked() {
                    ui.ctx().data_mut(|w| w.insert_temp(id, !expanded));
                }

                // Render self
                response = self.item_composer.compose(data, context, ui);

                // Render children vertically if expanded
                if expanded {
                    ui.vertical(|ui| {
                        for field in inner_fields {
                            // For nested fields, we just display them rather than recursing
                            // since they might not be of type T
                            call_on_recursive_value!(field, display, context, ui);
                        }
                    });
                }
            } else {
                // No expand button needed
                if has_children {
                    ui.add_space(22.0); // Space for missing expand button
                }

                // Render self
                response = self.item_composer.compose(data, context, ui);

                // Render children vertically
                if has_children {
                    ui.vertical(|ui| {
                        for field in inner_fields {
                            // For nested fields, we just display them rather than recursing
                            // since they might not be of type T
                            call_on_recursive_value!(field, display, context, ui);
                        }
                    });
                }
            }
        });

        response
    }
}

/// Mutable composer implementation for editable trees
impl<T: FRecursive + Clone, C: ComposerMut<T>> ComposerMut<T> for TreeComposer<T, C> {
    fn compose_mut(&self, data: &mut T, context: &Context, ui: &mut Ui) -> bool {
        self.render_tree_node_mut(data, context, ui, 0)
    }
}

impl<T: FRecursive + Clone, C: ComposerMut<T>> TreeComposer<T, C> {
    fn render_tree_node_mut(
        &self,
        data: &mut T,
        context: &Context,
        ui: &mut Ui,
        depth: usize,
    ) -> bool {
        let mut changed = false;

        // Horizontal layout for self, then vertical for inner fields
        ui.horizontal(|ui| {
            // Indentation
            ui.add_space(self.indent * depth as f32);

            // Get inner fields to check if expandable
            let inner_fields_check = data.get_inner_fields_mut();
            let has_children = !inner_fields_check.is_empty();

            if has_children && self.show_expand_button {
                let id = ui.id().with(depth).with(data as *const _ as usize);
                let expanded = ui
                    .ctx()
                    .data(|r| r.get_temp::<bool>(id))
                    .unwrap_or(self.expanded_by_default);

                if ui.button(if expanded { "▼" } else { "▶" }).clicked() {
                    ui.ctx().data_mut(|w| w.insert_temp(id, !expanded));
                }

                // Render self
                changed |= self.item_composer.compose_mut(data, context, ui);

                // Render children vertically if expanded
                if expanded {
                    ui.vertical(|ui| {
                        let inner_fields = data.get_inner_fields_mut();
                        for mut field in inner_fields {
                            // For nested fields, we edit them directly rather than recursing
                            // since they might not be of type T
                            changed |= call_on_recursive_value_mut!(field, edit, context, ui);
                        }
                    });
                }
            } else {
                // No expand button needed
                if has_children {
                    ui.add_space(22.0); // Space for missing expand button
                }

                // Render self
                changed |= self.item_composer.compose_mut(data, context, ui);

                // Render children vertically
                if has_children {
                    ui.vertical(|ui| {
                        let inner_fields = data.get_inner_fields_mut();
                        for mut field in inner_fields {
                            // For nested fields, we edit them directly rather than recursing
                            // since they might not be of type T
                            changed |= call_on_recursive_value_mut!(field, edit, context, ui);
                        }
                    });
                }
            }
        });

        changed
    }
}

/// Specialized tree composer for Expression editing
pub struct ExpressionTreeComposer;

impl ExpressionTreeComposer {
    pub fn new()
    -> RecursiveComposer<impl FnMut(&mut Ui, &Context, &mut RecursiveFieldMut<'_>) -> bool> {
        RecursiveComposer::new(
            |ui: &mut Ui, context: &Context, field: &mut RecursiveFieldMut<'_>| -> bool {
                match &mut field.value {
                    RecursiveValueMut::Expr(e) => {
                        if let Some(n) = (**e)
                            .see_mut(context)
                            .ctxbtn()
                            .add_paste()
                            .add_copy()
                            .ui_enum(ui)
                            .selector_changed()
                        {
                            RecursiveValueMut::replace_expr_and_move_fields(e, n.clone());
                            true
                        } else {
                            false
                        }
                    }
                    _ => call_on_recursive_value_mut!(field, edit, context, ui),
                }
            },
        )
        .with_layout(RecursiveLayout::HorizontalVertical)
    }
}

/// Specialized tree composer for PainterAction editing
pub struct PainterActionTreeComposer;

impl PainterActionTreeComposer {
    pub fn new()
    -> RecursiveComposer<impl FnMut(&mut Ui, &Context, &mut RecursiveFieldMut<'_>) -> bool> {
        RecursiveComposer::new(
            |ui: &mut Ui, context: &Context, field: &mut RecursiveFieldMut<'_>| -> bool {
                match &mut field.value {
                    RecursiveValueMut::PainterAction(pa) => {
                        if let Some(n) = (**pa)
                            .see_mut(context)
                            .ctxbtn()
                            .add_paste()
                            .add_copy()
                            .ui_enum(ui)
                            .selector_changed()
                        {
                            RecursiveValueMut::replace_painter_action_and_move_fields(
                                pa,
                                n.clone(),
                            );
                            true
                        } else {
                            false
                        }
                    }
                    _ => call_on_recursive_value_mut!(field, edit, context, ui),
                }
            },
        )
        .with_layout(RecursiveLayout::HorizontalVertical)
    }
}

/// Specialized tree composer for Action editing
pub struct ActionTreeComposer;

impl ActionTreeComposer {
    pub fn new()
    -> RecursiveComposer<impl FnMut(&mut Ui, &Context, &mut RecursiveFieldMut<'_>) -> bool> {
        RecursiveComposer::new(
            |ui: &mut Ui, context: &Context, field: &mut RecursiveFieldMut<'_>| -> bool {
                match &mut field.value {
                    RecursiveValueMut::Action(a) => {
                        if let Some(n) = (**a)
                            .see_mut(context)
                            .ctxbtn()
                            .add_paste()
                            .add_copy()
                            .ui_enum(ui)
                            .selector_changed()
                        {
                            RecursiveValueMut::replace_action_and_move_fields(a, n.clone());
                            true
                        } else {
                            false
                        }
                    }
                    _ => call_on_recursive_value_mut!(field, edit, context, ui),
                }
            },
        )
        .with_layout(RecursiveLayout::HorizontalVertical)
    }
}

/// Helper function to create a tree composer with custom item renderer
pub fn tree_with_renderer<T, F>(renderer: F, indent: f32) -> TreeComposer<T, impl Composer<T>>
where
    T: FRecursive + Clone,
    F: FnMut(&T, &Context, &mut Ui) -> Response + 'static,
{
    struct RendererComposer<T, F> {
        renderer: std::cell::RefCell<F>,
        _phantom: PhantomData<T>,
    }

    impl<T, F> Composer<T> for RendererComposer<T, F>
    where
        F: FnMut(&T, &Context, &mut Ui) -> Response,
    {
        fn compose(&self, data: &T, context: &Context, ui: &mut Ui) -> Response {
            (self.renderer.borrow_mut())(data, context, ui)
        }
    }

    TreeComposer::new(RendererComposer {
        renderer: std::cell::RefCell::new(renderer),
        _phantom: PhantomData,
    })
    .with_indent(indent)
}

/// Helper function to create a mutable tree composer with custom item renderer
pub fn tree_mut_with_renderer<T, F>(
    renderer: F,
    indent: f32,
) -> TreeComposer<T, impl ComposerMut<T>>
where
    T: FRecursive + Clone,
    F: FnMut(&mut T, &Context, &mut Ui) -> bool + 'static,
{
    struct RendererComposerMut<T, F> {
        renderer: std::cell::RefCell<F>,
        _phantom: PhantomData<T>,
    }

    impl<T, F> ComposerMut<T> for RendererComposerMut<T, F>
    where
        F: FnMut(&mut T, &Context, &mut Ui) -> bool,
    {
        fn compose_mut(&self, data: &mut T, context: &Context, ui: &mut Ui) -> bool {
            (self.renderer.borrow_mut())(data, context, ui)
        }
    }

    TreeComposer::new(RendererComposerMut {
        renderer: std::cell::RefCell::new(renderer),
        _phantom: PhantomData,
    })
    .with_indent(indent)
}
