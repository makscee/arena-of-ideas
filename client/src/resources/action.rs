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
            Action::use_ability(house_name, ability_name, color) => {
                // TODO: Migrate to Rhai script execution system
                // This action type is being replaced by NUnitBehavior with RhaiScript
                log::warn!(
                    "Action::use_ability not yet migrated to Rhai scripts: {} / {}",
                    house_name,
                    ability_name
                );
            }
            Action::apply_status(house_name, status_name, color) => {
                // TODO: Migrate to Rhai script execution system
                // This action type is being replaced by NStatusBehavior with RhaiScript
                log::warn!(
                    "Action::apply_status not yet migrated to Rhai scripts: {} / {}",
                    house_name,
                    status_name
                );
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
