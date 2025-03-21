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
        self.units.remove(u as usize);
        self.triggers
            .retain_mut(|(UnitTriggerRef { unit, trigger: _ }, _)| {
                if *unit == u {
                    return false;
                } else if *unit > u {
                    *unit -= 1;
                }
                true
            });
        for (_, actions) in self.triggers.iter_mut() {
            actions.retain_mut(
                |UnitActionRef {
                     unit,
                     trigger: _,
                     action: _,
                 }| {
                    if *unit == u {
                        return false;
                    } else if *unit > u {
                        *unit -= 1;
                    }
                    true
                },
            );
        }
    }
    pub fn remove_trigger(&mut self, r: UnitTriggerRef) {
        self.triggers.retain(|(t, _)| !r.eq(t));
    }
    pub fn remove_action(&mut self, r: UnitActionRef) {
        for (_, a) in self.triggers.iter_mut() {
            a.retain(|a| !r.eq(a));
        }
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
        unit: u8,
        trigger: u8,
        context: &'a Context,
    ) -> Result<&'a Trigger, ExpressionError> {
        let reaction = self.get_behavior(unit, context)?;
        Ok(&reaction.triggers[trigger as usize].trigger)
    }
    pub fn get_action<'a>(
        &self,
        r: &UnitActionRef,
        context: &'a Context,
    ) -> Result<(Entity, &'a Action), ExpressionError> {
        let reaction = self.get_behavior(r.unit, context)?;
        Ok((
            reaction.entity(),
            &reaction.triggers[r.trigger as usize].actions[r.action as usize],
        ))
    }
    pub fn react(
        &self,
        event: &Event,
        context: &mut Context,
    ) -> Result<Vec<BattleAction>, ExpressionError> {
        let mut battle_actions: Vec<BattleAction> = default();
        for (UnitTriggerRef { unit, trigger }, actions) in &self.triggers {
            if self
                .get_trigger(*unit, *trigger, context)?
                .fire(event, context)
            {
                for action in actions {
                    let (entity, action) = self.get_action(action, context)?;
                    let action = action.clone();
                    context.set_caster(entity);
                    battle_actions.extend(action.process(context)?);
                }
            }
        }
        Ok(battle_actions)
    }
    pub fn paint(&self, rect: Rect, ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        let entity = self.entity();
        let units = self.units_entities(&Context::new_world(world))?;
        for unit in units {
            let Some(rep) = world.get::<Representation>(unit) else {
                continue;
            };
            let context = Context::new_world(world)
                .set_owner(unit)
                .set_owner(entity)
                .take();
            RepresentationPlugin::paint_rect(rect, &context, &rep.material, ui)?;
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
}
