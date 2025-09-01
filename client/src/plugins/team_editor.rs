use super::*;

pub struct TeamEditor {
    team_entity: Entity,
    empty_slot_actions: Vec<String>,
    filled_slot_actions: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum TeamAction {
    MoveUnit {
        unit_id: u64,
        target: u64,
    },
    BenchUnit {
        unit_id: u64,
    },
    ContextMenuAction {
        slot_id: u64,
        action_name: String,
        unit_id: Option<u64>,
    },
    AddSlot {
        fusion_id: u64,
    },
    ChangeActionRange {
        slot_id: u64,
        start: u8,
        length: u8,
    },
    ChangeTrigger {
        fusion_id: u64,
        trigger: u64,
    },
}

impl TeamEditor {
    pub fn new(team_entity: Entity) -> Self {
        Self {
            team_entity,
            empty_slot_actions: Vec::new(),
            filled_slot_actions: Vec::new(),
        }
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

        let state_id = egui::Id::new(self.team_entity.index()).with("team_editor_selected_fusion");
        let mut selected_fusion_id =
            ui.memory(|m| m.data.get_temp::<Option<u64>>(state_id).unwrap_or(None));

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

        let unlinked_units = self.get_unlinked_units(context, &fusion_slots)?;
        let total_columns = fusions.len() + 1; // +1 for bench column

        let mut selected_column_rect = None;
        let mut clicked_fusion_id = None;

        ui.columns(total_columns, |columns| {
            let mut column_idx = 0;

            for fusion in &fusions {
                let empty_slots = vec![];
                let slots = fusion_slots.get(&fusion.id).unwrap_or(&empty_slots);

                if selected_fusion_id == Some(fusion.id) {
                    self.render_fusion_actions_column(
                        &mut columns[column_idx],
                        fusion,
                        slots,
                        context,
                        &mut actions,
                    );
                    selected_column_rect = Some(columns[column_idx].min_rect());
                } else {
                    let clicked = self.render_fusion_column_with_actions_display(
                        &mut columns[column_idx],
                        fusion,
                        slots,
                        context,
                        &mut actions,
                    );
                    if clicked {
                        clicked_fusion_id = Some(fusion.id);
                    }
                }
                column_idx += 1;
            }
            // Bench column (last column)
            self.render_bench_column(
                &mut columns[column_idx],
                &unlinked_units,
                context,
                &mut actions,
            );
        });

        if let Some(fusion_id) = clicked_fusion_id {
            selected_fusion_id = Some(fusion_id);
            ui.memory_mut(|m| {
                m.data.insert_temp(state_id, selected_fusion_id);
            });
        }
        if clicked_fusion_id.is_none()
            && ui.input(|i| i.pointer.any_click())
            && selected_fusion_id.is_some()
        {
            if let Some(pointer_pos) = ui.input(|i| i.pointer.interact_pos()) {
                let clicked_outside =
                    selected_column_rect.map_or(true, |rect| !rect.contains(pointer_pos));
                if clicked_outside {
                    selected_fusion_id = None;
                    ui.memory_mut(|m| {
                        m.data.insert_temp(state_id, selected_fusion_id);
                    });
                }
            }
        }

        // Process additional actions from unit moves and bench moves
        let mut all_actions = actions;
        let mut additional_actions = Vec::new();

        for action in &all_actions {
            match action {
                TeamAction::MoveUnit { unit_id, target } => {
                    let move_additional =
                        self.get_move_unit_additional_actions(*unit_id, *target, context)?;
                    additional_actions.extend(move_additional);
                }
                TeamAction::BenchUnit { unit_id } => {
                    let bench_additional =
                        self.get_bench_unit_additional_actions(*unit_id, context)?;
                    additional_actions.extend(bench_additional);
                }
                _ => {}
            }
        }

        all_actions.extend(additional_actions);
        Ok(all_actions)
    }

    fn get_unlinked_units<'a>(
        &self,
        context: &'a Context,
        _fusion_slots: &HashMap<u64, Vec<&NFusionSlot>>,
    ) -> Result<Vec<&'a NUnit>, ExpressionError> {
        let all_units = context
            .collect_children_components_recursive::<NUnit>(context.id(self.team_entity)?)?;

        Ok(all_units
            .into_iter()
            .filter(|unit| context.first_child::<NFusionSlot>(unit.id).is_err())
            .collect())
    }

    fn get_available_triggers(
        &self,
        context: &Context,
        slots: &[&NFusionSlot],
    ) -> Result<(Vec<Trigger>, HashMap<Trigger, u64>), ExpressionError> {
        let mut trigger_map = HashMap::new();

        for slot in slots {
            if let Some(unit) = Self::get_slot_unit(slot.id, context) {
                if let Ok(unit_behavior) = context.first_parent_recursive::<NUnitBehavior>(unit.id)
                {
                    let trigger = unit_behavior.reaction.trigger.clone();
                    if !trigger_map.contains_key(&trigger) {
                        trigger_map.insert(trigger, unit.id);
                    }
                }
            }
        }

        let mut triggers: Vec<Trigger> = trigger_map.keys().cloned().collect();
        triggers.sort_by(|a, b| a.as_ref().cmp(b.as_ref()));

        Ok((triggers, trigger_map))
    }

    fn get_move_unit_additional_actions(
        &self,
        unit_id: u64,
        target_id: u64,
        context: &Context,
    ) -> Result<Vec<TeamAction>, ExpressionError> {
        let mut additional_actions = Vec::new();

        let old_fusion_id = if let Ok(old_slot) = context.first_child::<NFusionSlot>(unit_id) {
            let old_fusion = context.first_child::<NFusion>(old_slot.id)?;
            Some((old_fusion.id, old_slot.id))
        } else {
            None
        };

        let new_fusion_id =
            if let Ok(new_slot) = context.get::<NFusionSlot>(context.entity(target_id)?) {
                let new_fusion = context.first_child::<NFusion>(new_slot.id)?;
                Some(new_fusion.id)
            } else {
                None
            };

        let is_same_fusion = match (old_fusion_id, new_fusion_id) {
            (Some((old_id, _)), Some(new_id)) => old_id == new_id,
            _ => false,
        };

        // Handle moving out of previous slot
        if let Some((old_fusion_id, old_slot_id)) = old_fusion_id {
            // Reset action range for the slot being vacated
            additional_actions.push(TeamAction::ChangeActionRange {
                slot_id: old_slot_id,
                start: 0,
                length: 0,
            });

            // Update trigger only if moving to different fusion (not same fusion)
            if !is_same_fusion {
                let old_fusion = context.get::<NFusion>(context.entity(old_fusion_id)?)?;
                if old_fusion.trigger_unit == unit_id {
                    let remaining_slots =
                        context.collect_parents_components::<NFusionSlot>(old_fusion_id)?;
                    let mut new_trigger_ref = 0u64;

                    for slot in remaining_slots {
                        if slot.id != old_slot_id {
                            if let Ok(unit) = context.first_parent::<NUnit>(slot.id) {
                                if context
                                    .first_parent_recursive::<NUnitBehavior>(unit.id)
                                    .is_ok()
                                {
                                    new_trigger_ref = unit.id;
                                    break;
                                }
                            }
                        }
                    }

                    additional_actions.push(TeamAction::ChangeTrigger {
                        fusion_id: old_fusion_id,
                        trigger: new_trigger_ref,
                    });
                }
            }
        }

        // Handle moving into new slot
        if let Some(new_fusion_id) = new_fusion_id {
            let new_fusion = context.get::<NFusion>(context.entity(new_fusion_id)?)?;

            // If fusion has no trigger set (unit id is 0), set it to this unit
            if new_fusion.trigger_unit == 0 {
                if context
                    .first_parent_recursive::<NUnitBehavior>(unit_id)
                    .is_ok()
                {
                    additional_actions.push(TeamAction::ChangeTrigger {
                        fusion_id: new_fusion_id,
                        trigger: unit_id,
                    });
                }
            }

            // Set action range to select all actions for the moved unit
            if let Ok(unit_behavior) = context.first_parent_recursive::<NUnitBehavior>(unit_id) {
                let reaction = &unit_behavior.reaction;
                additional_actions.push(TeamAction::ChangeActionRange {
                    slot_id: target_id,
                    start: 0,
                    length: reaction.actions.len() as u8,
                });
            }
        }

        Ok(additional_actions)
    }

    fn get_bench_unit_additional_actions(
        &self,
        unit_id: u64,
        context: &Context,
    ) -> Result<Vec<TeamAction>, ExpressionError> {
        let mut additional_actions = Vec::new();

        // Handle moving to bench
        if let Ok(old_slot) = context.first_child::<NFusionSlot>(unit_id) {
            let old_fusion = context.first_child::<NFusion>(old_slot.id)?;

            // Reset action range for the slot being vacated
            additional_actions.push(TeamAction::ChangeActionRange {
                slot_id: old_slot.id,
                start: 0,
                length: 0,
            });

            // If this unit was the trigger unit, update trigger to another unit or default
            if old_fusion.trigger_unit == unit_id {
                let remaining_slots =
                    context.collect_parents_components::<NFusionSlot>(old_fusion.id)?;
                let mut new_trigger_ref = 0u64;

                for slot in remaining_slots {
                    if slot.id != old_slot.id {
                        if let Ok(unit) = context.first_parent::<NUnit>(slot.id) {
                            if context
                                .first_parent_recursive::<NUnitBehavior>(unit.id)
                                .is_ok()
                            {
                                new_trigger_ref = unit.id;
                                break;
                            }
                        }
                    }
                }

                additional_actions.push(TeamAction::ChangeTrigger {
                    fusion_id: old_fusion.id,
                    trigger: new_trigger_ref,
                });
            }
        }

        Ok(additional_actions)
    }

    fn render_bench_column(
        &self,
        ui: &mut Ui,
        unlinked_units: &[&NUnit],
        context: &Context,
        actions: &mut Vec<TeamAction>,
    ) {
        ui.vertical(|ui| {
            ui.label("Bench");
            ui.separator();

            for unit in unlinked_units {
                let resp = self.render_unit_with_representation(
                    ui,
                    unit,
                    egui::Vec2::new(60.0, 60.0),
                    context,
                );
                self.handle_bench_unit_interactions(resp, unit.id, actions);
            }

            // DndArea for dropping units into bench
            let mut drop_rect = ui.min_rect();
            drop_rect.extend_with_y(drop_rect.top() + 60.0);
            if drop_rect.height() > 10.0 {
                if let Some(dragged_unit_id) = DndArea::<u64>::new(drop_rect)
                    .text("Drop unit to bench")
                    .ui(ui)
                {
                    actions.push(TeamAction::BenchUnit {
                        unit_id: *dragged_unit_id,
                    });
                }
            }
        });
    }

    fn handle_bench_unit_interactions(
        &self,
        resp: Response,
        unit_id: u64,
        actions: &mut Vec<TeamAction>,
    ) {
        // Handle context menu
        if !self.filled_slot_actions.is_empty() {
            resp.bar_menu(|ui| {
                for action_name in &self.filled_slot_actions {
                    if ui.button(action_name).clicked() {
                        actions.push(TeamAction::ContextMenuAction {
                            slot_id: 0,
                            action_name: action_name.clone(),
                            unit_id: Some(unit_id),
                        });
                        ui.close_menu();
                    }
                }
            });
        }
    }

    fn render_fusion_column_with_actions_display(
        &self,
        ui: &mut Ui,
        fusion: &NFusion,
        slots: &[&NFusionSlot],
        context: &Context,
        actions: &mut Vec<TeamAction>,
    ) -> bool {
        let mut clicked = false;

        ui.vertical(|ui| {
            ui.horizontal(|ui| {
                ui.label(format!("Fusion {}", fusion.index));
                if ui.button("Add Slot").clicked() {
                    actions.push(TeamAction::AddSlot {
                        fusion_id: fusion.id,
                    });
                }
            });

            let group_response = ui.group(|ui| {
                self.render_fusion_action_sequence(fusion, context, ui);
            });
            let actions_response = ui.allocate_rect(group_response.response.rect, Sense::click());

            if actions_response.hovered() {
                ui.output_mut(|o| o.cursor_icon = egui::CursorIcon::PointingHand);
            }
            if actions_response.clicked() {
                clicked = true;
            }

            for slot in slots {
                self.render_slot(ui, slot.id, context, actions);
            }
        });

        clicked
    }

    fn render_fusion_action_sequence(&self, fusion: &NFusion, context: &Context, ui: &mut Ui) {
        if let Ok(trigger) = NFusion::get_trigger(context, fusion.trigger_unit) {
            ui.horizontal(|ui| {
                Icon::Lightning.show(ui);
                trigger.cstr().label(ui);
            });
            ui.separator();
        }
        match fusion.gather_fusion_actions(context) {
            Ok(fusion_actions) => {
                if fusion_actions.is_empty() {
                    ui.label("No actions selected");
                } else {
                    ui.vertical(|ui| {
                        for (unit_id, action) in fusion_actions {
                            if let Ok(unit) = context.get_by_id::<NUnit>(unit_id) {
                                context
                                    .with_owner_ref(unit.entity(), |context| {
                                        action.view_title(
                                            ViewContext::new(ui).non_interactible(true),
                                            context,
                                            ui,
                                        );
                                        Ok(())
                                    })
                                    .ui(ui);
                            } else {
                                action.cstr().label_w(ui);
                            }
                        }
                    });
                }
            }
            Err(e) => e.ui(ui),
        }
    }

    fn render_slot(
        &self,
        ui: &mut Ui,
        slot_id: u64,
        context: &Context,
        actions: &mut Vec<TeamAction>,
    ) {
        let current_unit = Self::get_slot_unit(slot_id, context);
        let resp = self.render_unit_in_slot(ui, current_unit, context);
        let slot_rect = resp.rect;

        if current_unit.is_some() {
            self.handle_unit_interactions(&resp, slot_id, current_unit, actions);
        } else {
            self.handle_empty_slot_interactions(&resp, slot_id, actions);
        }

        // DndArea for dropping units onto slots
        if let Some(dragged_unit_id) = DndArea::<u64>::new(slot_rect)
            .text_fn(ui, |_| {
                if current_unit.is_some() {
                    "Swap units".to_string()
                } else {
                    "Drop unit here".to_string()
                }
            })
            .ui(ui)
        {
            actions.push(TeamAction::MoveUnit {
                unit_id: *dragged_unit_id,
                target: slot_id,
            });
        }
    }

    fn get_slot_unit<'b>(slot_id: u64, context: &'b Context) -> Option<&'b NUnit> {
        context.first_parent::<NUnit>(slot_id).ok()
    }

    fn render_unit_in_slot(
        &self,
        ui: &mut Ui,
        unit: Option<&NUnit>,
        context: &Context,
    ) -> Response {
        let size = egui::Vec2::new(60.0, 60.0);

        if let Some(unit) = unit {
            self.render_unit_with_representation(ui, unit, size, context)
        } else {
            Self::render_empty_slot(ui, size, context)
        }
    }

    fn render_unit_with_representation(
        &self,
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
        resp: &Response,
        target: u64,
        current_unit: Option<&NUnit>,
        actions: &mut Vec<TeamAction>,
    ) {
        // Handle context menu for filled slots
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
        resp: &Response,
        slot_id: u64,
        actions: &mut Vec<TeamAction>,
    ) {
        // Handle context menu for empty slots
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
    }

    fn render_fusion_actions_column(
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
            });
            ui.separator();

            // Trigger selector
            if let Ok((available_triggers, trigger_map)) =
                self.get_available_triggers(context, slots)
            {
                if !available_triggers.is_empty() {
                    let current_trigger =
                        if let Ok(trigger) = NFusion::get_trigger(context, fusion.trigger_unit) {
                            trigger.clone()
                        } else {
                            Trigger::default()
                        };

                    let mut selected_trigger = current_trigger;
                    if Selector::new("").ui_iter(&mut selected_trigger, &available_triggers, ui) {
                        if let Some(trigger_ref) = trigger_map.get(&selected_trigger) {
                            actions.push(TeamAction::ChangeTrigger {
                                fusion_id: fusion.id,
                                trigger: trigger_ref.clone(),
                            });
                        }
                    }
                    ui.separator();
                }
            }

            for slot in slots {
                if let Some(unit) = Self::get_slot_unit(slot.id, context) {
                    ui.group(|ui| {
                        if let Ok(unit_behavior) =
                            context.first_parent_recursive::<NUnitBehavior>(unit.id)
                        {
                            let max_actions = unit_behavior.reaction.actions.len() as u8;
                            if max_actions > 0 {
                                let (current_start, current_len) =
                                    (slot.actions.start, slot.actions.length);

                                let range_selector = RangeSelector::new(max_actions)
                                    .range(current_start, current_len)
                                    .border_thickness(2.0)
                                    .drag_threshold(8.0)
                                    .show_drag_hints(true)
                                    .show_debug_info(false)
                                    .id(egui::Id::new("team_range_selector").with(slot.id));

                                let (_, range_changed) = range_selector.ui(
                                    ui,
                                    context,
                                    |item_ui, ctx, action_idx, is_in_range| {
                                        let reaction = &unit_behavior.reaction;
                                        if let Some(action) = reaction.actions.get(action_idx) {
                                            let vctx =
                                                ViewContext::new(item_ui).non_interactible(true);
                                            if is_in_range {
                                                Self::render_action_normal(
                                                    item_ui, ctx, unit, action, vctx,
                                                );
                                            } else {
                                                Self::render_action_greyed(
                                                    item_ui, ctx, unit, action, vctx,
                                                );
                                            }
                                        }
                                        Ok(())
                                    },
                                );

                                if let Some((new_start, new_length)) = range_changed {
                                    actions.push(TeamAction::ChangeActionRange {
                                        slot_id: slot.id,
                                        start: new_start,
                                        length: new_length,
                                    });
                                }
                            } else {
                                ui.label("No unit actions");
                            }
                        } else {
                            ui.label("No unit behavior");
                        }
                    });
                } else {
                    ui.group(|ui| {
                        ui.label(format!("Empty Slot {}", slot.index));
                    });
                }
            }
        });
    }

    fn render_action_normal(
        ui: &mut Ui,
        context: &Context,
        _unit: &NUnit,
        action: &Action,
        vctx: ViewContext,
    ) {
        ui.horizontal(|ui| {
            action.view_title(vctx, context, ui);
        });
    }

    fn render_action_greyed(
        ui: &mut Ui,
        context: &Context,
        _unit: &NUnit,
        action: &Action,
        vctx: ViewContext,
    ) {
        ui.horizontal(|ui| {
            ui.style_mut().visuals.widgets.inactive.weak_bg_fill = ui
                .style()
                .visuals
                .widgets
                .inactive
                .weak_bg_fill
                .gamma_multiply(0.5);
            action.view_title(vctx, context, ui);
        });
    }
}
