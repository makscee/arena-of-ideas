use super::*;

#[derive(Clone)]
pub struct ButtonComponent {
    action: fn(&mut Resources, &mut legion::World),
}

impl ButtonComponent {
    pub fn new(action: fn(&mut Resources, &mut legion::World)) -> Self {
        Self { action }
    }

    pub fn click(&self, resources: &mut Resources, world: &mut legion::World) {
        (self.action)(resources, world)
    }
}
