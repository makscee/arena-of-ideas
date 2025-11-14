use super::*;

impl NFusion {
    pub fn remove_unit(&mut self, id: u64) -> NodeResult<()> {
        if self.trigger_unit.id() == Some(id) {
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

    pub fn units<'a>(&self, ctx: &'a ClientContext) -> Result<Vec<&'a NUnit>, NodeError> {
        let slots = self.get_slots(ctx)?;
        let mut units = Vec::new();
        for slot in slots {
            if let Ok(unit) = slot.unit_ref(ctx) {
                units.push(unit);
            }
        }
        Ok(units)
    }

    pub fn stat_sum(&self, ctx: &ClientContext, var: VarName) -> NodeResult<i32> {
        let mut result = 0;
        let units = self.units(ctx)?;
        for unit in units {
            result += match var {
                VarName::hp => unit.stats_ref(ctx)?.hp,
                VarName::pwr => unit.stats_ref(ctx)?.pwr,
                _ => panic!(),
            };
        }
        Ok(result)
    }

    pub fn recalculate_stats(&mut self, ctx: &mut ClientContext) -> NodeResult<()> {
        self.pwr_set(self.stat_sum(ctx, VarName::pwr)?);
        self.hp_set(self.stat_sum(ctx, VarName::hp)?);
        Ok(())
    }

    pub fn get_slots<'a>(&self, ctx: &'a ClientContext) -> Result<Vec<&'a NFusionSlot>, NodeError> {
        let mut slots = ctx.load_children_ref::<NFusionSlot>(self.id)?;
        slots.sort_by_key(|s| s.index);
        Ok(slots)
    }

    pub fn gather_fusion_actions<'a>(
        &self,
        ctx: &'a ClientContext,
    ) -> Result<Vec<(u64, Action)>, NodeError> {
        let slots = self.get_slots(ctx)?;
        let mut all_actions = Vec::new();

        for slot in slots {
            if let Ok(unit) = slot.unit_ref(ctx) {
                if let Ok(desc) = unit.description_ref(ctx) {
                    if let Ok(unit_behavior) = desc.behavior_ref(ctx) {
                        let reaction = &unit_behavior.reaction;
                        let start = slot.actions.start as usize;
                        let end = (slot.actions.start + slot.actions.length) as usize;

                        for i in start..end.min(reaction.actions.len()) {
                            if let Some(action) = reaction.actions.get(i).cloned() {
                                all_actions.push((unit.id, action));
                            }
                        }
                    }
                }
            }
        }

        Ok(all_actions)
    }

    fn get_behavior<'a>(ctx: &'a ClientContext, unit: u64) -> Result<&'a NUnitBehavior, NodeError> {
        let unit = ctx.load_ref::<NUnit>(unit).track()?;
        let desc = unit.description_ref(ctx).track()?;
        desc.behavior_ref(ctx)
    }

    pub fn get_trigger<'a>(ctx: &'a ClientContext, unit_id: u64) -> Result<&'a Trigger, NodeError> {
        Ok(&Self::get_behavior(ctx, unit_id).track()?.reaction.trigger)
    }

    pub fn react_actions(
        &self,
        event: &Event,
        ctx: &mut ClientContext,
    ) -> Result<Vec<(u64, Action)>, NodeError> {
        if let Some(unit_id) = self.trigger_unit.id() {
            if Self::get_trigger(ctx, unit_id)
                .track()?
                .fire(event, ctx)
                .unwrap_or_default()
            {
                return self.gather_fusion_actions(ctx).track();
            }
        }
        Ok(default())
    }

    pub fn react(
        &self,
        event: &Event,
        ctx: &mut ClientContext,
    ) -> Result<Vec<BattleAction>, NodeError> {
        let actions = self.react_actions(event, ctx)?;
        if !actions.is_empty() {
            let mut battle_actions: Vec<BattleAction> = default();
            for (unit_id, action) in actions {
                ctx.set_caster(unit_id);
                battle_actions.extend(action.process(ctx).track()?);
            }
            Ok(battle_actions)
        } else {
            Ok(default())
        }
    }

    pub fn paint(&self, rect: Rect, ctx: &mut ClientContext, ui: &mut Ui) -> NodeResult<()> {
        let units = self.units(ctx)?;
        for unit in units {
            if let Ok(desc) = unit.description_ref(ctx) {
                if let Ok(rep) = desc.representation_ref(ctx) {
                    ctx.exec_ref(|ctx| {
                        ctx.with_owner(unit.id, |ctx| {
                            rep.material.paint(rect, ctx, ui);
                            Ok(())
                        })
                    })
                    .ui(ui);
                }
            }
        }
        for rep in ctx.load_children_ref::<NUnitRepresentation>(self.id)? {
            ctx.exec_ref(|ctx| {
                ctx.with_owner(self.id, |ctx| {
                    rep.material.paint(rect, ctx, ui);
                    Ok(())
                })
            })
            .ui(ui);
        }
        Ok(())
    }

    pub fn show_status_tags(
        &self,
        rect: Rect,
        ctx: &mut ClientContext,
        ui: &mut Ui,
    ) -> NodeResult<()> {
        let ui = &mut ui.new_child(
            UiBuilder::new()
                .max_rect(
                    Rect::from_center_size(rect.center_bottom(), egui::vec2(rect.width(), 0.0))
                        .translate(egui::vec2(0.0, 15.0)),
                )
                .layout(Layout::left_to_right(Align::Center).with_main_wrap(true)),
        );
        for status in ctx.load_children_ref::<NStatusMagic>(self.id)? {
            if !ctx
                .get_var_inherited(status.id, VarName::visible)
                .get_bool()?
            {
                continue;
            }
            let color = ctx
                .get_var_inherited(status.id, VarName::color)
                .get_color()?;
            let x = ctx.get_var_inherited(status.id, VarName::stax).get_i32()?;
            if x > 0 {
                TagWidget::new_name_value(status.name().to_string().cut_start(2), color, x)
                    .ui(ui)
                    .on_hover_ui(|ui| {
                        ctx.exec_ref(|ctx| {
                            ctx.with_owner(status.id, |ctx| {
                                status.render_card(ctx, ui);
                                Ok(())
                            })
                        })
                        .ui(ui);
                    });
            }
        }
        Ok(())
    }
}
