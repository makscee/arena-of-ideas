use super::*;

use event::Event;
use strum_macros::Display;

#[derive(Deserialize, Serialize, Clone, Debug, Default, Display)]
pub enum Trigger {
    AfterDamageTaken(EffectWrapped),
    BattleStart(EffectWrapped),
    #[default]
    Noop,
}

impl Trigger {
    pub fn catch_event(&self, event: &Event) -> Option<Trigger> {
        match self {
            Trigger::Noop => None,
            Trigger::AfterDamageTaken(..) => match event {
                Event::DamageTaken { .. } => Some(self.clone()),
                _ => None,
            },
            Trigger::BattleStart(..) => match event {
                Event::BattleStart => Some(self.clone()),
                _ => None,
            },
        }
    }

    pub fn fire(self, context: &Context, status: Entity, world: &mut World) {
        match self {
            Trigger::AfterDamageTaken(effect) | Trigger::BattleStart(effect) => {
                let context = context
                    .clone()
                    .set_owner(world.get::<Parent>(status).unwrap().get(), world)
                    .set_status(status, world);
                ActionPlugin::queue_effect(effect, context, world);
            }
            Trigger::Noop => panic!("Can't fire {self}"),
        }
    }
}
