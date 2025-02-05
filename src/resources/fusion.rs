use super::*;

impl Fusion {
    pub fn init(self, world: &mut World) -> Result<(), ExpressionError> {
        let entity = self.entity();
        let units = self.units(&Context::new_world(world))?;
        let mut fusion_stats = UnitStats::default();
        for u in units {
            let stats = world.get::<UnitStats>(u).to_e("Unit stats not found")?;
            fusion_stats.hp += stats.hp;
            fusion_stats.dmg += stats.dmg;
            fusion_stats.pwr += stats.pwr;
        }
        world.entity_mut(entity).insert(fusion_stats);
        Ok(())
    }
    pub fn units(&self, context: &Context) -> Result<Vec<Entity>, ExpressionError> {
        let team = context
            .get_parent(self.entity())
            .to_e("Fusion parent not found")?;
        let fusion = &self.unit;
        let units = context
            .children_components_recursive::<Unit>(team)
            .into_iter()
            .filter_map(|(e, u)| {
                if fusion.units.contains(&u.name) {
                    Some((u.name.as_str(), e))
                } else {
                    None
                }
            })
            .sorted_by_key(|(name, _)| fusion.units.iter().find_position(|n| n.eq(name)).unwrap())
            .map(|(_, e)| e)
            .collect();
        Ok(units)
    }
    pub fn react(
        &self,
        event: &Event,
        context: &mut Context,
    ) -> Result<Vec<BattleAction>, ExpressionError> {
        let mut battle_actions: Vec<BattleAction> = default();
        let fusion = &self.unit;
        let units = self.units(context)?;
        let reactions = units
            .iter()
            .map(|e| context.get_component::<Reaction>(*e).unwrap().clone())
            .collect_vec();
        let reacted = fusion.triggers.iter().any(|i| {
            reactions[*i as usize]
                .react(event, context)
                .unwrap_or_default()
        });
        if !reacted {
            return Ok(default());
        }
        for (unit, action) in &fusion.actions {
            let unit = *unit as usize;
            let entity = units[unit];
            let action = reactions[unit].actions.0[*action as usize].as_ref();
            context.set_caster(entity);
            battle_actions.extend(action.process(context)?);
        }

        Ok(battle_actions)
    }
    pub fn paint(&self, rect: Rect, ui: &mut Ui, world: &World) -> Result<(), ExpressionError> {
        let entity = self.entity();
        let units = self.units(&Context::new_world(world))?;
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
