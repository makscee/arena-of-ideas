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
    List(Vec<Box<Trigger>>),
    #[default]
    Noop,
}

impl Trigger {
    pub fn catch_event(&self, event: &Event) -> Vec<Trigger> {
        match self {
            Trigger::Noop | Trigger::ChangeVar(..) => default(),
            Trigger::List(triggers) => triggers
                .into_iter()
                .map(|t| t.catch_event(event))
                .flatten()
                .collect_vec(),
            Trigger::AfterDamageTaken(..) => match event {
                Event::DamageTaken { .. } => vec![self.clone()],
                _ => default(),
            },
            Trigger::AfterDamageDealt(..) => match event {
                Event::DamageDealt { .. } => vec![self.clone()],
                _ => default(),
            },
            Trigger::BattleStart(..) => match event {
                Event::BattleStart => vec![self.clone()],
                _ => default(),
            },
            Trigger::TurnStart(..) => match event {
                Event::TurnStart => vec![self.clone()],
                _ => default(),
            },
            Trigger::BeforeStrike(..) => match event {
                Event::BeforeStrike(..) => vec![self.clone()],
                _ => default(),
            },
            Trigger::AllyDeath(..) => match event {
                Event::Death(..) => vec![self.clone()],
                _ => default(),
            },
            Trigger::BeforeDeath(..) => match event {
                Event::Death(..) => vec![self.clone()],
                _ => default(),
            },
            Trigger::AfterKill(..) => match event {
                Event::Kill { .. } => vec![self.clone()],
                _ => default(),
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

    pub fn collect_delta_triggers(&self) -> Vec<Trigger> {
        match self {
            Trigger::ChangeVar(_, _) => vec![self.clone()],
            Trigger::List(triggers) => triggers
                .into_iter()
                .map(|t| t.collect_delta_triggers())
                .flatten()
                .collect_vec(),
            _ => default(),
        }
    }
}
