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
        println!(
            "{} {}",
            "action:".dimmed().purple(),
            self.title_recursive(ctx).to_colored(),
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
                    let position = ctx.owner_var(VarName::position)?;
                    for id in &ids {
                        add_target_vfx(ctx, &mut actions, &position, id)?;
                    }
                    ctx.add_targets(ids);
                }
                Err(e) => error!("add_target error: {e}"),
            },
            Action::set_target(x) => match x.get_u64_list(ctx) {
                Ok(ids) => {
                    let position = ctx.owner_var(VarName::position)?;
                    for id in &ids {
                        add_target_vfx(ctx, &mut actions, &position, id)?;
                    }
                    ctx.set_targets(ids);
                }
                Err(e) => error!("add_target error: {e}"),
            },
            Action::deal_damage => {
                let owner = ctx.owner()?;
                let value = ctx.get_var(VarName::value).get_i32()?;
                if value > 0 {
                    let targets = ctx.get_targets();
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
                let owner = ctx.owner()?;
                let value = ctx.get_var(VarName::value).get_i32()?;
                if value > 0 {
                    for target in ctx.get_targets() {
                        actions.push(BattleAction::heal(owner, target, value));
                    }
                }
            }
            Action::use_ability(ability_id) => {
                let owner = ctx.owner()?;
                let ability = ctx.load::<NAbilityMagic>(*ability_id)?;
                let x = ctx
                    .load::<NUnitState>(owner)
                    .unwrap_or_default()
                    .stax
                    .at_least(1);
                let color = ctx.color();
                let name = ability.ability_name.clone();
                let effect = ability.effect.load_node(ctx)?.actions.clone();
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
                let position = ctx.get_var(VarName::position).track()?;
                actions.push(BattleAction::new_text(text, position).into());
            }
            Action::apply_status(status_id) => {
                let owner = ctx.owner()?;
                let status = ctx.load::<NStatusMagic>(*status_id)?;
                let x = ctx
                    .load::<NUnitState>(owner)
                    .unwrap_or_default()
                    .stax
                    .at_least(1);
                let color = ctx.color();
                let name = status.status_name.clone();
                let status = status
                    .clone()
                    .load_components(ctx)?
                    .take()
                    .with_state(NState::new(next_id(), player_id(), x));
                let targets = ctx.get_targets();
                if targets.is_empty() {
                    return Err("No targets".into());
                }
                let text = format!("apply [b x{x}] [{} {name}]", color.to_hex());
                actions.push(
                    BattleAction::new_text(
                        text,
                        ctx.get_var(VarName::position).get_vec2().track()?,
                    )
                    .into(),
                );
                for target in targets {
                    actions.push(BattleAction::apply_status(
                        owner,
                        target,
                        status.clone().remap_ids(),
                        color,
                    ));
                    let position = ctx
                        .get_var_inherited(target, VarName::position)
                        .get_vec2()?;
                    actions.push(
                        BattleAction::new_text(
                            format!("gain [b x{x}] [{} {name}]", color.to_hex()),
                            position,
                        )
                        .into(),
                    );
                    actions.push(BattleAction::wait(animation_time()));
                }
            }
            Action::set_status(x) => {
                let status_id = x.get_u64(ctx)?;
                ctx.set_status(status_id);
            }
            Action::change_status_stax(x) => {
                let stack_change = x.get_i32(ctx)?;
                if let Some(status_id) = ctx.status() {
                    let mut status = ctx.load::<NStatusMagic>(status_id).track()?;
                    let current_stax = status.state.load_node_mut(ctx).track()?.stax;
                    let name = status.name();
                    let color = ctx.color();
                    let new_stax = current_stax + stack_change;
                    actions.push(BattleAction::var_set(
                        status.state.get()?.id,
                        VarName::stax,
                        new_stax.into(),
                    ));

                    let stack_text = if stack_change > 0 {
                        format!("+{}", stack_change)
                    } else {
                        format!("{}", stack_change)
                    };
                    let text = format!("[b {}x] [{} {name}]", stack_text, color.to_hex());
                    let position = ctx.get_var(VarName::position).track()?;
                    actions.push(BattleAction::new_text(text, position).into());
                    actions.push(BattleAction::wait(animation_time()));
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

fn add_target_vfx(
    ctx: &mut Context<Sources<'_>>,
    actions: &mut Vec<BattleAction>,
    position: &VarValue,
    id: &u64,
) -> Result<(), NodeError> {
    actions.push(
        BattleAction::new_vfx("target_add_vfx")
            .with_var(VarName::position, position.clone())
            .with_var(
                VarName::extra_position,
                ctx.get_var_inherited(*id, VarName::position)?,
            )
            .with_var(VarName::color, YELLOW)
            .into(),
    );
    actions.push(
        BattleAction::new_text("+ target", position.clone())
            .with_var(VarName::color, YELLOW)
            .into(),
    );
    actions.push(BattleAction::wait(animation_time()));
    Ok(())
}
