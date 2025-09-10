use super::super::*;
use crate::ui::see::{RecursiveFieldMut, RecursiveValueMut, SFnShowMut, ToRecursiveValueMut};
use crate::{Action, Expression, Material, PainterAction, Reaction};

/// Generic list composer that wraps any other composer to provide list manipulation functionality
pub struct EditableListComposer<C> {
    item_composer: C,
    allow_reorder: bool,
    allow_add_remove: bool,
}

impl<C> EditableListComposer<C> {
    pub fn new(item_composer: C) -> Self {
        Self {
            item_composer,
            allow_reorder: true,
            allow_add_remove: true,
        }
    }

    pub fn with_reorder(mut self, allow: bool) -> Self {
        self.allow_reorder = allow;
        self
    }

    pub fn with_add_remove(mut self, allow: bool) -> Self {
        self.allow_add_remove = allow;
        self
    }
}

impl<T, C> ComposerMut<Vec<T>> for EditableListComposer<C>
where
    T: Clone + Default,
    C: ComposerMut<T>,
{
    fn compose_mut(&self, data: &mut Vec<T>, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        let mut to_remove = None;
        let mut to_move = None;
        let len = data.len();

        for (i, item) in data.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                // Reorder buttons
                if self.allow_reorder {
                    ui.vertical(|ui| {
                        ui.add_enabled_ui(i > 0, |ui| {
                            if ui.small_button("↑").clicked() && i > 0 {
                                to_move = Some((i, i - 1));
                            }
                        });
                        ui.add_enabled_ui(i < len - 1, |ui| {
                            if ui.small_button("↓").clicked() && i < len - 1 {
                                to_move = Some((i, i + 1));
                            }
                        });
                    });
                }

                // Item content using the provided composer
                ui.vertical(|ui| {
                    ui.label(format!("Item #{}", i + 1));
                    changed |= self.item_composer.compose_mut(item, context, ui);
                });

                // Delete button
                if self.allow_add_remove {
                    if ui.small_button("🗑").clicked() {
                        to_remove = Some(i);
                    }
                }
            });

            ui.separator();
        }

        // Handle moves
        if let Some((from, to)) = to_move {
            data.swap(from, to);
            changed = true;
        }

        // Handle removals
        if let Some(idx) = to_remove {
            data.remove(idx);
            changed = true;
        }

        // Add button
        if self.allow_add_remove {
            if ui.button("➕ Add Item").clicked() {
                data.push(T::default());
                changed = true;
            }
        }

        changed
    }
}

/// Immutable list composer for read-only display
impl<T, C> Composer<Vec<T>> for EditableListComposer<C>
where
    C: Composer<T>,
{
    fn compose(&self, data: &Vec<T>, context: &Context, ui: &mut Ui) -> Response {
        let mut response = ui.label("");

        for (i, item) in data.iter().enumerate() {
            ui.group(|ui| {
                ui.label(format!("Item #{}", i + 1));
                response = response.union(self.item_composer.compose(item, context, ui));
            });
        }

        response
    }
}

/// Specialized recursive composer for types that implement ToRecursiveValueMut
pub struct RecursiveComposer;

impl RecursiveComposer {
    pub fn compose_recursive_value(
        field: &mut RecursiveFieldMut,
        context: &Context,
        ui: &mut Ui,
    ) -> bool {
        let mut changed = false;

        match &mut field.value {
            RecursiveValueMut::Expr(e) => {
                let response = (**e)
                    .see_mut(context)
                    .ctxbtn()
                    .add_copy()
                    .add_paste()
                    .ui_enum(ui);

                if let Some(new_expr) = response.selector_changed() {
                    RecursiveValueMut::replace_expr_and_move_fields(e, new_expr.clone());
                    changed = true;
                }

                if let Some(replacement) = response.pasted() {
                    **e = replacement.clone();
                    changed = true;
                }

                ui.indent("expr_fields", |ui| {
                    changed |= (**e).show_mut(context, ui);
                });
            }
            RecursiveValueMut::Action(a) => {
                let response = (**a)
                    .see_mut(context)
                    .ctxbtn()
                    .add_copy()
                    .add_paste()
                    .ui_enum(ui);

                if let Some(new_action) = response.selector_changed() {
                    RecursiveValueMut::replace_action_and_move_fields(a, new_action.clone());
                    changed = true;
                }

                if let Some(replacement) = response.pasted() {
                    **a = replacement.clone();
                    changed = true;
                }

                ui.indent("action_fields", |ui| {
                    changed |= (**a).show_mut(context, ui);
                });
            }
            RecursiveValueMut::PainterAction(pa) => {
                let response = (**pa)
                    .see_mut(context)
                    .ctxbtn()
                    .add_copy()
                    .add_paste()
                    .ui_enum(ui);

                if let Some(new_action) = response.selector_changed() {
                    RecursiveValueMut::replace_painter_action_and_move_fields(
                        pa,
                        new_action.clone(),
                    );
                    changed = true;
                }

                if let Some(replacement) = response.pasted() {
                    **pa = replacement.clone();
                    changed = true;
                }

                ui.indent("painter_fields", |ui| {
                    changed |= (**pa).show_mut(context, ui);
                });
            }
            RecursiveValueMut::Reaction(r) => {
                let composer = ReactionComposer;
                changed |= composer.compose_mut(*r, context, ui);
            }
            RecursiveValueMut::Material(m) => {
                let composer = MaterialComposer;
                changed |= composer.compose_mut(*m, context, ui);
            }
            RecursiveValueMut::Var(v) => {
                changed |= (**v).show_mut(context, ui);
            }
            RecursiveValueMut::VarValue(v) => {
                changed |= (**v).show_mut(context, ui);
            }
            RecursiveValueMut::HexColor(c) => {
                changed |= (**c).show_mut(context, ui);
            }
            RecursiveValueMut::String(s) => {
                changed |= (**s).show_mut(context, ui);
            }
            RecursiveValueMut::I32(i) => {
                changed |= (**i).show_mut(context, ui);
            }
            RecursiveValueMut::F32(f) => {
                changed |= (**f).show_mut(context, ui);
            }
            RecursiveValueMut::Bool(b) => {
                changed |= (**b).show_mut(context, ui);
            }
            RecursiveValueMut::Vec2(v) => {
                changed |= (**v).show_mut(context, ui);
            }
        }

        changed
    }
}

/// Specialized composer for PainterAction with selector and context buttons
pub struct PainterActionComposer;

impl ComposerMut<PainterAction> for PainterActionComposer {
    fn compose_mut(&self, data: &mut PainterAction, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        // Show selector with context buttons
        let response = data
            .see_mut(context)
            .ctxbtn()
            .add_copy()
            .add_paste()
            .ui_enum(ui);

        // Handle selector changes
        if let Some(new_action) = response.selector_changed() {
            RecursiveValueMut::replace_painter_action_and_move_fields(data, new_action.clone());
            changed = true;
        }

        // Handle paste
        if let Some(replacement) = response.pasted() {
            *data = replacement.clone();
            changed = true;
        }

        // Show fields for the current variant
        ui.indent("painter_fields", |ui| {
            changed |= data.show_mut(context, ui);
        });

        changed
    }
}

/// Specialized composer for Action with selector and context buttons
pub struct ActionComposer;

impl ComposerMut<Action> for ActionComposer {
    fn compose_mut(&self, data: &mut Action, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        // Show selector with context buttons
        let response = data
            .see_mut(context)
            .ctxbtn()
            .add_copy()
            .add_paste()
            .ui_enum(ui);

        // Handle selector changes
        if let Some(new_action) = response.selector_changed() {
            RecursiveValueMut::replace_action_and_move_fields(data, new_action.clone());
            changed = true;
        }

        // Handle paste
        if let Some(replacement) = response.pasted() {
            *data = replacement.clone();
            changed = true;
        }

        // Show fields for the current variant
        ui.indent("action_fields", |ui| {
            changed |= data.show_mut(context, ui);
        });

        changed
    }
}

/// Specialized composer for Expression with selector and context buttons
pub struct ExpressionComposer;

impl ComposerMut<Expression> for ExpressionComposer {
    fn compose_mut(&self, data: &mut Expression, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        // Show selector with context buttons
        let response = data
            .see_mut(context)
            .ctxbtn()
            .add_copy()
            .add_paste()
            .ui_enum(ui);

        // Handle selector changes
        if let Some(new_expr) = response.selector_changed() {
            RecursiveValueMut::replace_expr_and_move_fields(data, new_expr.clone());
            changed = true;
        }

        // Handle paste
        if let Some(replacement) = response.pasted() {
            *data = replacement.clone();
            changed = true;
        }

        // Show fields for the current variant
        ui.indent("expr_fields", |ui| {
            changed |= data.show_mut(context, ui);
        });

        changed
    }
}

/// Specialized composer for Reaction with trigger and actions
pub struct ReactionComposer;

impl ComposerMut<Reaction> for ReactionComposer {
    fn compose_mut(&self, data: &mut Reaction, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            ui.label("Trigger:");
            if data.trigger.show_mut(context, ui) {
                changed = true;
            }
        });

        ui.separator();
        ui.label("Actions:");

        // Use EditableListComposer with ActionComposer for the actions list
        let list_composer = EditableListComposer::new(ActionComposer);
        changed |= list_composer.compose_mut(&mut data.actions, context, ui);

        changed
    }
}

/// Specialized composer for Material with preview and painter actions
pub struct MaterialComposer;

impl ComposerMut<Material> for MaterialComposer {
    fn compose_mut(&self, data: &mut Material, context: &Context, ui: &mut Ui) -> bool {
        use crate::plugins::RepresentationPlugin;
        use egui::{Sense, Stroke};

        let mut changed = false;

        ui.horizontal(|ui| {
            ui.vertical(|ui| {
                ui.label("Preview:");
                let size = 100.0;
                let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
                RepresentationPlugin::paint_rect(rect, context, data, ui).ui(ui);
                ui.painter().rect_stroke(
                    rect,
                    0,
                    Stroke::new(1.0, subtle_borders_and_separators()),
                    egui::StrokeKind::Middle,
                );
            });

            ui.separator();

            ui.vertical(|ui| {
                ui.label("Painter Actions:");
                let list_composer = EditableListComposer::new(PainterActionComposer);
                changed |= list_composer.compose_mut(&mut data.0, context, ui);
            });
        });

        changed
    }
}

/// Composer for individual recursive items
pub struct RecursiveItemComposer;

impl<T> ComposerMut<T> for RecursiveItemComposer
where
    T: ToRecursiveValueMut,
{
    fn compose_mut(&self, data: &mut T, context: &Context, ui: &mut Ui) -> bool {
        let mut field = RecursiveFieldMut::named("item", data.to_recursive_value_mut());
        RecursiveComposer::compose_recursive_value(&mut field, context, ui)
    }
}

/// Extension methods for RenderBuilder to use list composers
impl<'a, T> RenderBuilder<'a, Vec<T>>
where
    T: Clone + Default,
{
    /// Edit a list with a custom composer for items
    pub fn edit_list_with<C>(self, composer: C, ui: &mut Ui) -> bool
    where
        C: ComposerMut<T>,
    {
        match self.data {
            RenderDataRef::Mutable(data) => {
                let list_composer = EditableListComposer::new(composer);
                list_composer.compose_mut(data, self.ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }

    /// Display a list with a custom composer for items (read-only)
    pub fn show_list_with<C>(self, composer: C, ui: &mut Ui) -> Response
    where
        C: Composer<T>,
    {
        match self.data {
            RenderDataRef::Mutable(data) => {
                let list_composer = EditableListComposer::new(composer);
                list_composer.compose(data, self.ctx, ui)
            }
            RenderDataRef::Immutable(data) => {
                let list_composer = EditableListComposer::new(composer);
                list_composer.compose(data, self.ctx, ui)
            }
        }
    }
}

/// Specialized extensions for recursive types
impl<'a, T> RenderBuilder<'a, Vec<T>>
where
    T: ToRecursiveValueMut + Clone + Default,
{
    pub fn edit_recursive_list(self, ui: &mut Ui) -> bool {
        self.edit_list_with(RecursiveItemComposer, ui)
    }
}

/// Specialized extensions for commonly used types
impl<'a> RenderBuilder<'a, Vec<Action>> {
    pub fn edit_action_list(self, ui: &mut Ui) -> bool {
        self.edit_list_with(ActionComposer, ui)
    }
}

impl<'a> RenderBuilder<'a, Vec<PainterAction>> {
    pub fn edit_painter_action_list(self, ui: &mut Ui) -> bool {
        self.edit_list_with(PainterActionComposer, ui)
    }
}

impl<'a> RenderBuilder<'a, Vec<Expression>> {
    pub fn edit_expression_list(self, ui: &mut Ui) -> bool {
        self.edit_list_with(ExpressionComposer, ui)
    }
}

impl<'a> RenderBuilder<'a, Vec<Reaction>> {
    pub fn edit_reaction_list(self, ui: &mut Ui) -> bool {
        self.edit_list_with(ReactionComposer, ui)
    }
}

/// Extensions for individual recursive types
impl<'a> RenderBuilder<'a, Material> {
    pub fn edit_material(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => {
                fn field_renderer(
                    ui: &mut Ui,
                    context: &Context<'_>,
                    field: RecursiveFieldMut<'_>,
                ) {
                    match field.value {
                        RecursiveValueMut::Expr(expression) => expression.edit(context, ui),
                        RecursiveValueMut::Action(action) => action.edit(context, ui),
                        RecursiveValueMut::PainterAction(painter_action) => {
                            painter_action.edit(context, ui)
                        }
                        RecursiveValueMut::Var(var_name) => var_name.edit(context, ui),
                        RecursiveValueMut::VarValue(var_value) => var_value.edit(context, ui),
                        RecursiveValueMut::HexColor(hex_color) => hex_color.edit(context, ui),
                        RecursiveValueMut::String(_) => todo!(),
                        RecursiveValueMut::I32(v) => v.edit(context, ui),
                        RecursiveValueMut::F32(v) => v.edit(context, ui),
                        RecursiveValueMut::Bool(_) => todo!(),
                        RecursiveValueMut::Vec2(vec2) => todo!(),
                        RecursiveValueMut::Reaction(reaction) => todo!(),
                        RecursiveValueMut::Material(material) => material.edit(context, ui),
                    };
                }
                let composer = RecursiveMutComposer::new(field_renderer);
                composer.compose_mut(data, self.ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

impl<'a> RenderBuilder<'a, Expression> {
    pub fn edit_expression(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => {
                let composer = ExpressionComposer;
                composer.compose_mut(data, self.ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

impl<'a> RenderBuilder<'a, Action> {
    pub fn edit_action(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => {
                let composer = ActionComposer;
                composer.compose_mut(data, self.ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

impl<'a> RenderBuilder<'a, PainterAction> {
    pub fn edit_painter_action(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => {
                fn field_renderer(
                    ui: &mut Ui,
                    context: &Context<'_>,
                    field: RecursiveFieldMut<'_>,
                ) {
                    call_on_recursive_value_mut!(field, show, context, ui);
                }
                let composer = RecursiveMutComposer::new(field_renderer);
                composer.compose_mut(data, self.ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

impl<'a> RenderBuilder<'a, Reaction> {
    pub fn edit_reaction(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => {
                let composer = ReactionComposer;
                composer.compose_mut(data, self.ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}
