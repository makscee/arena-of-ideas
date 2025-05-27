use super::*;

impl NFusion {
    pub fn remove_unit(&mut self, id: u64) {
        self.behavior.retain(|(t, _)| t.unit != id);
        for (_, ars) in &mut self.behavior {
            ars.retain(|ar| ar.unit != id);
        }
    }

    /// Returns the total number of actions currently in this fusion's behavior
    pub fn get_action_count(&self) -> usize {
        self.behavior.iter().map(|(_, actions)| actions.len()).sum()
    }

    /// Returns the maximum number of actions allowed based on fusion level
    /// Base limit is 5, with +2 actions per level
    pub fn get_action_limit(&self) -> usize {
        self.lvl as usize * 2
    }

    /// Returns true if more actions can be added to this fusion
    pub fn can_add_action(&self) -> bool {
        self.get_action_count() < self.get_action_limit()
    }

    /// Helper function to get a unit's tier through the context
    /// Tier is calculated from the unit's NBehavior based on action complexity
    pub fn get_unit_tier(context: &Context, unit_id: u64) -> Result<u8, ExpressionError> {
        if let Ok(behavior) = context.first_parent_recursive::<NBehavior>(unit_id) {
            Ok(behavior.tier())
        } else {
            Ok(0)
        }
    }
    pub fn units<'a>(&self, context: &'a Context) -> Result<Vec<&'a NUnit>, ExpressionError> {
        context.collect_parents_components::<NUnit>(self.id)
    }
    fn get_behavior<'a>(context: &'a Context, unit: u64) -> Result<&'a NBehavior, ExpressionError> {
        context.first_parent_recursive::<NBehavior>(unit)
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
        ar: &UnitActionRef,
    ) -> Result<&'a Action, ExpressionError> {
        Self::get_behavior(context, ar.unit)?
            .reactions
            .get(ar.trigger as usize)
            .to_e_not_found()?
            .actions
            .get(ar.action as usize)
            .to_e_not_found()
    }
    pub fn react(
        &self,
        event: &Event,
        context: &Context,
    ) -> Result<Vec<BattleAction>, ExpressionError> {
        let mut battle_actions: Vec<BattleAction> = default();
        context.with_layer_ref_r(ContextLayer::Owner(self.entity()), |context| {
            for (tr, actions) in &self.behavior {
                if Self::get_trigger(context, tr)?.fire(event, context) {
                    for ar in actions {
                        let action = Self::get_action(context, ar)?;
                        let action = action.clone();
                        context.add_caster(context.entity(ar.unit)?);
                        battle_actions.extend(action.process(context)?);
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
            let Ok(rep) = context.first_parent_recursive::<NRepresentation>(unit.id) else {
                continue;
            };
            context
                .with_owner_ref(unit_entity, |context| {
                    RepresentationPlugin::paint_rect(rect, &context, &rep.material, ui)
                })
                .ui(ui);
        }
        if let Ok(reps) = context.collect_children_components::<NRepresentation>(self.id) {
            for rep in reps {
                context
                    .with_owner_ref(entity, |context| {
                        RepresentationPlugin::paint_rect(rect, context, &rep.material, ui)
                    })
                    .ui(ui);
            }
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

        ui.vertical(|ui| -> Result<(), ExpressionError> {
            for unit in &units {
                let b = Self::get_behavior(context, unit.id)?;
                for (ti, r) in b.reactions.iter().enumerate() {
                    if self
                        .behavior
                        .iter()
                        .any(|(t, _)| t.unit == unit.id && t.trigger as usize == ti)
                    {
                        continue;
                    }
                    if r.trigger.cstr().button(ui).clicked() {
                        self.behavior.push((
                            UnitTriggerRef {
                                unit: unit.id,
                                trigger: ti as u8,
                            },
                            default(),
                        ));
                        changed = true;
                    }
                }
            }
            if self.behavior.is_empty() {
                return Ok(());
            }

            // Display action count and limit with level controls
            ui.horizontal(|ui| {
                ui.label(format!(
                    "Actions: {}/{}",
                    self.get_action_count(),
                    self.get_action_limit()
                ));
                ui.separator();
                ui.label("Level:");
                if ui.button("-").clicked() && self.lvl > 0 {
                    self.lvl -= 1;
                    changed = true;
                }
                ui.label(format!("{}", self.lvl));
                if ui.button("+").clicked() {
                    self.lvl += 1;
                    changed = true;
                }
            });

            if self.can_add_action() {
                for unit in &units {
                    let b = Self::get_behavior(context, unit.id)?;
                    for (ti, r) in b.reactions.iter().enumerate() {
                        for (ai, action) in r.actions.iter().enumerate() {
                            let ar = UnitActionRef {
                                unit: unit.id,
                                trigger: ti as u8,
                                action: ai as u8,
                            };
                            if self.behavior.iter().any(|(_, ars)| ars.contains(&ar)) {
                                continue;
                            }
                            context
                                .with_owner_ref(unit.entity(), |context| {
                                    if action
                                        .title_cstr(ViewContext::new(ui), context)
                                        .button(ui)
                                        .clicked()
                                    {
                                        self.behavior.get_mut(0).unwrap().1.push(ar);
                                        changed = true;
                                    }
                                    Ok(())
                                })
                                .ui(ui);
                        }
                    }
                }
            } else {
                ui.label("Action limit reached! Increase fusion level to add more actions.");
            }
            Ok(())
        })
        .inner?;
        space(ui);
        ui.vertical(|ui| -> Result<(), ExpressionError> {
            let mut new_behavior = None;
            for (ti, (tr, actions)) in self.behavior.iter().enumerate() {
                let trigger = Self::get_trigger(context, tr)?;
                if trigger.cstr().button(ui).clicked() {
                    let mut behavior = self.behavior.clone();
                    behavior.remove(ti);
                    new_behavior = Some(behavior);
                }
                for (ai, ar) in actions.iter().enumerate() {
                    let action = Self::get_action(context, ar)?;
                    ui.horizontal(|ui| -> Result<(), ExpressionError> {
                        if ti + 1 < self.behavior.len() || ai + 1 < actions.len() {
                            if "🔽".cstr().button(ui).clicked() {
                                let mut behavior = self.behavior.clone();
                                if ai == actions.len() - 1 {
                                    behavior[ti].1.remove(ai);
                                    behavior[ti + 1].1.insert(0, *ar);
                                } else {
                                    behavior[ti].1.swap(ai, ai + 1);
                                }
                                new_behavior = Some(behavior);
                            }
                        }
                        if ti > 0 || ai > 0 {
                            if "🔼".cstr().button(ui).clicked() {
                                let mut behavior = self.behavior.clone();
                                if ai > 0 {
                                    behavior[ti].1.swap(ai, ai - 1);
                                } else {
                                    behavior[ti].1.remove(ai);
                                    behavior[ti - 1].1.push(*ar);
                                }
                                new_behavior = Some(behavior);
                            }
                        }
                        context.with_owner_ref(context.entity(ar.unit)?, |context| {
                            if action
                                .title_cstr(ViewContext::new(ui), context)
                                .button(ui)
                                .clicked()
                            {
                                let mut behavior = self.behavior.clone();
                                behavior[ti].1.remove(ai);
                                new_behavior = Some(behavior);
                            }
                            Ok(())
                        })
                    });
                }
            }
            if let Some(mut new_behavior) = new_behavior {
                mem::swap(&mut self.behavior, &mut new_behavior);
                changed = true;
            }
            Ok(())
        })
        .inner?;
        Ok(changed)
    }
    pub fn editor(
        &self,
        context: &Context,
        response: Response,
        on_add_unit: &mut impl FnMut(NFusion, u64),
        on_remove_unit: &mut impl FnMut(NFusion, u64),
    ) -> Result<Option<NFusion>, ExpressionError> {
        let mut edited: Option<NFusion> = None;
        let team = context.first_parent_recursive::<NTeam>(self.id)?;
        let roster = team.roster_units_load(context);
        let units = self.units(context)?;
        response
            .on_hover_ui(|ui| {
                self.show_card(context, ui).ui(ui);
            })
            .bar_menu(|ui| {
                ui.menu_button("add unit", |ui| {
                    for unit in &roster {
                        if units.iter().any(|u| u.id == unit.id) {
                            continue;
                        }
                        context
                            .with_owner_ref(unit.entity(), |context| {
                                match unit.show_tag(context, ui) {
                                    Ok(response) => {
                                        if response.clicked() {
                                            on_add_unit(self.clone(), unit.id);
                                        }
                                        Ok(())
                                    }
                                    Err(e) => Err(e),
                                }
                            })
                            .ui(ui);
                    }
                });
                if !units.is_empty() {
                    ui.menu_button("remove unit", |ui| {
                        for unit in &units {
                            if unit.cstr().button(ui).clicked() {
                                on_remove_unit(self.clone(), unit.id);
                            }
                        }
                    });
                    ui.menu_button("edit", |ui| {
                        let mut fusion = self.clone();
                        let r = fusion.show_editor(context, ui);
                        if let Ok(c) = &r {
                            if *c {
                                edited = Some(fusion);
                            }
                        } else {
                            r.ui(ui);
                        }
                    });
                }
            });
        Ok(edited)
    }
    pub fn slots_editor(
        team: Entity,
        context: &Context,
        ui: &mut Ui,
        on_empty: impl FnOnce(&mut Ui),
        on_edited: impl FnOnce(NFusion),
        mut on_add_unit: impl FnMut(NFusion, u64),
        mut on_remove_unit: impl FnMut(NFusion, u64),
        mut on_reorder: impl FnMut(Vec<u64>),
    ) -> Result<(), ExpressionError> {
        let team = context.get::<NTeam>(team)?;
        let fusions: HashMap<usize, &NFusion> = HashMap::from_iter(
            team.fusions_load(context)
                .into_iter()
                .map(|f| (f.slot as usize, f)),
        );
        let team_slots = global_settings().team_slots as usize;
        ui.columns(team_slots, |ui| {
            for i in (0..team_slots).rev() {
                let ui = &mut ui[i];
                let i = team_slots - i - 1;
                if let Some(fusion) = fusions.get(&i) {
                    let response = slot_rect_button(ui, |rect, ui| {
                        fusion.paint(rect, context, ui).ui(ui);
                    });
                    if response.dragged() {
                        if let Some(pos) = ui.ctx().pointer_latest_pos() {
                            let origin = response.rect.center();
                            ui.painter().arrow(
                                origin,
                                pos - origin,
                                ui.visuals().widgets.hovered.fg_stroke,
                            );
                        }
                    }
                    response.dnd_set_drag_payload(i);
                    if let Some(j) = response.dnd_release_payload::<usize>() {
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
                    match fusion.editor(context, response, &mut on_add_unit, &mut on_remove_unit) {
                        Ok(edited) => {
                            if let Some(fusion) = edited {
                                on_edited(fusion);
                                return;
                            }
                        }
                        Err(e) => {
                            let ui = &mut ui.new_child(UiBuilder::new().max_rect(ui.min_rect()));
                            e.cstr().label_w(ui);
                        }
                    }
                } else {
                    on_empty(ui);
                    return;
                }
            }
        });
        Ok(())
    }
}
