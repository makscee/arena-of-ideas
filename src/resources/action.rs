use super::*;

pub trait ActionsImpl {
    fn process(&self, context: &mut Context) -> Result<Vec<BattleAction>, ExpressionError>;
}

pub trait ActionImpl {
    fn process(&self, context: &mut Context) -> Result<Vec<BattleAction>, ExpressionError>;
}

impl ActionsImpl for Vec<Action> {
    fn process(&self, context: &mut Context) -> Result<Vec<BattleAction>, ExpressionError> {
        let mut actions: Vec<BattleAction> = default();
        for a in self {
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
                let value = x.get_value(context)?;
                context.set_value_var(value);
            }
            Action::add_value(x) => {
                let value = x.get_value(context)?;
                context.set_value_var(context.get_value().unwrap_or_default().add(&value)?);
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
                    for target in context.collect_targets() {
                        actions.push(BattleAction::damage(owner, target, value));
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
                let caster = context.caster_entity()?.id(context)?;
                let ability = context.first_parent_recursive::<NAbilityMagic>(caster)?;
                let name = &ability.ability_name;
                let entity = ability.entity().id(context)?;
                let ability_actions = context
                    .first_parent_recursive::<NAbilityEffect>(entity)?
                    .actions
                    .clone();
                let color = context
                    .first_parent_recursive::<NHouseColor>(caster)?
                    .color
                    .c32();
                let text = format!("use ability [{} [b {name}]]", color.to_hex());
                actions.push(BattleAction::vfx(
                    HashMap::from_iter([
                        (VarName::text, text.into()),
                        (VarName::color, tokens_global().high_contrast_text().into()),
                        (VarName::position, context.get_var(VarName::position)?),
                    ]),
                    "text".into(),
                ));
                let lvl = context.get_i32(VarName::lvl)?;
                let value = context.get_i32(VarName::value).unwrap_or_default();
                context.with_layer_r(
                    ContextLayer::Var(VarName::value, (value + lvl).into()),
                    |context| {
                        actions.extend(ability_actions.process(context)?);
                        Ok(())
                    },
                )?;
            }
            Action::apply_status => {
                let caster = context.caster_entity()?.id(context)?;
                let targets = context.collect_targets();
                if targets.is_empty() {
                    return Err("No targets".into());
                }
                let status = context.first_parent_recursive::<NStatusMagic>(caster)?;
                let name = &status.status_name;
                let mut status = status.clone();
                let mut description = context
                    .first_parent_recursive::<NStatusDescription>(status.id)?
                    .clone();
                let mut behavior = context
                    .first_parent_recursive::<NStatusBehavior>(status.id)?
                    .clone();
                let color = context.get_color(VarName::color)?;
                let text = format!("apply [{} [b {name}]]", color.to_hex());
                actions.push(BattleAction::vfx(
                    HashMap::from_iter([
                        (VarName::text, text.into()),
                        (VarName::color, tokens_global().high_contrast_text().into()),
                        (VarName::position, context.get_var(VarName::position)?),
                    ]),
                    "text".into(),
                ));
                let representation = context
                    .first_child_recursive::<NStatusRepresentation>(status.id)
                    .ok()
                    .cloned()
                    .map(|mut r| {
                        r.id = 0;
                        r
                    });
                status.id = 0;
                description.id = 0;
                behavior.id = 0;
                description.behavior = Some(behavior);
                status.description = Some(description);
                status.representation = representation;
                let lvl = context.get_i32(VarName::lvl)?;
                let value = context.get_i32(VarName::value).unwrap_or_default();
                for target in targets {
                    actions.push(BattleAction::apply_status(
                        target,
                        status.clone(),
                        lvl + value,
                        color,
                    ));
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
