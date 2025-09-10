use super::super::*;
use crate::plugins::RepresentationPlugin;
use crate::ui::see::{CstrTrait, RecursiveFieldMut, RecursiveValueMut, SFnShowMut};
use egui::{CollapsingHeader, ScrollArea, Sense, Stroke};

/// A generic recursive editor that can handle any recursive type (Expression, Action, Material, Behavior, etc.)
pub struct RecursiveEditor;

impl RecursiveEditor {
    /// Edit any recursive field with full tree navigation and manipulation
    pub fn edit_field(mut field: RecursiveFieldMut, context: &Context, ui: &mut Ui) -> bool {
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
            RecursiveValueMut::Action(action) => {
                if let Some(n) = (**action)
                    .see_mut(context)
                    .ctxbtn()
                    .add_paste()
                    .add_copy()
                    .ui_enum(ui)
                    .selector_changed()
                {
                    RecursiveValueMut::replace_action_and_move_fields(action, n.clone());
                    true
                } else {
                    false
                }
            }
            RecursiveValueMut::PainterAction(painter_action) => {
                if let Some(n) = (**painter_action)
                    .see_mut(context)
                    .ctxbtn()
                    .add_paste()
                    .add_copy()
                    .ui_enum(ui)
                    .selector_changed()
                {
                    RecursiveValueMut::replace_painter_action_and_move_fields(
                        painter_action,
                        n.clone(),
                    );
                    true
                } else {
                    false
                }
            }
            RecursiveValueMut::Reaction(reaction) => {
                ui.vertical(|ui| {
                    ui.horizontal(|ui| {
                        ui.label("Trigger:");
                        reaction.trigger.show_mut(context, ui);
                    });
                    ui.label("Actions:");
                    for (i, action) in reaction.actions.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            format!("[{}]", i).cstr().label(ui);
                            action.show_mut(context, ui);
                        });
                    }
                });
                false
            }
            RecursiveValueMut::Material(material) => {
                ui.vertical(|ui| {
                    ui.label("Painter Actions:");
                    for (i, action) in material.0.iter_mut().enumerate() {
                        ui.horizontal(|ui| {
                            format!("[{}]", i).cstr().label(ui);
                            action.show_mut(context, ui);
                        });
                    }
                });
                false
            }
            _ => false,
        }
    }

    /// Render a full recursive editor UI for any type that supports recursive editing
    pub fn ui<T>(data: &mut T, context: &Context, ui: &mut Ui) -> bool
    where
        T: FRecursive + ToRecursiveValueMut + RecursiveFieldsMut,
    {
        let mut changed = false;
        data.render_mut(context)
            .recursive_mut(ui, |ui, context, field| {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        if !field.name.is_empty() {
                            format!("[tw [s {}]]", field.name).label(ui);
                        }
                        changed |= Self::edit_field(field, context, ui);
                    });
                });
            });
        changed
    }

    /// Render a Material editor with preview
    pub fn edit_material(material: &mut Material, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.horizontal(|ui| {
            // Left side: Material preview
            ui.vertical(|ui| {
                ui.label("Preview:");
                let size = 100.0;
                let (rect, _) = ui.allocate_exact_size(egui::vec2(size, size), Sense::hover());
                RepresentationPlugin::paint_rect(rect, context, material, ui).ui(ui);
                ui.painter().rect_stroke(
                    rect,
                    0,
                    Stroke::new(1.0, subtle_borders_and_separators()),
                    egui::StrokeKind::Middle,
                );
            });

            ui.separator();

            // Right side: Recursive editor for PainterActions
            ui.vertical(|ui| {
                ui.label("Painter Actions:");
                ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                    changed |= Self::ui(material, context, ui);

                    // Add/Remove actions
                    ui.separator();
                    ui.horizontal(|ui| {
                        if ui.button("➕ Add Action").clicked() {
                            material
                                .0
                                .push(PainterAction::circle(Box::new(Expression::f32(0.5))));
                            changed = true;
                        }

                        if !material.0.is_empty() && ui.button("➖ Remove Last").clicked() {
                            material.0.pop();
                            changed = true;
                        }
                    });
                });
            });
        });

        changed
    }

    /// Render a Behavior editor (contains Reaction)
    pub fn edit_behavior(behavior: &mut NUnitBehavior, context: &Context, ui: &mut Ui) -> bool {
        let mut changed = false;

        ui.vertical(|ui| {
            // Magic type selector
            ui.horizontal(|ui| {
                ui.label("Magic Type:");
                changed |= behavior.magic_type.show_mut(context, ui);
            });

            ui.separator();

            // Reaction editor with full recursive support
            ui.label("Reaction:");
            ui.group(|ui| {
                ScrollArea::vertical().max_height(400.0).show(ui, |ui| {
                    changed |= Self::ui(&mut behavior.reaction, context, ui);
                });
            });
        });

        changed
    }

    /// Create a collapsible tree view for nested structures
    pub fn tree_view<T>(
        data: &mut T,
        context: &Context,
        ui: &mut Ui,
        id_salt: impl std::hash::Hash,
    ) -> bool
    where
        T: FRecursive + ToRecursiveValueMut + RecursiveFieldsMut + FTitle,
    {
        let mut changed = false;

        CollapsingHeader::new(data.title(context).get_text())
            .id_salt(id_salt)
            .show(ui, |ui| {
                changed = Self::ui(data, context, ui);
            });

        changed
    }

    /// Edit with inline actions (copy, paste, delete)
    pub fn edit_with_actions<T>(
        data: &mut T,
        context: &Context,
        ui: &mut Ui,
        on_delete: impl FnOnce(),
    ) -> bool
    where
        T: FRecursive + ToRecursiveValueMut + RecursiveFieldsMut + FTitle + FCopy + FPaste,
    {
        let mut changed = false;

        ui.horizontal(|ui| {
            // Title
            data.title(context).label(ui);

            ui.with_layout(egui::Layout::right_to_left(egui::Align::Center), |ui| {
                // Action buttons
                if ui.small_button("🗑").on_hover_text("Delete").clicked() {
                    on_delete();
                    changed = true;
                }

                if ui.small_button("📋").on_hover_text("Paste").clicked() {
                    if let Some(pasted) = T::paste_from_clipboard() {
                        *data = pasted;
                        changed = true;
                    }
                }

                if ui.small_button("📑").on_hover_text("Copy").clicked() {
                    data.copy_to_clipboard();
                }
            });
        });

        if !changed {
            ui.indent("content", |ui| {
                changed = Self::ui(data, context, ui);
            });
        }

        changed
    }

    /// Helper to render a list of recursive items with add/remove/reorder
    pub fn edit_list<T>(items: &mut Vec<T>, context: &Context, ui: &mut Ui, item_name: &str) -> bool
    where
        T: FRecursive + ToRecursiveValueMut + RecursiveFieldsMut + FTitle + Default + Clone,
    {
        let mut changed = false;
        let mut to_remove = None;
        let mut to_move = None;
        let len = items.len();
        for (i, item) in items.iter_mut().enumerate() {
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
                    changed |= Self::tree_view(item, context, ui, i);
                });

                // Delete button
                if ui.small_button("🗑").clicked() {
                    to_remove = Some(i);
                }
            });

            ui.separator();
        }

        // Handle removals
        if let Some(idx) = to_remove {
            items.remove(idx);
            changed = true;
        }

        // Handle moves
        if let Some((from, to)) = to_move {
            items.swap(from, to);
            changed = true;
        }

        // Add button
        if ui.button(format!("➕ Add {}", item_name)).clicked() {
            items.push(T::default());
            changed = true;
        }

        changed
    }
}

/// Extension trait for Material
pub trait MaterialExt {
    fn edit_advanced(&mut self, context: &Context, ui: &mut Ui) -> bool;
}

impl MaterialExt for Material {
    fn edit_advanced(&mut self, context: &Context, ui: &mut Ui) -> bool {
        RecursiveEditor::edit_material(self, context, ui)
    }
}

/// Extension trait for NUnitBehavior
pub trait BehaviorExt {
    fn edit_advanced(&mut self, context: &Context, ui: &mut Ui) -> bool;
}

impl BehaviorExt for NUnitBehavior {
    fn edit_advanced(&mut self, context: &Context, ui: &mut Ui) -> bool {
        RecursiveEditor::edit_behavior(self, context, ui)
    }
}

/// Extension for RenderBuilder to use recursive editor
impl<'a, T> RenderBuilder<'a, T>
where
    T: FRecursive + ToRecursiveValueMut + RecursiveFieldsMut,
{
    /// Edit with the recursive editor
    pub fn edit_recursive(self, ui: &mut Ui) -> bool {
        match self.data {
            RenderDataRef::Mutable(data) => RecursiveEditor::ui(data, self.ctx, ui),
            RenderDataRef::Immutable(_) => {
                panic!("Cannot edit immutable data");
            }
        }
    }
}
