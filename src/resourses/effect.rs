use super::*;
use event::Event;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub enum Effect {
    Damage { value: Option<Expression> },
    Debug(Expression),
}

impl Effect {
    pub fn process(self, context: Context, world: &mut World) -> Result<()> {
        match self {
            Effect::Damage { value } => {
                if let Some(value) = value {
                    let target = context.get_target().context("Target not found")?;
                    let value = value.get_int(&context, world)?;
                    debug!("Damage {value} {target:?}");
                    VarState::change_int(target, VarName::Hp, -value, world)?;
                    Event::DamageTaken {
                        unit: target,
                        value,
                    }
                    .send(world);
                }
            }
            Effect::Debug(msg) => {
                let msg = msg.get_string(&context, world)?;
                debug!("Debug effect: {msg}",);
            }
        }
        Ok(())
    }
}
