use super::*;

impl NFusion {
    pub fn remove_unit(&mut self, id: u64) -> Result<(), ExpressionError> {
        if self.trigger.unit == id {
            self.trigger = Default::default();
        }

        if let Some(unit_index) = self.units.ids.iter().position(|u| *u == id) {
            // Remove the behavior at the corresponding index
            if unit_index < self.behavior.len() {
                self.behavior.remove(unit_index);
            }
            self.units.ids.remove(unit_index);
        }

        Ok(())
    }

    pub fn get_action_count(&self) -> usize {
        self.behavior.iter().map(|ar| ar.length as usize).sum()
    }

    pub fn can_add_action(&self) -> bool {
        self.get_action_count() < self.action_limit as usize
    }

    pub fn get_unit_tier(context: &Context, unit_id: u64) -> Result<u8, ExpressionError> {
        if let Ok(behavior) = context.first_parent_recursive::<NUnitBehavior>(unit_id) {
            Ok(behavior.reactions.tier())
        } else {
            Ok(0)
        }
    }
    pub fn units<'a>(&self, context: &'a Context) -> Result<Vec<&'a NUnit>, ExpressionError> {
        context.collect_components::<NUnit>(self.units.ids.iter().copied())
    }
    fn get_behavior<'a>(
        context: &'a Context,
        unit: u64,
    ) -> Result<&'a NUnitBehavior, ExpressionError> {
        context.first_parent_recursive::<NUnitBehavior>(unit)
    }
    pub fn get_trigger<'a>(
        context: &'a Context,
        tr: &UnitTriggerRef,
    ) -> Result<&'a Trigger, ExpressionError> {
        Self::get_behavior(context, tr.unit)?
            .reactions
            .get(tr.trigger as usize)
            .to_e_not_found()
            .map(|b| &b.trigger)
    }
    pub fn get_action<'a>(
        context: &'a Context,
        unit_id: u64,
        ar: &UnitActionRange,
        index: usize,
    ) -> Result<&'a Action, ExpressionError> {
        Self::get_behavior(context, unit_id)?
            .reactions
            .get(ar.trigger as usize)
            .to_e_not_found()?
            .actions
            .get(ar.start as usize + index)
            .to_e_not_found()
    }
    pub fn react(
        &self,
        event: &Event,
        context: &Context,
    ) -> Result<Vec<BattleAction>, ExpressionError> {
        let mut battle_actions: Vec<BattleAction> = default();
        context.with_layer_ref_r(ContextLayer::Owner(self.entity()), |context| {
            if Self::get_trigger(context, &self.trigger)?
                .fire(event, context)
                .unwrap_or_default()
            {
                let units = self.units(context)?;
                let unit_ids: Vec<u64> = units.iter().map(|u| u.id).collect();
                for (unit_index, ar) in self.behavior.iter().enumerate() {
                    if let Some(unit_id) = unit_ids.get(unit_index) {
                        for i in 0..ar.length as usize {
                            let action = Self::get_action(context, *unit_id, ar, i)?;
                            let action = action.clone();
                            context.add_caster(context.entity(*unit_id)?);
                            battle_actions.extend(action.process(context)?);
                        }
                    }
                }
            }
            Ok(())
        })?;
        Ok(battle_actions)
    }
    pub fn paint(&self, rect: Rect, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        let entity = self.entity();
        let units = self.units(context)?;
        for unit in units {
            let unit_entity = unit.entity();
            let Ok(rep) = context.first_parent_recursive::<NUnitRepresentation>(unit.id) else {
                continue;
            };
            context
                .with_owner_ref(unit_entity, |context| {
                    RepresentationPlugin::paint_rect(rect, &context, &rep.material, ui)
                })
                .ui(ui);
        }
        for rep in context.collect_children_components::<NUnitRepresentation>(self.id)? {
            context
                .with_owner_ref(entity, |context| {
                    RepresentationPlugin::paint_rect(rect, context, &rep.material, ui)
                })
                .ui(ui);
        }
        Ok(())
    }
    pub fn find_by_slot(slot: i32, world: &mut World) -> Option<Self> {
        world.query::<&NFusion>().iter(&world).find_map(|f| {
            if f.slot == slot {
                Some(f.clone())
            } else {
                None
            }
        })
    }
    pub fn show_editor(&mut self, context: &Context, ui: &mut Ui) -> Result<bool, ExpressionError> {
        let mut changed = false;
        let units = self.units(context)?;
        ui.horizontal(|ui| -> Result<(), ExpressionError> {
            ui.vertical(|ui| -> Result<(), ExpressionError> {
                // Select trigger
                ui.label("Select Trigger:");
                for unit in &units {
                    let b = Self::get_behavior(context, unit.id)?;
                    for (ti, r) in b.reactions.iter().enumerate() {
                        let trigger_ref = UnitTriggerRef {
                            unit: unit.id,
                            trigger: ti as u8,
                        };
                        if self.trigger == trigger_ref {
                            continue;
                        }
                        if r.trigger.cstr().button(ui).clicked() {
                            self.trigger = trigger_ref;
                            changed = true;
                        }
                    }
                }

                if self.trigger.unit == 0 {
                    return Ok(());
                }

                ui.separator();
                ui.label("Add Action Ranges:");
                ui.horizontal(|ui| {
                    ui.label(format!(
                        "Actions: {}/{}",
                        self.get_action_count(),
                        self.action_limit
                    ));
                });

                if self.can_add_action() {
                    for (unit_index, unit) in units.iter().enumerate() {
                        let b = Self::get_behavior(context, unit.id)?;
                        for (ti, r) in b.reactions.iter().enumerate() {
                            if r.actions.is_empty() {
                                continue;
                            }
                            ui.label(format!("Unit {} Trigger {}", unit.id, ti));
                            ui.horizontal(|ui| {
                                for start in 0..r.actions.len() {
                                    for length in 1..=(r.actions.len() - start) {
                                        let ar = UnitActionRange {
                                            trigger: ti as u8,
                                            start: start as u8,
                                            length: length as u8,
                                        };

                                        // Check if this range overlaps with existing ones at this unit index
                                        let overlaps = self.behavior.get(unit_index).map_or(
                                            false,
                                            |existing| {
                                                existing.trigger == ar.trigger
                                                    && (existing.start < ar.start + ar.length
                                                        && ar.start
                                                            < existing.start + existing.length)
                                            },
                                        );

                                        if !overlaps {
                                            let range_text = if length == 1 {
                                                format!("Action {}", start)
                                            } else {
                                                format!("Actions {}-{}", start, start + length - 1)
                                            };

                                            if ui.button(range_text).clicked() {
                                                // Ensure behavior vector has the same length as units
                                                match self.units(context) {
                                                    Ok(units) => {
                                                        self.behavior.resize(
                                                            units.len(),
                                                            UnitActionRange {
                                                                trigger: 0,
                                                                start: 0,
                                                                length: 0,
                                                            },
                                                        );

                                                        // Update the behavior at the unit index
                                                        if unit_index < self.behavior.len() {
                                                            self.behavior[unit_index] = ar;
                                                            changed = true;
                                                        }
                                                    }
                                                    Err(_) => {}
                                                }
                                            }
                                        }
                                    }
                                }
                            });
                        }
                    }
                } else {
                    ui.label("Action limit reached");
                }
                Ok(())
            })
            .inner?;
            space(ui);
            ui.vertical(|ui| -> Result<(), ExpressionError> {
                // Show current trigger
                if self.trigger.unit != 0 {
                    let trigger = Self::get_trigger(context, &self.trigger)?;
                    ui.label("Current Trigger:");
                    if trigger.cstr().button(ui).clicked() {
                        self.trigger = Default::default();
                        self.behavior.clear();
                        changed = true;
                    }
                }

                ui.separator();
                ui.label("Action Ranges:");

                let mut new_behavior = None;
                for (i, ar) in self.behavior.iter().enumerate() {
                    ui.horizontal(|ui| -> Result<(), ExpressionError> {
                        ui.vertical(|ui| {
                            let can_move_up = i > 0;
                            let can_move_down = i < self.behavior.len() - 1;
                            let size = (LINE_HEIGHT * 0.5).v2();
                            if RectButton::new_size(size)
                                .enabled(can_move_up)
                                .no_bar_check(true)
                                .ui(ui, |color, rect, _, ui| {
                                    triangle(rect, color, 0, ui);
                                })
                                .clicked()
                            {
                                let mut behavior = self.behavior.clone();
                                behavior.swap(i, i - 1);
                                new_behavior = Some(behavior);
                            }
                            if RectButton::new_size(size)
                                .enabled(can_move_down)
                                .no_bar_check(true)
                                .ui(ui, |color, rect, _, ui| {
                                    triangle(rect, color, 2, ui);
                                })
                                .clicked()
                            {
                                let mut behavior = self.behavior.clone();
                                behavior.swap(i, i + 1);
                                new_behavior = Some(behavior);
                            }
                        });

                        let range_text = if ar.length == 1 {
                            format!("Unit {} Action {}", i, ar.start)
                        } else {
                            format!(
                                "Unit {} Actions {}-{}",
                                i,
                                ar.start,
                                ar.start + ar.length - 1
                            )
                        };

                        if ui.button(range_text).clicked() {
                            let mut behavior = self.behavior.clone();
                            behavior.remove(i);
                            new_behavior = Some(behavior);
                        }
                        Ok(())
                    });
                }

                if let Some(mut new_behavior) = new_behavior {
                    mem::swap(&mut self.behavior, &mut new_behavior);
                    changed = true;
                }
                Ok(())
            })
            .inner
        })
        .inner?;
        Ok(changed)
    }

    // Extract range selector functionality for reuse
    pub fn render_unit_range_selector(
        &mut self,
        ui: &mut Ui,
        context: &Context,
        unit: &NUnit,
        slot_idx: usize,
    ) -> Result<bool, ExpressionError> {
        let mut changed = false;

        if let Ok(behavior) = context.first_parent_recursive::<NUnitBehavior>(unit.id) {
            let action_range = self.get_action_range(slot_idx);
            let max_actions = Self::get_max_actions(&behavior, &self.trigger);

            if max_actions > 0 {
                let (current_start, current_len) = action_range;

                let range_selector = RangeSelector::new(max_actions)
                    .range(current_start, current_len)
                    .border_thickness(3.0)
                    .drag_threshold(12.0)
                    .show_drag_hints(false)
                    .show_debug_info(false)
                    .id(egui::Id::new(format!("fusion_range_selector_{}", unit.id)));

                let (_, range_changed) =
                    range_selector.ui(ui, context, |item_ui, ctx, action_idx, is_in_range| {
                        if let Some(reaction) =
                            behavior.reactions.get(self.trigger.trigger as usize)
                        {
                            if let Some(action) = reaction.actions.get(action_idx) {
                                let vctx = ViewContext::new(item_ui).non_interactible(true);
                                if is_in_range {
                                    Self::render_action_normal(item_ui, ctx, unit, action, vctx);
                                } else {
                                    Self::render_action_greyed(item_ui, ctx, unit, action, vctx);
                                }
                            }
                        }
                        Ok(())
                    });

                if let Some((new_start, new_length)) = range_changed {
                    // Ensure behavior vector has the correct size
                    if let Ok(units) = self.units(context) {
                        if self.behavior.len() < units.len() {
                            self.behavior.resize(
                                units.len(),
                                UnitActionRange {
                                    trigger: 0,
                                    start: 0,
                                    length: 0,
                                },
                            );
                        }

                        if let Some(action_ref) = self.behavior.get_mut(slot_idx) {
                            action_ref.start = new_start;
                            action_ref.length = new_length;
                            changed = true;
                        }
                    }
                }
            }
        }

        Ok(changed)
    }

    fn get_action_range(&self, slot_idx: usize) -> (u8, u8) {
        let current_action_ref = self.behavior.get(slot_idx);
        let current_start = current_action_ref.map(|ar| ar.start).unwrap_or(0);
        let current_len = current_action_ref.map(|ar| ar.length).unwrap_or(0);
        (current_start, current_len)
    }

    fn get_max_actions(behavior: &NUnitBehavior, trigger: &UnitTriggerRef) -> u8 {
        behavior
            .reactions
            .get(trigger.trigger as usize)
            .map(|reaction| reaction.actions.len() as u8)
            .unwrap_or(0)
    }

    fn render_action_normal(
        ui: &mut Ui,
        context: &Context,
        unit: &NUnit,
        action: &Action,
        vctx: ViewContext,
    ) {
        if let Ok(entity) = context.entity(unit.id) {
            context
                .with_owner_ref(entity, |context| {
                    action.title_cstr(vctx, context).label_w(ui);
                    Ok(())
                })
                .ui(ui);
        }
    }

    fn render_action_greyed(
        ui: &mut Ui,
        context: &Context,
        unit: &NUnit,
        action: &Action,
        vctx: ViewContext,
    ) {
        let old_style = ui.visuals().clone();
        ui.visuals_mut().override_text_color = Some(egui::Color32::GRAY);

        if let Ok(entity) = context.entity(unit.id) {
            context
                .with_owner_ref(entity, |context| {
                    action
                        .title_cstr(vctx, context)
                        .as_label_alpha(0.5, ui.style())
                        .wrap()
                        .ui(ui);
                    Ok(())
                })
                .ui(ui);
        }

        *ui.visuals_mut() = old_style;
    }
    pub fn slots_editor(
        team: Entity,
        context: &Context,
        ui: &mut Ui,
        slot: impl Fn(&mut Ui, &Response, &NFusion),
        mut on_reorder: impl FnMut(Vec<u64>),
    ) -> Result<(), ExpressionError> {
        let team = context.get::<NTeam>(team)?;
        let fusions: HashMap<usize, &NFusion> = HashMap::from_iter(
            team.fusions_load(context)
                .into_iter()
                .map(|f| (f.slot as usize, f)),
        );
        if ui.available_width() < 30.0 {
            return Ok(());
        }
        ui.columns(fusions.len(), |ui| {
            for i in (0..fusions.len()).rev() {
                let ui = &mut ui[i];
                let i = fusions.len() - i - 1;
                let fusion = fusions.get(&i).unwrap();
                let resp = if fusion.units.ids.is_empty() {
                    MatRect::new(ui.available_size()).ui(ui, context)
                } else {
                    // Get all unit representations
                    let units = fusion.units(context).unwrap_or_default();
                    let mut mat_rect = MatRect::new(ui.available_size());

                    // Add unit representations
                    for unit in units {
                        if let Ok(rep) =
                            context.first_parent_recursive::<NUnitRepresentation>(unit.id)
                        {
                            mat_rect = mat_rect.add_mat(&rep.material, unit.id);
                        }
                    }

                    // Add fusion-specific representations
                    if let Ok(fusion_reps) =
                        context.collect_children_components::<NUnitRepresentation>(fusion.id)
                    {
                        for rep in fusion_reps {
                            mat_rect = mat_rect.add_mat(&rep.material, fusion.id);
                        }
                    }

                    mat_rect.unit_rep_with_default(fusion.id).ui(ui, context)
                };

                if resp.dragged() && !fusion.units.ids.is_empty() {
                    if let Some(pos) = ui.ctx().pointer_latest_pos() {
                        let origin = resp.rect.center();
                        ui.painter().arrow(
                            origin,
                            pos - origin,
                            ui.visuals().widgets.hovered.fg_stroke,
                        );
                    }
                }
                resp.dnd_set_drag_payload(i);
                if let Some(j) = resp.dnd_release_payload::<usize>() {
                    if i == *j {
                        continue;
                    }
                    let mut fusions = fusions
                        .iter()
                        .sorted_by_key(|(i, _)| **i)
                        .map(|(_, f)| f.id)
                        .collect_vec();
                    let id = fusions.remove(*j);
                    fusions.insert(i, id);
                    on_reorder(fusions);
                }
                slot(ui, &resp, fusion);
            }
        });
        Ok(())
    }
}
