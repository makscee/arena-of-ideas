use super::*;

use event::Event;
use strum_macros::Display;

#[derive(Deserialize, Serialize, Clone, Debug, Default, Display)]
pub enum Trigger {
    AfterDamageTaken(Effect),
    AfterDamageDealt(Effect),
    BattleStart(Effect),
    TurnStart(Effect),
    BeforeStrike(Effect),
    AllyDeath(Effect),
    BeforeDeath(Effect),
    AfterKill(Effect),
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
            Trigger::AfterDamageDealt(..) => match event {
                Event::DamageDealt { .. } => Some(self.clone()),
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
            Trigger::BeforeStrike(..) => match event {
                Event::BeforeStrike(..) => Some(self.clone()),
                _ => None,
            },
            Trigger::AllyDeath(..) => match event {
                Event::Death(..) => Some(self.clone()),
                _ => None,
            },
            Trigger::BeforeDeath(..) => match event {
                Event::Death(..) => Some(self.clone()),
                _ => None,
            },
            Trigger::AfterKill(..) => match event {
                Event::Kill { .. } => Some(self.clone()),
                _ => None,
            },
        }
    }

    pub fn fire(self, event: &Event, context: &Context, status: Entity, world: &mut World) {
        let mut context = mem::take(
            context
                .clone()
                .set_owner(get_parent(status, world), world)
                .set_status(status, world),
        );
        match self {
            Trigger::AfterDamageTaken(effect)
            | Trigger::AfterDamageDealt(effect)
            | Trigger::BattleStart(effect)
            | Trigger::TurnStart(effect)
            | Trigger::BeforeStrike(effect) => {
                ActionPlugin::push_back(effect, context, world);
            }
            Trigger::AllyDeath(effect) => {
                let dead = match event {
                    Event::Death(unit) => *unit,
                    _ => panic!(),
                };
                let owner = get_parent(status, world);
                if UnitPlugin::get_faction(dead, world).eq(&UnitPlugin::get_faction(owner, world)) {
                    ActionPlugin::push_back(effect, context, world);
                }
            }
            Trigger::BeforeDeath(effect) => {
                let dead = match event {
                    Event::Death(unit) => *unit,
                    _ => panic!(),
                };
                let owner = get_parent(status, world);
                if dead.eq(&owner) {
                    ActionPlugin::push_back(effect, context, world);
                }
            }
            Trigger::AfterKill(effect) => {
                let target = match event {
                    Event::Kill { owner: _, target } => *target,
                    _ => panic!(),
                };
                context.set_target(target, world);
                ActionPlugin::push_back(effect, context, world);
            }
            _ => panic!("Trigger {self} can not be fired"),
        }
    }
}
