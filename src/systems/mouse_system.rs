use std::collections::VecDeque;

use super::*;

pub struct MouseSystem {
    drag_start: Option<vec2<f32>>,
}

impl MouseSystem {
    pub fn new() -> Self {
        Self {
            drag_start: default(),
        }
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
        &mut self,
        world: &mut legion::World,
        resources: &mut Resources,
        hovered: Option<legion::Entity>,
    ) {
        if resources
            .input
            .down_mouse_buttons
            .contains(&geng::MouseButton::Left)
        {
            self.drag_start = Some(resources.input.mouse_pos);
            if let Some(dragged) = hovered {
                if world
                    .entry(dragged)
                    .unwrap()
                    .get_component_mut::<InputComponent>()
                    .unwrap()
                    .set_dragged(true, resources.global_time)
                {
                    resources.input.dragged_entity = hovered;
                }
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

    fn handle_press(
        &mut self,
        world: &mut legion::World,
        resources: &mut Resources,
        hovered: Option<legion::Entity>,
    ) {
        if hovered.is_none() {
            return;
        }
        if let Some(drag_start) = self.drag_start {
            if !resources
                .input
                .pressed_mouse_buttons
                .contains(&geng::MouseButton::Left)
            {
                self.drag_start = None;
                if let Some(mut entry) = world.entry(hovered.unwrap()) {
                    entry
                        .get_component_mut::<InputComponent>()
                        .unwrap()
                        .set_pressed(
                            drag_start == resources.input.mouse_pos,
                            resources.global_time,
                        );
                }
            }
        }
    }
}

impl System for MouseSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        let hovered = Self::get_hovered_entity(world, resources);
        Self::handle_hover(world, resources, hovered);
        self.handle_drag(world, resources, hovered);
        self.handle_press(world, resources, hovered);
    }
}
