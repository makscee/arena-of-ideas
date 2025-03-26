use super::*;

impl Fusion {
    pub fn init(entity: Entity, world: &mut World) -> Result<(), ExpressionError> {
        let fusion = world.get::<Fusion>(entity).to_e("Fusion not found")?;
        let units = fusion.units_entities(&Context::new_world(world))?;
        let mut fusion_stats = UnitStats::default();
        for u in units {
            let stats = world.get::<UnitStats>(u).to_e("Unit stats not found")?;
            fusion_stats.hp += stats.hp;
            fusion_stats.dmg += stats.dmg;
            fusion_stats.pwr += stats.pwr;
        }
        NodeState::from_world_mut(entity, world)
            .unwrap()
            .init_vars(fusion_stats.get_vars());
        world.entity_mut(entity).insert(fusion_stats);
        Ok(())
    }
    pub fn units_entities(&self, context: &Context) -> Result<Vec<Entity>, ExpressionError> {
        let mut units: Vec<Entity> = default();
        for unit in &self.units {
            units.push(context.entity_by_name(unit)?);
        }
        Ok(units)
    }
    pub fn get_unit(&self, unit: u8, context: &Context) -> Result<Entity, ExpressionError> {
        let unit = &self.units[unit as usize];
        context.entity_by_name(unit)
    }
    pub fn remove_unit(&mut self, u: u8) {
        // self.units.remove(u as usize);
        // self.triggers
        //     .retain_mut(|(UnitTriggerRef { unit, trigger: _ }, _)| {
        //         if *unit == u {
        //             return false;
        //         } else if *unit > u {
        //             *unit -= 1;
        //         }
        //         true
        //     });
        // for (_, actions) in self.triggers.iter_mut() {
        //     actions.retain_mut(
        //         |UnitActionRef {
        //              unit,
        //              trigger: _,
        //              action: _,
        //          }| {
        //             if *unit == u {
        //                 return false;
        //             } else if *unit > u {
        //                 *unit -= 1;
        //             }
        //             true
        //         },
        //     );
        // }
    }
    pub fn remove_trigger(&mut self, r: UnitTriggerRef) {
        // self.triggers.retain(|(t, _)| !r.eq(t));
    }
    pub fn remove_action(&mut self, r: UnitActionRef) {
        // for (_, a) in self.triggers.iter_mut() {
        //     a.retain(|a| !r.eq(a));
        // }
    }
    pub fn get_behavior<'a>(
        &self,
        unit: u8,
        context: &'a Context,
    ) -> Result<&'a Behavior, ExpressionError> {
        let unit = self.get_unit(unit, context)?;
        context
            .get_component::<Behavior>(unit)
            .to_e("Behavior not found")
    }
    pub fn get_trigger<'a>(
        &self,
        r: UnitTriggerRef,
        context: &'a Context,
    ) -> Result<&'a Trigger, ExpressionError> {
        let reaction = self.get_behavior(r.unit, context)?;
        Ok(&reaction.reactions[r.trigger as usize].trigger)
    }
    pub fn get_action<'a>(
        &self,
        r: &UnitActionRef,
        context: &'a Context,
    ) -> Result<(Entity, &'a Action), ExpressionError> {
        let behavior = self.get_behavior(r.unit, context)?;
        Ok((
            behavior.entity(),
            &behavior.reactions[r.trigger as usize].actions[r.action as usize],
        ))
    }
    pub fn react(
        &self,
        event: &Event,
        context: &mut Context,
    ) -> Result<Vec<BattleAction>, ExpressionError> {
        let mut battle_actions: Vec<BattleAction> = default();
        if self
            .get_trigger(self.trigger, context)?
            .fire(event, context)
        {
            for r in &self.actions {
                let (entity, action) = self.get_action(r, context)?;
                let action = action.clone();
                context.set_caster(entity);
                battle_actions.extend(action.process(context)?);
            }
        }
        Ok(battle_actions)
    }
    pub fn paint(&self, rect: Rect, context: &Context, ui: &mut Ui) -> Result<(), ExpressionError> {
        let entity = self.entity();
        let units = self.units_entities(context)?;
        for unit in units {
            let Some(rep) = context.get_component::<Representation>(unit) else {
                continue;
            };
            let context = context.clone().set_owner(unit).set_owner(entity).take();
            RepresentationPlugin::paint_rect(rect, &context, &rep.material, ui)?;
        }
        if let Some(rep) = context.get_component::<Representation>(entity) {
            RepresentationPlugin::paint_rect(
                rect,
                context.clone().set_owner(entity),
                &rep.material,
                ui,
            )?;
        }
        Ok(())
    }
    pub fn find_by_slot(slot: i32, world: &mut World) -> Option<Self> {
        world.query::<&Fusion>().iter(&world).find_map(|f| {
            if f.slot == slot {
                Some(f.clone())
            } else {
                None
            }
        })
    }
    pub fn show_editor(&mut self, context: &Context, ui: &mut Ui) -> Result<bool, ExpressionError> {
        let units = self.units_entities(context)?;
        let behaviors = (0..self.units.len())
            .filter_map(|u| {
                self.get_behavior(u as u8, context)
                    .ok()
                    .map(|b| (u as u8, b.clone()))
            })
            .collect_vec();
        let mut changed = false;
        ui.vertical(|ui| {
            for (u, b) in &behaviors {
                for (t, reaction) in b.reactions.iter().enumerate() {
                    let active = self.trigger.unit == *u && self.trigger.trigger == t as u8;
                    if reaction
                        .trigger
                        .cstr()
                        .as_button()
                        .active(active, ui)
                        .ui(ui)
                        .clicked()
                    {
                        self.trigger = UnitTriggerRef {
                            unit: *u,
                            trigger: t as u8,
                        };
                        changed = true;
                    }
                }
            }
        });
        space(ui);
        ui.vertical(|ui| {
            for (u, b) in &behaviors {
                for (t, (a, action)) in b
                    .reactions
                    .iter()
                    .enumerate()
                    .flat_map(|(i, r)| r.actions.0.iter().enumerate().map(move |a| (i, a)))
                {
                    let r = UnitActionRef {
                        unit: *u,
                        trigger: t as u8,
                        action: a as u8,
                    };
                    if self.actions.contains(&r) {
                        continue;
                    }
                    if action.cstr().as_button().ui(ui).clicked() {
                        self.actions.push(r);
                        changed = true;
                    }
                }
            }
            space(ui);
            for (i, r) in self.actions.clone().into_iter().enumerate() {
                let (_, action) = self.get_action(&r, context).unwrap();
                ui.horizontal(|ui| {
                    if i + 1 < self.actions.len() {
                        if "🔽".cstr().button(ui).clicked() {
                            self.actions.swap(i, i + 1);
                            changed = true;
                        }
                    }
                    if i > 0 {
                        if "🔼".cstr().button(ui).clicked() {
                            self.actions.swap(i, i - 1);
                            changed = true;
                        }
                    }
                    if action.cstr().as_button().active(true, ui).ui(ui).clicked() {
                        self.actions.remove(i);
                        changed = true;
                    }
                });
            }
        });
        Ok(changed)
    }
}
