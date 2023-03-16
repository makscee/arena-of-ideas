use super::*;

pub struct InputSystem {
    press_start: Option<vec2<f32>>,
    is_dragging: bool,
}

impl InputSystem {
    pub fn new() -> Self {
        Self {
            press_start: default(),
            is_dragging: false,
        }
    }

    pub fn set_hovered_entity(
        entity: Option<legion::Entity>,
        resources: &mut Resources,
    ) -> Option<legion::Entity> {
        resources.input.prev_hovered = resources.input.cur_hovered;
        if resources.input.cur_hovered == entity
            || resources
                .input
                .pressed_mouse_buttons
                .contains(&geng::MouseButton::Left)
        {
            return resources.input.cur_hovered;
        }

        if let Some(prev_hovered) = resources.input.cur_hovered {
            resources
                .input
                .hover_data
                .insert(prev_hovered, (false, resources.global_time));
        }
        match entity {
            Some(entity) => {
                resources
                    .input
                    .hover_data
                    .insert(entity, (true, resources.global_time));
            }
            None => {}
        }

        resources.input.cur_hovered = entity;
        return resources.input.cur_hovered;
    }

    fn send_event(
        resources: &mut Resources,
        world: &mut legion::World,
        entity: legion::Entity,
        event: InputEvent,
    ) {
        resources
            .input
            .listeners
            .get(&entity)
            .cloned()
            .and_then(|f| {
                (f)(entity, resources, world, event);
                Some(())
            });
    }

    pub fn handle_events(&mut self, world: &mut legion::World, resources: &mut Resources) {
        if resources
            .input
            .down_mouse_buttons
            .contains(&geng::MouseButton::Left)
        {
            self.press_start = Some(resources.input.mouse_pos);
        }
        resources.input.cur_dragged = None;

        if let Some(hovered) = resources.input.cur_hovered {
            if resources
                .input
                .down_mouse_buttons
                .contains(&geng::MouseButton::Left)
            {
                Self::send_event(resources, world, hovered, InputEvent::PressStart);
            }
            if resources
                .input
                .pressed_mouse_buttons
                .contains(&geng::MouseButton::Left)
            {
                Self::send_event(resources, world, hovered, InputEvent::Press);
                if let Some(press_start) = self.press_start {
                    if !self.is_dragging && (press_start - resources.input.mouse_pos).len() > 0.01 {
                        self.is_dragging = true;
                        Self::send_event(resources, world, hovered, InputEvent::DragStart);
                    }
                }
                if self.is_dragging {
                    Self::send_event(resources, world, hovered, InputEvent::Drag);
                    resources.input.cur_dragged = Some(hovered);
                }
            }
            if resources
                .input
                .up_mouse_buttons
                .contains(&geng::MouseButton::Left)
            {
                Self::send_event(resources, world, hovered, InputEvent::PressStop);
                if !self.is_dragging {
                    Self::send_event(resources, world, hovered, InputEvent::Click);
                } else {
                    Self::send_event(resources, world, hovered, InputEvent::DragStop);
                }
            }
            if resources.input.cur_hovered != resources.input.prev_hovered {
                Self::send_event(resources, world, hovered, InputEvent::HoverStart);
            }
            Self::send_event(resources, world, hovered, InputEvent::Hover);
        }
        if resources.input.prev_hovered.is_some()
            && resources.input.prev_hovered != resources.input.cur_hovered
        {
            Self::send_event(
                resources,
                world,
                resources.input.prev_hovered.unwrap(),
                InputEvent::HoverStop,
            );
        }

        if resources
            .input
            .up_mouse_buttons
            .contains(&geng::MouseButton::Left)
        {
            self.press_start = None;
            self.is_dragging = false;
        }
    }
}

impl System for InputSystem {
    fn update(&mut self, world: &mut legion::World, resources: &mut Resources) {
        self.handle_events(world, resources);
    }
}
