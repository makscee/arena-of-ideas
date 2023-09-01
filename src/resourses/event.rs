use super::*;

#[derive(Debug)]
pub enum Event {
    DamageTaken { unit: Entity, value: i32 },
}

impl Event {
    pub fn send(self, world: &mut World) {
        match &self {
            Event::DamageTaken { unit, .. } => {
                let statuses = Status::collect_all_statuses(*unit, world);
                Status::notify(statuses, &self, world);
            }
        }
    }
}
