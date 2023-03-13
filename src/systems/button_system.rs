use super::*;

pub struct ButtonSystem {}

impl ButtonSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl System for ButtonSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        <(&ButtonComponent, &InputComponent)>::query()
            .iter(world)
            .find_map(
                |(button, input)| match input.is_pressed(resources.global_time) {
                    true => Some(button.clone()),
                    false => None,
                },
            )
            .and_then(|button| {
                button.click(resources, world);
                Some(())
            });
    }
}
