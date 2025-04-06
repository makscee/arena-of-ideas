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
            Action::noop => {}
            Action::debug(x) => {
                dbg!(x.get_value(context))?;
            }
            Action::set_value(x) => {
                context.set_value(x.get_value(context)?);
            }
            Action::add_value(x) => {
                context.set_value(context.get_value()?.add(&x.get_value(context)?)?);
            }
            Action::subtract_value(x) => {
                context.set_value(context.get_value()?.sub(&x.get_value(context)?)?);
            }
            Action::add_target(x) => {
                for entity in x.get_entity_list(context)? {
                    context.add_target(entity);
                }
            }
            Action::deal_damage => {
                let owner = context.get_owner()?;
                let value = context.get_value()?.get_i32()?;
                if value > 0 {
                    for target in context.collect_targets()? {
                        actions.push(BattleAction::damage(owner, target, value));
                    }
                }
            }
            Action::heal_damage => {
                let owner = context.get_owner()?;
                let value = context.get_value()?.get_i32()?;
                if value > 0 {
                    for target in context.collect_targets()? {
                        actions.push(BattleAction::heal(owner, target, value));
                    }
                }
            }
            Action::use_ability => {
                let caster = context.get_caster()?;
                let ability = context
                    .find_parent_component::<AbilityMagic>(caster)
                    .to_e_fn(|| format!("Failed to find AbilityMagic of {caster}"))?;
                let name = &ability.ability_name;
                let entity = ability.entity();
                let ability_actions = context
                    .get_component::<AbilityEffect>(entity)
                    .to_e("AbilityEffect not found")?
                    .actions
                    .clone();
                let color = context.get_color(VarName::color)?;
                let text = format!("use ability [{} [b {name}]]", color.to_hex());
                actions.push(BattleAction::vfx(
                    HashMap::from_iter([
                        (VarName::text, text.into()),
                        (VarName::color, tokens_global().high_contrast_text().into()),
                        (VarName::position, context.get_var(VarName::position)?),
                    ]),
                    "text".into(),
                ));
                actions.extend(ability_actions.process(context)?);
            }
            Action::apply_status => {
                let caster = context.get_caster()?;
                let status = context
                    .find_parent_component::<StatusMagic>(caster)
                    .to_e_fn(|| format!("Failed to find StatusMagic of {caster}"))?;
                let name = &status.status_name;
                let entity = status.entity();
                let mut status = status.clone();
                let mut description = context
                    .get_component::<StatusDescription>(entity)
                    .to_e("StatusDescription not found")?
                    .clone();
                let behavior = context
                    .get_component::<Behavior>(entity)
                    .to_e("Behavior not found")?
                    .clone();
                let color = context.get_color(VarName::color)?;
                let text = format!("gain [{} [b {name}]]", color.to_hex());
                actions.push(BattleAction::vfx(
                    HashMap::from_iter([
                        (VarName::text, text.into()),
                        (VarName::color, tokens_global().high_contrast_text().into()),
                        (VarName::position, context.get_var(VarName::position)?),
                    ]),
                    "text".into(),
                ));
                let representation = context.get_component::<Representation>(entity).cloned();
                description.behavior = Some(behavior);
                status.description = Some(description);
                status.representation = representation;
                actions.push(BattleAction::apply_status(
                    context.get_owner()?,
                    status,
                    1,
                    color,
                ));
            }
            Action::repeat(x, vec) => {
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
