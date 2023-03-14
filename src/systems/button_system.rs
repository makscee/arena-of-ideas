use super::*;

pub struct ButtonSystem {}

impl ButtonSystem {
    pub fn new() -> Self {
        Self {}
    }
}

impl System for ButtonSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        <(&ButtonComponent, &InputComponent, &EntityComponent)>::query()
            .iter(world)
            .find_map(
                |(button, input, entity)| match input.is_pressed(resources.global_time) {
                    true => Some((button.clone(), entity.entity)),
                    false => None,
                },
            )
            .and_then(|(button, entity)| {
                button.click(entity, resources, world);
                Some(())
            });
    }
}
