use super::*;

use event::Event;

#[derive(Deserialize, Serialize, Clone, Debug)]
pub enum Trigger {
    AfterDamageTaken { effect: Effect },
}

impl Trigger {
    pub fn catch_event(
        &self,
        event: &Event,
        status_entity: Entity,
        world: &mut World,
    ) -> Result<()> {
        match self {
            Trigger::AfterDamageTaken { effect } => match event {
                Event::DamageTaken { unit, value } => {
                    let context = Context::from_owner(*unit)
                        .set_status(status_entity)
                        .set_var(VarName::Value, VarValue::Int(*value));
                    ActionPlugin::queue_effect(effect.clone(), context, world);
                }
            },
        }
        Ok(())
    }
}
