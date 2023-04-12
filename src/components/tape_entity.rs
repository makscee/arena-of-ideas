/// All entities with this component will be drawn to node on lock
pub struct TapeEntityComponent {
    pub entity: legion::Entity,
}

impl TapeEntityComponent {
    pub fn new(entity: legion::Entity) -> Self {
        Self { entity }
    }
}
