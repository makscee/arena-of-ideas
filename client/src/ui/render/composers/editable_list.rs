use super::super::*;
use crate::plugins::RepresentationPlugin;
use crate::ui::see::{RecursiveFieldMut, RecursiveValueMut, SFnShowMut, ToRecursiveValueMut};
use crate::{Action, Expression, Material, PainterAction, Reaction};
use egui::{Sense, Stroke};

/// Generic composer for any type that implements RecursiveValueMut
pub struct RecursiveComposer;

impl RecursiveComposer {
    /// Compose any type that can be converted to RecursiveValueMut
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
                ui.horizontal(|ui| {
                    ui.label("Trigger:");
                    if (**r).trigger.show_mut(context, ui) {
                        changed = true;
                    }
                });

                ui.label("Actions:");
                ui.indent("reaction_actions", |ui| {
                    changed |= RecursiveListEditor::edit_list(&mut (**r).actions, context, ui);
                });
            }
            RecursiveValueMut::Material(m) => {
                ui.horizontal(|ui| {
                    ui.vertical(|ui| {
                        ui.label("Preview:");
                        let size = 100.0;
                        let (rect, _) =
                            ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
                        RepresentationPlugin::paint_rect(rect, context, *m, ui).ui(ui);
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
                        changed |= RecursiveListEditor::edit_list(&mut (**m).0, context, ui);
                    });
                });
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

/// Editor for lists of recursive types with full manipulation controls
pub struct RecursiveListEditor;

impl RecursiveListEditor {
    /// Edit any list of types that implement ToRecursiveValueMut
    pub fn edit_list<T>(data: &mut Vec<T>, context: &Context, ui: &mut Ui) -> bool
    where
        T: ToRecursiveValueMut + Clone + Default,
    {
        let mut changed = false;
        let mut to_remove = None;
        let mut to_move = None;
        let len = data.len();

        for (i, item) in data.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                // Reorder buttons
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

                // Item content
                ui.vertical(|ui| {
                    ui.label(format!("Item #{}", i + 1));
                    let mut field = RecursiveFieldMut::indexed(i, item.to_recursive_value_mut());
                    changed |= RecursiveComposer::compose_recursive_value(&mut field, context, ui);
                });

                // Delete button
                if ui.small_button("🗑").clicked() {
                    to_remove = Some(i);
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
        if ui.button("➕ Add Item").clicked() {
            data.push(T::default());
            changed = true;
        }

        changed
    }

    /// Edit any type with recursive fields
    pub fn edit_recursive<T>(data: &mut T, context: &Context, ui: &mut Ui) -> bool
    where
        T: FRecursiveMut + ToRecursiveValueMut,
    {
        let mut changed = false;
        let fields = data.recursive_fields_mut();

        for mut field in fields {
            if !field.name.is_empty() {
                ui.label(&field.name);
            }

            ui.indent(format!("field_{}", field.name), |ui| {
                changed |= RecursiveComposer::compose_recursive_value(&mut field, context, ui);
            });
        }

        changed
    }
}

/// Generic composer that works with any RecursiveValueMut type
pub struct UniversalComposer;

impl<T> ComposerMut<T> for UniversalComposer
where
    T: FRecursiveMut + ToRecursiveValueMut,
{
    fn compose_mut(&self, data: &mut T, context: &Context, ui: &mut Ui) -> bool {
        RecursiveListEditor::edit_recursive(data, context, ui)
    }
}

/// Composer specifically for Vec<T> where T implements ToRecursiveValueMut
pub struct VecComposer<T> {
    _phantom: std::marker::PhantomData<T>,
}

impl<T> VecComposer<T> {
    pub fn new() -> Self {
        Self {
            _phantom: std::marker::PhantomData,
        }
    }
}

impl<T> ComposerMut<Vec<T>> for VecComposer<T>
where
    T: ToRecursiveValueMut + Clone + Default,
{
    fn compose_mut(&self, data: &mut Vec<T>, context: &Context, ui: &mut Ui) -> bool {
        RecursiveListEditor::edit_list(data, context, ui)
    }
}

/// Extension methods for RenderBuilder to use recursive composers
impl<'a, T> RenderBuilder<'a, T>
where
    T: FRecursiveMut + ToRecursiveValueMut,
{
    /// Edit any recursive type with full tree navigation
    pub fn edit_recursive_value(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => RecursiveListEditor::edit_recursive(data, self.ctx, ui),
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

/// Extension for editing Vec<T> where T implements recursive traits
impl<'a, T> RenderBuilder<'a, Vec<T>>
where
    T: ToRecursiveValueMut + Clone + Default,
{
    pub fn edit_recursive_list(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => RecursiveListEditor::edit_list(data, self.ctx, ui),
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

/// Specialized extensions for commonly used types
impl<'a> RenderBuilder<'a, Vec<Action>> {
    pub fn edit_action_list(self, ui: &mut Ui) -> bool {
        self.edit_recursive_list(ui)
    }
}

impl<'a> RenderBuilder<'a, Vec<PainterAction>> {
    pub fn edit_painter_action_list(self, ui: &mut Ui) -> bool {
        self.edit_recursive_list(ui)
    }
}

impl<'a> RenderBuilder<'a, Vec<Expression>> {
    pub fn edit_expression_list(self, ui: &mut Ui) -> bool {
        self.edit_recursive_list(ui)
    }
}

impl<'a> RenderBuilder<'a, Vec<Reaction>> {
    pub fn edit_reaction_list(self, ui: &mut Ui) -> bool {
        self.edit_recursive_list(ui)
    }
}

impl<'a> RenderBuilder<'a, Material> {
    pub fn edit_material(self, ui: &mut Ui) -> bool {
        self.edit_recursive_value(ui)
    }
}

impl<'a> RenderBuilder<'a, Expression> {
    pub fn edit_expression(self, ui: &mut Ui) -> bool {
        self.edit_recursive_value(ui)
    }
}

impl<'a> RenderBuilder<'a, Action> {
    pub fn edit_action(self, ui: &mut Ui) -> bool {
        self.edit_recursive_value(ui)
    }
}

impl<'a> RenderBuilder<'a, PainterAction> {
    pub fn edit_painter_action(self, ui: &mut Ui) -> bool {
        self.edit_recursive_value(ui)
    }
}

impl<'a> RenderBuilder<'a, Reaction> {
    pub fn edit_reaction(self, ui: &mut Ui) -> bool {
        self.edit_recursive_value(ui)
    }
}
