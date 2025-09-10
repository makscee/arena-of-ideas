use super::super::*;
use crate::plugins::RepresentationPlugin;
use crate::ui::see::{RecursiveValueMut, SFnShowMut};
use crate::{Action, Expression, Material, PainterAction, Reaction, Trigger};
use egui::{Sense, Stroke};

/// A composer for editing lists of recursive items with full manipulation controls
pub struct EditableListComposer<T> {
    item_name: String,
    allow_reorder: bool,
    allow_delete: bool,
    allow_add: bool,
    allow_copy_paste: bool,
    default_item: Box<dyn Fn() -> T>,
}

impl<T: Clone> EditableListComposer<T> {
    pub fn new(item_name: impl Into<String>, default_item: impl Fn() -> T + 'static) -> Self {
        Self {
            item_name: item_name.into(),
            allow_reorder: true,
            allow_delete: true,
            allow_add: true,
            allow_copy_paste: true,
            default_item: Box::new(default_item),
        }
    }

    pub fn with_reorder(mut self, allow: bool) -> Self {
        self.allow_reorder = allow;
        self
    }

    pub fn with_delete(mut self, allow: bool) -> Self {
        self.allow_delete = allow;
        self
    }

    pub fn with_add(mut self, allow: bool) -> Self {
        self.allow_add = allow;
        self
    }

    pub fn with_copy_paste(mut self, allow: bool) -> Self {
        self.allow_copy_paste = allow;
        self
    }
}

impl<T> ComposerMut<Vec<T>> for EditableListComposer<T>
where
    T: Clone + 'static,
{
    fn compose_mut(&self, data: &mut Vec<T>, _context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        let mut to_remove = None;
        let mut to_move = None;
        let len = data.len();

        for (i, _item) in data.iter().enumerate() {
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

                // Item content - this is where the type-specific rendering happens
                ui.vertical(|ui| {
                    ui.label(format!("{} #{}", self.item_name, i + 1));
                });

                // Delete button
                if self.allow_delete && ui.small_button("🗑").clicked() {
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
        if self.allow_add && ui.button(format!("➕ Add {}", self.item_name)).clicked() {
            data.push((self.default_item)());
            changed = true;
        }

        changed
    }
}

/// Specialized composer for editing Action lists with recursive rendering
pub struct ActionListComposer {
    allow_reorder: bool,
    allow_delete: bool,
    allow_add: bool,
}

impl Default for ActionListComposer {
    fn default() -> Self {
        Self {
            allow_reorder: true,
            allow_delete: true,
            allow_add: true,
        }
    }
}

impl ActionListComposer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ComposerMut<Vec<Action>> for ActionListComposer {
    fn compose_mut(&self, data: &mut Vec<Action>, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        let mut to_remove = None;
        let mut to_move = None;
        let len = data.len();

        for (i, action) in data.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                // Reorder buttons
                if self.allow_reorder {
                    ui.vertical(|ui| {
                        ui.add_enabled_ui(i > 0, |ui| {
                            if ui.small_button("↑").clicked() {
                                to_move = Some((i, i - 1));
                            }
                        });
                        ui.add_enabled_ui(i < len - 1, |ui| {
                            if ui.small_button("↓").clicked() {
                                to_move = Some((i, i + 1));
                            }
                        });
                    });
                }

                // Action selector and recursive fields
                ui.vertical(|ui| {
                    // Use the render system for the selector with context menu
                    let response = action
                        .see_mut(context)
                        .ctxbtn()
                        .add_copy()
                        .add_paste()
                        .ui_enum(ui);

                    if let Some(new_action) = response.selector_changed() {
                        // Preserve fields when changing action type
                        RecursiveValueMut::replace_action_and_move_fields(
                            action,
                            new_action.clone(),
                        );
                        changed = true;
                    }

                    if let Some(replacement) = response.pasted() {
                        *action = replacement.clone();
                        changed = true;
                    }

                    // Show recursive fields for editing
                    ui.indent("fields", |ui| {
                        changed |= action.show_mut(context, ui);
                    });
                });

                // Delete button
                if self.allow_delete && ui.small_button("🗑").clicked() {
                    to_remove = Some(i);
                }
            });
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
        if self.allow_add && ui.button("➕ Add Action").clicked() {
            data.push(Action::noop);
            changed = true;
        }

        changed
    }
}

/// Specialized composer for editing PainterAction lists
pub struct PainterActionListComposer {
    allow_reorder: bool,
    allow_delete: bool,
    allow_add: bool,
}

impl Default for PainterActionListComposer {
    fn default() -> Self {
        Self {
            allow_reorder: true,
            allow_delete: true,
            allow_add: true,
        }
    }
}

impl PainterActionListComposer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ComposerMut<Vec<PainterAction>> for PainterActionListComposer {
    fn compose_mut(&self, data: &mut Vec<PainterAction>, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        let mut to_remove = None;
        let mut to_move = None;
        let len = data.len();

        for (i, action) in data.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                // Reorder buttons
                if self.allow_reorder {
                    ui.vertical(|ui| {
                        ui.add_enabled_ui(i > 0, |ui| {
                            if ui.small_button("↑").clicked() {
                                to_move = Some((i, i - 1));
                            }
                        });
                        ui.add_enabled_ui(i < len - 1, |ui| {
                            if ui.small_button("↓").clicked() {
                                to_move = Some((i, i + 1));
                            }
                        });
                    });
                }

                // PainterAction selector and recursive fields
                ui.vertical(|ui| {
                    // Use the render system for the selector with context menu
                    let response = action
                        .see_mut(context)
                        .ctxbtn()
                        .add_copy()
                        .add_paste()
                        .ui_enum(ui);

                    if let Some(new_action) = response.selector_changed() {
                        // Preserve fields when changing action type
                        RecursiveValueMut::replace_painter_action_and_move_fields(
                            action,
                            new_action.clone(),
                        );
                        changed = true;
                    }

                    if let Some(replacement) = response.pasted() {
                        *action = replacement.clone();
                        changed = true;
                    }

                    // Show recursive fields for editing
                    ui.indent("fields", |ui| {
                        changed |= action.show_mut(context, ui);
                    });
                });

                // Delete button
                if self.allow_delete && ui.small_button("🗑").clicked() {
                    to_remove = Some(i);
                }
            });
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
        if self.allow_add && ui.button("➕ Add Painter Action").clicked() {
            data.push(PainterAction::circle(Box::new(Expression::f32(0.5))));
            changed = true;
        }

        changed
    }
}

/// Specialized composer for editing Reaction lists
pub struct ReactionListComposer {
    allow_reorder: bool,
    allow_delete: bool,
    allow_add: bool,
}

impl Default for ReactionListComposer {
    fn default() -> Self {
        Self {
            allow_reorder: true,
            allow_delete: true,
            allow_add: true,
        }
    }
}

impl ReactionListComposer {
    pub fn new() -> Self {
        Self::default()
    }
}

impl ComposerMut<Vec<Reaction>> for ReactionListComposer {
    fn compose_mut(&self, data: &mut Vec<Reaction>, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        let mut to_remove = None;
        let mut to_move = None;
        let len = data.len();

        for (i, reaction) in data.iter_mut().enumerate() {
            ui.group(|ui| {
                ui.horizontal(|ui| {
                    // Reorder buttons
                    if self.allow_reorder {
                        ui.vertical(|ui| {
                            ui.add_enabled_ui(i > 0, |ui| {
                                if ui.small_button("↑").clicked() {
                                    to_move = Some((i, i - 1));
                                }
                            });
                            ui.add_enabled_ui(i < len - 1, |ui| {
                                if ui.small_button("↓").clicked() {
                                    to_move = Some((i, i + 1));
                                }
                            });
                        });
                    }

                    // Reaction content
                    ui.vertical(|ui| {
                        // Edit trigger
                        ui.horizontal(|ui| {
                            ui.label("Trigger:");
                            if reaction.trigger.show_mut(context, ui) {
                                changed = true;
                            }
                        });

                        // Edit actions using ActionListComposer
                        ui.label("Actions:");
                        if ActionListComposer::new().compose_mut(&mut reaction.actions, context, ui)
                        {
                            changed = true;
                        }
                    });

                    // Delete button
                    if self.allow_delete && ui.small_button("🗑").clicked() {
                        to_remove = Some(i);
                    }
                });
            });
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
        if self.allow_add && ui.button("➕ Add Reaction").clicked() {
            data.push(Reaction {
                trigger: Trigger::BattleStart,
                actions: vec![Action::noop],
            });
            changed = true;
        }

        changed
    }
}

/// Composer for editing Material with preview
pub struct MaterialComposer {
    show_preview: bool,
    preview_size: f32,
}

impl Default for MaterialComposer {
    fn default() -> Self {
        Self {
            show_preview: true,
            preview_size: 100.0,
        }
    }
}

impl MaterialComposer {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn with_preview_size(mut self, size: f32) -> Self {
        self.preview_size = size;
        self
    }
}

impl ComposerMut<Material> for MaterialComposer {
    fn compose_mut(&self, data: &mut Material, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            // Preview
            if self.show_preview {
                ui.vertical(|ui| {
                    ui.label("Preview:");
                    let size = self.preview_size;
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
            }

            // Painter actions editor
            ui.vertical(|ui| {
                ui.label("Painter Actions:");
                if PainterActionListComposer::new().compose_mut(&mut data.0, context, ui) {
                    changed = true;
                }
            });
        });

        changed
    }
}

/// Extension methods for RenderBuilder to use these composers
impl<'a> RenderBuilder<'a, Vec<Action>> {
    pub fn edit_action_list(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => {
                ActionListComposer::new().compose_mut(data, self.ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

impl<'a> RenderBuilder<'a, Vec<PainterAction>> {
    pub fn edit_painter_action_list(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => {
                PainterActionListComposer::new().compose_mut(data, self.ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

impl<'a> RenderBuilder<'a, Vec<Reaction>> {
    pub fn edit_reaction_list(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => {
                ReactionListComposer::new().compose_mut(data, self.ctx, ui)
            }
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

impl<'a> RenderBuilder<'a, Material> {
    pub fn edit_material(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => MaterialComposer::new().compose_mut(data, self.ctx, ui),
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}

/// Generic recursive list editor for Expression lists
pub struct ExpressionListComposer;

impl ComposerMut<Vec<Expression>> for ExpressionListComposer {
    fn compose_mut(&self, data: &mut Vec<Expression>, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;
        let mut to_remove = None;
        let mut to_move = None;
        let len = data.len();

        for (i, expr) in data.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                // Reorder buttons
                ui.vertical(|ui| {
                    ui.add_enabled_ui(i > 0, |ui| {
                        if ui.small_button("↑").clicked() {
                            to_move = Some((i, i - 1));
                        }
                    });
                    ui.add_enabled_ui(i < len - 1, |ui| {
                        if ui.small_button("↓").clicked() {
                            to_move = Some((i, i + 1));
                        }
                    });
                });

                // Expression selector and recursive fields
                ui.vertical(|ui| {
                    let response = expr
                        .see_mut(context)
                        .ctxbtn()
                        .add_copy()
                        .add_paste()
                        .ui_enum(ui);

                    if let Some(new_expr) = response.selector_changed() {
                        RecursiveValueMut::replace_expr_and_move_fields(expr, new_expr.clone());
                        changed = true;
                    }

                    if let Some(replacement) = response.pasted() {
                        *expr = replacement.clone();
                        changed = true;
                    }

                    // Show recursive fields
                    ui.indent("fields", |ui| {
                        changed |= expr.show_mut(context, ui);
                    });
                });

                // Delete button
                if ui.small_button("🗑").clicked() {
                    to_remove = Some(i);
                }
            });
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
        if ui.button("➕ Add Expression").clicked() {
            data.push(Expression::f32(0.0));
            changed = true;
        }

        changed
    }
}

impl<'a> RenderBuilder<'a, Vec<Expression>> {
    pub fn edit_expression_list(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => ExpressionListComposer.compose_mut(data, self.ctx, ui),
            RenderDataRef::Immutable(_) => panic!("Cannot edit immutable data"),
        }
    }
}
