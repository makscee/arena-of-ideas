use crate::prelude::*;

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

    pub fn edit(&self, team: &NTeam, ui: &mut Ui) -> (Option<NTeam>, Vec<TeamAction>) {
        let mut actions = Vec::new();

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
                        &mut actions,
                    );
                } else {
                    self.render_fusion_column_with_actions_display(
                        &mut columns[column_idx],
                        fusion,
                        &slots,
                        team,
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

            self.render_bench_column(&mut columns[column_idx], &unlinked_units, team);
        });

        if let Some(id) = clicked_fusion_id {
            selected_fusion_id = Some(id);
        }

        ui.memory_mut(|m| m.data.insert_temp(state_id, selected_fusion_id));

        if !actions.is_empty() {
            let mut result_team = team.clone();
            for action in &actions {
                let _ = result_team.apply_action(action.clone());
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

    fn render_bench_column(&self, ui: &mut Ui, unlinked_units: &[u64], team: &NTeam) {
        ui.vertical(|ui| {
            ui.label("Bench");
            ui.separator();

            for &unit_id in unlinked_units {
                self.handle_bench_unit_interactions(ui, unit_id, team);
            }
        });
    }

    fn handle_bench_unit_interactions(&self, ui: &mut Ui, unit_id: u64, team: &NTeam) {
        let response = self.render_unit_with_representation(ui, unit_id, team);

        if response.drag_started() {
            ui.memory_mut(|mem| {
                mem.data
                    .insert_temp(egui::Id::new("dragging_unit"), unit_id)
            });
        }

        if let Some(unit) = self.find_unit_in_team(team, unit_id) {
            response.on_hover_text(unit.unit_name.clone());
        }
    }

    fn render_fusion_column_with_actions_display(
        &self,
        ui: &mut Ui,
        fusion: &NFusion,
        slots: &[&NFusionSlot],
        team: &NTeam,
        actions: &mut Vec<TeamAction>,
    ) {
        ui.vertical(|ui| {
            ui.label(format!("Fusion {}", fusion.index + 1));
            ui.separator();

            self.render_fusion_action_sequence(ui, fusion, slots);
            ui.separator();

            for slot in slots {
                self.render_slot(ui, slot, fusion.id, team, actions);
            }

            if ui.button("+ Add Slot").clicked() {
                actions.push(TeamAction::AddSlot {
                    fusion_id: fusion.id,
                });
            }
        });
    }

    fn render_fusion_action_sequence(&self, ui: &mut Ui, fusion: &NFusion, slots: &[&NFusionSlot]) {
        ui.horizontal(|ui| {
            for slot in slots {
                let range = &slot.actions;
                let is_trigger = slot.unit.id().unwrap_or(0) == fusion.trigger_unit;

                if is_trigger {
                    ui.label("🎯");
                }

                let end_range = range.start + range.length;
                for i in range.start..end_range {
                    if i == 0 {
                        self.render_action_normal(ui, i as i32);
                    } else {
                        self.render_action_greyed(ui, i as i32);
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
        actions: &mut Vec<TeamAction>,
    ) {
        ui.horizontal(|ui| {
            let slot_response = slot_rect_button(60.0.v2(), ui, |_, ui| {
                if slot.unit.id().is_some() {
                    if let Some(unit_id) = slot.unit.id() {
                        self.handle_unit_interactions(
                            ui, unit_id, fusion_id, slot.index, team, actions,
                        );
                    }
                } else {
                    self.handle_empty_slot_interactions(ui, fusion_id, slot.index, team, actions);
                }
            });
            let rect = slot_response.rect;
            if ui.ctx().dragged_id().is_some() {
                if let Some(dragging_unit) =
                    ui.memory(|mem| mem.data.get_temp::<u64>(egui::Id::new("dragging_unit")))
                {
                    if ui.rect_contains_pointer(rect) {
                        ui.painter().rect_stroke(
                            rect,
                            0.0,
                            Stroke::new(2.0, Color32::from_rgb(100, 200, 100)),
                            egui::epaint::StrokeKind::Outside,
                        );

                        if ui.input(|i| i.pointer.any_released()) {
                            actions.push(TeamAction::MoveUnit {
                                unit_id: dragging_unit,
                                target: UnitTarget::Slot {
                                    fusion_id,
                                    slot_index: slot.index,
                                },
                            });
                        }
                    }
                }
            }
        });
    }

    fn render_unit_with_representation(&self, ui: &mut Ui, unit_id: u64, team: &NTeam) -> Response {
        if let Some(unit) = self.find_unit_in_team(team, unit_id) {
            ui.label(&unit.unit_name)
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
        actions: &mut Vec<TeamAction>,
    ) {
        let response = self.render_unit_with_representation(ui, unit_id, team);

        if response.drag_started() {
            ui.memory_mut(|mem| {
                mem.data
                    .insert_temp(egui::Id::new("dragging_unit"), unit_id)
            });
        }

        if let Some(unit) = self.find_unit_in_team(team, unit_id) {
            response.clone().on_hover_text(&unit.unit_name);
        }

        response.context_menu(|ui| {
            if ui.button("Bench").clicked() {
                actions.push(TeamAction::BenchUnit { unit_id });
                ui.close_menu();
            }

            for (name, action) in &self.filled_slot_actions {
                if ui.button(name).clicked() {
                    action(team, fusion_id, unit_id, slot_index);
                    ui.close_menu();
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
        _actions: &mut Vec<TeamAction>,
    ) {
        let response = self.render_empty_slot(ui);

        response.context_menu(|ui| {
            for (name, action) in &self.empty_slot_actions {
                if ui.button(name).clicked() {
                    action(team, fusion_id, slot_index);
                    ui.close_menu();
                }
            }
        });
    }

    fn render_fusion_actions_column(
        &self,
        ui: &mut Ui,
        fusion: &NFusion,
        slots: &[&NFusionSlot],
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

                        let mut start = slot.actions.start as i32;
                        let mut length = slot.actions.length as i32;

                        ui.horizontal(|ui| {
                            ui.label("Start:");
                            if ui.add(egui::DragValue::new(&mut start).speed(1)).changed() {
                                actions.push(TeamAction::ChangeActionRange {
                                    slot_id: slot.id,
                                    start,
                                    length,
                                });
                            }
                        });

                        ui.horizontal(|ui| {
                            ui.label("Length:");
                            if ui
                                .add(egui::DragValue::new(&mut length).speed(1).range(1..=10))
                                .changed()
                            {
                                actions.push(TeamAction::ChangeActionRange {
                                    slot_id: slot.id,
                                    start,
                                    length,
                                });
                            }
                        });

                        ui.horizontal(|ui| {
                            for i in start..start + length {
                                if i == 0 {
                                    self.render_action_normal(ui, i);
                                } else {
                                    self.render_action_greyed(ui, i);
                                }
                            }
                        });
                    });
                });
            }
        });
    }

    fn render_action_normal(&self, ui: &mut Ui, index: i32) {
        ui.label(format!("A{}", index));
    }

    fn render_action_greyed(&self, ui: &mut Ui, index: i32) {
        ui.add(egui::Label::new(
            egui::RichText::new(format!("A{}", index)).color(Color32::from_gray(128)),
        ));
    }
}

impl NTeam {
    pub fn apply_action(&mut self, action: TeamAction) -> NodeResult<()> {
        match action {
            TeamAction::MoveUnit { unit_id, target } => {
                if let Ok(fusions) = self.fusions.get_mut() {
                    for fusion in fusions.iter_mut() {
                        if let Ok(slots) = fusion.slots.get_mut() {
                            slots.retain(|slot| slot.unit.id().unwrap_or(0) != unit_id);
                        }
                    }
                }

                if let UnitTarget::Slot {
                    fusion_id,
                    slot_index,
                } = target
                {
                    if let Ok(fusions) = self.fusions.get_mut() {
                        if let Some(fusion) = fusions.iter_mut().find(|f| f.id == fusion_id) {
                            // Find unit before mutably borrowing fusion
                            let mut found_unit = None;
                            if let Some(houses) = self.houses.get() {
                                for house in houses {
                                    if let Some(units) = house.units.get() {
                                        if let Some(unit) = units.iter().find(|u| u.id == unit_id) {
                                            found_unit = Some(unit.clone());
                                            break;
                                        }
                                    }
                                }
                            }

                            if let Some(unit) = found_unit {
                                let new_slot = NFusionSlot::new(
                                    next_id(),
                                    slot_index,
                                    UnitActionRange::default(),
                                )
                                .with_unit(unit);

                                if let Ok(slots) = fusion.slots.get_mut() {
                                    slots.push(new_slot);
                                    slots.sort_by_key(|s| s.index);
                                }
                            }
                        }
                    }
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
                        fusion.slots_push(NFusionSlot::new(
                            next_id(),
                            fusion.slots()?.len() as i32,
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
}
