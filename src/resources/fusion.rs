use super::*;

impl NFusion {
    pub fn remove_unit(&mut self, id: u64) {
        if self.trigger.unit == id {
            self.trigger = Default::default();
        }
        self.behavior.retain(|ar| ar.unit != id);
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
        ar: &UnitActionRef,
        index: usize,
    ) -> Result<&'a Action, ExpressionError> {
        Self::get_behavior(context, ar.unit)?
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
                for ar in &self.behavior {
                    for i in 0..ar.length as usize {
                        let action = Self::get_action(context, ar, i)?;
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
        ui.horizontal(|ui| {
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
                    for unit in &units {
                        let b = Self::get_behavior(context, unit.id)?;
                        for (ti, r) in b.reactions.iter().enumerate() {
                            if r.actions.is_empty() {
                                continue;
                            }
                            ui.label(format!("Unit {} Trigger {}", unit.id, ti));
                            ui.horizontal(|ui| {
                                for start in 0..r.actions.len() {
                                    for length in 1..=(r.actions.len() - start) {
                                        let ar = UnitActionRef {
                                            unit: unit.id,
                                            trigger: ti as u8,
                                            start: start as u8,
                                            length: length as u8,
                                        };

                                        // Check if this range overlaps with existing ones
                                        let overlaps = self.behavior.iter().any(|existing| {
                                            existing.unit == ar.unit
                                                && existing.trigger == ar.trigger
                                                && (existing.start < ar.start + ar.length
                                                    && ar.start < existing.start + existing.length)
                                        });

                                        if !overlaps {
                                            let range_text = if length == 1 {
                                                format!("Action {}", start)
                                            } else {
                                                format!("Actions {}-{}", start, start + length - 1)
                                            };

                                            if ui.button(range_text).clicked() {
                                                self.behavior.push(ar);
                                                changed = true;
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
                            format!("Unit {} Action {}", ar.unit, ar.start)
                        } else {
                            format!(
                                "Unit {} Actions {}-{}",
                                ar.unit,
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
    pub fn editor(
        &self,
        context: &Context,
        response: Response,
        on_add_unit: &mut impl FnMut(NFusion, u64),
        on_remove_unit: &mut impl FnMut(NFusion, u64),
    ) -> Result<Option<NFusion>, ExpressionError> {
        let mut edited: Option<NFusion> = None;
        let team = context.first_parent::<NTeam>(self.id)?;
        response
            .on_hover_ui(|ui| {
                self.show_card(context, ui).ui(ui);
            })
            .bar_menu(|ui| {
                let Ok(units) = self.units(context) else {
                    return;
                };
                ui.menu_button("add unit", |ui| {
                    let roster = team.roster_units_load(context);
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
        slot: impl Fn(&mut Ui, &Response, &NFusion),
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
        if ui.available_width() < 30.0 {
            return Ok(());
        }
        ui.columns(fusions.len(), |ui| {
            for i in (0..fusions.len()).rev() {
                let ui = &mut ui[i];
                let i = fusions.len() - i - 1;
                let fusion = fusions.get(&i).unwrap();
                let resp = slot_rect_button(ui.available_size(), ui, |rect, ui| {
                    if fusion.units.ids.is_empty() {
                        return;
                    }
                    fusion.paint(rect, context, ui).ui(ui);
                });

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

                match fusion.editor(context, resp.clone(), &mut on_add_unit, &mut on_remove_unit) {
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
                slot(ui, &resp, fusion);
            }
        });
        Ok(())
    }
}
