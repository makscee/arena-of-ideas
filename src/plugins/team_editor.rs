use super::*;

pub struct TeamEditor {
    team_entity: Entity,
    bench_entity: Option<Entity>,
    empty_slot_actions: Vec<String>,
    filled_slot_actions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TeamAction {
    MoveUnit {
        unit_id: u64,
        target: u64,
    },
    ContextMenuAction {
        slot_id: u64,
        action_name: String,
        unit_id: Option<u64>,
    },
    AddSlot {
        fusion_id: u64,
    },
}

impl TeamEditor {
    pub fn new(team_entity: Entity) -> Self {
        Self {
            team_entity,
            bench_entity: None,
            empty_slot_actions: Vec::new(),
            filled_slot_actions: Vec::new(),
        }
    }

    pub fn with_bench(mut self, bench_entity: Entity) -> Self {
        self.bench_entity = Some(bench_entity);
        self
    }

    pub fn empty_slot_action(mut self, action_name: &str) -> Self {
        self.empty_slot_actions.push(action_name.to_string());
        self
    }

    pub fn filled_slot_action(mut self, action_name: &str) -> Self {
        self.filled_slot_actions.push(action_name.to_string());
        self
    }

    pub fn ui(self, ui: &mut Ui, context: &Context) -> Result<Vec<TeamAction>, ExpressionError> {
        let team = context.get::<NTeam>(self.team_entity)?;
        let mut actions = Vec::new();

        let bench_slots = if let Some(bench_entity) = self.bench_entity {
            let mut slots =
                context.collect_children_components::<NBenchSlot>(context.id(bench_entity)?)?;
            slots.sort_by_key(|s| s.index);
            Some(slots)
        } else {
            None
        };

        let fusions = team
            .fusions_load(context)
            .into_iter()
            .sorted_by_key(|f| f.index)
            .collect_vec();

        let mut fusion_slots: HashMap<u64, Vec<&NFusionSlot>> = HashMap::new();
        for fusion in &fusions {
            let slots = context
                .collect_parents_components::<NFusionSlot>(fusion.id)?
                .into_iter()
                .sorted_by_key(|s| s.index)
                .collect_vec();
            fusion_slots.insert(fusion.id, slots);
        }

        let bench_column_count = if bench_slots.is_some() { 1 } else { 0 };
        let total_columns = bench_column_count + fusions.len();

        if total_columns == 0 {
            return Ok(actions);
        }

        ui.columns(total_columns, |columns| {
            let mut column_idx = 0;

            if let Some(ref bench_slots) = bench_slots {
                self.render_bench_column(
                    &mut columns[column_idx],
                    bench_slots,
                    context,
                    &mut actions,
                );
                column_idx += 1;
            }

            for fusion in &fusions {
                let empty_slots = vec![];
                let slots = fusion_slots.get(&fusion.id).unwrap_or(&empty_slots);
                self.render_fusion_column(
                    &mut columns[column_idx],
                    fusion,
                    slots,
                    context,
                    &mut actions,
                );
                column_idx += 1;
            }
        });

        Ok(actions)
    }

    fn render_bench_column(
        &self,
        ui: &mut Ui,
        bench_slots: &[&NBenchSlot],
        context: &Context,
        actions: &mut Vec<TeamAction>,
    ) {
        ui.vertical(|ui| {
            ui.label("Bench");
            ui.separator();

            for slot in bench_slots {
                self.render_slot(ui, slot.id, context, actions);
            }
        });
    }

    fn render_fusion_column(
        &self,
        ui: &mut Ui,
        fusion: &NFusion,
        slots: &[&NFusionSlot],
        context: &Context,
        actions: &mut Vec<TeamAction>,
    ) {
        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Fusion {}", fusion.index));
                if ui.button("Add Slot").clicked() {
                    actions.push(TeamAction::AddSlot {
                        fusion_id: fusion.id,
                    });
                }
            });
            ui.separator();

            for slot in slots {
                self.render_slot(ui, slot.id, context, actions);
            }
        });
    }

    fn render_slot(
        &self,
        ui: &mut Ui,
        slot_id: u64,
        context: &Context,
        actions: &mut Vec<TeamAction>,
    ) {
        let unit = Self::get_slot_unit(slot_id, context);
        let resp = Self::render_unit_in_slot(ui, unit, context);

        if unit.is_some() {
            self.handle_unit_interactions(resp, slot_id, unit, actions);
        } else {
            self.handle_empty_slot_interactions(resp, slot_id, actions);
        }
    }

    fn get_slot_unit<'b>(slot_id: u64, context: &'b Context) -> Option<&'b NUnit> {
        context.first_parent::<NUnit>(slot_id).ok()
    }

    fn render_unit_in_slot(ui: &mut Ui, unit: Option<&NUnit>, context: &Context) -> Response {
        let size = egui::Vec2::new(60.0, 60.0);

        if let Some(unit) = unit {
            Self::render_unit_with_representation(ui, unit, size, context)
        } else {
            Self::render_empty_slot(ui, size, context)
        }
    }

    fn render_unit_with_representation(
        ui: &mut Ui,
        unit: &NUnit,
        size: egui::Vec2,
        context: &Context,
    ) -> Response {
        let mut mat_rect = MatRect::new(size);

        if let Ok(rep) = context.first_parent_recursive::<NUnitRepresentation>(unit.id) {
            mat_rect = mat_rect.add_mat(&rep.material, unit.id);
        }

        let resp = mat_rect.unit_rep_with_default(unit.id).ui(ui, context);

        if resp.dragged() {
            if let Some(pos) = ui.ctx().pointer_latest_pos() {
                let origin = resp.rect.center();
                ui.painter()
                    .arrow(origin, pos - origin, ui.visuals().widgets.hovered.fg_stroke);
            }
        }

        resp.dnd_set_drag_payload(unit.id);
        resp
    }

    fn render_empty_slot(ui: &mut Ui, size: egui::Vec2, context: &Context) -> Response {
        MatRect::new(size).ui(ui, context)
    }

    fn handle_unit_interactions(
        &self,
        resp: Response,
        target: u64,
        current_unit: Option<&NUnit>,
        actions: &mut Vec<TeamAction>,
    ) {
        if let Some(dragged_unit_id) = resp.dnd_release_payload::<u64>() {
            if let Some(unit) = current_unit {
                if unit.id == *dragged_unit_id {
                    return;
                }
            }

            actions.push(TeamAction::MoveUnit {
                unit_id: *dragged_unit_id,
                target,
            });
        }

        if current_unit.is_some() && !self.filled_slot_actions.is_empty() {
            resp.bar_menu(|ui| {
                for action_name in &self.filled_slot_actions {
                    if ui.button(action_name).clicked() {
                        actions.push(TeamAction::ContextMenuAction {
                            slot_id: target,
                            action_name: action_name.clone(),
                            unit_id: current_unit.map(|u| u.id),
                        });
                        ui.close_menu();
                    }
                }
            });
        }
    }

    fn handle_empty_slot_interactions(
        &self,
        resp: Response,
        slot_id: u64,
        actions: &mut Vec<TeamAction>,
    ) {
        if !self.empty_slot_actions.is_empty() {
            resp.bar_menu(|ui| {
                for action_name in &self.empty_slot_actions {
                    if ui.button(action_name).clicked() {
                        actions.push(TeamAction::ContextMenuAction {
                            slot_id,
                            action_name: action_name.clone(),
                            unit_id: None,
                        });
                        ui.close_menu();
                    }
                }
            });
        }

        if let Some(dragged_unit_id) = resp.dnd_release_payload::<u64>() {
            actions.push(TeamAction::MoveUnit {
                unit_id: *dragged_unit_id,
                target: slot_id,
            });
        }
    }
}
