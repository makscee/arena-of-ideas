use crate::prelude::*;
use crate::ui::{DndArea, MatRect, RangeSelector};
use bevy_egui::egui::{DragAndDrop, Frame, Response, Sense};

#[derive(Debug, Clone)]
pub struct DraggedUnit {
    pub unit_id: u64,
    pub from_location: UnitTarget,
    pub drag_start_pos: Option<Pos2>,
}

#[derive(Debug, Clone, PartialEq)]
enum EditMode {
    Normal,
    EditingFusion(u64),
}

pub struct TeamEditor {
    empty_slot_actions: Vec<(
        String,
        Box<dyn FnOnce(&mut NTeam, u64, i32, &ClientContext, &mut Ui)>,
    )>,
    filled_slot_actions: Vec<(
        String,
        Box<dyn FnOnce(&mut NTeam, u64, u64, i32, &ClientContext, &mut Ui)>,
    )>,
    action_handler: Option<Box<dyn FnMut(&ClientContext, &TeamAction) -> NodeResult<()>>>,
    add_slot_text_fn: Option<Box<dyn Fn(&NFusion) -> String>>,
}

pub enum TeamAction {
    MoveUnit {
        unit_id: u64,
        target: UnitTarget,
    },
    BenchUnit {
        unit_id: u64,
    },
    StackUnit {
        unit_id: u64,
        target_unit_id: u64,
    },
    AddSlot {
        fusion_id: u64,
    },
    ChangeActionRange {
        slot_id: u64,
        start: i32,
        length: i32,
    },
    ChangeTrigger {
        fusion_id: u64,
        trigger: u64,
    },
    CustomEmptySlotAction {
        action_fn: Box<dyn FnOnce(&mut NTeam, u64, i32, &ClientContext, &mut Ui)>,
        fusion_id: u64,
        slot_index: i32,
    },
    CustomFilledSlotAction {
        action_fn: Box<dyn FnOnce(&mut NTeam, u64, u64, i32, &ClientContext, &mut Ui)>,
        fusion_id: u64,
        unit_id: u64,
        slot_index: i32,
    },
}

#[derive(Debug, Clone)]
pub enum UnitTarget {
    Bench,
    Slot { fusion_id: u64, slot_index: i32 },
}

impl TeamEditor {
    pub fn new() -> Self {
        Self {
            empty_slot_actions: vec![],
            filled_slot_actions: vec![],
            action_handler: None,
            add_slot_text_fn: None,
        }
    }

    pub fn with_action_handler<F>(mut self, handler: F) -> Self
    where
        F: FnMut(&ClientContext, &TeamAction) -> NodeResult<()> + 'static,
    {
        self.action_handler = Some(Box::new(handler));
        self
    }

    pub fn empty_slot_action(
        mut self,
        name: String,
        action: Box<dyn FnOnce(&mut NTeam, u64, i32, &ClientContext, &mut Ui)>,
    ) -> Self {
        self.empty_slot_actions.push((name, action));
        self
    }

    pub fn filled_slot_action(
        mut self,
        name: String,
        action: Box<dyn FnOnce(&mut NTeam, u64, u64, i32, &ClientContext, &mut Ui)>,
    ) -> Self {
        self.filled_slot_actions.push((name, action));
        self
    }

    pub fn with_add_slot_text<F>(mut self, text_fn: F) -> Self
    where
        F: Fn(&NFusion) -> String + 'static,
    {
        self.add_slot_text_fn = Some(Box::new(text_fn));
        self
    }

    pub fn edit(mut self, team: &NTeam, ctx: &ClientContext, ui: &mut Ui) -> Option<NTeam> {
        let mut actions = Vec::new();

        self.draw_drag_visual(ui);

        let state_id = egui::Id::new(team.id).with("team_editor_mode");
        let mut edit_mode = ui.memory(|m| {
            m.data
                .get_temp::<EditMode>(state_id)
                .unwrap_or(EditMode::Normal)
        });

        let fusions = team
            .fusions
            .get()
            .map(|f| f.iter().sorted_by_key(|f| f.index).collect_vec())
            .unwrap_or_default();

        let unlinked_units = self.get_unlinked_units(team);
        let mut mode_change = None;

        // Reserve fixed height for bench at bottom
        let bench_height = 70.0;
        let available_rect = ui.available_rect_before_wrap();
        let fusions_rect = egui::Rect::from_min_size(
            available_rect.min,
            egui::Vec2::new(
                available_rect.width(),
                available_rect.height() - bench_height,
            ),
        );
        let bench_rect = egui::Rect::from_min_size(
            egui::Pos2::new(available_rect.min.x, available_rect.max.y - bench_height),
            egui::Vec2::new(available_rect.width(), bench_height),
        );

        // Render fusions in scrollable area
        ui.scope_builder(UiBuilder::new().max_rect(fusions_rect), |ui| {
            egui::ScrollArea::vertical()
                .max_height(fusions_rect.height())
                .show(ui, |ui| {
                    if !fusions.is_empty() {
                        ui.columns(fusions.len(), |columns| {
                            for (idx, fusion) in fusions.iter().enumerate() {
                                let slots = fusion
                                    .slots
                                    .get()
                                    .map(|s| s.iter().sorted_by_key(|s| s.index).collect_vec())
                                    .unwrap_or_default();
                                let ui = &mut columns[idx];
                                match edit_mode {
                                    EditMode::Normal => {
                                        let response = self.render_fusion_normal_mode(
                                            ui,
                                            fusion,
                                            &slots,
                                            team,
                                            ctx,
                                            &mut actions,
                                        );
                                        ui.scope_builder(
                                            UiBuilder::new().max_rect(response.rect),
                                            |ui| {
                                                let response =
                                                    ui.allocate_rect(response.rect, Sense::click());
                                                if response.clicked() {
                                                    mode_change =
                                                        Some(EditMode::EditingFusion(fusion.id));
                                                }
                                                if response.hovered() {
                                                    ui.painter().rect_stroke(
                                                        response.rect,
                                                        6,
                                                        YELLOW.stroke(),
                                                        egui::StrokeKind::Middle,
                                                    );
                                                }
                                            },
                                        );
                                    }
                                    EditMode::EditingFusion(editing_id)
                                        if editing_id == fusion.id =>
                                    {
                                        self.render_fusion_edit_mode(
                                            ui,
                                            fusion,
                                            &slots,
                                            team,
                                            ctx,
                                            &mut actions,
                                        );

                                        if columns[idx].response().clicked_elsewhere() {
                                            mode_change = Some(EditMode::Normal);
                                        }
                                    }
                                    _ => {
                                        self.render_fusion_inactive(ui, fusion, &slots, team, ctx);
                                    }
                                }
                            }
                        });
                    }
                });
        });

        // Render bench at bottom as full-width rectangle
        ui.scope_builder(UiBuilder::new().max_rect(bench_rect), |ui| {
            ui.columns(2, |ui| {
                ui[0].separator();
                ui[1].separator();
                self.render_bench(&mut ui[0], &unlinked_units, team, ctx, &mut actions);
                self.render_houses(&mut ui[1], ctx, team);
            });
        });

        if let Some(new_mode) = mode_change {
            edit_mode = new_mode;
        }

        ui.memory_mut(|m| m.data.insert_temp(state_id, edit_mode));
        if !actions.is_empty() {
            let mut result_team = team.clone();
            let mut has_changes = false;

            for action in actions {
                if let Some(ref mut handler) = self.action_handler {
                    handler(ctx, &action).notify_error_op();
                }
                match action {
                    TeamAction::CustomEmptySlotAction {
                        action_fn,
                        fusion_id,
                        slot_index,
                    } => {
                        action_fn(&mut result_team, fusion_id, slot_index, ctx, ui);
                        has_changes = true;
                    }
                    TeamAction::CustomFilledSlotAction {
                        action_fn,
                        fusion_id,
                        unit_id,
                        slot_index,
                    } => {
                        action_fn(&mut result_team, fusion_id, unit_id, slot_index, ctx, ui);
                        has_changes = true;
                    }
                    other => {
                        result_team.apply_action(other).notify_error_op();
                        has_changes = true;
                    }
                }
            }

            if has_changes {
                result_team.fix_integrity().notify_error_op();
                Some(result_team)
            } else {
                None
            }
        } else {
            None
        }
    }

    fn render_fusion_normal_mode(
        &mut self,
        ui: &mut Ui,
        fusion: &NFusion,
        slots: &[&NFusionSlot],
        team: &NTeam,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) -> Response {
        ui.vertical(|ui| {
            format!("[s [tw Fusion #]]{}", fusion.index).label_w(ui);

            // Always show trigger selector
            let triggers = self.get_available_triggers(fusion, team);
            if !triggers.is_empty() {
                let mut trigger_index = triggers
                    .iter()
                    .position(|(id, _)| Some(*id) == fusion.trigger_unit.id())
                    .unwrap_or(0);

                egui::ComboBox::from_id_salt(format!("normal_{}", fusion.id))
                    .selected_text(triggers[trigger_index].1.cstr().widget(1.0, ui.style()))
                    .show_ui(ui, |ui| {
                        for (i, (id, name)) in triggers.iter().enumerate() {
                            if ui
                                .selectable_value(
                                    &mut trigger_index,
                                    i,
                                    name.cstr().widget(1.0, ui.style()),
                                )
                                .clicked()
                            {
                                actions.push(TeamAction::ChangeTrigger {
                                    fusion_id: fusion.id,
                                    trigger: *id,
                                });
                            }
                        }
                    });
                ui.separator();
            }

            let frame = Frame::new()
                .inner_margin(4.0)
                .stroke(ui.visuals().window_stroke())
                .fill(ui.visuals().faint_bg_color);
            let inner_response = frame
                .show(ui, |ui| {
                    let _ = TeamEditor::render_action_list(ui, fusion, &slots, ctx);
                })
                .response;
            if inner_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            ui.separator();
            for slot in slots {
                self.render_unit_slot(ui, slot, fusion.id, team, ctx, actions);
            }

            let add_slot_text = if let Some(ref text_fn) = self.add_slot_text_fn {
                format!("+ {}", text_fn(fusion))
            } else {
                "+ Slot".to_string()
            };
            if add_slot_text.button(ui).clicked() {
                actions.push(TeamAction::AddSlot {
                    fusion_id: fusion.id,
                });
            }
            inner_response
        })
        .inner
    }

    fn render_fusion_edit_mode(
        &self,
        ui: &mut Ui,
        fusion: &NFusion,
        slots: &[&NFusionSlot],
        team: &NTeam,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        let resp = ui
            .vertical(|ui| {
                ui.label(format!("🔧 Editing Fusion {}", fusion.index + 1));
                ui.separator();

                let triggers = self.get_available_triggers(fusion, team);
                if !triggers.is_empty() {
                    let mut trigger_index = triggers
                        .iter()
                        .position(|(id, _)| Some(*id) == fusion.trigger_unit.id())
                        .unwrap_or(0);

                    egui::ComboBox::from_id_salt(fusion.id)
                        .selected_text(triggers[trigger_index].1.cstr().widget(1.0, ui.style()))
                        .show_ui(ui, |ui| {
                            for (i, (id, name)) in triggers.iter().enumerate() {
                                if ui
                                    .selectable_value(
                                        &mut trigger_index,
                                        i,
                                        name.cstr().widget(1.0, ui.style()),
                                    )
                                    .clicked()
                                {
                                    actions.push(TeamAction::ChangeTrigger {
                                        fusion_id: fusion.id,
                                        trigger: *id,
                                    });
                                }
                            }
                        });
                }
                for slot in slots {
                    if !slot.unit.is_none() {
                        self.render_action_range_editor(ui, slot, ctx, actions);
                    }
                }
            })
            .response;
        resp.show_tooltip_ui(|ui| {
            ui.set_width(400.0);
            Self::render_action_list(ui, fusion, slots, ctx).ui(ui);
        });
    }

    fn render_fusion_inactive(
        &self,
        ui: &mut Ui,
        fusion: &NFusion,
        slots: &[&NFusionSlot],
        team: &NTeam,
        ctx: &ClientContext,
    ) {
        ui.disable();
        ui.vertical(|ui| {
            format!("[s [tw Fusion #]]{}", fusion.index).label_w(ui);
            let _ = TeamEditor::render_action_list(ui, fusion, slots, ctx);
            ui.separator();
            for slot in slots {
                if let Some(unit_id) = slot.unit.id() {
                    self.render_unit_display(ui, unit_id, team, ctx);
                } else {
                    TeamEditor::render_empty_slot(ui, ctx);
                }
            }
        });
    }

    fn render_action_list(
        ui: &mut Ui,
        fusion: &NFusion,
        slots: &[&NFusionSlot],
        ctx: &ClientContext,
    ) -> NodeResult<()> {
        ui.vertical(|ui| {
            ui.style_mut().override_text_style = Some(TextStyle::Small);
            if let Some(id) = fusion.trigger_unit.id() {
                ctx.load::<NUnit>(id)?
                    .description_ref(ctx)?
                    .behavior_ref(ctx)?
                    .reaction
                    .trigger
                    .display(ctx, ui);
            }
            ScrollArea::horizontal().id_salt(fusion.id).show(ui, |ui| {
                let mut showed = 0;
                for slot in slots {
                    if let Some(unit_id) = slot.unit.id() {
                        if let Ok(unit) = ctx.load::<NUnit>(unit_id) {
                            if let Ok(unit_desc) = unit.description_ref(ctx) {
                                if let Ok(unit_behavior) = unit_desc.behavior_ref(ctx) {
                                    let actions = &unit_behavior.reaction.actions;
                                    let range = &slot.actions;
                                    ctx.exec_ref(|ctx| {
                                        ctx.with_owner(unit.id, |ctx| {
                                            for i in range.start
                                                ..(range.start + range.length)
                                                    .min(actions.len() as u8)
                                            {
                                                showed += 1;
                                                if let Some(action) = actions.get(i as usize) {
                                                    ui.horizontal(|ui| {
                                                        action.display_recursive(ctx, ui);
                                                    });
                                                }
                                            }
                                            Ok(())
                                        })
                                    })
                                    .ui(ui);
                                }
                            }
                        }
                    }
                }
                for _ in 0..(fusion.actions_limit - showed) {
                    "[tw ...]".cstr().label(ui);
                }
            });
            Ok(())
        })
        .inner
    }

    fn render_action_range_editor(
        &self,
        ui: &mut Ui,
        slot: &NFusionSlot,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        ui.group(|ui| {
            ui.vertical(|ui| {
                if let Some(unit_id) = slot.unit.id() {
                    if let Ok(unit) = ctx.load::<NUnit>(unit_id) {
                        ui.label(&unit.unit_name);

                        if let Ok(unit_desc) = unit.description_ref(ctx) {
                            if let Ok(unit_behavior) = unit_desc.behavior_ref(ctx) {
                                let total_actions = unit_behavior.reaction.actions.len() as u8;

                                if total_actions > 0 {
                                    let range_selector = RangeSelector::new(total_actions)
                                        .range(slot.actions.start, slot.actions.length)
                                        .id(egui::Id::new(slot.id).with("range_edit"))
                                        .border_thickness(3.0)
                                        .drag_threshold(10.0);

                                    let (_, range_change) = range_selector.ui(
                                        ui,
                                        ctx,
                                        |ui, _, action_index, is_in_range| {
                                            if let Some(action) =
                                                unit_behavior.reaction.actions.get(action_index)
                                            {
                                                let title = action.title(ctx);
                                                if is_in_range {
                                                    title.label_w(ui);
                                                } else {
                                                    title
                                                        .cstr()
                                                        .get_text()
                                                        .cstr_c(ui.visuals().weak_text_color())
                                                        .label_w(ui);
                                                }
                                            }
                                            Ok(())
                                        },
                                    );

                                    if let Some((new_start, new_length)) = range_change {
                                        actions.push(TeamAction::ChangeActionRange {
                                            slot_id: slot.id,
                                            start: new_start as i32,
                                            length: new_length as i32,
                                        });
                                    }
                                }
                            }
                        }
                    }
                }
            });
        });
    }

    fn render_unit_slot(
        &mut self,
        ui: &mut Ui,
        slot: &NFusionSlot,
        fusion_id: u64,
        team: &NTeam,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        if let Some(unit_id) = slot.unit.id() {
            self.handle_slot_unit_interactions(
                ui, unit_id, fusion_id, slot.index, team, ctx, actions,
            );
        } else {
            self.handle_empty_slot_interactions(ui, fusion_id, slot.index, team, ctx, actions);
        }
    }

    fn handle_slot_unit_interactions(
        &mut self,
        ui: &mut Ui,
        unit_id: u64,
        fusion_id: u64,
        slot_index: i32,
        team: &NTeam,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        let mut response = TeamEditor::render_unit_with_representation(ui, unit_id, team, ctx);

        if response.drag_started() {
            let from_location = UnitTarget::Slot {
                fusion_id,
                slot_index,
            };
            let drag_start_pos = Some(response.rect.center());
            let dragged_unit = DraggedUnit {
                unit_id,
                from_location,
                drag_start_pos,
            };
            DragAndDrop::set_payload(ui.ctx(), dragged_unit);
        }

        if let Some(unit) = TeamEditor::find_unit_in_team(team, unit_id) {
            response = response.on_hover_ui(|ui| {
                unit.as_card().compose(ctx, ui);
            });
        }

        response.bar_menu(|ui| {
            for (i, (name, _)) in self.filled_slot_actions.iter().enumerate() {
                if ui.button(name).clicked() {
                    let (_, action) = self.filled_slot_actions.remove(i);
                    actions.push(TeamAction::CustomFilledSlotAction {
                        action_fn: action,
                        fusion_id,
                        unit_id,
                        slot_index,
                    });
                    ui.close();
                    break;
                }
            }
        });

        if let Some(dropped_unit) = DndArea::<DraggedUnit>::new(response.rect)
            .id(format!("slot_{}_{}", fusion_id, slot_index))
            .text_fn(ui, |dragged| {
                if dragged.unit_id != unit_id {
                    let dragged_unit = TeamEditor::find_unit_in_team(team, dragged.unit_id);
                    let current_unit = TeamEditor::find_unit_in_team(team, unit_id);
                    if let (Some(dragged_unit), Some(current_unit)) = (dragged_unit, current_unit) {
                        if dragged_unit.unit_name == current_unit.unit_name {
                            return "Stack\nunits".to_string();
                        }
                    }
                    "Swap\nunits".to_string()
                } else {
                    "Same\nunit".to_string()
                }
            })
            .ui(ui)
        {
            if dropped_unit.unit_id != unit_id {
                let dragged_unit = TeamEditor::find_unit_in_team(team, dropped_unit.unit_id);
                let current_unit = TeamEditor::find_unit_in_team(team, unit_id);

                if let (Some(dragged_unit), Some(current_unit)) = (dragged_unit, current_unit) {
                    if dragged_unit.unit_name == current_unit.unit_name {
                        actions.push(TeamAction::StackUnit {
                            unit_id: dropped_unit.unit_id,
                            target_unit_id: unit_id,
                        });
                        return;
                    }
                }

                match dropped_unit.from_location {
                    UnitTarget::Slot {
                        fusion_id: _from_fusion,
                        slot_index: _from_slot,
                    } => {
                        actions.push(TeamAction::MoveUnit {
                            unit_id: dropped_unit.unit_id,
                            target: UnitTarget::Slot {
                                fusion_id,
                                slot_index,
                            },
                        });
                    }
                    UnitTarget::Bench => {
                        actions.push(TeamAction::MoveUnit {
                            unit_id: dropped_unit.unit_id,
                            target: UnitTarget::Slot {
                                fusion_id,
                                slot_index,
                            },
                        });
                    }
                }
            }
        }
    }

    fn handle_empty_slot_interactions(
        &mut self,
        ui: &mut Ui,
        fusion_id: u64,
        slot_index: i32,
        _team: &NTeam,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        let response = TeamEditor::render_empty_slot(ui, ctx);

        if let Some(dropped_unit) = DndArea::<DraggedUnit>::new(response.rect)
            .id(format!("empty_slot_{}_{}", fusion_id, slot_index))
            .text_fn(ui, |_| "Place\nunit".to_string())
            .ui(ui)
        {
            actions.push(TeamAction::MoveUnit {
                unit_id: dropped_unit.unit_id,
                target: UnitTarget::Slot {
                    fusion_id,
                    slot_index,
                },
            });
        }

        response.bar_menu(|ui| {
            for (i, (name, _)) in self.empty_slot_actions.iter().enumerate() {
                if ui.button(name).clicked() {
                    let (_, action) = self.empty_slot_actions.remove(i);
                    actions.push(TeamAction::CustomEmptySlotAction {
                        action_fn: action,
                        fusion_id,
                        slot_index,
                    });
                    ui.close();
                    break;
                }
            }
        });
    }

    fn render_houses(&self, ui: &mut Ui, ctx: &ClientContext, team: &NTeam) {
        ui.horizontal_wrapped(|ui| {
            for house in team.houses().unwrap() {
                house.as_tag().compose(ctx, ui);
            }
        });
    }

    fn render_bench(
        &self,
        ui: &mut Ui,
        unlinked_units: &[&NUnit],
        team: &NTeam,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        let rect = ui.available_rect_before_wrap();
        ui.horizontal(|ui| {
            "[s [tw Bench]]".cstr().label_w(ui);
            ui.separator();

            ui.horizontal(|ui| {
                for unit in unlinked_units {
                    self.handle_bench_unit_interactions(ui, unit.id, team, ctx, actions);
                }
            });

            if let Some(dropped_unit) = DndArea::<DraggedUnit>::new(rect)
                .id("bench_drop")
                .text_fn(ui, |dragged| {
                    if let Some(unit) = TeamEditor::find_unit_in_team(team, dragged.unit_id) {
                        format!("Bench {}", unit.unit_name)
                    } else {
                        "Bench Unit".to_string()
                    }
                })
                .ui(ui)
            {
                actions.push(TeamAction::BenchUnit {
                    unit_id: dropped_unit.unit_id,
                });
            }
        });
    }

    fn handle_bench_unit_interactions(
        &self,
        ui: &mut Ui,
        unit_id: u64,
        team: &NTeam,
        ctx: &ClientContext,
        _actions: &mut Vec<TeamAction>,
    ) {
        let response = TeamEditor::render_unit_with_representation(ui, unit_id, team, ctx);

        if response.drag_started() {
            let from_location = UnitTarget::Bench;
            let drag_start_pos = Some(response.rect.center());
            let dragged_unit = DraggedUnit {
                unit_id,
                from_location,
                drag_start_pos,
            };
            DragAndDrop::set_payload(ui.ctx(), dragged_unit);
        }

        if let Some(unit) = TeamEditor::find_unit_in_team(team, unit_id) {
            response.on_hover_ui(|ui| {
                unit.as_card().compose(ctx, ui);
            });
        }
    }

    fn render_unit_with_representation(
        ui: &mut Ui,
        unit_id: u64,
        team: &NTeam,
        ctx: &ClientContext,
    ) -> Response {
        let is_inspected = ui
            .inspected_node_for_parent(team.id)
            .is_some_and(|id| id == unit_id);
        if let Some(unit) = TeamEditor::find_unit_in_team(team, unit_id) {
            if let Ok(desc) = unit.description_ref(ctx) {
                if let Ok(rep) = desc.representation_ref(ctx) {
                    MatRect::new(egui::Vec2::new(60.0, 60.0))
                        .add_mat(&rep.material, unit.id)
                        .unit_rep_with_default(unit.id)
                        .active(is_inspected)
                        .ui(ui, ctx)
                } else {
                    MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, ctx)
                }
            } else {
                MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, ctx)
            }
        } else {
            ui.label(format!("Unit #{}", unit_id))
        }
    }

    fn render_unit_display(&self, ui: &mut Ui, unit_id: u64, team: &NTeam, ctx: &ClientContext) {
        if let Some(unit) = TeamEditor::find_unit_in_team(team, unit_id) {
            if let Ok(desc) = unit.description_ref(ctx) {
                if let Ok(rep) = desc.representation_ref(ctx) {
                    MatRect::new(egui::Vec2::new(60.0, 60.0))
                        .add_mat(&rep.material, unit.id)
                        .unit_rep_with_default(unit.id)
                        .ui(ui, ctx);
                } else {
                    MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, ctx);
                }
            } else {
                MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, ctx);
            }
        } else {
            ui.label(format!("Unit #{}", unit_id));
        }
    }

    fn render_empty_slot(ui: &mut Ui, ctx: &ClientContext) -> Response {
        MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, ctx)
    }

    fn draw_drag_visual(&self, ui: &mut Ui) {
        if DragAndDrop::has_payload_of_type::<DraggedUnit>(ui.ctx()) {
            if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                if let Some(payload) = DragAndDrop::payload::<DraggedUnit>(ui.ctx()) {
                    if let Some(start_pos) = payload.drag_start_pos {
                        self.draw_drag_arrow(ui, start_pos, pointer_pos);
                    }
                }
            }
        }
    }

    fn draw_drag_arrow(&self, ui: &mut Ui, start_pos: Pos2, pointer_pos: Pos2) {
        let arrow_color = Color32::from_rgb(255, 200, 0);
        let stroke = Stroke::new(3.0, arrow_color);

        ui.painter().line_segment([start_pos, pointer_pos], stroke);

        let arrow_length = 12.0;
        let arrow_angle = 0.5;
        let direction = (pointer_pos - start_pos).normalized();
        let perpendicular = egui::Vec2::new(-direction.y, direction.x);

        let arrow_base = pointer_pos - direction * arrow_length;
        let arrow_side1 = arrow_base + perpendicular * arrow_length * arrow_angle;
        let arrow_side2 = arrow_base - perpendicular * arrow_length * arrow_angle;

        ui.painter().line_segment([arrow_base, arrow_side1], stroke);
        ui.painter().line_segment([arrow_base, arrow_side2], stroke);
        ui.painter()
            .line_segment([arrow_side1, pointer_pos], stroke);
        ui.painter()
            .line_segment([arrow_side2, pointer_pos], stroke);
    }

    fn get_unlinked_units<'a>(&self, team: &'a NTeam) -> Vec<&'a NUnit> {
        let mut linked_units = HashSet::new();

        if let Ok(fusions) = team.fusions.get() {
            for fusion in fusions {
                if let Ok(slots) = fusion.slots.get() {
                    for slot in slots {
                        if let Some(unit_id) = slot.unit.id() {
                            linked_units.insert(unit_id);
                        }
                    }
                }
            }
        }

        let mut unlinked = Vec::new();
        if let Ok(houses) = team.houses.get() {
            for house in houses {
                if let Ok(units) = house.units.get() {
                    for unit in units {
                        if !linked_units.contains(&unit.id) {
                            unlinked.push(unit);
                        }
                    }
                }
            }
        }
        unlinked
    }

    fn get_available_triggers(&self, fusion: &NFusion, team: &NTeam) -> Vec<(u64, String)> {
        let mut triggers = vec![];

        if let Ok(slots) = fusion.slots.get() {
            for slot in slots {
                if let Some(unit_id) = slot.unit.id() {
                    if let Some(unit) = Self::find_unit_in_team(team, unit_id) {
                        if let Ok(trigger) = unit
                            .description()
                            .and_then(|d| d.behavior())
                            .map(|b| b.reaction.trigger)
                        {
                            triggers.push((unit_id, trigger.cstr()))
                        }
                    }
                }
            }
        }

        triggers
    }

    fn find_unit_in_team(team: &NTeam, unit_id: u64) -> Option<NUnit> {
        if let Ok(houses) = team.houses.get() {
            for house in houses {
                if let Ok(units) = house.units.get() {
                    for unit in units {
                        if unit.id == unit_id {
                            return Some(unit.clone());
                        }
                    }
                }
            }
        }
        None
    }
}

impl NTeam {
    pub fn fix_integrity(&mut self) -> NodeResult<()> {
        let mut existing_unit_ids = std::collections::HashSet::new();

        // Collect all existing unit IDs
        if let Ok(houses) = self.houses.get() {
            for house in houses {
                if let Ok(units) = house.units.get() {
                    for unit in units {
                        existing_unit_ids.insert(unit.id);
                    }
                }
            }
        }

        // Fix broken unit references in fusion slots
        if let Ok(fusions) = self.fusions.get_mut() {
            for fusion in fusions {
                let mut available_units = HashSet::new();
                if let Ok(slots) = fusion.slots.get_mut() {
                    for slot in slots {
                        if let Some(unit_id) = slot.unit.id() {
                            if !existing_unit_ids.contains(&unit_id) {
                                slot.unit = Ref::none();
                                slot.set_dirty(true);
                            } else {
                                available_units.insert(unit_id);
                            }
                        }
                    }
                }
                if fusion.trigger_unit.id().is_some()
                    && !available_units.contains(&fusion.trigger_unit.id().unwrap())
                {
                    fusion.trigger_unit = Ref::none();
                    fusion.set_dirty(true);
                }
                if fusion.trigger_unit.id().is_none() {
                    if let Ok(slots) = fusion.slots.get() {
                        for slot in slots {
                            if let Some(unit_id) = slot.unit.id() {
                                if available_units.contains(&unit_id) {
                                    fusion.trigger_unit = Ref::Id(unit_id);
                                    fusion.set_dirty(true);
                                    break;
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    pub fn apply_action(&mut self, action: TeamAction) -> NodeResult<()> {
        match action {
            TeamAction::MoveUnit { unit_id, target } => {
                // Find the source fusion and slot for the unit being moved
                let source_fusion_id = {
                    let mut source_id = None;
                    if let Ok(fusions) = self.fusions.get() {
                        for fusion in fusions {
                            if let Ok(slots) = fusion.slots.get() {
                                if slots
                                    .iter()
                                    .any(|s| matches!(s.unit, Ref::Id(id) if id == unit_id))
                                {
                                    source_id = Some(fusion.id);
                                    break;
                                }
                            }
                        }
                    }
                    source_id
                };

                let from_slot_id = self
                    .fusions
                    .get()?
                    .iter()
                    .filter_map(|f| f.slots.get().ok())
                    .flatten()
                    .find(|s| matches!(s.unit, Ref::Id(id) if id == unit_id))
                    .map(|s| s.id);

                match target {
                    UnitTarget::Slot {
                        fusion_id,
                        slot_index,
                    } => {
                        // Check if target fusion is empty before the move
                        let target_fusion_is_empty = {
                            if let Ok(fusions) = self.fusions.get() {
                                if let Some(fusion) = fusions.iter().find(|f| f.id == fusion_id) {
                                    if let Ok(slots) = fusion.slots.get() {
                                        slots.iter().all(|s| matches!(s.unit, Ref::None))
                                    } else {
                                        false
                                    }
                                } else {
                                    false
                                }
                            } else {
                                false
                            }
                        };

                        // First, get the target slot's current unit ID (if any)
                        let target_unit_id = {
                            let target_slot = self.fusion_slot_mut(fusion_id, slot_index)?;
                            match target_slot.unit {
                                Ref::Id(id) => Some(id),
                                _ => None,
                            }
                        };

                        // Now handle the move/swap logic
                        if let Some(displaced_unit_id) = target_unit_id {
                            // Swap: Move displaced unit to source slot (if there is one)
                            if let Some(source_slot_id) = from_slot_id {
                                for fusion in self.fusions.get_mut()?.iter_mut() {
                                    if let Ok(slots) = fusion.slots.get_mut() {
                                        if let Some(slot) =
                                            slots.iter_mut().find(|s| s.id == source_slot_id)
                                        {
                                            slot.unit = Ref::new_id(displaced_unit_id);
                                            slot.set_dirty(true);
                                            break;
                                        }
                                    }
                                }
                            }
                        } else {
                            // Simple move - clear the source slot
                            if let Some(source_slot_id) = from_slot_id {
                                for fusion in self.fusions.get_mut()?.iter_mut() {
                                    if let Ok(slots) = fusion.slots.get_mut() {
                                        if let Some(slot) =
                                            slots.iter_mut().find(|s| s.id == source_slot_id)
                                        {
                                            slot.unit = Ref::none();
                                            slot.set_dirty(true);
                                            break;
                                        }
                                    }
                                }
                            }
                        }

                        // Set the target slot to have the moved unit
                        let target_slot = self.fusion_slot_mut(fusion_id, slot_index)?;
                        target_slot.unit = Ref::new_id(unit_id);
                        target_slot.set_dirty(true);

                        // Handle trigger unit logic
                        if let Ok(fusions) = self.fusions.get_mut() {
                            // If unit was moved from one fusion to another
                            if let Some(src_fusion_id) = source_fusion_id {
                                if src_fusion_id != fusion_id {
                                    // Check if moved unit was the trigger of source fusion
                                    if let Some(source_fusion) =
                                        fusions.iter_mut().find(|f| f.id == src_fusion_id)
                                    {
                                        if matches!(source_fusion.trigger_unit, Ref::Id(id) if id == unit_id)
                                        {
                                            // Try to find another unit as trigger in source fusion
                                            let new_trigger =
                                                if let Ok(slots) = source_fusion.slots.get() {
                                                    slots
                                                        .iter()
                                                        .find_map(|s| s.unit.id())
                                                        .filter(|&id| id != unit_id)
                                                } else {
                                                    None
                                                };

                                            if let Some(new_trigger_id) = new_trigger {
                                                source_fusion.trigger_unit =
                                                    Ref::new_id(new_trigger_id);
                                            } else {
                                                source_fusion.trigger_unit = Ref::none();
                                            }
                                        }
                                    }

                                    // If target fusion was empty, set moved unit as trigger
                                    if target_fusion_is_empty {
                                        if let Some(target_fusion) =
                                            fusions.iter_mut().find(|f| f.id == fusion_id)
                                        {
                                            target_fusion.trigger_unit = Ref::new_id(unit_id);
                                        }
                                    }
                                }
                            }
                        }
                    }
                    UnitTarget::Bench => {
                        // Move to bench - clear the source slot
                        if let Some(source_slot_id) = from_slot_id {
                            for fusion in self.fusions.get_mut()?.iter_mut() {
                                if let Ok(slots) = fusion.slots.get_mut() {
                                    if let Some(slot) =
                                        slots.iter_mut().find(|s| s.id == source_slot_id)
                                    {
                                        slot.unit = Ref::none();
                                        slot.set_dirty(true);
                                        break;
                                    }
                                }
                            }
                        }

                        // Handle trigger unit logic for bench moves
                        if let (Some(src_fusion_id), Ok(fusions)) =
                            (source_fusion_id, self.fusions.get_mut())
                        {
                            if let Some(source_fusion) =
                                fusions.iter_mut().find(|f| f.id == src_fusion_id)
                            {
                                if matches!(source_fusion.trigger_unit, Ref::Id(id) if id == unit_id)
                                {
                                    // Try to find another unit as trigger in source fusion
                                    let new_trigger = if let Ok(slots) = source_fusion.slots.get() {
                                        slots
                                            .iter()
                                            .find_map(|s| s.unit.id())
                                            .filter(|&id| id != unit_id)
                                    } else {
                                        None
                                    };

                                    if let Some(new_trigger_id) = new_trigger {
                                        source_fusion.trigger_unit = Ref::new_id(new_trigger_id);
                                    } else {
                                        source_fusion.trigger_unit = Ref::none();
                                    }
                                }
                            }
                        }
                    }
                }
            }
            TeamAction::BenchUnit { unit_id } => {
                if let Ok(slot) = self.find_unit_slot_mut(unit_id) {
                    slot.unit = Ref::default();
                }
            }
            TeamAction::AddSlot { fusion_id } => {
                let fusions = self.fusions.get_mut()?;
                if let Some(fusion) = fusions.iter_mut().find(|f| f.id == fusion_id) {
                    let new_index = fusion.slots().map(|s| s.len() as i32).unwrap_or_default();
                    fusion.slots_push(NFusionSlot::new(
                        next_id(),
                        player_id(),
                        new_index,
                        default(),
                    ))?;
                }
            }
            TeamAction::ChangeActionRange {
                slot_id,
                start,
                length,
            } => {
                if let Ok(fusions) = self.fusions.get_mut() {
                    for fusion in fusions {
                        if let Ok(slots) = fusion.slots.get_mut() {
                            if let Some(slot) = slots.iter_mut().find(|s| s.id == slot_id) {
                                slot.actions.start = start as u8;
                                slot.actions.length = length as u8;
                            }
                        }
                    }
                }
            }
            TeamAction::ChangeTrigger { fusion_id, trigger } => {
                if let Ok(fusions) = self.fusions.get_mut() {
                    if let Some(fusion) = fusions.iter_mut().find(|f| f.id == fusion_id) {
                        fusion.trigger_unit = Ref::Id(trigger);
                    }
                }
            }
            TeamAction::StackUnit {
                unit_id: _,
                target_unit_id: _,
            } => {
                // Client-side stacking is handled by server reducer
                // This is just a placeholder for the action
            }
            TeamAction::CustomEmptySlotAction {
                action_fn: _,
                fusion_id: _,
                slot_index: _,
            } => {
                // Custom actions are already handled in edit() method
            }
            TeamAction::CustomFilledSlotAction {
                action_fn: _,
                fusion_id: _,
                unit_id: _,
                slot_index: _,
            } => {
                // Custom actions are already handled in edit() method
            }
        }
        Ok(())
    }

    fn find_unit_slot_mut(&mut self, unit_id: u64) -> NodeResult<&mut NFusionSlot> {
        for fusion in self.fusions.get_mut().track()? {
            if let Ok(slots) = fusion.slots.get_mut() {
                for slot in slots {
                    if slot.unit.id().unwrap_or_default() == unit_id {
                        return Ok(slot);
                    }
                }
            }
        }
        Err(NodeError::custom("Unit slot not found"))
    }

    pub fn fusion_slot_mut(&mut self, id: u64, index: i32) -> NodeResult<&mut NFusionSlot> {
        let fusions = self.fusions.get_mut()?;
        if let Some(fusion) = fusions.iter_mut().find(|f| f.id == id) {
            if let Some(slot) = fusion
                .slots
                .get_mut()
                .unwrap()
                .iter_mut()
                .find(|s| s.index == index)
            {
                Ok(slot)
            } else {
                Err(NodeError::custom("Fusion slot not found"))
            }
        } else {
            Err(NodeError::custom("Fusion not found"))
        }
    }
}
