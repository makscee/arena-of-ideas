use super::*;

impl NFusion {
    pub fn remove_unit(&mut self, id: u64) -> NodeResult<()> {
        if self.trigger_unit == id {
            self.trigger_unit = Default::default();
        }

        Ok(())
    }

    pub fn get_action_count(&self, ctx: &ClientContext) -> Result<usize, NodeError> {
        let slots = self.get_slots(ctx)?;
        Ok(slots.iter().map(|slot| slot.actions.length as usize).sum())
    }

    pub fn can_add_action(&self, ctx: &ClientContext) -> Result<bool, NodeError> {
        Ok(self.get_action_count(ctx)? < self.actions_limit as usize)
    }

    pub fn get_unit_tier(ctx: &ClientContext, unit_id: u64) -> Result<u8, NodeError> {
        if let Ok(unit) = ctx.load::<NUnit>(unit_id) {
            if let Ok(desc) = unit.description_ref(ctx) {
                if let Ok(behavior) = desc.behavior_ref(ctx) {
                    return Ok(behavior.reaction.tier());
                }
            }
        }
        Ok(0)
    }

    pub fn units<'a>(&self, ctx: &'a ClientContext) -> Result<Vec<&'a NUnit>, NodeError> {
        let slots = self.get_slots(ctx)?;
        let mut units = Vec::new();
        for slot in slots {
            if let Some(unit_id) = slot.unit.id() {
                if let Ok(unit) = ctx.load::<NUnit>(unit_id) {
                    units.push(unit);
                }
            }
        }
        Ok(units)
    }

    pub fn get_slots<'a>(&self, ctx: &'a ClientContext) -> Result<Vec<&'a NFusionSlot>, NodeError> {
        let mut slots = ctx.collect_children::<NFusionSlot>(self.id)?;
        slots.sort_by_key(|s| s.index);
        Ok(slots)
    }

    pub fn gather_fusion_actions<'a>(
        &self,
        ctx: &'a ClientContext,
    ) -> Result<Vec<(u64, &'a Action)>, NodeError> {
        let slots = self.get_slots(ctx)?;
        let mut all_actions = Vec::new();

        for slot in slots {
            if let Some(unit_id) = slot.unit.id() {
                if let Ok(unit) = ctx.load::<NUnit>(unit_id) {
                    if let Ok(desc) = unit.description_ref(ctx) {
                        if let Ok(unit_behavior) = desc.behavior_ref(ctx) {
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
            }
        }

        Ok(all_actions)
    }

    fn get_behavior<'a>(ctx: &'a ClientContext, unit: u64) -> Result<&'a NUnitBehavior, NodeError> {
        let unit = ctx.load::<NUnit>(unit).track()?;
        let desc = unit.description_ref(ctx).track()?;
        desc.behavior_ref(ctx)
    }

    pub fn get_trigger<'a>(ctx: &'a ClientContext, unit_id: u64) -> Result<&'a Trigger, NodeError> {
        Ok(&Self::get_behavior(ctx, unit_id).track()?.reaction.trigger)
    }

    pub fn react(
        &self,
        event: &Event,
        ctx: &ClientContext,
    ) -> Result<Vec<BattleAction>, NodeError> {
        let mut battle_actions: Vec<BattleAction> = default();
        ctx.with_temp_owner(self.id, |ctx| {
            if Self::get_trigger(ctx, self.trigger_unit)
                .track()?
                .fire(event, ctx)
                .unwrap_or_default()
            {
                let fusion_actions = self.gather_fusion_actions(ctx).track()?;
                let cloned_actions: Vec<(u64, Action)> = fusion_actions
                    .into_iter()
                    .map(|(unit_id, action)| (unit_id, action.clone()))
                    .collect();

                for (unit_id, action) in cloned_actions {
                    ctx.set_caster(unit_id);
                    battle_actions.extend(action.process(ctx).track()?);
                }
            }
            Ok(())
        })?;

        Ok(battle_actions)
    }

    pub fn paint(&self, rect: Rect, ctx: &mut ClientContext, ui: &mut Ui) -> NodeResult<()> {
        let units = self.units(ctx)?;
        for unit in units {
            if let Ok(desc) = unit.description_ref(ctx) {
                if let Ok(rep) = desc.representation_ref(ctx) {
                    ctx.with_temp_owner(unit.id, |ctx| {
                        let r = RepresentationPlugin::paint_rect(rect, &ctx, &rep.material, ui);
                        match &r {
                            Ok(_) => {}
                            Err(e) => {
                                dbg!(e);
                                ctx.debug_layers();
                                panic!();
                            }
                        }
                        r
                    })
                    .ui(ui);
                }
            }
        }
        for rep in ctx.collect_children::<NUnitRepresentation>(self.id)? {
            ctx.with_temp_owner(self.id, |ctx| {
                RepresentationPlugin::paint_rect(rect, ctx, &rep.material, ui)
            })
            .ui(ui);
        }
        Ok(())
    }
}
