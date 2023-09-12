use super::*;
use event::Event;
use strum_macros::Display;

#[derive(Serialize, Deserialize, Clone, Debug, Default, Display)]
pub enum Effect {
    Damage(Option<Expression>),
    UseAbility(String),
    Debug(Expression),
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
}

impl EffectWrapped {
    pub fn process(self, mut context: Context, world: &mut World) -> Result<()> {
        debug!("Processing {}", &self.effect);
        if let Some(entity) = self.target {
            let entity = entity.get_entity(&context, world)?;
            context = context.set_target(entity, world);
        }
        match self.effect {
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
                debug!("Debug effect: {msg}",);
            }
            Effect::Noop => {}
            Effect::UseAbility(ability) => {
                let house = context
                    .get_var(VarName::House, world)
                    .context("House not found")?
                    .get_string()?;
                let effect = Pools::get_ability(&ability, &house, world).effect.clone();
                ActionPlugin::queue_effect(effect.wrap(), context, world);
            }
        }
        Ok(())
    }
}
