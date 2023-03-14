use super::*;

#[derive(Clone)]
pub struct ButtonComponent {
    pub icon: Option<Image>,
    action: fn(legion::Entity, &mut Resources, &mut legion::World),
}

impl ButtonComponent {
    pub fn new(action: fn(legion::Entity, &mut Resources, &mut legion::World)) -> Self {
        Self { action, icon: None }
    }

    pub fn click(
        &self,
        entity: legion::Entity,
        resources: &mut Resources,
        world: &mut legion::World,
    ) {
        (self.action)(entity, resources, world)
    }
}