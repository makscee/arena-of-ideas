use super::*;

use event::Event;
use strum_macros::Display;

#[derive(Deserialize, Serialize, Clone, Debug, Default, Display)]
pub enum Trigger {
    AfterDamageTaken(EffectWrapped),
    BattleStart(EffectWrapped),
    TurnStart(EffectWrapped),
    ChangeVar(VarName, Expression),
    #[default]
    Noop,
}

impl Trigger {
    pub fn catch_event(&self, event: &Event) -> Option<Trigger> {
        match self {
            Trigger::Noop | Trigger::ChangeVar(..) => None,
            Trigger::AfterDamageTaken(..) => match event {
                Event::DamageTaken { .. } => Some(self.clone()),
                _ => None,
            },
            Trigger::BattleStart(..) => match event {
                Event::BattleStart => Some(self.clone()),
                _ => None,
            },
            Trigger::TurnStart(..) => match event {
                Event::TurnStart => Some(self.clone()),
                _ => None,
            },
        }
    }

    pub fn fire(self, context: &Context, status: Entity, world: &mut World) {
        match self {
            Trigger::AfterDamageTaken(effect)
            | Trigger::BattleStart(effect)
            | Trigger::TurnStart(effect) => {
                let context = mem::take(
                    context
                        .clone()
                        .set_owner(world.get::<Parent>(status).unwrap().get(), world)
                        .set_status(status, world),
                );
                ActionPlugin::push_back(effect, context, world);
            }
            _ => panic!("Trigger {self} can not be fired"),
        }
    }
}
