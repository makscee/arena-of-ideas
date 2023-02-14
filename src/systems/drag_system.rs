use super::*;

pub struct DragSystem {}

impl DragSystem {
    pub fn new() -> Self {
        Self {}
    }

    fn get_hovered_unit(world: &legion::World, resources: &Resources) -> Option<legion::Entity> {
        <(&PositionComponent, &EntityComponent, &UnitComponent)>::query()
            .iter(world)
            .find_map(|(position, entity, _)| {
                if (resources.mouse_pos - position.0).len() < UNIT_RADIUS {
                    Some(entity.entity)
                } else {
                    None
                }
            })
    }
}

impl System for DragSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if resources
            .down_mouse_buttons
            .contains(&geng::MouseButton::Left)
        {
            if let Some(dragged) = Self::get_hovered_unit(world, resources) {
                resources.dragged_entity = Some(dragged);
                world
                    .entry(dragged)
                    .unwrap()
                    .add_component(DragComponent {});
            }
        }
        if resources.dragged_entity.is_some()
            && !resources
                .pressed_mouse_buttons
                .contains(&geng::MouseButton::Left)
        {
            world
                .entry(resources.dragged_entity.unwrap())
                .unwrap()
                .remove_component::<DragComponent>();
            resources.dragged_entity = None;
        }
        if let Some(dragged) = resources.dragged_entity {
            world
                .entry(dragged)
                .unwrap()
                .get_component_mut::<PositionComponent>()
                .unwrap()
                .0 = resources.mouse_pos;
        }
    }
}
