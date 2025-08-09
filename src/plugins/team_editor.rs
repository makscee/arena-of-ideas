use super::*;

// Usage example:
// let actions = TeamEditor::new(team_entity, context)
//     .ui(ui)?;
//
// for action in actions {
//     match action {
//         TeamAction::MoveUnit { unit_id, target } => {
//             // Handle unit movement
//         }
//         TeamAction::ContextMenuAction { slot_id, action_name } => {
//             // Handle context menu action
//         }
//     }
// }

pub struct TeamEditor<'a> {
    team_entity: Entity,
    context: &'a Context<'a>,
}

#[derive(Debug, Clone, Copy)]
pub enum SlotTarget {
    Bench(u64),  // NBenchSlot id
    Fusion(u64), // NFusionSlot id
}

#[derive(Debug, Clone)]
pub enum TeamAction {
    MoveUnit { unit_id: u64, target: SlotTarget },
    ContextMenuAction { slot_id: u64, action_name: String },
}

impl<'a> TeamEditor<'a> {
    pub fn new(team_entity: Entity, context: &'a Context<'a>) -> Self {
        Self {
            team_entity,
            context,
        }
    }

    pub fn ui(self, ui: &mut Ui) -> Result<Vec<TeamAction>, ExpressionError> {
        let team = self.context.get::<NTeam>(self.team_entity)?;
        let mut actions = Vec::new();

        // Get bench slots
        let bench_slots = self
            .context
            .collect_children_components::<NBenchSlot>(team.id)?;
        let mut bench_slots_sorted: Vec<&NBenchSlot> = bench_slots.into_iter().collect();
        bench_slots_sorted.sort_by_key(|s| s.index);

        // Get fusions and their slots
        let fusions = team.fusions_load(self.context);
        let fusions_sorted = fusions.into_iter().sorted_by_key(|f| f.index).collect_vec();

        let mut fusion_slots: HashMap<u64, Vec<&NFusionSlot>> = HashMap::new();
        for fusion in &fusions_sorted {
            let slots = self
                .context
                .collect_children_components::<NFusionSlot>(fusion.id)?;
            let mut slots_sorted: Vec<&NFusionSlot> = slots.into_iter().collect();
            slots_sorted.sort_by_key(|s| s.index);
            fusion_slots.insert(fusion.id, slots_sorted);
        }

        // Calculate columns: bench + fusions
        let total_columns = 1 + fusions_sorted.len();

        if total_columns == 0 {
            return Ok(actions);
        }

        ui.columns(total_columns, |columns| {
            // Render bench column
            Self::render_bench_column(
                &mut columns[0],
                &bench_slots_sorted,
                self.context,
                &mut actions,
            );

            // Render fusion columns
            for (fusion_idx, fusion) in fusions_sorted.iter().enumerate() {
                let column_idx = fusion_idx + 1;
                let empty_slots = vec![];
                let slots = fusion_slots.get(&fusion.id).unwrap_or(&empty_slots);
                Self::render_fusion_column(
                    &mut columns[column_idx],
                    fusion,
                    slots,
                    self.context,
                    &mut actions,
                );
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
                Self::render_bench_slot(ui, slot, context, actions);
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
            ui.label(format!("Fusion {}", fusion.index));
            ui.separator();

            for slot in slots {
                Self::render_fusion_slot(ui, slot, context, actions);
            }
        });
    }

    fn render_bench_slot(
        ui: &mut Ui,
        slot: &NBenchSlot,
        context: &Context,
        actions: &mut Vec<TeamAction>,
    ) {
        let unit = Self::get_slot_unit(slot.id, context);
        let resp = Self::render_unit_in_slot(ui, unit, context);

        if unit.is_some() {
            Self::handle_unit_interactions(resp, SlotTarget::Bench(slot.id), unit, actions);
        } else {
            Self::handle_empty_slot_interactions(resp, slot.id, actions);
        }
    }

    fn render_fusion_slot(
        ui: &mut Ui,
        slot: &NFusionSlot,
        context: &Context,
        actions: &mut Vec<TeamAction>,
    ) {
        let unit = Self::get_slot_unit(slot.id, context);
        let resp = Self::render_unit_in_slot(ui, unit, context);

        if unit.is_some() {
            Self::handle_unit_interactions(resp, SlotTarget::Fusion(slot.id), unit, actions);
        } else {
            Self::handle_empty_slot_interactions(resp, slot.id, actions);
        }
    }

    fn get_slot_unit<'b>(slot_id: u64, context: &'b Context) -> Option<&'b NUnit> {
        context
            .collect_children_components::<NUnit>(slot_id)
            .ok()
            .and_then(|units| units.into_iter().next())
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

        // Add unit representation
        if let Ok(rep) = context.first_parent_recursive::<NUnitRepresentation>(unit.id) {
            mat_rect = mat_rect.add_mat(&rep.material, unit.id);
        }

        let resp = mat_rect.unit_rep_with_default(unit.id).ui(ui, context);

        // Handle dragging visual feedback
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
        target: SlotTarget,
        current_unit: Option<&NUnit>,
        actions: &mut Vec<TeamAction>,
    ) {
        // Handle drop onto this slot
        if let Some(dragged_unit_id) = resp.dnd_release_payload::<u64>() {
            // Don't move unit to its current slot
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

    fn handle_empty_slot_interactions(resp: Response, slot_id: u64, actions: &mut Vec<TeamAction>) {
        // Handle right-click context menu for empty slots
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

        // Handle drop onto empty slot
        if let Some(dragged_unit_id) = resp.dnd_release_payload::<u64>() {
            // Determine target type based on slot context
            // For now, we assume bench slots for empty drops
            let target = SlotTarget::Bench(slot_id);
            actions.push(TeamAction::MoveUnit {
                unit_id: *dragged_unit_id,
                target,
            });
        }
    }
}
