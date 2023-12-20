use super::*;

#[derive(Debug, Display, PartialEq, Eq, Serialize, Deserialize, Default, Clone)]
pub enum Event {
    IncomingDamage {
        owner: Entity,
        value: i32,
    },
    DamageTaken {
        owner: Entity,
        value: i32,
    },
    OutgoingDamage {
        owner: Entity,
        target: Entity,
        value: i32,
    },
    DamageDealt {
        owner: Entity,
        target: Entity,
        value: i32,
    },
    #[default]
    BattleStart,
    TurnStart,
    TurnEnd,
    BeforeStrike(Entity),
    AfterStrike(Entity),
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
            Event::DamageTaken { owner, value } | Event::IncomingDamage { owner, value } => {
                context.set_var(VarName::Value, VarValue::Int(*value));
                Status::collect_entity_statuses(*owner, world)
            }
            Event::BattleStart | Event::TurnStart | Event::TurnEnd | Event::Death(..) => {
                Status::collect_all_statuses(world)
            }
            Event::BeforeStrike(unit) | Event::AfterStrike(unit) => {
                Status::collect_entity_statuses(*unit, world)
            }
            Event::Kill { owner, target } => {
                context.set_target(*target, world);
                Status::collect_entity_statuses(*owner, world)
            }
            Event::OutgoingDamage {
                owner,
                target,
                value,
            }
            | Event::DamageDealt {
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
        let statuses = Status::filter_active_statuses(statuses, get_insert_head(world), world);
        Status::notify(statuses, &self, &context, world);
        self
    }

    pub fn map(self, value: &mut VarValue, world: &mut World) -> Self {
        let (var, context, statuses) = match &self {
            Event::IncomingDamage { owner, .. } => (
                VarName::IncomingDamage,
                Context::new_named(self.to_string())
                    .set_owner(*owner, world)
                    .take(),
                Status::collect_entity_statuses(*owner, world),
            ),
            _ => panic!("Can't map {self}"),
        };
        let statuses = Status::filter_active_statuses(statuses, get_insert_head(world), world);
        for status in statuses {
            Status::map_var(status, var, value, &context, world);
        }
        self
    }

    pub fn spin(self, world: &mut World) {
        ActionPlugin::spin(world);
    }
}
