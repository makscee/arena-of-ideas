use bevy_egui::egui::{DragAndDrop, UiKind};

use crate::prelude::*;
use crate::ui::{DndArea, RangeSelector};

#[derive(Debug, Clone)]
pub struct DraggedUnit {
    pub unit_id: u64,
    pub from_location: UnitTarget,
    pub drag_start_pos: Option<Pos2>,
}

pub struct TeamEditor {
    empty_slot_actions: Vec<(String, Box<dyn Fn(&NTeam, u64, i32)>)>,
    filled_slot_actions: Vec<(String, Box<dyn Fn(&NTeam, u64, u64, i32)>)>,
}

#[derive(Debug, Clone)]
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
        }
    }

    pub fn empty_slot_action(
        mut self,
        name: String,
        action: Box<dyn Fn(&NTeam, u64, i32)>,
    ) -> Self {
        self.empty_slot_actions.push((name, action));
        self
    }

    pub fn filled_slot_action(
        mut self,
        name: String,
        action: Box<dyn Fn(&NTeam, u64, u64, i32)>,
    ) -> Self {
        self.filled_slot_actions.push((name, action));
        self
    }

    pub fn edit(
        &self,
        team: &NTeam,
        context: &ClientContext,
        ui: &mut Ui,
    ) -> (Option<NTeam>, Vec<TeamAction>) {
        let mut actions = Vec::new();

        // Handle drag arrow visualization
        if DragAndDrop::has_payload_of_type::<DraggedUnit>(ui.ctx()) {
            if let Some(pointer_pos) = ui.ctx().pointer_latest_pos() {
                if let Some(payload) = DragAndDrop::payload::<DraggedUnit>(ui.ctx()) {
                    if let Some(start_pos) = payload.drag_start_pos {
                        self.draw_drag_arrow(ui, start_pos, pointer_pos);
                    }
                }
            }
        }

        let state_id = egui::Id::new(team.id).with("team_editor_selected_fusion");
        let mut selected_fusion_id =
            ui.memory(|m| m.data.get_temp::<Option<u64>>(state_id).unwrap_or(None));

        let fusions = if let Some(fusions) = team.fusions.get() {
            fusions.iter().sorted_by_key(|f| f.index).collect_vec()
        } else {
            vec![]
        };

        let unlinked_units = self.get_unlinked_units(team);
        let total_columns = fusions.len() + 1;

        let mut selected_column_rect = None;
        let mut clicked_fusion_id = None;

        ui.columns(total_columns, |columns| {
            let mut column_idx = 0;

            for fusion in &fusions {
                let slots = if let Some(slots) = fusion.slots.get() {
                    slots.iter().sorted_by_key(|s| s.index).collect_vec()
                } else {
                    vec![]
                };

                if selected_fusion_id == Some(fusion.id) {
                    self.render_fusion_actions_column(
                        &mut columns[column_idx],
                        fusion,
                        &slots,
                        context,
                        &mut actions,
                    );
                } else {
                    self.render_fusion_column_with_actions_display(
                        &mut columns[column_idx],
                        fusion,
                        &slots,
                        team,
                        context,
                        &mut actions,
                    );
                }

                if columns[column_idx].ctx().dragged_id().is_some() {
                    selected_column_rect = Some(columns[column_idx].min_rect());
                }

                if columns[column_idx].response().clicked() {
                    clicked_fusion_id = Some(fusion.id);
                }

                column_idx += 1;
            }

            self.render_bench_column(
                &mut columns[column_idx],
                &unlinked_units,
                team,
                context,
                &mut actions,
            );
        });

        if let Some(id) = clicked_fusion_id {
            selected_fusion_id = Some(id);
        }

        ui.memory_mut(|m| m.data.insert_temp(state_id, selected_fusion_id));

        if !actions.is_empty() {
            let mut result_team = team.clone();
            for action in &actions {
                result_team.apply_action(action.clone()).log();
            }
            (Some(result_team), actions)
        } else {
            (None, actions)
        }
    }

    fn get_unlinked_units(&self, team: &NTeam) -> Vec<u64> {
        let mut linked_units = HashSet::new();

        if let Some(fusions) = team.fusions.get() {
            for fusion in fusions {
                if let Some(slots) = fusion.slots.get() {
                    for slot in slots {
                        if let Some(unit_id) = slot.unit.id() {
                            linked_units.insert(unit_id);
                        }
                    }
                }
            }
        }

        let mut unlinked = vec![];
        if let Some(houses) = team.houses.get() {
            for house in houses {
                if let Some(units) = house.units.get() {
                    for unit in units {
                        if !linked_units.contains(&unit.id) {
                            unlinked.push(unit.id);
                        }
                    }
                }
            }
        }

        unlinked
    }

    fn get_available_triggers(&self, fusion: &NFusion) -> Vec<(u64, String)> {
        let mut triggers = vec![];

        if let Some(slots) = fusion.slots.get() {
            for slot in slots {
                if let Some(unit_id) = slot.unit.id() {
                    triggers.push((unit_id, format!("Unit {}", slot.index + 1)));
                }
            }
        }

        triggers
    }

    fn render_bench_column(
        &self,
        ui: &mut Ui,
        unlinked_units: &[u64],
        team: &NTeam,
        context: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        ui.vertical(|ui| {
            ui.label("Bench");
            ui.separator();

            for &unit_id in unlinked_units {
                self.handle_bench_unit_interactions(ui, unit_id, team, context, actions);
            }

            // Add bench drop area
            let available_space = ui.available_rect_before_wrap();
            if let Some(dropped_unit) = DndArea::<DraggedUnit>::new(available_space)
                .id("bench_drop")
                .text_fn(ui, |dragged| {
                    if let Some(unit) = self.find_unit_in_team(team, dragged.unit_id) {
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
        context: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        let response = self.render_unit_with_representation(ui, unit_id, team, context);

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

        if let Some(unit) = self.find_unit_in_team(team, unit_id) {
            response.clone().on_hover_text(unit.unit_name.clone());
        }

        // Handle dropping on this bench unit
        if let Some(dropped_unit) = DndArea::<DraggedUnit>::new(response.rect)
            .id(format!("bench_unit_{}", unit_id))
            .text_fn(ui, |dragged| {
                if dragged.unit_id != unit_id {
                    format!("Swap positions")
                } else {
                    "Same unit".to_string()
                }
            })
            .ui(ui)
        {
            if dropped_unit.unit_id != unit_id {
                match dropped_unit.from_location {
                    UnitTarget::Slot {
                        fusion_id,
                        slot_index,
                    } => {
                        // Swap: move bench unit to slot, bench the dragged unit
                        actions.push(TeamAction::MoveUnit {
                            unit_id: unit_id,
                            target: UnitTarget::Slot {
                                fusion_id,
                                slot_index,
                            },
                        });
                        actions.push(TeamAction::BenchUnit {
                            unit_id: dropped_unit.unit_id,
                        });
                    }
                    UnitTarget::Bench => {
                        // Both on bench - no action needed
                    }
                }
            }
        }
    }

    fn render_fusion_column_with_actions_display(
        &self,
        ui: &mut Ui,
        fusion: &NFusion,
        slots: &[&NFusionSlot],
        team: &NTeam,
        context: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        ui.vertical(|ui| {
            ui.label(format!("Fusion {}", fusion.index + 1));
            ui.separator();

            self.render_fusion_action_sequence(ui, fusion, slots, context);
            ui.separator();

            for slot in slots {
                self.render_slot(ui, slot, fusion.id, team, context, actions);
            }

            if ui.button("+ Add Slot").clicked() {
                actions.push(TeamAction::AddSlot {
                    fusion_id: fusion.id,
                });
            }
        });
    }

    fn render_fusion_action_sequence(
        &self,
        ui: &mut Ui,
        fusion: &NFusion,
        slots: &[&NFusionSlot],
        ctx: &ClientContext,
    ) {
        ui.horizontal(|ui| {
            for slot in slots {
                let range = &slot.actions;
                let is_trigger = slot.unit.id().unwrap_or(0) == fusion.trigger_unit;

                if is_trigger {
                    ui.label("🎯");
                }

                if let Some(unit_id) = slot.unit.id() {
                    if let Ok(unit) = ctx.load::<NUnit>(unit_id) {
                        if let Ok(unit_desc) = unit.description_ref(ctx) {
                            if let Ok(unit_behavior) = unit_desc.behavior_ref(ctx) {
                                let total_actions = unit_behavior.reaction.actions.len() as u8;
                                let end_range = (range.start + range.length).min(total_actions);

                                for i in range.start..end_range {
                                    if let Some(action) =
                                        unit_behavior.reaction.actions.get(i as usize)
                                    {
                                        self.render_action_normal(ui, ctx, action);
                                    } else {
                                        self.render_action_greyed(ui, ctx, &Action::noop);
                                    }
                                }
                            }
                        }
                    }
                }

                ui.add_space(4.0);
            }
        });
    }

    fn render_slot(
        &self,
        ui: &mut Ui,
        slot: &NFusionSlot,
        fusion_id: u64,
        team: &NTeam,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        let state_id = egui::Id::new(team.id).with("team_editor_selected_fusion");
        let selected_fusion_id =
            ui.memory(|m| m.data.get_temp::<Option<u64>>(state_id).unwrap_or(None));

        let is_fusion_editing = selected_fusion_id == Some(fusion_id);

        if is_fusion_editing && slot.unit.id().is_some() {
            if let Some(unit_id) = slot.unit.id() {
                if let Ok(unit) = ctx.load::<NUnit>(unit_id) {
                    if let Ok(unit_desc) = unit.description_ref(ctx) {
                        if let Ok(unit_behavior) = unit_desc.behavior_ref(ctx) {
                            let total_actions = unit_behavior.reaction.actions.len() as u8;

                            if total_actions > 0 {
                                ui.vertical(|ui| {
                                    ui.label(format!("Slot {} Actions", slot.index + 1));

                                    let range_selector = RangeSelector::new(total_actions)
                                        .range(slot.actions.start, slot.actions.length)
                                        .id(egui::Id::new(slot.id).with("slot_range_selector"))
                                        .border_thickness(3.0)
                                        .drag_threshold(10.0);

                                    let (_, range_change) = range_selector.ui(
                                        ui,
                                        ctx,
                                        |ui, _, action_index, is_in_range| {
                                            if let Some(action) =
                                                unit_behavior.reaction.actions.get(action_index)
                                            {
                                                if is_in_range {
                                                    self.render_action_normal(ui, ctx, action);
                                                } else {
                                                    self.render_action_greyed(ui, ctx, action);
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
                                });
                                return;
                            }
                        }
                    }
                }
            }
        }

        ui.horizontal(|ui| {
            let slot_response = slot_rect_button(60.0.v2(), ui, |_, ui| {
                if slot.unit.id().is_some() {
                    if let Some(unit_id) = slot.unit.id() {
                        self.handle_unit_interactions(
                            ui, unit_id, fusion_id, slot.index, team, ctx, actions,
                        );
                    }
                } else {
                    self.handle_empty_slot_interactions(ui, fusion_id, slot.index, team, actions);
                }
            });

            // Handle empty slot drop target outside of the slot_rect_button
            if slot.unit.id().is_none() {
                if let Some(dropped_unit) = DndArea::<DraggedUnit>::new(slot_response.rect)
                    .id(format!("empty_slot_{}_{}", fusion_id, slot.index))
                    .text_fn(ui, |dragged| {
                        if let Some(unit) = self.find_unit_in_team(team, dragged.unit_id) {
                            format!("Place {}", unit.unit_name)
                        } else {
                            "Place Unit".to_string()
                        }
                    })
                    .ui(ui)
                {
                    actions.push(TeamAction::MoveUnit {
                        unit_id: dropped_unit.unit_id,
                        target: UnitTarget::Slot {
                            fusion_id,
                            slot_index: slot.index,
                        },
                    });
                }
            }
        });
    }

    fn render_unit_with_representation(
        &self,
        ui: &mut Ui,
        unit_id: u64,
        team: &NTeam,
        context: &ClientContext,
    ) -> Response {
        if let Some(unit) = self.find_unit_in_team(team, unit_id) {
            if let Ok(desc) = unit.description_ref(context) {
                if let Ok(rep) = desc.representation_ref(context) {
                    MatRect::new(egui::Vec2::new(60.0, 60.0))
                        .add_mat(&rep.material, unit.id)
                        .unit_rep_with_default(unit.id)
                        .ui(ui, context)
                } else {
                    MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, context)
                }
            } else {
                MatRect::new(egui::Vec2::new(60.0, 60.0)).ui(ui, context)
            }
        } else {
            ui.label("[Unknown Unit]")
        }
    }

    fn find_unit_in_team<'a>(&self, team: &'a NTeam, unit_id: u64) -> Option<&'a NUnit> {
        if let Some(houses) = team.houses.get() {
            for house in houses {
                if let Some(units) = house.units.get() {
                    for unit in units {
                        if unit.id == unit_id {
                            return Some(unit);
                        }
                    }
                }
            }
        }
        None
    }

    fn render_empty_slot(&self, ui: &mut Ui) -> Response {
        ui.label("[Empty]")
    }

    fn handle_unit_interactions(
        &self,
        ui: &mut Ui,
        unit_id: u64,
        fusion_id: u64,
        slot_index: i32,
        team: &NTeam,
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        let response = self.render_unit_with_representation(ui, unit_id, team, ctx);

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

        if let Some(unit) = self.find_unit_in_team(team, unit_id) {
            response.clone().on_hover_text(&unit.unit_name);
        }

        // Handle dropping on this slot unit
        if let Some(dropped_unit) = DndArea::<DraggedUnit>::new(response.rect)
            .id(format!("slot_unit_{}_{}", fusion_id, slot_index))
            .text_fn(ui, |dragged| {
                if dragged.unit_id != unit_id {
                    format!("Swap positions")
                } else {
                    "Same unit".to_string()
                }
            })
            .ui(ui)
        {
            if dropped_unit.unit_id != unit_id {
                match dropped_unit.from_location {
                    UnitTarget::Bench => {
                        // Move bench unit to this slot, current unit goes to bench
                        actions.push(TeamAction::MoveUnit {
                            unit_id: dropped_unit.unit_id,
                            target: UnitTarget::Slot {
                                fusion_id,
                                slot_index,
                            },
                        });
                        actions.push(TeamAction::BenchUnit { unit_id });
                    }
                    UnitTarget::Slot {
                        fusion_id: from_fusion_id,
                        slot_index: from_slot_index,
                    } => {
                        // Swap positions
                        actions.push(TeamAction::MoveUnit {
                            unit_id: dropped_unit.unit_id,
                            target: UnitTarget::Slot {
                                fusion_id,
                                slot_index,
                            },
                        });
                        actions.push(TeamAction::MoveUnit {
                            unit_id: unit_id,
                            target: UnitTarget::Slot {
                                fusion_id: from_fusion_id,
                                slot_index: from_slot_index,
                            },
                        });
                    }
                }
            }
        }

        response.clone().context_menu(|ui| {
            if ui.button("Bench").clicked() {
                actions.push(TeamAction::BenchUnit { unit_id });
                ui.close_kind(UiKind::Menu);
            }

            for (name, action) in &self.filled_slot_actions {
                if ui.button(name).clicked() {
                    action(team, fusion_id, unit_id, slot_index);
                    ui.close_kind(UiKind::Menu);
                }
            }
        });
    }

    fn handle_empty_slot_interactions(
        &self,
        ui: &mut Ui,
        fusion_id: u64,
        slot_index: i32,
        team: &NTeam,
        actions: &mut Vec<TeamAction>,
    ) {
        let response = self.render_empty_slot(ui);

        response.context_menu(|ui| {
            for (name, action) in &self.empty_slot_actions {
                if ui.button(name).clicked() {
                    action(team, fusion_id, slot_index);
                    ui.close_kind(UiKind::Menu);
                }
            }
        });
    }

    fn draw_drag_arrow(&self, ui: &mut Ui, start_pos: Pos2, pointer_pos: Pos2) {
        let arrow_color = Color32::from_rgb(255, 200, 0);
        let stroke = Stroke::new(3.0, arrow_color);

        ui.painter().line_segment([start_pos, pointer_pos], stroke);

        // Draw arrow head
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

    fn render_fusion_actions_column(
        &self,
        ui: &mut Ui,
        fusion: &NFusion,
        slots: &[&NFusionSlot],
        ctx: &ClientContext,
        actions: &mut Vec<TeamAction>,
    ) {
        ui.vertical(|ui| {
            ui.label(format!("Fusion {} Actions", fusion.index + 1));
            ui.separator();

            let triggers = self.get_available_triggers(fusion);
            let mut trigger_index = triggers
                .iter()
                .position(|(id, _)| *id == fusion.trigger_unit)
                .unwrap_or(0);

            ui.horizontal(|ui| {
                ui.label("Trigger:");
                egui::ComboBox::from_id_salt(fusion.id)
                    .selected_text(&triggers[trigger_index].1)
                    .show_ui(ui, |ui| {
                        for (i, (id, name)) in triggers.iter().enumerate() {
                            if ui.selectable_value(&mut trigger_index, i, name).clicked() {
                                actions.push(TeamAction::ChangeTrigger {
                                    fusion_id: fusion.id,
                                    trigger: *id,
                                });
                            }
                        }
                    });
            });

            ui.separator();

            for slot in slots {
                ui.group(|ui| {
                    ui.vertical(|ui| {
                        ui.label(format!(
                            "Slot {} - {}",
                            slot.index + 1,
                            slot.unit.id().unwrap_or_default()
                        ));

                        if let Some(unit_id) = slot.unit.id() {
                            if let Ok(unit) = ctx.load::<NUnit>(unit_id) {
                                if let Ok(unit_desc) = unit.description_ref(ctx) {
                                    if let Ok(unit_behavior) = unit_desc.behavior_ref(ctx) {
                                        let total_actions =
                                            unit_behavior.reaction.actions.len() as u8;

                                        if total_actions > 0 {
                                            let range_selector = RangeSelector::new(total_actions)
                                                .range(slot.actions.start, slot.actions.length)
                                                .id(egui::Id::new(slot.id).with("range_selector"))
                                                .border_thickness(3.0)
                                                .drag_threshold(10.0);

                                            let (_, range_change) = range_selector.ui(
                                                ui,
                                                ctx,
                                                |ui, _, action_index, is_in_range| {
                                                    if let Some(action) = unit_behavior
                                                        .reaction
                                                        .actions
                                                        .get(action_index)
                                                    {
                                                        if is_in_range {
                                                            self.render_action_normal(
                                                                ui, ctx, action,
                                                            );
                                                        } else {
                                                            self.render_action_greyed(
                                                                ui, ctx, action,
                                                            );
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
        });
    }

    fn render_action_normal(&self, ui: &mut Ui, context: &ClientContext, action: &Action) {
        action.title(context).label(ui);
    }

    fn render_action_greyed(&self, ui: &mut Ui, context: &ClientContext, action: &Action) {
        action
            .title(context)
            .cstr_c(ui.visuals().weak_text_color())
            .label(ui);
    }
}

impl NTeam {
    pub fn apply_action(&mut self, action: TeamAction) -> NodeResult<()> {
        match action {
            TeamAction::MoveUnit { unit_id, target } => {
                self.find_unit_slot_mut(unit_id).track()?.unit = Ref::none();

                if let UnitTarget::Slot {
                    fusion_id,
                    slot_index,
                } = target
                {
                    let unit = self.find_unit(unit_id).track()?;
                    let slot = self.fusion_slot_mut(fusion_id, slot_index).track()?;
                    slot.unit_set(unit);
                }
            }
            TeamAction::BenchUnit { unit_id } => {
                if let Ok(fusions) = self.fusions.get_mut() {
                    for fusion in fusions.iter_mut() {
                        if let Ok(slots) = fusion.slots.get_mut() {
                            slots.retain(|slot| slot.unit.id().unwrap_or(0) != unit_id);
                        }
                    }
                }
            }
            TeamAction::AddSlot { fusion_id } => {
                if let Ok(fusions) = self.fusions.get_mut() {
                    if let Some(fusion) = fusions.iter_mut().find(|f| f.id == fusion_id) {
                        let index = fusion.slots().map(|s| s.len() as i32).unwrap_or(0);
                        fusion.slots_push(NFusionSlot::new(
                            next_id(),
                            index,
                            UnitActionRange::default(),
                        ))?;
                    }
                }
            }
            TeamAction::ChangeActionRange {
                slot_id,
                start,
                length,
            } => {
                if let Ok(fusions) = self.fusions.get_mut() {
                    for fusion in fusions.iter_mut() {
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
        }
        Ok(())
    }

    fn find_unit(&self, unit_id: u64) -> NodeResult<NUnit> {
        if let Some(houses) = self.houses.get() {
            for house in houses {
                if let Some(units) = house.units.get() {
                    for unit in units {
                        if unit.id == unit_id {
                            return Ok(unit.clone());
                        }
                    }
                }
            }
        }
        Err(NodeError::custom("Unit not found"))
    }

    fn find_unit_slot_mut(&mut self, unit_id: u64) -> NodeResult<&mut NFusionSlot> {
        for fusion in self.fusions.get_mut().track()? {
            for slot in fusion.slots.get_mut().track()? {
                if slot.unit.id().unwrap_or_default() == unit_id {
                    return Ok(slot);
                }
            }
        }
        Err(NodeError::custom("Unit slot not found"))
    }

    fn fusion_slot_mut(&mut self, id: u64, index: i32) -> NodeResult<&mut NFusionSlot> {
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
