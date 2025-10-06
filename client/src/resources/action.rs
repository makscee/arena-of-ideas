use super::*;

pub trait ActionsImpl {
    fn process(&self, context: &mut ClientContext) -> NodeResult<Vec<BattleAction>>;
}

pub trait ActionImpl {
    fn process(&self, context: &mut ClientContext) -> NodeResult<Vec<BattleAction>>;
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
                ctx.set_var(VarName::value, value);
            }
            Action::add_value(x) => {
                let value = x.get_value(ctx)?;
                ctx.set_var(
                    VarName::value,
                    ctx.get_var(VarName::value)
                        .unwrap_or(1.into())
                        .add(&value)?,
                );
            }
            Action::subtract_value(x) => {
                let value = x.get_value(ctx)?;
                ctx.set_var(VarName::value, ctx.get_var(VarName::value)?.sub(&value)?);
            }
            Action::add_target(x) => match x.get_ids_list(ctx) {
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
                    .ok_or_else(|| NodeError::Custom("No owner in context".into()))?;
                let value = ctx.get_i32(VarName::value)?;
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
                    .ok_or_else(|| NodeError::Custom("No owner in context".into()))?;
                let value = ctx.get_i32(VarName::value)?;
                if value > 0 {
                    for target in ctx.collect_targets() {
                        actions.push(BattleAction::heal(owner, target, value));
                    }
                }
            }
            Action::use_ability => {
                let caster = ctx.caster().to_not_found()?;
                let house = ctx.load_first_parent_recursive::<NHouse>(caster)?;
                let color = house.color_ref(ctx)?.color.c32();
                let value = ctx.get_i32(VarName::value).unwrap_or(1);
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
                    actions.push(BattleAction::vfx(
                        HashMap::from_iter([
                            (VarName::text, text.into()),
                            (VarName::color, high_contrast_text().into()),
                            (VarName::position, ctx.get_var(VarName::position)?),
                        ]),
                        "text".into(),
                    ));
                } else {
                    return Err("Ability not found".into());
                }
            }
            Action::apply_status => {
                let caster = ctx.caster().to_not_found()?;
                let house = ctx.load_first_parent_recursive::<NHouse>(caster)?;
                let color = house.color_ref(ctx)?.color.c32();
                let value = ctx.get_i32(VarName::value).unwrap_or(1);
                if let Ok(status) = house.status_ref(ctx) {
                    let name = status.status_name.clone();
                    let mut status = status.clone();
                    let mut description = status.description_load(ctx)?.clone();
                    let mut behavior = description.behavior_load(ctx)?.clone();
                    let representation =
                        status.representation_load(ctx).ok().cloned().map(|mut r| {
                            r.id = 0;
                            r
                        });
                    status.id = 0;
                    description.id = 0;
                    behavior.id = 0;
                    description.behavior.state_mut().set(behavior.clone());
                    status.description.state_mut().set(description.clone());
                    if let Some(repr) = representation {
                        status.representation.state_mut().set(repr);
                    }
                    let targets = ctx.collect_targets();
                    for target in targets {
                        actions.push(BattleAction::apply_status(
                            target,
                            status.clone(),
                            value,
                            color,
                        ));
                    }
                    let text = format!("apply status [{} [b {name}] [th {value}]]", color.to_hex());
                    actions.push(BattleAction::vfx(
                        HashMap::from_iter([
                            (VarName::text, text.into()),
                            (VarName::color, high_contrast_text().into()),
                            (VarName::position, ctx.get_var(VarName::position)?),
                        ]),
                        "text".into(),
                    ));
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
