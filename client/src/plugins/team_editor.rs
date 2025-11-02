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
}

pub enum TeamAction {
    MoveUnit {
        unit_id: u64,
        target: UnitTarget,
    },
    BenchUnit {
        unit_id: u64,
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
        let bench_height = 60.0;
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
            self.render_bench_horizontal(ui, &unlinked_units, team, ctx, &mut actions);
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
            let frame = Frame::new()
                .inner_margin(4.0)
                .stroke(ui.visuals().window_stroke())
                .fill(ui.visuals().faint_bg_color);
            let inner_response = frame
                .show(ui, |ui| {
                    TeamEditor::render_action_list_static(ui, fusion, &slots, ctx);
                })
                .response;
            if inner_response.hovered() {
                ui.ctx().set_cursor_icon(egui::CursorIcon::PointingHand);
            }

            ui.separator();
            for slot in slots {
                self.render_unit_slot(ui, slot, fusion.id, team, ctx, actions);
            }

            if ui.button("+ Add Slot").clicked() {
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
        ui.vertical(|ui| {
            ui.label(format!("🔧 Editing Fusion {}", fusion.index + 1));
            ui.separator();

            let triggers = self.get_available_triggers(fusion, team);
            if !triggers.is_empty() {
                let mut trigger_index = triggers
                    .iter()
                    .position(|(id, _)| *id == fusion.trigger_unit)
                    .unwrap_or(0);

                ui.label("Trigger:");
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
                ui.separator();
            }

            for slot in slots {
                self.render_action_range_editor(ui, slot, ctx, actions);
            }
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
            TeamEditor::render_action_list_static(ui, fusion, slots, ctx);
            ui.separator();
            for slot in slots {
                if let Some(unit_id) = slot.unit.id() {
                    self.render_unit_display(ui, unit_id, team, ctx);
                } else {
                    TeamEditor::render_empty_slot_static(ui, ctx);
                }
            }
        });
    }

    fn render_action_list_static(
        ui: &mut Ui,
        fusion: &NFusion,
        slots: &[&NFusionSlot],
        ctx: &ClientContext,
    ) -> NodeResult<()> {
        ui.vertical(|ui| {
            let mut action_list = Vec::new();
            if fusion.trigger_unit != 0 {
                ctx.load::<NUnit>(fusion.trigger_unit)?
                    .description_ref(ctx)?
                    .behavior_ref(ctx)?
                    .reaction
                    .trigger
                    .display(ctx, ui);
            }
            for slot in slots {
                if let Some(unit_id) = slot.unit.id() {
                    if let Ok(unit) = ctx.load::<NUnit>(unit_id) {
                        if let Ok(unit_desc) = unit.description_ref(ctx) {
                            if let Ok(unit_behavior) = unit_desc.behavior_ref(ctx) {
                                let actions = &unit_behavior.reaction.actions;
                                let range = &slot.actions;
                                let is_trigger = unit_id == fusion.trigger_unit;

                                for i in range.start
                                    ..(range.start + range.length).min(actions.len() as u8)
                                {
                                    if let Some(action) = actions.get(i as usize) {
                                        let title = action.title(ctx);
                                        if is_trigger {
                                            action_list.push(format!("🎯 {}", title));
                                        } else {
                                            action_list.push(title);
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }

            if action_list.is_empty() {
                "[s [tw No actions configured]]".cstr().label_w(ui);
            } else {
                for action_title in action_list {
                    action_title.label_w(ui);
                }
            }
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
        let response = TeamEditor::render_unit_with_representation_static(ui, unit_id, team, ctx);

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

        if let Some(unit) = TeamEditor::find_unit_in_team_static(team, unit_id) {
            response.clone().on_hover_text(unit.unit_name.clone());
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
                    "Swap units".to_string()
                } else {
                    "Same unit".to_string()
                }
            })
            .ui(ui)
        {
            if dropped_unit.unit_id != unit_id {
                match dropped_unit.from_location {
                    UnitTarget::Slot {
                        fusion_id: from_fusion,
                        slot_index: from_slot,
                    } => {
                        actions.push(TeamAction::MoveUnit {
                            unit_id: dropped_unit.unit_id,
                            target: UnitTarget::Slot {
                                fusion_id,
                                slot_index,
                            },
                        });
                        actions.push(TeamAction::MoveUnit {
                            unit_id,
                            target: UnitTarget::Slot {
                                fusion_id: from_fusion,
                                slot_index: from_slot,
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
        let response = TeamEditor::render_empty_slot_static(ui, ctx);

        if let Some(dropped_unit) = DndArea::<DraggedUnit>::new(response.rect)
            .id(format!("empty_slot_{}_{}", fusion_id, slot_index))
            .text_fn(ui, |_| "Place unit".to_string())
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

    fn render_bench_horizontal(
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
                    if let Some(unit) = TeamEditor::find_unit_in_team_static(team, dragged.unit_id)
                    {
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
        actions: &mut Vec<TeamAction>,
    ) {
        let response = TeamEditor::render_unit_with_representation_static(ui, unit_id, team, ctx);

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

        if let Some(unit) = TeamEditor::find_unit_in_team_static(team, unit_id) {
            response.clone().on_hover_text(unit.unit_name.clone());
        }
    }

    fn render_unit_with_representation_static(
        ui: &mut Ui,
        unit_id: u64,
        team: &NTeam,
        ctx: &ClientContext,
    ) -> Response {
        let is_inspected = ui
            .inspected_node_for_parent(team.id)
            .is_some_and(|id| id == unit_id);
        if let Some(unit) = TeamEditor::find_unit_in_team_static(team, unit_id) {
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
        if let Some(unit) = TeamEditor::find_unit_in_team_static(team, unit_id) {
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

    fn render_empty_slot_static(ui: &mut Ui, ctx: &ClientContext) -> Response {
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
                    if let Some(unit) = Self::find_unit_in_team_static(team, unit_id) {
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

    fn find_unit_in_team_static(team: &NTeam, unit_id: u64) -> Option<NUnit> {
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
                if fusion.trigger_unit != 0 && !available_units.contains(&fusion.trigger_unit) {
                    fusion.trigger_unit = 0;
                    fusion.set_dirty(true);
                }
                if fusion.trigger_unit == 0 {
                    if let Ok(slots) = fusion.slots.get() {
                        for slot in slots {
                            if let Some(unit_id) = slot.unit.id() {
                                if available_units.contains(&unit_id) {
                                    fusion.trigger_unit = unit_id;
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
                match self.find_unit_slot_mut(unit_id) {
                    Ok(from_slot) => {
                        from_slot.unit = Ref::none();
                        from_slot.set_dirty(true);
                    }
                    Err(e) => e.log(),
                }

                match target {
                    UnitTarget::Slot {
                        fusion_id,
                        slot_index,
                    } => {
                        let slot = self.fusion_slot_mut(fusion_id, slot_index)?;
                        slot.unit = Ref::new_id(unit_id);
                        slot.set_dirty(true);
                    }
                    UnitTarget::Bench => {}
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
                        fusion.trigger_unit = trigger;
                    }
                }
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
