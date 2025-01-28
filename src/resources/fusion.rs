use super::*;

impl Fusion {
    fn units(&self, context: &Context) -> Result<Vec<Entity>, ExpressionError> {
        let team = context.get_parent(self.entity())?;
        let fusion = &self.unit;
        let units = context
            .collect_children_components::<Unit>(team)
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
        let reacted = fusion
            .triggers
            .iter()
            .any(|i| reactions[*i as usize].react(event));
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
}
