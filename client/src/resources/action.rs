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
    fn process(&self, context: &mut ClientContext) -> NodeResult<Vec<BattleAction>> {
        info!(
            "{} {}",
            "action:".dimmed().purple(),
            self.cstr().to_colored()
        );
        let mut actions = Vec::default();
        match self {
            Action::noop => {}
            Action::debug(x) => {
                dbg!(x.get_value(context))?;
            }
            Action::set_value(x) => {
                let value = x.get_value(context)?;
                context.set_value_var(value);
            }
            Action::add_value(x) => {
                let value = x.get_value(context)?;
                context.set_value_var(context.get_value().unwrap_or(1.into()).add(&value)?);
            }
            Action::subtract_value(x) => {
                let value = x.get_value(context)?;
                context.set_value_var(context.get_value()?.sub(&value)?);
            }
            Action::add_target(x) => match x.get_entity_list(context) {
                Ok(entities) => {
                    for entity in entities {
                        context.add_target(entity);
                    }
                }
                Err(e) => error!("add_target error: {e}"),
            },
            Action::deal_damage => {
                let owner = context.owner_entity()?;
                let value = context.get_value()?.get_i32()?;
                if value > 0 {
                    let targets = context.collect_targets();
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
                let owner = context.owner_entity()?;
                let value = context.get_value()?.get_i32()?;
                if value > 0 {
                    for target in context.collect_targets() {
                        actions.push(BattleAction::heal(owner, target, value));
                    }
                }
            }
            Action::use_ability => {
                let caster = context.caster().to_not_found()?;
                let house = context.load_first_parent_recursive::<NHouse>(caster)?;
                let color = house.color_load(context)?.color.c32();
                let value = context.get_i32(VarName::value).unwrap_or(1);
                if let Ok(ability) = house.ability_load(context) {
                    let name = ability.ability_name.clone();
                    let effect = ability
                        .description_load(context)?
                        .effect_load(context)?
                        .actions
                        .clone();
                    context.with_layer_r(
                        ContextLayer::Var(VarName::value, value.into()),
                        |context| {
                            actions.extend(effect.process(context)?);
                            Ok(())
                        },
                    )?;
                    let text = format!("use ability [{} [b {name}] [th {value}]]", color.to_hex());
                    actions.push(BattleAction::vfx(
                        HashMap::from_iter([
                            (VarName::text, text.into()),
                            (VarName::color, high_contrast_text().into()),
                            (VarName::position, context.get_var(VarName::position)?),
                        ]),
                        "text".into(),
                    ));
                } else {
                    return Err("Ability not found".into());
                }
            }
            Action::apply_status => {
                let caster = context.caster().to_not_found()?;
                let house = context.load_first_parent_recursive::<NHouse>(caster)?;
                let color = house.color_ref(context)?.color.c32();
                let value = context.get_i32(VarName::value).unwrap_or(1);
                if let Ok(status) = house.status_ref(context) {
                    let name = status.status_name.clone();
                    let mut status = status.clone();
                    let mut description = status.description_load(context)?.clone();
                    let mut behavior = description.behavior_load(context)?.clone();
                    let representation =
                        status
                            .representation_load(context)
                            .ok()
                            .cloned()
                            .map(|mut r| {
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
                    let targets = context.collect_targets();
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
                            (VarName::position, context.get_var(VarName::position)?),
                        ]),
                        "text".into(),
                    ));
                } else {
                    return Err("Status not found".into());
                }
            }
            Action::repeat(x, vec) => {
                for _ in 0..x.get_i32(context)? {
                    for a in vec {
                        actions.extend(a.process(context)?);
                    }
                }
            }
        };
        Ok(actions)
    }
}
