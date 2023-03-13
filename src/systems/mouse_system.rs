use std::collections::VecDeque;

use super::*;

pub struct MouseSystem {}

impl MouseSystem {
    pub fn new() -> Self {
        Self {}
    }

    fn get_hovered_entity(world: &legion::World, resources: &Resources) -> Option<legion::Entity> {
        let mut entities = VecDeque::new();
        <(&AreaComponent, &EntityComponent, &InputComponent)>::query()
            .iter(world)
            .for_each(|(area, entity, input)| {
                if input.is_dragged() {
                    entities.push_front(entity.entity);
                } else if area.contains(resources.input.mouse_pos) && input.hovered.is_some() {
                    entities.push_back(entity.entity);
                }
            });
        match entities.is_empty() {
            true => None,
            false => Some(entities[0]),
        }
    }

    fn handle_hover(
        world: &mut legion::World,
        resources: &mut Resources,
        hovered: Option<legion::Entity>,
    ) {
        if let Some(hovered) = hovered {
            if let Some(old_hovered) = resources.input.hovered_entity {
                if old_hovered == hovered {
                    return;
                }
            }
            world
                .entry(hovered)
                .unwrap()
                .get_component_mut::<InputComponent>()
                .unwrap()
                .set_hovered(true, resources.global_time);
        }
        if resources.input.hovered_entity != hovered {
            resources.input.hovered_entity.and_then(|entity| {
                if let Some(mut prev) = world.entry(entity) {
                    prev.get_component_mut::<InputComponent>()
                        .unwrap()
                        .set_hovered(false, resources.global_time);
                }
                Some(())
            });

            resources.input.hovered_entity = hovered;
        }
    }

    fn handle_drag(
        world: &mut legion::World,
        resources: &mut Resources,
        hovered: Option<legion::Entity>,
    ) {
        if resources
            .input
            .down_mouse_buttons
            .contains(&geng::MouseButton::Left)
        {
            if let Some(dragged) = hovered {
                world
                    .entry(dragged)
                    .unwrap()
                    .get_component_mut::<InputComponent>()
                    .unwrap()
                    .set_dragged(true, resources.global_time);
                resources.input.dragged_entity = hovered;
            }
        }
        if resources.input.dragged_entity.is_some()
            && !resources
                .input
                .pressed_mouse_buttons
                .contains(&geng::MouseButton::Left)
        {
            let dragged = resources.input.dragged_entity.unwrap();
            world
                .entry(dragged)
                .unwrap()
                .get_component_mut::<InputComponent>()
                .unwrap()
                .set_dragged(false, resources.global_time);
            resources.input.dragged_entity = None;
        }
        if let Some(dragged) = resources.input.dragged_entity {
            world
                .entry(dragged)
                .unwrap()
                .get_component_mut::<AreaComponent>()
                .unwrap()
                .position = resources.input.mouse_pos;
        }
    }
}

impl System for MouseSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        let hovered = Self::get_hovered_entity(world, resources);
        Self::handle_hover(world, resources, hovered);
        Self::handle_drag(world, resources, hovered);
    }
}
