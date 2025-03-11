use super::*;

pub trait ActionsImpl {
    fn process(&self, context: &mut Context) -> Result<Vec<BattleAction>, ExpressionError>;
}

pub trait ActionImpl {
    fn process(&self, context: &mut Context) -> Result<Vec<BattleAction>, ExpressionError>;
}

impl ActionsImpl for Actions {
    fn process(&self, context: &mut Context) -> Result<Vec<BattleAction>, ExpressionError> {
        let mut actions: Vec<BattleAction> = default();
        for a in &self.0 {
            actions.extend(a.process(context)?);
        }
        Ok(actions)
    }
}

impl ActionImpl for Action {
    fn process(&self, context: &mut Context) -> Result<Vec<BattleAction>, ExpressionError> {
        info!(
            "{} {}",
            "action:".dimmed().purple(),
            self.cstr().to_colored()
        );
        let mut actions = Vec::default();
        match self {
            Action::Noop => {}
            Action::Debug(x) => {
                dbg!(x.get_value(context))?;
            }
            Action::SetValue(x) => {
                context.set_value(x.get_value(context)?);
            }
            Action::AddValue(x) => {
                context.set_value(context.get_value()?.add(&x.get_value(context)?)?);
            }
            Action::SubtractValue(x) => {
                context.set_value(context.get_value()?.sub(&x.get_value(context)?)?);
            }
            Action::AddTarget(x) => {
                for entity in x.get_entity_list(context)? {
                    context.add_target(entity);
                }
            }
            Action::DealDamage => {
                let owner = context.get_owner()?;
                let value = context.get_value()?.get_i32()?;
                if value > 0 {
                    for target in context.collect_targets()? {
                        actions.push(BattleAction::Damage(owner, target, value));
                    }
                }
            }
            Action::HealDamage => {
                let owner = context.get_owner()?;
                let value = context.get_value()?.get_i32()?;
                if value > 0 {
                    for target in context.collect_targets()? {
                        actions.push(BattleAction::Heal(owner, target, value));
                    }
                }
            }
            Action::UseAbility => {
                let caster = context.get_caster()?;
                if let Some(ability) = context.find_parent_component::<ActionAbility>(caster) {
                    let name = &ability.name;
                    let entity = ability.entity();
                    let ability_actions = context
                        .get_component::<AbilityEffect>(entity)
                        .to_e("AbilityEffect not found")?
                        .actions
                        .clone();
                    let color = context
                        .find_parent_component::<HouseColor>(entity)
                        .to_e("House not found")?
                        .color
                        .clone();
                    let text = format!("use ability [{color} [b {name}]]");
                    actions.push(BattleAction::Vfx(
                        HashMap::from_iter([
                            (VarName::text, text.into()),
                            (VarName::color, VISIBLE_LIGHT.into()),
                            (VarName::position, context.get_var(VarName::position)?),
                        ]),
                        "text".into(),
                    ));
                    actions.extend(ability_actions.process(context)?);
                } else if let Some(status) = context.find_parent_component::<StatusAbility>(caster)
                {
                    let name = &status.name;
                    let entity = status.entity();
                    let mut status = status.clone();
                    let mut description = context
                        .get_component::<StatusAbilityDescription>(entity)
                        .to_e("StatusDescription not found")?
                        .clone();
                    let reaction = context
                        .get_component::<Reaction>(entity)
                        .to_e("Reaction not found")?
                        .clone();
                    let color = context
                        .find_parent_component::<HouseColor>(entity)
                        .to_e("House not found")?
                        .color
                        .clone();
                    let text = format!("gain [{color} [b {name}]]");
                    actions.push(BattleAction::Vfx(
                        HashMap::from_iter([
                            (VarName::text, text.into()),
                            (VarName::color, VISIBLE_LIGHT.into()),
                            (VarName::position, context.get_var(VarName::position)?),
                        ]),
                        "text".into(),
                    ));
                    let representation = context.get_component::<Representation>(entity).cloned();
                    description.reaction = Some(reaction);
                    status.description = Some(description);
                    status.representation = representation;
                    actions.push(BattleAction::ApplyStatus(
                        context.get_owner()?,
                        status,
                        1,
                        color.c32(),
                    ));
                }
            }
            Action::Repeat(x, vec) => {
                for _ in 0..x.get_i32(context)? {
                    let context = &mut context.clone();
                    for a in vec {
                        actions.extend(a.process(context)?);
                    }
                }
            }
        };
        Ok(actions)
    }
}
