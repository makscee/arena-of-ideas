use super::*;

#[derive(Debug, Display)]
pub enum Event {
    DamageTaken { unit: Entity, value: i32 },
    BattleStart,
}

impl Event {
    pub fn send(self, world: &mut World) {
        let mut context = Context::new_named(self.to_string());
        let statuses = match &self {
            Event::DamageTaken { unit, value } => {
                context = context.set_var(VarName::Value, VarValue::Int(*value));
                Status::collect_entity_statuses(*unit, world)
            }
            Event::BattleStart => Status::collect_all_statuses(world),
            // _ => panic!("Event {self} can not be sent"),
        };
        Status::notify(statuses, &self, &context, world);
    }
}
