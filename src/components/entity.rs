use super::*;

/// Used to get a link to actual Entity
#[derive(Debug)]
pub struct EntityComponent {
    pub entity: legion::Entity,
}

impl EntityComponent {
    pub fn new(entity: legion::Entity) -> Self {
        Self { entity }
    }
}
