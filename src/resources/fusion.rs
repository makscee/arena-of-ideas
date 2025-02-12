use super::*;

impl Fusion {
    pub fn init(self, world: &mut World) -> Result<(), ExpressionError> {
        let entity = self.entity();
        let units = self.units_entities(&Context::new_world(world))?;
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
    pub fn get_reaction<'a>(
        &self,
        unit: u8,
        context: &'a Context,
    ) -> Result<&'a Reaction, ExpressionError> {
        let unit = self.get_unit(unit, context)?;
        context
            .get_component::<Reaction>(unit)
            .to_e("Reaction not found")
    }
    pub fn get_trigger<'a>(
        &self,
        unit: u8,
        trigger: u8,
        context: &'a Context,
    ) -> Result<&'a Trigger, ExpressionError> {
        let reaction = self.get_reaction(unit, context)?;
        Ok(&reaction.triggers[trigger as usize].0)
    }
    pub fn get_action<'a>(
        &self,
        r: &UnitActionRef,
        context: &'a Context,
    ) -> Result<(Entity, &'a Action), ExpressionError> {
        let reaction = self.get_reaction(r.unit, context)?;
        Ok((
            reaction.entity(),
            &reaction.triggers[r.trigger as usize].1[r.action as usize],
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
                    battle_actions.extend(action.process(context.clone().set_caster(entity))?);
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
}
