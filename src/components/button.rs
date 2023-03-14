use super::*;

#[derive(Clone)]
pub struct ButtonComponent {
    pub icon: Option<Image>,
    action: fn(legion::Entity, &mut Resources, &mut legion::World, ButtonState),
}

impl ButtonComponent {
    pub fn new(
        action: fn(legion::Entity, &mut Resources, &mut legion::World, ButtonState),
    ) -> Self {
        Self { action, icon: None }
    }

    pub fn click(
        &self,
        entity: legion::Entity,
        resources: &mut Resources,
        world: &mut legion::World,
    ) {
        (self.action)(entity, resources, world, ButtonState::Clicked)
    }

    pub fn press(
        &self,
        entity: legion::Entity,
        resources: &mut Resources,
        world: &mut legion::World,
        ts: Time,
    ) {
        (self.action)(entity, resources, world, ButtonState::Pressed { ts })
    }
}

#[derive(Clone, Debug)]
pub enum ButtonState {
    Pressed { ts: Time },
    Clicked,
}
