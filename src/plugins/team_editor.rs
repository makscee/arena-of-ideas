use std::ops::Not;

use super::*;

pub struct TeamEditor<'a> {
    team_entity: Entity,
    context: &'a Context<'a>,
    bench_entity: Option<Entity>,
}

#[derive(Debug, Clone)]
pub enum TeamAction {
    MoveUnit { unit_id: u64, target: u64 },
    ContextMenuAction { slot_id: u64, action_name: String },
    AddSlot { fusion_id: u64 },
}

impl<'a> TeamEditor<'a> {
    pub fn new(team_entity: Entity, context: &'a Context<'a>) -> Self {
        Self {
            team_entity,
            context,
            bench_entity: None,
        }
    }

    pub fn with_bench(mut self, bench_entity: Entity) -> Self {
        self.bench_entity = Some(bench_entity);
        self
    }

    pub fn ui(self, ui: &mut Ui) -> Result<Vec<TeamAction>, ExpressionError> {
        let team = self.context.get::<NTeam>(self.team_entity)?;
        let mut actions = Vec::new();

        let bench_slots = if let Some(bench_entity) = self.bench_entity {
            let mut slots = self
                .context
                .collect_children_components::<NBenchSlot>(self.context.id(bench_entity)?)?;
            slots.sort_by_key(|s| s.index);
            Some(slots)
        } else {
            None
        };

        let fusions = team
            .fusions_load(self.context)
            .into_iter()
            .sorted_by_key(|f| f.index)
            .collect_vec();

        let mut fusion_slots: HashMap<u64, Vec<&NFusionSlot>> = HashMap::new();
        for fusion in &fusions {
            let slots = self
                .context
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
                Self::render_bench_column(
                    &mut columns[column_idx],
                    bench_slots,
                    self.context,
                    &mut actions,
                );
                column_idx += 1;
            }

            for fusion in &fusions {
                let empty_slots = vec![];
                let slots = fusion_slots.get(&fusion.id).unwrap_or(&empty_slots);
                Self::render_fusion_column(
                    &mut columns[column_idx],
                    fusion,
                    slots,
                    self.context,
                    &mut actions,
                );
                column_idx += 1;
            }
        });

        Ok(actions)
    }

    fn render_bench_column(
        ui: &mut Ui,
        bench_slots: &[&NBenchSlot],
        context: &Context,
        actions: &mut Vec<TeamAction>,
    ) {
        ui.vertical(|ui| {
            ui.label("Bench");
            ui.separator();

            for slot in bench_slots {
                Self::render_slot(ui, slot.id, context, actions);
            }
        });
    }

    fn render_fusion_column(
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
                Self::render_slot(ui, slot.id, context, actions);
            }
        });
    }

    fn render_slot(ui: &mut Ui, slot_id: u64, context: &Context, actions: &mut Vec<TeamAction>) {
        let unit = Self::get_slot_unit(slot_id, context);
        let resp = Self::render_unit_in_slot(ui, unit, context);

        if unit.is_some() {
            Self::handle_unit_interactions(resp, slot_id, unit, actions);
        } else {
            Self::handle_empty_slot_interactions(resp, slot_id, context, actions);
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
    }

    fn handle_empty_slot_interactions(
        resp: Response,
        slot_id: u64,
        context: &Context,
        actions: &mut Vec<TeamAction>,
    ) {
        resp.context_menu(|ui| {
            if ui.button("Add Default Unit").clicked() {
                actions.push(TeamAction::ContextMenuAction {
                    slot_id,
                    action_name: "Add Default Unit".to_string(),
                });
                ui.close_menu();
            }

            if ui.button("Add Random Unit").clicked() {
                actions.push(TeamAction::ContextMenuAction {
                    slot_id,
                    action_name: "Add Random Unit".to_string(),
                });
                ui.close_menu();
            }
        });

        if let Some(dragged_unit_id) = resp.dnd_release_payload::<u64>() {
            actions.push(TeamAction::MoveUnit {
                unit_id: *dragged_unit_id,
                target: slot_id,
            });
        }
    }
}
