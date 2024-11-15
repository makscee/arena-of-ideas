use super::*;

#[derive(Debug, PartialEq, Eq, Serialize, Deserialize, Default, Clone, AsRefStr)]
pub enum Event {
    #[default]
    BattleStart,
    TurnStart,
    TurnEnd,
    BeforeStrike(Entity, Entity),
    AfterStrike(Entity, Entity),
    Death(Entity),
    Kill {
        owner: Entity,
        target: Entity,
    },
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
    Summon(Entity),
    UseAbility(String),
    ApplyStatus(String),
}

impl Event {
    pub fn send_with_context(self, mut context: Context, world: &mut World) -> Self {
        if client_settings().dev_mode {
            debug!("{} {}", "Send event".dimmed(), self.cstr());
        }
        context.set_event(self.clone());
        ActionPlugin::register_event(self.clone(), world);
        let units = match &self {
            Event::DamageTaken { owner, value } | Event::IncomingDamage { owner, value } => {
                context.set_var(VarName::Value, VarValue::Int(*value));
                [*owner].into()
            }
            Event::BattleStart
            | Event::TurnStart
            | Event::TurnEnd
            | Event::Death(..)
            | Event::Summon(..)
            | Event::UseAbility(..)
            | Event::ApplyStatus(..) => {
                let mut units = UnitPlugin::collect_alive(world);
                units.sort_by_key(|e| {
                    Context::new(*e)
                        .get_int(VarName::Slot, world)
                        .unwrap_or_default()
                });
                match &self {
                    Event::Death(e) | Event::Summon(e) => {
                        context.set_target(*e);
                    }
                    _ => {}
                };
                units
            }
            Event::BeforeStrike(owner, target) | Event::AfterStrike(owner, target) => {
                context.set_target(*target);
                if !UnitPlugin::is_dead(*owner, world) {
                    [*owner].into()
                } else {
                    default()
                }
            }
            Event::Kill { owner, target } => {
                context.set_target(*target);
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
                    .set_target(*target)
                    .set_var(VarName::Value, VarValue::Int(*value));
                [*owner].into()
            }
        };
        for unit in units {
            ActionPlugin::event_push_back(
                self.clone(),
                context.clone().set_owner(unit).take(),
                world,
            );
        }
        self
    }

    pub fn send(self, world: &mut World) -> Self {
        self.send_with_context(Context::empty(), world)
    }
    pub fn map(self, value: &mut VarValue, world: &mut World) -> Self {
        let context = match &self {
            Event::IncomingDamage { owner, value: _ } => Context::new(*owner),
            _ => {
                return self;
            }
        };
        Status::map_var(&self, value, &context, world);
        self
    }

    pub fn process(self, context: Context, world: &mut World) -> bool {
        Status::notify(&self, &context, world)
    }
}

impl ToCstr for Event {
    fn cstr(&self) -> Cstr {
        let mut s = self.as_ref().cstr_c(CYAN);
        match self {
            Event::BattleStart | Event::TurnStart | Event::TurnEnd => {}
            Event::BeforeStrike(a, b) | Event::AfterStrike(a, b) => {
                s += &format!("({}, {})", entity_name_with_id(*a), entity_name_with_id(*b));
            }
            Event::Summon(a) | Event::Death(a) => {
                s += &format!("({})", entity_name_with_id(*a));
            }
            Event::Kill { owner, target } => {
                s += &format!(
                    "({} -> {})",
                    entity_name_with_id(*owner),
                    entity_name_with_id(*target)
                );
            }
            Event::IncomingDamage { owner, value } | Event::DamageTaken { owner, value } => {
                s += &format!("({} [vl {value}])", entity_name_with_id(*owner));
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
                s += &format!(
                    "({} -> {}: [vl {value}])",
                    entity_name_with_id(*owner),
                    entity_name_with_id(*target)
                );
            }
            Event::UseAbility(name) | Event::ApplyStatus(name) => {
                s += &name.cstr_c(name_color(name));
            }
        }
        s
    }
}
