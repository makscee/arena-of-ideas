use super::*;

pub struct ButtonComponent {
    action: fn(&mut Resources, &mut legion::World),
    icon: Image,
    color: Rgba<f32>,
}

impl ButtonComponent {
    pub fn new(
        action: fn(&mut Resources, &mut legion::World),
        icon: Image,
        color: Rgba<f32>,
    ) -> Self {
        Self {
            action,
            icon,
            color,
        }
    }

    pub fn click(&self, resources: &mut Resources, world: &mut legion::World) {
        (self.action)(resources, world)
    }
}
