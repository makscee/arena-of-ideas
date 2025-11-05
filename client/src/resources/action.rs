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
                let caster = ctx.caster().to_not_found().track()?;
                let house = ctx
                    .load_first_parent_recursive_ref::<NHouse>(caster)
                    .track()?;
                let x = ctx
                    .load::<NState>(caster)
                    .unwrap_or_default()
                    .stax
                    .at_most(house.state_ref(ctx).cloned().unwrap_or_default().stax)
                    .at_least(1);
                let color = house.color_ref(ctx)?.color.c32();
                if let Ok(ability) = house.ability_ref(ctx) {
                    let name = ability.ability_name.clone();
                    let effect = ability
                        .description_ref(ctx)
                        .track()?
                        .effect_ref(ctx)?
                        .actions
                        .clone();
                    ctx.with_layers(
                        [
                            ContextLayer::Var(VarName::stax, x.into()),
                            ContextLayer::Var(VarName::value, x.into()),
                        ],
                        |ctx| {
                            actions.extend(effect.process(ctx).track()?);
                            Ok(())
                        },
                    )?;
                    let text = format!("use [b x{x}] [{} {name}]", color.to_hex());
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
                let caster = ctx.caster().to_not_found().track()?;
                let house = ctx
                    .load_first_parent_recursive_ref::<NHouse>(caster)
                    .track()?;
                let x = ctx
                    .load::<NState>(caster)
                    .unwrap_or_default()
                    .stax
                    .at_most(house.state_ref(ctx).cloned().unwrap_or_default().stax)
                    .at_least(1);
                let color = house.color_ref(ctx)?.color.c32();
                if let Ok(status) = house.status_ref(ctx) {
                    let name = status.status_name.clone();
                    let status = status
                        .clone()
                        .load_components(ctx)?
                        .take()
                        .with_state(NState::new(next_id(), player_id(), x));
                    let targets = ctx.collect_targets();
                    for target in targets {
                        actions.push(BattleAction::apply_status(
                            target,
                            status.clone().remap_ids(),
                            color,
                        ));
                    }
                    let text = format!("apply [b x{x}] [{} {name}]", color.to_hex());
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
            Action::set_status(x) => {
                let status_id = x.get_u64(ctx)?;
                ctx.set_status(status_id);
            }
            Action::change_status_stax(x) => {
                let stack_change = x.get_i32(ctx)?;
                if let Some(status_id) = ctx.status() {
                    let mut status = ctx.load::<NStatusMagic>(status_id)?;
                    let current_stax = status.state_load(ctx)?.stax;
                    let name = status.name();
                    let color = ctx.color();
                    let new_stax = current_stax + stack_change;
                    actions.push(BattleAction::var_set(
                        status.state()?.id,
                        VarName::stax,
                        new_stax.into(),
                    ));

                    let stack_text = if stack_change > 0 {
                        format!("+{}", stack_change)
                    } else {
                        format!("{}", stack_change)
                    };
                    let text = format!("[b {}x] [{} {name}]", stack_text, color.to_hex());
                    actions.push(
                        BattleAction::new_vfx("text")
                            .with_var(VarName::text, text)
                            .with_var(VarName::color, color)
                            .with_var(VarName::position, ctx.get_var(VarName::position)?)
                            .into(),
                    );
                    actions.push(BattleAction::wait(ANIMATION));
                } else {
                    return Err(NodeError::custom(
                        "No status in context for change_status_stax",
                    ));
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
