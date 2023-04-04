use super::*;

/// Used to get a link to actual Entity
#[derive(Debug)]
pub struct EntityComponent {
    pub entity: legion::Entity,
    pub ts: i64,
}

impl EntityComponent {
    pub fn new(entity: legion::Entity) -> Self {
        let ts = ts_nano();
        Self { entity, ts }
    }
}
