use super::*;

pub trait ActionsImpl {
    #[must_use]
    fn process(&self, ctx: &mut ClientContext) -> NodeResult<Vec<BattleAction>>;
}

pub trait ActionImpl {
    fn process(&self, ctx: &mut ClientContext) -> NodeResult<Vec<BattleAction>>;
}

impl ActionsImpl for Vec<Action> {
    fn process(&self, context: &mut ClientContext) -> NodeResult<Vec<BattleAction>> {
        let mut actions: Vec<BattleAction> = default();
        for a in self {
            actions.extend(a.process(context)?);
        }
        Ok(actions)
    }
}

impl ActionImpl for Action {
    fn process(&self, ctx: &mut ClientContext) -> NodeResult<Vec<BattleAction>> {
        info!(
            "{} {}",
            "action:".dimmed().purple(),
            self.cstr().to_colored()
        );
        let mut actions = Vec::default();
        match self {
            Action::noop => {}
            Action::debug(x) => {
                dbg!(x.get_value(ctx))?;
            }
            Action::set_value(x) => {
                let value = x.get_value(ctx)?;
                ctx.set_var_layer(VarName::value, value);
            }
            Action::add_value(x) => {
                let value = x.get_value(ctx)?;
                ctx.set_var_layer(
                    VarName::value,
                    ctx.get_var(VarName::value)
                        .unwrap_or(1.into())
                        .add(&value)?,
                );
            }
            Action::subtract_value(x) => {
                let value = x.get_value(ctx)?;
                ctx.set_var_layer(VarName::value, ctx.get_var(VarName::value)?.sub(&value)?);
            }
            Action::add_target(x) => match x.get_u64_list(ctx) {
                Ok(ids) => {
                    for id in ids {
                        ctx.add_target(id);
                    }
                }
                Err(e) => error!("add_target error: {e}"),
            },
            Action::deal_damage => {
                let owner = ctx
                    .owner()
                    .ok_or_else(|| NodeError::custom("No owner in context"))?;
                let value = ctx.get_var(VarName::value).get_i32()?;
                if value > 0 {
                    let targets = ctx.collect_targets();
                    if targets.is_empty() {
                        error!("No targets found for deal_damage");
                    } else {
                        for target in targets {
                            debug!(
                                "deal_damage: owner={}, target={}, value={}",
                                owner, target, value
                            );
                            actions.push(BattleAction::damage(owner, target, value));
                        }
                    }
                }
            }
            Action::heal_damage => {
                let owner = ctx
                    .owner()
                    .ok_or_else(|| NodeError::custom("No owner in context"))?;
                let value = ctx.get_var(VarName::value).get_i32()?;
                if value > 0 {
                    for target in ctx.collect_targets() {
                        actions.push(BattleAction::heal(owner, target, value));
                    }
                }
            }
            Action::use_ability => {
                let caster = ctx.caster().to_not_found()?;
                let house = ctx.load_first_parent_recursive_ref::<NHouse>(caster)?;
                let color = house.color_ref(ctx)?.color.c32();
                let value = ctx.get_var(VarName::value).get_i32().unwrap_or(1);
                if let Ok(ability) = house.ability_ref(ctx) {
                    let name = ability.ability_name.clone();
                    let effect = ability
                        .description_ref(ctx)?
                        .effect_ref(ctx)?
                        .actions
                        .clone();
                    ctx.with_layer(ContextLayer::Var(VarName::value, value.into()), |ctx| {
                        actions.extend(effect.process(ctx)?);
                        Ok(())
                    })?;
                    let text = format!("use ability [{} [b {name}] [th {value}]]", color.to_hex());
                    actions.push(
                        BattleAction::new_vfx("text")
                            .with_var(VarName::text, text)
                            .with_var(VarName::color, high_contrast_text())
                            .with_var(VarName::position, ctx.get_var(VarName::position)?)
                            .into(),
                    );
                } else {
                    return Err("Ability not found".into());
                }
            }
            Action::apply_status => {
                let caster = ctx.caster().to_not_found()?;
                let house = ctx.load_first_parent_recursive_ref::<NHouse>(caster)?;
                let color = house.color_ref(ctx)?.color.c32();
                let value = ctx.get_var(VarName::value).get_i32().unwrap_or(1);
                if let Ok(status) = house.status_ref(ctx) {
                    let name = status.status_name.clone();
                    let status = status
                        .clone()
                        .load_components(ctx)?
                        .take()
                        .with_state(NStatusState::new(next_id(), value));
                    let targets = ctx.collect_targets();
                    for target in targets {
                        actions.push(BattleAction::apply_status(
                            target,
                            status.clone().remap_ids(),
                            color,
                        ));
                    }
                    let text = format!("apply status [{} [b {name}] [th {value}]]", color.to_hex());
                    actions.push(
                        BattleAction::new_vfx("text")
                            .with_var(VarName::text, text)
                            .with_var(VarName::color, high_contrast_text())
                            .with_var(VarName::position, ctx.get_var(VarName::position)?)
                            .into(),
                    );
                } else {
                    return Err("Status not found".into());
                }
            }
            Action::repeat(x, vec) => {
                for _ in 0..x.get_i32(ctx)? {
                    for a in vec {
                        actions.extend(a.process(ctx)?);
                    }
                }
            }
        };
        Ok(actions)
    }
}
