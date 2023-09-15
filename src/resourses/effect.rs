use std::ops::Deref;

use super::*;

#[derive(Serialize, Deserialize, Clone, Debug, Default, Display)]
pub enum Effect {
    Damage(Option<Expression>),
    UseAbility(String),
    AddStatus(String),
    Debug(Expression),
    List(Vec<Box<EffectWrapped>>),
    #[default]
    Noop,
}

impl Effect {
    pub fn wrap(self) -> EffectWrapped {
        EffectWrapped {
            effect: self,
            ..default()
        }
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Default)]
pub struct EffectWrapped {
    pub effect: Effect,
    pub owner: Option<Expression>,
    pub target: Option<Expression>,
    pub vars: Option<Vec<(VarName, VarValue)>>,
}

impl EffectWrapped {
    pub fn invoke(&self, context: &mut Context, world: &mut World) -> Result<()> {
        debug!("Processing {}\n{}", &self.effect, context);
        if let Some(entity) = &self.target {
            context.set_target(entity.get_entity(&context, world)?, world);
        }
        if let Some(vars) = &self.vars {
            for (var, value) in vars {
                context.set_var(*var, value.clone());
            }
        }
        match &self.effect {
            Effect::Damage(value) => {
                let target = context.get_target().context("Target not found")?;
                let value = match value {
                    Some(value) => value.get_int(&context, world)?,
                    None => context
                        .get_var(VarName::Atk, world)
                        .context("Can't find ATK")?
                        .get_int()?,
                };
                debug!("Damage {value} {target:?}");
                VarState::change_int(target, VarName::Hp, -value, world)?;
                Event::DamageTaken {
                    unit: target,
                    value,
                }
                .send(world);
            }
            Effect::Debug(msg) => {
                let msg = msg.get_string(&context, world)?;
                debug!("Debug effect: {msg}");
            }
            Effect::Noop => {}
            Effect::UseAbility(ability) => {
                let house = context
                    .get_var(VarName::House, world)
                    .context("House not found")?
                    .get_string()?;
                let effect = Pools::get_ability(&ability, &house, world).effect.clone();
                ActionPlugin::push_front(effect, context.clone(), world);
            }
            Effect::AddStatus(status) => {
                let house = context
                    .get_var(VarName::House, world)
                    .context("House not found")?
                    .get_string()?;
                let charges = context
                    .get_var(VarName::Charges, world)
                    .unwrap_or(VarValue::Int(1))
                    .get_int()?;
                Status::change_charges(&status, &house, context.target(), charges, world)?;
            }
            Effect::List(effects) => {
                for effect in effects {
                    ActionPlugin::push_front(effect.deref().clone(), context.clone(), world)
                }
            }
        }
        Ok(())
    }
}
