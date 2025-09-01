use super::*;

impl NFusion {
    pub fn remove_unit(&mut self, id: u64) -> Result<(), ExpressionError> {
        if self.trigger_unit == id {
            self.trigger_unit = Default::default();
        }

        Ok(())
    }

    pub fn get_action_count(&self, context: &Context) -> Result<usize, ExpressionError> {
        let slots = self.get_slots(context)?;
        Ok(slots.iter().map(|slot| slot.actions.length as usize).sum())
    }

    pub fn can_add_action(&self, context: &Context) -> Result<bool, ExpressionError> {
        Ok(self.get_action_count(context)? < self.actions_limit as usize)
    }

    pub fn get_unit_tier(context: &Context, unit_id: u64) -> Result<u8, ExpressionError> {
        if let Ok(behavior) = context.first_parent_recursive::<NUnitBehavior>(unit_id) {
            Ok(behavior.reaction.tier())
        } else {
            Ok(0)
        }
    }

    pub fn units<'a>(&self, context: &'a Context) -> Result<Vec<&'a NUnit>, ExpressionError> {
        let slots = self.get_slots(context)?;
        let mut units = Vec::new();
        for slot in slots {
            if let Ok(unit) = context.first_parent::<NUnit>(slot.id) {
                units.push(unit);
            }
        }
        Ok(units)
    }

    pub fn get_slots<'a>(
        &self,
        context: &'a Context,
    ) -> Result<Vec<&'a NFusionSlot>, ExpressionError> {
        let mut slots = context.collect_parents_components::<NFusionSlot>(self.id)?;
        slots.sort_by_key(|s| s.index);
        Ok(slots)
    }

    pub fn gather_fusion_actions<'a>(
        &self,
        context: &'a Context,
    ) -> Result<Vec<(u64, &'a Action)>, ExpressionError> {
        let slots = self.get_slots(context)?;
        let mut all_actions = Vec::new();

        for slot in slots {
            if let Ok(unit) = context.first_parent::<NUnit>(slot.id) {
                if let Ok(unit_behavior) = context.first_parent_recursive::<NUnitBehavior>(unit.id)
                {
                    let reaction = &unit_behavior.reaction;
                    let start = slot.actions.start as usize;
                    let end = (slot.actions.start + slot.actions.length) as usize;

                    for i in start..end.min(reaction.actions.len()) {
                        if let Some(action) = reaction.actions.get(i) {
                            all_actions.push((unit.id, action));
                        }
                    }
                }
            }
        }

        Ok(all_actions)
    }

    fn get_behavior<'a>(
        context: &'a Context,
        unit: u64,
    ) -> Result<&'a NUnitBehavior, ExpressionError> {
        context.first_parent_recursive::<NUnitBehavior>(unit)
    }

    pub fn get_trigger<'a>(
        context: &'a Context,
        unit_id: u64,
    ) -> Result<&'a Trigger, ExpressionError> {
        Ok(&Self::get_behavior(context, unit_id)?.reaction.trigger)
    }

    pub fn react(
        &self,
        event: &Event,
        context: &Context,
    ) -> Result<Vec<BattleAction>, ExpressionError> {
        let mut battle_actions: Vec<BattleAction> = default();

        context.with_layer_ref_r(ContextLayer::Owner(self.entity()), |context| {
            if Self::get_trigger(context, self.trigger_unit)?
                .fire(event, context)
                .unwrap_or_default()
            {
                let fusion_actions = self.gather_fusion_actions(context)?;
                let cloned_actions: Vec<(u64, Action)> = fusion_actions
                    .into_iter()
                    .map(|(unit_id, action)| (unit_id, action.clone()))
                    .collect();

                for (unit_id, action) in cloned_actions {
                    let unit_entity = context.entity(unit_id)?;
                    context.add_caster(unit_entity);
                    battle_actions.extend(action.process(context)?);
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
}
