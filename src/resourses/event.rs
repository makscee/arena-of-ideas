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
        let units = match &self {
            Event::DamageTaken { owner, value } | Event::IncomingDamage { owner, value } => {
                context.set_var(VarName::Value, VarValue::Int(*value));
                [*owner].into()
            }
            Event::BattleStart | Event::TurnStart | Event::TurnEnd | Event::Death(..) => {
                let mut units = UnitPlugin::collect_all(world);
                units.sort_by_key(|e| VarState::get(*e, world).get_int(VarName::Slot).unwrap());
                units
            }
            Event::BeforeStrike(unit) | Event::AfterStrike(unit) => [*unit].into(),
            Event::Kill { owner, target } => {
                context.set_target(*target, world);
                [*owner].into()
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
                [*owner].into()
            }
        };
        for unit in units {
            ActionPlugin::event_push_back(
                self.clone(),
                context.clone().set_owner(unit, world).take(),
                world,
            );
        }
        self
    }

    pub fn process(self, context: Context, world: &mut World) -> bool {
        let statuses = Status::collect_entity_statuses(context.owner(), world);
        let statuses = Status::filter_active_statuses(statuses, f32::MAX, world);
        Status::notify(statuses, &self, &context, world)
    }

    pub fn map(self, value: &mut VarValue, world: &mut World) -> Self {
        let (context, statuses) = match &self {
            Event::IncomingDamage { owner, .. } => (
                Context::new_named(self.to_string())
                    .set_owner(*owner, world)
                    .take(),
                Status::collect_entity_statuses(*owner, world),
            ),
            _ => panic!("Can't map {self}"),
        };
        let statuses =
            Status::filter_active_statuses(statuses, GameTimer::get().insert_head(), world);
        for status in statuses {
            Status::map_var(status, &self, value, &context, world);
        }
        self
    }

    pub fn spin(self, world: &mut World) {
        ActionPlugin::spin(world);
    }
}
