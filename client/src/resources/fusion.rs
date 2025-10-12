use super::*;

impl NFusion {
    pub fn remove_unit(&mut self, id: u64) -> NodeResult<()> {
        if self.trigger_unit == id {
            self.trigger_unit = Default::default();
        }

        Ok(())
    }

    pub fn get_action_count(&self, context: &ClientContext) -> Result<usize, NodeError> {
        let slots = self.get_slots(context)?;
        Ok(slots.iter().map(|slot| slot.actions.length as usize).sum())
    }

    pub fn can_add_action(&self, context: &ClientContext) -> Result<bool, NodeError> {
        Ok(self.get_action_count(context)? < self.actions_limit as usize)
    }

    pub fn get_unit_tier(context: &ClientContext, unit_id: u64) -> Result<u8, NodeError> {
        if let Ok(behavior) = context.load_first_parent_recursive::<NUnitBehavior>(unit_id) {
            Ok(behavior.reaction.tier())
        } else {
            Ok(0)
        }
    }

    pub fn units<'a>(&self, context: &'a ClientContext) -> Result<Vec<&'a NUnit>, NodeError> {
        let slots = self.get_slots(context)?;
        let mut units = Vec::new();
        for slot in slots {
            if let Ok(unit) = context.load_first_parent::<NUnit>(slot.id) {
                units.push(unit);
            }
        }
        Ok(units)
    }

    pub fn get_slots<'a>(
        &self,
        context: &'a ClientContext,
    ) -> Result<Vec<&'a NFusionSlot>, NodeError> {
        let mut slots = context.collect_children::<NFusionSlot>(self.id)?;
        slots.sort_by_key(|s| s.index);
        Ok(slots)
    }

    pub fn gather_fusion_actions<'a>(
        &self,
        context: &'a ClientContext,
    ) -> Result<Vec<(u64, &'a Action)>, NodeError> {
        let slots = self.get_slots(context)?;
        let mut all_actions = Vec::new();

        for slot in slots {
            if let Ok(unit) = context.load_first_parent::<NUnit>(slot.id) {
                if let Ok(unit_behavior) =
                    context.load_first_parent_recursive::<NUnitBehavior>(unit.id)
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
        context: &'a ClientContext,
        unit: u64,
    ) -> Result<&'a NUnitBehavior, NodeError> {
        context.load_first_parent_recursive::<NUnitBehavior>(unit)
    }

    pub fn get_trigger<'a>(
        context: &'a ClientContext,
        unit_id: u64,
    ) -> Result<&'a Trigger, NodeError> {
        Ok(&Self::get_behavior(context, unit_id)?.reaction.trigger)
    }

    pub fn react(
        &self,
        event: &Event,
        context: &ClientContext,
    ) -> Result<Vec<BattleAction>, NodeError> {
        let mut battle_actions: Vec<BattleAction> = default();
        context.with_temp_owner(self.id, |context| {
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
                    context.set_caster(unit_id);
                    battle_actions.extend(action.process(context)?);
                }
            }
            Ok(())
        })?;

        Ok(battle_actions)
    }

    pub fn paint(&self, rect: Rect, ctx: &mut ClientContext, ui: &mut Ui) -> NodeResult<()> {
        let units = self.units(ctx)?;
        for unit in units {
            let Ok(rep) = ctx.load_first_parent_recursive::<NUnitRepresentation>(unit.id) else {
                continue;
            };
            ctx.with_temp_owner(unit.id, |context| {
                RepresentationPlugin::paint_rect(rect, &context, &rep.material, ui)
            })
            .ui(ui);
        }
        for rep in ctx.collect_children::<NUnitRepresentation>(self.id)? {
            ctx.with_temp_owner(self.id, |context| {
                RepresentationPlugin::paint_rect(rect, context, &rep.material, ui)
            })
            .ui(ui);
        }
        Ok(())
    }
}
