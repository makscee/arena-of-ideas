use super::*;

impl NFusion {
    pub fn remove_unit(&mut self, id: u64) -> Result<(), ExpressionError> {
        if self.trigger.unit == id {
            self.trigger = Default::default();
        }

        // if let Some(unit_index) = self.units.ids.iter().position(|u| *u == id) {
        //     if unit_index < self.behavior.len() {
        //         self.behavior.remove(unit_index);
        //     }
        //     self.units.ids.remove(unit_index);
        // }

        Ok(())
    }

    pub fn get_action_count(&self) -> usize {
        // self.behavior.iter().map(|ar| ar.length as usize).sum()
        0
    }

    pub fn can_add_action(&self) -> bool {
        // self.get_action_count() < self.action_limit as usize
        true
    }

    pub fn get_unit_tier(context: &Context, unit_id: u64) -> Result<u8, ExpressionError> {
        if let Ok(behavior) = context.first_parent_recursive::<NUnitBehavior>(unit_id) {
            Ok(behavior.reactions.tier())
        } else {
            Ok(0)
        }
    }
    pub fn units<'a>(&self, context: &'a Context) -> Result<Vec<&'a NUnit>, ExpressionError> {
        context.collect_parents_components_recursive(self.id)
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
                // for (unit_index, ar) in self.behavior.iter().enumerate() {
                //     if let Some(unit_id) = unit_ids.get(unit_index) {
                //         for i in 0..ar.length as usize {
                //             let action = Self::get_action(context, *unit_id, ar, i)?;
                //             let action = action.clone();
                //             context.add_caster(context.entity(*unit_id)?);
                //             battle_actions.extend(action.process(context)?);
                //         }
                //     }
                // }
            }
            Ok(())
        })?;
        Ok(battle_actions)
    }
    pub fn paint(&self, rect: Rect, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        debug!("fusion fn");
        let entity = self.entity();
        let units = self.units(context)?;
        dbg!(&units);
        for unit in units {
            let unit_entity = unit.entity();
            let Ok(rep) = context.first_parent_recursive::<NUnitRepresentation>(unit.id) else {
                continue;
            };
            dbg!(&rep);
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

    fn get_action_range(&self, slot_idx: usize) -> (u8, u8) {
        // let current_action_ref = self.behavior.get(slot_idx);
        // let current_start = current_action_ref.map(|ar| ar.start).unwrap_or(0);
        // let current_len = current_action_ref.map(|ar| ar.length).unwrap_or(0);
        // (current_start, current_len)
        (0, 0)
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
                .map(|f| (f.index as usize, f)),
        );
        if ui.available_width() < 30.0 {
            return Ok(());
        }
        ui.columns(fusions.len(), |ui| {
            for i in (0..fusions.len()).rev() {
                let ui = &mut ui[i];
                let i = fusions.len() - i - 1;
                let fusion = fusions.get(&i).unwrap();
                // let resp = if fusion.units.ids.is_empty() {
                let resp = if true {
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

                if resp.dragged()
                //&& !fusion.units.ids.is_empty()
                {
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
