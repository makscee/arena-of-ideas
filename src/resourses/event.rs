use super::*;

#[derive(Debug, Display)]
pub enum Event {
    DamageTaken {
        owner: Entity,
        value: i32,
    },
    DamageDealt {
        owner: Entity,
        target: Entity,
        value: i32,
    },
    BattleStart,
    TurnStart,
    TurnEnd,
    BeforeStrike(Entity),
    Death(Entity),
    Kill {
        owner: Entity,
        target: Entity,
    },
}

impl Event {
    pub fn send(self, world: &mut World) -> Self {
        debug!("Send event {self:?}");
        let mut context = Context::new_named(self.to_string());
        let statuses = match &self {
            Event::DamageTaken { owner, value } => {
                context.set_var(VarName::Value, VarValue::Int(*value));
                Status::collect_entity_statuses(*owner, world)
            }
            Event::BattleStart | Event::TurnStart | Event::TurnEnd | Event::Death(..) => {
                Status::collect_all_statuses(world)
            }
            Event::BeforeStrike(unit) => Status::collect_entity_statuses(*unit, world),
            Event::Kill { owner, target } => {
                context.set_target(*target, world);
                Status::collect_entity_statuses(*owner, world)
            }
            Event::DamageDealt {
                owner,
                target,
                value,
            } => {
                context
                    .set_target(*target, world)
                    .set_var(VarName::Value, VarValue::Int(*value));
                Status::collect_entity_statuses(*owner, world)
            }
        };
        Status::notify(statuses, &self, &context, world);
        self
    }

    pub fn spin(self, world: &mut World) {
        ActionPlugin::spin(world);
    }
}
